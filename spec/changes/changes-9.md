# Changes 9 — Round 9 (conversions and exit statuses)

Line numbers are in the files after the change. `00-README.md` was not edited:
the section status table is unchanged and nothing is reopened. Section
numbering is unchanged. Only `architecture/19-cli-and-manifest.md` was edited
among the architecture files; 04, 05, 12, 13, 14, 15 and 16 were not touched.

| Item | Resolution | File: lines |
|------|------------|-------------|
| R9-1 failed conversions | §3.8.4 bullet: a conversion that does not fit panics with "`{a}` cannot be converted to `{T}`" and a reason | `03-lexical.md`: 477–481 |
| R9-2 exit statuses | §12.5: the exit-status list is a table aligned with architecture 16 §16.6.1 (0, 1, 2, 3, 70, 101) | `12-modules.md`: 224–244 |
| R9-3 `exit(code)` | §12.5 paragraph (0 to 255, any task, flushes output, runs no `Cleanup`); §15.1 prelude table row | `12-modules.md`: 235–244; `15-stdlib.md`: 19 |
| Log | "Round 9" table | `21-review-log.md`: 318–329 |
| Architecture (Picasso's file) | 19: new §19.2.1 exit statuses (links 16 §16.6.1); new "Target files" subsection (15 §15.9.1, §15.9.2, §15.9.5); `wasm32-wasi` → `wasm32-wasip1`; cache-key row names the resolved target file | `architecture/19-cli-and-manifest.md`: 71–92, 243, 250, 267–291, 689 |
| Build | `build.sh` run (Markdown only) | — |
