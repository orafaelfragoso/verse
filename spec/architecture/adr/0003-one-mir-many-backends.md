# ADR-0003: One mono MIR, many backends

Status: proposed.

## Context

Requirements R1 (script and compile) and R2 (many targets) and the agreed rule
that script and compiled modes must behave identically.

## Decision

All execution tiers consume the **same monomorphized MIR** in which moves,
drops, checked arithmetic, panics, unwind edges and yield points are explicit.
Backends are lowerings, not interpretations of the language.

## Alternatives

1. **Separate interpreter over the AST plus a compiler.** Fastest to start,
   but two semantics; every feature is implemented twice and the behaviors
   drift.
2. **Compile to C and use the system C compiler.** Wide target coverage, but a
   hard external dependency, slow, poor debugging, no sandboxed environment.
3. **LLVM IR only.** Strong optimization, but slow for scripts, huge dependency,
   and does not cover the bytecode or Wasm-size goals.
4. **Transpile to Rust.** Leans on rustc's checks, but compile times and
   dependency weight are unacceptable for a scripting mode.

## Consequences

- The VM stays small, because ownership and generics are already resolved.
- Overflow, bounds and panic policy live in one place.
- A target costs a lowering and a platform layer ([04](../04-backends-and-targets.md), section 4.3).
- MIR becomes a stability-sensitive internal interface; it is versioned, and
  plugin backends ([07](../07-plugins.md)) see a stable serialized form.
