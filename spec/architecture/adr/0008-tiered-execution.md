# ADR-0008: Bytecode VM first, Cranelift second, LLVM for release

Status: accepted (2026-10-18, all three owners).

## Decision

Build backends in this order: bytecode VM (M1), Cranelift (M2), Wasm (M3), LLVM
(M7). The VM is the reference implementation and the debug target.

## Why

- The VM is the cheapest way to get an end-to-end language and the best oracle
  for differential testing.
- Cranelift compiles faster than LLVM, which keeps `verse build` and
  `verse run --fast` pleasant.
- LLVM covers the targets Cranelift does not (32-bit ARM, 32-bit RISC-V,
  Xtensa) and gives the best release performance.

## Alternatives

- **LLVM first.** Best code, slowest path to a working language.
- **JIT-only for scripts.** Fast, but complex to debug and not available on
  every target (Wasm, iOS-style restrictions).
- **Tree-walking interpreter.** Simpler but slower; the bytecode design costs
  little more because MIR already exists.

## Consequences

- LLVM is an optional feature in the toolchain (large, long to build).
- The 32-bit embedded plan depends on LLVM; Cranelift's target list must be
  re-verified in M2.
