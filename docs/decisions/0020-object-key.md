# ADR-0020: Object code is keyed by the instance's optimized body

Status: accepted (2026-10-19).

## Context

Object code was cached under `blake3(source, compiler version, flags,
verse.api of imports)` (02 §2.7). An instance of an imported generic takes
its body from the dependency, and the dependency's `verse.api` stays the same
when only that body changes, so a compiled build could link stale object
code. [ADR-0011](0011-verify-before-first-statement.md) closed the same hole
for verified instances and protos with the transitive verified-instance key.

That key is not enough for object code. The optimizer reads facts about the
whole program: `inline` takes in a callee that has one caller in the program,
and `refcount-opt` drops atomic counts when the program never spawns
(03 §3.4). An instance whose verified-instance key is unchanged can therefore
compile to different code when another part of the program changes.

## Decision

1. Object code is stored per instance under its **object key**:
   `blake3(optimized mono MIR of the instance, layouts of the types it uses,
   compiler version, backend, codegen flags, target)` (02 §2.7).
2. The optimized body holds every callee body the optimizer took in and
   every whole-program decision it made for the instance, so the key changes
   whenever the code would.
3. The optimizer reruns on each verified instance, which the cache already
   holds; a hit skips code generation, the expensive step.
4. Parsed modules and signatures keep the module key. Protos keep the
   verified-instance key plus the bytecode compiler's flags, because the
   bytecode compiler reads the verified body, not the optimized one.

## Alternatives

- **The verified-instance key plus codegen flags**, as the issue proposed.
  It misses the whole-program facts above.
- **The verified-instance key plus a list of the whole-program facts the
  optimizer read.** Correct only while that list matches what every pass
  reads; a new pass that reads one more fact would break the cache silently.
- **Hash the dependency's full source into each importer's key.** Correct,
  but every change to a dependency rebuilds every importer.

## Consequences

- A compiled build never links stale object code, however a dependency or
  the rest of the program changes.
- Every build pays for optimization of every reachable instance. That this
  costs a small part of code generation is an assumption, measured in M2; if
  it does not hold, the optimizer's own results get a cache with the
  whole-program facts in their key.
- The per-instance edge lists of 06 §6.10 already hash the optimized body, so
  shaking and code generation share one view of an instance.
