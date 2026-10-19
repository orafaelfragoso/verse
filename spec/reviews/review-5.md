# Review 5: architecture documents 02–05 and 09–12 (Michelangelo)

Scope: readability, wording and contradictions. Technical design is Bob's and
I do not question it here. Verdict: **no blockers**. Eight small fixes, all
about text that went stale after the rulings, plus one clarity item.

## Stale "needs a ruling" text (all of these are now ruled)

| # | Where | Now says | Should say |
|---|-------|----------|------------|
| A5-1 | 04 §4.4, "Static or shared library" | "needs a ruling on foreign interfaces, Q2" | "foreign interfaces are specified in the 0.2 `extern` section (Q2, `reviews/extern-semantics.md`)" |
| A5-2 | 04 §4.5, "Stable ABI" | "once Q2 is ruled on" | same pointer, and drop "once" |
| A5-3 | 05 §5.4, `abort` row | "Needs a ruling: Q5" | "ruled in Q5: panics clean up on every target, and `abort` is an opt-in profile" (use the wording Q5 actually records) |
| A5-4 | 05 §5.5, last bullet | "`Set` needs a ruling for determinism (Q4)" | "`Set` also iterates in insertion order (§9.4, Q4)" |
| A5-5 | 09 §9.4, P7 last column | "Q5 for `abort` mode" | "ruled (Q5)" |
| A5-6 | 11, risks table, `Text` row | "ruling Q3" | "ruled (Q3); benchmark in M1" |
| A5-7 | 12, Q3 and Q4 | "Spec text pending" | "Applied in spec round 7" (Q2 correctly stays "pending (0.2)") |

## Wording and clarity

- **A5-8.** 03 §3.2, the `otherwise` row writes `Present(x)` and `Absent`. The
  spec and the debugger say "present" and `none`. Use `present(x)` and
  `none` so the same word means the same thing in every document, as 08 §8.7
  requires. (`Option<DropFlag>` in the MIR block is Rust-style type notation
  for the compiler's own data and may stay, but say once in 03 that MIR
  notation is not Verse.)
- **A5-9.** 05 §5.6 shows `capability fs { open, read, … }`. It looks like
  Verse syntax and Verse has no such form. Add one line above it: "This is a
  description of the interface, not Verse code."
- **A5-10.** 02 §2.1 draws the pipeline without the Wasm library frontend that
  07 §7.4 and 04 §4.4 describe (a Wasm component is translated to mono MIR).
  Add it to the diagram as a second input into `Mono MIR`, or one sentence
  under the diagram. A reader who starts at 02 should not meet it first in 07.

## Checked and fine

- No banned word from the 08 §8.7 table in 02–05 and 09–12, outside places that
  name Wasm features (`exception handling`, `throw`) or other products, which is
  correct.
- Diagnostics are data first (02 §2.8) and agree with 08's one-vocabulary rule.
- 09 states every number as a target with a gate, and 10 ties tests to spec
  sentences. Both are plain to read.
- 11 gives no durations, which is honest.

## Owners

Bob applies A5-1 to A5-10 (his files); Picasso republishes.
