# 12. Open questions

Decisions that touch **the meaning of the language** or its standard library.
The implementation will not decide them silently. Each has the problem,
alternatives, and a recommendation. Status is `open` until the language owners
rule.

## Q1. First-party domain libraries and their names

**Problem.** The owner wants finance, machine learning, servers and GUI
support, optional and shaken out of builds. Spec §15.2 lists only general
modules.

**Options.**
A. Everything under `std.*` (`std.ml`, `std.gui`), shipped with the toolchain.
B. First-party packages with their own names (`import Tensor from ml`),
   versioned with the toolchain, installed on demand through the package manager.

**Recommendation: B.** `std` stays small and portable to every target
(including microcontrollers). Packages are optional by construction, can
release on their own schedule, and use the existing package syntax (§16.3).
Shaking still applies because it works on reachable code, not on package
boundaries.

**Ruling (Picasso, accepted in the room):** B. Finance, ML, web and GUI ship
as optional packages, not inside `std`. 06 says the same.

**Status:** ruled.

## Q2. Foreign function interface

**Problem.** The spec has no way to call C, or to be called from C. GUI,
ML (BLAS, GPU), databases and microcontroller drivers need it. Grep of the
spec for `extern` and `foreign` finds nothing.

**Options.**
A. No FFI in 0.1. Native capabilities come only from runtime slices and
   plugins ([07](07-plugins.md)).
B. A minimal `extern` declaration form restricted to the C ABI, with
   ownership rules (`take` for owned pointers, borrowed for views).
C. FFI only through Wasm components.

**Recommendation:** A for 0.1.

**Ruling (Picasso, accepted in the room):** `extern` declarations get their
own spec section, marked "0.2, needed for `bindgen`", so 0.1 does not carry
them. Semantics as drafted by Picasso: an `extern` function is callable only
inside a `with` block that holds the matching `sys` capability, and it takes
and returns only plain data types. Michelangelo writes the readability
constraints (`reviews/extern-readability.md`).

**Implementation consequence.** The `bindgen` plugin kind ([07](07-plugins.md))
ships only with 0.2. The reachability analysis already treats `extern`
functions as capability uses, so tree-shaking stays exact
([ADR-0004](adr/0004-whole-program-reachability.md)).

**Status:** ruled. Spec text pending (0.2).

## Q3. Complexity of `Text` indexing

**Problem.** `Text` is indexed by characters (§4.1). On non-ASCII text that
makes indexing and `len()` O(n) unless cached. Users reading the spec may
assume O(1).

**Recommendation.** State in §4.1 that indexing is O(1) for ASCII text and
O(n) otherwise (with a cached length), and offer `text.bytes()` (already
there) for hot loops. No semantic change.

**Ruling (Picasso, accepted in the room):** `Text` stays character-indexed and
§4.1 states the cost. A documented byte-view API gives fast access to
non-ASCII text. This is the same as the `text.bytes()` mitigation in
[09](09-performance.md), P1, now with a documented API.

**Status:** ruled. Applied in spec round 7.

## Q4. `Set` iteration order

**Problem.** `Set` is "unordered" (§9.4). Programs that iterate a set can
print different output between runs, between tiers, or between machines.
That breaks reproducible builds in tests, differential testing, and
record-and-replay debugging (M6).

**Options.**
A. Keep "unordered" and randomize per run (security against hash flooding,
   but breaks replay).
B. Define iteration as insertion order, as `Map` already is (§9.2).
C. Keep "unordered" in the spec but guarantee one deterministic order per
   program version (an implementation promise, not a language promise).

**Recommendation: B.** Costs one dense array. One rule for both collections.
Hash flooding is handled by a keyed hash that the runtime seeds
deterministically in debug and replay mode.

**Ruling (Picasso, accepted in the room):** B. `Set` iterates in insertion
order, the same as `Map`. Picasso updates spec §9.4 and logs it in §21.

**Implementation consequence.** `Set<T>` uses the compact-dictionary layout of
`Map` (an index table plus a dense entry array), so `Set` costs about 4 bytes
per element more than an unordered open-addressing table (assumption, to be
measured in M1). Budgeted in [09](09-performance.md), P8. Hash seeds are fixed
in record and replay mode.

