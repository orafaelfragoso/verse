# Contributing

Maintainers commit directly to `main`, one change per commit. Other
contributors open a pull request, which lands as one squashed commit: its
title becomes the commit subject and its body the commit body. Either way,
write the message for someone reading `git log` a year from now.

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

## Committing to `main` (maintainers)

1. Pull first: `git pull --ff-only`. Never force push.
2. One change per commit, in the commit format above. The body has the same
   three parts as a pull request: Why, What changed, Verification.
3. Run `tools/docs/check.sh` before pushing when docs changed.

## Pull requests (other contributors)

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
