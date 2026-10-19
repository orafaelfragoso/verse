# 15. Targets, ABI and platform layers

This document specifies what a **target** is at the level a backend, the
runtime and the tier-0 VM must agree on: the data layout of every Verse type,
the calling convention of each target family, the unwinding and abort ABI, exit
statuses, the `verse.target` description file, the platform layers and their
capability mapping, and the boundary to foreign code. [04](backends-and-targets.md)
introduces tiers and targets and [05](runtime.md) the runtime; [13](mir-specification.md)
fixes the mono MIR that every backend consumes and [14](bytecode-and-vm.md)
the tier-0 VM. This document is the detail behind
[04 §4.3](backends-and-targets.md#43-what-a-target-is) and
[04 §4.5](backends-and-targets.md#45-calling-convention-and-abi).

Nothing here is implemented and no number is measured.

## 15.1 Purpose and scope

- The internal ABI ([04 §4.5](backends-and-targets.md#45-calling-convention-and-abi))
  is free to change between compiler versions. The only
  stable ABIs are the C ABI for `extern` calls and library exports, and the
  Wasm component ABI for component exports. Everything below is **internal**: it is
  versioned with the compiler, it may change in any release, and no artifact
  outside one toolchain build may rely on it.
- Internal does not mean unspecified. Two backends, the runtime
  and the VM must agree on layout and calls inside one build, so the rules are
  written down once and each backend is tested against them ([15.12](#1512-conformance-and-tests)).
- An ABI never changes what a program means (spec §4.1, §9.3).
  Sizes, alignments, field order, register use and the unwinding mechanism are
  not observable from Verse; the differential suite ([14 §14.12](bytecode-and-vm.md#1412-differential-testing-contract))
  compares behaviour only.
- Goals: **G1** one layout function, computed once on mono MIR, so that no
  backend invents a layout; **G2** one calling convention per target family
  that a stock debugger, profiler and unwinder understand; **G3** the same
  observable behaviour of panics, `Cleanup`, cancellation and exit statuses on
  every tier; **G4** a new target of a known family is data only (R2).
- Non-goals: a stable ABI for third parties, the plugin rules themselves
  ([07](plugins.md)), the instruction-level encoding of the VM
  ([14](bytecode-and-vm.md)), and the per-capability operation lists
  ([05 §5.6](runtime.md#56-the-sys-capability-interface)).

**The four ABI levels.**

| Level | What it fixes | Used between | Stability |
|-------|---------------|--------------|-----------|
| **A. Data layout** ([15.3](#153-data-layout)) | size, alignment, offsets, niches, header words | every tier, `verse-rt`, debuggers (via the debug map) | internal, `rev`-versioned with MIR ([13.9](mir-specification.md#139-versioning-policy)) |
| **B. Verse call convention** ([15.4](#154-the-verse-call-convention), [15.5](#155-per-target-conventions)) | how a mono MIR `call` is lowered | compiled Verse code to compiled Verse code | internal |
| **C. Runtime ABI** ([15.4](#154-the-verse-call-convention), [15.7](#157-unwinding-and-abort-abi)) | entry points of `verse-rt`, the panic record, the unwind state | compiled code and `verse-rt`, VM and `verse-rt` | internal; Rust `"C-unwind"` functions |
| **D. External ABIs** ([15.11](#1511-foreign-call-boundary)) | C ABI, Wasm component ABI, WASI | a Verse program and the outside world | stable |

## 15.2 Terms

| Term | Meaning |
|------|---------|
| **W** | pointer width in bytes of the target: 8 when `pointer_width = 64`, 4 when 32. Every size below that depends on it is written in W |
| **word** | W bytes, little endian |
| **Scalar kind** | `Int`, `Float`, `Bool`, `Char`, the sized numerics, `Float32`, and every pointer. A scalar is passed and returned as one value of its own width, even when that is wider than a word (`Int` on a 32-bit target) |
| **ZST** | a type of size 0 (`Nothing`, an empty tuple, a record with no fields, a closure with no captures) |
| **Niche** | a bit pattern of a field that no valid value of the field uses, usable to encode the cases of an enclosing enum ([15.3.4](#1534-enums-option-and-fallible-types)) |
| **TCB** | the *task control block* of `verse-rt`: per task state such as the unwind state, the cancel flag, the stack limit. Reached through the thread-local task pointer (stackful targets) or one global (single-threaded targets) |
| **Platform layer** | one of `std-os`, `std-wasi`, `std-web`, `std-embedded` ([05 §5.1](runtime.md#51-feature-slices)): the allocator, the entry point and the `sys-*` capabilities of a target |
| **Target file** | the TOML data of [15.9](#159-the-versetarget-description-file), a `verse.target` |
| **Verse-cc** | the internal call convention of [15.4](#154-the-verse-call-convention) |

## 15.3 Data layout

### 15.3.1 Rules

All layout is computed by one function in the compiler, `layout(type, target)`,
which the `layout` pass ([13.3](mir-specification.md#133-types-and-layout-facts))
runs on mono MIR and which prints the `layout` lines of the text format. The
backends, the VM and the debugger read those lines; none of them recomputes
anything. The function depends only on the type and on `pointer_width`, so two
targets of the same width produce the same layout.

| # | Rule |
|---|------|
| L1 | All targets in 0.1 are **little endian**. `endian = "big"` is reserved and rejected |
| L2 | `Int` and `Float` have size 8 and **alignment 8 on every target**, including 32-bit ones. No Verse type has alignment above 8, so the allocator only has to return 8-aligned blocks (16 on 64-bit targets, for convenience) |
| L3 | Every size is a multiple of its alignment. A type's alignment is the maximum of its fields' |
| L4 | **Record and tuple fields** are placed in order of descending alignment, ties in declaration order, with no padding between fields except what alignment needs. ZST fields take no space. A record's `size` is rounded up to its alignment. Field names and indexes in MIR never change (13.3) |
| L5 | An **enum case** is laid out as a record whose first field is the tag, pinned at offset 0 ([15.3.4](#1534-enums-option-and-fallible-types)), so small payloads pack after the tag |
| L6 | Layout is **not observable** from Verse: there is no `repr`, no `size_of`, no way to read bytes of a value except through `math` conversions and `Bytes` |
| L7 | The maximum size of a value is `2^(8W-1) - 1` bytes. A larger type is a compile error that names the type (assumption) |
| L8 | Drop flags are MIR locals (`dN`, [13.2](mir-specification.md#132-textual-format)); they are **never** stored inside a value. A value has no hidden word for ownership state |
| L9 | Any change to these rules, to a header word or to a niche value increments the MIR `rev` ([13.9](mir-specification.md#139-versioning-policy)) and regenerates golden files in the same commit |

One tuple has a fixed layout that the `layout` pass must not reorder, because
the VM and the debugger rely on it: the result of a `checked_*` operation,
`(T, Bool)`, is *value at offset 0, flag at offset 8* (the "next slot" of
[14.4](bytecode-and-vm.md#144-instruction-set-and-encoding)). The constraint
is stated in 14, 13.3 and 03 §3.4 pass 12.

**Declaration order is what users see.** L4 reorders fields, but the debugger
and `verse.api` always show the fields of a record and a tuple in declaration
order, so layout never leaks into what a user sees (L6).

### 15.3.2 Scalars

| Type | Size / align (64-bit) | Size / align (32-bit) | Representation | Niches |
|------|----------------------|----------------------|----------------|--------|
| `Int` | 8 / 8 | 8 / 8 | two's complement `i64` | none |
| `Float` | 8 / 8 | 8 / 8 | IEEE 754 binary64 | none (NaN payloads are observable through arithmetic, so no pattern is reserved; a NaN is canonicalised only where its bits are observed) |
| `Bool` | 1 / 1 | 1 / 1 | byte `0` or `1`; any other value is invalid | 254 values (`2..=255`) |
| `Char` | 4 / 4 | 4 / 4 | `u32`, a Unicode scalar value | all values above `0x10FFFF` (`0xFFFFFFFF - 0x10FFFF`); the surrogate gap is not used |
| `Nothing` | 0 / 1 | 0 / 1 | ZST | none |
| `Int8`, `Int16`, `Int32`, `Int64` and the `UInt` forms | natural / natural | natural / natural | two's complement or unsigned | none |
| `Float32` | 4 / 4 | 4 / 4 | IEEE 754 binary32 | none |
| `&T`, `&mut T` (MIR only) | W / W | W / W | address of the place; **never null** | 1 (zero); not stored in any type ([13.5](mir-specification.md#135-verifier-rules) V7) |
| `&dyn Trait` | 2W / W | 2W / W | `(data ptr, vtable ptr)` | 1 (data ptr zero) |

`Int` is 64-bit on 32-bit targets: it takes a register pair, and checked
arithmetic compiles to multi-word code ([14.5](bytecode-and-vm.md#145-checked-arithmetic-bounds-and-division)).
A narrow integer in a register is always kept **extended to the full
register width by its producer** according to its own signedness (`Bool` and
`Char` zero-extended); consumers may rely on it. It is the Apple
AArch64 and RISC-V rule, and Verse-cc adopts it on every target so the callee
never re-extends.

**Float bits.** Arithmetic is untouched. `to_bits`, hashing,
byte serialization and `Show` turn every NaN into `0x7FF8000000000000`. A hash of a
float, where one exists, treats `-0.0` as `0.0`, because they compare equal;
`Float` itself is not `Hash` (spec §9.4), so this binds only internal hashing and
a library wrapper such as `OrderedFloat`. `to_bits` keeps the sign of
zero; `Show` prints `NaN` for every NaN and `-0.0` for negative zero.

### 15.3.3 Runtime-owned types: `Text`, `Bytes`, `List`, `Map`, `Set`

These types are opaque to MIR (13.3): code outside `verse-rt` and the backend
intrinsics never reads their words, with **one exception**: the `len` rvalue
and element access of `List` ([13.4](mir-specification.md#134-statements-and-terminators), V12) are inline loads
at the fixed offsets below. The table gives the contract; bit assignments marked
*private* may change in any release.

| Type | Size / align (64-bit) | Size / align (32-bit) | Words | `needs_drop` | Niches |
|------|----------------------|----------------------|-------|--------------|--------|
| `Text` | 24 / 8 | 12 / 4 | see below | yes | the unused values of the form byte (232 on 64-bit) |
| `List<T>` | 24 / 8 | 12 / 4 | `ptr`, `len`, `cap` | yes (also when `T` has no glue: it frees the buffer) | `ptr` zero |
| `Bytes` | 24 / 8 | 12 / 4 | same as `List<UInt8>` | yes | `ptr` zero |
| `Map<K,V>` | 32 / 8 | 16 / 4 | `entries`, `len`, `cap`, `indices` | yes | `entries` zero |
| `Set<T>` | 32 / 8 | 16 / 4 | same as `Map<T, Nothing>` | yes | `entries` zero |

**`List<T>`.** `ptr` points to `cap` contiguous elements of stride
`size_of(T)` (already a multiple of its alignment); `len <= cap`; both are
unsigned words. An empty list that never allocated has `cap = 0` and `ptr` =
`align_of(T)` (non-null, never dereferenced, never freed), so creating `[]`
needs no heap and the pointer niche holds. For a ZST `T`, `cap` is
`2^(8W)-1` and nothing is allocated. The MIR `len` rvalue is `Int`: the stored
word is zero-extended. A bounds check compares the **unsigned** index with
`len` (`LT_U`, [14.5](bytecode-and-vm.md#145-checked-arithmetic-bounds-and-division)),
so a negative `Int` index fails the check on every width. On a 32-bit target
a list therefore holds at most `2^32 - 1` elements and a larger length is an
`out_of_memory` panic.

**`Text`** (assumption, as in 03 §3.5). A `Text` is
UTF-8, in one of two forms told apart by the **last byte** of the header (byte
`3W-1`, the *form byte*):

| Form byte | Form | Contents |
|-----------|------|----------|
| `0..=3W-2` (`0..=22` on 64-bit, `0..=10` on 32-bit) | **inline** | the byte length; the UTF-8 bytes are at offset 0, capacity `3W-2` (22 or 10 bytes); byte `3W-2` is the ASCII flag |
| `0xFF` | **heap or static** | three words: `ptr` to the first byte, byte length, and *private* words holding the capacity, an ASCII flag and the cached character count |

The remaining form-byte values are the niche. A heap form whose capacity is
`0` is a **static literal**: it points into read-only data, and `drop` never
frees it. That is how borrowed `Text` literals work without a heap
([05 §5.7](runtime.md#57-embedded-profile)). Building, growing and
indexing go through `verse-rt`; spec §4.1 states the cost of character indexing.
The inline capacity of 10 bytes on 32-bit targets is an assumption, revisited with measurements in M7.

**`Map` and `Set`** ([03 §3.5](ir-and-passes.md#35-layout-rules)). The
compact dictionary: `entries` points to a dense array of
`{ hash: usize, key: K, value: V }` in insertion order (for `Set`, no `value`);
`indices` points to an open-addressing table of 1-, 2-, 4- or W-byte entry
indexes whose width is chosen from `cap`; `len` is the live entry count.
`len` is at word 1, exactly as in `List`, so the MIR `len` rvalue is one load
for all three. How a removed entry is represented (tombstone or shifting), the
`used` count, the table's load factor and the hash seed are *private*. The
hash seed is process-wide and fixed in record and replay mode (14.10).

### 15.3.4 Enums, `Option` and fallible types

Every sum type is an enum to the layout function: user `enum`s, `T?` (case
`#0` none, `#1` present), and `A | B | E` (cases in normalized written order,
[13.3](mir-specification.md#133-types-and-layout-facts)). The
`layout` line records one of three **tag kinds** (13.2): `none`, `byte`,
`niche @OFFSET`.

1. **`tag none`**: a type with exactly one case (or no inhabited case but one)
   has no tag, and is laid out as a record.
2. **`tag niche @OFFSET`**: used when **exactly one case has data** and every
   other case is payload-free, and the data case contains a field with a niche
   (15.3.2). The niche is read at byte offset `OFFSET` as an unsigned integer
   of the niche field's width. Let `v0` be the first invalid value and
   `k = x - v0` (unsigned). If `k < ncases - 1` the value is the `k`-th
   payload-free case in declaration order; otherwise it is the data case.
   Several fields may have niches: the field with the **most** invalid values
   is used, ties go to the lowest offset. Niches compose: `Bool??` is one byte
   (`0`, `1` valid; `2` and `3` are the two nones).
3. **`tag byte`**: otherwise. The tag is an unsigned integer at **offset 0**
   of the smallest width in `u8`, `u16`, `u32` that holds `ncases`; case `i`
   has tag `i`. Each case is laid out as the record `{ tag, field... }` (L5), so
   `(p as #k).f` has a per-case offset (13.2, `(place as #k)`), and the size is
   the maximum over cases, rounded to the maximum alignment.

`discriminant p` is a load plus, for a niche, a compare and a select; the VM's
`GETDISC` and `GETDISCN` decode exactly this ([14.4](bytecode-and-vm.md#144-instruction-set-and-encoding)).
`set_discriminant` writes the same encoding and is only emitted by `layout`
and SROA (13.4).

**Recursive enums** are boxed by the compiler (spec §4.10 point 3): the
recursive field has the internal type `Box<T>`, one non-null word that owns a
heap allocation of `size_of(T)` bytes, freed by the drop glue. `Box<T>` is not
nameable from Verse.

**`Error`** in `A | Error` is the standard library's error type; its layout is defined by
the standard library, and this document assumes it is a boxed trait object `(data ptr, vtable
ptr)` with a non-null niche, so that `Nothing | Error` is 16 bytes with no tag
(assumption, until the standard library fixes `Error`).

### 15.3.5 Closures, function values and `dyn`

- **Closure value** (`aggregate closure path(operands)`, 13.2). Every closure
  has its own anonymous type, so the callee is known statically and a call is a
  direct `call` of the closure body's instance. The value is a **record of its
  captures** (L4), moved or copied in; a closure with no captures is a ZST. The
  body takes the record as its first parameter: `&Env` for a body that may be
  called repeatedly, `Env` owned for a body that moves a capture out (spec
  §5, captures). Borrowed captures cannot be stored (V7).
- **Non-escaping closure** (`aggregate closure_ref path(operands)`, 13.2). A
  lambda that only *borrows* its captures (spec §5.4) is a record of borrows,
  built in the caller's frame, and the value is a borrow of that record. Like
  any borrow (V7) it may be passed as an argument but is never stored,
  returned, sent to a task or captured by an escaping closure, so the borrows
  inside it cannot outlive the call. A callee sees it as a function value of
  unknown callee (below). Lambda lifting into extra borrowed parameters is not
  used, because it fails when the callee is not known, as in a generic
  `map(f)`.
- **Function value of unknown callee** (a parameter of function type that is
  not a known closure): the same representation as a trait object of a
  built-in call trait, `dyn Fn`, namely `(data ptr, vtable ptr)` with a vtable
  slot for the call. A function item with no captures uses a null `data` and a
  static vtable.
- **`dyn Trait`**: `(data ptr, vtable ptr)` (03 §3.5), 2W bytes, align W. For an
  *owned* trait object `data` points to a heap allocation of `vtable.size`
  bytes. A **vtable** is a static, read-only array of words
  `[drop_glue, size, align, slot_0, slot_1, ...]`; slots follow the trait's
  declaration order, each a pointer to a mono instance in Verse-cc whose
  first parameter is the receiver. `drop_glue` is null for a type with no glue.
  The only callee form on a vtable is `dyn #slot` (13.3).

### 15.3.6 `Cleanup` values and drop glue

A type with the `Cleanup` trait has **the same layout as it would without
it**: the trait adds no field. What it adds is the `cleanup` fact in its
`layout` line and a **drop glue function** with these rules:

- **Symbol and signature.** `drop<T>(p: ptr)`: one pointer to the value, no
  result, `nounwind` as seen by callers (MIR calls it with `unwind none` from
  `cleanup` blocks, V15). It runs `cleanup(mut self)` if the type has it, then
  drops the parts (fields and tuple elements in declaration
  order, list elements by index, map and set entries in insertion order; locals
  drop in reverse because a later local may borrow an earlier one, which the
  fields of one value cannot do), then frees the storage if the value owns heap
  memory.
- **`cleanup` is a catch point.** The glue calls the user's `cleanup` inside
  a runtime catch (15.7.5). A panic that escapes `cleanup` is recorded and does
  not leave the glue. This is what keeps cleanup blocks `nounwind` for the
  verifier (V15).
- **Blocking.** `cleanup` may block, and cancellation is
  **masked** while it runs; a cancel request that arrives then takes effect at
  the first blocking point after the cleanup finishes. The TCB carries a mask
  depth counter (`cancel_mask`), `drop` of a `Cleanup` type is a MIR terminator
  with edges ([13.4](mir-specification.md#134-statements-and-terminators)),
  and the glue of such a type is a *may-yield* function, which on
  state-machine targets uses the resumable convention ([15.6.2](#1562-resumable-functions-state-machine-and-cooperative-targets)).
  A task blocked inside a cleanup counts as blocked for deadlock detection
  (15.8.1); only `exit(code)` ignores it.
- **Platform handles.** `File`, `Socket`, `Process`, and so on are
  records in the standard library whose first field is a handle `Int`: a file descriptor on
  Unix, a `HANDLE` widened to 64 bits on Windows, a WASI `fd` widened, an index
  into the glue's handle table on `std-web`, a peripheral id on `std-embedded`.
  A handle has no meaning in a different process or after a replay without
  the recorded capability results ([14.10](bytecode-and-vm.md#1410-record-and-replay)).
- **Partial moves.** A field of a `Cleanup` type is never moved out (V4), so a
  glue never sees a partly initialised value.

### 15.3.7 Handles: tasks, channels, `Shared`, `Guarded`

All of these are **one non-null word** (W bytes, align W, one niche), pointing to a runtime-owned heap
cell. None is `Copy` except where noted. The cell layouts are *private* to
`verse-rt`; what compiled code relies on is the size, the niche and the
operations.

| Type | Word points to | Drop | Notes |
|------|----------------|------|-------|
| `Task<T>` | task cell: state, result slot for `T \| TaskPanicked \| Cancelled`, join list, panic record pointer | never dropped directly: consumed by `yield wait` or `task.abandon` (V17). Inside another value, that value's glue cancels it and then joins it, so the glue may yield | not `Copy`. In tier 0 the slot holds an opaque 64-bit value, not necessarily a pointer, and the debugger shows `t<N>` |
| `Sender<T>` | channel core: sender count, receiver flag, bounded buffer, wait queues | decrements the sender count; closes the channel at zero | `Channel.new<T>()` returns a pair `(Sender<T>, Receiver<T>)` of size 2W |
| `Receiver<T>` | the same channel core | closes the receiving side | not `Copy` |
| `Shared<T>` | allocation `{ strong: usize, weak: usize, value: T }` | strong count down; at zero run `T`'s glue, then weak count down and free | counts are **atomic** when the program can spawn tasks and plain otherwise (`refcount-opt`, 05 §5.2): two code shapes, **one layout** |
| `Weak<T>` | the same allocation | weak count down | |
| `Guarded<T>` | allocation `{ strong: usize, lock: usize, waiters: ptr, value: T }` | as `Shared` | the lock word and queue are used by `yield lock`; a guard value of `with g.lock() as mut value` is a `&mut T` (MIR only) plus a release on drop |

Reference counts are `usize`; an overflow aborts the program (a reference
cycle of 2^32 handles on a 32-bit target is a bug, not a panic). The cell of a
task, channel or `Shared` is allocated through the target's allocator. On a
`--no-heap` build a task or a `Shared` cannot be reached (05 §5.7), and a channel is
sized at build time: the `capacity` of every `Channel.new` must be an `Int`
literal (or the default 16), and the compiler places the channel core and its
buffer in static memory, one region per creation site. A capacity computed at
run time is a compile error (V0916). A creation site that runs again while the
channel it made is still alive panics with `out_of_memory` (R0007).

### 15.3.8 Worked layout examples (64-bit)

The `layout` lines are in the format of 13.2.

```mir
layout Int size 8 align 8 copy tag none
layout Point size 16 align 8 copy tag none          # record Point { x: Int, y: Int }: x@0 y@8
layout Mixed size 16 align 8 copy tag none          # { a: Bool, b: Int, c: Int8, d: Bool }: b@0 a@8 c@9 d@10, size 11 -> 16
layout Int? size 16 align 8 copy tag byte           # tag@0, payload@8
layout Bool? size 1 align 1 copy tag niche @0       # none = 2
layout Text size 24 align 8 needs_drop tag none
layout Text? size 24 align 8 needs_drop tag niche @23   # none = form byte 0xFE
layout List<Int>? size 24 align 8 needs_drop tag niche @0   # none = ptr 0
layout Cmd size 3 align 1 copy tag byte             # enum Cmd { Move(Int8, Int8), Quit }: tag@0 Move: a@1 b@2
layout Shape size 24 align 8 copy tag byte          # Circle(Float) tag@0 f@8; Rect(Float, Float) f@8 g@16; Empty
layout Tree size 24 align 8 needs_drop tag niche @8 # enum Tree { Leaf, Node(Int, Tree, Tree) }: Node n@0 left@8 right@16; Leaf = left 0
layout Nothing | Error size 16 align 8 needs_drop tag niche @0  # assumption of 15.3.4
```

On a 32-bit target the same declarations give: `Int?` 16 / 8 (unchanged, `Int`
is 8-aligned), `Text` 12 / 4, `Text?` 12 / 4 with the niche at byte 11, and
`Tree` 16 / 8 with `Node { n: Int @0, left @8, right @12 }` and `tag niche @8`
(`Box<Tree>` is 4 bytes, but the `Int` field forces alignment 8).

## 15.4 The Verse call convention

**Decision.** Verse-cc is **the platform's C calling convention applied to a
lowered signature that contains only scalars and pointers**. The lowering is
the same on every compiled target ([15.4.1](#1541-lowering-a-signature)); the
register assignment is then whatever the platform convention says for those
scalars ([15.5](#155-per-target-conventions)). Consequences: stock debuggers,
profilers and unwinders understand every frame; calls into and out of
`verse-rt` (a Rust library using `extern "C-unwind"`) need no adapter; and a
backend that can emit a C-ABI call can emit a Verse call. This refines
[04 §4.5](backends-and-targets.md#45-calling-convention-and-abi) ("small
values in registers, large values by hidden pointer, `mut` and borrowed
arguments by pointer") and is the "later ABI step" of
[13.6](mir-specification.md#136-generic-mir-and-mono-mir).

Tier 0 does not use Verse-cc; it has its own convention ([15.5.7](#1557-tier-0-the-vm)),
and the two never meet in 0.1. Mixed-mode execution (a later hot-function JIT)
needs adapters between the VM window and Verse-cc; they are specified with the
JIT.

### 15.4.1 Lowering a signature

Mono MIR gives each parameter a type and a mode word (`owned`, `borrowed`,
`exclusive`, [13.7](mir-specification.md#137-debug-info-and-source-spans)).
Each parameter and the result are first **classified by layout**:

| Class | Rule | Lowered as |
|-------|------|------------|
| **Zero** | ZST (size 0) | nothing: no argument and no result |
| **Scalar** | scalar kind (15.2): `Int`, `Float`, `Bool`, `Char`, sized numerics, pointers | one value of its own width; narrow integers extended to the register width by the caller (arguments) or callee (results) |
| **Pair** | any other type with `size <= 2W` | `ceil(size / W)` integer words, the little-endian memory image of the value cut in words; the last word is zero-extended and padding is undefined. A `Float` inside a pair travels in an integer register |
| **Memory** | any type with `size > 2W` | a pointer (rules below) |

The class is a function of the mono type and `pointer_width` only. Then, by
mode:

1. **`owned` parameter** (a plain value parameter, `take`, a captured
   environment moved in). Scalar and Pair classes are passed by value. A Memory
   class value is passed as a **pointer to a caller-owned temporary**. The
   caller gives the *contents* away: the callee drops them, and the caller
   neither reads nor drops the temporary afterwards (it still owns the storage and may
   reuse it). This is the MIR `move` of a call operand (V1, V4).
2. **`borrowed` parameter** (a shared borrow, `&T`). If `T` is `Copy` and its
   class is Scalar or Pair, it is passed **by value**: a shared borrow cannot be
   stored and its source cannot change while it is live (V7, V9), so the callee
   cannot tell. Otherwise it is a **pointer**. `&dyn Trait` is two pointers.
3. **`exclusive` parameter** (`mut`, `&mut T`). Always a pointer, even for a
   scalar.
4. **Result.** Zero: none. Scalar: returned as that scalar. Pair of one word:
   returned as an integer. Pair of two words: returned as the two-word integer
   pair of the platform (`rax:rdx`, `x0:x1`, `a0:a1`, `r0:r1`; a Wasm
   multi-value result). Memory: a **hidden first parameter**, a pointer to
   caller-allocated, uninitialised, suitably aligned space; the callee
   initialises it before returning normally; on an unwind edge it is
   left uninitialised and the caller does not drop it (V14).
5. **Order** is the MIR order: `_1`, `_2`, ... after the hidden result pointer.
   A Pair of two words occupies two consecutive argument positions.
6. **Hidden parameters.** None in 0.1 for a function that cannot yield. The
   task, the unwind state and the stack limit are reached through the TCB, not
   through a register. A may-yield function on a state-machine target is not
   lowered with Verse-cc at all ([15.6.2](#1562-resumable-functions-state-machine-and-cooperative-targets)).
7. **No tail calls.** The spec promises none (14.6); a backend may not turn a
   call into a jump, so a deep recursion reaches the stack check of
   [15.6.4](#1564-stack-bounds-and-overflow).
8. **Results of unwinding calls.** A call that can unwind has a normal return
   and, on targets that use native or Wasm exceptions, nothing else. On
   `status-code` targets the unwind state is a word in the TCB tested after
   the call ([15.7.2](#1572-the-three-mechanisms)); the result of a call that
   returned with the state set is undefined and ignored.

**Worked example.** `fn f(a: Int, b: Text, take c: Text, d: Float, e: Bool): Int?`
on a 64-bit target: `a` Scalar `i64`; `b` is a shared borrow of a non-`Copy`
type, a pointer; `c` is Memory (24 bytes), a pointer to a temporary the callee
drops; `d` Scalar `f64`; `e` Scalar `i8`, zero-extended; the result `Int?` is
16 bytes, a two-word Pair.

| Target | Lowered signature | Assignment |
|--------|------------------|------------|
| x86-64 SysV | `(i64, ptr, ptr, f64, i8) -> {i64, i64}` | `rdi, rsi, rdx, xmm0, ecx` -> `rax:rdx` |
| x86-64 Windows | same | `rcx, rdx, r8, xmm3, [rsp+40]` -> `rax:rdx` (argument positions are shared between integer and float registers; the fifth goes above the 32-byte shadow space) |
| AArch64 | same | `x0, x1, x2, d0, w3` -> `x0:x1` |
| wasm32 | `(sret: i32, i64, i32, i32, f64, i32) -> ()` | the result is 16 bytes, more than `2W = 8`, so it is Memory |

And `fn g(a: Int, b: Text): Int` on a 32-bit target: `a` is a Scalar that takes
a register pair, `b` a pointer, the result a Scalar pair: Thumb AAPCS
`r0:r1, r2` -> `r0:r1`; RISC-V `ilp32` `a0:a1, a2` -> `a0:a1`.

### 15.4.2 Calls into the runtime

Compiled code calls `verse-rt` through Rust `extern "C-unwind"` functions with
the **platform C convention directly**, no lowering step: scalars and
pointers only, results as a scalar, or through a pointer argument. The set is
the *runtime ABI* (level C); its members are named by the MIR `intrinsic`
operations (13.4, V23) and by the `sys` callees. Rules:

- **No Verse type crosses by value.** `Text`, `List` and the other headers are
  passed by pointer.
- **Intrinsics cannot unwind or yield** (V23). A runtime function that can fail
  (`rt_alloc` out of memory) returns a status, and the MIR already has the
  `panic out_of_memory` branch that tests it.
- **`rt_panic(kind, site, a, b)`** starts a panic ([15.7.1](#1571-the-panic-record-and-the-unwind-state)). On native and Wasm-EH targets it
  does not return; on `status-code` targets it returns with the unwind state
  set, and the generated code branches to the unwind edge.
- **Runtime-to-Verse calls** are limited to the task entry trampoline, the
  `poll` entry of a resumable function, the `drop_glue` words of vtables and
  `Shared` cells, and the program's `<script>` function. All use Verse-cc and
  are reached through `"C-unwind"` pointers (assumption).
- **Symbols** of compiled Verse functions are internal (hidden visibility),
  named from the instance key ([02 §2.6](compiler-architecture.md#26-monomorphization)) and a hash; they carry no promise.
  An `extern` function is called by its C name (spec §12.6).

## 15.5 Per-target conventions

Each family below names the platform convention, the registers it gives the
lowered signature, and what the stackful context switch must save. "Callee
saved" is as the platform defines it; Verse adds no register of its own (no
reserved register for a task pointer or an unwind state in 0.1).

| | x86-64 SysV | x86-64 Windows | AArch64 | ARM32 / Thumb | RISC-V 64 | RISC-V 32 | wasm32 |
|--|-------------|----------------|---------|---------------|-----------|-----------|--------|
| `abi` value in the target file | `sysv` | `win64` | `aapcs64` | `aapcs`, `aapcs-vfp` | `lp64d` | `ilp32` | `wasm-basic` |
| Integer arguments | `rdi rsi rdx rcx r8 r9` | `rcx rdx r8 r9` | `x0`-`x7` | `r0`-`r3`; 64-bit in even pairs | `a0`-`a7` | `a0`-`a7`; 64-bit in even pairs | wasm parameters |
| Float arguments | `xmm0`-`xmm7` | `xmm0`-`xmm3` (positional) | `v0`-`v7` | `s0`-`s15` / `d0`-`d7` on `aapcs-vfp` with a double FPU; else core registers | `fa0`-`fa7` | core registers (no FPU in `ilp32`) | `f32`, `f64` |
| Results | `rax rdx`; `xmm0` | `rax rdx`; `xmm0` | `x0 x1`; `v0` | `r0 r1`; `s0`/`d0` on VFP | `a0 a1`; `fa0` | `a0 a1` | multi-value |
| Hidden result pointer | first integer argument | first integer argument (`rcx`) | the first integer argument in Verse-cc (the platform's `x8` is used by C structs only, so it is not relied on) | first integer argument (`r0`) | first integer argument | first integer argument | first parameter |
| Callee saved | `rbx rbp r12`-`r15` | `rbx rbp rdi rsi r12`-`r15 xmm6`-`xmm15` | `x19`-`x28 x29 x30`; low 64 bits of `v8`-`v15` | `r4`-`r11` (`r9` is treated as callee saved), `s16`-`s31` on VFP | `s0`-`s11`, `fs0`-`fs11` | `s0`-`s11` | none (the engine) |
| Stack alignment at a call | 16 | 16, plus 32-byte shadow space | 16 | 8 | 16 | 16 | n/a |
| Not used by Verse code | red zone ignored in code that can run on a task stack | | `x18` (platform register) | | `gp`, `tp` | `gp`, `tp` | |
| Unwinding default (`unwind =`) | `native` | `native` | `native` | `status-code` (bare metal) | `native` (Linux) | `status-code` | `status-code` (see 15.7.2) |
| Stack check | prologue limit check (15.6.4) | same | same | same | same | same | shadow stack limit |

Two cautions on the table. **Positional registers on Windows:** a Pair of two
words uses two positions, so a call with more than four argument words puts
the rest on the stack in order. **Binary64 on 32-bit cores:**
`Float` is binary64; on `thumbv7em-none-eabihf` the FPU `fpv4-sp-d16` is single
precision only, so a `Float` operation is a runtime-library call and a `Float`
argument follows the *target ABI string* the toolchain compiles `verse-rt`
with, which both sides share (assumption, verified in M7).

### 15.5.1 x86-64 SysV (Linux, macOS, `-musl`)

- Verse-cc is the System V AMD64 convention over the lowered signature. A Pair
  is two integer arguments, so `(Float, Float)` uses `rdi`/`rsi`, not `xmm0`/`xmm1`.
- Frame pointers are kept in the `dev` and `test` profiles so that a profiler
  and the stack of a panic can walk frames without DWARF (assumption; the
  release profile may omit them).
- **Context switch** of a stackful task saves `rsp`, `rbx`, `rbp`, `r12`-`r15`,
  the x87 control word and `mxcsr`, in an assembly routine of the runtime
  (05 §5.3).
- `macOS` follows the SysV rules above. Its linker and unwinder differences
  (Mach-O, `__eh_frame` plus compact unwind) are the target file's
  `object_format`, not a calling-convention change.

### 15.5.2 x86-64 Windows

- Verse-cc is the Microsoft x64 convention over the lowered signature, with
  the 32-byte shadow space, and `rcx` for a hidden result pointer.
- **Stack probes.** A frame larger than one page calls the probe helper before
  touching new pages, as the platform requires.
- **Fibers.** Stackful tasks are implemented with fibers or an equivalent
  assembly switch (04 §4.2). The switch saves the callee-saved registers above
  (including `xmm6`-`xmm15`) **and updates the stack bounds in the thread
  environment block** (`StackBase`, `StackLimit`, `DeallocationStack`), so that
  the platform's probes and unwinder see the right stack.
- **Unwinding** uses the platform's table-based SEH: `.pdata` and `.xdata` unwind
  information with a Verse language-specific handler in `verse-rt`, and the
  raise is `RaiseException` with a Verse-specific customer exception code
  (15.7.2). Cranelift's unwind information on Windows (x64 and ARM64) is
  verified in M5; LLVM is the fallback.

### 15.5.3 AArch64 (Linux, macOS, Windows)

- Verse-cc is AAPCS64 over the lowered signature. `x18` is never allocated
  (the platform register of macOS and Windows; reserving it everywhere keeps
  one rule).
- The Apple and Windows variants differ from the base only in narrow-argument
  extension and variadics (neither is used by Verse-cc, which extends narrow
  integers itself, 15.3.2) and in unwinding (compact unwind and SEH
  respectively, set by `object_format`).
- **Context switch** saves `x19`-`x30`, `sp`, and `d8`-`d15`.
- Pointer authentication (`arm64e`) is not a 0.1 target.

### 15.5.4 ARM32 and Thumb

- Verse-cc is AAPCS (`thumbv6m`/`thumbv7m`, soft float) or AAPCS-VFP
  (`thumbv7em-none-eabihf`) over the lowered signature. A 64-bit scalar (`Int`,
  `Float`) takes an even-numbered register pair and 8-byte stack alignment, and
  the stack is 8-aligned at a call.
- Thumb-only cores execute Thumb-2 or Thumb-1 code; interworking veneers are the
  linker's. Calls between Verse functions are direct `bl`.
- **No stackful tasks.** Cortex-M targets use the cooperative executor
  ([15.6.2](#1562-resumable-functions-state-machine-and-cooperative-targets)).
- **Atomics by triple.** `thumbv7m-none-eabi` (Cortex-M3) and
  `thumbv7em-none-eabi`, `thumbv7em-none-eabihf` (Cortex-M4, M7) are ARMv7-M
  and ARMv7E-M. They have `LDREX`/`STREX`, so compare-and-swap is native and
  their target files set `atomics = "native"`. `thumbv6m-none-eabi`
  (Cortex-M0, M0+) is ARMv6-M. It has no `LDREX`/`STREX` and no
  compare-and-swap, only single loads and stores, so its target file sets
  `atomics = "none"`.
- **`Int` cost.** Every `Int` operation is a register pair operation;
  range analysis lowers what it can prove to 32 bits (05 §5.7).
- ARM32 Linux (`armv7-unknown-linux-gnueabihf`) is **not** in
  [04 §4.2](backends-and-targets.md#42-target-matrix). The convention above
  would apply with `unwind = "native"` (EHABI tables) and `tasks = "stackful"`;
  adding it is a new target file and a context-switch routine. i686, RISC-V 32
  Linux and big-endian targets are also outside 0.1 on the same terms.

### 15.5.5 RISC-V

- **RV64** (`riscv64gc-unknown-linux-gnu`) uses the `lp64d` convention. `gp` and
  `tp` are never allocated.
- **RV32** (`riscv32imc-unknown-none-elf`, for example ESP32-C3) uses `ilp32`
  (soft float, no hardware atomics, no `Float` registers): `Int` and `Float`
  are even register pairs. `atomics = "none"`, which is valid only because the
  target is single core with interrupts masked around runtime critical
  sections.
- **Context switch** (RV64 only) saves `ra`, `sp`, `s0`-`s11` and `fs0`-`fs11`.
- Cranelift covers RV64 and not RV32 (04 §4.1); RV32 is LLVM only.

### 15.5.6 wasm32: WASI and the browser

- **Value types.** `Int` is `i64`, `Float` is `f64`, `Float32` is `f32`, pointers,
  `Bool`, `Char` and narrower integers are `i32` (extended as in 15.3.2). `W = 4`.
- **Verse-cc** lowers a signature as in 15.4.1 with `2W = 8`, so any aggregate
  larger than 8 bytes, including `Int?` and every header, is Memory. A Pair
  result is a multi-value result. Multi-value is part of the Wasm 2.0 core and
  is required by the emitter (assumption).
- **Shadow stack.** Wasm has no addressable stack. Memory-class temporaries,
  address-taken locals and every frame region that a `ref` points to live on a
  shadow stack in linear memory, with the stack pointer in a mutable global
  (`__verse_sp`) and the limit in another (`__verse_stack_limit`). The stack is
  placed at the **lowest addresses**, below static data, so an overflow that
  the explicit check misses reaches address 0 and traps rather than corrupting data.
- **Linear memory** grows with `memory.grow` as the allocator needs it, up to the
  `[memory] heap` cap of the target ([15.9](#159-the-versetarget-description-file));
  a refused grow is `out_of_memory`.
- **Imports and exports** by artifact kind ([04 §4.4](backends-and-targets.md#44-linking-and-artifacts)):

  | Artifact | `[wasm] imports` | Entry | Notes |
  |----------|------------------|-------|-------|
  | WASI p1 command | `wasi_snapshot_preview1` | export `_start`, export `memory` | `proc_exit(code)` for exit |
  | WASI p2 command component | `wasi:cli`, `wasi:clocks`, `wasi:random`, `wasi:filesystem`, `wasi:sockets` as used | export `wasi:cli/run` | built with the component tooling; `exit-with-code` for exit ([15.8](#158-exit-codes)) |
  | Browser module | a small JavaScript glue ABI (`verse_host.*`) | export `verse_start`, `verse_wake`, export `memory` | the glue is generated next to the module |
  | Library component | WIT generated from `export` items | per WIT | canonical ABI below |

- **Component ABI at the boundary.** Verse-cc is **not** the component ABI. A
  component export gets a generated wrapper that lowers and lifts values by the
  canonical ABI: `Int` to `s64`, `Float` to `f64`, `Bool` to `bool`, `Char` to
  `char`, `Text` to `string` (UTF-8, which matches the `Text` encoding so lifting
  is one copy), `List<T>` to `list<T>`, `T?` to `option<T>`, `A | Error` to
  `result<A, error>`, records to `record`, enums to `variant`, tuples to
  `tuple`, `Map<K,V>` to `list<tuple<K,V>>` and `Set<T>` to `list<T>` in
  insertion order (assumption). `Task`, channels, `Shared`, `Guarded`,
  closures and `dyn` values cannot cross a component boundary and make the
  `export` an error. The wrappers use `cabi_realloc`, backed by the Verse allocator.
- **Tasks** are state machines ([15.6.2](#1562-resumable-functions-state-machine-and-cooperative-targets)). A blocking host operation is a pollable (WASI p2) or a
  glue callback (browser); the scheduler returns to the host while tasks wait.

### 15.5.7 Tier 0: the VM

The VM's convention is [14.3](bytecode-and-vm.md#143-value-representation-and-call-records).
It differs from Verse-cc deliberately, because it never meets compiled code:

| Aspect | Tier 0 | Verse-cc |
|--------|--------|----------|
| Arguments | built **in place** in a window at the top of the caller's frame; the callee's base is the window start; nothing is copied | registers and stack of the platform |
| Aggregates | always inline in `ceil(size/8)` slots, laid out by `layout` | classified Zero / Scalar / Pair / Memory |
| Borrows | **always a host address** in a slot, including a shared borrow of an `Int` | by value when `Copy` and small |
| Result | at the window start | registers or hidden pointer |
| Unwinding | exception table per proto, read only while unwinding | per `unwind` ([15.7](#157-unwinding-and-abort-abi)) |
| Layout | the **host's** pointer width (`.vsb` records it and is rejected on a mismatch, 14.1) | the target's |
| `Int` on a 32-bit host | one 8-byte slot, two-word arithmetic | register pair |

Observable behaviour is identical (14.12). A slot is 8 bytes on every host,
so a 32-bit host's `Text` header (12 bytes) occupies two slots with the second
half-used.

### 15.5.8 Stackful context switch, summary

| Target | Saved by the switch | Stack bounds register |
|--------|--------------------|-----------------------|
| x86-64 SysV | `rsp rbx rbp r12`-`r15`, x87 CW, `mxcsr` | none (limit in TCB) |
| x86-64 Windows | the above, `rdi rsi`, `xmm6`-`xmm15` | TEB `StackBase`, `StackLimit`, `DeallocationStack` |
| AArch64 | `x19`-`x30 sp d8`-`d15` | none |
| RV64 | `ra sp s0`-`s11 fs0`-`fs11` | none |

The switch is written once per row in assembly in `verse-rt`; it is the only
target-specific code of the `stackful` strategy. A stackful target needs a row
here; `tasks = "stackful"` with an architecture that has none is a target-file
error ([15.9.4](#1594-validation)).

## 15.6 Tasks across the ABI

The observable behaviour of tasks is one thing ([ADR-0007](../decisions/0007-task-model.md),
[05 §5.3](runtime.md#53-tasks-and-scheduling)); this section fixes only what
the strategies need from the ABI. The scheduler policy is 14 §14.7.

### 15.6.1 Stackful targets

- A task has its own stack (a lazily committed reservation with a guard page;
  the reservation is `[tasks] max_stack` of the target file, 8 MiB by default,
  committed as touched). A task is created with an initial frame for the **task entry
  trampoline**, a runtime function that calls the task's root function with the
  arguments moved in at `spawn` (13.4) and that is the **catch point** of the
  task ([15.7.5](#1575-catch-points)).
- The unwinder must stop at the trampoline: the trampoline frame is the last
  frame of the stack and its call-frame information marks the return address
  undefined.
- The TCB holds the stack bounds (`stack_lo`, `stack_limit`), the unwind state
  ([15.7.1](#1571-the-panic-record-and-the-unwind-state)), `cancel_requested`,
  and `cancel_mask`.

### 15.6.2 Resumable functions: state-machine and cooperative targets

On `tasks = "state-machine"` and `"cooperative"` targets a **may-yield**
function (`mayyield`, 13.2, V18) is not lowered with Verse-cc. It becomes a
**resumable function**:

```
struct F_State {            // layout by the `layout` pass, L4 applies
    resume: u32,            // 0 = not started, k = after yield point k, DONE = finished
    ...                     // every local that is live across a yield point
    ...                     // the State of each may-yield callee that can be active (inline, not boxed)
}
fn F_poll(state: ptr, reason: u32) -> u32     // reason: NORMAL = 0, CANCEL = 1
                                              // result: DONE = 0, PENDING = 1, UNWOUND = 2
```

- The arguments of `F` are written into `F_State` by the caller before the
  first poll; the result is written to the `-> place` of the caller's `call`,
  which lives in the *caller's* state or frame.
- A `call` of a may-yield callee in a may-yield caller becomes: initialise the
  callee state inside the caller state, `poll` it, and if it returns
  `PENDING`, store the resume point and return `PENDING` up. Inline embedding
  makes the state size static; a may-yield function that is recursive would
  have unbounded state, which is why the build rejects a waiting function that
  calls itself on these targets (V0909, [16](../reference/diagnostics-catalog.md#162-families)).
- A `yield` terminator stores its operands, stores the resume point, registers
  with the scheduler (channel, task, lock, timer, I/O) and returns `PENDING`.
  The runtime writes the value of the `yield` (the received item, the awaited
  task's result) into its destination place in the state before the next
  `poll`.
- **Cancellation** is delivered at the next `poll` of a parked function:
  `reason = CANCEL` makes the poll set the unwind state to `cancel` and take
  the unwind edge of the `yield` that parked (13.4). A function that is not
  parked sees the request at its next `yield`.
- **Unwinding between resumable functions** is always by status: a callee that
  unwinds returns `UNWOUND` from its `poll`, and the caller's cleanup block
  runs as ordinary code. The unwind mechanism of the target ([15.7.2](#1572-the-three-mechanisms))
  applies only inside a non-yielding function and across a `poll` call.
- Functions outside the may-yield set are plain Verse-cc functions and pay
  nothing (ADR-0007).
- The **cooperative** executor adds only the run queue, the timer and the
  interrupt-to-ready path of 05 §5.7; no thread and no atomic is involved.
- Where an engine offers stack switching, a Wasm target may declare
  `tasks = "stackful"` in its target file and use [15.6.1](#1561-stackful-targets);
  that is a later target, not a 0.1 one.

### 15.6.3 Tier 0

Tasks are VM coroutines ([14.7](bytecode-and-vm.md#147-tasks-scheduler-and-seed)).
Nothing in this document applies beyond the unwinding and exit rules.

### 15.6.4 Stack bounds and overflow

A guard page fault cannot run cleanup, because the cleanup blocks need
stack that is exhausted, and a signal handler cannot unwind arbitrary frames
safely. So **every compiled target uses an explicit check**:

- The prologue of a function that is not a leaf compares the stack pointer
  (minus the frame size) with `stack_limit` in the TCB and, if it is below,
  branches to a cold block that executes `panic stack_overflow`. The limit is
  set `headroom` above the real end of the stack (assumption: 4 KiB on small
  targets, 16 KiB on large ones, tuned in M2) so that cleanup blocks and the panic path run
  in the reserve. Cost target: at most 3 instructions per call.
- The **guard page** stays as a backstop for code that does not check (a runtime
  function, foreign code). A fault there prints the R0004 text of
  [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics), "recursion is too
  deep in task N", exactly like the prologue check, and **aborts** with status
  101, running no cleanup. Every stack-overflow path prints R0004; none has its
  own wording.
- On Wasm the check is against `__verse_stack_limit` and the shadow stack; the
  engine's own call-stack limit is outside Verse's control and, when it fires
  first, traps the instance with a host-defined status. Each recursive function
  gets a minimum shadow frame, so the Verse check usually fires first.
- Tier 0 checks `frame_slots` against the segment (14.3) and reports the same
  panic (R0004). The depth at which it fires may differ between tiers (14.12).

## 15.7 Unwinding and abort ABI

Spec §8.5 and §11.5: a panic or a cancellation stops the task and runs
`Cleanup` for the owners in scope, in reverse declaration order. MIR already
holds every edge (13.4, V14, V15); this section fixes how a target realizes
them. Panics clean up on every target; `abort` is allowed only when no
`Cleanup` value is live at a panic site.

### 15.7.1 The panic record and the unwind state

Each task has an **unwind state** in its TCB: `none`, `panic` or `cancel`, and
a pointer to a **panic record**. Compiled code reads it only on `status-code`
targets; elsewhere the unwinder carries it.

```
struct PanicRecord {            // allocated by verse-rt; layout private, shown for the contract
    kind:      u8,              // overflow=1 div_zero=2 index_oob=3 slice_oob=4 assert=5
                                // expect_none=6 neg_exponent=7 stack_overflow=8 out_of_memory=9
    site:      u32,             // index into the program's panic-site table: instance, span,
                                // dbg clause, and for `overflow` the operator and the type
    task:      u32,             // task id (the main task is t0, 14.7)
    vals:      [i64; 2],        // index_oob: index, len; overflow: the operands; assert: message id
    secondary: *PanicRecord,    // chain of later panics of the same task (15.7.4)
}
```

- `rt_panic(kind, site, a, b)` creates the record, **prints the report to
  standard error at once** (spec §8.5, §11.2; the structured record first,
  rendered as in [05 §5.4](runtime.md#54-panics-and-unwinding), as JSON
  when `VERSE_PANIC=json`), sets the state to `panic`, and starts unwinding.
  Wording comes from `events.toml`, never from the ABI.
- **Cancellation** has state `cancel`, no record, and **prints nothing**
  (spec §11.5). It is raised only at a `yield` ([13.4](mir-specification.md#134-statements-and-terminators)).
- The **user-level call list** of the record is built from the unwinder's frames
  and the debug map (13.7), so it requires the debug map: in a `debug = false`
  build the report has the message and span and **no stack**; the structured
  record carries `stack = null`, and differential tests compare the stack only
  with debug on.

### 15.7.2 The three mechanisms

`unwind` in the target file ([04 §4.3](backends-and-targets.md#43-what-a-target-is))
names the mechanism for **ordinary Verse-cc functions**.

| `unwind` | Raise | Cleanup entry | Continue | Cost on the success path |
|----------|-------|---------------|----------|---------------------------|
| `native` | `_Unwind_RaiseException` (Unix) or `RaiseException` with a Verse customer exception code (Windows) | a **landing pad** per MIR `cleanup` block, found through `.eh_frame` and the `LSDA`, or SEH unwind info on Windows; the personality is `verse_eh_personality` in `verse-rt` | `resume_unwind` is `_Unwind_Resume` (`RtlUnwind` path on Windows) | none |
| `wasm-eh` | `throw $verse_unwind` with the tag payload `(kind: i32, record: i32)` | each cleanup block is a `try_table` with `catch_all_ref` | `throw_ref` | none, plus the engine's cost for `try_table` regions |
| `status-code` | `rt_panic` returns with the state set | after every call with an unwind edge: `load state; branch if nonzero` to the cleanup block | `resume_unwind` is a normal `return` with the state still set | one load and one branch per call that can unwind |
| `abort` | `rt_panic` prints, then ends the process (15.7.6) | none | none | none |

Details that each backend must follow:

- **Native.** Verse code has **no catch clauses**: a landing pad only runs a
  cleanup block and resumes, because `otherwise`, `match` and `?` never see an
  unwind (14.8). The only catches are the runtime's ([15.7.5](#1575-catch-points)). The
  exception object is a small struct with a class tag `VERSE` and a pointer to the
  panic record (or null for cancellation). The unwind is **two-phase** only
  because the platform unwinder is; phase 1 finds the task trampoline.
- **Status code** (`thumb`, RV32, Wasm without EH). The state is **one word in the
  TCB**. On single-threaded targets that word is a global (a Wasm global, a
  fixed RAM address), and the scheduler **swaps it with the saved copy of each
  task at every task switch**, so a task that yields inside a cleanup block
  keeps its own state. A call that can unwind is followed by the test;
  nothing else changes in the callee's signature. It is chosen over a
  second result register because it needs no change to the platform
  convention on any of these targets; M7 measures it against a second result
  register on Cortex-M.
- **Wasm EH** is a per-target-file opt-in in 0.1: shipped Wasm target files
  default to `status-code`, because it works on every engine and the visible
  order of `Cleanup` is the same (05 §5.4). The `-eh` variants set `wasm-eh`.
  The default switches to `wasm-eh` at M3 only if both hold: every engine on
  the supported list (the browsers and WASI runtimes of the Wasm differential
  tests) runs Wasm exception handling, and `wasm-eh` binaries are smaller or
  faster than `status-code` ones ([11](../roadmap.md), M3).
- **Cleanup blocks are ordinary code** on every mechanism: only `drop`,
  `set_flag`, `storage_dead`, intrinsics, `task.abandon` and calls to
  `nounwind` functions (V15), so the same block serves the normal and the
  unwinding path of a status-code target.

### 15.7.3 `task.abandon` and unwaited tasks

`intrinsic task.abandon(move t)` ([13.4](mir-specification.md#134-statements-and-terminators))
requests cancellation of the child `t` and **hands its join to the
runtime**: the runtime finishes the child (it unwinds with its own `Cleanup`)
before the parent's own unwinding completes. Only a child cancelled by an unwinding parent prints
nothing for that cancellation (spec §11.3); an explicit `cancel()` keeps the
§11.3 behaviour (an unwaited task that failed, including by cancellation, is
reported and makes the exit non-zero). A child that fails with its own panic or
error while unwinding is still reported on standard error and counts for the
exit status.

### 15.7.4 A panic inside `Cleanup`

This section is the target mechanism. The rule is **task-level**: a panic
inside a `Cleanup` never takes the process down by itself.

1. The user's `cleanup(mut self)` runs inside the drop glue's catch point
   (15.7.5). If it panics, the panic unwinds `cleanup`'s own frames, running
   *their* cleanups, and stops at the glue.
2. The runtime **records the second panic**, prints it at once like any panic,
   and appends it to the `secondary` chain of the task's primary record. If
   the task was not unwinding, the second panic is the primary one.
3. The glue **returns normally to its cleanup block**: the remaining fields of
   the value and every later `drop` in the chain still run (no resource is
   leaked because one `cleanup` failed). The unwind state of the task is
   unchanged.
4. When the unwinding finishes at the task boundary, the task ends **panicked
   with the primary record**; `TaskPanicked.trace()` shows the chain. At the
   main task this is exit status 101 (15.8).
5. A panic in a `Cleanup` that runs **during cancellation** is handled the same
   way: the task ends as panicked, not cancelled, with that panic primary.
6. A panic in a `Cleanup` on the **normal path** (no unwind in progress): the
   glue stores the record as the task's panic and returns; the caller then
   starts unwinding. In MIR this needs the `drop` of a `Cleanup` type to have
   an unwind edge after the call, so the
   `drop` of a `Cleanup` type is a terminator whose `unwind` edge starts the
   unwinding at once ([13.4](mir-specification.md#134-statements-and-terminators)).

The MIR verifier is unchanged: the cleanup blocks stay `nounwind` (V15).

### 15.7.5 Catch points

The runtime owns every catch. There are exactly three:

| Catch point | Where | What it does |
|-------------|-------|--------------|
| **Task boundary** | the task entry trampoline (stackful), the scheduler's call of the root `poll` (state-machine, cooperative), the `Pick` of tier 0 | turns the state into the task's outcome: `ok`, `panicked(record)` or `cancelled`; wakes joiners; the `TaskPanicked` and `Cancelled` errors are values only from here on (14.8) |
| **`cleanup` call** | the drop glue (15.3.6) | the rule of 15.7.4 |
| **Program entry** | the runtime start function, around the main task | maps the main task's outcome to the exit status ([15.8](#158-exit-codes)) |

Nothing else catches. In particular a **foreign frame is never unwound
through**: `extern` functions are `nounwind`, and callbacks from foreign
code into Verse do not exist. A foreign
function that unwinds is undefined behaviour ([07 §7.5](plugins.md#75-ownership-and-lifetime)).

### 15.7.6 The `abort` profile and `unreachable`

- **`abort`** (profile `unwind = "abort"` or a target file with `unwind =
  "abort"`): `rt_panic` prints the report, flushes standard error and ends the
  process with status 101. It is allowed only when no `Cleanup` value is live
  at any panic site (V0908 otherwise). Under `abort` a panic in a non-main
  task cannot become `TaskPanicked` at `.wait()`, so an `abort` build **also
  fails with V0910 when a `spawn` is reachable**.
- **`unreachable`** (13.4) and an internal runtime error are **traps**: native
  code executes the architecture's trap instruction, Wasm executes `unreachable`,
  the VM stops with a bug report ([14.4](bytecode-and-vm.md#144-instruction-set-and-encoding)). No cleanup runs. Where the runtime is the cause (a
  runtime trap, an internal error, `unreachable`, a VM `TRAP`, a failed run-time
  capability re-check), the exit status is **3**, "a bug in Verse" ([16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses)). A host trap that Verse cannot control (an
  engine's own call-stack limit, a kill signal) keeps the status the host gives.

## 15.8 Exit codes

Spec §12.5 fixes the exit status of a program: `0` on normal completion; `1` if
a top-level `?` fails or an unwaited task failed (§11.3); `101` on a panic; or
the value passed to `exit(code)`, which overrides the unwaited-task status. This
section adds the cases the spec does not cover and says how the status reaches
the host on each platform layer. It agrees with spec §12.5 and §15.1,
[16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses) and
[14.12](bytecode-and-vm.md#1412-differential-testing-contract).

### 15.8.1 Statuses

| Status | Meaning | Source |
|--------|---------|--------|
| `0` | normal completion; the main task finished and all its tasks were waited | spec §12.5 |
| `1` | a top-level `?` failed (the error is printed to standard error), or an unwaited task failed with an error | spec §12.5, §11.3 |
| `2` | errors in the source, or a capability the target does not offer or the manifest does not grant (V0901, V0902, a `--deny`), found by the build-time check in `check`, `build` or when running a file: the program did not start | [16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses), [14.9](bytecode-and-vm.md#149-sys-capability-calls-and-the-capability-check) |
| `3` | a bug in Verse: a compiler bug (V9901, V9902) found before the program started, when running a file as in `build`; or a trap the runtime itself caused (`unreachable`, an internal runtime error, a VM `TRAP`); or a capability re-check that fails at run time, because the build-time check should have caught it (15.7.6) | 16 §16.6.1, 14.4, 14.9 |
| `101` | a panic in the main task, or in an unwaited task (an `out_of_memory` and a `stack_overflow` are panics); a panic in a `Cleanup` is task-level and ends the main task with this status, with the first panic primary (15.7.4); also the `abort` profile (15.7.6) and a guard-page fault, which prints the R0004 text (15.6.4); also `exit` with a code outside 0 to 255 (R0013) | spec §8.5, 05 §5.4, 14.7, 16 §16.6 |
| `70` | **deadlock** (R0008): no runnable task, no pending completion, no armed timer or interrupt source, and blocked tasks exist. The report lists each blocked task and what it waits for ([08 §8.7](debugging.md#87-vocabulary-and-wording), stop reason `deadlocked`) | 14.7, 14.12 and 16 §16.6.1 |
| `code` | `exit(code)`, `code` in 0 to 255; see below | spec §12.5, 16 §16.6 (R0013) |
| host-defined | a host trap or signal that Verse does not control (an engine's call-stack limit, a kill signal), and a foreign unwind: the host reports it | 15.7.6 |

**`exit(code)`** (spec §12.5 and §15.1). It ends the
program **immediately**, from any task: it runs **no `Cleanup`**, does not
cancel the other tasks, never raises R0009, flushes standard output and
standard error, writes the `debug` end event, and delivers `code`. A code
outside 0 to 255 is not an exit: it is a panic, **R0013**, on every platform
(status 101, with `Cleanup` as for any panic), so the same call means the same
on every target. A program that needs its cleanups returns from `main` or ends
with a failed `?`. There is no second, cleanup-aware exit: those two paths
already run every `Cleanup` and wait for the other tasks (spec §11.3).

**Deadlock detection** is the same predicate on every tier and is computed by
the scheduler of the platform layer: *no runnable task, no pending I/O
completion, no timer, no interrupt source armed for a waiting task*.
A program that waits on stdin, a socket, a timer or a pin is **not**
deadlocked. A task blocked inside a `Cleanup` counts as blocked; its
line in the R0008 report reads "waiting inside the cleanup of {type}" (fact
`cleanup_of`), and the debugger stop is `deadlocked` with the same fact. Only
`exit(code)` ignores a blocked cleanup. On `std-web` the host event loop may still wake the program, so
"no pending completion" includes outstanding host callbacks registered through
the glue; with none outstanding the report is printed to the console and the
exit status `70` is delivered to the glue.

Statuses 2 and 3 of a compiler or build-time check are decided before the
program starts, so they never compete with the others.

**When more than one applies.**
An explicit `exit(code)` always wins (spec §12.5); otherwise `70` if the
run ended by deadlock; otherwise `101` if the main task or any unwaited task
panicked; otherwise `1` if a top-level `?` failed or an unwaited task failed
with an error; otherwise `0`.

### 15.8.2 Delivery by platform layer

| Layer | How the status leaves | Range | Notes |
|-------|-----------------------|-------|-------|
| `std-os`, Unix | `exit(status)` after flushing standard output and error; `_exit` from a signal-unsafe context | 0 to 255 | `exit(256)` never reaches the OS: it is panic R0013 |
| `std-os`, Windows | `ExitProcess(status)` | 0 to 255 | same |
| `std-wasi` p1 | `proc_exit(status)` | 0 to 255 | same |
| `std-wasi` p2 | `wasi:cli/exit.exit-with-code(status)` | `u8` | stabilized in WASI 0.2.12; an older host has only `exit(result)`, where every nonzero status becomes failure; the runtime falls back to `exit(err)` there with a warning (verified in M3) |
| `std-web` | the glue calls `onExit(status)`; the page is not closed; the status is also logged to the console | 0 to 255 | there is no process |
| `std-embedded` | the target's `[exit] on_exit`: `halt` (low-power loop), `reset`, `semihosting` (the semihosting exit call, used with an emulator in CI), or `host` (return to the board's own startup code) | 0 to 255 | status is also written to the console if `stdio` is granted |
| tier 0 | the `verse` process exits with the status | as the host | the VM is a host process |

The flushing order is fixed: the main task has returned or unwound, its
tasks have been waited (14.7) (a call to `exit` skips these two steps), standard
output and then standard error are flushed, the `debug` slice writes its end event (`End`, 14.10), and then the
status is delivered.

## 15.9 The `verse.target` description file

A target is **data plus two small implementations**
([04 §4.3](backends-and-targets.md#43-what-a-target-is)): lowering (a
backend) and a platform layer. The data is a TOML file, the **target file**.
This section is its full schema. The example of 04 §4.3 shows the core keys;
every one keeps its name and meaning here.

### 15.9.1 Files and lookup

A target has a **name**: the key by which `verse.toml` (`[targets] default`,
`check`, `[targets.<name>]`, [19 §19.3](../reference/cli-and-manifest.md#193-versetoml))
and `[library.targets]` refer to it.

| File | Where | Defines |
|------|-------|---------|
| `targets/<name>.toml` | the toolchain; the shipped targets | the target `<name>` (the file stem; `[target] name` may repeat it) |
| `targets/<name>.target` | a project | a project-local target `<name>` |
| `verse.target` | the package root | exactly one project-local target, whose `[target] name` is **required** |

All three have the same schema. A name is resolved in this order: project
`targets/<name>.target`, the package-root `verse.target` if its `name`
matches, then the toolchain. The reserved name `host` is not a file: it is the
shipped target of the machine running the compiler. A project-local target may
not reuse the name of a shipped one (a build error). The target file is read
as data: it runs no code, in line with the rest of the manifest (21 §21.2, point 2).
Its blake3 hash is part of every cache key ([21 §21.10](packages.md#2110-reproducible-builds-and-caching),
02 §2.7).

`[target] inherits = "<name>"` starts from another target and overrides
only the tables and keys written. It is the way to describe a board: the
CPU target is shipped and the board adds memory, capabilities and an exit
rule.

### 15.9.2 Schema

Unknown tables and keys are errors (as in 19). Sizes are text
with `K`, `M`, `G` suffixes (powers of 1024) or integers (bytes).

**`[target]`**

| Key | Type | Required | Meaning |
|-----|------|----------|---------|
| `name` | text | in `verse.target` | the target's name; `[a-z][a-z0-9_-]*` |
| `triple` | text | yes | the LLVM and Rust style triple, used by the backends and the linker |
| `inherits` | text | no | a target name to start from |
| `pointer_width` | `32` \| `64` | yes | W is `pointer_width / 8` (15.2); drives all layout |
| `endian` | `"little"` | yes | `"big"` is reserved (L1) |
| `abi` | `sysv` \| `win64` \| `aapcs64` \| `aapcs` \| `aapcs-vfp` \| `lp64d` \| `ilp32` \| `wasm-basic` | derived from `triple` | the row of 15.5 Verse-cc uses; set explicitly only for a custom target |
| `backend` | `cranelift` \| `llvm` \| `wasm` | yes | the **default** backend ([04 §4.1](backends-and-targets.md#41-tiers)); `[profile.*] backend = "auto"` (19) picks this |
| `backends` | list | `[backend]` | the backends that can build this target; a profile naming another is an error. Cranelift is not listed for 32-bit ARM or RISC-V (04 §4.1) |
| `cpu`, `features` | text, list of text | no | passed to the backend (for example `"cortex-m4"`, `["+vfp4d16sp"]`) |
| `unwind` | `status-code` \| `native` \| `wasm-eh` \| `abort` | yes | the mechanism of [15.7.2](#1572-the-three-mechanisms) |
| `tasks` | `stackful` \| `state-machine` \| `cooperative` | yes | the strategy of ADR-0007; 05 §5.3 |
| `threads` | bool | `false` | whether the program may run on more than one OS thread (04 §4.2 "Threads") |
| `atomics` | `native` \| `none` | `native` | `native`: the architecture has compare-and-swap. `none`: it has only single loads and stores, so the target is single core with interrupts masked around runtime critical sections (15.5.4) |
| `float` | `hard` \| `soft` | `hard` | the float ABI variant |
| `fpu` | `double` \| `single` \| `none` | `double` | hardware binary64 (`single`: binary64 is a library call, `none`: all floats) |
| `runtime_slice` | `std-os` \| `std-wasi` \| `std-web` \| `std-embedded` \| `core` | yes | the **platform layer** ([15.10](#1510-platform-layers-and-capability-mapping)); the key name is 04's, the value is 05's "platform layer" |
| `object_format` | `elf` \| `macho` \| `coff` \| `wasm` | yes | |
| `linker` | `rust-lld` \| `lld-link` \| `ld64.lld` \| `wasm-ld` | yes | the bundled linker (04 §4.4) |

**`[platform]`**

| Key | Type | Meaning |
|-----|------|---------|
| `os` | `linux` \| `macos` \| `windows` \| `wasi` \| `web` \| `none` | selects the provider code of the layer |
| `libc` | `gnu` \| `musl` \| `msvc` \| `none` | the C runtime the executable links; `none` for freestanding |
| `startup` | `libc-main` \| `reset-vector` \| `wasm-start` \| `wasm-export` | who calls the runtime start function ([15.10.1](#15101-entry-and-startup)) |

**`[memory]`** (embedded and Wasm; absent for `std-os`)

| Key | Type | Meaning |
|-----|------|---------|
| `flash`, `ram` | `{ origin = "0x...", size = "..." }` | memory regions for the generated linker script |
| `stack` | size | the single stack of a cooperative target, or the shadow stack of a Wasm module |
| `heap` | size | the heap: a fixed region on `std-embedded`; the **cap** of linear memory growth on Wasm; absent means unbounded (`--max-memory` applies). Overridden per project by `[targets.<name>] heap` |
| `initial` | size | Wasm initial linear memory |

**`[tasks]`**

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `max_stack` | size | `"8M"` | the most stack one task may use; `--max-stack` overrides it (19). Stacks grow as they are used (VM segments, 14.3; pages committed on first touch on a stackful target, where this is the address space reserved per task), so the cap costs nothing until used. On a `cooperative` or `state-machine` target the tasks run on one shared stack, and the target file sets its own value, at most `[memory] stack` |
| `workers` | `"cores"` \| integer | `"cores"` | stackful with `threads = true`; ignored otherwise |

**`[exit]`**

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `on_exit` | `process` \| `host` \| `halt` \| `reset` \| `semihosting` | from `os` | how a status is delivered (15.8.2) |

**`[wasm]`** (`backend = "wasm"` only)

| Key | Type | Meaning |
|-----|------|---------|
| `artifact` | `command` \| `component` \| `module` | the default artifact; the manifest's `[build] artifact` may refine it |
| `imports` | `wasi_snapshot_preview1` \| `wasi:cli` \| `js` | the host interface (15.5.6) |
| `features` | list | engine features the emitter may use: `multi-value`, `exception-handling`, `bulk-memory`, `reference-types` |

**`[capabilities]`** ([15.10.3](#15103-the-capability-table)): a map from capability name to `false`, `true` or a text *detail*. What the target
can provide to `sys` ([05 §5.6](runtime.md#56-the-sys-capability-interface)).

### 15.9.3 Examples

A desktop target:

```toml
# targets/x86_64-unknown-linux-gnu.toml
[target]
triple          = "x86_64-unknown-linux-gnu"
pointer_width   = 64
endian          = "little"
backend         = "cranelift"
backends        = ["cranelift", "llvm"]
unwind          = "native"
tasks           = "stackful"
threads         = true
atomics         = "native"
float           = "hard"
fpu             = "double"
runtime_slice   = "std-os"
object_format   = "elf"
linker          = "rust-lld"

[platform]
os      = "linux"
libc    = "gnu"
startup = "libc-main"

[tasks]
max_stack = "8M"

[exit]
on_exit = "process"

[capabilities]
stdio   = true
fs      = true
net     = true
clock   = true
random  = "os"
process = true
os      = true
env     = true
gpio    = false
surface = false          # becomes true when sys-surface ships (M9)
gpu     = false
```

A WASI target. It uses status codes, so it needs no engine feature beyond
multi-value:

```toml
# targets/wasm32-wasip1.toml
[target]
triple          = "wasm32-wasip1"
pointer_width   = 32
endian          = "little"
backend         = "wasm"
unwind          = "status-code"
tasks           = "state-machine"
threads         = false
atomics         = "none"
runtime_slice   = "std-wasi"
object_format   = "wasm"
linker          = "wasm-ld"

[platform]
os      = "wasi"
libc    = "none"
startup = "wasm-export"

[memory]
stack   = "1M"
initial = "1M"
heap    = "256M"           # cap of linear memory growth

[exit]
on_exit = "host"

[wasm]
artifact = "command"
imports  = "wasi_snapshot_preview1"
features = ["multi-value", "bulk-memory"]

[capabilities]
stdio   = true
fs      = "preopened"      # only directories the host preopens
net     = false            # p1 has no connect
clock   = true
random  = "os"
process = "exit-only"
env     = true
```

The browser target, with the JavaScript glue:

```toml
# targets/wasm32-unknown-unknown.toml
[target]
triple          = "wasm32-unknown-unknown"
pointer_width   = 32
endian          = "little"
backend         = "wasm"
unwind          = "status-code"
tasks           = "state-machine"
runtime_slice   = "std-web"
object_format   = "wasm"
linker          = "wasm-ld"

[platform]
os      = "web"
startup = "wasm-export"

[memory]
stack   = "512K"
initial = "2M"
heap    = "512M"

[exit]
on_exit = "host"

[wasm]
artifact = "module"
imports  = "js"
features = ["multi-value", "bulk-memory"]

[capabilities]
stdio   = "console"        # stdout and stderr go to console.log and console.error
clock   = true             # performance.now and Date.now
random  = "os"             # crypto.getRandomValues
env     = "args-only"      # arguments from the start call
surface = false            # becomes "canvas" with sys-surface (M9)
fs      = false
net     = false
process = "exit-only"
```

A Cortex-M target (the example of 04 §4.3, with the memory data it lacks):

```toml
# targets/thumbv7em-none-eabihf.toml
[target]
triple          = "thumbv7em-none-eabihf"
pointer_width   = 32
endian          = "little"
backend         = "llvm"
cpu             = "cortex-m4"
unwind          = "status-code"
tasks           = "cooperative"
threads         = false
atomics         = "native"
float           = "hard"
fpu             = "single"
runtime_slice   = "std-embedded"
object_format   = "elf"
linker          = "rust-lld"

[platform]
os      = "none"
libc    = "none"
startup = "reset-vector"

[memory]
stack = "4K"
heap  = "16K"              # a default; a project overrides it with [targets.<name>] heap

[tasks]
max_stack = "4K"           # not the hosted 8M: tasks share the one stack of [memory]

[exit]
on_exit = "halt"

[capabilities]
clock  = "monotonic"       # a timer tick; no wall clock
random = "hardware"
fs     = false
net    = false
gpio   = true
uart   = true
stdio  = "uart"            # print goes to the UART console (20, iot)
```

A board, inheriting the CPU target and adding what is specific to the board.
This is a `verse.target` in the package root:

```toml
# verse.target
[target]
name     = "my-board"
inherits = "thumbv7em-none-eabihf"

[memory]
flash = { origin = "0x08000000", size = "512K" }
ram   = { origin = "0x20000000", size = "128K" }
stack = "8K"

[exit]
on_exit = "semihosting"    # an emulator in CI sees the status

[capabilities]
i2c = true
spi = true
```

### 15.9.4 Validation

A target file is checked when it is loaded; a violation is a build error with
a message naming the key. The codes are V0911 to V0915 of the V09xx family of
[16](../reference/diagnostics-catalog.md#162-families); the meanings
below are those of 16. Like any source error, each ends the run with status 2.

| Rule | Code | Reason |
|------|------|--------|
| `backend` is in `backends`; `wasm` needs `object_format = "wasm"` and `[wasm]`; `cranelift` needs `pointer_width = 64` | V0911 | 04 §4.1 |
| `tasks = "stackful"` needs an architecture with a context-switch row in 15.5.8 | V0912 | 05 §5.3 |
| `tasks = "state-machine"` or `"cooperative"` forbids `threads = true` | V0911 | no workers are specified for them in 0.1 |
| `atomics = "none"` forbids `threads = true` | V0911 | 15.5.4, 15.5.5 |
| `unwind = "wasm-eh"` needs `backend = "wasm"` (V0911) and `"exception-handling"` in `[wasm] features` (V0912) | V0911, V0912 | 15.7.2 |
| `unwind = "native"` needs a backend that emits unwind tables for the architecture | V0911 | 15.7.2 |
| every capability key has a provider in the platform layer's menu ([15.10.3](#15103-the-capability-table)); a detail value is one the provider knows | V0913 | no capability without code |
| `[memory] heap` on `std-os` is an error; on `std-embedded` it must fit in `ram` after `stack` and static data | V0914 | the linker script is generated |
| `pointer_width`, `endian` and `abi` agree with `triple` | V0911 | one source of truth for layout |
| the name in `[targets]`, `check` or `--target` resolves to a file (15.9.1) | V0915 | 16 lists the places looked in |

### 15.9.5 How the target file relates to the manifest

The target file says **what the platform is**. `verse.toml` says **what this
project asks of it**. A key lives in exactly one place:

| Question | Target file | `verse.toml` | Rule |
|----------|-------------|--------------|------|
| Which targets does the project build and check? | no | `[targets] default`, `check` | by target name |
| How big is the heap? | `[memory] heap` (the default for the board) | `[targets.<t>] heap` | the manifest **wins** (19); `[targets.<t>] no_heap = true` forbids allocation altogether (05 §5.7) |
| Which entry file, which module variants? | no | `[targets.<t>] entry`, `files` | per-target code is chosen by files, never by conditions in source (06 §6.9) |
| Which capabilities can the target provide? | `[capabilities]` | no | what the hardware and platform layer can do |
| Which capabilities may the program use? | no | `[capabilities] grant` (19) | an application's explicit allowance |
| Which tier and optimizer? | `backend`, `backends` | `[profile.*] backend`, `opt`, `debug` | the profile picks among `backends` |
| Panic mechanism? | `unwind` | `[profile.*] unwind = "target" \| "abort"` | `target` means the file's mechanism; `abort` is the profile of 15.7.6 |
| Is a library available here? | no | `[library.targets] available` / `unavailable`, by **target name** (06 §6.9) | a custom target is matched by its `name` |

**Effective capabilities.** A reachable capability use must pass, in order:
the target provides it (V0902), the manifest grants it when `[capabilities]`
is present (V0901), and no `--deny` removes it (14.9). In set terms:
`required ⊆ (provided(target) ∩ grant) \ deny`, and the first failing step is
named in the message. The check is the *capability check* that runs right
after reachability ([02 §2.2](compiler-architecture.md#22-stage-contracts)).

A project-local target and `[targets.<t>]` coexist: the first declares the
platform, the second the project's choices for it.

```toml
# verse.toml
[targets]
default = "host"
check   = ["host", "wasm32-wasip1", "my-board"]

[targets.my-board]
heap  = "24K"              # replaces [memory] heap of verse.target
entry = "src/board.vs"

[capabilities]
grant = ["stdio", "gpio", "i2c", "clock"]
```

## 15.10 Platform layers and capability mapping

Each target names one **platform layer** in `runtime_slice`
([05 §5.1](runtime.md#51-feature-slices)). A layer is a **menu** of runtime
slices: the allocator, the entry point and the `sys-*` capability providers
the target can offer. A build links only the slices that reachability asks for,
never the whole layer ([06 §6.5](stdlib-and-shaking.md#65-capabilities-and-runtime-slices)).

| Layer | Targets ([04 §4.2](backends-and-targets.md#42-target-matrix)) | Allocator | Tasks | Unwind default | Threads |
|-------|----------|-----------|-------|----------------|---------|
| `std-os` | Linux, macOS, Windows, RISC-V 64 Linux | size-class allocator with thread caches (05 §5.2) | stackful, work-stealing | `native` | yes |
| `std-wasi` | `wasm32-wasip1`, `wasm32-wasip2` | small single-thread allocator over linear memory | state machines | `status-code` | no (single thread first) |
| `std-web` | `wasm32-unknown-unknown` | the same, with the glue's memory | state machines | `status-code` | no (workers later) |
| `std-embedded` | Cortex-M, ESP32 RISC-V, later Xtensa | fixed-region bump or TLSF allocator in `[memory] heap` | cooperative executor | `status-code` | no |
| `core` only | bare-metal x86-64 (later) | none | none | `status-code` | no |

### 15.10.1 Entry and startup

All layers run the same program shape: the runtime start function creates the
main task `t0`, which runs the `<script>` function (spec §12: top-level
statements are the only entry; 14.7), runs the scheduler until the main task has
returned and its tasks are waited (or a task calls `exit`), then performs the exit sequence of
[15.8.2](#1582-delivery-by-platform-layer).

| Layer | Who calls the start function | Before it | Arguments and environment |
|-------|------------------------------|-----------|---------------------------|
| `std-os` | the C runtime's `main` (`startup = "libc-main"`) | allocator and stdio initialised, signal handlers for the guard-page backstop installed (15.6.4), one worker per core started lazily | `args` from `argv` without the program name (spec §12), `env` from the environment |
| `std-wasi` p1 | the export `_start` | the allocator is set up over the initial memory | `args_get`, `environ_get` |
| `std-wasi` p2 | the export `wasi:cli/run` | as above | `wasi:cli/environment` |
| `std-web` | the export `verse_start(args)`, called by the glue | as above | the `args` given to `verse_start`; `env` is empty unless the glue passes one |
| `std-embedded` | the **reset vector**: the runtime's `Reset` handler | `.data` copied, `.bss` zeroed, the heap region and the timer initialised | `args` is empty and `env` is empty (06 §6.5) |

`std-web` and `std-wasi` p2 are **reactive**: `verse_start` runs until every
task is parked, returns to the host, and is re-entered through `verse_wake`
(browser) or the pollable (WASI) when something completes. Nothing blocks the host.

### 15.10.2 What each layer provides

| Layer | Providers (slices) in 0.1, planned | Not provided |
|-------|------------------------------------|--------------|
| `std-os` | `sys-stdio`, `sys-clock`, `sys-random`, `sys-fs`, `sys-net`, `sys-process`, `sys-os`, `sys-env`, `debug`; later `sys-surface`, `sys-gpu`; on a Linux board `sys-gpio`, `sys-i2c`, `sys-spi`, `sys-uart` through device files when the target file grants them (20) | none |
| `std-wasi` | `sys-stdio`, `sys-clock`, `sys-random`, `sys-fs` (preopened), `sys-env`, `sys-process` (exit only), `debug`; `sys-net` on p2 only | `os`, `surface`, `gpu`, a device bus |
| `std-web` | `sys-stdio` (console), `sys-clock`, `sys-random`, `sys-env` (arguments), `sys-process` (exit only), `debug`; later `sys-surface` (canvas, M9) and `sys-gpu` (WebGPU) | `fs`, `net` listening, `process` spawn, `os` |
| `std-embedded` | `sys-clock`, `sys-random` (hardware or seeded), `sys-stdio` over a UART or semihosting, `sys-gpio`, `sys-uart`, `sys-i2c`, `sys-spi`, `debug` | `fs`, `process`, `os`, usually `net` |

### 15.10.3 The capability table

Each cell is what a target file may write for that capability and layer; `-`
means the layer has no provider, and a target that sets it is rejected
(15.9.4). A **detail** is a text value naming a restriction; the
capability check treats any non-`false` value as provided, and the detail
reaches the provider and the documentation.

| Capability | `std-os` | `std-wasi` | `std-web` | `std-embedded` |
|------------|----------|------------|-----------|----------------|
| `stdio` | `true` (fds 0, 1, 2; console handles on Windows) | `true` | `"console"` (`console.log`, `console.error`; `input` is a glue callback and may be absent) | `"uart"`, `"semihosting"`, `false` |
| `fs` | `true` | `"preopened"` (only the directories the host maps in) | - | - (a flash file system is a later layer) |
| `net` | `true` (sockets, and `tls_*`) | p2: `true` through `wasi:sockets` if the host grants it; p1: `false` | - (a client through `fetch` is not in 0.1) | `false`, or `true` when a board package supplies a stack |
| `clock` | `true` (`monotonic_now`, `wall_now`) | `true` | `true` | `"monotonic"` (no wall clock) or `true` with an RTC |
| `random` | `"os"` | `"os"` | `"os"` (`crypto.getRandomValues`) | `"hardware"` or `"seeded"` (documented as deterministic) |
| `process` | `true` (spawn, wait, exit) | `"exit-only"` | `"exit-only"` | `"exit-only"` (halt or reset, 15.8.2) |
| `os` | `true` (signals, ids, users, permissions, system calls) | - | - | - |
| `env` | `true` | `true` | `"args-only"` | `"empty"` |
| `gpio`, `uart`, `i2c`, `spi` | `true` on a Linux board, through device files | - | - | `true` per board |
| `surface` | per OS, when `sys-surface` ships (M9) | - | `"canvas"` (M9) | - |
| `gpu` | per OS, when `sys-gpu` ships | - | `"webgpu"` (planned) | - |

Notes:

- The capability names are those of 05 §5.6 and 06 §6.5. There is no `timer`
  capability; timers use `clock`.
- A **detail** never changes the call signature of an operation: `fs =
  "preopened"` makes `open` of a path outside the preopens return an `Error`
  value, not a build error. What the target cannot do at all is the build
  error V0902.
- **Granting a capability to a program** is separate from the target
  providing it ([15.9.5](#1595-how-the-target-file-relates-to-the-manifest)).
- On the browser and WASI the providers call host functions (imports); on `std-os`
  they call the OS; on `std-embedded` they read registers. In every layer the
  provider sits behind the same Rust trait per capability, so a test or a
  replay substitutes a fake (virtual clock, in-memory file system) with no
  change to compiled code ([14.9](bytecode-and-vm.md#149-sys-capability-calls-and-the-capability-check),
  05 §5.6 reason 4).
- A custom target that needs a capability outside this table (a CAN bus) has no
  way to add a provider in 0.1. A board driver is called through a plugin,
  outside the capability system ([07 §7.1](plugins.md#71-what-a-plugin-is)).

## 15.11 Foreign-call boundary

The rules for plugins and `extern` are in [07](plugins.md) and spec §12.6.
This section fixes what they mean at the ABI level.

- **The stable ABI is the platform C ABI.** A foreign call is a C-ABI call.
  Verse-cc is the C convention over scalars and pointers (15.4), so a foreign
  function whose parameters are plain data needs **no adapter** in compiled
  code. The type mapping is [07 §7.4](plugins.md#74-values-across-the-boundary):
  `Int` is `int64_t`, `Float` is `double`, `Bool` is a byte, `Char` is
  `uint32_t`, `Pointer` is a pointer-width address, and a `Copy` record is a
  C struct with the target's C layout, which may differ from its Verse layout
  (15.3); the call copies it.
- **Aggregates and Verse headers never cross.** `Text`, `List` and the other
  types of 15.3.3 are not C types. A `Bytes` argument passes a pointer to its
  data, followed by a zero byte; a slice that is not followed by one is copied
  for the call.
- **No unwinding through foreign frames.** A foreign call is `nounwind`
  (`call foreign` with `unwind none`, 13.4), and foreign code never calls back
  into Verse (15.7.5).
- **Tasks.** A foreign call runs on the calling task's stack and thread and is
  not a yield point. On a state-machine target it blocks the single thread
  until it returns. The stack of a stackful task must hold the foreign
  function's frames within the target file's `[tasks] max_stack`
  ([15.6.1](#1561-stackful-targets)); the guard page turns an overflow there into R0004.
- **Script mode.** The VM calls through a stub generated per signature by
  `verse-ffi`, which moves VM registers into C-ABI arguments and back. The stub
  follows the same mapping, so a call means the same in every tier.
- **Capabilities.** A foreign call is not a capability use. `--deny` and
  `[capabilities]` do not reach foreign code ([07 §7.1](plugins.md#71-what-a-plugin-is)).

## 15.12 Conformance and tests

All **proposals**; they extend the plan of [10](testing-and-verification.md).

1. **Layout goldens.** `verse dump mir` prints the `layout` lines for a fixed
   corpus of types (15.3.8 and more) for `x86_64`, `aarch64`, `wasm32` and
   `thumbv7em`. A change to a number is a `rev` bump (L9).
2. **One source of truth.** `verse-rt` declares its headers as `#[repr(C)]`
   Rust types, and its build asserts `size_of` and `align_of` of each against the
   `layout` function of the compiler, for every shipped width. A mismatch fails
   the runtime build, not a program.
3. **ABI torture programs.** A generated suite makes calls with every
   combination of class (Zero, Scalar, Pair, Memory), mode (owned, borrowed,
   exclusive) and result class, with 0 to 12 parameters, and checks the values
   in each backend and in the VM. It also checks that a Memory result is left
   uninitialised and undropped when the callee unwinds (V14).
4. **Unwinding matrix.** The same panic, cancellation and panic-in-`Cleanup` programs run with
   each `unwind` value a target supports; the suite compares the order of
   `Cleanup`, the panic record and the exit status (14.12).
5. **Exit statuses.** A table-driven test per platform layer for each row of
   15.8.1, including the deadlock program, run under the harness of each
   layer (a process, a WASI host, a JavaScript host, an emulator with
   semihosting).
6. **Target files.** Every shipped target file is loaded, validated and
   round-tripped by the schema of 15.9.2; every example in this document is a
   test fixture.
