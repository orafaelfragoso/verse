# ADR-0019: A cycle among constant initializers is an error

Status: accepted (2026-10-19).

## Context

Spec §12.4 makes a module constant lazy: it is initialized once, on first
use. No doc said what happens when two constants of one module read each
other (`low = high - 10`, `high = low + 10`). A lazy initializer that
re-enters itself either loops or reads a half-built value. V0310 covers
cycles between modules, and [ADR-0010](0010-import-cycles-are-an-error.md)
makes those impossible, so a cycle inside one module is the only one left.

## Decision

1. A cycle among the initializers of one module's constants is a compile
   error, V0426, that names the constants in it (spec §12.4). Its forms
   follow V0310: two constants, three or more ("initialized from itself
   through `b`"), and one constant that reads itself.
2. The graph has an edge from an initializer to every constant of its
   module that the code it could run reads. That code is the initializer,
   the functions of the module it names (called or used as values), the
   functions those name, every lambda written in them, and every method the
   module implements for a type of the module that a value in that code has.
3. The graph needs no other edges. Code in another module reaches this
   module's code only through function values and values of its types that
   this module's code handed over, because that module cannot import this
   one (spec §12.2).
4. Type inference reports it, not name resolution: method calls and the
   types of values are known only after inference, so the code is V0426 in
   the V04xx family (16 §16.2.1).
5. The runtime has no reentry guard: an initializer never re-enters itself.

## Alternatives

- **Leave it undefined.** The docs would promise neither a loop nor a value.
- **Panic on reentry at run time.** It works, but the compiler has the whole
  graph at check, and a failure check can find should not wait for the run.
- **Follow only the calls an initializer actually makes.** That needs
  evaluating the initializer. The rule above over-approximates: a lambda
  that is never called still counts. A refused program has a one-line fix,
  giving one constant a value that does not read the other.
- **A V03xx code, as the issue proposed.** Name resolution does not know
  which method a call reaches, so it could not see a cycle through a method.

## Consequences

- The first use of a constant always sees a finished value, in every tier.
- Shaking and the VM keep treating an initializer as an ordinary instance
  reached through its first read
  ([06 §6.4](../architecture/stdlib-and-shaking.md#64-the-reachability-algorithm)),
  with no reentry state.
- A rare valid program, such as one whose cycle runs only through a lambda
  that is never called, is refused and must be restructured.
