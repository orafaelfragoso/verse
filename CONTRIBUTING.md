# Contributing

Every change reaches `main` through a pull request, and every pull request
lands as one squashed commit. The pull request title becomes the commit
subject and the pull request body becomes the commit body, so write both for
someone reading `git log` a year from now.

## Commit format

```
<type>(<scope>): <summary>
```

- The scope is optional. Add `!` before the colon for a breaking change:
  `spec(std)!: rename http.serve to http.listen`.
- The summary is imperative ("add", not "added"), has no final period, and
  the whole line is at most 72 characters.
- The `pr` workflow checks every pull request title against this format.

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
`cli`, `tooling`, `packages`, `plugins`, `targets`.

Examples:

```
feat(compiler): parse extern blocks
fix(vm): run cleanup blocks when a task is cancelled
spec(std): give http.serve a handler that can fail
ci: require the pr-title check on main
```

## Pull requests

1. Branch from `main`. Name the branch after the change: `extern-blocks`.
2. Keep one change per pull request. A reviewer should be able to state it in
   one sentence.
3. Fill in the template. **Why** says what problem this solves and links the
   issue (`Closes #12`). **What changed** lists the changes a reviewer
   cannot see at a glance. **Verification** lists the commands you ran and
   what they printed.
4. Squash merge is the only merge method. History on `main` stays linear, one
   commit per pull request, so `git log`, `git bisect` and `git revert` all
   work on whole changes.

## Issues

Pick a template:

- **Bug**: Verse behaves differently from the spec or the docs.
- **Design proposal**: a change to the language, the standard library or the
  architecture. It starts as `status:needs-design`.
- **Task**: scoped work with acceptance criteria anyone can check. Mark it
  `status:ready` when someone, human or agent, can start without asking a
  question.

Each issue gets one type label (`bug`, `feature`, `design`, `docs`, `perf`,
`chore`), one or more `area:` labels, and the milestone of the
[roadmap](docs/roadmap.md) stage it belongs to.

## Repository settings

`tools/github/setup.sh <owner/repo>` applies the merge settings, labels,
milestones and branch rules. Rerunning it converges to the same state. Change
labels or rules in that script, and milestones in the roadmap, then run it.
