---
name: opening-a-pull-request
description: Covers how a change reaches `main` in the Verse repository. Branch naming, one change per pull request, the `<type>(<scope>): <summary>` title format checked by the `pr-title` job, the Why / What changed / Verification body that becomes the squash commit body, linking issues, labels and milestones, `gh pr create`, and squash merge. Use when opening, titling, describing or merging a pull request, writing a commit message, running `gh pr create` or `gh pr merge`, or when a push to `main` is rejected.
---

# Opening a pull request

Every change reaches `main` through a pull request, and every pull request
lands as one squashed commit. The title becomes the commit subject. The body
becomes the commit body. Write both for someone reading `git log` a year from
now. The source of these rules is
[CONTRIBUTING.md](../../../CONTRIBUTING.md); the title check is
[.github/workflows/pr.yml](../../../.github/workflows/pr.yml).

## Rules

- Never push to `main`. The `default-branch` ruleset rejects direct pushes,
  force pushes and deletion of `main`, requires linear history, and allows
  only squash merges through a pull request.
- One change per pull request. A reviewer must be able to state it in one
  sentence. If the sentence needs "and", split the pull request.
- The `pr-title` check is required. A pull request whose title fails it
  cannot merge.
- Every review thread must be resolved before merge.
- Squash merge is the only method. The repository writes the pull request
  title as the commit subject and the pull request body as the commit
  message, and deletes the branch after merge.
- Commits on the branch disappear in the squash. Only the title and the body
  reach `main`, so put the explanation there.

## Steps

1. Update `main` and branch from it. Name the branch after the change, in
   kebab-case, with no prefix:

   ```sh
   git switch main && git pull --ff-only
   git switch -c extern-blocks
   ```

2. Make the change. Keep it to one change (see Rules).
3. Run the checks the change needs:
   - Docs changed: `tools/docs/check.sh`. It must end with
     `docs check: 0 failed`. See the `editing-docs` skill.
   - Code or tests changed: the test layers the `writing-tests` skill names
     for that change.
4. Write the title (see Title format).
5. Write the body (see Body).
6. Push and open the pull request:

   ```sh
   git push -u origin extern-blocks
   gh pr create --base main \
     --title "feat(compiler): parse extern blocks" \
     --body-file /path/to/body.md \
     --label feature --label area:compiler \
     --milestone "M8: Plugins and generators"
   ```

   Use `--body-file`, not `--fill`. `--fill` copies branch commit messages,
   which are not the body you want on `main`.
7. Wait for the `pr-title` check:

   ```sh
   gh pr checks --watch
   ```

   If it fails, fix the title with `gh pr edit --title "..."`. The check
   reruns on `edited`.
8. Merge with squash once the check passes and every review thread is
   resolved:

   ```sh
   gh pr merge --squash --auto
   ```

   Do not pass `--subject` or `--body`. The repository already uses the
   pull request title and body, so the commit matches what was reviewed.

## Title format

```
<type>(<scope>): <summary>
```

- The scope is optional. Add `!` before the colon for a breaking change:
  `spec(std)!: rename http.serve to http.listen`.
- The summary is imperative ("add", not "added"), starts with no space, and
  has no final period.
- The whole line is at most 72 characters.

| Type | Use for |
|------|---------|
| `feat` | new behavior in the toolchain or the standard library |
| `fix` | behavior that did not match the spec or the docs |
| `perf` | speed, memory or binary size, with no change in behavior |
| `refactor` | structure only, with no change in behavior |
| `test` | tests only |
| `docs` | documentation that does not change the design |
| `spec` | a change to the language, the standard library API or the architecture |
| `build` | Cargo, toolchain, build scripts |
| `ci` | GitHub workflows and repository settings |
| `chore` | anything else that changes no behavior |

Scopes match the `area:` labels: `spec`, `compiler`, `vm`, `runtime`, `std`,
`cli`, `tooling`, `packages`, `plugins`, `targets`. The check rejects any
other scope.

Pick `docs` or `spec` by what the change does, not by which folder it
touches. Rewording a section is `docs`. Changing what Verse means or how it is
built is `spec`, even when only Markdown changes.

Good titles:

```
feat(compiler): parse extern blocks
fix(vm): run cleanup blocks when a task is cancelled
spec(std): give http.serve a handler that can fail
ci: require the pr-title check on main
build: add the docs check scripts
```

Titles the check rejects:

```
Docs: move things                      type is capitalized
feat(parser): parse extern blocks      parser is not a scope
fix(vm): run cleanup on cancel.        final period
feat(vm) : add tracing                 space before the colon
```

To test a title locally, run the check's own pattern:

```sh
TITLE='feat(compiler): parse extern blocks'
types='feat|fix|perf|refactor|test|docs|spec|build|ci|chore'
scopes='spec|compiler|vm|runtime|std|cli|tooling|packages|plugins|targets'
[[ $TITLE =~ ^($types)(\(($scopes)\))?!?:\ [^\ ].*[^.\ ]$ ]] \
  && (( ${#TITLE} <= 72 )) && echo ok || echo rejected
```

## Body

The template has three headings:
[pull_request_template.md](../../../.github/pull_request_template.md). Keep
all three. The body becomes the commit body on `main`, so it must read as
finished prose.

- **Why**: the problem this solves, in one to three sentences. Link the issue
  with `Closes #12` so merging closes it. Use `Refs #12` when the pull request
  does only part of the issue.
- **What changed**: the changes a reviewer cannot see at a glance. One bullet
  each. Do not restate the diff file by file.
- **Verification**: the commands you ran and what they printed. Paste the
  summary line, not the whole log.

Do not write:

- placeholders such as `TODO`, `TBD`, `<describe>` or an empty heading;
- task lists (`- [ ]`). They mean nothing in `git log`;
- "this PR", "I", or notes about review rounds;
- claims you did not run. If a check could not run, say which and why.

Example:

```markdown
## Why

Agents and contributors had no single command to check the docs before a
pull request. Closes #14.

## What changed

- `tools/docs/check.sh` runs the link, diagnostic code, diagnostic family,
  document number and `std.` prefix checks, then builds the spec.
- The link check also reads `.claude/skills/*/SKILL.md`.

## Verification

- `tools/docs/check.sh` printed `docs check: 0 failed`.
- A broken anchor in a skill and a `std.http` reference each made it exit 1.
```

## Labels and milestone

Labels and milestones belong mainly on issues (see the `filing-an-issue`
skill). Add them to a pull request when it has no issue, or when they help a
reviewer filter:

- one type label: `bug`, `feature`, `design`, `docs`, `perf`, `chore`;
- the `area:` label that matches the title scope;
- the milestone of the roadmap stage the work belongs to, for example
  `M0: Foundations`. List them with
  `gh api repos/{owner}/{repo}/milestones --jq '.[].title'`.

## After merge

A squash commit is not an ancestor of the branch commits, so `git branch -d`
refuses. Confirm the pull request merged, then delete the local branch:

```sh
gh pr view extern-blocks --json state --jq .state   # prints MERGED
git switch main && git pull --ff-only
git branch -D extern-blocks
```

The remote branch is deleted on merge.

## When something fails

| Symptom | Cause | Fix |
|---------|-------|-----|
| a push to `main` fails with "Repository rule violations found" | direct push to `main` | push a branch and open a pull request |
| `pr-title` fails | title does not match the format or is over 72 characters | `gh pr edit --title "..."` |
| merge blocked with the check green | an unresolved review thread | resolve or answer the thread |
| `gh pr merge` refuses `--merge` or `--rebase` | squash is the only method | use `--squash` |
