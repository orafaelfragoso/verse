# ADR-0007: One task semantics, three lowering strategies

Status: proposed.

## Context

Spec §11: lightweight tasks, no `async`/`await`, no function coloring, many
cores. Targets include Wasm and microcontrollers, which have no threads or no
stack switching.

## Decision

Define task behavior once (the conformance suite) and implement it with:
**stackful tasks** on native targets, **compiler-generated state machines**
for functions that may yield on Wasm and embedded targets, and a
**cooperative executor** on single-core devices. The "may yield" set is a
compiler-internal effect analysis that never appears in user code.

## Alternatives

- **OS threads per task:** simple, but 1 million tasks is not possible.
- **`async` functions in the language:** rejected by the spec (coloring).
- **Green threads on every target, including a Wasm interpreter shim:** slow.
- **Continuation-passing everywhere:** uniform, but slows ordinary code.

## Consequences

- The `task-lower` pass and three runtime schedulers must be tested against the
  same suite (differential task tests with a seeded scheduler).
- Functions outside the may-yield set pay nothing on any target.
