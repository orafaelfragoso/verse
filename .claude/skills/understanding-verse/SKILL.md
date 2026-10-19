---
name: understanding-verse
description: Explains how the Verse toolchain is designed and routes to one reference file per subsystem, each with what to read, how it works, the rules that must hold, what depends on it and the checks after a change. Use before any change to the compiler, MIR, VM, backends, runtime, tasks, standard library, CLI, packages, plugins, debugger, language server or diagnostics; when touching crates/, std/ or tests/; when a crate is named (verse-syntax, verse-hir, verse-types, verse-own, verse-mir, verse-reach, verse-ffi, verse-bc, verse-cl, verse-llvm, verse-wasm, verse-rt, verse-pkg, verse-fmt, verse-lsp, verse-dbg, verse-cli); or for questions such as "how does Verse work", "where does this belong", "what does this stage guarantee".
---

# Understanding Verse

Verse is a statically typed, general-purpose language whose overriding goal is
readability ([spec §1](../../../docs/language/01-philosophy.md#1-philosophy-and-design-principles)).
It ships as one command, `verse`, with one compiler: a target-independent front
end, a middle end on MIR, and interchangeable last stages: a bytecode VM,
Cranelift, LLVM and a WebAssembly emitter
([00](../../../docs/architecture/overview.md#what-the-implementation-is)).
The toolchain is written in Rust
([ADR-0001](../../../docs/decisions/0001-implementation-language.md)), and
nothing is implemented yet, so every number in the docs is a target
([docs/README](../../../docs/README.md)).

## The pipeline

From [02 §2.1](../../../docs/architecture/compiler-architecture.md#21-pipeline);
the contract of each stage is
[02 §2.2](../../../docs/architecture/compiler-architecture.md#22-stage-contracts).

```
.vs source -> Lexer -> CST (lossless) -> AST -> Name resolution -> HIR
  -> Type inference -> THIR -> Ownership analysis (on generic code)
  -> Generic MIR -> Monomorphization -> Mono MIR -> MIR optimizer
  -> Reachability and capability check
       -> Bytecode (tier 0) | Cranelift (tier 1) | LLVM (tier 2) | Wasm (tier W)
```

Everything left of Generic MIR is target independent and library independent.
Everything from Mono MIR onward knows the target.

## If you touch X, read references/Y.md

Read only the reference you need. Each one links to the doc sections that are
the source of truth. Crates and folders are those of
[02 §2.9](../../../docs/architecture/compiler-architecture.md#29-crate-layout).

| You touch | Read |
|-----------|------|
| `verse-syntax` (lexer, CST, parser, AST), `verse-fmt` | [parser-and-formatter](references/parser-and-formatter.md) |
| `verse-hir` (resolution, HIR, THIR), `verse-types` (inference, traits) | [names-and-types](references/names-and-types.md) |
| `verse-own` | [ownership-checking](references/ownership-checking.md) |
| `verse-mir` (MIR, build, verifier, optimizer, mono) | [mir-and-optimization](references/mir-and-optimization.md) |
| `verse-bc`, `verse-cl`, `verse-llvm`, `verse-wasm`, target files | [vm-and-backends](references/vm-and-backends.md) |
| `verse-rt` (slices, memory, tasks, panics) | [runtime-and-tasks](references/runtime-and-tasks.md) |
| `verse-ffi`, `extern`, `[plugins]`, `verse bind` | [native-plugins](references/native-plugins.md) |
| `verse-pkg`, `verse-cli`, `verse.toml`, `verse.lock`, `verse.api` | [packages-and-cli](references/packages-and-cli.md) |
| `verse-dbg`, `verse-lsp`, `events.toml`, `.vsrec` | [debugger-and-language-server](references/debugger-and-language-server.md) |
| `std/`, first-party packages, `verse-reach` (reachability, tree-shaking, capabilities; doc 06) | [standard-library](references/standard-library.md) |
| a diagnostic code, message or `verse explain` text | [diagnostics](references/diagnostics.md) |

`tests/` holds the conformance, differential, golden and fuzz corpora, and
`bench/` the benchmarks. The test layers are
[10 §10.1](../../../docs/architecture/testing-and-verification.md#101-test-layers).

The crate order is strictly downward. A back edge, such as `verse-types`
importing a backend, is a build error in CI
([02 §2.9](../../../docs/architecture/compiler-architecture.md#29-crate-layout)).

## Rules that hold everywhere

- One meaning in both modes. `verse app.vs` and `verse build` behave
  identically, because every tier consumes the same mono MIR
  ([00](../../../docs/architecture/overview.md#guiding-principles)).
- The specification wins over the architecture docs
  ([architecture README](../../../docs/architecture/README.md)).
- A violated stage invariant is a compiler bug, caught by `verse-verify`
  ([02 §2.2](../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
- Diagnostics are data first, text second
  ([02 §2.8](../../../docs/architecture/compiler-architecture.md#28-diagnostics)).

## Workflow skills

- editing-docs: changing anything under docs/, ADRs, diagnostic codes, and
  running `tools/docs/check.sh`.
- writing-tests: choosing the test layer and writing the test.
- opening-a-pull-request: branch, commit format, template and checks.
- filing-an-issue: templates, labels and milestones.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
