# Architecture decision records

One file per decision. Format: context, decision, alternatives considered,
consequences. A decision is `accepted`, `proposed` or `superseded`. ADRs 0001 to 0008
were accepted on 2026-10-18.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-implementation-language.md) | Implementation language: Rust | accepted (2026-10-18) |
| [0002](0002-ownership-on-generic-code.md) | Ownership analysis on generic code, as a local dataflow pass | accepted (2026-10-18) |
| [0003](0003-one-mir-many-backends.md) | One mono MIR, many backends | accepted (2026-10-18) |
| [0004](0004-whole-program-reachability.md) | Whole-program reachability for tree-shaking | accepted (2026-10-18) |
| [0005](0005-native-c-abi-plugins.md) | Plugins are native C-ABI libraries | accepted (2026-10-18) |
| [0006](0006-debugging-over-dap.md) | Debugging over DAP, with record and replay | accepted (2026-10-18) |
| [0007](0007-task-model.md) | One task semantics, three native lowerings and VM tasks | accepted (2026-10-18) |
| [0008](0008-tiered-execution.md) | Bytecode VM first, Cranelift second, LLVM for release | accepted (2026-10-18) |
| [0009](0009-cargo-disk-budget.md) | Cargo disk budget | accepted (2026-10-19) |
| [0010](0010-import-cycles-are-an-error.md) | Import cycles are an error | accepted (2026-10-19) |
| [0011](0011-verify-before-first-statement.md) | Verify every reachable instance before the first statement | accepted (2026-10-19) |
| [0012](0012-one-call-limit-and-trace.md) | One call limit and one trace on every tier | accepted (2026-10-19) |
| [0013](0013-drop-glue-runs-no-user-code.md) | Values close at their last use, and drop glue runs no user code | accepted (2026-10-19) |
| [0014](0014-assignment-copies.md) | Assignment copies, and storage is shared until written | accepted (2026-10-19) |
| [0015](0015-refine-a-binding.md) | A binding can be refined by consuming it | accepted (2026-10-19) |
| [0016](0016-raises-clause.md) | A `raises` clause keeps error sets exact | accepted (2026-10-19) |
| [0017](0017-prefix-try.md) | Propagation is the prefix word `try` | accepted (2026-10-19) |
| [0018](0018-exported-signatures-in-full.md) | Exported signatures are written in full | accepted (2026-10-19) |
| [0019](0019-constant-initialization-cycles.md) | A cycle among constant initializers is an error | accepted (2026-10-19) |
| [0020](0020-object-key.md) | Object code is keyed by the instance's optimized body | accepted (2026-10-19) |
| [0021](0021-task-panicked-without-heap.md) | `TaskPanicked` without a heap | accepted (2026-10-19) |
