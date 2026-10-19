# ADR-0007: One task semantics, three native lowerings and VM tasks

Status: accepted (2026-10-18, all three owners).

## Context

Spec §11: lightweight tasks, no `async`/`await`, no function coloring, many
cores. Targets include Wasm and microcontrollers, which have no threads or no
stack switching.

## Decision

Define task behavior once (the conformance suite) and implement it with
**three lowerings in compiled code**: **stackful tasks** on native targets,
**compiler-generated state machines** for functions that may yield on Wasm
targets without stack switching, and a **cooperative executor** (the same state
machines, no threads) on single-core devices. Tier 0 adds a **fourth
implementation that is not a lowering**: the bytecode VM runs tasks as its own
coroutines (a stack chain and call records per task, switched by the
interpreter loop; [14 §14.7](../14-bytecode-and-vm.md#147-tasks-scheduler-and-seed)).
The "may yield" set is a compiler-internal effect analysis that never appears in
user code.

## Alternatives

- **OS threads per task:** simple, but 1 million tasks is not possible.
- **`async` functions in the language:** rejected by the spec (coloring).
- **Green threads on every target, including a Wasm interpreter shim:** slow.
- **Continuation-passing everywhere:** uniform, but slows ordinary code.

## Consequences

- The `task-lower` pass, the three runtime schedulers and the VM's scheduler
  must be tested against the same suite (differential task tests with a seeded
  scheduler, whose policy is specified once in 14 §14.7 and implemented by
  `verse-rt` in its single-worker seeded mode).
- Functions outside the may-yield set pay nothing on any target.
