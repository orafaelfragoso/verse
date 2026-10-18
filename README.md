# Verse

Verse is an indentation-based, readable, GC-free programming language with inferred types.

This repository holds the language specification (v0.1) and the implementation architecture (draft 1).

**Start here:** [`spec/00-README.md`](spec/00-README.md)

## Layout

- [`spec/`](spec/) — the specification, split into 21 numbered section files (`00-README.md` … `21-review-log.md`), plus `grammar.ebnf.md`, review notes (`reviews/`), change logs (`changes/`) and the build tooling (`build.sh`, `tools/`).
- [`dist/`](dist/) — the built, single-file spec: [`verse-spec.md`](dist/verse-spec.md), [`verse-spec.html`](dist/verse-spec.html) and [`verse-spec.pdf`](dist/verse-spec.pdf). Regenerate with `spec/build.sh`.
- [`spec/architecture/`](spec/architecture/README.md) — how Verse is built: compiler pipeline, backends and targets, runtime, standard-library tree-shaking, plugins, MCP debugging, performance, testing, roadmap, open questions and decision records. Nothing is implemented yet; every number is a target.
