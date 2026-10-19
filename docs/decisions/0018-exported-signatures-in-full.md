# ADR-0018: Exported signatures are written in full

Status: accepted (2026-10-19).

## Context

Any parameter could be left unannotated, exported ones included. An
unannotated parameter was an implicit generic with trait, callable and
structural constraints (`has field`, `has method`) that could not be written
by hand, so `verse.api` recorded public signatures in a notation their users
could not write. The rule brought V0419 (a new instance per recursion depth),
an inferred `Copy` bound for getters with its call-site error V0508, the
`has` bound of generic MIR, and the roadmap's first-ranked risk: "implicit
generics with structural constraints make inference or error messages bad".

## Decision

1. **Exports are written in full.** An `export`ed function, record field and
   trait method writes every parameter type, its return type and its
   `raises` clause (spec §4.2 rule 1, §12.3). No return type means `Nothing`;
   no clause means it cannot fail. A missing parameter type is V0418, which
   becomes the rule rather than a fallback.
2. **Private code infers.** Inside a module, a parameter used only through
   trait operations is an implicit generic with trait bounds. A parameter
   whose body reads a field or calls an inherent method takes the one
   concrete type the calls in its module pass; two calls that disagree are
   V0425, which asks for an annotation or a generic with a trait bound.
3. **No structural constraints.** Generics are explicit with trait bounds.
   Generic MIR has no `has` bound, and V21 states that no field projection
   has a type-parameter base (13 §13.2, §13.5).
4. **No inferred `Copy` bound.** A generic body that copies a `T` writes
   `T: Copy`, and the definition reports a missing one. V0508 is unassigned.
5. **V0419 is unassigned.** Recursion needs annotations (rule 3), so no
   parameter type grows with recursion depth.
6. **`verse.api` records what the author wrote**, so its review diff is a
   diff of source; the build still compares it (spec §16.4).

## Alternatives

- **Keep inferred exports with `verse.api` as the guard.** The file guards
  against drift but cannot make the constraint language writable.
- **Make structural constraints writable** (`V: has field x: F`). That adds a
  type-system feature to express the inferred form; requiring the
  annotation is smaller.
- **Keep structural constraints for private functions.** Possible, but the
  module's calls already fix the type, and dropping them removes the `has`
  bound from MIR and the whole class of structural error messages.

Prior art: Rust and Swift require full signatures on public functions and
infer inside bodies; OCaml and Haskell infer everywhere and recommend
signatures at module boundaries.

## Consequences

- `fn greet(name)` over every `Show` type cannot be a public API as written;
  it is `export fn greet<T: Show>(name: T): Text`, or a private helper behind
  an annotated export.
- Inference is per module and never needs structural solving; the M1 risk
  in the roadmap is about error messages for trait bounds and module-typed
  parameters only.
- A private helper used from two call sites with different record types
  needs an annotation or a trait.
- [ADR-0002](0002-ownership-on-generic-code.md) holds; its consequence about
  writing inferred `Copy` bounds into `verse.api` no longer has anything to
  write.
