# 03. IR and pass specification

## 3.1 The representations

| IR | Purpose | Shape | Typed? |
|----|---------|-------|--------|
| Tokens | lexing | flat vector with trivia | no |
| CST | formatter, LSP, error recovery | green tree, lossless | no |
| AST | structure, spans | arena of enums | no |
| HIR | desugared core language | arena of enums, names resolved | no |
| THIR | typed core | HIR plus a type per expression | yes, generic |
| Generic MIR | ownership, drops, unwinding | CFG of basic blocks | yes, generic |
| Mono MIR | optimization, backends | same shape, no type parameters | yes, concrete |

## 3.2 HIR: the desugared core

HIR removes every piece of convenience syntax so later stages handle a small
language. The table gives the mapping. Source forms are from the spec. The core notation in the table and in the MIR blocks below is the compiler's own notation for its data. It is not Verse code.

| Surface form | HIR core |
|--------------|----------|
| `a otherwise b` (§3.8.2) | `match a { present(x) -> x; none -> b }` over the optional or fallible type, typed later |
| `try expr` (§8.3) | early `return` of the absent or error case from the enclosing function or lambda |
| `if x is p` (§6.2) | single-arm match with fallthrough |
| `.name` key-path lambda (§5.4) | lambda with one parameter |
| `x if cond` postfix guards (§6.7) | `if cond` block |
| `"{a} and {b:.2}"` | concatenation of `Show` calls and format calls |
| `for x in it` (§6.3) | loop over `Iterable.next` with a pattern |
| plain dot through optionals (§8.1) | `Lift(chain)` nodes at consuming positions only; other positions error |
| `a.b.c(args)` uniform call syntax (§5.7) | resolved call node (method or function) |

## 3.3 MIR

MIR is the only IR the backends and the optimizer see. It is a CFG in SSA-like
form for temporaries, with named, typed, **mutable locals** for user bindings
(as in Rust MIR). A later pass can promote locals to SSA values for the
backends that want it (Cranelift and LLVM build SSA themselves).

### Core definitions

This section is the overview. The exact grammar, printed form, every
operand and the verifier rules V1 to V24 are in
[13](mir-specification.md), which is the detailed source; if the two
differ, 13 wins and this file is corrected.

```
Function { id, params: [Local], ret: Type, locals: [LocalDecl], blocks: [Block],
           attrs }                                 // attrs: caps, mayyield, nounwind, recursive
Block     { stmts: [Stmt], term: Terminator, is_cleanup: bool, span }

Operand = Move(Place) | Copy(Place) | Const       // ownership made explicit

Stmt =                                             // every Stmt has a span and a debug name
  | Assign(Place, Rvalue)
  | StorageLive(Local) | StorageDead(Local)
  | Drop(Place, flag: Option<DropFlag>)         // run the type's drop glue: no user code
  | SetFlag(DropFlag, bool)
  | SetDiscriminant(Place, Variant)
  | Intrinsic(IntrinsicOp, [Operand])            // backend-visible primitives

Rvalue =
  | Use(Operand)
  | BorrowShared(Place) | BorrowMut(Place)        // second-class, see invariant 4
  | Binary(op, Operand, Operand)                  // wrapping ops only
  | CheckedBinary(op, Operand, Operand)           // returns (value, overflowed)
  | Cast(kind, Operand, Type)                     // cannot fail
  | CheckedCast(kind, Operand, Type)              // returns (value, failed)
  | Aggregate(Kind, [Operand]) | Len(Place) | Discriminant(Place)

Terminator =
  | Goto(Block) | Switch(Operand, [(Const, Block)], Block)
  | Call(Callee, [Operand], dest: Place, next: Block, unwind: Unwind)
  | CallDyn(vtable_slot, [Operand], dest, next, unwind)
  | Return(propagated: bool) | Unreachable | ResumeUnwind
  | Panic(PanicKind, [Fact], unwind: Unwind)     // structured kind and named facts, not a message
  | Yield(YieldKind, [Operand], dest: Place, resume: Block, unwind: Unwind)
                                                  // scheduler point: send, recv, wait, join, sleep, lock, sys, pause
  | Spawn(Callee, [Operand], dest: Place, next: Block, unwind: Unwind)
                                                  // arguments already moved
Unwind = Cleanup(Block) | Continue | None
```

