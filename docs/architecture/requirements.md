# 01. Requirements

The seven requirements below come from the project owner. Each is rewritten
as something we can test, with the design element that satisfies it.

| ID | Requirement | Testable form | Satisfied by |
|----|-------------|---------------|--------------|
| R1 | Runs as a scripted language and can also be compiled | `verse run x.vs` works with no build step and starts in under 20 ms for a hello-world (target). `verse build x.vs` produces an executable whose observable behavior is identical (checked by differential tests) | One front end and one mid-level IR feeding an interpreter tier and compiled tiers ([ADR-0003](../decisions/0003-one-mir-many-backends.md), [04](backends-and-targets.md)) |
| R2 | Easy to compile to Wasm, native binaries, ARM, x86, IoT | Adding a target means writing a backend lowering and a runtime platform layer. It must not touch the front end or the middle end | Target-independent mono MIR, explicit target descriptions, a thin platform layer ([04](backends-and-targets.md), [05](runtime.md)) |
| R3 | Extensible with plugins | A third party can add a lint, a code generator, a bindgen, or a capability provider without forking the compiler, and cannot read files or the network unless granted | Sandboxed WebAssembly component plugins with declared capabilities ([ADR-0005](../decisions/0005-wasm-component-plugins.md), [07](plugins.md)) |
| R4 | Really fast and efficient | Compile speed, run speed, memory, binary size and startup have budgets and a benchmark suite that fails CI on regression | [09](performance.md), data layout and monomorphization decisions in [02](compiler-architecture.md) and [05](runtime.md) |
| R5 | Name the implementation language | One language for compiler, runtime and tooling, chosen with alternatives compared | **Rust** ([ADR-0001](../decisions/0001-implementation-language.md)) |
| R6 | Standard library with builtins for finance, ML, servers, GUI and so on; optional; only what the AST uses is built in | A hello-world binary contains no networking, no JSON, no GUI. `verse build --size-report` explains every included item | Library tiers, whole-program reachability, capability-selected runtime slices ([ADR-0004](../decisions/0004-whole-program-reachability.md), [06](stdlib-and-shaking.md)) |
| R7 | Debugging that is easy, especially for LLMs, in the spirit of MCP | An LLM can launch a program, stop it, inspect values with their ownership state, step forward and backward, and ask why something happened, all through a Model Context Protocol server with bounded, structured output | `verse mcp`, deterministic record and replay, structured diagnostics ([ADR-0006](../decisions/0006-mcp-first-debugging.md), [08](debugging-and-mcp.md)) |

## Constraints taken from the specification

These are facts from spec 0.1 that shape the architecture. Section numbers
refer to the specification.

1. **Indentation-significant, lossless-tool-friendly syntax** (§3.3, §3.4). The
   lexer produces `INDENT`, `DEDENT`, `NEWLINE`. The formatter and the
   language server need a lossless syntax tree.
2. **Types are visually optional** (§4.2). Unannotated parameters are implicit
   generics with inferred trait, callable and structural constraints. Code is
   specialized per concrete type. Recursion needs annotations (rule 3).
3. **Ownership is checked once, at the definition** (§4.2 rule 9, §10.6). So the
   ownership check runs on **generic** code, before specialization, and treats
   unannotated types as opaque and non-`Copy`. This is the most important
   ordering constraint in the pipeline.
4. **No garbage collector, no stored references** (§4.10, §5.3, §18.10).
   Borrows are second-class. This removes the need for lifetime inference and
   keeps the borrow checker a local dataflow analysis.
5. **Whether a binding is a view or an owner depends on later use** (§4.9,
   §18.10). The ownership pass needs last-use liveness and must be able to say
   which later use kept a source alive.
6. **Inference never crosses a module boundary** and exported signatures are
   pinned in `verse.api` (§4.2, §12.3, §16.4). This makes per-module,
   parallel and incremental compilation natural.
7. **`Cleanup` runs on panic and on cancellation** (§8.5, §11.5, §13). The
   middle end needs explicit unwind edges.
8. **Lightweight tasks, no `async`/`await`, no function coloring, tasks run
   across all cores** (§11.1). Blocking calls must yield to a scheduler
   without changing the function's type.
9. **Integer overflow, out-of-bounds, divide by zero stop the program**
   (§1.3, §4.1). Checks are part of the semantics. Optimization may remove only
   checks it can prove cannot fail.
10. **Dependencies cannot run code at build time** (§16.3). Plugins must be
    opt-in at the root project and sandboxed.
11. **One formatter with no options and `verse api`** (§16.1, §16.4).
12. **Map preserves insertion order** (§9.2). Useful for deterministic replay.
    `Set` also iterates in insertion order (§9.4).
