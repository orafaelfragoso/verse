# ADR-0023: The stack-check headroom is set per target

Status: accepted (2026-10-19).

## Context

The prologue stack check fires `headroom` above the real end of a task's
stack, so cleanup blocks and the panic path run in the reserve (15 §15.6.4).
The headroom was an assumption of 4 KiB on small targets. The Cortex-M target
gives a task `max_stack = "4K"` (15 §15.9.3), so the limit sat at the start
of the stack and every non-leaf prologue would fail the check.

## Decision

1. The headroom is a key of the target file, `[tasks] headroom`, 16 KiB by
   default (15 §15.9.2). The Cortex-M target sets 512 bytes.
2. It must hold the cold block, `rt_panic` with its streaming report, and the
   glue that cleanup blocks call. That glue's depth is bounded by the types,
   because deep data goes through the drop stack
   ([ADR-0022](0022-drop-stack.md)). That 512 bytes hold this path on
   Cortex-M is an assumption, measured in M7.
3. A target file whose headroom is more than half of `max_stack`, or of
   `[memory] stack` when it is set, is refused when it loads, with V0911
   (`{key}` cannot be used with `{other}`). No new code is needed.

## Alternatives

- **Keep 4 KiB and raise the Cortex-M stack.** It costs RAM on the targets
  that have the least.
- **Run the panic path on a separate fixed stack.** The headroom could then
  be near zero, but the runtime needs a stack switch on every target, and
  cleanup blocks still run on the task's stack.

## Consequences

- A small target can give a task a stack of a few KiB and still have a
  working stack check.
- A target author who sets a too-large headroom gets a load-time error, not
  a program whose every call fails.
- The M7 measurement of the panic path decides the Cortex-M value; a larger
  measured value raises the headroom in the target file, not in the compiler.
