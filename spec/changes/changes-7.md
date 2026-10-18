# Changes 7 — Round 7 (accepted rulings)

Line numbers are in the files after the change. `00-README.md` was not edited:
the section status table is unchanged and nothing is reopened. Section
numbering is unchanged. Architecture files by Bob and Michelangelo were not
touched.

| Item | Resolution | File: lines |
|------|------------|-------------|
| R7-1 `Set` order | §9.4 "unordered" replaced by insertion order like `Map`; set-operation order; §9.2 re-insert sentence; §18.15 bullet. Grep for `unordered`/order of sets in 15, 18, 19, 20 found nothing else | `09-collections.md`: 28–30, 108–110, 119–121; `18-design-notes.md`: 323–324 |
| R7-2 `Text` indexing | Linear-cost sentence and `text.bytes()` view (constant-time index, `Int` 0–255, §9.3 view rules, `.copy()` to `Bytes`). `bytes()` already existed in §4.1 and §20.2; no other `Text` method clashes | `04-types.md`: 19–26 |
| R7-3 unused imports | Paragraph: warning, no suppression (delete the line), imports add no code | `12-modules.md`: 57–61 |
| R7-4 tool table | Four rows | `16-tooling.md`: 14–17 |
| R7-5 `[library]` | Paragraph, toml example (native table marked 0.2), first-party packages sentence | `16-tooling.md`: 90–108 |
| R7-6 domain libraries | Sentence after the core-modules table | `15-stdlib.md`: 51–52 |
| R7-7 `extern` | §18.15 bullet "Reserved for 0.2" | `18-design-notes.md`: 345–348 |
| Proposal | 0.2 `extern` semantics draft, E1–E11, following `reviews/extern-readability.md` | `reviews/extern-semantics.md` (new) |
| Log | "Round 7 (accepted rulings)" table | `21-review-log.md`: 289–302 |
| Build | `build.sh` run | — |
