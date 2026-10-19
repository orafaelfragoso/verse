# Changes 8 — Round 8 (architecture follow-ups)

Line numbers are in the files after the change. `00-README.md` was not edited:
the section status table is unchanged and nothing is reopened. Section
numbering is unchanged. Bob's and Michelangelo's architecture files were not
touched.

| Item | Resolution | File: lines |
|------|------------|-------------|
| R8-1 package names | §16.3 example `word-tools` → `word_tools`; identifier rule `[a-z][a-z0-9_]*` added to the import bullet | `16-tooling.md`: 65, 81–84 |
| R8-2 `verse why` | Tool-table row is `verse why <item-or-path>`, told apart by form (item → chain of uses; path → generating plugin) | `16-tooling.md`: 15 |
| R8-3 `verse build` | Tool-table row: builds for the selected profile and target (dev, release, WebAssembly, bytecode) | `16-tooling.md`: 13 |
| R8-4 variants of an imported type | §12.2 paragraph, example and error wording (review-6 R6-2); the qualified form is in §4.4, not §4.10 | `12-modules.md`: 63–82 |
| R8-5 byte buffer | §18.15 bullet "Reserved for 0.2: a fixed-size mutable byte buffer", next to the `extern` bullet | `18-design-notes.md`: 349–352 |
| Log | "Round 8 (architecture follow-ups)" table | `21-review-log.md`: 305–316 |
| Architecture (Picasso's files) | 06: ruled `gui` name; Wasm-implemented packages are ordinary dependencies translated ahead of time (§6.2, §6.4, §6.7, R-4, R-5, P6-1, §6.13); `finance` example has no capabilities. 00: four plugin kinds (R3 row, glossary). 19: four kinds in `[plugins]`; R8-1/2/3 and review-6 R6-1 marked resolved; all P19 agreed; `verse why` openers; links renumbered (16 diagnostics, 18 plugin interface). 20: P20-1 only `Calendar.named`; other P20 agreed, P20-6 0.2; variant-import conflict resolved | `architecture/00-overview.md`, `06-…`, `19-…`, `20-…` |
| Build | `build.sh` run | — |
