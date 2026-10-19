# Native plugins

How a Verse program calls a C-ABI library: `[plugins.<name>]` in
`verse.toml`, `extern` blocks, the type mapping, loading in script mode and
compiled mode, and `verse bind`.

## Read first

1. [ADR-0005 Native C-ABI plugins](../../../../docs/decisions/0005-native-c-abi-plugins.md).
2. Spec [§12.6 Foreign functions (`extern`)](../../../../docs/language/12-modules.md#126-foreign-functions-extern).
3. [07 Plugins](../../../../docs/architecture/plugins.md), in order:
   [§7.1](../../../../docs/architecture/plugins.md#71-what-a-plugin-is) to
   [§7.9](../../../../docs/architecture/plugins.md#79-diagnostics).
4. [15 §15.11 Foreign-call boundary](../../../../docs/architecture/target-and-abi.md#1511-foreign-call-boundary)
   and [§15.7.5 Catch points](../../../../docs/architecture/target-and-abi.md#1575-catch-points).
5. [06 §6.7 Dynamic cases](../../../../docs/architecture/stdlib-and-shaking.md#67-dynamic-cases),
   the `extern` row.
6. The [V10xx table](../../../../docs/reference/diagnostics-catalog.md#v10xx-foreign-calls)
   and the [`[plugins]` manifest section](../../../../docs/reference/cli-and-manifest.md#plugins).

## What it does

- A plugin is a compiled library with a C interface. C, C++, Zig and Rust
  can all write one
  ([07 §7.1](../../../../docs/architecture/plugins.md#71-what-a-plugin-is)).
- Inputs: a `[plugins.<name>]` section with `lib`, `path`, `build`, `link`
  and `targets`, and `extern <name>` blocks of full C signatures
  ([07 §7.2](../../../../docs/architecture/plugins.md#72-declaring-a-plugin),
  [07 §7.3](../../../../docs/architecture/plugins.md#73-extern-blocks)).
- Outputs: in script mode, the opened shared library plus a call stub per
  signature. In compiled mode, link lines for a dynamic or static library
  ([07 §7.6](../../../../docs/architecture/plugins.md#76-loading-in-script-mode-and-compiled-mode)).
- The crate `verse-ffi` owns loading, the VM call stubs and link lines
  ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).
- Who reports what
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)):

| Stage | Codes |
|-------|-------|
| Name resolution | V1006 (`extern` block names no plugin) |
| Type inference | V1002, V1003, V1010 (foreign signatures) |
| Backends | V1005, V1007 to V1009, V1011, V1012 (linking foreign code) |
| `verse-pkg` | V0834 (a dependency with `path` or `build`), V0801 (unknown key) |
| `verse-rt` | R0014 (null where the signature says `Pointer`) |

- Plugins are M8: `extern` blocks, `[plugins]`, the VM loader and stubs,
  static and dynamic linking, `verse bind`
  ([roadmap](../../../../docs/roadmap.md#11-roadmap)). No code exists yet.

## How it works

**One mechanism: the platform C ABI.** Why: any language that exports C
functions can write a plugin with no Verse tooling on its side. Sandboxed Wasm
components, capability-gated calls, process plugins and a Verse-specific ABI
were rejected ([ADR-0005](../../../../docs/decisions/0005-native-c-abi-plugins.md)).

**A call is a normal call.** `extern` functions are module items, imported and
exported like any other. A C name that is not a Verse value name is renamed
with `as`. There is no `unsafe` marker and no `with` block
([07 §7.3](../../../../docs/architecture/plugins.md#73-extern-blocks)).

**Type mapping.** Only sized numbers, `Int`, `Float`, `Bool`, `Char`,
`Pointer`, `Pointer?`, `Bytes` and byte views as parameters, `Copy` records of
these, and `Nothing` as a result cross. `Text` never crosses: write
`.bytes()`. A length is passed explicitly. `mut` is a pointer to the caller's
value ([07 §7.4](../../../../docs/architecture/plugins.md#74-values-across-the-boundary)).

**Ownership.** A plain parameter is borrowed for the call. A `mut` parameter
is an exclusive borrow. A result is owned by the caller, and Verse never frees
what a `Pointer` points to. The usual wrapper is a record that holds the
`Pointer` and implements `Cleanup`
([07 §7.5](../../../../docs/architecture/plugins.md#75-ownership-and-lifetime)).

**In MIR** a foreign call is `call foreign plugin.symbol(...)` with
`unwind none`. A `Pointer` result is checked for null, and a null reaches
`panic foreign_null`
([13 §13.4](../../../../docs/architecture/mir-specification.md#134-statements-and-terminators)).

**At the ABI.** Compiled code needs no adapter for plain data. A `Copy` record
is copied to the target's C layout, which may differ from its Verse layout. A
`Bytes` argument passes a pointer followed by a zero byte; a slice without one
is copied. In script mode a `verse-ffi` stub moves VM registers into C-ABI
arguments and back
([15 §15.11](../../../../docs/architecture/target-and-abi.md#1511-foreign-call-boundary)).

**Loading.** Reachability adds an edge from each reachable call to the plugin
and its symbol. A plugin with no reachable call is neither loaded nor linked
([07 §7.6](../../../../docs/architecture/plugins.md#76-loading-in-script-mode-and-compiled-mode)).

| | Script mode | Compiled mode |
|---|---|---|
| when | before the first statement runs | at link time |
| how | `dlopen` or `LoadLibrary`, then resolve each reachable symbol | the linker, dynamic or static per `link` |
| missing library or symbol | V1007 or V1008, exit status 2 | the build fails with V1007 or V1008 |

**Tasks.** A foreign call holds the calling task's thread until it returns.
It is not a yield point or a cancellation point. On a state-machine target it
blocks the single thread
([15 §15.11](../../../../docs/architecture/target-and-abi.md#1511-foreign-call-boundary)).

**`verse bind`** reads a C header and writes one `extern` block to
`--out <file>`. Inexpressible declarations become comments that say why. The
author decides nullability by editing
([07 §7.7](../../../../docs/architecture/plugins.md#77-verse-bind)).

**Targets.** Native targets link dynamic or static. WASI and microcontrollers
link static only. Browser Wasm has no plugins: a reachable foreign call is
V1005 ([07 §7.8](../../../../docs/architecture/plugins.md#78-targets)).

## Rules that must hold

- **A plugin never changes the language.** It adds no keyword, operator,
  annotation or grammar rule
  ([07 §7.1](../../../../docs/architecture/plugins.md#71-what-a-plugin-is)).
- **Declaring is the only opt-in.** No other grant exists (V1006)
  ([07 §7.2](../../../../docs/architecture/plugins.md#72-declaring-a-plugin)).
- **A dependency may declare `lib` plugins only.** `path` or `build` in a
  dependency is V0834, because building a dependency never runs code.
- **`build` never runs during `verse check`.**
- **`take` is not allowed** (V1010). Verse memory is never handed to foreign
  code. No callbacks and no variadic functions
  ([07 §7.4](../../../../docs/architecture/plugins.md#74-values-across-the-boundary)).
- **No unwinding through foreign frames.** `extern` functions are
  `nounwind`, and foreign code never calls back into Verse
  ([15 §15.7.5](../../../../docs/architecture/target-and-abi.md#1575-catch-points)).
- **A load failure is a start-up failure**, never a late one. Every reachable
  symbol is resolved before the first statement, in both modes
  ([07 §7.6](../../../../docs/architecture/plugins.md#76-loading-in-script-mode-and-compiled-mode)).
- **Both modes run the same checks, the same type mapping and the same
  machine code** inside the library.
- **A foreign call is not a capability use.** `--deny` and `[capabilities]`
  do not reach foreign code
  ([15 §15.11](../../../../docs/architecture/target-and-abi.md#1511-foreign-call-boundary)).
- **`verse bind` never overwrites** an existing file.

## What depends on it

- Upstream, it trusts name resolution and type inference for the Verse side
  of each signature. See [names-and-types.md](names-and-types.md).
- It trusts `verse-pkg` for the `[plugins]` section. See
  [packages-and-cli.md](packages-and-cli.md).
- Reachability supplies the reached symbols, and the size report lists each
  linked plugin with the chain that reached it
  ([06 §6.7](../../../../docs/architecture/stdlib-and-shaking.md#67-dynamic-cases)).
- The VM depends on `verse-ffi` stubs, and native backends emit direct C-ABI
  calls. See [vm-and-backends.md](vm-and-backends.md).
- The debugger shows a foreign call as `called` with `foreign: true`, does
  not step into it, and marks returned values `from_foreign`
  ([07 §7.5](../../../../docs/architecture/plugins.md#75-ownership-and-lifetime)).
  See [debugger-and-language-server.md](debugger-and-language-server.md).
- Changing the type mapping touches spec §12.6, 07 §7.4, 15 §15.11,
  `verse bind` and the allowed-types list of V1002.

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - must-fail programs under `tests/must-fail/` with the expected V10xx code;
   - golden tests of each diagnostic's rendered text;
   - conformance tests for spec §12.6;
   - differential tests. The M8 exit criterion is the same program calling
     SQLite and a Rust `cdylib` in tier 0 and in a native build, with
     identical output ([roadmap](../../../../docs/roadmap.md#11-roadmap));
   - unit tests for stubs and library lookup.
2. Update doc 07 and 15 §15.11. A change to what crosses or to `extern`
   syntax changes spec §12.6 in a `spec` pull request. Manifest keys also
   update 19 `[plugins]`.
3. New codes go in V10xx and follow the catalog rules, each with its three
   tests. See [diagnostics.md](diagnostics.md).
4. Use the commit format of
   [CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format) with the scope
   `plugins`.
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
