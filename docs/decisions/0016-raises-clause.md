# ADR-0016: A `raises` clause keeps error sets exact

Status: accepted (2026-10-19).

## Context

A fallible type was a union, and a specific error type next to `Error` was
absorbed: `Text | FetchError | Error` normalized to `Text | Error`, and once a
value was widened to `Error`, `?` could not narrow it back. `.wait()` always
widened to `T | Error`. A caller's view of what could fail was coarser than
what the body did, and recovering the type took a run-time `match` on a fact
the compiler had.

## Decision

1. **The fallible type is `T raises E`.** `E` is a raise set of error types
   joined with `|`. Every member implements `Error`. An error type is never a
   member of a union (V0424). `raises Error` is the open set and stands alone
   (V0423) (spec §4.6).
2. **Sets are exact.** `?` keeps every member (spec §8.3), a task keeps the
   set of the function it runs, and `wait()` adds `TaskPanicked` and
   `Cancelled` (spec §11.1). A function type carries its set, and a function
   with a smaller set fits a type with a larger one (spec §4.7).
3. **Checked both ways.** An error the body can raise that the clause does not
   list is an error (V0420). A listed member that nothing can raise is a
   warning (V0421). The open set is never warned about (spec §8.2).
4. **Inference.** A function whose return type is written and has no clause
   cannot fail. A function whose return type is inferred has its set inferred
   too (spec §4.2 rule 5). `verse.api` records the exact set (spec §12.3).
5. **`otherwise return`.** A bare `otherwise return` on a fallible value, in a
   function whose clause could carry that error, is an error (V0422): `?`
   would pass the reason on. In a function that cannot fail, the bare form
   still discards the error deliberately, and `otherwise return value` is
   always allowed (spec §3.8.2). The proposal also allowed the bare form when
   the clause can carry the error; that is the case where discarding is most
   likely a mistake, so this decision refuses it there instead.
6. **MIR keeps unions.** `A raises E` is the MIR union `A | E`, with the
   success cases first and the set exact (13 §13.3); layout and `?` lowering
   are unchanged.
7. **`raises` is a keyword** (spec §3.6).

## Alternatives

- **Keep union absorption.** It loses information by construction.
- **Infer the set with no clause, everywhere.** Possible for private
  functions, and done for those with an inferred return type; at an exported
  boundary the set is part of the signature `verse.api` records.
- **Checked exceptions as in Java.** Spec §8 keeps errors as values; `raises`
  declares a value's type and adds no control flow.

Prior art: Swift's typed `throws`; Zig's error sets, which are exact and
inferred for private functions.

## Consequences

- Callers and `match` see exactly what can fail, and `wait()` no longer
  widens.
- Adding a failure mode to a library function changes its signature and
  shows in a `verse.api` diff.
- Application code that does not care keeps writing `raises Error` and
  `.context("…")?`.
- The type checker owns V0420 to V0424, in the V04xx family (16 §16.2.1).
