# Changes 6 — Review 4 (Michelangelo full read)

Rulings on `reviews/review-4.md` plus R4-14. Line numbers are in the files
after the change. `00-README.md` was not edited (section status is set by the
reviewer). The binding checker was rerun on all examples (only the
intentional error lines and known false positives), and greps for `?.` and
`let` in code examples found none left.

| Item | Resolution | File: lines |
|------|------------|-------------|
| R4-1 visible = locals | §3.9 definition of "visible"; paragraph on hiding prelude/imported names, collision warning, `std.log` `error` vs prelude; grammar note | `03-lexical.md`: 533–534, 557–568; `grammar.ebnf.md`: 93–94 |
| R4-2 `a?.b` error | §3.8 level-1 note; §8.1 paragraph; grammar note; §16.1 formatter sentence; §18.11 bullet | `03-lexical.md`: 302–306; `08-errors.md`: 100–107; `grammar.ebnf.md`: 190–192; `16-tooling.md`: 32; `18-design-notes.md`: 245–248 |
| R4-3 getters | §4.2 rule 9 rewritten (inferred `Copy` bound, call-site error example); §5.3 example (annotated `first_name`, unannotated `age_of`) and paragraph; §18.10 bullet | `04-types.md`: 113–117, 120–122, 124–130; `05-functions.md`: 124, 126, 129, 131–133, 135–136; `18-design-notes.md`: 193–196 |
| R4-4 raw text | `std.json` example uses a multi-line `r"""…"""`; §3.7.3 raw-text sentence expanded | `15-stdlib.md`: 79–82; `03-lexical.md`: 275–279 |
| R4-5 destructuring | §3.9 paragraph + swap example; §7.7 pointer; grammar note; tour swap | `03-lexical.md`: 516–531; `07-pattern-matching.md`: 176–178; `grammar.ebnf.md`: 94–98; `02-tour.md`: 64–68 |
| R4-6 nested literals | §3.7.2 one-level rule; style note; grammar note | `03-lexical.md`: 236–238, 253–257; `grammar.ebnf.md`: 242–243 |
| R4-7 §4.9 sentence | Rewritten | `04-types.md`: 322–326 |
| R4-8 `10.seconds()` | §3.7 sentence; grammar `FLOAT` note | `03-lexical.md`: 155–157; `grammar.ebnf.md`: 235–236 |
| R4-9 four ways out | §8.1 tool table row `.or_error`; "four ways out" | `08-errors.md`: 46, 62–63 |
| R4-10 §3.8 row 1 | Row shortened; "Notes on level 1" list | `03-lexical.md`: 287, 298–307 |
| R4-11 counting idiom | Declined; idiom already present | — (`09-collections.md`: 71, unchanged) |
| R4-12 nested optionals | Style note | `08-errors.md`: 110–113 |
| R4-13 `.to_list()` | Confirmed absent (RF-32) | — (`11-concurrency.md`: 184, unchanged) |
| R4-14 call-site sentence | Same sentence in §4.2 rule 9, §5.3, §10.6 | `04-types.md`: 133–135; `05-functions.md`: 137–139; `10-traits-generics.md`: 169–172 |
| Log | "Review 4 (Michelangelo full read)" table | `21-review-log.md`: 262–282 |
| Build | `build.sh` run | — |

## Re-verification (R5-1…R5-5)

| Item | Resolution | File: lines |
|------|------------|-------------|
| R5-1 help text | "return `person.name.copy()`, or take the parameter with `take`" | `04-types.md`: 129–130 |
| R5-2 formatter list | One list | `16-tooling.md`: 27–30 |
| R5-3 import warning | Warning on the import line, stated in the §3.9 hiding rule (no §16 diagnostics list exists) | `03-lexical.md`: 568–570 |
| R5-4 pointers | §12.2 and §15.1 one-line pointers to §3.9 | `12-modules.md`: 53–55; `15-stdlib.md`: 10–11 |
| R5-5 §18.9 | Locals-only and swap clauses | `18-design-notes.md`: 132–135 |
| Log | Rows R5-1…R5-5 appended to the Review 4 (full read) table | `21-review-log.md`: 283–287 |
| Build | `build.sh` run; binding checker unchanged | — |
