# Verse documentation

| Folder | What it holds |
|--------|---------------|
| [language/](language/README.md) | The language specification (v0.1) in numbered chapters, read in order. Chapter 17 is the grammar, chapter 18 the design notes. |
| [architecture/](architecture/README.md) | How Verse is built: compiler, IR and MIR, bytecode VM, backends, target and ABI, runtime, standard library and tree-shaking, packages, native plugins, debugging, performance, testing. |
| [reference/](reference/) | Lookup material: [CLI and `verse.toml`](reference/cli-and-manifest.md), [diagnostics catalog](reference/diagnostics-catalog.md), [debugger event vocabulary](reference/events.toml). |
| [decisions/](decisions/README.md) | Architecture decision records, one file per decision. |
| [roadmap.md](roadmap.md) | Milestones with dependencies and exit criteria. |

Architecture and reference documents have stable numbers. Other documents
cite them by number, as `05 §5.6`. A missing number belongs to no current
document.

| No. | Document | No. | Document |
|-----|----------|-----|----------|
| 00 | [Overview](architecture/overview.md) | 11 | [Roadmap](roadmap.md) |
| 01 | [Requirements](architecture/requirements.md) | 13 | [MIR specification](architecture/mir-specification.md) |
| 02 | [Compiler architecture](architecture/compiler-architecture.md) | 14 | [Bytecode and VM](architecture/bytecode-and-vm.md) |
| 03 | [IR and pass specification](architecture/ir-and-passes.md) | 15 | [Target and ABI](architecture/target-and-abi.md) |
| 04 | [Execution tiers and backends](architecture/backends-and-targets.md) | 16 | [Diagnostics catalog](reference/diagnostics-catalog.md) |
| 05 | [Runtime](architecture/runtime.md) | 19 | [CLI, manifests and package manager](reference/cli-and-manifest.md) |
| 06 | [Standard library and tree-shaking](architecture/stdlib-and-shaking.md) | 20 | [First-party packages](architecture/first-party-packages.md) |
| 07 | [Plugins](architecture/plugins.md) | 21 | [Packages](architecture/packages.md) |
| 08 | [Debugging](architecture/debugging.md) | | |
| 09 | [Performance](architecture/performance.md) | | |
| 10 | [Testing and verification](architecture/testing-and-verification.md) | | |

Nothing is implemented yet. Every number in these documents is a target, not a
measurement.

Build the single-file specification with `tools/docs/build.sh`. It writes
`dist/verse-spec.md`, and also HTML and PDF unless `NO_RENDER=1` is set.
`dist/` is not tracked.
