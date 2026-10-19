# ADR-0002: Ownership analysis on generic code, as a local dataflow pass

Status: accepted (2026-10-18).

## Context

Spec §4.2 rule 9 says ownership is checked once, at the definition, treating
unannotated and generic types as opaque and non-`Copy`. Spec §4.10 and §5.3 say
borrows are second-class: they cannot be stored, returned or sent to a task.

## Decision

Implement ownership as a **forward and backward dataflow analysis over the
generic THIR/MIR of one function**, with no lifetime variables and no region
inference. The result drives drop elaboration and explains errors.

## Alternatives

1. **Port Rust's borrow checker (NLL / Polonius).** Solves stored references
   with lifetimes. Verse has neither, so this is unused complexity and slower.
2. **Check after monomorphization.** Simpler for the checker, but breaks the
   rule that errors appear at the definition, gives different answers per
   instantiation, and multiplies the cost by the number of instances.
3. **Reference counting everywhere.** No analysis needed, but it contradicts
   the spec's "no garbage collection, deterministic frees" and costs run time.

## Consequences

- Linear-time per function, parallel across functions.
- The pass must be able to say *which later use* kept a source alive. The
  analysis keeps the liveness evidence for diagnostics.
- A bug here is a soundness bug in the language. It gets a model-based
  differential test and a poisoning-allocator fuzzer ([10](../architecture/testing-and-verification.md)).
- The inferred `Copy` bound for getters is an output of this pass, written into
  the function's signature record and into `verse.api`.
