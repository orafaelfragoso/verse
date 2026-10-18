# ADR-0006: MCP-first debugging with record and replay

Status: proposed.

## Context

Requirement R7: debugging that LLMs can drive step by step, in the spirit of
the Model Context Protocol. Michelangelo's rule: debug messages use the same
names and wording as compiler messages.

## Decision

1. The primary debug interface is an **MCP server** (`verse mcp`) over stdio
   and HTTP, exposing structured tools ([08](../08-debugging-and-mcp.md)).
   A Debug Adapter Protocol bridge is secondary, for editors.
2. Debugging runs on the **bytecode VM**, the semantic reference, with
   **deterministic record and replay**: any run, including a native one, can
   record its nondeterministic inputs and be replayed in the VM with
   time-travel.
3. All diagnostics and events share one structured record and one vocabulary.

## Alternatives

- **DAP only.** Editor oriented; too chatty and unbounded for an LLM context
  window.
- **gdb/lldb with pretty printers.** Works for native only, and exposes the
  machine, not the language (no ownership state).
- **Print debugging only.** Works, but is slow and lossy.

## Consequences

- Needs the scheduler, clock, random and I/O to be capability-mediated so they
  can be recorded ([05](../05-runtime.md), section 5.6).
- Needs `Set` order to be deterministic (Q4 in [12](../12-open-questions.md)).
- Output sizes are bounded by explicit budgets in every tool call.
