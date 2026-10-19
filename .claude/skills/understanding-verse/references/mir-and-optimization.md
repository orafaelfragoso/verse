# MIR and optimization

The middle of the pipeline: MIR build, generic MIR, monomorphization, mono
MIR, the verifier and the optimizer passes. Every execution tier consumes the
result.

## Read first

1. [ADR-0003 One mono MIR, many backends](../../../../docs/decisions/0003-one-mir-many-backends.md).
2. [02 §2.1 Pipeline](../../../../docs/architecture/compiler-architecture.md#21-pipeline)
   and [02 §2.2 Stage contracts](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts),
   the rows from MIR build to Reachability.
3. [02 §2.6 Monomorphization](../../../../docs/architecture/compiler-architecture.md#26-monomorphization).
4. [03 §3.3 MIR](../../../../docs/architecture/ir-and-passes.md#33-mir),
   [03 §3.4 Pass list](../../../../docs/architecture/ir-and-passes.md#34-pass-list)
   and [03 §3.5 Layout rules](../../../../docs/architecture/ir-and-passes.md#35-layout-rules).
5. [13 MIR specification](../../../../docs/architecture/mir-specification.md), in order:
   [§13.1](../../../../docs/architecture/mir-specification.md#131-purpose-and-relation-to-03),
   [§13.3](../../../../docs/architecture/mir-specification.md#133-types-and-layout-facts),
   [§13.4](../../../../docs/architecture/mir-specification.md#134-statements-and-terminators),
   [§13.5](../../../../docs/architecture/mir-specification.md#135-verifier-rules),
   [§13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir),
   [§13.7](../../../../docs/architecture/mir-specification.md#137-debug-info-and-source-spans),
   [§13.9](../../../../docs/architecture/mir-specification.md#139-versioning-policy).
   Keep [§13.2 Textual format](../../../../docs/architecture/mir-specification.md#132-textual-format)
   and [§13.8 Worked examples](../../../../docs/architecture/mir-specification.md#138-worked-examples)
   open as reference.
6. [10 §10.3 Differential testing](../../../../docs/architecture/testing-and-verification.md#103-differential-testing)
   and [10 §10.6 Formal methods](../../../../docs/architecture/testing-and-verification.md#106-formal-methods-selective).

Doc 13 is the detailed source. If 03 and 13 differ, 13 wins and 03 is
corrected ([03 §3.3](../../../../docs/architecture/ir-and-passes.md#core-definitions)).

## What it does

| Stage | Input | Output | Errors it owns |
|-------|-------|--------|----------------|
| MIR build | THIR plus ownership facts | CFG of basic blocks; locals typed; explicit `Move`, `Copy`, `Drop`, unwind edges | none |
| Monomorphization | generic MIR | no type parameters, no structural constraints, no `Show` or `Eq` left abstract; trait calls resolved except calls through a trait name used as a type | none |
| Optimizer | mono MIR | semantics preserved; verifier passes after every pass in debug builds | none |

Source: [02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts).

- The crate `verse-mir` owns the MIR definition, MIR build, the verifier, the
  optimizer and monomorphization
  ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).
- Everything left of generic MIR is target independent. Everything from mono
  MIR onward knows the target
  ([02 §2.1](../../../../docs/architecture/compiler-architecture.md#21-pipeline)).
- **Generic MIR** may mention type parameters and unresolved trait calls and
  has no layout. **Mono MIR** has every parameter substituted, every trait call
  resolved or a `dyn` slot, and layout attached
  ([13 §13.1](../../../../docs/architecture/mir-specification.md#131-purpose-and-relation-to-03)).
- `verse dump mir` prints the MIR text format, including `layout` lines, for
  golden tests ([19 §19.2](../../../../docs/reference/cli-and-manifest.md#192-commands)).
- No code exists yet. Generic MIR and mono belong to M1
  ([roadmap](../../../../docs/roadmap.md#11-roadmap)).

## How it works

**Shape.** MIR is a CFG in SSA-like form for temporaries, with named, typed,
mutable locals for user bindings, as in Rust MIR. Operands, not rvalues, carry
`move` and `copy`
([03 §3.3](../../../../docs/architecture/ir-and-passes.md#33-mir)). Why: with
explicit moves and drops every backend, including the bytecode interpreter,
runs identical semantics with no ownership logic of its own
([03 Why this shape](../../../../docs/architecture/ir-and-passes.md#why-this-shape)).

**Checks as data.** `checked_*` returns `(T, Bool)` and is followed by a
`switch` to a `panic overflow` block. Division by zero and bounds are explicit
`switch` tests too. Why: overflow policy is a MIR concern, so range analysis
deletes checks in one place for all targets
([03 Why this shape](../../../../docs/architecture/ir-and-passes.md#why-this-shape)).

**Unwind edges as data.** `call`, `spawn`, `yield`, `panic` and terminator
`drop` name `unwind bbN`, `unwind continue` or `unwind none`. Why: each
backend picks its own mechanism (native tables, Wasm exception handling,
status codes) from the same edges
([13 §13.4](../../../../docs/architecture/mir-specification.md#134-statements-and-terminators)).

**Two drop forms.** The `drop` statement is for a type whose `cleanup` fact is
false; it cannot unwind or yield. The terminator `drop` is for a type with
`Cleanup`, or an unknown type in generic MIR; it may block or panic
([13 §13.4](../../../../docs/architecture/mir-specification.md#drop-order)).

**Tasks.** `spawn` and `yield` are terminators with operands, a destination, a
continuation and an unwind edge. Cancellation arrives at a `yield` and takes
its unwind edge ([03 §3.3](../../../../docs/architecture/ir-and-passes.md#33-mir)).

**Debug info.** Every statement and terminator has a span and a `dbg` clause.
There is no mark statement: stepping marks come from spans
([13 §13.7](../../../../docs/architecture/mir-specification.md#137-debug-info-and-source-spans)).

**Storage.** Statements and terminators sit in flat arrays per function, and
blocks hold index ranges. Nodes live in per-module arenas with typed `u32`
indices, and types are interned. These are assumptions until the M0
benchmarks run
([02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)).

**Text format.** The printer is the only writer. The parser accepts exactly
what the printer writes, and `parse(print(m)) == m` is a unit test
([13 §13.2](../../../../docs/architecture/mir-specification.md#132-textual-format)).

**Monomorphization** maps `(generic fn, [TypeId])` to a mono function. It
substitutes types, resolves `<T as Trait>::m` or leaves a `dyn` slot, renames
the function, attaches layouts, and recomputes `caps` and `mayyield`. It is
demand driven from entry statements, exports and tests. Why: this gives exact
reachability for free
([02 §2.6](../../../../docs/architecture/compiler-architecture.md#26-monomorphization),
[13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir)).
Code bloat is held down by identical code folding, and a trait name used as a
type gives one shared body.

**Script mode.** Before the first statement, the reachability pre-pass
monomorphizes every reachable instance and verifies it with the mono rules,
exactly as a compiled build does. It builds no bytecode, runs no optimizer
pass, and caches its result per module. Only bytecode generation waits for the
first call, so a verifier failure never happens mid-run
([13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir),
[ADR-0011](../../../../docs/decisions/0011-verify-before-first-statement.md)).

**Optimizer.** Fourteen mono MIR passes run in order: `verify`,
`simplify-cfg`, `inline`, `sroa`, `const-prop` and `fold`, `move-elim`,
`copy-elide`, `range-analysis`, `devirtualize`, `drop-opt`, `dce`, `layout`,
`refcount-opt`, `task-lower`. Each states what it must preserve, and each can
be switched off with `--no-opt-pass=NAME` so the differential tester can
isolate a miscompile
([03 §3.4](../../../../docs/architecture/ir-and-passes.md#34-pass-list)).
Optimization level 0 runs only `verify`. Debug sessions and replays use it so
steps mean the same in every session
([13 §13.7](../../../../docs/architecture/mir-specification.md#137-debug-info-and-source-spans)).

## Rules that must hold

- **V1 to V24.** `verse-verify` checks them after MIR build, after
  monomorphization, and after every optimizer pass in debug builds and CI.
  Generic MIR is checked against V1 to V21, V23 and V24. Mono MIR is checked
  against V1 to V20 and V22 to V24
  ([13 §13.5](../../../../docs/architecture/mir-specification.md#135-verifier-rules),
  [13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir)).
  A failure is the internal error V9902.
- **The verifier is independent.** It is written independently of the passes
  it checks. V3 to V9 re-derive what the ownership pass decided, as a safety
  net and as the oracle of the soundness fuzzer
  ([13 §13.5](../../../../docs/architecture/mir-specification.md#135-verifier-rules),
  [10 §10.6](../../../../docs/architecture/testing-and-verification.md#106-formal-methods-selective)).
- **Borrows are second-class (V7).** A borrow never reaches a field, an
  element, the return place, a `spawn` operand or a capture.
- **A backend holds no ownership logic and picks no overflow or panic policy**
  ([13 §13.1](../../../../docs/architecture/mir-specification.md#131-purpose-and-relation-to-03)).
- **Monomorphization is a pure substitution.** It never adds or removes a
  move, drop or panic, and never changes block structure, spans or `dbg`
  clauses. Turning a `move` of a `Copy` type into a `copy` is `move-elim`'s
  job ([13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir)).
  Ownership facts are never recomputed for a specialization
  ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)).
- **`layout` is the only pass that may change field order.** It must not
  reorder `(T, Bool)` of a checked operation: value at offset 0, flag next
  ([13 §13.3](../../../../docs/architecture/mir-specification.md#133-types-and-layout-facts)).
- **Panic sites stay put.** One `panic` per source site. Two merge only when
  kind, operands and span are equal
  ([13 §13.7](../../../../docs/architecture/mir-specification.md#137-debug-info-and-source-spans)).
  No step may reorder two panic sites
  ([14 §14.5](../../../../docs/architecture/bytecode-and-vm.md#145-checked-arithmetic-bounds-and-division)).
- **Folding a checked operation that overflows stays a panic**, never a wrap
  ([03 §3.4](../../../../docs/architecture/ir-and-passes.md#34-pass-list), pass 5).
- **Inlining keeps the inlined spans** and adds the call site as a parent
  span ([13 §13.7](../../../../docs/architecture/mir-specification.md#137-debug-info-and-source-spans)).
- **MIR is internal and unversioned for third parties.** Any change to the
  grammar, a statement, a terminator or a printing rule increments `rev` and
  regenerates golden files in the same commit
  ([13 §13.9](../../../../docs/architecture/mir-specification.md#139-versioning-policy)).
- **Deterministic output.** No hash-map iteration order leaks into output,
  and instance ordering is stable
  ([02 §2.7](../../../../docs/architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)).

## What depends on it

- Upstream, MIR build trusts THIR and the ownership facts of `verse-own`.
  See [ownership-checking.md](ownership-checking.md).
- Downstream, `verse-reach` reads `caps(...)` and allocation intrinsics for
  the capability check and `--no-heap`
  ([13 §13.3](../../../../docs/architecture/mir-specification.md#133-types-and-layout-facts)).
- Every tier lowers mono MIR: `verse-bc`, `verse-cl`, `verse-llvm`,
  `verse-wasm`. A new statement or terminator needs a lowering in each. See
  [vm-and-backends.md](vm-and-backends.md).
- The debugger reads spans, `dbg` clauses and mode words. A step is one
  executed mono MIR statement or terminator at level 0, so changing them
  changes step numbers and recordings
  ([13 §13.7](../../../../docs/architecture/mir-specification.md#137-debug-info-and-source-spans)).
  See [debugger-and-language-server.md](debugger-and-language-server.md).
- `layout` output is ABI level A, shared with `verse-rt` and the debug map
  ([15 §15.1](../../../../docs/architecture/target-and-abi.md#151-purpose-and-scope)).
- Each MIR panic kind maps to one `R` code and its facts
  ([16 §16.6](../../../../docs/reference/diagnostics-catalog.md#166-run-time-panics)).
- The crate dependency direction is strictly downward. A back edge is a build
  error ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - unit tests, including `parse(print(m)) == m`;
   - golden MIR dumps under `tests/golden/`. Every listing of 13 §13.8 is a
     golden test;
   - a verifier case for each rule you add, with a rejected example;
   - differential tests: optimized equals unoptimized on the conformance suite
     (`verse difftest`);
   - the ownership soundness fuzzer, when V3 to V9 change
     ([10 §10.4](../../../../docs/architecture/testing-and-verification.md#104-safety-properties-we-test-explicitly)).
2. A new pass needs a one-paragraph specification, a golden test, a
   differential test and a benchmark that shows it helps, or it is deleted
   ([03 §3.4](../../../../docs/architecture/ir-and-passes.md#34-pass-list)).
3. Update doc 13 first, then the summary in 03. A layout change also updates
   [15 §15.3](../../../../docs/architecture/target-and-abi.md#153-data-layout).
4. Optimizer passes report only V99xx. `task-lower` reports V0909. A new pass
   needs a row in
   [16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass).
   See [diagnostics.md](diagnostics.md).
5. Use the commit format of
   [CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format). The scope is
   `compiler`. A pass that only improves speed or size is `perf`. A change to
   doc 13 or 03 is `spec`.
6. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
