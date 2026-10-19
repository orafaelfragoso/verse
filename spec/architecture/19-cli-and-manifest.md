# Architecture 19. CLI, manifests and package manager

Owner: Picasso. Status: **draft 1**. This is the implementable reference for
the `verse` commands that manage projects, and for the three files a project
carries: `verse.toml` (what the author asks for), `verse.lock` (what was
chosen), and `verse.api` (what a package promises). Labels follow the
[README](README.md) conventions: **Fact** (stated by the spec or a ruled
document), **Decision** (chosen here, inside the spec's room), **Assumption**,
**Target**, **Proposal** (needs a ruling, collected in §19.14).

Nothing here changes what a Verse program means. The spec (§12, §16.3, §16.4)
wins wherever this document disagrees with it.

Related documents: [06](06-stdlib-and-shaking.md) (reachability, size report,
`[library]`), [07](07-plugins.md) (plugins, trust), [18](18-plugin-interface.md)
(the plugin interface: manifest, grants, limits), [02 §2.7](02-compiler-architecture.md#27-incremental-and-parallel-compilation)
(cache), [04 §4.3](04-backends-and-targets.md#43-what-a-target-is) (target
files), [20](20-first-party-packages.md) (the first-party packages), and
[16-diagnostics-catalog.md](16-diagnostics-catalog.md),
[17-mcp-tool-schemas.md](17-mcp-tool-schemas.md) ([§17.8](17-mcp-tool-schemas.md#178-package-manager-tools)
covers the package manager), all Michelangelo's. The codes shown below are in
16, and their long forms are read with `verse explain`
([16 §16.5](16-diagnostics-catalog.md#165-verse-explain)).

## 19.1 Summary

1. **Three files, three jobs.** `verse.toml` is written by people, `verse.lock`
   and `verse.api` are written by the tool and committed. A reviewer reads a
   diff of any of them and knows what changed (Fact: spec §16.3, §16.4).
2. **No code runs to build a dependency.** No build scripts, no install
   hooks, no plugins from dependencies (Fact: spec §16.3, 07 §7.5).
3. **Minimal version selection** picks versions, so the same manifests give
   the same graph without a solver and without the lock (Decision, §19.7).
4. **The required version bump is computed** from the `verse.api` diff, and
   `verse publish` refuses a version that understates it (Decision, §19.5).
5. **Everything is content-addressed** with blake3, the same hash family the
   compiler and the shaking cache use (Fact: 02 §2.7, 06 §6.10).

## 19.2 Commands

The spec's tool table (§16) lists the commands; this section fixes their
behaviour. Commands marked *new* are proposals (P19-1).

| Command | What it does | Reads | Writes | Network |
|---------|--------------|-------|--------|---------|
| `verse new name` | project skeleton: `verse.toml`, `src/main.vs`, `.gitignore` with `.verse/` | — | new folder | no |
| `verse add pkg[@req]` | adds a dependency at the newest release (or `req`), resolves, updates the lock | toml, lock, index | toml, lock | yes (cached) |
| `verse add --dev pkg` | the same, into `[dev-dependencies]` | | | |
| `verse remove pkg` | removes it; the lock drops what nothing else needs | toml, lock | toml, lock | no |
| `verse update [pkg…]` | raises locked versions to the newest allowed by `verse.toml` (§19.7); shows each change and each capability change | toml, lock, index | lock | yes |
| `verse lock` *new* | resolves and writes `verse.lock` without building; `--check` fails if the lock is stale (for CI) | toml, lock, index | lock | yes, unless `--offline` |
| `verse vendor` *new* | copies every locked package into `vendor/` and records it in the lock (§19.9) | lock | `vendor/`, lock | yes, unless cached |
| `verse publish` | checks, computes the required bump, packs and uploads (§19.5, §19.8) | everything | registry | yes |
| `verse yank pkg@ver` *new* | marks a published version yanked (§19.8); `--undo` reverses | — | registry | yes |
| `verse why <item-or-path>` | spec §16: for an item, the chain from a root (06 §6.6); for a file path, the plugin that wrote it (07 §7.7); this document adds package names and capabilities (§19.11, P19-8) | build graph, lock | — | no |
| `verse api update` / `verse api check` | as spec §16.4 | sources | `verse.api` | no |
| `verse api diff [ver]` *new* | the `verse.api` diff against a published version, with the required bump | `verse.api`, registry | — | yes (cached) |
| `verse run`, `build`, `check`, `test` | read the lock; resolve only if it is missing or stale (and say so) | toml, lock | lock (only if stale) | only if resolving |

Global flags, accepted by every command that may touch the network:
`--offline` (never connect; use the cache and `vendor/`), `--locked` (fail
instead of changing `verse.lock`), `--frozen` (both). CI uses `--frozen`
(Decision; the names follow Cargo, which users already know).

Every command takes `--json` and then prints records, not prose, so an LLM
driving `verse` and a person at a terminal see the same facts (consistent
with [08](08-debugging-and-mcp.md)). The shapes follow the rules and shared
types of [17-mcp-tool-schemas.md](17-mcp-tool-schemas.md) (17.1, 17.2), and
17 applies them to the package manager in [§17.8](17-mcp-tool-schemas.md#178-package-manager-tools) (P19-12, agreed).

## 19.3 `verse.toml`

TOML 1.0. Unknown sections and keys are errors, not warnings, because a
misspelt key silently ignored would change a build (Decision). Every key below
is listed with its type and default. Sections marked *spec* are in spec
§16.3; the others are proposals (P19-2) unless another document already
uses them.

### `[package]` *spec*

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `name` | text | required | the package name, also its root module name (§19.4) |
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
term = { version = "0.3", registry = "corp" }    # a named alternative registry (§19.8)
```

| Form | Meaning |
|------|---------|
| `"1.2"` | caret requirement: `>= 1.2.0, < 2.0.0`; for `0.y`, `>= 0.y.0, < 0.(y+1).0` (Fact for 1.x: the spec's comment; the `0.y` rule is a Decision, as in Cargo) |
| `"=1.2.3"` | exactly that version (discouraged; the registry shows a warning on publish) |
| `{ version = …, registry = … }` | from a named registry |
| `{ git = …, tag \| rev = … }` | git source; `branch` is not accepted, because a branch is not reproducible (Decision) |
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
at the next major (Decision: one version per major, spec §16.3).
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
`[dev-dependencies]` are visible only to `test` blocks and doc tests (Fact:
spec §16.3, 06 §6.7).

### `[library]` *spec*

As spec §16.3 (R7-5) and [06 §6.9](06-stdlib-and-shaking.md#69-package-format-of-a-domain-library):

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `modules` | list of module paths | every module under `src/` | the importable modules; others are private to the package |
| `capabilities` | list of capability names | `[]` | the most any module may reach; `verse publish` fails if reachability from the exported items finds more |
| `size_budget` | text, size (`"40K"`, `"2M"`) | none | `verse publish` builds the package's `examples/` and fails if the package's share exceeds it |
| `[library.targets] default` | `"all"` or `"none"` | `"all"` | |
| `[library.targets] available` / `unavailable` | list of target names | `[]` | exceptions to `default` |
| `[library.native] links`, `link-search`, `pure` | lists, bool | — | **0.2**, with `extern` ([extern proposal](../reviews/extern-semantics.md)) |

### `[plugins]` (root only)

As [07 §7.5](07-plugins.md#75-trust-and-grants) and Q8: a separate section,
never in `[dependencies]`, and accepted only in the root manifest (workspace
root, §19.10). A dependency's manifest with `[plugins]` is rejected at
resolution (§19.12).

```toml
[plugins]
jsonschema = { version = "1.2", kind = "generator", reads = ["schema/"], writes = ["generated/"] }
style      = { version = "0.4", kind = "lint", promote = ["style/prefer-map"] }
```

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `version` | requirement | required | resolved like a dependency, locked with its content hash |
| `kind` | `lint` \| `generator` \| `bindgen` \| `tool` | required | must equal the kind in the plugin's own manifest ([18 §18.2](18-plugin-interface.md#182-the-plugin-manifest)); a mismatch is V0814 |
| `reads`, `writes` | lists of paths | `[]` | the grant; the tool refuses to run a plugin that asks for more ([18 §18.2](18-plugin-interface.md#182-the-plugin-manifest), [18 §18.6](18-plugin-interface.md#186-capabilities), 07 §7.5); that is V0815 |
| `capabilities` | list | `[]` | other capabilities granted (for a `tool` plugin, for example `net`) |
| `promote` | list of rule codes | `[]` | lint rules raised to `error` (07 §7.6, [18 §18.7](18-plugin-interface.md#187-what-a-plugin-reports)) |

A `kind` that differs from the plugin's own manifest stops the build:

```
error: plugin `jsonschema` is a `generator`, but verse.toml says `lint`
  --> verse.toml:6:40
   |
 6 | jsonschema = { version = "1.2", kind = "lint" }
   |                                        ^^^^^^ the plugin's own manifest says `generator`
   |
help: change `kind` in verse.toml to "generator"
explain: V0814 (run `verse explain V0814`)
```

A plugin that asks for more than the grant stops the build before it runs:

```
error: plugin `jsonschema` asks for more than verse.toml grants
  --> verse.toml:6:53
   |
 6 | jsonschema = { version = "1.2", kind = "generator", writes = ["generated/"] }
   |                                                     ^^^^^^^^^^^^^^^^^^^^^^^ it asks to write `docs/`; the grant is `generated/`
   |
help: grant it in `[plugins]`, or do not use this plugin
explain: V0815 (run `verse explain V0815`)
```

### `[capabilities]` (applications) — Proposal P19-3

What the program is granted. Absent means "whatever the target provides".
Present means the build fails if reachability finds a capability outside the
list; `verse run --deny` (spec §16) narrows it further for one run.

```toml
[capabilities]
grant = ["stdio", "net", "clock"]
```

This turns "my CLI tool never touches the network" into a checked fact, and
makes a dependency update that starts reaching `net` fail the build instead
of passing silently.

### `[targets]`

The targets the project is built and checked for, by name of a target file
([04 §4.3](04-backends-and-targets.md#43-what-a-target-is)). `verse build`
with no `--target` uses `default`. Per-target data only, never per-target
source conditions (Fact: P6-7 ruling).

```toml
[targets]
default = "host"
check   = ["host", "wasm32-wasi", "thumbv7em-none-eabihf"]   # `verse check` runs on each

[targets.thumbv7em-none-eabihf]
heap   = "32K"            # the embedded heap of 05 §5.7, here per target
no_heap = false           # true makes every build behave as `--no-heap`
entry  = "src/board.vs"   # a different entry file for this target
```

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `default` | target name | `"host"` | |
| `check` | list of target names | `[default]` | `verse check` checks every listed target ([06 §6.8](06-stdlib-and-shaking.md#68-per-target-constraints)) |
| `<target>.heap` | size | target file's value | replaces 05 §5.7's `[embedded] heap` (P19-4) |
| `<target>.no_heap` | bool | `false` | |
| `<target>.entry` | path | `src/main.vs` | the per-target choice is a file, not a condition in source (P6-7) |
| `<target>.files` | map of module path to file | `{}` | module variants for that target (P6-7); each variant must produce the same `verse.api` |

### `[profile.dev]`, `[profile.release]`, `[profile.test]`

| Key | Type | dev | release | test | Meaning |
|-----|------|-----|---------|------|---------|
| `backend` | `auto` \| `bytecode` \| `cranelift` \| `llvm` | `cranelift` | `llvm` | `cranelift` | tier ([04](04-backends-and-targets.md)); `auto` lets the target file decide |
| `opt` | `0`–`3`, `"size"` | `0` | `3` | `1` | optimizer level (03 §3.4) |
| `debug` | bool | `true` | `false` | `true` | Verse debug map and DWARF ([08](08-debugging-and-mcp.md)) |
| `unwind` | `target` \| `abort` | `target` | `target` | `target` | `abort` only as ruled in Q5: the build fails unless no `Cleanup` value is live at any panic site |
| `size_report` | bool | `false` | `false` | `false` | as `--size-report` |

There is deliberately no key that turns off overflow or bounds checks: they
are part of what a program means (spec §4.1, §9.3), so a profile cannot change
them. A custom profile is `[profile.name]` with `inherits = "release"`. The profile
is part of every cache key (§19.9).

### `[run]`, `[build]`, `[test]`

Defaults for command-line flags, so CI and people run the same thing.
Command-line flags win.

| Section | Key | Type | Default |
|---------|-----|------|---------|
| `[run]` | `deny` | list of capabilities | `[]` |
| `[run]` | `max_memory` | size | none ([05 §5.2](05-runtime.md#52-memory)) |
| `[build]` | `out` | path | `.verse/out/<target>/<profile>/` |
| `[build]` | `artifact` | `executable` \| `library` \| `wasm-component` \| `bytecode` | `executable` ([04 §4.4](04-backends-and-targets.md#44-linking-and-artifacts)) |
| `[test]` | `filter` | text | none |
| `[test]` | `fakes` | list of capabilities | `["clock", "random"]`: tests get the virtual clock and a seeded random source ([05 §5.6](05-runtime.md#56-the-sys-capability-interface)) |
| `[test]` | `timeout` | duration text (`"10s"`) | `"60s"` |

### Feature selection

**Decision: there is no `[features]` section in 0.1.** Cargo-style features
exist mainly to avoid compiling and linking unused code. In Verse that is
already free: an import costs nothing and only reached code is built
([06](06-stdlib-and-shaking.md)). The remaining uses are covered by data
that does not change what a source file means:

| Need | Verse answer |
|------|--------------|
| Leave out heavy code | do not reach it; the size report shows what is in |
| Optional dependency | a separate small package (`charts` and `charts_svg`), imported only where needed |
| Different code per target | `[targets.<t>.files]` (P6-7) |
| Different behaviour per build | values read at run time (`env`, `args`, a config file), or a separate entry file |

Why: a feature flag that changes which declarations exist is conditional
compilation, which P6-7 ruled out. It also makes "what does this file mean?"
depend on the whole graph's feature union. If real programs need more,
P19-5 proposes the smallest addition: named sets of optional dependencies,
never of source.

### `[workspace]`

See §19.10.

## 19.4 Package identity and naming

**A package name is its root module name** (Fact: spec §16.3,
`import red, bold from colors`). So a name must be a valid module path
segment:

| Rule | Why |
|------|-----|
| `[a-z][a-z0-9_]*`, at most 64 characters | a lowercase value identifier (spec §3.5); no hyphens, which cannot appear in `import … from name` |
| not a keyword, not `std`, `verse`, `prelude`, `generated`, `vendor` | they already mean something |
| not a prelude name (`print`, `args`, …) | an import of it would hide the prelude name (spec §3.9) |
| reserved first-party names: `finance`, `ml`, `web`, `gui`, `iot` (Fact: P6-1, spec §16.3) | published only by the Verse project |
| no name that differs from an existing one only by `_` or by one of `0`/`o`, `1`/`l` | confusable names are a supply-chain attack (registry rule, Decision) |
| first come, first served, never reused after deletion | a name always means the same package |

Fact: spec §16.3 states this rule (R8-1) and its example is `word_tools`.

The registry refuses a name that is taken, reserved, too close to an existing
one, or used before by another owner (wording as in
[16](16-diagnostics-catalog.md#v08xx-packages-and-api)):

```
error: the package name `finance` is not available
  --> verse.toml:2:8
   |
 2 | name = "finance"
   |        ^^^^^^^^^ reserved
   |
help: choose another name
explain: V0833 (run `verse explain V0833`)
```

```
error: the package name `co1ors` is not available
  --> verse.toml:2:8
   |
 2 | name = "co1ors"
   |        ^^^^^^^^ too close to `colors`
   |
help: choose another name
explain: V0833 (run `verse explain V0833`)
```

**Module paths inside a package.** A package `colors` with `src/colors.vs`
and `src/colors/term.vs` exports modules `colors` and `colors.term`
(spec §12.1 applied per package). Every module path a package exports starts
with the package name, so two packages can never export the same module, and
a module name tells the reader which package it came from (one obvious name,
06 §6.2).

**First-party packages** are ordinary packages: same registry, same lock, same
`verse.api` rules, no compiler privileges (Fact: 06 §6.2). They are published
by the Verse project's registry account; the registry refuses those names from
anyone else.

## 19.5 Versioning from `verse.api`

Every package follows semver, and the tool **computes the smallest correct
bump** by comparing the new `verse.api` with the one in the latest published
version of the same major (Decision). `verse publish` refuses a version
number below the required bump; it accepts a larger bump.

| Change in `verse.api` | Required bump | Why it breaks or not |
|-----------------------|---------------|----------------------|
| an exported item, module or field removed or renamed | major | callers stop compiling |
| a parameter added without a default, removed, reordered or renamed | major | calls by position or by name break |
| a parameter added *with* a default | major | the function's type `fn(…): R` changes, so uses as a function value break (spec §4.7). Proposal P19-6 discusses relaxing this |
| a parameter or return type changed, including a new or removed `Error` or `?` in the return type | major | |
| an inferred bound added or tightened (for example the inferred `Copy` bound of spec §4.2 rule 9) | major | callers that met the old bound may not meet the new one |
| a bound removed or loosened | minor | every old caller still compiles |
| a variant added to an exported variant type | major | exhaustive `match` in callers breaks (spec §7) |
| a field added to an exported record, with a default, after the existing fields | minor | named and positional construction (spec §4.3) still compile |
| a field added anywhere else or without a default | major | positional construction breaks |
| a field of an exported variant case added | major | positional variant patterns (`Circle(r)`, spec §7.3) break |
| a required method added to an exported trait | major | other packages' `impl`s break |
| a default method added to an exported trait | minor | assumption: the spec has no method-resolution ambiguity that this could create (to verify against §10) |
| a trait `impl` for an exported type added | minor | |
| a trait `impl` removed | major | |
| an exported item, module or field added | minor | |
| `[library] capabilities` grew | minor, and shown in `verse update` (§19.7) | not an API change, but a supply-chain fact users want to see |
| `verse` toolchain requirement raised | minor | |
| no `verse.api` change | patch | |

`0.y` versions: the rules shift one place (a "major" change bumps `y`, any
other change bumps the patch), as in Cargo (Decision).

**What it cannot see.** Behaviour changes with the same signatures are
invisible to a diff. The computed bump is a minimum; the author can always
choose a larger one. The registry shows both "declared" and "computed".

Message (wording as in [16-diagnostics-catalog.md](16-diagnostics-catalog.md#v08xx-packages-and-api)):

```
error: version 1.4.0 is too small for these changes; publish as 2.0.0
  --> verse.api (compared with colors 1.3.2)
   |
   | - export fn paint(text: Text, color: Color): Text
   | + export fn paint(text: Text, color: Color, bold: Bool = false): Text
   |   a parameter was added, so `paint` has a new function type
   |
help: set `version = "2.0.0"` in verse.toml
help: or keep `paint` as it was and add a new function for the new parameter
explain: V0803 (run `verse explain V0803`)
```

## 19.6 `verse.lock`

TOML, written only by the tool, sorted by name then version, one entry per
package, so diffs are small and merge conflicts are rare (Decision). It is
committed for applications (Fact: spec §16.3) and also for libraries,
where it is used for the library's own tests and examples and ignored by
dependents (Decision: dependents always resolve with MVS, §19.7).

```toml
# verse.lock — generated by `verse`; do not edit
version   = 1
toolchain = "0.1.3"
resolver  = "mvs-1"

[[package]]
name     = "colors"
version  = "1.4.0"
source   = "registry+https://registry.verse-lang.org"
checksum = "blake3:<64 hex digits>"      # of the published archive
api      = "blake3:<64 hex digits>"      # of its verse.api
capabilities = ["stdio"]                  # reachable from its exports, at publish time
dependencies = ["term 0.3.2"]

[[package]]
name     = "http_extra"
version  = "0.4.1"
source   = "git+https://example.com/http_extra.git?tag=v0.4.1#<commit id>"
checksum = "blake3:<64 hex digits>"      # of the source tree as packed
api      = "blake3:<64 hex digits>"

[[plugin]]
name     = "jsonschema"
version  = "1.2.0"
kind     = "generator"
checksum = "blake3:<64 hex digits>"      # of the Wasm component
grants   = { reads = ["schema/"], writes = ["generated/"], capabilities = [] }
```

| Field | Pinned because |
|-------|----------------|
| `toolchain` | the exact compiler version used; a different toolchain warns (V0823, below) and rebuilds, since cache keys include it |
| `resolver` | a future resolver change cannot silently change old locks |
| `checksum` | integrity: a mirror, a cache or a vendored copy is accepted only if its bytes hash to this |
| `api` | dependents are checked against `verse.api` only (spec §16.4); a changed api with the same version number is an error (V0824, below: registry corruption or a republish) |
| `capabilities` | `verse update` shows the difference before it is accepted |
| plugin `grants` | a plugin update that asks for more than the locked grant stops with a message instead of running ([18 §18.6](18-plugin-interface.md#186-capabilities), 07 §7.5) |

A lock made by another toolchain:

```
warning: this lock was made with 0.1.3, and you run 0.1.4
  --> verse.lock:3:13
   |
 3 | toolchain = "0.1.3"
   |             ^^^^^^^ the toolchain differs
   |
help: run `verse lock` to record 0.1.4; the build continues and does not reuse old caches
explain: V0823 (run `verse explain V0823`)
```

A package whose `api` changed for the same version:

```
error: `colors` 1.4.0 has a different API from the one in your lock
  --> verse.lock:11:12
   |
11 | api      = "blake3:3d71…"
   |            ^^^^^^^^^^^^^^ the lock says `blake3:3d71…`, the package says `blake3:b8e0…`
   |
help: `verse lock --refetch colors`
help: if it persists, the version was republished
explain: V0824 (run `verse explain V0824`)
```

## 19.7 Resolution: minimal version selection

**Decision: minimal version selection (MVS),** the algorithm of Go modules,
adapted to the spec's "one version of each package per major version"
(§16.3).

1. Each requirement `name "X.Y"` is a *minimum* `X.Y.0` within major `X`.
2. Walk the graph from the root. For each `(name, major)` keep the largest
   minimum seen. Requirements of a version are read from its index entry, so
   no archive is downloaded to resolve.
3. The selected version of each `(name, major)` is that largest minimum.
4. The root's lock entries act as extra minimums. This is how `verse update`
   works: it writes newer versions into the lock, and they win because they are
   larger.
5. Yanked versions are skipped when *adding* or *updating*, but a lock that
   names one still builds, with a warning (§19.8).

Different majors of one package may coexist in the graph (`colors 1.x` for one
dependency and `colors 2.x` for another). Each package's `import … from colors`
refers to the major it declared, and types from different majors are
different types. The compiler names both versions in any type mismatch (V0825), for example:

```
error: `Color` from `colors` 1.4.0 is not `Color` from `colors` 2.0.1
  --> src/report.vs:31:17
   |
31 |     paint("hi", charts.title_color())
   |                 ^^^^^^^^^^^^^^^^^^^^ two major versions of `colors` are in this build
   |
help: update `charts`, or pin one version of `colors`
explain: V0825 (run `verse explain V0825`)
```

**Why MVS, not a SAT solver (Cargo, npm, pip) or newest-compatible.**

| Property | MVS | newest-compatible with a solver |
|----------|-----|--------------------------------|
| Same manifests give the same graph without a lock | yes | no: the answer changes when anything is published |
| Cost | linear in the graph | NP-complete in general; needs heuristics and backtracking |
| Explaining the result | each version is "the largest minimum someone asked for", and `verse why pkg` prints who | the solver's search is hard to explain |
| Gets fixes automatically | no, only through `verse update` | yes, at the cost of surprise |
| Fits "one version per major" | directly | needs extra constraints |

The spec values reproducibility and explanation (§16.3, and Rafael's R7 for
tools that an LLM can drive). MVS gives both. The usual objection, that users
do not get bug fixes automatically, is answered by `verse update` showing
newer releases, and by the registry marking versions with security advisories
(P19-7).

**`verse update` output** (Decision):

```
updating colors   1.4.0 -> 1.5.1   api: compatible (minor)
updating term     0.3.2 -> 0.3.4   api: unchanged
warning: updating `colors` adds the `net` capability
   = note: 1.4.0 did not reach `net`; 1.5.1 does, from `colors.fetch_palette`, which your code does not call today
help: grant it in `[capabilities]`, or stay on 1.4.0
help: to refuse it, keep `[capabilities] grant` without `net`, or run `verse update --exclude colors`
explain: V0827 (run `verse explain V0827`)
```

## 19.8 Registry protocol

Decision: a static, cacheable protocol over HTTPS, so a mirror is a plain file
server and a registry outage does not stop builds that have a cache or
`vendor/`.

| Resource | Path | Content |
|----------|------|---------|
| configuration | `/config.json` | protocol version, download URL template, signing keys (key ids and public keys), mirrors |
| index entry | `/index/<first two letters>/<name>` | one JSON line per version: `version`, `checksum`, `api`, `dependencies` (name, requirement, registry), `verse` (toolchain range), `capabilities`, `targets`, `yanked`, `advisories` |
| index signature | `/index/<…>/<name>.sig` | signature over the index file |
| archive | `/archive/<name>/<version>.vpkg` | a `tar` of the `include` files, compressed with zstd; bytes are deterministic (sorted entries, zeroed times and owners) |
| `verse.api` | `/api/<name>/<version>.api` | the interface file alone, for `verse api diff` and for resolving without the archive |

**Integrity and signing.**

- The lock's `checksum` (blake3 of the archive) is the final authority: an
  archive from any source is accepted only if it matches.
- Index files are signed by the registry with Ed25519 keys listed in
  `/config.json`, with key rotation and expiry following The Update
  Framework's ideas (root, timestamp and targets roles). This protects first
  resolution, when there is no lock yet (Decision; the TUF role layout is an
  Assumption to be finalised in M5).
- Publishers authenticate with a token; first-party packages additionally
  require two maintainers to approve a release (Proposal P19-7).

**Yanking.** `verse yank colors@1.5.0` sets `yanked` in the index. Yanked
versions are never chosen by `add` or `update`. Existing locks keep building,
with a warning that names the reason the publisher gave. Archives are never
deleted, so old locks stay reproducible (Decision). Deletion exists only for
legal reasons and then breaks the affected locks loudly.

The warning for a locked yanked version:

```
warning: `colors` 1.4.0, in your lock, was yanked: data loss in `save`
  --> verse.lock:8:12
   |
 8 | version  = "1.4.0"
   |            ^^^^^^^
   |
help: `verse update colors` to move to 1.5.1
explain: V0826 (run `verse explain V0826`)
```

**Alternative registries and mirrors.** User or CI configuration (not the
project's `verse.toml`, so a cloned project cannot redirect downloads):

```toml
# ~/.config/verse/config.toml
[registries.corp]
index = "https://verse.corp.example/"

[mirror]
"https://registry.verse-lang.org" = "https://mirror.example/verse/"
```

A mirror needs no trust, since every byte is checked against the signed index
and the lock.

**Offline mode.** `--offline` uses the global package store
(`~/.cache/verse/pkg/<blake3>/`) and `vendor/`. If something is missing, the
error names it and the command that would fetch it.

```
error: `colors` 1.4.0 is not in the cache or in `vendor/`, and `--offline` is on
  --> verse.lock:8:12
   |
 8 | version  = "1.4.0"
   |            ^^^^^^^
   |
help: run `verse vendor` while connected, or remove `--offline`
explain: V0828 (run `verse explain V0828`)
```

**Vendoring.** `verse vendor` copies every locked package into
`vendor/<name>-<version>/` and adds `vendored = true` to the lock entries.
Builds prefer `vendor/` and verify each tree against the lock's `checksum`;
an edited vendored file is an error, not a silent patch (Decision: a local
change belongs in a `path` dependency, which the reader can see in
`verse.toml`).

```
error: `colors-1.4.0/src/colors.vs` in `vendor/` was edited
  --> vendor/colors-1.4.0/src/colors.vs
   |
   = note: it does not match the lock's checksum for `colors`
help: `verse vendor --restore colors`, or copy the package into your own source
explain: V0829 (run `verse explain V0829`)
```

## 19.9 Reproducible builds and caching

Fact (spec §16.3, 02 §2.7, 04 §4.4): the same inputs and `verse.lock` give
identical bytes. The inputs are exactly:

| Input | Hashed as |
|-------|-----------|
| source files of the root and every locked package | blake3 of each file |
| `verse.api` of every package | blake3 (the lock's `api`) |
| `verse.lock` | the resolved graph, not the file's formatting |
| toolchain | compiler version and the hash of the prebuilt runtime slices for the target |
| target file | blake3 of the target TOML (04 §4.3) |
| profile | the resolved profile table |
| flags that change output | `--no-heap`, `--size-report` (report only), `--deny` (check only) |
| plugin outputs | generated files are sources, so they are hashed as files (07 §7.3) |

Not inputs: time, paths, user name, environment variables, the order of
files on disk, the number of cores. Output contains no paths or timestamps
(04 §4.4).

**One cache, shared.** The package manager stores packages in the global
content-addressed store; the compiler's per-project `.verse/cache/` keys
modules and objects by `blake3(source, compiler version, flags, verse.api of
imports)` (02 §2.7); the shaking results of 06 §6.10 are keyed by
`blake3(root set, edge-list hashes, target file, profile)`. All three use the
same hash, so a package's parsed modules and per-instance edge lists can be
reused across projects that lock the same version (Assumption: a global
object cache is safe because every key includes the toolchain and target;
measured in M5).

`verse build --verify-reproducible` *new* builds twice in clean directories
and compares the bytes (for CI; P19-1).
When the bytes differ:

```
error: two clean builds of this project differ
  --> generated/item.vs
   |
   = note: `generated/item.vs` has different bytes
help: run `verse why generated/item.vs`; the usual cause is a plugin that is not deterministic
explain: V0832 (run `verse explain V0832`)
```

## 19.10 Workspaces

Several packages in one repository share one lock, one cache and one set of
plugins:

```toml
# verse.toml at the repository root
[workspace]
members = ["apps/server", "libs/shapes", "libs/money_utils"]

[plugins]                                 # only here, never in members
style = { version = "0.4", kind = "lint" }
```

| Rule | Why |
|------|-----|
| one `verse.lock` at the workspace root | one graph; MVS runs once over all members |
| members depend on each other with `{ path = "../shapes" }`; at publish time a path dependency must also name a version, which is checked to exist in the registry | a published package never depends on a path |
| the workspace root is the "root project" of 07 §7.5 | plugins stay opt-in at one place |
| `verse test`, `check`, `build` at the root run for every member; `-p name` selects one | |
| each member has its own `verse.api` and is published separately | the api is per package |
| a member's `[capabilities]` and `[targets]` are its own | a server and a firmware can share a library |

## 19.11 `verse why`

One command answers "why is this here?" for three kinds of argument, chosen
by its form. Items and file paths are spec §16 (R8-2); packages and
capabilities are Proposal P19-8:

| Argument | Example | Answer |
|----------|---------|--------|
| an item (dotted path) | `verse why std.net.get` | the edge chain from a root (06 §6.6) |
| a package | `verse why colors` | the dependency chain that requires the selected version, and the largest minimum that chose it (§19.7) |
| a file path (contains `/` or ends in `.vs`) | `verse why generated/item.vs` | the plugin that wrote it (07 §7.7) |
| a capability | `verse why net` | the chain to the first `sys.net` use (06 §6.6) |

```
$ verse why colors
colors 1.5.1 is selected because:
  your verse.lock asks for >= 1.5.1 (written by `verse update`)
  charts 2.0.0 asks for >= 1.4.0
  your verse.toml asks for >= 1.2.0

$ verse why std.net.get
std.net.get is included because:
  src/report.vs:22 calls colors.fetch_palette
  colors.fetch_palette calls std.net.get
```

The openers are fixed wording (review-6 R6-3), taken from
[`vocab/events.toml`](vocab/events.toml): `why.package` ("… is selected
because:"), `why.item` ("… is included because:"), `why.capability` ("this
program reaches `…` because:") and `why.file` ("… was written by …"). Each
answer is a **chain**. The MCP tools are `why_package`, `why_item` and
`why_capability` (17 §17.8).

Ambiguity is impossible by construction: package names never contain dots
(§19.4), items always do, capabilities are a closed list, and paths contain
`/` or `.vs`. A package named like a capability (`net`) is refused by the
registry (V0833, §19.4).

## 19.12 Supply-chain security

| Threat | Defence | Status |
|--------|---------|--------|
| code execution while building a dependency | no build scripts, no install hooks (spec §16.3); `[library.native]` is data (0.2) | Fact |
| a dependency adds a plugin | `[plugins]` accepted only at the root (07 §7.5); a dependency manifest with `[plugins]` is rejected | Fact / Decision |
| a plugin does more than allowed | Wasm sandbox, grants locked in `verse.lock`, budgets (07 §7.5, [18 §18.6](18-plugin-interface.md#186-capabilities), [§18.10](18-plugin-interface.md#1810-limits)) | Fact |
| a dependency starts using the network | capabilities in the index and lock; `verse update` shows the change; `[capabilities] grant` fails the build (P19-3) | Proposal |
| a tampered archive or mirror | blake3 checksums in the lock; signed index | Decision |
| typosquatting | name rules of §19.4 | Decision |
| a republished version | archives are immutable; a different `api` or `checksum` for the same version is an error | Decision |
| an abandoned package taken over | names are never reused; ownership transfer is shown in `verse update` | Decision |

Compile-time code execution does not exist in Verse at all (no macros, no
compile-time evaluation, Q10), so reading and type-checking a dependency is
safe by construction. Running its tests is not, and `verse test` runs only
the root's tests unless `--deps` is given.

**Messages.** Codes and wording are those of
[16-diagnostics-catalog.md](16-diagnostics-catalog.md#v08xx-packages-and-api).
Each block shows its code on the last line, as in
[16 §16.4.1](16-diagnostics-catalog.md#1641-how-a-code-is-shown).

```
error: `charts` lists plugins, but only the root project may
  --> ~/.cache/verse/pkg/…/charts-2.0.0/verse.toml:14:1
   |
14 | [plugins]
   | ^^^^^^^^^ a dependency cannot run code while your project builds
   |
help: add the plugin to your own verse.toml under [plugins]
explain: V0805 (run `verse explain V0805`)
```

```
error: `colors` 1.5.1 does not match its checksum
  --> verse.lock:9:12
   |
 9 | checksum = "blake3:9f2c…"
   |            ^^^^^^^^^^^^^^ the archive hashes to `blake3:41ab…`
   |
help: verse lock --refetch colors
help: a mirror is not trusted, so the lock is still correct
explain: V0806 (run `verse explain V0806`)
```

```
error: this program reaches the `net` capability, which verse.toml does not grant
  --> src/report.vs:22:5
   |
22 |     palette = fetch_palette("warm")?
   |               ^^^^^^^^^^^^^ `colors.fetch_palette` calls `std.net.get`
   |
help: add "net" to [capabilities] grant
help: or see the whole chain: verse why net
explain: V0901 (run `verse explain V0901`)
```

```
error: version 2.3 of `shapes` does not exist
  --> verse.toml:8:10
   |
 8 | shapes = "2.3"
   |          ^^^^^ the newest release is 2.1.4
   |
help: shapes = "2.1"
explain: V0804 (run `verse explain V0804`)
```

**Where each code from 16 appears here**

| Code | Message (16) | Where here |
|------|--------------|------------|
| V0801 | `{key}` is not a key of `[{section}]` | §19.3 (unknown keys are errors) |
| V0802 | the package name `{name}` cannot be imported | §19.4 |
| V0803 | version {have} is too small for these changes; publish as {need} | §19.5 (message) |
| V0804 | version {req} of `{pkg}` does not exist | §19.12 (message), §19.7 |
| V0805 | `{pkg}` lists plugins, but only the root project may | §19.12 (message), §19.3 `[plugins]` |
| V0806 | `{pkg}` {ver} does not match its checksum | §19.12 (message), §19.6 `checksum` |
| V0807 | `verse.lock` is out of date | §19.2 (`--locked`), §19.6 |
| V0808 | `{pkg}` needs {T}, which `{other}` does not provide | §19.7 (two majors) |
| V0809 | a path dependency cannot be published | §19.10 |
| V0810, V0811 | `verse.api` changed / differs from the source | §19.5, spec §16.4 |
| V0812 | `{name}` is not a valid license | §19.3 `license` |
| V0813 | a yanked version cannot be chosen | §19.7 step 5, §19.8 |
| V0814 | plugin `{name}` is a `{k1}`, but verse.toml says `{k2}` | §19.3 `[plugins]` (message) |
| V0815 | plugin `{name}` asks for more than verse.toml grants | §19.3 `[plugins]` (message) |
| V0823 | this lock was made with {old}, and you run {new} | §19.6 (message) |
| V0824 | `{pkg}` {ver} has a different API from the one in your lock | §19.6 (message) |
| V0825 | `{T}` from `{pkg}` {a} is not `{T}` from `{pkg}` {b} | §19.7 (message) |
| V0826 | `{pkg}` {ver}, in your lock, was yanked: {reason} | §19.8 Yanking (message), §19.7 step 5 |
| V0827 | updating `{pkg}` adds the `{cap}` capability | §19.7 (`verse update` output) |
| V0828 | `{pkg}` {ver} is not in the cache or in `vendor/`, and `--offline` is on | §19.8 Offline mode (message) |
| V0829 | `{file}` in `vendor/` was edited | §19.8 Vendoring (message) |
| V0830 | `branch` is not accepted | §19.3 `[dependencies]` (message) |
| V0831 | `{req}` has no upper bound | §19.3 `[dependencies]` (message) |
| V0832 | two clean builds of this project differ | §19.9 (message) |
| V0833 | the package name `{name}` is not available | §19.4 (messages) |
| V0901 | this program reaches the `{cap}` capability, which verse.toml does not grant | §19.12 (message), §19.3 `[capabilities]` |
| V0904 | `{pkg}` reaches `{cap}`, which it does not declare | §19.3 `[library] capabilities` |
| V0905 | this package uses {n} KB, but its size budget is {m} KB | §19.3 `size_budget` |
| V0906 | `{module}` has different public items on `{target}` | §19.3 `[targets.<t>.files]` |
| V0907 | the file `{f}` for target `{t}` does not exist | §19.3 `[targets.<t>.files]` |

## 19.13 Implementation notes

- Crate: `verse-pkg` (02 §2.9) owns `verse.toml`, `verse.lock`, the registry
  client and `verse.api` comparison. The api diff reuses the compiler's
  signature representation, so "compatible" means exactly what the type
  checker means.
- Resolution never downloads archives; it reads index entries (and `.api`
  files for `verse api diff`).
- Network access is concurrent and limited to resolution and fetching; build,
  check and run never touch the network once the store is filled.
- Targets (to measure in M5, none measured yet): `verse lock` on a warm cache
  for a graph of 200 packages under 100 ms; `verse run` adds no measurable
  time when the lock is current (it only compares the hash of `verse.toml`).

## 19.14 Proposals and rulings

All agreed by Michelangelo in `reviews/review-6.md` (R6-3).

| # | Proposal | Touches | Ruling |
|---|----------|---------|----------------|
| P19-1 | New commands: `verse lock` (`--check`), `verse vendor`, `verse yank`, `verse api diff`, `build --verify-reproducible`; global `--offline`, `--locked`, `--frozen`, `--json` | spec §16 tool table | agreed |
| P19-2 | The `verse.toml` reference of §19.3: `[package]` keys (`authors`, `repository`, `readme`, `include`), dependency forms (`registry`, `path`, git `tag`/`rev`, no `branch`, no unbounded ranges), `[plugins]`, `[targets]`, `[profile.*]`, `[run]`, `[build]`, `[test]`, `[workspace]`; unknown keys are errors | spec §16.3 | agreed |
| P19-3 | `[capabilities] grant` for applications: the build fails if reachability finds an ungranted capability | spec §16.3, 06 §6.8 | agreed; it uses the word *grant* as ruled in `reviews/extern-review.md` |
| P19-4 | `[targets.<t>] heap` replaces 05 §5.7's `[embedded] heap` | 05 (Bob) | agreed, per target |
| P19-5 | No `[features]` in 0.1; if needed later, only named sets of optional dependencies, never source conditions | spec §16.3, P6-7 | agreed |
| P19-6 | Adding a defaulted parameter is a major change, because it changes the function type. Alternative: minor, and function-value uses of library functions get a lint | versioning, spec §4.7 | agreed: **major** (a changed function type is a changed promise) |
| P19-7 | Security advisories in the index, shown by `verse update`; two-maintainer approval for first-party releases | registry | agreed |
| P19-8 | `verse why` also accepts a package name or a capability, distinguished by form (items and paths are already spec §16, R8-2) | spec §16 | agreed; wording in §19.11 (two openers, answer is a chain) |
| P19-9 | Package names are `[a-z][a-z0-9_]*` | spec §16.3 | agreed and applied (R8-1) |
| P19-10 | Minimal version selection, with lock entries as extra minimums | spec §16.3 ("a single version per major") | agreed |
| P19-11 | Libraries commit `verse.lock` too (used for their own tests, ignored by dependents) | spec §16.3 says "committed for applications" | agreed |
| P19-12 | MCP tools for the package manager (`lock`, `why` for packages, `api_diff`, `update` preview) with schemas in 17, and their diagnostic codes in 16 | 16, 17 (Michelangelo) | agreed; 17 §17.8 and the V08xx codes of 16 |

## 19.15 Conflicts with other documents

1. *Resolved (R8-1):* spec §16.3's example is now `word_tools`, with the
   identifier rule for package names.
2. *Resolved (R8-2):* spec §16 has one `verse why <item-or-path>`, matching
   [07 §7.7](07-plugins.md#77-discoverability).
3. **[05 §5.7](05-runtime.md#57-embedded-profile)** puts the heap size in an
   `[embedded]` section; this document puts it per target (P19-4).
4. *Resolved by Michelangelo (review-6 R6-1) in [07 §7.4](07-plugins.md#74-wasm-implemented-packages-are-ordinary-packages):*
   a Wasm-implemented package is an ordinary dependency, not a plugin, so it
   goes in `[dependencies]` and any package may depend on it; 06 §6.2 is
   updated.
5. *Resolved (R8-3):* spec §16's `verse build` row now says it builds for the
   selected profile and target (dev, release, WebAssembly, bytecode).
