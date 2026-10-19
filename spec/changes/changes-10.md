# Changes 10 — Round 10 (cleanup, drop order, unwinding, NaN)

Line numbers are in the files after the change. `changes-9.md` was closed by the
publish of 21:30 (America/Sao_Paulo) and is not edited. `00-README.md` was not
edited: the section status table is unchanged and nothing is reopened. Section
numbering is unchanged. Nothing under `architecture/` or `reviews/` was touched.
Q19 (the debugger step) is architecture-only and changes no spec text.

| Item | Resolution | File: lines |
|------|------------|-------------|
| R10-1 `cleanup` may block | §13.1 paragraph: `cleanup` may block without limit; a task blocked inside one still counts as blocked for deadlock detection; only `exit` ignores it | `13-resources.md`: 48–51 |
| R10-2 cancellation masked | §11.5 paragraph: a cancel request arriving during a cleanup takes effect at the first blocking point after it | `11-concurrency.md`: 286–288 |
| R10-3 drop order | §4.10 point 4: locals in reverse order of declaration (a later local may borrow an earlier one); fields and tuple elements in declaration order, list elements by index, `Map` and `Set` in insertion order | `04-types.md`: 491–495 |
| R10-4 parent that unwinds | §11.3 paragraph: a panicking or cancelled function cancels its unwaited tasks, then waits; cancelled-only children print nothing, a child's own panic or error is reported and counts for the exit status | `11-concurrency.md`: 137–142 |
| R10-5 `NaN` and the sign of zero | §3.8.4 bullet: every observable `NaN` is one quiet `NaN`, `Show` prints `NaN` and `-0.0`, the sign of zero is kept; `Float` is not `Hash` (§9.4), so no hashing wording | `03-lexical.md`: 470–473 |
| Log | "Round 10" table | `21-review-log.md`: 331–343 |
| Build | `build.sh` run (Markdown only) | — |
