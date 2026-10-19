# 14. Bytecode and the tier-0 VM

This document specifies tier 0 of [04 §4.1](backends-and-targets.md#41-tiers):
the register-based bytecode and the virtual machine that runs it. It consumes
mono MIR ([13](mir-specification.md)); it contains no ownership logic, no
generics and no trait resolution.

Status: **draft 3**. Labels follow the [README](README.md).
Every number is a **target** or **assumption**; nothing has been measured. Open
items at the end are **proposals**.

## 14.1 Role

1. **Reference semantics** (**decision**, [ADR-0008](../decisions/0008-tiered-execution.md)).
   Where a compiled tier and the VM disagree, the VM is presumed right until the
   spec says otherwise. It is the oracle of [10 §10.3](testing-and-verification.md#103-differential-testing).
2. **Script mode.** `verse run`, `verse test` and `verse repl` run here. Compile
   cost is lazy and per function ([14.6](#146-calls-and-lazy-instantiation)).
3. **Debug target.** Every debug session runs in the VM, live or by replay of a
   recording made by any tier ([08 §8.2](debugging-and-mcp.md#82-architecture)).
4. **Identical behaviour.** Same output, same panics, same `Cleanup` order,
   same task events as the compiled tiers, for a given seed ([14.12](#1412-differential-testing-contract)).
5. **Not goals.** Peak speed, a stable bytecode format for third parties, and
   multi-threaded execution in 0.1 ([14.14](#1414-open-questions-proposals)).

Pipeline: mono MIR, verifier ([13.5](mir-specification.md#135-verifier-rules)),
bytecode compiler (`verse-bc`), a **proto** per function, interpreter loop.
A **bundle** (`.vsb`, [04 §4.4](backends-and-targets.md#44-linking-and-artifacts)) is the serialized
set of protos plus tables; it carries the compiler version, the pointer width
of the host that made it, and is rejected on a mismatch of either (assumption).

## 14.2 Design rules

- **Typed instructions, no run-time type dispatch** (04 §4.1). The operand
  width and kind are in the opcode.
- **No tags in registers.** A slot holds raw bits. Names, types and ownership
  states for the debugger come from side tables ([14.3](#143-value-representation-and-call-records)), never from the slot.
- **Panics, bounds and overflow are control flow already in MIR.** The VM
  adds no policy ([14.5](#145-checked-arithmetic-bounds-and-division)).
- **Messages are data.** The VM never builds user wording; the panic record
  carries a kind and the facts of [13 §13.4](mir-specification.md#134-statements-and-terminators), rendered with the text and
  code of [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics) and the words of `events.toml`.

## 14.3 Value representation and call records

**Slots.** A register is an 8-byte **slot**. Operands name slots relative to
the **base** of the current call. A scalar takes one slot; an aggregate takes
`ceil(size / 8)` consecutive slots laid out exactly as the mono `layout` says
(byte offsets inside the slots), so a field is read with a frame-relative load
at a compile-time offset. `Int`, `Float` and borrows are one slot; `Text`,
`List<T>` and `Map` headers are 3 or more (03 §3.5, assumption).

**Borrows** are host addresses held in a slot. A `ref` is `REF a, b`: the
address of slot `b`. Stacks never move (below), so a borrow stays valid for its
live range (V9).

**Task stacks** are chains of **segments** that are never reallocated or moved.
A call checks that the callee's `frame_slots` fit in the current segment and
else allocates the next one. Initial segment: 1 KiB (assumption). A task that
cannot grow (limit `--max-stack`, default to be tuned) panics with
`stack_overflow` (R0004, "recursion is too deep in task N", [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics);
[05 §5.3](runtime.md#53-tasks-and-scheduling)); the VM supplies the fact `n`.

**Frame layout** of one call, from the base: `[ return place | parameters | named
locals | temporaries ]`, where the return place is MIR `_0` and the parameters
are `_1...`. A caller builds the arguments in a **window** at the top of its
own frame; the callee's base is the window start, so arguments are not copied
and the result lands in the caller's window.

**Call records** live in a separate vector per task (not in the slots), 16
bytes each (target): `proto: u32, return_pc: u32, base: u32, seg: u16,
flags: u16` (flags: cleanup-running, unwinding). A task also keeps a **cancel
mask** depth ([14.8](#148-unwinding-and-cleanup)). They are the VM's internal
"frame"; users and tools see them as **calls** (08 §8.7 says "call" and "stack",
not "frame").

**Debug-only state.** In a debug session each proto has a local table (name,
slot range, type id, mode word, bound-at pc) and each task keeps a **shadow
init bitmap**, one bit per local, updated by `MOV`/`DROP` through the event
table. The debugger derives `owned`, `moved`, `view`, `exclusive`, `dropped`
and `not_yet_bound` from these ([13.7](mir-specification.md#137-debug-info-and-source-spans)). Non-debug runs have neither.

**Side tables per proto**: constant pool, span table (`pc -> span id, dbg id`),
event table (`pc -> event word, name, destination`), exception table
([14.8](#148-unwinding-and-cleanup)), local table, `caps` bitmask.

## 14.4 Instruction set and encoding

**Words** are 32 bits, little endian. Forms: `ABC` = `op:8 a:8 b:8 c:8`;
`AI` = `op:8 a:8 imm:16`; `J` = `op:8 off:24` (signed, in words). When any
slot index is 256 or more the compiler emits `WIDE` (opcode `0xFF`) before the
instruction: the 8-bit fields are ignored and one extension word per two
operands follows, each operand 16 bits. A function therefore has at most 65,536
slots (512 KiB); a bigger frame is a compile error that names the local
(assumption: no real program needs it; heap boxing is the fallback).
Opcodes fit in one byte (target: fewer than 200 after size specialisation).
Some opcodes are followed by one `J` word (marked **+J**): a fused
check-and-branch that costs one dispatch.

| Group | Instructions | Operands | Semantics | Panics |
|-------|--------------|----------|-----------|--------|
| Move | `MOV`, `MOVN`, `LDI`, `LDK`, `ZERO` | `a b`, `a b n`, `a imm16`, `a k`, `a n` | copy 1 or n slots; load small or pooled constant. MIR `move` and `copy` both compile here; they differ only in the event table | none |
| Address | `REF`, `LEA`, `LDF8/16/32/64`, `STF8/16/32/64`, `LD*`, `ST*`, `LDN`, `STN` | `a b`, `a b off`, `a b off` | address of a slot, pointer plus offset, frame-relative and pointer-relative loads and stores of 1, 2, 4, 8 or n bytes | none (index checks are separate, V12) |
| Integer | `WADD`, `WSUB`, `WMUL`, `AND`, `OR`, `XOR`, `SHL`, `SHR`, `NEG`, `NOT` | `a b c` | 64-bit wrapping and bit ops; `SHL`, `SHR` mask the count to 0..63 | none |
| Checked | `ADD_CHK`, `SUB_CHK`, `MUL_CHK`, `DIV_CHK`, `REM_CHK`, `NEG_CHK`, `POW_CHK`, `CAST_CHK` (**+J**) | `a b c`; `CAST_CHK a b kind` | write the value to `a`, the overflow flag to `a+1` (the fixed `(T, Bool)` layout of [13.3](mir-specification.md#133-types-and-layout-facts)); when the flag is set jump by the `J` word, else skip it. `CAST_CHK` is MIR `checked_cast` (`narrow`, `f2i`, `int_char`); its jump goes to a `panic convert` block (R0012) | none: the jump goes to a MIR `panic` block |
| Compare | `EQ`, `NE`, `LT`, `LE`, `ULT`, `ULE`, `FEQ`, `FLT`, `FLE` | `a b c` | write a `Bool`; `ULT`, `ULE` read both as unsigned (MIR `ult`, `ule`; `ugt`, `uge` swap the operands); `ULT` is the bounds test of V12 | none |
| Fused branch | `JEQ`, `JLT`, `JLE`, `JZ`, `JNZ` | `b c` + `J` | compare and branch in one dispatch | none |
| Float | `FADD`, `FSUB`, `FMUL`, `FDIV`, `FREM`, `FNEG`, `I2F` | `a b c` | IEEE 754 binary64, no fused multiply-add, round to nearest even; MIR `f2i` is `CAST_CHK` | none |
| Control | `JMP`, `JT`, `JF`, `SWITCH`, `SWITCHS`, `TRAP` | `off24`; `a off`; `a k` | jump; dense jump table in the pool; sparse table with binary search; `TRAP` = `unreachable` or internal error | `TRAP` stops the VM with a bug report (exit status 3) |
| Enum | `GETDISC`, `GETDISCN`, `SETDISC` | `a b k` | read the case from a tag byte or a niche (decode data in the pool), or write it | none |
| Call | `CALL`, `CALLD`, `CALLI`, `CALLS`, `RET` | `a f16`; `a b k`; `a id16`; `a op16` | call instance `f` with window at `a`; through a vtable slot; runtime intrinsic; `sys` capability op ([14.9](#149-sys-capability-calls-and-the-capability-check)); return to caller | callee's |
| Drop | `DROP`, `DROPC` | `a glue16` | `DROP`: MIR statement `drop`, or terminator `drop` of a type without `cleanup`: run the glue of the type at slot `a`, which cannot unwind or park. `DROPC`: terminator `drop` of a type with `cleanup`: push a call record for `cleanup` (the loop does not recurse), raise the cancel mask, then drop the parts in the order of [13.4](mir-specification.md#134-statements-and-terminators) and continue at `next`. A drop with a flag compiles to `JF flag; DROP` | `DROPC`: panics inside `cleanup` follow [14.8](#148-unwinding-and-cleanup); it may park |
| Task | `SPAWN`, `SEND`, `RECV`, `WAIT`, `JOIN`, `SLEEP`, `LOCK`, `PAUSE`, `ABANDON` | `a f16 d`, `a b`, ... | MIR `spawn`, `yield` and `intrinsic task.abandon`; each may park the task ([14.7](#147-tasks-scheduler-and-seed)). `JOIN` is `WAIT` that reports a failed child as in spec §11.3 | cancellation arrives here when the mask is 0 ([14.8](#148-unwinding-and-cleanup)) |
| Panic | `PANIC`, `RESUME` | `kind imm16`; none | start unwinding with a kind and a pooled list of fact operands (slots or constants, in the order of the table in [13.4](mir-specification.md#134-statements-and-terminators)); continue unwinding at the end of a cleanup chain | by definition |
| Debug | `MARK`, `BREAK` | `span16`; `bp16` | source-line mark (debug protos only); breakpoint ([14.11](#1411-debug-hooks)) | none |
| Misc | `NOP`, `WIDE` | | padding; operand widening | none |

Compiler rules: a MIR `checked_*` followed by the mandatory `switch` on its
flag (V10) becomes one `*_CHK`+J instruction when the flag has no other use,
otherwise the unfused form writes both slots and a `JT`. The tuple `(T, Bool)`
produced by `checked_*` has the fixed layout *(value at offset 0, flag in the
next slot)*; the `layout` pass must not reorder it (stated in 03 §3.4 pass 12
and [13.3](mir-specification.md#133-types-and-layout-facts)).

## 14.5 Checked arithmetic, bounds and division

All three are explicit MIR control flow; the VM executes it.

- **Overflow.** `ADD_CHK a b c` computes with the host's overflow-detecting add;
  on overflow it jumps to the block that ends in `panic overflow add Int`
  (V10). `Int` is 64-bit on every host, including 32-bit ones, where the
  VM uses two-word arithmetic (assumption; cost noted in [09](performance.md)).
- **Division and remainder.** The zero test is its own MIR `switch` on
  `eq(d, 0)` (V11), compiled to `JZ`. `DIV_CHK` then handles only `Int.MIN / -1`.
  `/` truncates toward zero and `%` has the sign of the dividend (spec §3.8.4).
- **Bounds.** The check is `ULT i, len` (MIR `ult`; a negative index is a huge
  unsigned value) and a `JT` to the `panic index_oob` block (V12). When MIR
  has the pair `ge(i, 0)` and `lt(i, len)` instead, the two tests compile as
  they are written. Element access after it
  is an unchecked `LD*` through the list pointer. In debug builds the VM also
  keeps an assertion that an unchecked access is inside the allocation, so a
  verifier gap shows as an internal error rather than silent corruption.
- **Power.** `POW_CHK` only detects overflow; the negative-exponent test is a
  MIR `switch` to `panic neg_exponent` (V11).
- **Panic record.** `PANIC kind k` builds `{kind, facts, span, task, call
  list}`; the span comes from the span table of the faulting pc. The kinds,
  their codes (R0001 to R0012) and facts are those of
  [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics) and the table in
  13.4: an index panic carries `i`, `n`, `what` and `unit` (08 §8.6). Wording is
  not in the VM.
- **Order.** MIR order decides which panic is reported first. No VM or
  optimizer step may reorder two panic sites ([03 §3.4](ir-and-passes.md#34-pass-list), passes 5, 8, 11).

## 14.6 Calls and lazy instantiation

- **Before the first instruction**, `verse run` runs the reachability pre-pass
  of [02 §2.6](compiler-architecture.md#26-monomorphization): no bytecode, no
  optimizer, cached per module. It yields the reachable instance keys with their
  `caps` and `mayyield` ([13.6](mir-specification.md#136-generic-mir-and-mono-mir)), and the capability check of
  [14.9](#149-sys-capability-calls-and-the-capability-check) runs on it.
- **Direct calls** use a **function table**: `CALL a f` indexes it. An entry is
  `Unresolved(instance key)`, `Compiled(proto)` or `Compiling`. The first call
  of an unresolved entry runs the compiler service: generic MIR of the callee
  (cached), substitute the type arguments (instance key `(DefId, [TypeId])`),
  verify (V18 and V19 against the pre-pass facts), build the proto, store it in
  the table entry. Later calls pay one indirection and no check: code is never
  patched.
- **Keys are computed at the caller.** While compiling a proto, each call site's
  callee key is built by substituting the caller's type arguments into the
  callee's type arguments; trait calls are resolved then. The callee itself is
  not compiled until it is called. Code generation therefore scales with what
  runs; only the cheap pre-pass scales with what is reachable (02 §2.6).
- **`dyn` calls** go through vtables built when the trait object is first
  created; each slot is a function-table index, resolved on its first call.
- **Recursion** and mutual recursion work because the entry is `Compiling`
  only while bytecode is generated, not while it runs.
- **Calling convention** is [14.3](#143-value-representation-and-call-records): window, base, result at the window start. No tail calls (the spec promises none).
- **Cache.** Protos are cached under `.verse/cache/` keyed by `blake3(source,
  compiler version, flags, verse.api of imports)` ([02 §2.7](compiler-architecture.md#27-incremental-and-parallel-compilation)). The unit is the
  instance, not the module, because generic instances cross modules (04 §4.1 uses
  the same key; instances are grouped per module on disk for storage).

## 14.7 Tasks, scheduler and seed

(Decision, [ADR-0007](../decisions/0007-task-model.md): VM tasks are a fourth
*implementation* of the task semantics, not a lowering; the three lowerings are
native-code strategies. Observable behaviour is the same as the others.)

- **A task** owns a stack chain, a call-record vector, a pending-unwind state (the
  list of panics so far, [14.8](#148-unwinding-and-cleanup)), a `cancel_requested` flag and a cancel
  mask depth. Switching is a pointer swap in the loop; the loop
  is not recursive, so no native stack switching is needed. Ids come from one
  counter in spawn order; the main task is `t0` and runs the `<script>` function.
- **One worker thread** in 0.1 (proposal P14-1). `Guarded` interleavings are
  therefore scheduler choices, which is why recording them is cheap.
- **Switch points** are only the `yield` terminators that cannot proceed, a
  `DROPC` whose `cleanup` blocks, an explicit `pause`, and task exit. `spawn` appends the child to the run queue
  and does not switch. A `send` that does not block does not switch. No
  preemption by instruction count (P14-2). This is what native tiers can match
  in seeded mode.
- **Policy without a seed**: FIFO run queue; each wait queue (channel, task,
  lock) wakes in arrival order; timers fire in `(deadline, task id)` order.
- **Policy with `--seed N`**: at each switch point choose index
  `rng.below(len)` into the runnable tasks sorted by id. The generator and
  `below` are fixed by this document so every tier produces the same choices
  (proposal: PCG-XSH-RR 64/32 seeded through SplitMix64, `below` by rejection
  sampling). `verse-rt` implements the same policy in its single-worker seeded
  mode ([05 §5.3](runtime.md#53-tasks-and-scheduling)); the multi-core scheduler is used only when no seed or
  recording is requested.
- **I/O completions** arrive from a host reactor and are delivered only at switch
  points, in arrival order (logged, [14.10](#1410-record-and-replay)). A virtual clock capability makes
  timers deterministic in tests.
- **Structured end of life.** When a function exits, its unwaited tasks are
  waited (spec §11.3) by the compiler-emitted `JOIN`s of
  [13.4](mir-specification.md#134-statements-and-terminators); unwaited
  failures set the exit status (1, or 101 for a panic).
- **Deadlock**. No runnable task, no pending I/O completion, no
  timer (a virtual clock jumps to the next timer instead), and blocked tasks
  exist (a task blocked inside a `Cleanup` counts as blocked): raise R0008 ([16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics)), stop with reason `deadlocked` (08 §8.7), list what each
  task waits for (a task inside a cleanup reads "waiting inside the cleanup of
  {type}", fact `cleanup_of`), and exit with status **70**. A program that waits
  for a socket, stdin or a timer is not deadlocked.

## 14.8 Unwinding and Cleanup

- **Exception table** per proto: sorted `(pc_lo, pc_hi, cleanup_pc)` entries made
  from MIR `unwind bbN` edges. `unwind continue` and `unwind none` make no
  entry. It is read only while unwinding, so the success path pays nothing.
- **Panic.** `PANIC` appends the panic to the task's list, prints the report of
  [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics) to standard error at once (spec §8.5, §11.2), and sets the
  pending-unwind state to `panic`. **Cancellation** sets `cancel_requested` and
  wakes the task if it is parked; at its next `yield` executed with the cancel
  mask at 0 (including one it is parked in) the VM sets the state to `cancel`
  and takes the instruction's unwind edge. Nothing is printed for cancellation
  (spec §11.5).
- **Cancel mask**. The mask depth rises while a `DROPC` or any
  cleanup block runs, so a cancel request that arrives then only sets
  `cancel_requested` and takes effect at the first `yield` executed afterwards
  with mask 0. A cleanup always runs to completion, even if it blocks; only
  `exit(code)`, which ends the process at once and runs no `Cleanup`, ignores a
  blocked cleanup.
- **Walk.** For the current call, look up its pc in the exception table. Found:
  jump to `cleanup_pc`; the cleanup block runs as ordinary code (`DROP`,
  `DROPC`, flag writes, `JF`/`SWITCH` on flags, `ABANDON`, and blocking
  `yield`s, V15) until `RESUME`, which pops the call record and repeats with the
  caller's return pc. Not found: pop at once.
- **Nothing intercepts.** `otherwise`, `match` and `?` are ordinary MIR and
  never see the unwind state; only the task boundary turns it into a value
  (`TaskPanicked`, `Cancelled`) for `.wait()`. At the main task: panic gives
  exit 101.
- **Children**. `ABANDON t`, and the unwind edge of `WAIT` and
  `JOIN` (taken when the waiter is cancelled or panics), request cancellation of
  the child and hand its join to the scheduler, which finishes it before the
  unwinding task's own join completes. Only a child cancelled by an
  unwinding parent prints nothing for that cancellation (spec §11.3); an
  explicit `cancel()` keeps the §11.3 behaviour (an unwaited task that failed,
  including by cancellation, is reported and makes the exit non-zero). A child
  that fails with its own panic or error while unwinding is still reported on
  standard error and counts for the exit status.
- **Panic inside `Cleanup`**. The panic prints at once and is
  appended to the task's list. It unwinds the `cleanup` call as usual, up to the
  `DROPC` that started it; the glue then still drops the parts of that value and
  the outer unwinding goes on with the next cleanup, so no resource is skipped.
  The VM raises R0009 with the facts `x` (the binding) and `first` (the first
  panic). The task ends as `panicked` with the first panic as primary; the
  others are attached to `TaskPanicked`. The process is not aborted. The same
  holds when a cancellation is the first cause.
- **Cost.** Zero instructions on the success path. Wasm and embedded tiers use
  status codes instead ([05 §5.4](runtime.md#54-panics-and-unwinding)); the visible order of `Cleanup` calls is the same.

## 14.9 `sys` capability calls and the capability check

- **Calls.** `CALLS a op16` invokes operation `op` of a capability (the table of
  [05 §5.6](runtime.md#56-the-sys-capability-interface): `fs`, `net`, `clock`, `random`, `stdio`, `process`, `env`, ...). Blocking
  operations are `yield sys` in MIR and park the task until the provider
  completes. The provider is a Rust trait object per capability, so tests and
  replay substitute fakes (virtual clock, in-memory file system, a log reader).
- **Check before run** (P6-5; the **capability
  check** pass of [02 §2.2](compiler-architecture.md#22-stage-contracts)). Before the first instruction, the VM
  takes `required` (union of `caps(...)` over the instances the pre-pass of
  [14.6](#146-calls-and-lazy-instantiation) found reachable) and `granted` (what the host target provides
  minus `--deny` flags). If `required` is not a subset, nothing runs; the
  message names the first call chain, in the shape of 04 §4.3: "this program
  calls `std.net.get`, which this run does not allow" (V09xx). Exit status 2:
  an error before the program starts ([16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses)).
- **Run-time re-check.** `CALLS` tests the capability bit of the proto in
  debug builds only; a failure is a VM bug report (exit status 3), never a user panic.
- **Debugger.** `evaluate` runs only expressions the compiler proves pure: no
  `CALLS`, no `SPAWN` ([08 §8.9](debugging-and-mcp.md#89-safety)).

## 14.10 Record and replay

Design goal: a recording is proportional to nondeterministic events, and a
recording made by **any** tier replays in the VM ([08 §8.5](debugging-and-mcp.md#85-deterministic-record-and-replay)).

**Recorded:** capability results with their arguments' digest; scheduler choices;
wake-up order; the global order of synchronisation operations (`send`,
`recv`, `wait`, lock acquire, close); the order and size of standard output and
error writes (a print from two tasks is observable); random seeds; signals; the
seed and capability grants. **Not recorded:** pure computation, memory
addresses, hash seeds (fixed in record and replay mode).

**File `.vsrec`** (internal, unversioned for third parties):

```
file   = header , block*
header = "VSREC" 0x00 | rev:u16 | ptr_width:u8 | program_hash:[u8;32]   (blake3 of source, lock file, compiler version and flags)
         | seed_mode:u8 | seed:u64 | granted_caps:u32 | args_digest:u64
block  = 0x01 | n_events:varint | zstd( event* )
event  = kind:u8 | task:varint | seq_delta:varint | payload     (seq: global event ordinal from 0)
```

| Kind | Payload | Replay use |
|------|---------|------------|
| `Pick` | none | which task runs next |
| `Wake` | object, reason (timer, io, channel, wait, cancel) | wake order |
| `Spawn`, `Finish` | child id; outcome (ok, cancelled, or panicked with the list of panics: count, then kind and code of each) | task tree, `TaskPanicked` |
| `Sync` | op, object id, outcome | per-object operation order |
| `Cap` | capability, op, args digest `u64`, result (inline up to 64 bytes, else a blob block) | the value `CALLS` returns |
| `Out` | stream, byte count, digest | validates output; fixes cross-task print order |
| `Sig`, `End` | signal; exit status (0, 1, 2, 3, 70, 101 or the `exit` code) | delivery point; end of run |

The target of [08 §8.5](debugging-and-mcp.md#85-deterministic-record-and-replay) is under 100 bytes per event record,
**before** compression (zstd then shrinks it). It counts the record and its
inline payload; a `Cap` result of more than 64 bytes lives in a blob block and
is not counted. A **ring-buffer** mode keeps the last N events and the snapshot
taken before them.

**Replay.** The VM runs the program, recompiled from the same source at
optimisation level 0 (the hash does not cover the bundle, since the recorded
run may have used another level), with a
*log-driven* scheduler and providers: each switch point consumes the next
`Pick`/`Wake`; each `CALLS` consumes a `Cap`, checks capability, operation and
args digest, and returns the logged bytes with **no real I/O**; each output
write is checked against `Out`. A mismatch stops with a *divergence* report
(event number, expected, actual) and never guesses. A compiled tier records with
the same event kinds from its `debug` slice, taking `Sync` order from one atomic
counter that exists only in record mode (it does not change behaviour).

**Steps** (08 §8.5). A *step* is one executed MIR statement or
terminator of the mono MIR at optimisation level 0, which runs no pass except
the read-only `verify` ([13.7](mir-specification.md#137-debug-info-and-source-spans)). A fused instruction counts the items it fuses,
from the event table. Steps exist only in the VM; the log is indexed by event
ordinal, so a native recording needs no step counter, and every replay and
debug session runs at level 0 so that step numbers mean the same for all.

**Time travel** (08 §8.4, `debug_back`, `debug_goto`, `why_changed`):

- **Snapshots** copy all task stacks and call records, the VM heap arena, the
  scheduler and PRNG state, shadow bitmaps and the log cursor. The arena is
  mapped at a fixed address so pointers stay valid on restore (proposal P14-5).
  Taken every K steps (K is an assumption to tune; default ring of recent
  snapshots plus sparse older ones).
- **`debug_goto step`**: restore the nearest earlier snapshot, run forward with
  hooks off to the step, stop. **`debug_back`** goes to the previous step of the
  requested mode (`line`, `into`, `over`, `out`, `task`): restore the snapshot
  before the current step, run forward once while indexing marks and call and
  return events, then go to the predecessor. Cost is at most one snapshot interval of
  forward execution.
- **`why_changed`/`history`**: replay from a snapshot with a watch ([14.11](#1411-debug-hooks)) on the
  place and list writes.
- **Requirements**: `Set` insertion order, fixed hash seeds, no behaviour
  that depends on addresses, bit-identical floating point ([14.12](#1412-differential-testing-contract)).
- **Secrets**: recordings are owner-only files; `verse record redact` removes
  chosen `Cap` results, and replay then reports a divergence at the first one
  that was needed (08 §8.9).

## 14.11 Debug hooks

The VM has two instances of the dispatch loop, `run::<false>` and `run::<true>`
(a const generic), and switches at an instruction boundary when hooks are armed
or cleared.

| Hook | Mechanism | Cost when disabled | Cost when armed |
|------|-----------|--------------------|-----------------|
| Breakpoint (line, function, condition, hit count, log point) | first word of a statement-boundary instruction replaced by `BREAK bp`; the original runs out of line, as in native debuggers; the pc comes from the span table, so it works in protos without marks; conditions use the pure evaluator | none | one trap per hit |
| Break on event kind (`moved`, `overflowed`, `spawned`, `propagated`, ...) | the event table lists every pc of that kind; those are patched like breakpoints | none | one trap per matching pc |
| Step (into, over, out, line, task) | step flag read by the instrumented loop; `line` uses `MARK` | none | one flag test per instruction |
| `MARK` | present only in protos compiled for debugging (`debug_launch`, `verse run --debug`) | **not emitted**: zero | one dispatch and one load and branch per source line |
| Watch (a place) | the instrumented loop compares the watched bytes after each store; a watch on a call's local ends when that call returns (`watch_changed`) | none | one compare per store |
| Event stream and trace | instrumented loop reads the event table and writes events to the **separate** trace channel | none | per event |

Instrumentation must not change values, `Cleanup` order or scheduler choices;
the differential suite runs the VM in both loops. Target: debug-marked
protos run within 1.3x of unmarked on the micro benchmarks of [09 §9.2](performance.md#92-benchmark-suite).

## 14.12 Differential-testing contract

The VM is compared with Cranelift, LLVM and Wasm by `verse difftest`
([10 §10.3](testing-and-verification.md#103-differential-testing)) on every program of the conformance suite, with the optimizer on and off, with each
pass off when a mismatch appears, and with recording on and off. All tiers
share the same `verse-rt` code for text formatting, hashing, collections,
and the scheduler policy of [14.7](#147-tasks-scheduler-and-seed), so the test mainly checks code generation.

**Must be identical** for the same program, arguments, seed, virtual clock and
capability fakes:

| Observable | Detail |
|------------|--------|
| standard output and error bytes | including the full panic text of [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics), stack lines too: they come from the user-level call list below |
| exit status | 0; 1 (a top-level `?` or an unwaited task failed); 2 (errors in the source, before the program starts); 3 (a compiler bug, V9901 or V9902, also before it starts); 70 (deadlock); 101 (panic); or the program's own `exit` code ([16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses)) |
| panic record | `kind`, facts, span, task, and the **user-level** call list (inlined calls reconstructed from spans, 13.7); which panic comes first; the secondary panics of a failed `Cleanup` (R0009) |
| order of `Cleanup` and drops | including on panic and cancellation |
| task events | `spawned`, `finished`, `waited`, `sent`, `received`, `closed` in the same sequence for a given seed; without a seed, only the order each task and each object sees |
| capability calls | capability, operation and arguments, in order |
| `Float` results | bit for bit: IEEE 754 binary64, no fast-math, no implicit fused multiply-add; every NaN is the canonical quiet NaN `0x7FF8000000000000` wherever bits can be seen (`to_bits`, hashing, byte serialization, `Show`). A hash of a float, where one exists, treats `-0.0` as `0.0` (they compare equal); this binds only internal hashing and a library wrapper such as `OrderedFloat`, because `Float` is not `Hash` (spec §9.4); `to_bits` keeps the sign of zero; `Show` prints `NaN` for every NaN and `-0.0` for negative zero |
| collection order | `Map` and `Set` iteration is insertion order; `Text` indexing by characters |

**May differ**: timing; addresses; memory use; the depth at which
`stack_overflow` fires; the allocation that hits `--max-memory`; the payload and
sign of a NaN while it is only computed with, never exposed; the order of unsynchronised output between tasks when no
seed is given.

**Replay leg.** A native run recorded with `--record`, replayed in the VM, must
reproduce every item in the table; a divergence is a bug in the recorder, the
VM or a tier. The harness reports the first differing observable, then bisects
passes (`--no-opt-pass=NAME`) and loops (marks on and off, hooks on and off).

## 14.13 Performance targets

All rows are **targets**, not measurements. Rows marked 09 repeat
[09 §9.1](performance.md#91-what-we-measure); the others are proposals to be
adopted into 09.

| Metric | Target | Source |
|--------|--------|--------|
| `verse run hello.vs`, warm cache, pre-pass included | under 20 ms wall time | 09 |
| reachability pre-pass, 10,000 reachable functions, cold module cache | under 50 ms; warm: under 5 ms | proposal |
| tier 0 on the benchmark suite | within 30x of optimized native, aiming at 10x to 20x | 09 |
| instructions executed per MIR statement, after fusion | 1.2 or fewer on average | proposal |
| bytecode size | 8 bytes or fewer per MIR statement on average | proposal |
| first-call cost: instantiate, verify and compile a 50-statement function | under 100 microseconds | proposal |
| VM `spawn` plus `wait` of an empty task | under 1 microsecond | proposal (09's 300 ns is for M4 native) |
| memory per idle VM task | under 1.25 KiB, so 1 million tasks fit in about 1.3 GB, inside 09's 2 GB budget | proposal |
| recording overhead of a compiled program, compute bound | under 5% | proposal |
| recording size | under 100 bytes per event record, before compression | 08 |
| time travel: `debug_goto` on a small heap | under 250 ms | proposal |
| debug-marked protos versus unmarked | within 1.3x | proposal |

## 14.14 Open questions (proposals)

| # | Question | Recommendation |
|---|----------|----------------|
| P14-1 | VM on one worker thread in 0.1? | Yes. Parallel tier-0 tasks add nothing the native tiers lack and break determinism. Revisit with M4 results |
| P14-2 | A task that spins on `is_done()` never yields and starves the others | `is_done` and similar non-blocking polls act as `pause` once every N calls, logged as a `Pick`; no instruction-count preemption |
| P14-3 | Capability check needs exact reachability (P6-5) but 02 §2.6 says script mode instantiates lazily | **Resolved** by the decision in [02 §2.6](compiler-architecture.md#26-monomorphization): `verse run` does a cheap reachability pre-pass (no code generation, no optimization), then the capability check, then lazy instantiation |
| P14-4 | NaN payload and sign are hardware dependent, but 10 §10.3 says floats are bit-identical | **Closed**: canonicalise every NaN where bits can be observed (`to_bits`, hashing, byte serialization, `Show`); arithmetic is untouched |
| P14-5 | Snapshots: fixed-address arena, or `fork()` where it exists | Fixed-address arena in the VM (works on Wasm hosts and under tests); `fork()` stays an optional accelerator |
| P14-6 | Should the VM treat `DROPC` of a `Cleanup` type as a possible switch point? | **Closed**: yes, with the cancel mask of 14.8; 13.10 M7 |
| P14-7 | Exit status and message for a deadlock | **Closed**: R0008 and exit 70. A panic inside `Cleanup` is R0009, task-level, no process abort |
| P14-8 | `.vsb` portability across pointer widths | Not portable in 0.1; a bundle is per host pointer width |
