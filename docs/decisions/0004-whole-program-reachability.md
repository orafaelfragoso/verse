# ADR-0004: Whole-program reachability for tree-shaking

Status: accepted (2026-10-18).

## Context

Requirement R6: the build includes only what the program uses, found by
analyzing the program, not by hand-picking options.

## Decision

Tree-shaking is a by-product of **demand-driven monomorphization** plus a
reachability pass over mono MIR:

1. Instantiate only from the entry statements (or exports for a library).
2. Collect the platform capabilities used by reachable intrinsics.
3. Select runtime slices from the capability set and link them with section
   garbage collection.
4. Emit a size report: every included function, type and slice, with the
   call chain that pulled it in.

## Alternatives

- **Module-granularity pruning by imports.** Easy, but a module with one used
  function ships all of it. Works only as a first approximation (and spec
  imports are explicit, so it is still a fast pre-filter).
- **Linker-only gc-sections.** Cannot prune vtables, structural trait
  implementations or runtime features.
- **Cargo-style feature flags chosen by the user.** Violates the requirement
  that the analysis chooses.
- **Run-time dynamic loading of library modules.** Defers the cost instead of
  removing it, needs a dynamic linker and a stable ABI on every target, is
  unavailable on microcontrollers and awkward in Wasm, and would make script
  and compiled modes load code differently.

## Consequences

- Reachability is **exact** when the language has no run-time reflection. The
  spec has none: `ToJson`, `Show`, `Eq` and `Hash` are structural and derived at
  compile time per used type. We rely on that.
- Values typed by a trait name (§10.3) need conservative vtables (all implementations of a trait
  for types that are instantiated). *Note:* the spec has no `dyn`
  keyword. [06](../architecture/stdlib-and-shaking.md)
  §6.4 narrows the rule to types actually converted to the trait type and to
  vtable slots actually called.
- "Exact" means exact with respect to the optimized program text, not to
  run-time values: a branch that never executes is still included.
- A foreign call is an ordinary call edge to its plugin and symbol, so the
  analysis stays exact on the Verse side. What foreign code does inside is
  outside it.
