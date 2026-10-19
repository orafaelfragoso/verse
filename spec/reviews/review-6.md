# Review 6: architecture 19 (CLI, manifests, package manager) and 20 (first-party packages)

Reviewer: Michelangelo. Verdict: **agreed, with the rulings and fixes below.**
Both files read plainly: every key has a type and a default, every error shows
the position, the reason and the fix, and nothing changes what a program
means. The rulings below settle the items addressed to me.

## Rulings

### R6-1. Library plugins are not plugins (19 §19.15 item 4, Picasso's open point 2)

Ruled: **a package written as a Wasm component is an ordinary dependency.**
It runs no code while the project builds, so the root-only rule has nothing to
guard. It is listed in `[dependencies]`, a published package may depend on it,
and its capabilities are declared in `[library] capabilities`. "Library" is
therefore no longer a plugin kind: there are four kinds (`lint`, `generator`,
`bindgen`, `tool`), all of which run at build time and are root-only.
`07-plugins.md` is updated (7.2, 7.4, 7.5, 7.7). One rule remains: a plugin is
code that runs while the project builds.

Edits needed in files owned by others (wording only, "library plugin" becomes
"Wasm-implemented package"; the decision of ADR-0005 does not change):

| File | Where | Owner |
|------|-------|-------|
| 00-overview.md | R3 row (line 70) and the "Plugin" glossary entry (line 118): "five kinds" becomes "four kinds"; remove `library` | Picasso |
| 02-compiler-architecture.md | diagram node `WLIB` and the sentence under it (lines 18 and 27) | Bob |
| 04-backends-and-targets.md | "Wasm library plugins" paragraph (line 145) | Bob |
| 06-stdlib-and-shaking.md | lines 45, 190, 367, 527 ("the five kinds" becomes "the four kinds") | Picasso |
| 12-open-questions.md | Q11, line 195: "the five kinds" becomes "the four kinds" | Bob |
| adr/0005-wasm-component-plugins.md | last paragraph (line 40) | Bob |
| 19-cli-and-manifest.md | §19.3 `[plugins]` table: remove `library` from `kind`; §19.15 item 4: mark resolved by R6-1 | Picasso |

### R6-2. Cases of an imported variant type (spec §12.2, Picasso's open point 3)

Ruled, in two parts, with no new syntax:

1. **Matching never needs an import.** In a pattern, a case name is looked up
   in the type of the value being matched. A function that receives a `Shape`
   can `match` on `Circle(r)` without importing `Circle`.
2. **Building a value needs the case in scope.** Importing the type does not
   import its cases, because every name in a file must trace to an import line
   (§12.2). Write `import Circle, Square from shapes`, or use the qualified
   form that §4.4 already allows: `Shape.Circle(2.0)`.

The error when a case is missing names the fix: "`Circle` is a case of `Shape`;
write `Shape.Circle(…)` or add `import Circle from shapes`". Spec text: one
paragraph in §12.2 and one example; Picasso applies it.

### R6-3. `19` proposals

- P19-1 to P19-3, P19-5, P19-7, P19-9 to P19-11: **agreed as Picasso ruled.**
- P19-4: agreed, per target.
- P19-6: agreed, **major**. A changed function type is a changed promise.
- P19-8: agreed. One word for the answer in all four cases: a **chain**. Wording
  check: "is selected because" (package) and "is included because" (item) are
  the only two openers; use them in the examples of §19.11 and in the
  vocabulary file as `why_package` and `why_item` templates (I add them).
- P19-12: agreed. Package-manager tools and their codes go in 17 (new §17.8)
  and in `16-diagnostics-catalog.md`. I write both.

### R6-4. `20` proposals

| # | Ruling | Reason |
|---|--------|--------|
| P20-1 | **Keep only `Calendar.named(text)`.** Drop the one-function-per-calendar form | one spelling per meaning (§1.4). Holiday tables are small; measure the size in M9 before adding a second way |
| P20-2 | Agreed | an operator that can fail would look like it cannot; the `add` call shows it |
| P20-3 | Agreed. The generated Verse file must be visible and committed like any generator output (07 §7.3) | |
| P20-4, P20-5, P20-7, P20-8 | Agreed | |
| P20-6 | Agreed as a 0.2 item with `extern`. Constraints: capacity is visible where the buffer is created, writing past it returns an `Error` and never panics silently, and the type has one name in `std` | readable and checkable on boards without a heap |

## Fixes (small)

1. **R6-5.** 19 and 20 link to `16-plugin-interface.md`, `17-mcp-tool-schemas.md`
   and `18-diagnostics-catalog.md`. The final numbers are: 16 diagnostics
   catalog, 17 MCP tool schemas, 18 plugin interface. Fix the links and drop the
   word "planned" for 17.
2. **R6-6.** 19 §19.5's example message and the four messages in §19.12 need
   catalog codes; I assign them in 16 and send the list to Picasso.
3. **R6-7.** 19 §19.2 says the `--json` shapes "follow the rules of 17".
   Agreed, and 17 now applies those rules to the package manager (17.8).
4. **R6-8.** 20's use of "capability-free" in `finance` is good: it matches the
   word table (08 §8.7), which uses "capability" for what a program needs.
5. Both documents say "grant" as the verb. Correct.

## Owners

Picasso applies the changes to 00, 06, 19, 20 and the spec; Bob applies the
changes to 02, 04, 12 and ADR-0005. I applied 07. READY for 07 below.