**Status:** ruled. Applied in spec round 7.

## Q5. Panic strategy: `Cleanup` and unwinding on targets without unwinding

**Problem.** Spec §8.5 and §11.5 say `Cleanup` runs on panic and
cancellation. Some targets (small microcontrollers) cannot afford unwind
tables, and an `abort` profile skips cleanup.

**Recommendation.** Keep the spec as is. Use status-code propagation on
those targets, which preserves cleanup (cost: a branch per fallible call). An
`abort` profile is allowed only for programs where the compiler can prove no
`Cleanup` type is live at any panic site, otherwise the build fails. This
needs the owners' agreement because it makes a profile that can reject
programs.

**Ruling (Picasso, accepted in the room):** recommendation accepted as written.

**Status:** ruled.

## Q6. Debug-visible names and messages

**Problem.** Michelangelo's rule: debugger output uses the same names and
wording as compiler messages. The protocol needs a controlled vocabulary.

**Recommendation.** One vocabulary file (`vocab/events.toml`) lists event
names and message templates; diagnostics and debug events both reference it.
Michelangelo owns the wording; the compiler enforces that every event type
has an entry.

**Ruling (Michelangelo):** accepted as recommended. `vocab/events.toml` is the single source for event names and message templates. The keys, templates, naming rules and the "don't say" word list are in [08](08-debugging-and-mcp.md) 8.7. The compiler fails the build when an event or diagnostic has no entry, and golden wording tests fail on a banned word.

**Status:** ruled.

## Q7. Debug builds versus release builds and semantics

**Problem.** Script and compiled modes must behave identically (an agreed
rule). Debug builds add instrumentation (trace points, leak checks) that must
not change observable behavior.

**Recommendation.** Instrumentation may only add output on a separate channel
(the trace stream), never change values, ordering of `Cleanup`, or scheduler
decisions unless record mode is on. The differential suite runs both with and
without instrumentation.

**Ruling (Picasso, accepted in the room):** recommendation accepted as written.

**Status:** ruled.

## Q8. Package namespace for first-party libraries and plugins

**Problem.** How do users tell a first-party package from a third-party one,
and a plugin from a library?

**Recommendation.** Plugins are declared in a separate `[plugins]` section of
`verse.toml` and never in `[dependencies]`; first-party packages are normal
dependencies with a documented reserved name list.

**Ruling (Picasso, accepted in the room):** recommendation accepted as written.

**Status:** ruled.

## Q9. `Int` width on 32-bit targets

**Problem.** `Int` is 64-bit everywhere (§4.1). On Cortex-M each operation costs
two or more instructions.

**Recommendation.** Keep 64-bit semantics. Rely on range analysis and on the
sized integer types in `std.num` for hot code. Document the cost in the
embedded guide.

**Ruling (Picasso, accepted in the room):** recommendation accepted as written.

**Status:** ruled.

## Q10. Compile-time evaluation

**Problem.** Constant folding is an optimization today. Users in embedded and
finance code often want guaranteed compile-time tables.

**Recommendation.** No new language feature in 0.1. The optimizer folds pure
constants; revisit if real programs need guarantees (it would be a language
change, so it goes to the owners).

**Status:** deferred.

## Q11. Transforming plugins (plugins that rewrite MIR)

**Problem.** A plugin that rewrites mono MIR could add optimizations or
instrumentation, but it can change what a program does, which breaks the rule
that a plugin never changes what a valid program means ([07](07-plugins.md)).

**Ruling (Picasso, accepted in the room):** transforming plugins stay out of
0.1. Only the four kinds in 07 ship (`lint`, `generator`, `bindgen`, `tool`). A transforming kind may be added later by a spec amendment.

**Implementation consequence.** The optimizer pass list ([03](03-ir-and-passes.md),
3.4) is closed to plugins. The mono MIR serialization is internal and
unversioned for third parties in 0.1, which removes a stability commitment.

**Status:** ruled.

## Q12. Stdlib packaging proposals (06 §6.12, P6-1 to P6-7)

**Ruling (Picasso, accepted in the room):** all seven proposals are accepted.

