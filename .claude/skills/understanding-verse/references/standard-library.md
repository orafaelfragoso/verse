# Standard library and reachability

The prelude, the standard library modules in `std/`, the first-party packages,
and the reachability and capability analysis that keeps a build to what the
program uses (`verse-reach`). Nothing is implemented or measured yet; the docs
are the design.

## Read first

1. [spec §15 Standard library overview](../../../../docs/language/15-stdlib.md#15-standard-library-overview):
   the conventions, [§15.1 the prelude](../../../../docs/language/15-stdlib.md#151-the-prelude),
   [§15.2 core modules](../../../../docs/language/15-stdlib.md#152-core-modules)
   and [§15.3 HTTP](../../../../docs/language/15-stdlib.md#153-http).
2. [ADR-0004](../../../../docs/decisions/0004-whole-program-reachability.md):
   why tree-shaking is reachability over mono MIR.
3. [06 Standard library and tree-shaking](../../../../docs/architecture/stdlib-and-shaking.md):
   [§6.2 tiers](../../../../docs/architecture/stdlib-and-shaking.md#62-library-tiers),
   [§6.4 the algorithm](../../../../docs/architecture/stdlib-and-shaking.md#64-the-reachability-algorithm),
   [§6.5 capabilities and slices](../../../../docs/architecture/stdlib-and-shaking.md#65-capabilities-and-runtime-slices),
   [§6.6 the size report](../../../../docs/architecture/stdlib-and-shaking.md#66-verse-build---size-report),
   [§6.8 per-target checks](../../../../docs/architecture/stdlib-and-shaking.md#68-per-target-constraints),
   [§6.9 package format](../../../../docs/architecture/stdlib-and-shaking.md#69-package-format-of-a-domain-library).
4. [05 §5.6 the `sys` capability interface](../../../../docs/architecture/runtime.md#56-the-sys-capability-interface)
   and [§5.1 feature slices](../../../../docs/architecture/runtime.md#51-feature-slices).
5. [20 First-party packages](../../../../docs/architecture/first-party-packages.md),
   starting with [§20.1](../../../../docs/architecture/first-party-packages.md#201-rules-every-package-follows),
   when you touch `finance`, `ml`, `web`, `gui` or `iot`.

## What it does

| Part | Owns | Source |
|------|------|--------|
| `std/` | the standard library, written in Verse | [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout) |
| `verse-reach` | reachability and capability analysis | [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout) |

Library tiers
([06 §6.2](../../../../docs/architecture/stdlib-and-shaking.md#62-library-tiers)):

| Tier | Ships with | How a program gets it |
|------|------------|-----------------------|
| Prelude (`print`, `input`, `args`, `env`, `exit`, `error`, built-in types and traits) | the toolchain | always visible, still shaken |
| Standard library (`fs`, `io`, `net`, `http`, `text`, `time`, `json`, `task`, `math`, `random`, `process`, `os`, `env`, `cli`, `log`) | the toolchain, versioned with it | `import fs`, `import get from http` |
| First-party packages (`finance`, `ml`, `web`, `gui`, `iot`) | the registry | `verse add`, then `import` |
| Third-party packages | the registry, git or a path | `verse add`, then `import` |

The standard library is not a package. It is never in `verse.lock`.

**Reachability** is the pipeline stage after the MIR optimizer
([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)):

- Input: optimized mono MIR. In script mode, the pre-pass of
  [02 §2.6](../../../../docs/architecture/compiler-architecture.md#26-monomorphization).
- Output: the functions, types, vtables, constants and capabilities the program
  uses, from its entry points. Each capability maps to a runtime slice.
- Errors: V0901 to V0905, V0908, V0910, V0916, V0917
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
  A missing or denied capability stops the build or run before any code runs.

## How it works

**Modules are Verse over `sys`.** Every standard library module is written in
Verse over `sys` capabilities, so it is portable wherever its capabilities
exist. User code reaches capability intrinsics only through the library
([06 §6.2](../../../../docs/architecture/stdlib-and-shaking.md#62-library-tiers),
[§6.5](../../../../docs/architecture/stdlib-and-shaking.md#65-capabilities-and-runtime-slices)).
Reasons for `sys`: small ports, exact reachability, natural denial, and fake
capabilities for tests
([05 §5.6](../../../../docs/architecture/runtime.md#56-the-sys-capability-interface)).

**Conventions.** A function that can fail returns `T | Error` or a module
error type. A function that may find nothing returns `T?`. Resources close
when they are dropped, and a close that can fail is a named method
([spec §15](../../../../docs/language/15-stdlib.md#15-standard-library-overview)).

**Importing is the opt-in.** Imports have no side effects, and module
constants initialize on first use. So an import alone adds nothing to a
binary, and leaving out an unused module cannot change behavior
([06 §6.3](../../../../docs/architecture/stdlib-and-shaking.md#63-importing-is-the-opt-in)).

**The algorithm.** Roots are the entry file's top-level statements, the
selected tests in a test build, or the exported items of a library artifact.
Edges are `call`, `spawn`, `fn-value`, `trait-coerce`, `dyn-call`, `const`,
`drop`, `cleanup`, `field-drop`, `layout`, `panic`, `intrinsic`, `extern` and
`slice`. A breadth-first worklist keeps the first reason found for each
item, so every recorded reason is a shortest chain from a root
([06 §6.4](../../../../docs/architecture/stdlib-and-shaking.md#64-the-reachability-algorithm)).

**Trait types.** The included slot bodies are converted types times called
methods. A vtable exists only if reachable code converts a `T` to the trait
type. A slot is needed only if a reachable `CallDyn` calls that method.

**Exactness.** The answer is exact for the program text because the language
has no reflection, no run-time code loading and no compile-time code
execution. `Show`, `Eq`, `Hash`, `ToJson` and `FromJson` are derived per used
type at compile time.

**Capabilities select slices.** `stdio`, `fs`, `net`, `clock`, `random`,
`process`, `os` and `env` map to `sys-*` slices. `core` is always in.
`unwind`, `tasks` and `debug` are selected by MIR facts
([06 §6.5](../../../../docs/architecture/stdlib-and-shaking.md#65-capabilities-and-runtime-slices)).
A hello world links `core` and `sys-stdio` only
([05 §5.1](../../../../docs/architecture/runtime.md#51-feature-slices)).

**Size report and `verse why`.** `verse build --size-report` prints one line
per included item with its immediate reason, the slices and capabilities, and
an excluded section. `verse why <item>` prints the full chain
([06 §6.6](../../../../docs/architecture/stdlib-and-shaking.md#66-verse-build---size-report)).

**Script mode.** The front end checks every imported module before running.
Every reachable instance is monomorphized and verified before start; bytecode
generation is lazy. Capabilities are still checked before start, so script and
compiled modes fail the same way
([06 §6.7](../../../../docs/architecture/stdlib-and-shaking.md#67-dynamic-cases)).

**Caching.** Per-instance edge lists and shaking results are keyed by blake3.
Slices are prebuilt per target and content-addressed
([06 §6.10](../../../../docs/architecture/stdlib-and-shaking.md#610-determinism-and-caching)).

## Rules that must hold

- Standard library modules are top-level: `import json`, never a prefixed
  form. Their names are reserved for a module, a dependency or a package
  ([spec §15.2](../../../../docs/language/15-stdlib.md#152-core-modules),
  [spec §12.2](../../../../docs/language/12-modules.md#122-importing)).
- One obvious name per idea. A first-party package never offers a second
  spelling of a standard library idea. HTTP serving is `http.serve`, and `web`
  builds on it
  ([06 §6.2](../../../../docs/architecture/stdlib-and-shaking.md#62-library-tiers),
  [20 §20.1](../../../../docs/architecture/first-party-packages.md#201-rules-every-package-follows)).
- File access is not in the prelude. It lives in `fs`
  ([spec §15.1](../../../../docs/language/15-stdlib.md#151-the-prelude)).
- First-party packages are ordinary packages: no compiler privileges, no
  special syntax, locked and checked against `verse.api`.
- Every first-party package declares `[library] capabilities`,
  `[library.targets]` and a `size_budget`, and keeps the rules of
  [20 §20.1](../../../../docs/architecture/first-party-packages.md#201-rules-every-package-follows):
  no stored references, failure in the type, resources are runtime-owned
  handles.
- Per-target code is chosen by files and manifest data, never by `cfg`-style
  conditions in source
  ([06 §6.9](../../../../docs/architecture/stdlib-and-shaking.md#69-package-format-of-a-domain-library)).
- Reachability runs on optimized mono MIR, never earlier, except the script
  mode pre-pass.
- The per-target checks run on the reachable set: capabilities, `--no-heap`
  allocation, bounded `spawn`, threads and library availability
  ([06 §6.8](../../../../docs/architecture/stdlib-and-shaking.md#68-per-target-constraints)).
- Output is stable: roots in source order, edges in MIR order, breadth-first
  traversal ([06 §6.10](../../../../docs/architecture/stdlib-and-shaking.md#610-determinism-and-caching)).
- `Set` iterates in insertion order, like `Map`.
- Capability fakes for tests live in a test-only slice and never reach a
  release build
  ([06 §6.11](../../../../docs/architecture/stdlib-and-shaking.md#611-risks-and-open-questions)).

Not decided in the docs:

- The first-party package APIs are sketches, settled in M9
  ([20](../../../../docs/architecture/first-party-packages.md)).
- The fields of `http.Request` are not listed in the spec
  ([06 §6.6](../../../../docs/architecture/stdlib-and-shaking.md#66-verse-build---size-report)).
- The operations of the `surface` and `gpu` capabilities are designed in M9
  ([06 §6.5](../../../../docs/architecture/stdlib-and-shaking.md#65-capabilities-and-runtime-slices)).
- Filling unused vtable slots with an unreachable trap is an assumption that
  needs a verifier rule
  ([06 §6.4](../../../../docs/architecture/stdlib-and-shaking.md#64-the-reachability-algorithm)).
- No sizes are measured. The targets are in
  [09](../../../../docs/architecture/performance.md).

## What depends on it

Upstream inputs it trusts:

- Demand-driven monomorphization, so uncalled instances never exist
  ([02 §2.6](../../../../docs/architecture/compiler-architecture.md#26-monomorphization)).
- The optimizer, which may remove calls before reachability runs
  ([03 §3.4](../../../../docs/architecture/ir-and-passes.md#34-pass-list)).
- The target file's list of provided capabilities
  ([04 §4.3](../../../../docs/architecture/backends-and-targets.md#43-what-a-target-is)).
- Spec §12.4: imports have no side effects and constants are lazy.

Downstream:

- The backends compile only the reachable set
  ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
- The linker gets only the slices reachability asks for
  ([05 §5.1](../../../../docs/architecture/runtime.md#51-feature-slices)).
- The package manager reads reachable capabilities for `[library] capabilities`,
  the lock's `capabilities` field and `verse update` warnings
  ([21 §21.8](../../../../docs/architecture/packages.md#218-resolution-minimal-version-selection)).
- `verse publish` checks `size_budget` against the size report.
- Every program that imports a module depends on its `verse.api`. A release
  needs the standard library's `verse.api` unchanged or an intentional diff
  ([10 §10.7](../../../../docs/architecture/testing-and-verification.md#107-release-gates)).

## After a change

1. Add tests by layer
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - Conformance tests for each normative sentence of spec §15 you implement.
     Spec examples are extracted and run
     ([10 §10.2](../../../../docs/architecture/testing-and-verification.md#102-specification-conformance)).
   - `test` blocks and doc tests in the module itself, run by `verse test`.
   - Differential tests: every tier gives the same output
     ([10 §10.3](../../../../docs/architecture/testing-and-verification.md#103-differential-testing)).
   - Golden tests of the size report and the why chain when reachability
     changes.
   - Must-fail tests with expected V09xx codes for capability and target
     errors.
   - Boundary tests for overflow, bounds and division in every tier
     ([10 §10.4](../../../../docs/architecture/testing-and-verification.md#104-safety-properties-we-test-explicitly)).
2. Check M5's exit criteria when you add a module: all spec §15 examples run,
   and shaking is verified on each module ([roadmap](../../../../docs/roadmap.md)).
3. Update the docs in the same pull request:
   - A module or signature goes in [spec §15](../../../../docs/language/15-stdlib.md).
   - A capability, slice or edge goes in [06](../../../../docs/architecture/stdlib-and-shaking.md)
     and [05](../../../../docs/architecture/runtime.md).
   - A first-party API goes in [20](../../../../docs/architecture/first-party-packages.md).
   - A new code goes in the [catalog](../../../../docs/reference/diagnostics-catalog.md),
     in V09xx. Codes are never reused.
   - Run `verse api update` and commit the `verse.api` diff.
4. Commit as `<type>(std): …`, for example `spec(std): give http.serve a handler
   that can fail` ([CONTRIBUTING](../../../../CONTRIBUTING.md)). Reachability
   changes use `compiler`.
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
