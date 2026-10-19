---
name: filing-an-issue
description: Covers how to file a Verse issue. Choosing the Bug, Design proposal or Task form, writing observable acceptance criteria, labels (one type label, `area:` labels, `status:needs-design`, `status:ready`, `status:blocked`, `good first issue`), milestones M0 to M9 from docs/roadmap.md, `gh issue create` with labels and milestone, and splitting a roadmap milestone into issues. Use when opening, triaging or labeling an issue, running `gh issue create` or `gh issue edit`, planning a milestone, or deciding whether an issue is ready to start.
---

# Filing an issue

The forms are in
[.github/ISSUE_TEMPLATE/](../../../.github/ISSUE_TEMPLATE/config.yml). Blank
issues are off. The labeling rules are in
[CONTRIBUTING.md](../../../CONTRIBUTING.md#issues). The milestones are the
stages of the [roadmap](../../../docs/roadmap.md).

## Pick the form

| Form | Use when | Labels it adds on the web |
|------|----------|---------------------------|
| [Bug](../../../.github/ISSUE_TEMPLATE/1-bug.yml) | Verse behaves differently from the spec or the docs | `bug` |
| [Design proposal](../../../.github/ISSUE_TEMPLATE/2-design.yml) | a change to the language, the standard library or the architecture | `design`, `status:needs-design` |
| [Task](../../../.github/ISSUE_TEMPLATE/3-task.yml) | scoped work with criteria anyone can check | none |

- If the docs do not say what should happen, it is not a bug. File a design
  proposal.
- If the work changes what the docs say, it is a design proposal, even when
  it looks small. Once the design is decided and the docs are updated, the
  implementation is a task.
- If the docs already specify the behavior and it is not built yet, it is a
  task with the `feature` label.

## Write the body

`gh issue create` does not render the forms. Pass a body whose `###`
headings match the form's fields, in the same order, so CLI issues read like
web issues.

**Bug**: Program (the smallest `.vs` file that shows it), Command, Expected
(link the spec or docs section), Actual (full output and exit status),
Version (`verse --version` or a commit SHA), Target.

**Design proposal**: Problem (show it with Verse code), Proposal (with Verse
code that uses it), Alternatives (and why this one wins), Affected docs
(files and sections, for example `docs/language/12-modules.md §12.6`). The
`editing-docs` skill lists every place a design is repeated.

**Task**: Goal, Acceptance criteria, Context, Out of scope.

- **Goal**: one or two sentences. What exists or works when this is done.
- **Acceptance criteria**: each item is observable. A command and its
  expected output, or a file and what it contains. Not "works well", "is
  clean" or "is tested".
- **Context**: the docs sections, files and issues to read first, linked.
- **Out of scope**: related work that must not be done here.

Acceptance criteria, weak and strong:

```markdown
- [ ] The lexer handles indentation.

- [ ] `verse check bad-indent.vs` on a file indented with a tab exits 2 and
      prints `explain: V0101`.
- [ ] Every ```verse block in docs/language/ parses with no V02xx error.
- [ ] `tests/golden/` has a token dump for each example in spec §3.3.
```

The first item cannot be checked. The other three can be checked by anyone,
human or agent, without asking a question.

## Labels

Every issue gets:

1. **One type label**: `bug`, `feature` (implements something the docs
   already specify), `design`, `docs`, `perf`, `chore`.
2. **One or more `area:` labels**: `area:spec`, `area:compiler`, `area:vm`,
   `area:runtime`, `area:std`, `area:cli`, `area:tooling`, `area:packages`,
   `area:plugins`, `area:targets`. They match the commit scopes in
   CONTRIBUTING.md.
3. **A status label when one applies**:

| Label | Means | Set when |
|-------|-------|----------|
| `status:needs-design` | a decision is needed before work starts | a design proposal opens, or a task turns out to need a decision |
| `status:ready` | fully specified: anyone, human or agent, can start | the checklist below holds |
| `status:blocked` | waiting on another issue or an outside change | name the blocker in the body: `Blocked by #12` |

Add `good first issue` when the work is small and self-contained: one crate
or one doc, no design decision, and context that fits in a few links.

Run `gh label list` to see the current set. Do not create labels. A label is
a repository setting; propose a new one in an issue.

### When an issue is `status:ready`

All of these hold:

- the form is complete, with no placeholder text;
- every acceptance criterion is observable;
- the design it depends on is in the docs, with no **Open:** sentence on the
  point it needs;
- the issues it depends on are closed;
- it has a type label, an `area:` label and a milestone.

Remove `status:needs-design` when you add `status:ready`.

## Milestones

Each issue gets the milestone of the roadmap stage it belongs to. The
milestone titles are `M<n>: <Name>`.

| Milestone | Roadmap stage |
|-----------|---------------|
| `M0: Foundations` | repo layout, CI, lexer, CST, parser, `verse fmt` skeleton, diagnostics framework, harnesses |
| `M1: Script core` | resolution, HIR, inference, ownership, MIR, mono, bytecode VM, minimal debug adapter |
| `M2: Native` | Cranelift for x86-64 and AArch64, linking, reachability, `--size-report`, the native cache |
| `M3: WebAssembly` | Wasm emitter, `std-wasi` and `std-web`, state-machine task lowering prototype |
| `M4: Tasks` | scheduler, channels, `Shared`, `Guarded`, cancellation |
| `M5: Standard library core` | the core modules, package manager, `verse.lock`, `verse.api`, Windows |
| `M6: Tooling` | language server, `verse doc`, `verse test --watch`, diagnostics JSON, record and replay |
| `M7: Release and embedded` | LLVM, release profile, Cortex-M, ESP32, `--no-heap`, RISC-V |
| `M8: Plugins and generators` | `extern`, `[plugins]`, `verse-ffi`, `verse bind`, `verse generate` |
| `M9: Domain libraries` | finance, ML, web and GUI, IoT packages |

The deliverables and exit criteria are in the
[roadmap table](../../../docs/roadmap.md#11-roadmap). The milestone
descriptions on GitHub copy them. A design proposal that no milestone needs
yet gets no milestone.

## Create the issue

```sh
gh issue create \
  --title "Lex indentation and report tabs as V0101" \
  --body-file /path/to/body.md \
  --label feature --label area:compiler --label status:ready \
  --milestone "M0: Foundations"
```

- The issue title is plain text, not the commit format. The pull request that
  closes it uses the commit format (see the `opening-a-pull-request` skill).
- A design proposal also needs `--label design --label status:needs-design`.
- Fix labels or the milestone later with
  `gh issue edit 12 --add-label status:ready --remove-label status:needs-design`
  and `gh issue edit 12 --milestone "M1: Script core"`.

## Splitting a milestone into issues

1. Read the milestone's row in the [roadmap](../../../docs/roadmap.md#11-roadmap):
   the deliverables and the exit criteria.
2. Make one task per deliverable that can be verified on its own. Split a
   deliverable further when it spans more than one crate of
   [02 §2.9](../../../docs/architecture/compiler-architecture.md#29-crate-layout)
   or needs more than one pull request.
3. Turn each exit criterion into acceptance criteria on the task that
   delivers it. An exit criterion that needs several tasks gets its own task
   that depends on them.
4. Order the tasks by the dependency direction of 02 §2.9 and the
   [sequencing principles](../../../docs/roadmap.md#principles-for-sequencing).
   Write `Blocked by #N` in the Context section and add `status:blocked`
   until #N closes.
5. File a design proposal, not a task, for any deliverable whose docs still
   have an **Open:** sentence or no section at all.
6. Link the roadmap section and the architecture sections in each task's
   Context.

Example for M0, deliverable "lexer":

```markdown
### Goal

`verse-syntax` turns a `.vs` file into a token stream with spans.

### Acceptance criteria

- [ ] Every ```verse block in docs/language/ lexes with no V01xx error.
- [ ] Each V01xx code of 16 §16.3 has a golden test of its rendered text.
- [ ] `cargo test -p verse-syntax` passes.

### Context

docs/roadmap.md (M0), docs/language/03-lexical.md, 02 §2.2, 16 §16.3 V01xx.

### Out of scope

The parser and the CST.
```
