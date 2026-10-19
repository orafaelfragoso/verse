# ADR-0006: Debugging over DAP, with record and replay

Status: accepted (2026-10-18).

## Context

Requirement R7: debugging that is easy. A debugger must show Verse values with
their ownership state, reproduce bugs that depend on timing, and work from
the tools people already use. Debug messages use the same names and wording
as compiler messages.

## Decision

1. The debugger is a **Debug Adapter Protocol** adapter, `verse debug`, on
   standard input and output or a local port
   ([08](../architecture/debugging.md)). Every client uses DAP.
2. Debugging runs on the **bytecode VM**, the semantic reference, with
   **deterministic record and replay**: any run, including a native one, can
   record its nondeterministic inputs and be replayed in the VM. Reverse
   execution is DAP's `stepBack` and `reverseContinue`.
3. Verse-specific data (ownership state, the panic report, Verse stop
   reasons) rides on DAP as extra fields and two `verse/` requests. A client
   that ignores them still has a full debugger.
4. All diagnostics and events share one structured record and one vocabulary.

## Alternatives

- **A Verse-only debug protocol.** Every client would need a new
  integration, and it would duplicate what DAP already covers.
- **gdb/lldb with pretty printers.** Works for native only, and exposes the
  machine, not the language (no ownership state).
- **Print debugging only.** Works, but is slow and lossy.

## Consequences

- Needs the scheduler, clock, random and I/O to be capability-mediated so they
  can be recorded ([05](../architecture/runtime.md), section 5.6).
- Needs `Set` order to be deterministic (insertion order, like `Map`).
- DAP requests that change program state (`goto`, `setVariable`) are not
  supported, because they would break replay.
