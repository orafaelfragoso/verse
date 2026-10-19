# 19. CLI, manifests and package manager

This is the implementable reference for
the `verse` commands that manage projects, and for the three files a project
carries: `verse.toml` (what the author asks for), `verse.lock` (what was
chosen), and `verse.api` (what a package promises).

Nothing here changes what a Verse program means. The spec (§12, §16.3, §16.4)
wins wherever this document disagrees with it.

Related documents: [06](../architecture/stdlib-and-shaking.md) (reachability, size report,
`[library]`), [07](../architecture/plugins.md) (native plugins), [02 §2.7](../architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)
(cache), [04 §4.3](../architecture/backends-and-targets.md#43-what-a-target-is) (target
files), [20](../architecture/first-party-packages.md) (the first-party packages), and
[16](diagnostics-catalog.md) (diagnostics), and
[08](../architecture/debugging.md) (the debugger). The codes shown below are in
16, and their long forms are read with `verse explain`
([16 §16.5](diagnostics-catalog.md#165-verse-explain)).

## 19.1 Summary

This is the reference for the commands, the `verse.toml` keys and the exit
statuses. What a package is, how `import` finds one, and how versions are
chosen, locked and published are in
[21. Packages](../architecture/packages.md).

## 19.2 Commands

The spec's tool table (§16) lists the commands; this section fixes their
behaviour. `verse lock`, `verse vendor`, `verse yank` and `verse api diff`
extend that table.

| Command | What it does | Reads | Writes | Network |
|---------|--------------|-------|--------|---------|
| `verse new name` | project skeleton: `verse.toml`, `src/main.vs`, `.gitignore` with `.verse/` | — | new folder | no |
| `verse add pkg[@req]` | adds a dependency at the newest release (or `req`), resolves, updates the lock | toml, lock, index | toml, lock | yes (cached) |
| `verse add --dev pkg` | the same, into `[dev-dependencies]` | | | |
| `verse remove pkg` | removes it; the lock drops what nothing else needs | toml, lock | toml, lock | no |
| `verse update [pkg…]` | raises locked versions to the newest allowed by `verse.toml` (21 §21.8); shows each change and each capability change; `--exclude pkg` leaves `pkg` at its locked version | toml, lock, index | lock | yes |
| `verse lock` | resolves and writes `verse.lock` without building; `--check` fails if the lock is stale (for CI); `--refetch pkg` fetches `pkg` again after a checksum or API mismatch (V0806, V0824) | toml, lock, index | lock | yes, unless `--offline` |
| `verse vendor` | copies every locked package into `vendor/` and records it in the lock (21 §21.9); `--restore pkg` puts back the locked copy of an edited package (V0829) | lock | `vendor/`, lock | yes, unless cached |
| `verse publish` | checks, computes the required bump, packs and uploads (21 §21.6, §21.9) | everything | registry | yes |
| `verse yank pkg@ver` | marks a published version yanked (21 §21.9); `--undo` reverses | — | registry | yes |
| `verse why <item-or-path>` | spec §16: for an item, the chain from a root (06 §6.6); for a file path, the `verse generate` entry that wrote it (spec §16.6); this document adds package names and capabilities (§19.4) | build graph, lock | — | no |
| `verse api update` / `verse api check` | as spec §16.4 | sources | `verse.api` | no |
| `verse generate [name]` | runs the `[generate]` entries (spec §16.6): empties `generated/<name>/` and runs the entry's tool | toml, lock, the inputs the tool reads | `generated/` | only if resolving |
| `verse bind <header.h> --plugin <name> --out <file>` | writes an `extern` block for a C header ([07 §7.7](../architecture/plugins.md#77-verse-bind)); never overwrites a file | the header | `<file>` | no |
| `verse api diff [ver]` | the `verse.api` diff against a published version, with the required bump | `verse.api`, registry | — | yes (cached) |
| `verse debug [app.vs] [--port n]` | the debug adapter: serves the Debug Adapter Protocol on standard input and output, or on a local port ([08 §8.3](../architecture/debugging.md#83-the-debug-adapter)) | sources, recordings | — | no |
| `verse test [path]` | runs `test` blocks and doc tests (spec §14); `--filter "name"` selects tests, `--watch` reruns on change. In a workspace only the root's tests run unless `--deps` is given ([21 §21.13](../architecture/packages.md#2113-supply-chain-security)) | sources, toml, lock | - | only if resolving |
| `verse fmt [path]` | formats code in the one canonical style (spec §16.1); `--check` changes nothing and fails on unformatted code (for CI) | sources | sources | no |
| `verse check` | type-checks without running (§19.2.3) | sources, toml, lock | - | only if resolving |
| `verse build [file.vs]` | builds the package, or one file, for the selected profile and target: an executable, a WebAssembly module or a bytecode bundle (§19.2.3) | sources, toml, lock | build output | only if resolving |
| `verse doc` | generates an HTML site from doc comments (spec §14.3) | sources | HTML | no |
| `verse repl` | an interactive prompt; it runs in tier 0 ([14 §14.1](../architecture/bytecode-and-vm.md#141-role)) | - | - | no |
| `verse lsp` | the language server for editors, with inline hints for inferred types ([08 §8.8](../architecture/debugging.md#88-verse-extensions)) | sources | - | no |
| `verse explain <code>` | prints the long form of a diagnostic ([16 §16.5](diagnostics-catalog.md#165-verse-explain)) | - | - | no |
| `verse record redact <file>` | removes chosen capability results from a recording; a replay then reports a divergence at the first removed result it needed ([08 §8.9](../architecture/debugging.md#89-safety), [14 §14.10](../architecture/bytecode-and-vm.md#1410-record-and-replay)) | the recording | the recording | no |
| `verse bench` | runs the benchmarks in `bench/`, stores the results as JSON and compares them with the committed baseline ([09 §9.2](../architecture/performance.md#92-benchmark-suite)) | `bench/` | results | no |
| `verse difftest` | runs each program of the conformance suite in tier 0, Cranelift, LLVM and Wasm, with the optimizer on and off and recording on and off, and reports the first differing observable ([10 §10.3](../architecture/testing-and-verification.md#103-differential-testing), [14 §14.12](../architecture/bytecode-and-vm.md#1412-differential-testing-contract)) | the suite | report | no |
| `verse dump mir` | prints the MIR text format, including the `layout` lines, for golden tests ([13 §13.2](../architecture/mir-specification.md#132-textual-format), [15 §15.12](../architecture/target-and-abi.md#1512-conformance-and-tests)) | sources | - | no |
| `verse app.vs`, `verse .`, `build`, `check`, `test` | read the lock; resolve only if it is missing or stale (and say so) | toml, lock | lock (only if stale) | only if resolving |

Global flags, accepted by every command that may touch the network:
`--offline` (never connect; use the cache and `vendor/`), `--locked` (fail
instead of changing `verse.lock`), `--frozen` (both). CI uses `--frozen`.
The names follow Cargo, which users already know.

Every command takes `--json` and then prints records, not prose, so a script
driving `verse` and a person at a terminal see the same facts. Positions are
`{"file", "line", "column"}`, and errors carry their code from
[16](diagnostics-catalog.md).

### 19.2.1 Running a file

`verse app.vs [args…]` compiles `app.vs` (cached) and runs it in script mode.
`verse .` runs the package in the current folder.

**Dispatch rule.** The first argument that is not a flag decides what `verse`
does. If it ends in `.vs` or contains `/`, it is a path to run. `.` is the
current folder. Any other word is a command.

| First argument | Example | What runs |
|----------------|---------|-----------|
| a path to a `.vs` file | `verse app.vs`, `verse tools/gen.vs` | that file, as the entry file (spec §12.5) |
| a path to a folder | `verse .`, `verse apps/server/` | the package in that folder, from its entry file: `[targets.<t>] entry` of the default target, `src/main.vs` unless the manifest says otherwise (§19.3) |
| a command | `verse build`, `verse test` | that command |
| none | `verse` | nothing: it prints the commands and how to run a file, and exits with status 0 |

A word that is not a command is an error, never a guess, even when a file
with that name exists. It exits with status 2:

```
error: no command `foo`; to run a file, write `foo.vs`
help: run `verse` with no arguments to see the commands
explain: V1101 (run `verse explain V1101`)
```

**Arguments.** Flags for `verse` come before the path. Every argument after
the path goes to the program unchanged, flags included, and the program reads
them as `args` (spec §12.5). No `--` separator is needed.

```
verse --deny net wordcount.vs --top 3 poem.txt
```

Here `--deny net` is for `verse`, and `args` is `["--top", "3", "poem.txt"]`.

| Flag | Effect |
|------|--------|
| `--deny <capability>` | refuse a capability; a program that can reach it fails before it starts (spec §16) |
| `--fast` | compile with Cranelift and run natively on this run. Without it, an unchanged script runs natively from its third run, built in the background during the second ([04 §4.1](../architecture/backends-and-targets.md#41-tiers)) |
| `--no-cache` | neither read nor write `.verse/cache/`: compile from source, and make no native build in the background |
| `--debug` | compile for a debugger and accept `attach` from `verse debug` ([14](../architecture/bytecode-and-vm.md), [08 §8.3](../architecture/debugging.md#83-the-debug-adapter)) |
| `--leak-check` | report leaked `Shared` cycles at exit ([05](../architecture/runtime.md)) |
| `--max-memory <size>` | cap the heap; running out is a panic ([05 §5.2](../architecture/runtime.md#52-memory)) |
| `--max-stack <size>` | cap each task's stack; a task that cannot grow panics with `stack_overflow` (R0004). The default is the target file's `[tasks] max_stack`, 8 MiB per task on hosted targets. Stacks grow as they are used, so the cap costs nothing until a task needs it ([14 §14.3](../architecture/bytecode-and-vm.md#143-value-representation-and-call-records), [15 §15.9.2](../architecture/target-and-abi.md#1592-schema)) |
| `--seed <n>` | seeded scheduler mode: scheduling is deterministic without a recording, so a failing concurrent run can be repeated ([08 §8.5](../architecture/debugging.md#85-deterministic-record-and-replay), policy in [14 §14.7](../architecture/bytecode-and-vm.md#147-tasks-scheduler-and-seed)) |

`[run]` in `verse.toml` sets defaults for these flags (§19.3).

### 19.2.2 Exit statuses

Running a file ends with the status of the program it ran; every command that
reports a problem uses the statuses below, so a script can tell the causes
apart. The table is the one of
[16 §16.6.1](diagnostics-catalog.md#1661-exit-statuses) (spec §12.5);
16 owns it.

| Status | Meaning |
|--------|---------|
| 0 | normal completion |
| 1 | a top-level `?` failed, or an unwaited task failed (spec §12.5) |
| 2 | `verse check` or `build`, or running a file, found errors in the source, or the command line names no command (V1101), so the program did not start. This includes a capability the target does not offer or the manifest does not grant (V0901, V0902, a `--deny`), found by the build-time check (15 §15.8.1) |
| 3 | a bug in Verse: a compiler bug (V9901 or V9902) found before the program started, when running a file as in `build`, or a trap inside the runtime that Verse itself caused. A run-time capability re-check that fails is also 3, because the build-time check should have caught it |
| 70 | a deadlock (R0008) |
| 101 | a panic |
| *code* | the program called `exit(code)`; `code` is 0 to 255, it runs no `Cleanup`, and it is never changed (spec §12.5) |

A build verifier failure is status 3, not 1. A package-manager error (a stale
lock under `--locked`, a failed resolution) is not covered by the spec or by
16 §16.6.1; this document assumes a non-zero status and fixes no number
(Assumption).

### 19.2.3 Building and checking

`verse check` takes `--target <name>`: it checks that everything the program
can reach is available on that target, without generating code
([06 §6.8](../architecture/stdlib-and-shaking.md#68-per-target-constraints)).
After a compiler failure (V9901), `verse check --report` writes a file to send
with the bug report.

| `verse build` flag | Effect |
|--------------------|--------|
| `--release` | build with `[profile.release]` (LLVM, §19.3) |
| `--profile <name>` | build with `[profile.<name>]` (§19.3) |
| `--target <name>` | build for a target file other than `[targets] default` (§19.3) |
| `--size-report` | also list every included item with its size and why it is included; `--size-report=json` prints records ([06 §6.6](../architecture/stdlib-and-shaking.md#66-verse-build---size-report)) |
| `--no-heap` | fail if any reachable function allocates ([05 §5.7](../architecture/runtime.md#57-embedded-profile)) |
| `--record` | the built program logs its nondeterministic inputs, so the VM can replay the run; setting `VERSE_RECORD=file` when the program runs does the same ([08 §8.2](../architecture/debugging.md#82-architecture)) |
| `--verify-reproducible` | build twice in clean directories and compare the bytes (for CI; [21 §21.10](../architecture/packages.md#2110-reproducible-builds-and-caching)) |

Compiler flags change what the compiler reports or how it optimizes, never
what a program means:

| Flag | Effect |
|------|--------|
| `--no-opt-pass=NAME` | turn off one optimizer pass, so a miscompile can be isolated ([03 §3.4](../architecture/ir-and-passes.md#34-pass-list)) |
| `--timings` | record the time and memory of every pass, as JSON for tools ([09 §9.5](../architecture/performance.md#95-compile-time-budget-discipline)) |
| `--explain-ownership` | show the `V0511` note on how chain ownership works, which is never printed by default ([16 §16.4](diagnostics-catalog.md#164-how-a-code-is-shown)) |

## 19.3 `verse.toml`

TOML 1.0. Unknown sections and keys are errors, not warnings, because a
misspelt key silently ignored would change a build. Every key below
is listed with its type and default. Sections marked *spec* are in spec
§16.3; this document defines the others.

### `[package]` *spec*

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `name` | text | required | the package name, also its root module name (21 §21.3) |
| `version` | text, semver | required | `MAJOR.MINOR.PATCH`, optional pre-release (`1.0.0-rc.1`), no build metadata |
| `verse` | text, version requirement | required | toolchain versions the package supports, same syntax as dependencies (`"0.1"` means `>= 0.1.0, < 0.2.0` while the major is 0) |
| `description` | text | `""` | one line, shown by the registry |
| `license` | text, SPDX expression | required to publish | `"MIT"`, `"MIT OR Apache-2.0"` |
| `authors` | list of text | `[]` | |
| `repository` | text, URL | none | |
| `readme` | text, path | `"README.md"` if present | |
| `include` | list of globs | `["src/**", "generated/**", "verse.toml", "verse.api", "README*", "LICENSE*"]` | what `verse publish` packs; nothing else is uploaded |

### `[dependencies]` and `[dev-dependencies]` *spec*

```toml
[dependencies]
colors = "1.2"                                   # registry, >= 1.2.0, < 2.0.0
finance = "1.0"                                  # first-party, same rules
http_extra = { git = "https://example.com/http_extra.git", tag = "v0.4.1" }
shapes = { path = "../shapes" }                  # local, for workspaces and development
term = { version = "0.3", registry = "corp" }    # a named alternative registry (21 §21.9)
```

| Form | Meaning |
|------|---------|
| `"1.2"` | caret requirement: `>= 1.2.0, < 2.0.0`; for `0.y`, `>= 0.y.0, < 0.(y+1).0` (the 1.x rule is the spec's; the `0.y` rule is Cargo's) |
| `"=1.2.3"` | exactly that version (discouraged; the registry shows a warning on publish) |
| `{ version = …, registry = … }` | from a named registry |
| `{ git = …, tag \| rev = … }` | git source; `branch` is not accepted, because a branch is not reproducible |
| `{ path = … }` | local source; not allowed in a published package |

A `branch` key is refused:

```
error: `branch` is not accepted
  --> verse.toml:9:60
   |
 9 | http_extra = { git = "https://example.com/http_extra.git", branch = "main" }
   |                                                            ^^^^^^ a branch changes without a new version
   |
help: use `tag = "…"` or `rev = "…"`
explain: V0830 (run `verse explain V0830`)
```

There is no `>=`-only form and no `*`: every requirement has an upper bound
at the next major (one version per major, spec §16.3).
A requirement without an upper bound is refused:

```
error: `>= 1.2` has no upper bound
  --> verse.toml:7:10
   |
 7 | colors = ">= 1.2"
   |          ^^^^^^^^ any new major version would be accepted
   |
help: write "1.2", which means `>= 1.2.0, < 2.0.0`
explain: V0831 (run `verse explain V0831`)
```
`[dev-dependencies]` are visible only to `test` blocks and doc tests (spec
§16.3, 06 §6.7).

### `[library]` *spec*

As spec §16.3 and [06 §6.9](../architecture/stdlib-and-shaking.md#69-package-format-of-a-domain-library):

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `modules` | list of module paths | every module under `src/` | the importable modules; others are private to the package |
| `capabilities` | list of capability names | `[]` | the most any module may reach; `verse publish` fails if reachability from the exported items finds more |
| `size_budget` | text, size (`"40K"`, `"2M"`) | none | `verse publish` builds the package's `examples/` and fails if the package's share exceeds it |
| `[library.targets] default` | `"all"` or `"none"` | `"all"` | |
| `[library.targets] available` / `unavailable` | list of target names | `[]` | exceptions to `default` |

### `[plugins]`

Native libraries called through `extern` ([07 §7.2](../architecture/plugins.md#72-declaring-a-plugin),
spec §12.6). One table per plugin; the name is what `extern <name>` refers to.

```toml
[plugins.sqlite]
lib = "sqlite3"

[plugins.fastmath]
path  = "native/fastmath/target/release"
build = "cargo build --release --manifest-path native/fastmath/Cargo.toml"
link  = "static"
```

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `lib` | text | the plugin name | the library's file name without platform prefix and suffix |
| `path` | path | none | the folder that holds the library; without it, the system linker paths and `pkg-config` are searched |
| `build` | text | none | a command run before each build or run that links or loads the library |
| `link` | `"dynamic"` \| `"static"` | `"dynamic"` | how compiled mode links it; script mode always needs a shared library (V1011) |
| `targets` | list of target names | every target | where the plugin is available ([07 §7.8](../architecture/plugins.md#78-targets)) |

A dependency may declare plugins with `lib` only. A dependency that uses
`path` or `build` is rejected at resolution (V0834, 21 §21.13).

### `[generate]` and `[tool]`

`[generate.<name>]` lists a generator run by `verse generate` (spec §16.6).
`[tool]` is how a package offers the command a generator runs.

```toml
[generate.items]
tool = "jsonschema"
args = ["schema/item.json"]
```

| Section | Key | Type | Default | Meaning |
|---------|-----|------|---------|---------|
| `[generate.<name>]` | `tool` | package name | required | a dependency (usually in `[dev-dependencies]`) that has a `[tool]` section |
| `[generate.<name>]` | `args` | list of text | `[]` | passed to the command, followed by `--out generated/<name>/` |
| `[tool]` | `main` | path | required | the entry file of the package's command; it is never part of a program that imports the package |

### `[capabilities]` (applications)

What the program is granted. Absent means "whatever the target provides".
Present means the build fails if reachability finds a capability outside the
list; `verse --deny <capability> app.vs` (§19.2.1) narrows it further for one run.

```toml
[capabilities]
grant = ["stdio", "net", "clock"]
```

This turns "my CLI tool never touches the network" into a checked fact, and
makes a dependency update that starts reaching `net` fail the build instead
of passing silently.

### `[targets]`

The targets the project is built and checked for, by name of a target file
([04 §4.3](../architecture/backends-and-targets.md#43-what-a-target-is), and see *Target files* below). `verse build`
with no `--target` uses `default`. Per-target data only, never per-target
source conditions ([06 §6.9](../architecture/stdlib-and-shaking.md#69-package-format-of-a-domain-library)).

```toml
[targets]
default = "host"
check   = ["host", "wasm32-wasip1", "thumbv7em-none-eabihf"]   # `verse check` runs on each

[targets.thumbv7em-none-eabihf]
heap   = "32K"            # the embedded heap of 05 §5.7, here per target
no_heap = false           # true makes every build behave as `--no-heap`
entry  = "src/board.vs"   # a different entry file for this target
```

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `default` | target name | `"host"` | |
| `check` | list of target names | `[default]` | `verse check` checks every listed target ([06 §6.8](../architecture/stdlib-and-shaking.md#68-per-target-constraints)) |
| `<target>.heap` | size | target file's value | the embedded heap of 05 §5.7 |
| `<target>.no_heap` | bool | `false` | |
| `<target>.entry` | path | `src/main.vs` | the per-target choice is a file, not a condition in source |
| `<target>.files` | map of module path to file | `{}` | module variants for that target; each variant must produce the same `verse.api` |

### Target files

A target name in `[targets]` (and in `[library.targets]`) refers to a **target
file**, TOML with the schema of
[15 §15.9.2](../architecture/target-and-abi.md#1592-schema). Where the files live and the
order in which a name is resolved are defined by
[15 §15.9.1](../architecture/target-and-abi.md#1591-files-and-lookup); this section only
restates them.

| File | Where | Defines |
|------|-------|---------|
| `targets/<name>.toml` | the toolchain | a shipped target, such as `wasm32-wasip1`, `wasm32-wasip2` or `wasm32-unknown-unknown` |
| `targets/<name>.target` | the project | a project-local target `<name>` |
| `verse.target` | the package root | exactly one project-local target; its `[target] name` is required |

A project selects a target by writing its name in `[targets] default` or
`check`, or with `--target <name>`; there is no other switch. The name `host`
is not a file: it is the shipped target of the machine running the compiler.
A project-local target may not reuse the name of a shipped one. A target file
is data and runs no code. `[targets.<t>]` (above) holds what the project asks
of the platform, and the target file holds what the platform is; which of the
two owns a key is the table in
[15 §15.9.5](../architecture/target-and-abi.md#1595-how-the-target-file-relates-to-the-manifest).
The hash of the resolved target file is part of every cache key (21 §21.10).

### `[profile.dev]`, `[profile.release]`, `[profile.test]`

| Key | Type | dev | release | test | Meaning |
|-----|------|-----|---------|------|---------|
| `backend` | `auto` \| `bytecode` \| `cranelift` \| `llvm` | `cranelift` | `llvm` | `cranelift` | tier ([04](../architecture/backends-and-targets.md)); `auto` lets the target file decide |
| `opt` | `0`–`3`, `"size"` | `0` | `3` | `1` | optimizer level (03 §3.4) |
| `debug` | bool | `true` | `false` | `true` | Verse debug map and DWARF ([08](../architecture/debugging.md)) |
| `unwind` | `target` \| `abort` | `target` | `target` | `target` | `abort` only when no `Cleanup` value is live at any panic site; otherwise the build fails |
| `size_report` | bool | `false` | `false` | `false` | as `--size-report` |

There is deliberately no key that turns off overflow or bounds checks: they
are part of what a program means (spec §4.1, §9.3), so a profile cannot change
them. A custom profile is `[profile.name]` with `inherits = "release"`. The profile
is part of every cache key (21 §21.10).

### `[run]`, `[build]`, `[test]`

Defaults for command-line flags, so CI and people run the same thing.
Command-line flags win. `[run]` applies when a file or the package runs
(§19.2.1).

| Section | Key | Type | Default |
|---------|-----|------|---------|
| `[run]` | `deny` | list of capabilities | `[]` |
| `[run]` | `max_memory` | size | none ([05 §5.2](../architecture/runtime.md#52-memory)) |
| `[run]` | `native_after` | integer | `2`: the run of an unchanged script that also builds it natively in the background; later runs use that build ([04 §4.1](../architecture/backends-and-targets.md#41-tiers)) |
| `[build]` | `out` | path | `.verse/out/<target>/<profile>/` |
| `[build]` | `artifact` | `executable` \| `library` \| `wasm-component` \| `bytecode` | `executable` ([04 §4.4](../architecture/backends-and-targets.md#44-linking-and-artifacts)) |
| `[test]` | `filter` | text | none |
| `[test]` | `fakes` | list of capabilities | `["clock", "random"]`: tests get the virtual clock and a seeded random source ([05 §5.6](../architecture/runtime.md#56-the-sys-capability-interface)) |
| `[test]` | `timeout` | duration text (`"10s"`) | `"60s"` |

### Features

There is no `[features]` section. Why, and what to use instead, is in
[21 §21.11](../architecture/packages.md#2111-features).

### `[workspace]`

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `members` | list of paths | required | the member packages, relative to the workspace root ([21 §21.12](../architecture/packages.md#2112-workspaces)) |

## 19.4 `verse why`

One command answers "why is this here?" for three kinds of argument, chosen
by its form. Items and file paths are spec §16; packages and
capabilities are added here:

| Argument | Example | Answer |
|----------|---------|--------|
| an item (dotted path) | `verse why http.get` | the edge chain from a root (06 §6.6) |
| a package | `verse why colors` | the dependency chain that requires the selected version, and the largest minimum that chose it (21 §21.8) |
| a file path (contains `/` or ends in `.vs`) | `verse why generated/items/item.vs` | the `[generate]` entry that wrote it (spec §16.6) |
| a capability | `verse why net` | the chain to the first `sys.net` use (06 §6.6) |

```
$ verse why colors
colors 1.5.1 is selected because:
  your verse.lock asks for >= 1.5.1 (written by `verse update`)
  charts 2.0.0 asks for >= 1.4.0
  your verse.toml asks for >= 1.2.0

$ verse why http.get
http.get is included because:
  src/report.vs:22 calls colors.fetch_palette
  colors.fetch_palette calls http.get
```

The openers are fixed wording, taken from
[`events.toml`](events.toml): `why.package` ("… is selected
because:"), `why.item` ("… is included because:"), `why.capability` ("this
program reaches `…` because:") and `why.file` ("… was written by …"). Each
answer is a **chain**.

Ambiguity is impossible by construction: package names never contain dots
(21 §21.3), items always do, capabilities are a closed list, and paths contain
`/` or `.vs`. A package named like a capability (`net`) is refused by the
registry (V0833, 21 §21.3).

## 19.5 Where each code appears

| Code | Message (16) | Where |
|------|--------------|------------|
| V0801 | `{key}` is not a key of `[{section}]` | §19.3 (unknown keys are errors) |
| V0802 | the package name `{name}` cannot be imported | 21 §21.3 |
| V0803 | version {have} is too small for these changes; publish as {need} | 21 §21.6 (message) |
| V0804 | version {req} of `{pkg}` does not exist | 21 §21.13 (message), §21.8 |
| V0806 | `{pkg}` {ver} does not match its checksum | 21 §21.13 (message), §21.7 `checksum` |
| V0807 | `verse.lock` is out of date | §19.2 (`--locked`), 21 §21.7 |
| V0808 | `{pkg}` needs {T}, which `{other}` does not provide | 21 §21.8 (two majors) |
| V0809 | a path dependency cannot be published | 21 §21.12 |
| V0810, V0811 | `verse.api` changed / differs from the source | 21 §21.6, spec §16.4 |
| V0812 | `{name}` is not a valid license | §19.3 `license` |
| V0813 | a yanked version cannot be chosen | 21 §21.8 step 5, §21.9 |
| V0823 | this lock was made with {old}, and you run {new} | 21 §21.7 (message) |
| V0824 | `{pkg}` {ver} has a different API from the one in your lock | 21 §21.7 (message) |
| V0825 | `{T}` from `{pkg}` {a} is not `{T}` from `{pkg}` {b} | 21 §21.8 (message) |
| V0826 | `{pkg}` {ver}, in your lock, was yanked: {reason} | 21 §21.9 Yanking (message), §21.8 step 5 |
| V0827 | updating `{pkg}` adds the `{cap}` capability | 21 §21.8 (`verse update` output) |
| V0828 | `{pkg}` {ver} is not in the cache or in `vendor/`, and `--offline` is on | 21 §21.9 Offline mode (message) |
| V0829 | `{file}` in `vendor/` was edited | 21 §21.9 Vendoring (message) |
| V0830 | `branch` is not accepted | §19.3 `[dependencies]` (message) |
| V0831 | `{req}` has no upper bound | §19.3 `[dependencies]` (message) |
| V0832 | two clean builds of this project differ | 21 §21.10 (message) |
| V0833 | the package name `{name}` is not available | 21 §21.3 (messages) |
| V0834 | `{pkg}` declares a plugin with `{key}`, which only the root project may use | 21 §21.13 (message), §19.3 `[plugins]` |
| V0835 | `{name}` is both a module of this package and a dependency | 21 §21.4 |
| V0836 | no module or package is named `{name}` | 21 §21.4 |
| V0837 | `{name}` is the name of a standard library module | 21 §21.4 |
| V0901 | this program reaches the `{cap}` capability, which verse.toml does not grant | 21 §21.13 (message), §19.3 `[capabilities]` |
| V0904 | `{pkg}` reaches `{cap}`, which it does not declare | §19.3 `[library] capabilities` |
| V0905 | this package uses {n} KB, but its size budget is {m} KB | §19.3 `size_budget` |
| V0906 | `{module}` has different public items on `{target}` | §19.3 `[targets.<t>.files]` |
| V0907 | the file `{f}` for target `{t}` does not exist | §19.3 `[targets.<t>.files]` |
