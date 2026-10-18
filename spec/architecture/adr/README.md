# Architecture decision records

One file per decision. Format: context, decision, alternatives considered,
consequences. A decision is `accepted`, `proposed` or `superseded`.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-implementation-language.md) | Implementation language: Rust | proposed |
| [0002](0002-ownership-on-generic-code.md) | Ownership analysis on generic code, as a local dataflow pass | proposed |
| [0003](0003-one-mir-many-backends.md) | One mono MIR, many backends | proposed |
| [0004](0004-whole-program-reachability.md) | Whole-program reachability for tree-shaking | proposed |
| [0005](0005-wasm-component-plugins.md) | Plugins are sandboxed Wasm components | proposed |
| [0006](0006-mcp-first-debugging.md) | MCP-first debugging with record and replay | proposed |
| [0007](0007-task-model.md) | One task semantics, three lowering strategies | proposed |
| [0008](0008-tiered-execution.md) | Bytecode VM first, Cranelift second, LLVM for release | proposed |
