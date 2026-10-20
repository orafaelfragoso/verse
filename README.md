# Verse

Verse is an indentation-based, readable, GC-free programming language with inferred types.

This repository holds the language specification (v0.1), the implementation architecture, and the M0 syntax front end and tooling.

**Start here:** [`docs/README.md`](docs/README.md)

## Layout

- [`docs/language/`](docs/language/README.md): the specification in numbered chapters, with the grammar and design notes.
- [`docs/architecture/`](docs/architecture/README.md): compiler, MIR, VM, backends, target and ABI, runtime, performance, testing. Later pipeline stages are planned; performance budgets are targets.
- [`docs/reference/`](docs/reference/): CLI and `verse.toml`, diagnostics catalog, debugger events.
- [`docs/decisions/`](docs/decisions/README.md): architecture decision records.
- [`docs/roadmap.md`](docs/roadmap.md): milestones.
- [`tools/docs/`](tools/docs/): `build.sh` concatenates the chapters into `dist/verse-spec.md` and renders HTML and PDF. `dist/` is built on demand and not tracked.

## Build and verify

`tools/check.sh` runs the fast M0 verification gates:

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 -m unittest discover -s tools/tests -v
python3 tools/check-dependencies.py
python3 tools/spec-coverage.py
tools/docs/check.sh
```

`cargo run -p verse-cli -- fmt path/to/file.vs` formats source.
`fmt --check` reports differences without writing. `explain V0209` prints a
diagnostic explanation. Running programs and semantic `check` are not available
in M0. The benchmark harness is `python3 bench/compiler/run.py`.
