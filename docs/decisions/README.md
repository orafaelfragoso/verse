# Architecture decision records

One file per decision. Format: context, decision, alternatives considered,
consequences. A decision is `accepted`, `proposed` or `superseded`. On 2026-10-18 all three
owners reviewed ADRs 0001 to 0008 and accepted them.

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-implementation-language.md) | Implementation language: Rust | accepted (2026-10-18, all three owners) |
| [0002](0002-ownership-on-generic-code.md) | Ownership analysis on generic code, as a local dataflow pass | accepted (2026-10-18, all three owners) |
| [0003](0003-one-mir-many-backends.md) | One mono MIR, many backends | accepted (2026-10-18, all three owners) |
| [0004](0004-whole-program-reachability.md) | Whole-program reachability for tree-shaking | accepted (2026-10-18, all three owners) |
| [0005](0005-wasm-component-plugins.md) | Plugins are sandboxed Wasm components | accepted (2026-10-18, all three owners) |
| [0006](0006-mcp-first-debugging.md) | MCP-first debugging with record and replay | accepted (2026-10-18, all three owners) |
| [0007](0007-task-model.md) | One task semantics, three native lowerings and VM tasks | accepted (2026-10-18, all three owners) |
| [0008](0008-tiered-execution.md) | Bytecode VM first, Cranelift second, LLVM for release | accepted (2026-10-18, all three owners) |
