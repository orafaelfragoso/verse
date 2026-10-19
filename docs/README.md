# Verse documentation

| Folder | What it holds |
|--------|---------------|
| [language/](language/README.md) | The language specification (v0.1) in numbered chapters, read in order. Chapter 17 is the grammar, chapter 18 the design notes. |
| [architecture/](architecture/README.md) | How Verse is built: compiler, IR and MIR, bytecode VM, backends, target and ABI, runtime, standard library and tree-shaking, plugins, debugging, performance, testing. |
| [reference/](reference/) | Lookup material: [CLI and `verse.toml`](reference/cli-and-manifest.md), [diagnostics catalog](reference/diagnostics-catalog.md), [MCP tool schemas](reference/mcp-tool-schemas.md), [debugger event vocabulary](reference/events.toml). |
| [decisions/](decisions/README.md) | Architecture decision records, one file per decision. |
| [roadmap.md](roadmap.md) | Milestones with dependencies and exit criteria. |

Nothing is implemented yet. Every number in these documents is a target, not a
measurement.

Build the single-file specification with `tools/docs/build.sh`. It writes
`dist/verse-spec.md`, and also HTML and PDF unless `NO_RENDER=1` is set.
`dist/` is not tracked.
