---
name: editing-docs
description: Covers how to change the Verse documentation. The docs layout (docs/language spec chapters, docs/architecture, docs/reference, docs/decisions ADRs, docs/roadmap.md), the writing rules, stable doc numbers and `§` references, carrying a design change through every doc that mentions it, writing an ADR, adding a diagnostic code, building the single-file spec, and running `tools/docs/check.sh`. Use when editing any file under docs/, README.md, events.toml or the diagnostics catalog, when changing the language or architecture design, when writing an ADR, or when a docs check fails.
---

# Editing docs

The docs are the source of truth for Verse. Nothing is implemented yet, so
every number in them is a target, not a measurement
([docs/README.md](../../../docs/README.md)).

## Read first

1. [docs/README.md](../../../docs/README.md): the folders and the document
   number table.
2. [docs/architecture/README.md](../../../docs/architecture/README.md): what
   each architecture document answers, and its conventions.
3. [docs/decisions/README.md](../../../docs/decisions/README.md): the ADR
   format and index.

## What belongs where

| Folder | Holds | Cited as |
|--------|-------|----------|
| [docs/language/](../../../docs/language/README.md) | the specification: what Verse means. Chapter 17 is the grammar, 18 the design notes, 20 the example programs | `spec §4.9` |
| [docs/architecture/](../../../docs/architecture/README.md) | how Verse is built: compiler, IR and MIR, VM, backends, runtime, standard library, packages, plugins, debugging, performance, testing | `02 §2.8` |
| [docs/reference/](../../../docs/reference/diagnostics-catalog.md) | lookup material: the diagnostics catalog (16), the CLI and `verse.toml` (19), the debugger event vocabulary `events.toml` | `16 §16.1` |
| [docs/decisions/](../../../docs/decisions/README.md) | one ADR per larger choice: the context, the decision, the alternatives and the consequences | `ADR-0005` |
| [docs/roadmap.md](../../../docs/roadmap.md) | milestones M0 to M9 with deliverables, dependencies and exit criteria | `11` |

When the spec and an architecture document disagree, the spec wins
([architecture README](../../../docs/architecture/README.md)). Put a rule
about meaning in the spec. Put a rule about how it is built in architecture.
Put the reason for a larger choice in an ADR.

## Writing rules

- State the design as decided. Write what is true now.
- No change notes. Do not write "was", "now", "previously", "we changed",
  or "new". History lives in `git log`.
- Nothing is built yet, so a removed feature leaves no trace: no reserved
  word for it, no error that names it ("`x` is not part of Verse"), no
  unassigned code kept for it.
- No people's names, no ownership notes, no review files, no review labels.
- A sentence that starts with **Open:** is a design question that is not
  decided. Use it only for that. When the question is decided, rewrite the
  sentence as the decision and drop the marker.
