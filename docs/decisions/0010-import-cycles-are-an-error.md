# ADR-0010: Import cycles are an error

Status: accepted (2026-10-19).

## Context

A file is a module, names are private unless marked `export`, and every name
in a file traces to a declaration or an import line (spec §12.1 to §12.3).
Imported modules hold only declarations and module-level constants, and a
constant is initialised lazily on first use (spec §12.4).

If two modules may import each other, a reader of `a.vs` cannot tell from its
imports whether `b.vs` depends back on it. Allowing such cycles for
declarations still needs a rule for the case where a constant on each side
reads the other, and whether that case fails depends on which constants a
program reads first.

## Decision

The import graph of a program is acyclic. Name resolution
([02 §2.2](../architecture/compiler-architecture.md#22-stage-contracts))
refuses a cycle with V0310 before any code runs, in `verse check`,
`verse build` and script mode alike. The diagnostic names every edge of the
cycle, each in its own file:

- `a` is the module of the cycle that comes first in path order, so the report
  does not depend on the order in which modules are checked. `b` is the module
  `a` imports.
- The message has three forms. A cycle of two modules reads "modules `a` and
  `b` import each other". A cycle of three or more reads "module `a` imports
  itself through `b`". A module that imports itself reads "module `a` imports
  itself".
- The primary label is on the import of `b` in `a`. Its text names the import
  that closes the cycle: "`b.vs:1` imports `a`". In a cycle of three or more
  modules, a secondary label marks every other import line in the cycle.
- The help is "move what they share into a module none of them imports".

Each cycle is reported once. Declarations that refer to each other, such as two
mutually recursive types, live in one module.

## Alternatives

- **Allow cycles for declarations, refuse a cycle in constant initialisation.**
  It costs one more rule, and the failure depends on which constants a program
  reads first.
- **Allow cycles and initialise constants eagerly at import.** Importing would
  then run code, against spec §12.4, and the module that loads second sees a
  half-initialised module, as in Python and TypeScript.

Prior art: Rust refuses a cycle between crates, and Swift Package Manager
refuses a cycle between targets. Python and TypeScript allow import cycles, and
a module in a cycle can see another one half-initialised.

## Consequences

- Nothing that a file imports depends back on it, directly or through other
  modules.
- A constant-initialisation cycle across modules cannot arise, so the
  constant rule of spec §12.4 needs no cycle case.
- The modules of a program form a directed acyclic graph, so a module is
  resolved and checked after the modules it imports, against their
  interface records (spec §12.3).
- Two types in different files that refer to each other move together into
  one module, often a third one that both files import.
