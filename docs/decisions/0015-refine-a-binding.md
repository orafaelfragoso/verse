# ADR-0015: A binding can be refined by consuming it

Status: accepted (2026-10-19).

## Context

Spec §3.9 binds a name once per function. A value transformed step by step
needed a new name at every step (`raw`, `text`, `lines`), and because
`x = …` is both a new binding and a reassignment, the spec required editors
and `verse fmt --check` to render a reassignment differently from a new
binding. A syntax that needs an editor to be read works against spec §1.1.

## Decision

1. `x = expr` on a visible immutable `x` is a **refinement** when `expr` uses
   `x` and `x` is bound in the same block (a parameter counts as bound in the
   function's body). The right side is evaluated with the old `x`, which is
   its last use, so the old value moves into it (spec §4.9). Then `x` names
   the new value, which may have another type.
2. It is not shadowing, because the old value is gone, and not reassignment,
   because `x` was never `mut`. A `mut` name is reassigned as before.
3. Only a plain name, with or without a type annotation, is refined. A
   pattern on the left always binds new names (spec §3.9, §7.7).
4. A name bound before a loop or an `if` cannot be refined inside it: the old
   value would be consumed on one path only. That stays V0302.
5. Every `x = …` line reads one of three ways from its text: the right side
   mentions `x` (a refinement), `mut x` was bound earlier (a reassignment),
   or neither (a new binding, the first `x`). The requirement that tools
   render reassignments distinctly is dropped.
6. V0303's help adds "or use `{name}` on the right side to refine it". V0313
   is unchanged: a refinement has the name on both sides, so it is never one
   edit away from another name.
7. In MIR a refinement is a new `binding` local with the same user name
   (13 §13.7). At any pc at most one local of a name is live.

## Alternatives

- **Add `let` and allow shadowing.** Rejected for the reason spec §18.9 drops
  `let`; a refinement reaches the same readability with no keyword.
- **Allow any rebinding after a last use** (`x = 5` after an unrelated `x`).
  The reader sees `x = 5` and cannot tell it is new; the refinement form is
  the one that reads.
- **Keep the editor requirement.** It makes the meaning of a line depend on
  the tool that shows it.

Prior art: Rust's `let x = f(x)` (shadowing with a keyword); OCaml and Elm
rebinding in `let … in`.

## Consequences

- Pipelines of transformations keep one name (`text = text.trim()`), and the
  type may change along the way.
- Name resolution checks the refinement rule (02 §2.2), and the ownership pass
  sees an ordinary last use of the old local.
- A refinement inside a nested block of an outer name is an error with the
  usual V0302 message, so a value is never consumed on one path only.
