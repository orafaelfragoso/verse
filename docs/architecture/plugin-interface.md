# 18. Plugin interface

What a plugin sees and can call: its manifest, the host interface, the
inputs and outputs of each kind, the capabilities, and the limits. This file
is the detail behind [07](plugins.md), which owns the rules. Where the two
differ, 07 wins and this file is fixed. Status: **draft 1**. The host mechanics (`verse-plugin`, the Wasm runtime,
[02 §2.9](compiler-architecture.md)) are in 02; this file fixes only the
shape a plugin author sees.

A plugin is a Wasm component that runs while the project builds. A package
written as a Wasm component that runs inside the program is not a plugin
(07 §7.4) and does not use this interface.

## 18.1 Rules

| # | Rule | Why |
|---|------|-----|
| 1 | The interface is data in, data out. A plugin receives records and returns records. It never receives a handle to the compiler | A plugin cannot change what the compiler does (07 §7.1) |
| 2 | Every record that describes a program is the record of a public tool: `Diagnostic` ([02 §2.8](compiler-architecture.md)) and the shared types of [17 §17.2](../reference/mcp-tool-schemas.md) | One vocabulary for the compiler, the debugger, the MCP server and plugins |
| 3 | A plugin sees only what its kind and its grant allow | Least access by construction |
| 4 | The interface has a version. A plugin names the version it was built for, and the host refuses a plugin built for a newer one | Old plugins keep working, new hosts stay free to add |
| 5 | The interface only grows within a major version. A removed or changed field is a new major version | Same rule as `verse.api` (spec §16.4) |
| 6 | A plugin is deterministic: same inputs and same plugin bytes give the same output | Builds repeat, and `verse why` can name the writer of a file (spec §16.3) |

## 18.2 The plugin manifest

Every plugin carries a `plugin.toml` inside its package. It states what the
plugin is and what it asks for. The root `verse.toml` states what it is
given ([19 §19.3](../reference/cli-and-manifest.md), `[plugins]`). The tool shows the
two side by side and stops if the request is larger than the grant (07 §7.5).

```toml
[plugin]
name        = "jsonschema"
version     = "1.2.0"
kind        = "generator"          # lint | generator | bindgen | tool
interface   = "1"                  # the plugin interface version, §18.9
description = "Records from a JSON schema."

[asks]
reads        = ["schema/"]         # paths it wants to read
writes       = ["generated/"]      # paths it wants to write
capabilities = []                  # other capabilities, §18.6

[limits]                           # optional; may only lower the host defaults
fuel   = 50_000_000
memory = "64 MiB"
```

| Rule | Detail |
|------|--------|
| `kind` must equal the kind in `verse.toml` | A mismatch is `V0814` |
| `writes` must lie under `generated/` for a `generator` and a `bindgen`, and is empty for a `lint` | The only place a plugin can put source is the folder every reader knows |
| A `lint` asks for no `reads` and no `writes` | It gets the typed program, which is not a path |
| Unknown keys are errors | Same as `verse.toml` (§19.3) |
| `[limits]` can lower the host budget, never raise it | The project owner sets the ceiling |

## 18.3 The host interface

The host offers a plugin the following calls. Each is a function of a Wasm
interface, written here in a plain table. The exact interface file is in
`verse-plugin` and is generated from this table (18.10).

| Call | Available to | Meaning |
|------|--------------|---------|
| `inputs()` | all | The inputs the plugin was granted, as a list of `Input` records (18.4) |
| `program()` | `lint`, `tool` when granted | The typed program as read-only records (18.5) |
| `report(diagnostic)` | `lint`, `generator`, `bindgen`, `tool` | Add one plugin diagnostic (18.7) |
| `emit(file)` | `generator`, `bindgen` | Add one generated source file (18.8) |
| `log(text)` | all | A message for the user at `--verbose`. It is not a diagnostic and never appears in `check --json` |
| `budget()` | all | What is left of fuel, time and memory, so a plugin can stop early and say so |

A plugin has no call for the file system, the network, the clock, randomness
or the environment. A plugin that needs one asks for a capability (18.6) and
only then gets the call that belongs to it.

A plugin exports one entry point per kind:

| Kind | Entry point | Returns |
|------|-------------|---------|
| `lint` | `check()` | nothing; results go through `report` |
| `generator` | `generate()` | nothing; results go through `emit` and `report` |
| `bindgen` | `generate()` | nothing; results go through `emit` and `report` |
| `tool` | `run(arguments)` | an exit status and optional output text |
| `tool` with `renders` | `render(value)` | one string, see 18.12 |

