# ADR-0003: One mono MIR, many backends

Status: accepted (2026-10-18).

## Context

Requirements R1 (script and compile) and R2 (many targets) and the rule
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
- A target costs a lowering and a platform layer ([04](../architecture/backends-and-targets.md), section 4.3).
- MIR becomes a stability-sensitive internal interface, specified in
  [13](../architecture/mir-specification.md) (text format, statements, verifier rules).
  In 0.1 its serialization is **internal and unversioned for third parties**:
  there are no backend plugins ([07](../architecture/plugins.md) plugins are
  native libraries called from a program), and nothing outside the repository may read or write MIR. An
  internal revision number in the header guards the persistent cache only
  ([13.9](../architecture/mir-specification.md#139-versioning-policy)).
- The first backend is the bytecode VM, specified in
  [14](../architecture/bytecode-and-vm.md).
