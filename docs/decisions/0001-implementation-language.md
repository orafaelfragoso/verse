# ADR-0001: Implementation language: Rust

Status: accepted (2026-10-18).

## Context

We need one language for the compiler, the runtime and the tooling. The
runtime must run with no OS on microcontrollers and in Wasm, and the compiler
must be fast, parallel, and embeddable (a language server, a plugin host, a
JIT).

## Decision

**Rust**, current stable, for `verse-*` crates, the runtime slices and the
CLI. The runtime's `core` slice is `no_std` plus `alloc`. A tiny amount of
assembly (context switch) and C only where a platform requires it. Later,
parts of the compiler may be rewritten in Verse itself (self-hosting is a
goal after the first stable release, not part of the plan).

## Alternatives

| Option | For | Against |
|--------|-----|---------|
| **Rust** | memory safety without a collector; enums and pattern matching fit ASTs and IRs; Cranelift, `wasm-encoder`, wasmtime and tree-sitter-style tools are native crates; `no_std` for embedded; one static binary; strong cross-compilation; excellent tooling for fuzzing and sanitizers | longer compile times; borrow-checker friction for graph-shaped IRs (solved by arenas and indices); steeper contributor learning curve |
| Zig | simple, fast builds, excellent C interop, explicit allocators | no mature Cranelift or Wasm tooling in the language; fewer libraries; language not yet stable (1.0 not reached); weaker safety in the compiler itself |
| C++ | LLVM is native; mature | memory-unsafe compiler; slower development; build complexity; poor fit for fuzz-proof parsers |
| Go | fast builds, easy concurrency | has a garbage collector, awkward for a runtime for a language with no GC and for embedded; weak at algebraic data types |
| OCaml / Haskell | superb for compilers (pattern matching) | runtime with a GC; harder to embed, harder to target microcontrollers; smaller contributor base |
| Self-hosted Verse | the dream | does not exist yet; cannot bootstrap |
| TypeScript | fastest prototyping; a browser playground for free | garbage-collected and JIT-dependent, so neither the runtime nor microcontroller targets can use it; compiler speed bounded by the JS engine; a second language would still be needed for the runtime |

## Consequences

- Contributors need Rust. The architecture avoids Rust's pain points with
  arenas, typed indices and interning ([02](../architecture/compiler-architecture.md)).
- Backend crates (Cranelift) are pinned and upgraded deliberately.
- Rust is also the language of the toolchain's runtime, so the `core` runtime
  slice is built with `-C panic=abort` internally and implements Verse's
  unwinding itself (section 5.4 of the runtime document).
- LLVM is reached through a Rust binding and is an optional feature.

## Risks

- **Compile times of the compiler itself.** A large Rust workspace builds
  slowly; the crate split in [02](../architecture/compiler-architecture.md) §2.9 and
  keeping LLVM optional are the mitigations.
- **LLVM linkage.** The binding ties each toolchain release to one LLVM major
  version and complicates cross-building the toolchain; tier 0, tier 1 and
  tier W must never depend on it.
- **Self-hosting stays a non-goal until the language is stable.** Rewriting
  parts in Verse before the spec and the MIR settle would double the work.