- Short plain sentences, one fact each. No long dashes. No filler.
- Match the existing terms exactly: script mode, compiled mode, tier 0,
  MIR, mono MIR, `--no-heap`, capabilities. The glossary is in
  [00](../../../docs/architecture/overview.md#glossary).
- A **target** is a measurable goal not measured yet. An **assumption** is
  believed true and not verified; each one has a roadmap milestone that
  verifies it.
- Standard library modules are top-level: `http`, never `std.http`. The
  `std-prefix` check enforces this.

## Doc numbers and `§` references

Architecture and reference documents have stable numbers. The title line is
`# NN. Title`, and sections are `## NN.M`. Other documents cite them as
`05 §5.6`, linked like
[02 §2.8](../../../docs/architecture/compiler-architecture.md#28-diagnostics).
A missing number (12, 17, 18) belongs to no current document.

- Spec chapters are cited as `spec §N`, never as a bare `[NN]` link.
- Link text that starts with a number must name the target document's
  number, and a `§` in it must start with that number. The `doc-numbers`
  check enforces both.
- A new architecture or reference document needs a number, a row in the
  tables of [docs/README.md](../../../docs/README.md) and
  [docs/architecture/README.md](../../../docs/architecture/README.md). The
  docs do not say whether a missing number may be reused. Take the next number
  after the highest one so an old citation never points at a new document.
- Spec chapters link each other as `NN-name.md#anchor`. After adding a link
  between chapters, run `python3 tools/docs/crosslink.py`, which rewrites
  in-page `#anchor` links whose anchor lives in another chapter.
- Renaming a heading changes its anchor. Run the checks; the `links` check
  lists every link that broke.

## Changing a design

A design change is done when every document that states the old design
states the new one. Search before you edit, then edit every hit.

1. Collect the terms: the keyword, the type or function names, the flag, the
   diagnostic codes, the section numbers.
2. List every file that mentions them:

   ```sh
   grep -rln -e 'extern' -e 'V1006' docs README.md
   grep -rn 'spec §12.6\|§12.6' docs
   ```

3. Walk this list in order. Each item is a place the docs repeat a design:
   1. the spec chapter that defines the feature, and the
      [tour](../../../docs/language/02-tour.md#2-quick-tour);
   2. [17 the grammar](../../../docs/language/17-grammar.md#17-ebnf-grammar-sketch),
      for any syntax change;
   3. [18 design notes](../../../docs/language/18-design-notes.md#18-design-notes)
      and the [comparison](../../../docs/language/19-comparison.md#19-verse-vs-other-languages),
      when they argue for the old design;
   4. the architecture documents that build it: stage contracts in
      [02 §2.2](../../../docs/architecture/compiler-architecture.md#22-stage-contracts),
      passes in [03 §3.4](../../../docs/architecture/ir-and-passes.md#34-pass-list),
      the runtime, the VM, target and ABI;
   5. the MIR rules: [13 §13.5](../../../docs/architecture/mir-specification.md#135-verifier-rules)
      and the invariants in [03 §3.3](../../../docs/architecture/ir-and-passes.md#invariants-checked-by-the-verifier);
   6. the [diagnostics catalog](../../../docs/reference/diagnostics-catalog.md)
      and [events.toml](../../../docs/reference/events.toml);
   7. the [CLI reference](../../../docs/reference/cli-and-manifest.md), for
      commands, flags, `verse.toml` keys and exit statuses;
   8. code examples: [20](../../../docs/language/20-examples.md#20-complete-example-programs)
      and every ```verse block that uses the feature;
   9. the [roadmap](../../../docs/roadmap.md) deliverables and exit criteria,
      and the [glossary](../../../docs/architecture/overview.md#glossary).
4. If the change reverses or replaces a larger choice, write an ADR (below).
5. Run `tools/docs/check.sh`.
6. Open the pull request with type `spec`. Use `docs` only when the design
   does not change. See the `opening-a-pull-request` skill.

A spec change that changes behavior must change a conformance test in the
same commit once tests exist
([10 §10.2](../../../docs/architecture/testing-and-verification.md#102-specification-conformance)).

## Writing an ADR

Write one when a choice is large enough that someone will ask why: a new
dependency or tool, a representation, a lowering strategy, a tradeoff that
rules out an alternative. A wording fix or a detail inside an existing
decision needs no ADR.

1. Take the next number. List the folder; after 0009 comes 0010.
2. Create `docs/decisions/NNNN-kebab-title.md`:

   ```markdown
   # ADR-0010: Title as a statement of the decision

   Status: proposed.

   ## Context

   ## Decision

   ## Alternatives

   ## Consequences
   ```

   Some ADRs add `## Risks` or `## References`. Status is `proposed` or
   `accepted (YYYY-MM-DD)`. A status line may add one
   sentence of caveat, as [ADR-0009](../../../docs/decisions/0009-cargo-disk-budget.md) does.
3. Add a row to the table in
   [docs/decisions/README.md](../../../docs/decisions/README.md):

   ```markdown
   | [0010](0010-kebab-title.md) | Title | proposed |
   ```

4. Link the ADR from the architecture section it explains.
5. To change a decision, amend the ADR that holds it, and edit the docs that
   state the rule. Do not write a new ADR that replaces an older ruling, in an
   ADR or in the docs. Write a new ADR only for a decision no doc or ADR has
   made yet.

## Adding or changing a diagnostic code

The rules are [16 §16.1](../../../docs/reference/diagnostics-catalog.md#161-rules).
Wording and placement in depth are in
`.claude/skills/understanding-verse/references/diagnostics.md`. The short
form:

- Until 0.1 is released, delete a code together with the rule it reports.
  Do not keep it as unassigned.
- `V` codes go in the family of the pass that finds the problem
  ([16 §16.2.1](../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
  Add the row to the family table in
  [16 §16.3](../../../docs/reference/diagnostics-catalog.md#163-catalog).
- `R` codes are run-time panics
  ([16 §16.6](../../../docs/reference/diagnostics-catalog.md#166-run-time-panics)).
  `D` codes are debugger errors, one `[errors.Dxxxx]` table in
  [events.toml](../../../docs/reference/events.toml).
- The message is one sentence of at most 15 words and uses no word from the
  "don't say" table of
  [08 §8.7](../../../docs/architecture/debugging.md#87-vocabulary-and-wording).
- Every `{name}` placeholder is a fact key (rule 10).
- A code the CLI or a manifest reports also gets a row in
  [19 §19.5](../../../docs/reference/cli-and-manifest.md#195-where-each-code-appears).
- Cite the code wherever the spec or architecture names the error.

The `diagnostic-codes` and `diagnostic-families` checks catch an undefined
code, a code defined twice, a pass with no row and a code no pass reports.

## Building the spec

```sh
NO_RENDER=1 tools/docs/build.sh   # dist/verse-spec.md only
tools/docs/build.sh               # also HTML, and PDF if google-chrome exists
```

`build.sh` concatenates the region of
[docs/language/README.md](../../../docs/language/README.md) between the
`spec:begin` and `spec:end` markers and the chapters in its `PARTS` list. A
new chapter needs an entry in `PARTS` and in the README contents. `dist/` is
not tracked.

## Checks

Run before every docs pull request:

```sh
tools/docs/check.sh
```

It runs each check, prints one summary line per check, builds the spec, and
ends with `docs check: N failed`. It exits 1 if any check fails.

| Check | Fails on |
|-------|----------|
| `links` | a relative link to a missing file or heading anchor, in docs/, README.md and `.claude/skills/*/SKILL.md` |
| `doc-numbers` | a bad `# NN. Title`, a duplicate number, a link or `NN §M` that names the wrong document, an index table missing a document |
| `diagnostic-codes` | a `V`, `R` or `D` code referenced but not defined, or defined twice |
| `diagnostic-families` | a pass with no row in 16.2.1, a row naming no pass or crate, a code no pass reports |
| `std-prefix` | any `std.` reference |
| `build` | `build.sh` fails |

Each check prints the file and line of every problem above its summary line.
The scripts are in `tools/docs/check/`.
