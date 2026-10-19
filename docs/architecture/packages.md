# 21. Packages

Status: **draft 1**. This document describes the package model: what a
package is, how it is named and found, how versions are chosen, locked and
published, and how the registry keeps all of it trustworthy. The commands,
their flags and every `verse.toml` key are in the reference,
[19](../reference/cli-and-manifest.md). Labels follow the
[README](README.md) conventions: **Fact** (stated by the spec or a ruled
document), **Decision**, **Assumption**, **Target**, **Proposal** (rulings in
[19 §19.6](../reference/cli-and-manifest.md#196-proposals-and-rulings)).

Nothing here changes what a Verse program means. The spec
([§12](../language/12-modules.md), [§16.3, §16.4](../language/16-tooling.md#163-packages))
wins wherever this document disagrees with it.

## 21.1 What a package is

A **package** is Verse source described by a `verse.toml`: a folder with
`src/`, published under one name in many versions. A project adds it with
`verse add`, and its code reaches a program through `import`. The compiler
builds a package from source, together with the program that imports it.

| | Package | Plugin |
|-|---------|--------|
| What it is | Verse source with a `verse.toml` | a compiled library with a C interface ([07](plugins.md)) |
| Comes from | a registry, a git repository or a path | the system, or a folder of the project |
| Declared in | `[dependencies]` | `[plugins.<name>]` |
| Used from source with | `import … from name` | `extern name` (spec §12.6) |
| Checked by Verse | fully: types, ownership, capabilities | only the Verse side of each call |
| Runs code to build | never | only the root project's `build` command |

A package shares Verse code. A plugin reaches code written in another
language. A package may declare plugins of its own, but only system
libraries by `lib` (§21.13).

The standard library is not a package. It ships with the toolchain, is
versioned with it, and is never in `verse.lock`
([06 §6.2](stdlib-and-shaking.md#62-library-tiers)).

**First-party packages** are ordinary packages: same registry, same lock, same
`verse.api` rules, no compiler privileges (Fact: 06 §6.2). They are published
by the Verse project's registry account; the registry refuses those names from
anyone else.

A package that declares a `[tool]` section ships a command. A **generator**
is such a package: `verse generate` runs its command to write ordinary `.vs`
files into `generated/` (spec [§16.6](../language/16-tooling.md#166-generators)).
A generator is usually a dev-dependency, and its command is never part of a
program that imports the package.

## 21.2 Principles

1. **Three files, three jobs.** `verse.toml` is written by people, `verse.lock`
   and `verse.api` are written by the tool and committed. A reviewer reads a
   diff of any of them and knows what changed (Fact: spec §16.3, §16.4).
2. **No code runs to build a dependency.** No build scripts, no install
   hooks, no plugin `build` commands from dependencies (Fact: spec §16.3, 07 §7.2).
3. **Minimal version selection** picks versions, so the same manifests give
   the same graph without a solver and without the lock (Decision, §21.8).
4. **The required version bump is computed** from the `verse.api` diff, and
   `verse publish` refuses a version that understates it (Decision, §21.6).
5. **Everything is content-addressed** with blake3, the same hash family the
   compiler and the shaking cache use (Fact: 02 §2.7, 06 §6.10).

## 21.3 Identity and naming

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

Fact: spec §16.3 states this rule and its example is `word_tools`.

The registry refuses a name that is taken, reserved, too close to an existing
one, or used before by another owner (wording as in
[16](../reference/diagnostics-catalog.md#v08xx-packages-and-api)):

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

## 21.4 How `import` finds a package

The first segment of a module path says where the module lives. In
`import red, bold from colors.term`, `colors` is exactly one of these:

| First segment | Found in | Version |
|---------------|----------|---------|
| a top-level module of this package | `src/colors.vs` or the folder `src/colors/` | this package's |
| a dependency | `[dependencies]`, and also `[dev-dependencies]` in `test` blocks and doc tests | the one in `verse.lock` |
| the standard library | the toolchain | the toolchain's |

The three sets of names never overlap. Standard-library names are reserved
(§21.3), and a dependency that has the name of a top-level module of the
package is an error (V0835). So the order of the rows never decides anything,
and a reader who sees `colors` knows where it comes from by looking at
`src/` and `verse.toml`. A first segment that is none of the three is an
error that suggests `verse add` (V0836).

An import names a package, never a version, a URL or a path. Where a package
comes from is written once, in `verse.toml`, and which version is used is
written once, in `verse.lock`.

**Module paths inside a package.** A package `colors` with `src/colors.vs`
and `src/colors/term.vs` exports modules `colors` and `colors.term`
(spec §12.1 applied per package). Every module path a package exports starts
with the package name, so two packages can never export the same module, and
a module name tells the reader which package it came from (one obvious name,
06 §6.2).

## 21.5 Adding, updating and publishing

A dependency moves through a few commands. Each one changes a file the
author commits, so a reviewer sees every step as a diff. Flags and the files
each command reads and writes are in
[19 §19.2](../reference/cli-and-manifest.md#192-commands).

| Command | What it changes |
|---------|-----------------|
| `verse add colors` | a line in `[dependencies]` at the newest release, and the lock |
| `verse remove colors` | the line, and the lock drops what nothing else needs |
| `verse update` | locked versions move up within `verse.toml`'s requirements (§21.8), and capability changes are shown before they are accepted |
| `verse api update` | `verse.api`, the package's promise (§21.6) |
| `verse publish` | a new version in the registry, at least as large as the `verse.api` diff requires (§21.6) |
| `verse yank colors@1.5.0` | marks a published version so `add` and `update` skip it (§21.9) |
| `verse vendor` | copies every locked package into `vendor/`, for builds with no network (§21.9) |

`verse build`, `check` and `test`, and running a file, read the lock. They
resolve only when it is missing or stale, and say so.

## 21.6 Versioning from `verse.api`

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
| `[library] capabilities` grew | minor, and shown in `verse update` (§21.8) | not an API change, but a supply-chain fact users want to see |
| `verse` toolchain requirement raised | minor | |
| no `verse.api` change | patch | |

`0.y` versions: the rules shift one place (a "major" change bumps `y`, any
other change bumps the patch), as in Cargo (Decision).

**What it cannot see.** Behaviour changes with the same signatures are
invisible to a diff. The computed bump is a minimum; the author can always
choose a larger one. The registry shows both "declared" and "computed".

Message (wording as in [16-diagnostics-catalog.md](../reference/diagnostics-catalog.md#v08xx-packages-and-api)):

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

## 21.7 `verse.lock`

TOML, written only by the tool, sorted by name then version, one entry per
package, so diffs are small and merge conflicts are rare (Decision). It is
committed for applications (Fact: spec §16.3) and also for libraries,
where it is used for the library's own tests and examples and ignored by
dependents (Decision: dependents always resolve with MVS, §21.8).

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
```

| Field | Pinned because |
|-------|----------------|
| `toolchain` | the exact compiler version used; a different toolchain warns (V0823, below) and rebuilds, since cache keys include it |
| `resolver` | a future resolver change cannot silently change old locks |
| `checksum` | integrity: a mirror, a cache or a vendored copy is accepted only if its bytes hash to this |
| `api` | dependents are checked against `verse.api` only (spec §16.4); a changed api with the same version number is an error (V0824, below: registry corruption or a republish) |
| `capabilities` | `verse update` shows the difference before it is accepted |

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

## 21.8 Resolution: minimal version selection

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
   names one still builds, with a warning (§21.9).

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

The spec values reproducibility and explanation (§16.3, and requirement R7 for
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

## 21.9 Registry protocol

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

## 21.10 Reproducible builds and caching

Fact (spec §16.3, 02 §2.7, 04 §4.4): the same inputs and `verse.lock` give
identical bytes. The inputs are exactly:

| Input | Hashed as |
|-------|-----------|
| source files of the root and every locked package | blake3 of each file |
| `verse.api` of every package | blake3 (the lock's `api`) |
| `verse.lock` | the resolved graph, not the file's formatting |
| toolchain | compiler version and the hash of the prebuilt runtime slices for the target |
| target file | blake3 of the resolved target file: the toolchain's `targets/<name>.toml`, the project's `targets/<name>.target`, or `verse.target` ([15 §15.9.1](target-and-abi.md#1591-files-and-lookup)) |
| profile | the resolved profile table |
| flags that change output | `--no-heap`, `--size-report` (report only), `--deny` (check only) |
| generated files | they are sources, so they are hashed as files (spec §16.6) |
| plugin libraries | blake3 of each linked library file; a system library is outside the build's control, so builds are reproducible only for the same system library (07 §7.6) |

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
  --> generated/items/item.vs
   |
   = note: `generated/items/item.vs` has different bytes
help: run `verse why generated/items/item.vs`; the usual cause is a generator that is not deterministic
explain: V0832 (run `verse explain V0832`)
```

## 21.11 Features

**Decision: there is no `[features]` section in 0.1.** Cargo-style features
exist mainly to avoid compiling and linking unused code. In Verse that is
already free: an import costs nothing and only reached code is built
([06](stdlib-and-shaking.md)). The remaining uses are covered by data
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

## 21.12 Workspaces

Several packages in one repository share one lock and one cache:

```toml
# verse.toml at the repository root
[workspace]
members = ["apps/server", "libs/shapes", "libs/money_utils"]
```

| Rule | Why |
|------|-----|
| one `verse.lock` at the workspace root | one graph; MVS runs once over all members |
| members depend on each other with `{ path = "../shapes" }`; at publish time a path dependency must also name a version, which is checked to exist in the registry | a published package never depends on a path |
| the workspace root is the root project of 07 §7.2 | a member may use `path` and `build` plugins, as the root may |
| `verse test`, `check`, `build` at the root run for every member; `-p name` selects one | |
| each member has its own `verse.api` and is published separately | the api is per package |
| a member's `[capabilities]` and `[targets]` are its own | a server and a firmware can share a library |

## 21.13 Supply-chain security

| Threat | Defence | Status |
|--------|---------|--------|
| code execution while building a dependency | no build scripts, no install hooks (spec §16.3); a dependency's plugins may name only a system library (`lib`), never `path` or `build` (V0834) | Fact / Decision |
| a dependency calls native code | its `[plugins]` are listed by `verse add` and `verse update`; Verse cannot check foreign code, so adding the package is trusting the library (07 §7.1) | Decision |
| a generator does harm | it runs only on `verse generate`, never on build, check, run or test (spec §16.6) | Decision |
| a dependency starts using the network | capabilities in the index and lock; `verse update` shows the change; `[capabilities] grant` fails the build (P19-3) | Proposal |
| a tampered archive or mirror | blake3 checksums in the lock; signed index | Decision |
| typosquatting | name rules of §21.3 | Decision |
| a republished version | archives are immutable; a different `api` or `checksum` for the same version is an error | Decision |
| an abandoned package taken over | names are never reused; ownership transfer is shown in `verse update` | Decision |

Compile-time code execution does not exist in Verse at all (no macros, no
compile-time evaluation), so reading and type-checking a dependency is
safe by construction. Running its tests is not, and `verse test` runs only
the root's tests unless `--deps` is given.

**Messages.** Codes and wording are those of
[16-diagnostics-catalog.md](../reference/diagnostics-catalog.md#v08xx-packages-and-api).
Each block shows its code on the last line, as in
[16 §16.4](../reference/diagnostics-catalog.md#164-how-a-code-is-shown).

```
error: `charts` declares a plugin with `build`, which only the root project may use
  --> ~/.cache/verse/pkg/…/charts-2.0.0/verse.toml:16:1
   |
16 | build = "make"
   | ^^^^^^^^^^^^^^ a dependency cannot build or ship native code
   |
help: declare the plugin in your own verse.toml
explain: V0834 (run `verse explain V0834`)
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

## 21.14 Implementation notes

- Crate: `verse-pkg` (02 §2.9) owns `verse.toml`, `verse.lock`, the registry
  client and `verse.api` comparison. The api diff reuses the compiler's
  signature representation, so "compatible" means exactly what the type
  checker means.
- Resolution never downloads archives; it reads index entries (and `.api`
  files for `verse api diff`).
- Network access is concurrent and limited to resolution and fetching; build,
  check and run never touch the network once the store is filled.
- Targets (to measure in M5, none measured yet): `verse lock` on a warm cache
  for a graph of 200 packages under 100 ms; running a file adds no measurable
  time when the lock is current (it only compares the hash of `verse.toml`).
