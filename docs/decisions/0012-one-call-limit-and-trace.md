# ADR-0012: One call limit and one trace on every tier

Status: accepted (2026-10-19).

## Context

`verse app.vs` runs a script in the VM, then natively from its third run, and
`verse build --release` makes a native binary (04 §4.1). A program that
recurses without end must stop the same way in all three, or the switch from
the VM to native code changes what the user sees.

Two choices decide whether that holds. A limit on stack size fires at a depth
that depends on frame sizes, and frame sizes differ between the VM, Cranelift,
LLVM, Wasm and each `opt` level. And a panic report builds its stack lines
from the debug map, so a release build without that map could not say where
it panicked.

## Decision

- **The limit is a count of calls, not a stack size.** Each task counts the
  calls of `recursive` functions it has in progress. A function is `recursive`
  when it is on a cycle of the call graph of the reachable instances. A direct
  call has an edge to its callee. A call through a function value has an edge
  to every reachable instance whose function value is created with the
  signature of the called value's type. A call through a `dyn` slot has an
  edge to the method of that slot in every reachable impl of its trait. AOT
  monomorphization computes the attribute before the optimizer runs, and the
  script-mode pre-pass computes it by the same rule, so every tier and every
  `opt` level counts the same calls (13 §13.3, §13.6).
- **Only `recursive` functions count.** Any other call chain has a depth fixed
  by the call graph. Counting only these functions makes the count independent
  of inlining and of leaf functions, and costs nothing in the other functions.
  No pass and no backend inlines a `recursive` function.
- **The default limit is 10,000 calls, on every target.** `--max-calls` and
  `[run] max_calls` change it; `verse build` writes the value into the
  executable. The VM counts at each call of a `recursive` proto; compiled code
  counts in the prologue that already holds the stack check (15 §15.6.4).
- **The stack reservation is the backstop.** `[tasks] max_stack` is the
  stack reserved per task. A task that fills its stack before it reaches the
  limit panics with the second form of R0004, "task {n} ran out of stack in
  `{f}`". The stack check, its headroom and the guard page guard it.
- **Unwinding keeps the count.** A frame left by unwinding keeps its call in
  the count. The task ends when its unwinding reaches the catch point, so the
  count is not used after that. This rule costs nothing on the success path
  and gives the same count on every unwind mechanism.
- **The trace is in every build.** The trace map, the part of the debug map
  that the user-level call list needs, is kept when `debug = false`
  (15 §15.7.1). Native targets walk the stack with their unwind tables. Wasm
  and `status-code` targets cannot, so each function that makes calls keeps a
  site word in its frame and stores the call site there before each call.
- **The trace is folded.** A block of 1 to 8 lines that repeats right after
  itself is printed three times, then one line gives the number of further
  repetitions (16 §16.6). The VM and every compiled tier print the same bytes.

The R0004 panic reads:

```
panic: recursion is too deep in task t0: 10000 calls in progress, the limit, in `sum_to`
```

## Alternatives

- **Keep the stack-size limit, and accept a different depth per tier.** This
  keeps the switch from the VM to native code visible to the user, and a
  release report has no trace.
- **Count calls in the VM only.** The VM and native code still stop at
  different depths.
- **Count every call.** Inlining removes calls in native code that the VM
  makes, and a leaf function has no prologue to count in. The count would
  change with the backend and the `opt` level.
- **A default of 1,000,000 calls.** The hosted reservation is 8 MiB, and 8 MiB
  less the 16 KiB headroom holds a million calls only of frames up to 8 bytes.
  With 16-byte frame alignment no frame is that small, and a million frames of
  64 bytes need 61 MiB. So on a hosted target the stack would always fill
  first, and native code would stop at another depth than the VM. 10,000
  calls fit in the same reservation with frames up to 832 bytes.
- **Python's limit of 1,000.** Python counts every interpreter call against
  `sys.getrecursionlimit()`, 1,000 by default. That is too low for a recursive
  walk of an ordinary tree or a parsed document.
- **Print site numbers on microcontrollers and let the toolchain render the
  trace.** This saves the flash of names, but a person reading a UART console
  sees numbers. The trace map is small (Consequences), so every target keeps
  it.
- **Frame pointers on `status-code` targets.** A walk by frame pointers needs a
  register in every function and still needs a map from return addresses to
  call sites. Wasm cannot read its own call stack at all. One site word per
  frame works on both.

Prior art: Python stops recursion at a count, not a stack size, and folds
repeated traceback lines as `[Previous line repeated 996 more times]`. Rust
finds a stack overflow with a guard page and aborts with no trace, which is
the outcome this decision avoids.

## Consequences

- A call of a `recursive` function costs at most 4 more instructions in the
  prologue and 3 on the return path than the stack check alone, on x86-64 and
  AArch64, with a count of calls left in the TCB (cost target, measured in
  M2). Other calls cost nothing more.
- A `recursive` function is never inlined, and the rule of 13 §13.3 makes
  generic code `recursive` through the lambdas it calls. A standard
  higher-order function such as `map` becomes `recursive` whenever a reachable
  lambda that fits its function-value slot calls it, and then it is not
  inlined into the loop. This works against 05 §5.5 and 09 §9.3, where
  iterator chains inline into loops. M2 measures it with the Chains
  benchmarks of 09 §9.2 run twice, once with a lambda that calls `map` in the
  program and once without.
- The goal of one depth on every tier holds on hosted stackful targets for
  frames under about 830 bytes (15 §15.6.4). It does not hold on `cooperative`
  targets, whose 4 KiB shared stack on Cortex-M fills long before 10,000
  calls, or on small Wasm shadow stacks: 512 KiB gives 52 bytes per call.
  There a compiled task prints the second form of R0004, which the VM never
  prints.
- Every build is larger by the trace map: at most 4 bytes per call site and
  per parent span of an inlined call, 4 bytes per function for its frame size
  on Wasm and `status-code` targets, and one copy of each function and file
  name (assumption, measured in M7 inside the 16 KB blinky budget). Native
  targets keep their unwind tables under the `abort` profile too (assumption:
  at most 10% of the code size, measured in M7). No profile and no target leaves it out. The report
  is printed while the walk runs, so a `--no-heap` build needs no memory that
  grows with the depth.
- On Wasm and `status-code` targets each call also stores a site word:
  3 Wasm instructions (`local.get`, `i32.const`, `i32.store`) and a shadow
  frame in a function that has none otherwise, or 2 Thumb-2 instructions
  (`mov`, `str`) on Cortex-M (assumption, measured in M3 for Wasm and M7 for
  Cortex-M).
- The one run flag for depth is `--max-calls`. No flag sets the stack size;
  `[tasks] max_stack` of the target file sizes the reservation.
- The differential tests compare the stack lines of every build, and the call
  at which the first form of R0004 fires. Whether a compiled task stops at the
  limit or at a full stack, and with it the R0004 text and stack lines, may
  differ (14 §14.12).