## 18.4 Inputs

An `Input` is one granted file or directory, read through the plugin's own
copy of its bytes. The host never gives a plugin a path it did not grant.

| Field | Meaning |
|-------|---------|
| `path` | Project-relative path, forward slashes |
| `kind` | `file` or `directory` |
| `bytes` | The content, for a file. For a directory: the list of `Input` records inside it |
| `hash` | The content hash used by the cache and by `verse.lock` |

Inputs are part of the plugin's cache key, along with the plugin's own hash
and its grant, so an unchanged plugin on unchanged inputs does not run again
([02 §2.7](compiler-architecture.md)).

## 18.5 The program a `lint` sees

A `lint` receives the typed program as read-only records, in the order of the
source. It does not receive the compiler's internal tree, which can change
between versions; it receives a stable view whose shape is part of the
interface version.

| Record | Fields |
|--------|--------|
| `Item` | `name`, `kind` (`function`, `type`, `trait`, `impl`, `constant`), `exported: bool`, `position`, `signature` (the text shown in `verse.api`), `type` (the inferred type, as text) |
| `Call` | `position`, `callee`, `argument_types`, `result_type` |
| `Binding` | `position`, `name`, `mutable: bool`, `type`, `uses: [position]` |
| `Match` | `position`, `subject_type`, `arms: [{pattern, position}]` |
| `Span` | `file`, `from`, `to` (the `position` shape of 17 §17.2) |
| `Source` | `file`, `text`, so that a suggestion can be written as an edit |