Notes on the shape (details and examples in [13](mir-specification.md)):

- **Operands, not rvalues, carry moves and copies.** `Copy` is only the
  implicit copy of a `Copy` type, bitwise or `counted` (13.3); the explicit
  `.copy()` is a call.
- **Spans, not marks.** MIR has no mark statement. Every statement and
  terminator has a span and a debug name, and the stepping marks that the
  bytecode VM and debuggers need are derived from spans ([13.7](mir-specification.md#137-debug-info-and-source-spans)).
- **Spawn and Yield** carry operands, a destination, a continuation block and
  an unwind edge. Cancellation (spec §11.5) is delivered at a `Yield` and takes
  its unwind edge.
- **Cleanup blocks** end in `ResumeUnwind`. `Unwind::Continue` means there is
  nothing to clean up in this function; `Unwind::None` means the call cannot
  unwind.
- **Drop flags** are written with `SetFlag`.
- **One drop form.** `Drop` is a statement: drop glue runs no user code, so it
  cannot unwind or yield. It frees the value and releases the runtime-owned
  handles in it ([13.4](mir-specification.md#134-statements-and-terminators)).

### Invariants (checked by the verifier)

The seven invariants below are the summary; [13.5](mir-specification.md#135-verifier-rules)
numbers the full rule set (V1 to V24) with a rejected example for each.

1. Every `Place` is typed and every operand matches the expected type.
2. A `Move(p)` is followed on every path by no read of `p` before it is
   reassigned (the verifier re-derives this independently of the ownership
   pass as a safety net).
3. Every `Drop` of a possibly-moved place has a drop flag.
4. Borrow values (`BorrowShared`, `BorrowMut`) are second-class: they appear
   only as call or `Yield` operands, as parameters, and as the source of a
   store into a local of borrow type (a temporary, or a binding that only
   looks at its source: `b = a` while `a` is used later, a `for` loop
   variable). They never reach a stored position (a field,
   an element, the return place, a `Spawn` operand or a captured variable).
5. Every `Call`, `Spawn`, `Yield` or `Panic` that can unwind names a cleanup
   block (or `Continue` when nothing needs cleanup), and cleanup blocks drop in
   reverse declaration order, contain no call or yield, and end in
   `ResumeUnwind`.
6. `CheckedBinary` and `CheckedCast` are followed by a `Switch` to a `Panic(Overflow)` block, so
   no backend decides overflow policy on its own.
7. In **mono** MIR no type contains a type parameter, and every callee is a
   concrete function or an intrinsic or a `dyn` slot.

### Why this shape

- **Explicit moves and drops** let every backend, including the plain
  bytecode interpreter, run identical semantics with no ownership logic of
  its own. The interpreter is a few hundred lines of dispatch because the hard
  part is already done.
- **Checked arithmetic as data** makes overflow policy a MIR concern, so
  optimizations (range analysis) can delete checks in one place for all
  targets.
- **Unwind edges as data** lets each backend choose the mechanism: native
  unwind tables, WebAssembly exception handling, or status-code propagation
  (see [05](runtime.md), section 5.4).

## 3.4 Pass list

Mono MIR optimizer passes, in order. Each pass states what it must preserve.
Every pass is individually switchable (`--no-opt-pass=NAME`) so the
differential tester can isolate a miscompile.

| # | Pass | Effect | Must preserve |
|---|------|--------|---------------|
| 1 | `verify` | check invariants | none (read-only) |
| 2 | `simplify-cfg` | merge and remove blocks, fold constant switches | control flow semantics |
| 3 | `inline` | inline small and single-call functions; always-inline for intrinsics; never a `recursive` function | unwind and drop order; the calls the call limit counts ([15 §15.6.4](target-and-abi.md#1564-stack-bounds-and-overflow)) |
| 4 | `sroa` | split records and tuples into scalars when they do not escape | field-level move state |
| 5 | `const-prop` and `fold` | evaluate constants, fold branches | overflow panics (folding a checked op that overflows stays a panic, not a wrap) |
| 6 | `move-elim` | turn the `copy` of a value at its last use into a `move`, which saves the count of a `counted` copy (spec §4.9: "a source at its last use is moved"); turn a `move` of a `bitwise` type into a `copy` | the order of drops that release runtime-owned handles |
| 7 | `copy-elide` | replace a `counted` copy, or a slice, that is never written and never outlives its source with a borrow (`ref`, `slice_ref`), which saves the count | no aliasing visible to `mut`; copy on write in `verse-rt` keeps every write private |
| 8 | `range-analysis` | prove bounds and arithmetic checks redundant | panics that can still occur |
| 9 | `devirtualize` | turn `dyn` calls into direct calls when the concrete type is known | semantics |
| 10 | `drop-opt` | remove drops of types with no heap data and no `cleanup` fact, merge flags | the order of drops that release runtime-owned handles |
| 11 | `dce` | delete unreachable blocks and unused pure computations | side effects and panics |
| 12 | `layout` | choose field order, niche optimization (`T?` of a pointer type costs no extra byte) | none observable; the `(T, Bool)` result of a checked operation keeps its order (value, then flag), because the bytecode VM and backends rely on it ([13.3](mir-specification.md#133-types-and-layout-facts), [14.4](bytecode-and-vm.md#144-instruction-set-and-encoding)) |
| 13 | `refcount-opt` | elide atomic operations on `Shared` when whole-program analysis shows the program never spawns, or elide the pair on a clone and drop in one block | counts at any observable point |
| 14 | `task-lower` | mark may-yield functions, build state machines or insert stack-switch points depending on target | task semantics |

Rules for adding a pass:

1. A pass has a one-paragraph specification of what it changes and preserves.
2. It has a golden test and a differential test (optimized equals unoptimized
   on the conformance suite).
3. It has a benchmark showing it helps, or it is deleted (spec principle:
   "no unnecessary complexity").

## 3.5 Layout rules

Decisions that backends and the runtime share:

- `Int` is 64-bit on every target, including 32-bit microcontrollers (spec
  §4.1). Cost on 32-bit cores is noted in [09](performance.md).
- Record and tuple fields are placed in order of descending alignment, ties
  in declaration order ([15 §15.3.1](target-and-abi.md#1531-rules), L4); the
  layout is not observable from Verse.
- `T?` uses a niche when `T` has a spare bit pattern (pointers, `Bool`,
  `Char`, enums with spare discriminants); otherwise a tag byte.
- Variant types use a tag plus the largest payload, with niche packing for
  single-payload variants.
- `Text` is a UTF-8 buffer with a small-string form: up to 22 bytes held
  inline in a 24-byte header on 64-bit targets, and up to 10 bytes in a
  12-byte header on 32-bit targets ([15 §15.3.3](target-and-abi.md#1533-runtime-owned-types-text-bytes-list-map-set);
  assumption, benchmarked in M1). The header also caches whether the text is pure ASCII, and its
  character count when known, to make `len()` and indexing cheap (see
  [09](performance.md), risk P1).
- `List<T>` is `(ptr, len, cap)`, contiguous. `Map` preserves insertion
  order: an indices table plus a dense entries array (the "compact dict"
  design). `Set` uses the same structure.
- Recursive variants are boxed by the compiler (spec §4.10 point 3).
- Trait objects are `(data ptr, vtable ptr)`.