| # | Ruling | Implementation consequence |
|---|--------|----------------------------|
| P6-1 | First-party package names are `finance`, `ml`, `web`, `gui`, `iot`; `std.net.serve` stays in std. | The resolver reserves the names; 07's example uses `gui`. |
| P6-2 | `verse.toml` has a `[library]` section: modules, capabilities, per-target availability, size budget. | The capability check and `--size-report` read it. |
| P6-3 | An import whose names are never used is a warning and adds no code. | A diagnostic with a code and a fix edit; reachability already ignores it. |
| P6-4 | `verse build --size-report` and `verse why <item>` join the §16 tool table. | The `size_report` tool in [08](08-debugging-and-mcp.md) calls the same query. |
| P6-5 | Capabilities are checked before running. | A pass called the **capability check** runs right after exact reachability from the program's entry points ([02 §2.2](02-compiler-architecture.md#22-stage-contracts), which owns its V09xx errors). In script mode `verse run` first does a cheap whole-program reachability pre-pass (no code generation, no optimization; [02 §2.6](02-compiler-architecture.md#26-monomorphization)), so a missing capability fails the same way in every tier before tier 0 starts, while instances are still compiled lazily. |
| P6-6 | New capabilities `surface` and `gpu`, slices `sys-surface` and `sys-gpu`. | Rows are in [05 §5.1](05-runtime.md#51-feature-slices); the `sys` interface entries are still to be specified (M5). |
| P6-7 | Per-target code is chosen by files and manifest data, never by `cfg`-style syntax. | Target selection lives in the build graph, not the parser. |

**Status:** ruled.

## Q13. A panic inside `Cleanup` while unwinding

**Problem.** `cleanup(mut self)` must not fail (spec §13.1), but a bug can still
panic inside it, possibly while another panic or a cancellation is unwinding
the task. The spec does not say what happens.

**Options.**
A. Abort the process (as Rust does for a panic in a destructor during unwinding).
B. The second panic replaces the first.
C. Report both panics; the task ends as panicked with the first one as primary;
   the remaining `Cleanup` calls of that task still run.

**Recommendation: C.** The process is not taken down for one task's bug (the
task boundary is where panics become values, §11.2), the first panic is never
lost, and no resource is leaked just because one cleanup failed. The secondary
panic is printed at once like any panic and is attached to `TaskPanicked`
(`trace()` shows both). A panic in a cleanup that runs during cancellation is
reported the same way. In MIR terms, cleanup blocks stay `nounwind` for the
verifier ([13.5](13-mir-specification.md#135-verifier-rules), V15); the runtime
handles the second panic. (13.10 M9 and 14.8 currently say "abort the process";
they follow this answer once it is ruled.)

**Status:** open.

## Q14. May `Cleanup` block?

**Problem.** Closing a buffered `File` flushes, which is I/O and can block the
task. State-machine and cooperative targets must know whether a `drop` can
yield ([13.10](13-mir-specification.md#1310-changes-against-03-and-open-items) M7). Cancellation arriving *during* a cleanup would also unwind a
cleanup.

**Options.**
A. Cleanup may not block. Calling a blocking function from `cleanup` is a compile
   error on every target (uniform, but `File`'s own cleanup could not flush).
B. Cleanup may block on native targets only, and is an error on
   state-machine targets (differing behaviour between tiers).
C. Cleanup may block everywhere, and **cancellation is masked** while a cleanup
   runs: a cancel request that arrives during cleanup takes effect at the first
   blocking point after the cleanup finishes.

**Recommendation: C.** A and B make resource types less useful or make a
program valid on one tier and invalid on another, which breaks the rule that
tiers behave identically. With C, `task-lower` treats a `drop` of a type that has
`Cleanup` as a possible yield (`drop` becomes a terminator with an unwind edge,
13.10 M7), and the cancellation mask keeps "cleanup runs to completion" true.
The cost is a mask flag per task and a blocked cleanup that cannot be cancelled;
a timeout belongs in the resource's own API.

**Status:** open.

## Q15. Drop order of fields and elements

**Problem.** Locals drop in reverse declaration order (spec §4.10, §13.3), but
the order in which the fields of one record, the elements of a list or the
entries of a map are dropped is unspecified. It is observable when two fields
have `Cleanup`.

**Options.** (A) declaration order; (B) reverse declaration order; (C)
unspecified.

**Recommendation: A**, declaration order for fields and tuple elements, index
order for lists, insertion order for maps and sets (the same order users see
when iterating). It is the order of construction, which needs no explanation,
and (C) would make differential testing depend on an undefined order. A type that
needs another order implements `Cleanup` and drops its parts itself.

**Status:** open.

## Q16. Unwaited child tasks when the parent unwinds

**Problem.** Spec §11.3: a function waits for its unwaited tasks before it
returns. It does not say what happens when the function does not return
normally (a panic, or the parent is cancelled), nor whether the wait can hang
behind a long child.

**Options.** (A) wait for the children without cancelling; (B) cancel the
children, then wait for them; (C) detach them.

**Recommendation: B.** Structure is kept (a task never outlives the code that
started it, §11.3) and a panicking parent does not block on a child that may
run for minutes. Each child unwinds with `Cleanup` and is waited before the
parent's own unwinding completes; their failures are reported on standard
error and count for the exit status as in §11.3. This is what the MIR
`intrinsic task.abandon` ([13.10](13-mir-specification.md#1310-changes-against-03-and-open-items) M8) does: request cancellation and hand the join to
the runtime. (C) is rejected because it breaks structured lifetimes.

**Status:** open.

## Q17. Exit status on deadlock

**Problem.** If every task is blocked and nothing can wake them (no pending I/O,
no timer), the program hangs forever. §12.5 lists the exit statuses 0, 1, 101
and `exit(code)`, and says nothing about this case.

**Options.** (A) hang (as most runtimes do); (B) stop with an error report and
status 1; (C) stop with an error report and a distinct status.

**Recommendation: C, status 70** (`EX_SOFTWARE` in `sysexits.h`: an internal
software error), so scripts and CI can tell a deadlock from a failed `?` (1) and
from a panic (101). The report is the debugger's `deadlocked` stop in text:
each blocked task and what it waits for ([08 §8.7](08-debugging-and-mcp.md#87-vocabulary-and-wording-owned-by-michelangelo)). Detection is "no runnable task,
no pending completion, no timer", which every tier's scheduler can compute.
Programs waiting on external input (a socket, stdin, a timer) are not
deadlocked. (14.7 and P14-7 currently say status 1; they follow this answer.)

**Status:** open.

## Q18. `Float` bit-identity and NaN payloads

**Problem.** 10 §10.3 promises bit-identical floating point on every tier. IEEE
754 leaves the sign and payload of a NaN produced by an operation to the
hardware, and x86-64, AArch64 and RISC-V differ.

**Options.** (A) promise bit-identity for everything except NaN, unspecified;
(B) canonicalize after every float operation (a check on a hot path); (C)
canonicalize only where the bits can be observed.

**Recommendation: C.** Every NaN becomes the canonical quiet NaN
`0x7FF8000000000000` at the points that expose bits: `to_bits`, hashing,
byte serialization (`json`, `fs`, `net`) and `Show`. Arithmetic is untouched, so
speed is unchanged; all other results stay bit-identical, with no fast-math and no
implicit fused multiply-add (10 §10.3). Equality of NaNs is already false, so
no program can observe the difference otherwise.

**Status:** open.

## Q19. Definition of a debugger "step"

**Problem.** 08 uses `step` as a position in a run (`debug_goto`, stop records,
event streams) without defining it, and a compiled tier cannot count what a VM
counts.

**Options.** (A) one source line; (B) one MIR statement or terminator; (C) one
VM instruction.

**Recommendation: B, measured in tier 0 on unoptimized mono MIR.** A fused VM
instruction counts the MIR items it fuses ([14 §14.10](14-bytecode-and-vm.md#1410-record-and-replay)); a line (A) is too
coarse for `why_changed`, and an instruction (C) changes with bytecode design.
Steps exist only in the VM. Recordings are indexed by the event ordinal, so a
native recording needs no step counter, and a replay always runs at
optimization level 0 so that step numbers mean the same thing for every
recording. 08 should state this definition.

**Status:** open.
