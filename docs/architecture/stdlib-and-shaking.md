# 06. Standard library and tree-shaking

Requirement R6: Verse has libraries for finance, machine learning, servers,
GUI and more; they are optional; and a build contains only what the program
actually uses, found by analysing the program at build time. This document
says how. The mechanism itself is decided in
[ADR-0004](../decisions/0004-whole-program-reachability.md); this document gives the
details, the report format, the edge cases and the package format.
The manifest and package manager are in [19](../reference/cli-and-manifest.md); the
first-party package APIs are sketched in [20](first-party-packages.md).

Nothing in this document has been implemented or measured.

## 6.1 Summary

1. **Importing is the opt-in.** A program gets a library only by naming it in
   an `import` (spec §12.2). There is no implicit "full standard library".
2. **Using is what costs.** An import alone adds nothing to a binary. Only
   functions, types, vtables, constants and capabilities that are
   *reachable* from the program's roots are built in.
3. **Reachability runs on optimized mono MIR**, where every generic is
   specialized and every call names its callee, so the answer is exact for
   the program text ([ADR-0004](../decisions/0004-whole-program-reachability.md)).
4. **Capabilities select runtime slices.** Reachable code that touches the
   platform does so through `sys` capabilities ([05 §5.6](runtime.md#56-the-sys-capability-interface)),
   and each capability maps to a runtime slice ([05 §5.1](runtime.md#51-feature-slices)).
5. **Every included item can say why.** `verse build --size-report` lists
   each item with the chain of edges from a root that pulled it in.

## 6.2 Library tiers

| Tier | What | Examples | Ships with | How a program gets it |
|------|------|----------|------------|-----------------------|
| Prelude | names available without import (spec §15.1) | `print`, `input`, `args`, `env`, `exit`, `error`, built-in types and traits | the toolchain | always visible; still shaken like everything else |
| Standard library | general-purpose top-level modules (spec §15.2) | `fs`, `io`, `net`, `http`, `text`, `time`, `json`, `task`, `math`, `random`, `process`, `os`, `env`, `cli`, `log` | the toolchain, versioned with it | `import fs`, `import get from http` |
| First-party optional packages | domain libraries maintained by the Verse project | finance, machine learning, web services, GUI, IoT peripherals: `finance`, `ml`, `web`, `gui`, `iot` | the package registry, installed on demand | `verse add finance`, then `import … from finance` |
| Third-party packages | anything published to the registry, git or path | `colors`, `http_extra` (spec §16.3 examples) | the registry | `verse add colors`, then `import red, bold from colors` |

Notes:

- **The standard library stays small and portable.** Every module is written in Verse
  over `sys` capabilities ([05 §5.6](runtime.md#56-the-sys-capability-interface)),
  so it is portable wherever its capabilities exist. A module whose
  capability a target lacks (for example `fs` on a Cortex-M board) still
  type-checks; using it on that target is a build error (§6.8).
- **First-party packages are ordinary packages.** They are versioned, locked
  in `verse.lock`, checked against their `verse.api` (spec §16.3, §16.4), and
  installed with `verse add` (spec §16). They get no compiler privileges and
  no special syntax. They may release on their own schedule; each release
  states the range of toolchain versions it supports (§6.9).
- **One obvious name per idea.** The first-party packages must not offer a
  second spelling of something the standard library already has. For example, HTTP serving
  is `http.serve` (spec §15.3); a web package builds routing and templates
  *on top of* it and does not ship its own server loop. The names `finance`,
  `ml`, `web`, `gui` and `iot` are reserved for these packages; each covers
  one idea.

## 6.3 Importing is the opt-in

The only signal a program gives about which libraries it wants is its import
lines. This is consistent with spec §12.2 (every name traceable to a
declaration or an import, no wildcard imports) and §15.1 (file access is not
in the prelude).

```verse
import read_text from fs
import parse, Json from json

fn load_config(path: Text): Json | Error
    text = read_text(path)?
    parse(text)

config = load_config("app.json")?
print(config)
```

This program reaches `read_text`, `parse` and whatever they call, `Show` for
`Json` (because `print` formats it), and the `fs` and `stdio` capabilities.
It reaches nothing else from `fs` or `json`.

Two facts from the spec make "import costs nothing" safe:

- **Imports have no side effects** (spec §12.4): an imported module contains
  only declarations and lazily initialized constants, so leaving out an
  unused module cannot change behaviour.
- **Module constants are initialized on first use** (spec §12.4), so a
  constant is included only if a reachable function reads it, and its
  initializer is reachable only through that read.

An import whose names are never used is reported as a warning by the
compiler; it never adds code.

## 6.4 The reachability algorithm

As [02 §2.1](compiler-architecture.md#21-pipeline) says, reachability
runs after the MIR optimizer, on optimized mono MIR. Monomorphization is
already demand driven ([02 §2.6](compiler-architecture.md#26-monomorphization)),
so instances that are never called are never created; the reachability pass
then works on the *optimized* bodies, where constant folding, inlining,
devirtualization and dead-code elimination ([03 §3.4](ir-and-passes.md#34-pass-list))
may have removed further calls.

### Roots

| Build | Roots |
|-------|-------|
| Program (`verse build`, `verse app.vs`) | the entry file's top-level statements. Verse has no `fn main` (spec §12.5); a function named `main` is reached only if the script calls it |
| Tests (`verse test`) | every selected `test` block and doc test (spec §14), plus the module constants they read. Entry-file statements are *not* roots, because tests never run them (spec §12.4, §14.1) |
| Library artifact (C ABI or Wasm component, [04 §4.4](backends-and-targets.md#44-linking-and-artifacts)) | every exported item, instantiated at the concrete types its exported signature names |
| Generator output | not a root by itself: generated `.vs` files are ordinary modules, reached through imports like any other ([spec §16.6](../language/16-tooling.md#166-generators)) |

### Edges

An item is a node; an edge says "if this is included, that must be too".

| From | Edge kind | To | Notes |
|------|-----------|----|-------|
| function | `call` | the callee instance (`Call` terminator) | after devirtualization |
| function | `spawn` | the spawned instance (`Spawn` terminator), and the `tasks` slice | spec §11.1 |
| function | `fn-value` | a function or lambda body used as a value (a function reference or a closure aggregate) | a function value may be called later through any path, so taking its value includes it |
| function | `trait-coerce` | the vtable for `(ConcreteType, Trait)` when a value is converted to a trait type (spec §10.3) | see *Trait types* below |
| function | `dyn-call` | slot *k* of trait *T* (`CallDyn` terminator) | combines with `trait-coerce` |
| function | `const` | a module constant's initializer instance | first read only; spec §12.4 |
| function | `drop` | drop glue for type *X* (a `Drop` statement) | |
| drop glue of *X* | `cleanup` | `Cleanup.cleanup` for *X*, if implemented | spec §13 |
| drop glue of *X* | `field-drop` | drop glue of each field, element or payload type that needs dropping | types without heap data or `Cleanup` have no drop glue ([03 §3.4](ir-and-passes.md#34-pass-list), `drop-opt`) |
| function | `layout` | the type's layout record | layout costs no code; it is recorded so the report can show which types exist |
| function | `panic` | panic machinery in `core`; the `unwind` slice if a reachable unwind edge runs a drop | spec §8.5 |
| function | `intrinsic` | the capability the intrinsic belongs to (`sys.net.accept` → `net`) | see §6.5 |
| function | `extern` | the plugin and its native symbol ([07 §7.6](plugins.md#76-loading-in-script-mode-and-compiled-mode)) | a foreign call adds no capability |
| capability | `slice` | the runtime slice that provides it | [05 §5.1](runtime.md#51-feature-slices) |

### Worklist

```
include(item, reason):
    if item not in included:
        included[item] = reason          # the first reason found is kept
        worklist.push(item)

for r in roots (in source order): include(r, Root)
while worklist not empty:
    item = worklist.pop_front()          # breadth first: shortest why chains
    for (edge, target) in edges(item):   # cached per instance, see 6.10
        include(target, Edge(item, edge))
    if item is a trait-coerce vtable (T, Trait) or a dyn-call slot (Trait, k):
        for every pair now present: include(slot body of T for k, VtableSlot)
```

Breadth-first order means the recorded reason is a shortest path from a
root, which keeps "why" answers short. The traversal is linear in the number
of included edges (Target: no super-linear step, consistent with
[09](performance.md)).

### Trait types

Spec §10.3 lets a trait name be used as a type (`List<Speaker>`), with calls
dispatched at run time. Two facts bound what such a call can reach:

- a vtable for `(T, Trait)` exists only if some reachable code converts a `T`
  to the trait type (`trait-coerce`);
- a slot is needed only if some reachable `CallDyn` calls that method of that
  trait (`dyn-call`).

So the included slot bodies are the intersection: *converted types × called
methods*. This is tighter than "all implementations for instantiated types"
in ADR-0004 and is still sound, because a trait-typed value can only be
created by a conversion the compiler sees. Unused slots in an included
vtable are filled with a trap that cannot be reached (Assumption: this needs
the verifier to prove that every `CallDyn` slot index is in the called set).

### Why the result is exact, and what "exact" means

- **Exact for the program text.** Every call in mono MIR names its callee or
  a trait slot; there is no reflection and no run-time code loading in the
  language (the spec has neither; `Show`, `Eq`, `Hash`, `ToJson` and
  `FromJson` are derived structurally at compile time per used type, spec
  §10.4, §15.2). So nothing can be called that the analysis did not see.
- **Conservative for run-time values.** A branch that never runs is still
  included, as in every static analysis. The optimizer removes branches it
  can prove dead; the size report shows the rest.
- **What would break exactness.** Run-time reflection, dynamic loading of
  Verse code, or code injected at build time. The language has none of these.
  A plugin is reached only through `extern` functions, which are ordinary call
  edges, and generators write visible source.

## 6.5 Capabilities and runtime slices

A **capability** is a named group of platform operations reached through the
`sys` interface ([05 §5.6](runtime.md#56-the-sys-capability-interface)).
Standard library code calls capability intrinsics; user code reaches them
only through the library. Foreign code in a plugin reaches the platform
directly, outside this table ([07 §7.1](plugins.md#71-what-a-plugin-is)).

| Capability | Typical source in the library | Runtime slice ([05 §5.1](runtime.md#51-feature-slices)) | Example target without it |
|------------|-------------------------------|----------------------------|---------------------------|
| `stdio` | `print`, `input` (prelude), `io`, error reports of a failed top-level `?` | `sys-stdio` | a headless board without UART console |
| `fs` | `fs` | `sys-fs` | `wasm32-unknown-unknown`, Cortex-M |
| `net` | `net`, `http` | `sys-net` | Cortex-M without a network stack |
| `clock` | `time` | `sys-clock` | none planned |
| `random` | `random` | `sys-random` | none planned (hardware or seeded) |
| `process` | `process`, `exit` | `sys-process` | browser, microcontrollers |
| `os` | `os` | `sys-os` | WASI, browser, microcontrollers |
| `env` | `env`, the prelude `env`, `args` | `sys-env` | microcontrollers (empty `args`) |
| `gpio`, `uart`, `i2c`, `spi` | IoT packages | `sys-gpio`, `sys-uart`, `sys-i2c`, `sys-spi` | desktop targets |
| `surface` (windows, input, drawing) | GUI package | `sys-surface` | servers, microcontrollers |
| `gpu` (compute) | ML package | `sys-gpu` | most targets |

The `surface` and `gpu` capabilities are named now; their operations are
designed in M9.

Non-capability slices are selected by MIR facts rather than by capabilities:

| Slice | Included when |
|-------|---------------|
| `core` | always (allocator, panic machinery, `Text`, `List`, `Map`, `Set` primitives) |
| `unwind` | a reachable unwind edge runs a drop (a `Cleanup` call or a heap free) *and* the target's unwind strategy needs runtime support (`native`); with `status-code` the cleanup paths are ordinary code (Assumption, see [05 §5.4](runtime.md#54-panics-and-unwinding)) |
| `tasks` | a `Spawn` terminator or a `task` primitive is reachable |
| `debug` | debug builds, running a file, or an attached debugger ([08](debugging.md)) |

Each target file declares which capabilities it provides
([04 §4.3](backends-and-targets.md#43-what-a-target-is)). After
reachability the build compares the needed set with the provided set; a
missing capability is a build error that names the call chain (§6.8).

`Set` iterates in insertion order, like `Map`, so both use the same
compact-dictionary code in `core`, and iteration order in size reports,
replays and test output is deterministic.

## 6.6 `verse build --size-report`

The flag is in the spec's tool table (§16) and is planned for M2
([roadmap](../roadmap.md)). The report is generated from the `included` map of §6.4, so it
explains every item by construction.

### Format

- A header: target, profile, backend, roots.
- One line per included item: name, kind (`root`, `function`, `impl`,
  `vtable`, `drop`, `const`, `slice`), size, and the *immediate* reason.
- Runtime slices and capabilities, each with the item that required it.
- An **excluded** section listing modules that were imported but contribute
  nothing, and slices that were considered and left out, with the reason.
- `--size-report=json` emits the same data as records, and
  `verse why <item>` prints the full chain from a root.

Sizes are shown as `<n>` below. **No sizes have been measured.** The binary
size targets are in [09](performance.md): hello world under 200 KB native
(stripped) and under 30 KB as Wasm.

### Worked example 1: hello world

```verse
print("Hello, world!")
```

```
$ verse build hello.vs --release --size-report
target   x86_64-unknown-linux-gnu    profile release    backend llvm
roots    1   hello.vs: entry statements

ITEM                                   KIND       SIZE  WHY
hello.vs: entry statements             root        <n>  root
print<Text>                            function    <n>  call      hello.vs:1
Show for Text                          impl        <n>  call      print<Text>
"Hello, world!"                        const       <n>  const     hello.vs:1
sys.stdio.write_stdout                 intrinsic     -  intrinsic print<Text>

RUNTIME
core                                   slice       <n>  always
sys-stdio                              slice       <n>  capability stdio <- sys.stdio.write_stdout

CAPABILITIES  stdio

EXCLUDED
unwind      no reachable unwind edge runs a drop
tasks       no spawn reachable
debug       release profile
```

`print<Text>` names the specialization chosen by the compiler. It is not Verse
syntax: explicit type arguments on free functions are a compile error
(spec §10.5).

### Worked example 2: a small web server

`serve` and `Response` are those of spec §15.3. The spec does not list the
fields of `Request` yet, so the field `path` below is **assumed** for
illustration.

```verse
import serve, Request, Response from http

fn handle(request: Request): Response | Error
    match request.path
        "/" -> Response(status: 200, headers: {:}, body: b"hello")
        _ -> Response(status: 404, headers: {:}, body: b"not found")

serve(port: 8080, handler: handle)?
```

```
$ verse build server.vs --release --size-report
target   x86_64-unknown-linux-gnu    profile release    backend llvm
roots    1   server.vs: entry statements

ITEM                                   KIND       SIZE  WHY
server.vs: entry statements            root        <n>  root
http.serve                             function    <n>  call      server.vs:8
handle                                 function    <n>  fn-value  server.vs:8 (passed as handler)
net.TcpListener.accept                 function    <n>  call      http.serve
http.parse_request                     function    <n>  call      http.serve
http.write_response                    function    <n>  call      http.serve
log.error                              function    <n>  call      http.serve (a handler error)
Map<Text, Text> (headers)              layout        -  layout    http.parse_request
drop Request                           drop        <n>  drop      http.serve
Cleanup for TcpStream                  impl        <n>  cleanup   drop TcpStream
…                                      …             …  …
sys.net.accept                         intrinsic     -  intrinsic net.TcpListener.accept
spawn per request                      function    <n>  spawn     http.serve

RUNTIME
core                                   slice       <n>  always
tasks                                  slice       <n>  spawn     http.serve
unwind                                 slice       <n>  drop on unwind edge in http.serve
sys-net                                slice       <n>  capability net <- sys.net.accept
sys-stdio                              slice       <n>  capability stdio <- error report of a failed top-level `?` (spec §12.5)

CAPABILITIES  net, stdio

EXCLUDED
http.get, post, put, delete      imported module members not reached
sys-fs                           no fs intrinsic reachable
```

The internal names (`TcpListener.accept` in `net`, `parse_request` in `http`,
`spawn per request`) are placeholders for whatever the library's
implementation uses; the shape of the report is the point.

### The why chain

```
$ verse why net --target x86_64-unknown-linux-gnu
this program reaches `net` because:
  sys.net.accept                    intrinsic
  <- net.TcpListener.accept         std/net/tcp.vs
  <- http.serve                     std/http/server.vs
  <- server.vs:8                    serve(port: 8080, handler: handle)?
  <- root: entry statements of server.vs
```

The chain is the stored reason of each node, followed back to a root
(§6.4). Because the traversal is breadth first, it is a shortest chain. A
runtime slice is explained through the capability that requires it: the
`RUNTIME` section names the capability (`sys-net` from `capability net`), and
`verse why net` prints the chain. The argument kinds of `verse why` are
[19 §19.4](../reference/cli-and-manifest.md#194-verse-why).

## 6.7 Dynamic cases

| Case | What happens | Source |
|------|--------------|--------|
| Trait types (`List<Speaker>`, spec §10.3) | converted types × called slots (§6.4) | refines ADR-0004 |
| Function values and lambdas (spec §4.7, §5.4) | taking a function's value or creating a closure includes its body (`fn-value`); a call through a function value then needs nothing new | detail of ADR-0004 |
| Reflection | none in the spec; `Show`, `Eq`, `Hash`, `ToJson`, `FromJson` are derived per used type and shaken like any function | spec §10.4 |
| `Show` for interpolation and `print` | each interpolated or printed type pulls in its own `Show` instance and nothing else | spec §10.4 |
| Generators | output is ordinary `.vs` source, reached through imports | spec §16.6 |
| `extern` functions | each reachable call is an edge to the plugin and its symbol; a plugin with no reachable call is neither linked nor loaded | [07 §7.6](plugins.md#76-loading-in-script-mode-and-compiled-mode) |
| Test builds | roots are the selected tests (§6.4); `[dev-dependencies]` are visible only here (spec §16.3); fakes for capabilities (virtual clock, in-memory fs, [05 §5.6](runtime.md#56-the-sys-capability-interface)) replace slices | spec §16.3 |
| Module constants | included on first reachable read; the initializer is infallible (spec §12.4), so leaving out an unread one changes nothing | spec §12.4 |

### Script mode (tier 0)

`verse app.vs` must mean exactly what `verse build` means ([00](overview.md),
principle 1). It differs only in *when* work happens:

- The front end checks the entry file and every module it imports, directly
  or transitively, before running anything. Packages and the standard library are checked
  against their `verse.api`, with cached results ([02 §2.7](compiler-architecture.md#27-incremental-and-parallel-compilation)).
  Modules that are not imported are never opened.
- Every reachable instance is monomorphized and verified before the first
  statement. Bytecode generation is lazy, per function, at first call
  ([02 §2.6](compiler-architecture.md#26-monomorphization)), so its cost
  scales with what runs.
- Because imports have no side effects and constants are lazy (spec §12.4),
  loading lazily cannot change behaviour.
- Capabilities are still checked: `verse --deny net app.vs` refuses a program
  whose reachable code needs `net` before it starts. The check runs
  reachability on the program before execution, so a denied capability fails
  at start, not halfway through, and script and compiled modes fail the same
  way.

## 6.8 Per-target constraints

After reachability, the build checks the result against the target:

| Check | Source of the rule | Failure message |
|-------|--------------------|------------------------------------------|
| Every needed capability is provided | target file `[capabilities]` ([04 §4.3](backends-and-targets.md#43-what-a-target-is)) | "this program calls `http.get`, which `thumbv7em-none-eabihf` cannot provide" followed by the why chain |
| No heap use under `--no-heap` | [05 §5.7](runtime.md#57-embedded-profile): no reachable allocation intrinsic | "`--no-heap`: `List.push` allocates, reached from `blink.vs:12`" |
| Bounded spawns under `--no-heap` | [15 §15.3.7](target-and-abi.md#1537-handles-tasks-channels-shared-guarded): every reachable `spawn` runs a bounded number of times | "this `spawn` can run without bound, and `--no-heap` is on" |
| No threads where the target has none | target `tasks` and `atomics` | spawning still works on cooperative targets; only true thread requirements fail |
| Library availability | the package's per-target table (§6.9) | "`gui` is not available for `riscv32imc-unknown-none-elf`" |

The check runs on the *reachable* set, so importing `http` in a library
used on a board is fine as long as the board build never reaches it.
`verse check --target T` runs the same check without code generation.

## 6.9 Package format of a domain library

A library package is a normal package (spec §16.3). To be shaken precisely
and to fail early on unsupported targets it declares a little more in its
`verse.toml`. The section is data only, so it keeps spec §16.3's "no build
scripts" promise.

```toml
[package]
name = "finance"
version = "1.0.0"
verse = "0.1"

[library]
modules      = ["finance", "finance.bonds", "finance.calendar"]
capabilities = []                   # none: finance is pure, dates are passed in (20 §20.2); time would be `clock`
size_budget  = "150K"               # fails `verse publish` if the example build exceeds it

[library.targets]                   # where the package is available
default      = "all"
unavailable  = []                   # for example ["thumbv7em-none-eabihf"] for a GUI package
```

| Field | Why it is needed |
|-------|------------------|
| `modules` | the public module list; anything else is private, which keeps one obvious name per idea and lets `verse doc` list them |
| `capabilities` | an upper bound the registry can show before installation; the real set is still computed by reachability |
| `size_budget` | a published promise checked by `verse publish` against the package's example programs (each first-party package sets one) |
| `targets` | lets the build fail with a clear message instead of a missing-capability error deep in a call chain |

Per-target code is chosen by files and manifest data (`[library.targets]`
here, `[targets.<t>] files` in [19](../reference/cli-and-manifest.md)), never
by `cfg`-style conditions in source. The spec has no such syntax, and a
plugin adds none ([07 §7.1](plugins.md#71-what-a-plugin-is)).

Using such a package, with assumed names:

```verse
import Decimal from math
import Money from finance

fn total(prices: List<Money>): Money | Error
    mut sum = Money(amount: Decimal.zero(), currency: "EUR")
    for price in prices
        sum = sum.add(price)?
    sum
```

(`Decimal.zero()`, `Money(amount:, currency:)` and `add` are illustrative; the
finance package API is not designed yet.)

## 6.10 Determinism and caching

- **Stable order.** Roots are taken in source order, edges in MIR order, and
  the worklist is breadth first, so two builds of the same inputs produce the
  same included set, the same reasons and the same report (spec §16.3:
  reproducible builds; [02 §2.7](compiler-architecture.md#27-incremental-and-parallel-compilation)).
- **Per-instance edge lists are cached.** The edges of one mono MIR instance
  depend only on its optimized body, so they are stored in `.verse/cache/`
  keyed by `blake3(instance body, target, flags)`. An incremental build
  re-derives edges only for changed instances and re-runs the (linear)
  traversal.
- **Shaking results are cached.** The included set, the capability set and the
  slice list are keyed by `blake3(root set, edge-list hashes, target file,
  profile)`. When the key is unchanged the link step reuses the previous
  object list and report.
- **Slices are prebuilt per target** and content-addressed, so selecting a
  slice never compiles the runtime.

## 6.11 Risks and open questions

| # | Risk or question | Why it matters | Handling |
|---|------------------|----------------|----------|
| R-1 | **Generic libraries are monomorphized in the consumer.** A library of unannotated functions is compiled once per concrete type, in every program that uses it | compile time and code size grow with use ([09](performance.md), P6) | identical code folding; the size report shows instance counts; library authors can annotate concrete types where specialization does not pay |
| R-2 | **ML needs native kernels and GPUs** | BLAS and GPU compute are not Verse code | pure-Verse kernels, a `gpu` capability (§6.5), and BLAS-like libraries through a plugin ([07](plugins.md)) |
| R-3 | **GUI needs platform toolkits** | windows and input are per-OS; the browser has its own | a `surface` capability with per-target platform layers (§6.5); a pure-Verse widget library on top; the web target uses `std-web` |
| R-5 | **Names of first-party packages** | the rule: one obvious name per idea | `finance`, `ml`, `web`, `gui`, `iot` (§6.2) |
| R-6 | **Plugins never add syntax** | a plugin that changed meaning would also break exactness | already a rule ([07 §7.1](plugins.md#71-what-a-plugin-is)); there are no transforming plugins |
| R-7 | **`print` pulls in `stdio` and the failed-`?` report pulls in `stderr`** | even "silent" programs may need `stdio` because a failed top-level `?` prints (spec §12.5) | acceptable; the report shows why |
| R-8 | **Test builds and fakes** | capability fakes must not leak into release builds | fakes live in a test-only slice selected only in `verse test` |