- It includes inferred types, because a style rule needs them ("this is a
  `List[Int]`, so `.map` fits").
- It has no ownership state (borrows, moves): those belong to the compiler
  and the debugger. A lint that needs them is an open point (18.11).
- It is the same view in every tier and on every target.

A `tool` plugin that was granted `program` receives the same view. A
`generator` or a `bindgen` never does.

## 18.6 Capabilities

A capability is a named permission. A plugin asks for it in `[asks]`, the
project grants it in `verse.toml`, and the grant is pinned in `verse.lock`.

| Capability | What it adds | Allowed for |
|------------|--------------|-------------|
| `program` | the `program()` call | `lint`, `tool` |
| `reads`, `writes` | `Input` records and `emit` paths under the granted folders | `generator`, `bindgen`, `tool` |
| `net` | outgoing connections to hosts named in the grant | `tool` |
| `clock` | the current time | `tool` |
| `process` | start programs named in the grant | `tool` |

- `lint`, `generator` and `bindgen` can use only `program` (a lint) and
  `reads` and `writes` (the others). They can never reach `net`, `clock` or
  `process`. A generator that fetches a schema over the network is two steps:
  a `tool` fetches it, and the generator reads the file.
- A `tool` may reach `net` only to hosts the grant names, for example
  `capabilities = ["net:registry.example.com"]`.
- There is no `random` and no `env`. A generator that needs an identifier
  derives it from its inputs.

The wording of a refused request is in [16](../reference/diagnostics-catalog.md):
V0901 for a capability that was not granted, and V0815 for a plugin that asks
for more than its grant.

## 18.7 What a plugin reports

`report` takes a `Diagnostic` with the fields of 02 §2.8 and these limits, all
checked by the host before the diagnostic reaches the user.

| Field | Limit |
|-------|-------|
| `code` | `plugin-name/rule`, where `plugin-name` is the plugin's own name. Anything else is dropped and `V0816` is reported in its place |
| `severity` | `warning` or `note`. An `error` is lowered to `warning`, unless the project promoted the rule (`promote` in 19 §19.3) |
| `message` | At most one sentence of at most 25 words, with no word from the "don't say" table (08 §8.7) |
| `position` | Must lie inside the program or a granted input |
| `suggestions` | A list of edits, each `{position, replacement}`. Each edit must still produce a program that parses; the host checks it and drops an edit that does not |
| `rule` documentation | Every rule has a long explanation in `plugin.toml` under `[rules."style/prefer-map"]`, shown by `verse explain` (16.5) |

A plugin that reports more than 200 diagnostics in one run has the rest
counted and shown as one line: `style reported 340 more warnings; use --all`.

A `lint` cannot remove, change or hide a compiler diagnostic, because it
never sees the list of compiler diagnostics. It sees only the program.

## 18.8 What a plugin writes

`emit` takes a file record and the host decides whether to accept it.

| Field | Rule |
|-------|------|
| `path` | Under `generated/`, relative, no `..`, no existing hand-written file |
| `text` | Verse source. The host parses it. A parse error is attributed to the plugin and nothing is written |
| `header` | The host adds the "Generated by" header of 07 §7.3. A plugin cannot omit or change it |
| `inputs` | The granted inputs this file came from, recorded for `verse why <path>` |

- A generator that emits the same path twice is an error, so one run cannot
  hide its own output.
- Files are written together at the end of a run. A run that fails writes
  none.
- Generated files are then checked by the compiler like any other source
  (07 §7.3). A type or ownership error in a generated file is shown with the
  note "this file was generated by `jsonschema` 1.2.0".

## 18.9 Interface version

The version is one integer, written in `plugin.toml` as `interface`.

| Change | Version |
|--------|---------|
| A new optional call, new record field or new capability | Same version; old plugins ignore it |
| A removed or renamed call, field or capability, or a changed meaning | New version |
| A plugin built for an older version | Runs if the host still supports it, and `verse plugins` shows the version |
| A plugin built for a newer version than the host | Refused with `V0817`, which names both versions |

The host supports the current version and the one before it. A plugin may
declare only one version.

## 18.10 Limits

Defaults, chosen by the host to be confirmed in M5, and recorded in `verse.lock`
only when the project changes them.

| Limit | Default | When exceeded |
|-------|---------|---------------|
| fuel | 100 million fuel units (about one per Wasm instruction) per run | the run stops: "`jsonschema` ran out of fuel (100 million steps)" |
| memory | 128 MiB | the run stops, naming the size |
| output | 4 MiB of generated text per run, 256 files | the run stops, naming the count or size |
| diagnostics | 200 shown per run (18.7) | the rest is counted |
| time | 10 seconds per run | the run is a host failure, see below |

The host runs at most as many plugins at once as `verse build` has jobs.

**What a stop is.** Fuel, memory and output are counted exactly, so the same
plugin on the same inputs stops at the same point on every machine. That stop
is part of the result and is cached like any other result. Time depends on the
machine, so a time stop is a host failure: it is never cached, and it is
reported as V0822, "this limit depends on the machine". This keeps one rule:
same inputs and same plugin bytes give the same output (18.1 rule 6).

**What a stop does to the build.** A stopped plugin writes nothing. For a
`lint` the stop is a warning (V0819, V0820 or V0821). For a `generator`, a
`bindgen` or a `tool` the same codes are errors, so the user sees the real
cause and not a later "unknown name" for a file that was never written. A time
stop is an error for every kind.

**Cache key.** A plugin result is cached by the hash of the plugin file, the
interface version, the granted inputs and the limits, so changing a limit
invalidates the cache.

`verse-plugin` generates its interface file from the tables of 18.3 to 18.8.
A build check fails if a call, a record field or a capability named here has
no entry in that file, or the other way round.

## 18.11 Open points

1. **Ownership state for lints.** Some rules want to know where a value is
   moved. Proposal: not in interface 1. The ownership view of the debugger
   ([08](debugging-and-mcp.md)) is the model to expose later.
2. **Renderers for `tool` plugins.** Decided and applied in 18.12.
3. **Catalog codes.** V0814 to V0817 are used here and are in 16 §16.3.
4. **Languages for writing a plugin.** Any language that produces a Wasm
   component works. A Verse SDK for plugin authors is a first-party package
   question for [20](first-party-packages.md).
5. **Bindgen inputs.** A C header needs a parser the plugin brings itself.
   Whether the host offers a shared one is deferred with `extern` (07 §7.8).

## 18.12 Renderers (kind `tool`)

A `tool` plugin may list `renders = ["TypeName", …]` in `plugin.toml` and export
`render(value)`. A renderer is a small, pure function that runs only inside the
debugger host. It never runs in the program, so script and compile modes stay
identical. Decision.

| Part | Rule |
|------|------|
| Input | the value record of [08 §8.4](debugging-and-mcp.md): type name, state, presence, and fields cut to the depth and size the debug call asked for. No addresses |
| Output | one string of at most 25 words, plain, present tense, in the words of 08 §8.7 |
| Check | the host checks the string against the "don't say" table. A string that fails, or is longer than 25 words, is dropped, the built-in summary is used, and the user sees one warning for the type |
| Both shown | the value record carries `summary` (built in) and `rendered` with `rendered_by`, so a reader always has the built-in line |
| Limits | 1 million fuel units, 50 ms and 16 MiB per call. A stop uses the built-in summary |
| Replay | recordings store raw values only; a renderer never changes a recording |
| Capabilities | none; a renderer asks for no inputs and no files |
