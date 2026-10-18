# Changes 5 — Rafael's changes (RF-1…RF-3)

Rafael's three language changes, agreed by Michelangelo, applied throughout
`/workspace/verse/`. Line numbers are in the files after the change.
`00-README.md` was not edited (no `let`, `?.` or imports in it; section status
is set by the reviewer). Review-log history rows were left unchanged.

| Item | Resolution | File: lines |
|------|------------|-------------|
| RF-1 rules | New "Bindings and assignment" subsection in §3.9: no `let`; binding vs reassignment table with the immutable-name error; typed and destructuring bindings; refutable only with `otherwise`; capitalisation; no shadowing (parameters, loop variables, pattern and `with … as` names; sibling blocks); the two warnings with a sample diagnostic | `03-lexical.md`: 452–514 |
| RF-1 summary | §1.3 bullet: `x = 1` / `mut count = 0`, bound once per function; §1.1 keyword example `let` → `mut` | `01-philosophy.md`: 12, 46–49 |
| RF-1a destructuring / refutable | §3.8.2 wording ("a binding with a refutable pattern", "`x = … otherwise return` is the guard"); §7.7 heading "Destructuring in bindings and `for`" and text; §18.7 | `03-lexical.md`: 340, 407, 420; `07-pattern-matching.md`: 173, 175–176, 186–188; `18-design-notes.md`: 94–96, 101 |
| RF-1b `mut` = "can change" | §4.9 mutability bullets rewritten, reassignment sentence, error comment; §5.3 paragraph (also: parameters count as bindings); §5.6 "immutable binding" | `04-types.md`: 418–429, 432–434; `05-functions.md`: 64–69, 339–341 |
| RF-1c no shadowing | §18.9 rewritten (heading "Immutability, ownership, no shadowing"; decision + why; supersedes inner-block shadowing); grammar note | `18-design-notes.md`: 120, 122–134, 139–148; `grammar.ebnf.md`: 88 |
| RF-1c entry file / module constants | §12.3/§12.4: "module-level constant", "top-level binding" (M-2 rules unchanged) | `12-modules.md`: 72, 127–128, 131–132, 134, 146, 148, 152; `18-design-notes.md`: 290 |
| RF-1d diagnostics | Unused-binding and edit-distance-1 warnings | `03-lexical.md`: 495–512 (inside 452–514); `18-design-notes.md`: 131–134 |
| RF-1e `let` reserved | Kept in §3.6 keyword list; new note with the friendly error; grammar note | `03-lexical.md`: 130–133; `grammar.ebnf.md`: 90 |
| RF-1f grammar | `binding = [mut] pattern [: type] = expr` (+ `otherwise exit` form), semantic note (binding vs assignment, capitalisation); top-level item list and `statement` use `binding`; `let _ =` → `_ =` | `grammar.ebnf.md`: 18, 72, 81–91 |
| RF-1f prose `let` | Doc-comment list, naming table, `if let` remarks, views ("a binding to a chain"), §19 table (Verse `x = …` / `mut x = …`; Rust restored to `let` / `let mut`), §18 tables | `03-lexical.md`: 22, 111, 358; `04-types.md`: 329; `06-control-flow.md`: 62–63; `07-pattern-matching.md`: 157–158; `09-collections.md`: 95, 124; `18-design-notes.md`: 179, 230–231, 369, 392; `19-comparison.md`: 9 |
| RF-1f code `let` | 214 mechanical rewrites in code blocks and inline spans (comment columns preserved) | `02-tour.md`: 29, 38, 48–49, 55, 58, 64, 68; `03-lexical.md`: 79, 84, 87, 114, 242, 313–314, 342, 381, 395, 399, 402, 412; `04-types.md`: 31–34, 78, 100, 160–164, 178–179, 230, 232, 242, 244, 274, 320, 327, 339, 348–349, 354–355, 358–359, 367–368, 385, 410–411, 436, 459; `05-functions.md`: 15, 48, 81–82, 163–166, 177–179, 191, 217–221, 245, 250, 253, 267–269, 282, 323, 359, 368; `06-control-flow.md`: 24–25, 33–34, 93, 120, 126, 187; `07-pattern-matching.md`: 20, 167, 179–181, 191, 211; `08-errors.md`: 9–10, 32–33, 111, 184–185, 209–213, 228–230, 253, 267, 275, 278, 290, 294, 296; `09-collections.md`: 8, 17, 20, 32, 37, 40, 97, 110–111, 113, 128, 130, 187, 202, 204, 208; `10-traits-generics.md`: 60, 108, 131; `11-concurrency.md`: 23, 26–27, 30, 60, 78–79, 81–82, 95, 168, 196, 201–202, 213, 219; `12-modules.md`: 65, 137, 141–143, 157; `13-resources.md`: 99, 105, 112; `15-stdlib.md`: 58, 60–61, 65, 79; `18-design-notes.md`: 183, 249, 390; `20-examples.md`: 23–24, 28, 42, 44, 52, 54–55, 57, 62, 66, 77, 83, 122–123, 132, 137–138, 141, 145, 147, 169, 235, 249, 260, 272 |
| RF-1f rename | Second `n` in the §3.8.2 `otherwise` operand example renamed `count` | `03-lexical.md`: 383 |
| RF-1f tour | Comment on `mut count = 0` | `02-tour.md`: 58 |
| RF-2 lifting rules | §8.1 intro table (`?` jobs), new "Plain dots through optionals" subsection: (a)–(d), error example, short-circuit, one flattened level, view rule, optional methods win, ambiguity error; `?.` row removed from tools table; §8.3 pointer | `08-errors.md`: 16, 18–20, 29, 47, 49–51, 53–75, 78–100, 104, 237 |
| RF-2 grammar | `"?." IDENT` removed from `postfix`; lifted-chain semantic note; fallback-operand comment; continuation intro | `grammar.ebnf.md`: 6–7, 144, 159–165 |
| RF-2 §3 | Continuation rule 3; precedence row 1; `otherwise` example; ownership paragraph; right-operand list | `03-lexical.md`: 60, 273, 330–334, 363–365, 376–377 |
| RF-2b `?` meanings | §1.2 table: three meanings | `01-philosophy.md`: 32 |
| RF-2c examples | Tour; §5.5; §9.3 (`x?.copy()` sentence → optional's `.copy()`); §9.5; §10.5/§10.6; §16.2 help | `02-tour.md`: 52; `05-functions.md`: 292, 298; `09-collections.md`: 88–89, 194–197, 210–211; `10-traits-generics.md`: 117, 148; `16-tooling.md`: 48–50 |
| RF-2c rationale | §18: `otherwise` example, flattening bullet, new "No `?.` operator; the consumer decides" bullet, `?` meanings | `18-design-notes.md`: 59, 216–218, 232–242 |
| RF-3 syntax | §12.2 example and prose (`import x, y from M`, per-name `as`, line always starts with `import`); grammar import rule | `12-modules.md`: 24–26, 30–36; `grammar.ebnf.md`: 23–25 |
| RF-3 examples | All item imports rewritten | `02-tour.md`: 9; `08-errors.md`: 181, 264; `11-concurrency.md`: 20, 76, 106, 166, 194, 211; `13-resources.md`: 24, 52, 81; `15-stdlib.md`: 30, 56; `16-tooling.md`: 76; `20-examples.md`: 109–110 |
| RF-3 rationale | §18.14 | `18-design-notes.md`: 293–294 |
| Log | "Review 4 / Rafael's changes" table (RF-1…RF-3 with sub-rows) | `21-review-log.md`: 204–226 |
| Build | `build.sh` run; `verse-spec.md`/`.html`/`.pdf` regenerated | — |

## Follow-up rulings (RF-4…RF-15)

Applied on Oct 18, 2026 after the reviewer's rulings on the 11 open questions,
with ruling 11 revised to the channel split (RF-15). Line numbers are in the
files after this round.

| Item | Resolution | File: lines |
|------|------------|-------------|
| RF-1g confirmed | `_` prefix suppresses the unused warning (text already present); parameters count as bindings (§5.3 sentence already present) | — (no edit) |
| RF-4 import parsing | §12.2: trailing `from` on the same logical line decides; one line or a parenthesised list, with example; grammar: parenthesised `import_list` and note | `12-modules.md`: 36–48; `grammar.ebnf.md`: 23–30 |
| RF-5 reassign rendering | Tooling requirement paragraph (inlay hint "reassign") | `03-lexical.md`: 521–524 |
| RF-6 scope rule | "You cannot bind a name that is currently visible"; inner-then-after allowed, before-then-inside forbidden; example `fn report` | `03-lexical.md`: 488–506 |
| RF-7 lambda parameters | Listed among bindings; example line `items.map(count -> …)` | `03-lexical.md`: 489, 505 |
| RF-8 `mut` parameter assignment | Paragraph + `fn reset` example in §3.9 and §5.3 | `03-lexical.md`: 513–519; `05-functions.md`: 69–78 |
| RF-9 unused warning | Warning only; snippets exempt; no warnings-as-errors | `03-lexical.md`: 529–530 |
| RF-10 `.or_error` | New lifted-chain bullet | `08-errors.md`: 94–96 |
| RF-11 parentheses | §8.1 paragraph (parens end a chain; fmt keeps them); §16.1 formatter sentence | `08-errors.md`: 100–102; `16-tooling.md`: 31–32 |
| RF-12 `MAX_RETRIES` | Error wording | `03-lexical.md`: 114–117 |
| RF-13 `fn port(config)` | One-line note after the §12.4 example | `12-modules.md`: 177–179 |
| RF-14 "last handle" | Superseded by RF-15; nothing applied separately (no "last sender" wording outside the R3-12 history row) | — |
| RF-15 channel split | §11.4 handle table (`Sender<T>`, `Receiver<T>` rows), `Copyable` paragraph, `Weak` sentence, channel prose, example, operations table, `for value in rx` | `11-concurrency.md`: 145–146, 152, 156–160, 167–171, 176–177, 179–181, 183–187, 189–190, 194–197, 199–200 |
| RF-15 §11.1 / §11.5 | Handle-copy bullet (`Sender`; `Receiver` moves); blocking points and table use `recv` | `11-concurrency.md`: 67, 71–73, 254, 271 |
| RF-15 Copyable lists | §4.10 point 5; §10.4 `Copyable` row; §15.2 `std.task` row (+ `Closed`); §18.10; §18.13 | `04-types.md`: 476, 479–480; `10-traits-generics.md`: 79; `15-stdlib.md`: 41; `18-design-notes.md`: 159–160, 274, 280–282 |
| RF-15 iterables | "channel receivers" in §6.3 and §10.4 `Iterable`; §1.3 handle list | `06-control-flow.md`: 68; `10-traits-generics.md`: 75; `01-philosophy.md`: 58 |
| Log | Rows RF-1g, RF-4…RF-15d appended to the Review 4 table | `21-review-log.md`: 227–243 |
| Build | `build.sh` run | — |

## Follow-up rulings (RF-16…RF-25)

Line numbers are in the files after this round. The binding checker was rerun
on all examples: only the intentional error lines of §3.9 and known false
positives were reported.

| Item | Resolution | File: lines |
|------|------------|-------------|
| RF-16 §3.9 error row | "immutable binding or immutable parameter" | `03-lexical.md`: 479 |
| RF-17 `as mut value` | §3.9 sentence; §11.4 table cell, example, paragraph, C-1 paragraph; §13.1 sentence; grammar `resource` | `03-lexical.md`: 512–514; `11-concurrency.md`: 148, 240, 250–255, 261–262; `13-resources.md`: 35–36; `grammar.ebnf.md`: 107–109 |
| RF-18 explicit type arguments | Grammar postfix alternative `"." IDENT type_args call` + note; §11.4 annotation alternative | `grammar.ebnf.md`: 165–166, 175–179; `11-concurrency.md`: 171–175 |
| RF-19 capacity | Default 16, 0 = rendezvous (prose and operations table) | `11-concurrency.md`: 171–173, 205 |
| RF-20 worker example | Example rewritten (`take tx`, tasks returned, consume then wait, 20 items / capacity 4); deadlock sentence | `11-concurrency.md`: 178, 183–184, 186–191, 194–195 |
| RF-21 blocked senders | `send` row | `11-concurrency.md`: 206 |
| RF-22 `send` fallible | Sentence pointing to §8.4 | `11-concurrency.md`: 196–197 |
| RF-23 §4.10 point 5 | `Receiver<T>` removed from the sharing list | `04-types.md`: 476–478 |
| RF-24 `Closed` | Named error type (§11.4 sentence, §15.2 row) | `11-concurrency.md`: 197–198; `15-stdlib.md`: 41 |
| RF-25 postfix `?` | §3.8 precedence row; §8.1 paragraph; §16.1 formatter sentence; grammar note | `03-lexical.md`: 274; `08-errors.md`: 98–106; `16-tooling.md`: 31–32; `grammar.ebnf.md`: 180–182 |
| Log | Rows RF-16…RF-25 appended to the Review 4 table | `21-review-log.md`: 244–253 |
| Build | `build.sh` run | — |

## Follow-up rulings (RF-26…RF-31)

Line numbers are in the files after this round. Binding checker rerun: no new
findings.

| Item | Resolution | File: lines |
|------|------------|-------------|
| RF-26 type arguments after a type name | Grammar `postfix` split (`TYPE_IDENT "." IDENT type_args call`, `postfix_op`) and note; §3.8.3 sentence | `grammar.ebnf.md`: 165–167, 176–181; `03-lexical.md`: 431–435 |
| RF-27 `.to_list()` | §11.4 worker and `Guarded` examples; §9.5 sentence | `11-concurrency.md`: 184, 243; `09-collections.md`: 183–186 |
| RF-28 import | `wait_all` added | `11-concurrency.md`: 235 |
| RF-29 file writes | `as mut target` in §13.3; §13.1 paragraph | `13-resources.md`: 87, 37–40 |
| RF-30 §4.10 | `Task<T>` removed from the sharing list | `04-types.md`: 476–479 |
| RF-31 free generic functions | §10.5 paragraph | `10-traits-generics.md`: 133–139 |
| Log | Rows RF-26…RF-31 | `21-review-log.md`: 254–259 |
| Build | `build.sh` run | — |

## Follow-up ruling (RF-32)

| Item | Resolution | File: lines |
|------|------------|-------------|
| RF-32 eager chains | §9.5 intro reworded (bound/returned chains collected at the end of the statement; only `for` targets and mid-chain uses stay lazy); RF-27 sentence reworded (owned list, side effects at statement end, `to_list()` not needed for binding) | `09-collections.md`: 122–128, 186–191 |
| RF-32 revert | `.to_list()` removed from the worker and `Guarded` examples (RF-27 reverted; `wait_all` import from RF-28 kept) | `11-concurrency.md`: 184, 243 |
| Log | Row RF-32 | `21-review-log.md`: 260 |
| Build | `build.sh` run; binding checker rerun, no new findings | — |
