# Packages and the CLI

The `verse` binary and the package manager: `verse-cli` and `verse-pkg`.
Nothing is implemented yet; the docs are the design.

## Read first

1. [spec §16 Tooling](../../../../docs/language/16-tooling.md#16-tooling):
   the command table and the dispatch rule in short.
2. [spec §16.3 Packages](../../../../docs/language/16-tooling.md#163-packages)
   and [§16.4 `verse.api`](../../../../docs/language/16-tooling.md#164-interface-files-verseapi).
3. [spec §12.2 Importing](../../../../docs/language/12-modules.md#122-importing)
   and [exported signatures and `verse.api`](../../../../docs/language/12-modules.md#exported-signatures-and-verseapi).
4. [21 Packages](../../../../docs/architecture/packages.md): the model, from
   [§21.1](../../../../docs/architecture/packages.md#211-what-a-package-is) to
   [§21.14](../../../../docs/architecture/packages.md#2114-implementation-notes).
5. [19 CLI, manifests and package manager](../../../../docs/reference/cli-and-manifest.md):
   every command ([§19.2](../../../../docs/reference/cli-and-manifest.md#192-commands)),
   every `verse.toml` key ([§19.3](../../../../docs/reference/cli-and-manifest.md#193-versetoml)),
   `verse why` ([§19.4](../../../../docs/reference/cli-and-manifest.md#194-verse-why)),
   and where each code appears ([§19.5](../../../../docs/reference/cli-and-manifest.md#195-where-each-code-appears)).
6. [16 V08xx](../../../../docs/reference/diagnostics-catalog.md#v08xx-packages-and-api)
   and [V11xx](../../../../docs/reference/diagnostics-catalog.md#v11xx-command-line):
   the codes these crates report.

## What it does

The crates sit near the end of the crate order of
[02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout).

| Crate | Owns | Source |
|-------|------|--------|
| `verse-pkg` | `verse.toml`, `verse.lock`, the registry client, the `verse.api` comparison | [21 §21.14](../../../../docs/architecture/packages.md#2114-implementation-notes) |
| `verse-cli` | the `verse` binary: dispatch, flags, `--json` output, exit statuses | [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout), [19 §19.2](../../../../docs/reference/cli-and-manifest.md#192-commands) |

Codes by crate ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)):

- `verse-pkg` reports V08xx, plus V0906 and V0907 (target variants in the manifest).
- `verse-cli` reports V11xx, plus V0911 to V0915 (target files).

Three files, three jobs
([21 §21.2](../../../../docs/architecture/packages.md#212-principles)):

| File | Written by | Holds |
|------|-----------|-------|
| `verse.toml` | people | what the author asks for |
| `verse.lock` | the tool, committed | what was chosen |
| `verse.api` | the tool, committed | what a package promises |

Inputs: the manifest, the lock, registry index entries and `.api` files.
Outputs: the lock, the resolved graph, the required version bump, a
published archive, and the command's result as text or `--json` records.

A package is Verse source with a `verse.toml`. A plugin is a compiled C
library declared under `[plugins]`. The table that tells them apart is
[21 §21.1](../../../../docs/architecture/packages.md#211-what-a-package-is).
The standard library is not a package and is never in `verse.lock`.

## How it works

**Naming.** A package name is its root module name, `[a-z][a-z0-9_]*`, at
most 64 characters. It is never a keyword, a standard library module, a
prelude name, a capability name or a reserved first-party name. Confusable
names are refused. Names are never reused
([21 §21.3](../../../../docs/architecture/packages.md#213-identity-and-naming)).

**Finding an import.** The first segment of a module path is a module of the
package, a dependency, or a standard library module. The three sets never
overlap, so row order never decides anything (V0835, V0836, V0837). An import
names a package, never a version, URL or path
([21 §21.4](../../../../docs/architecture/packages.md#214-how-import-finds-a-package)).

**Version bumps from `verse.api`.** `verse publish` compares the new
`verse.api` with the latest published one of the same major. It refuses a
version below the computed bump (V0803) and accepts a larger one. The
table of changes and bumps is
[21 §21.6](../../../../docs/architecture/packages.md#216-versioning-from-verseapi).
The diff reuses the compiler's signature representation, so "compatible"
means what the type checker means
([21 §21.14](../../../../docs/architecture/packages.md#2114-implementation-notes)).

**Resolution: minimal version selection.** Each requirement is a minimum
within its major. For each `(name, major)` keep the largest minimum. Lock
entries act as extra minimums, which is how `verse update` works. Yanked
versions are skipped when adding or updating
([21 §21.8](../../../../docs/architecture/packages.md#218-resolution-minimal-version-selection)).
Why MVS: the same manifests give the same graph without a lock, the cost is
linear, and `verse why pkg` can explain every choice. Different majors may
coexist; their types are different types (V0825).

**The lock.** TOML, sorted by name then version, one entry per package.
It pins `toolchain`, `resolver`, `checksum`, `api` and `capabilities`. The
reason for each field is in
[21 §21.7](../../../../docs/architecture/packages.md#217-verselock).

**The registry.** A static protocol over HTTPS: `config.json`, index
entries, index signatures, archives and `.api` files. The lock's blake3
`checksum` is the final authority, so a mirror needs no trust. Index files
are signed with Ed25519. Yanked archives are never deleted. `verse vendor`
copies locked packages into `vendor/` and verifies them against the lock
([21 §21.9](../../../../docs/architecture/packages.md#219-registry-protocol)).

**Reproducible builds.** The hashed inputs are listed in
[21 §21.10](../../../../docs/architecture/packages.md#2110-reproducible-builds-and-caching).
Time, paths, user name, environment variables, file order and core count are
not inputs. The package store, `.verse/cache/` and the shaking cache all use
blake3 ([02 §2.7](../../../../docs/architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)).

**Workspaces.** One lock and one cache at the root. Members depend on each
other by `path`, and a published member must also name a version.
`verse test`, `check` and `build` at the root run for every member; `-p name`
selects one
([21 §21.12](../../../../docs/architecture/packages.md#2112-workspaces)).

**Dispatch.** The first argument that is not a flag decides. A word ending
in `.vs` or containing `/` is a path to run. `.` is the current folder. Any
other word is a command. Flags for `verse` come before the path. Every
argument after the path goes to the program as `args`, with no `--`
([19 §19.2.1](../../../../docs/reference/cli-and-manifest.md#1921-running-a-file)).

**Lock reads.** Running a file, `build`, `check` and `test` read the lock.
They resolve only when it is missing or stale, and say so
([21 §21.5](../../../../docs/architecture/packages.md#215-adding-updating-and-publishing)).
`--offline`, `--locked` and `--frozen` are accepted by every command that may
touch the network. CI uses `--frozen`
([19 §19.2](../../../../docs/reference/cli-and-manifest.md#192-commands)).

**`verse why`.** One command, told apart by the form of its argument: an
item (dotted), a package, a file path, or a capability. Each answer is a
chain. Its openers come from `events.toml`
([19 §19.4](../../../../docs/reference/cli-and-manifest.md#194-verse-why)).

## Rules that must hold

- No code runs to build a dependency. No build scripts and no install hooks.
  A dependency's plugins may name only `lib`, never `path` or `build` (V0834)
  ([07 §7.2](../../../../docs/architecture/plugins.md#72-declaring-a-plugin),
  [21 §21.13](../../../../docs/architecture/packages.md#2113-supply-chain-security)).
- Only `verse generate` runs a generator. `build`, `check`, `test` and running a
  file never do ([spec §16.6](../../../../docs/language/16-tooling.md#166-generators)).
- `verse test` never runs a dependency's tests unless `--deps` is given
  ([21 §21.13](../../../../docs/architecture/packages.md#2113-supply-chain-security)).
- Unknown `verse.toml` sections and keys are errors (V0801)
  ([19 §19.3](../../../../docs/reference/cli-and-manifest.md#193-versetoml)).
- Every requirement has an upper bound at the next major (V0831). `branch` is
  refused (V0830). There is no `[features]` section
  ([21 §21.11](../../../../docs/architecture/packages.md#2111-features)).
- A changed `api` or `checksum` for the same version is an error (V0824, V0806).
  An edited vendored file is an error, never a silent patch (V0829).
- Resolution never downloads archives. Build, check and run never touch the
  network once the store is filled
  ([21 §21.14](../../../../docs/architecture/packages.md#2114-implementation-notes)).
- Alternative registries and mirrors live in user or CI configuration, never in
  `verse.toml`, so a cloned project cannot redirect downloads
  ([21 §21.9](../../../../docs/architecture/packages.md#219-registry-protocol)).
- Output contains no paths or timestamps, and the same inputs give the same
  bytes ([21 §21.10](../../../../docs/architecture/packages.md#2110-reproducible-builds-and-caching)).
- A word that is not a command is an error (V1101, status 2), never a guess,
  even when a file of that name exists
  ([19 §19.2.1](../../../../docs/reference/cli-and-manifest.md#1921-running-a-file)).
- Every command takes `--json` and prints records with the same facts as the
  text. Errors carry their code
  ([19 §19.2](../../../../docs/reference/cli-and-manifest.md#192-commands)).
- Exit statuses are the table of
  [16 §16.6.1](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses),
  which owns them. A build verifier failure is status 3, not 1.
- Compiler flags change what the compiler reports or how it optimizes, never
  what a program means. No profile key turns off overflow or bounds checks
  ([19 §19.2.3](../../../../docs/reference/cli-and-manifest.md#1923-building-and-checking),
  [19 §19.3](../../../../docs/reference/cli-and-manifest.md#193-versetoml)).

Not decided in the docs:

- The exit status of a package-manager error is an assumption: non-zero, no
  number fixed ([19 §19.2.2](../../../../docs/reference/cli-and-manifest.md#1922-exit-statuses)).
- The TUF role layout of the signed index is finalised in M5
  ([21 §21.9](../../../../docs/architecture/packages.md#219-registry-protocol)).
- A default trait method counting as a minor bump is an assumption, to verify
  against spec §10 ([21 §21.6](../../../../docs/architecture/packages.md#216-versioning-from-verseapi)).
- A global object cache shared across projects is an assumption, measured in M5
  ([21 §21.10](../../../../docs/architecture/packages.md#2110-reproducible-builds-and-caching)).
- Speed targets for `verse lock` are not measured
  ([21 §21.14](../../../../docs/architecture/packages.md#2114-implementation-notes)).

## What depends on it

Upstream inputs it trusts:

- The compiler's signature representation, for the `verse.api` diff
  ([21 §21.14](../../../../docs/architecture/packages.md#2114-implementation-notes)).
- Reachability from exported items, for `[library] capabilities` and the
  lock's `capabilities` field
  ([19 §19.3](../../../../docs/reference/cli-and-manifest.md#193-versetoml),
  [21 §21.7](../../../../docs/architecture/packages.md#217-verselock)).
- The size report, for `size_budget` at publish (V0905).
- Target files, resolved by name
  ([15 §15.9.1](../../../../docs/architecture/target-and-abi.md#1591-files-and-lookup)).

Downstream:

- Type inference checks importers against `verse.api` only, so packages
  compile independently
  ([02 §2.5](../../../../docs/architecture/compiler-architecture.md#25-type-inference)).
  A change to the `verse.api` format reaches every dependent and the bump
  table.
- Every cache key includes the toolchain, the lock graph, the target file and
  the profile. A change to an input list changes every key
  ([21 §21.10](../../../../docs/architecture/packages.md#2110-reproducible-builds-and-caching)).
- Scripts and CI read exit statuses and `--json` records. Changing either is
  a breaking change for them.
- `verse difftest`, `verse bench`, `verse dump mir` and `verse debug` are
  commands of this binary that drive other crates
  ([19 §19.2](../../../../docs/reference/cli-and-manifest.md#192-commands)).

The crate order is strictly downward. A back edge is a build error in CI
([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).

## After a change

1. Add tests by layer
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - Unit tests for resolution, the bump computation and manifest parsing.
   - Golden tests for each new or changed diagnostic. Each code needs three
     tests: the exact text, the `help:` edit when it is machine-applicable,
     and `verse explain` text
     ([16 §16.1](../../../../docs/reference/diagnostics-catalog.md#161-rules) rule 9).
   - Conformance tests when a rule of spec §12.2, §16.3 or §16.4 changes
     ([10 §10.2](../../../../docs/architecture/testing-and-verification.md#102-specification-conformance)).
   - Must-fail tests with expected codes for manifests and imports the tool
     must reject.
   - `verse build --verify-reproducible` in CI for anything that touches
     build inputs.
2. Update the docs in the same pull request:
   - A command, flag or key goes in [19](../../../../docs/reference/cli-and-manifest.md).
     If users see it, also update the spec tool table in §16.
   - Package behavior goes in [21](../../../../docs/architecture/packages.md).
   - A new code goes in the [catalog](../../../../docs/reference/diagnostics-catalog.md),
     in V08xx for `verse-pkg` or V11xx for `verse-cli`. Codes are never reused.
     Also update the table of [19 §19.5](../../../../docs/reference/cli-and-manifest.md#195-where-each-code-appears).
   - `verse why` wording lives in `events.toml`.
3. Commit as `<type>(packages): …` or `<type>(cli): …`. A change to the design
   is `spec(packages)` or `spec(cli)`.
4. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
