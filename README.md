# Verse

Verse is an indentation-based, readable, GC-free programming language with inferred types.

This repository holds the language specification (v0.1) and the implementation architecture (draft).

**Start here:** [`docs/README.md`](docs/README.md)

## Layout

- [`docs/language/`](docs/language/README.md): the specification in numbered chapters, with the grammar and design notes.
- [`docs/architecture/`](docs/architecture/README.md): how Verse is built. Compiler, MIR, VM, backends, target and ABI, runtime, performance, testing. Nothing is implemented yet; every number is a target.
- [`docs/reference/`](docs/reference/): CLI and `verse.toml`, diagnostics catalog, debugger events.
- [`docs/decisions/`](docs/decisions/README.md): architecture decision records.
- [`docs/roadmap.md`](docs/roadmap.md): milestones.
- [`tools/docs/`](tools/docs/): `build.sh` concatenates the chapters into `dist/verse-spec.md` and renders HTML and PDF. `dist/` is built on demand and not tracked.
