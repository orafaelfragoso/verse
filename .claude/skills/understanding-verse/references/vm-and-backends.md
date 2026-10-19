# VM and backends

The last stage of the pipeline: the tier-0 bytecode VM, the Cranelift and LLVM
native backends, the Wasm emitter, target files and artifacts. All of them
lower the same mono MIR.

## Read first

1. [ADR-0003 One mono MIR, many backends](../../../../docs/decisions/0003-one-mir-many-backends.md)
   and [ADR-0008 Tiered execution](../../../../docs/decisions/0008-tiered-execution.md).
2. [04 §4.1 Tiers](../../../../docs/architecture/backends-and-targets.md#41-tiers)
   and [04 §4.2 Target matrix](../../../../docs/architecture/backends-and-targets.md#42-target-matrix).
3. [04 §4.3 What a target is](../../../../docs/architecture/backends-and-targets.md#43-what-a-target-is),
   [§4.4 Linking and artifacts](../../../../docs/architecture/backends-and-targets.md#44-linking-and-artifacts),
   [§4.5 Calling convention and ABI](../../../../docs/architecture/backends-and-targets.md#45-calling-convention-and-abi).
4. [14 Bytecode and the tier-0 VM](../../../../docs/architecture/bytecode-and-vm.md),
   all of it. Start with
   [§14.1 Role](../../../../docs/architecture/bytecode-and-vm.md#141-role) and
   [§14.12 Differential-testing contract](../../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract).
5. [15 §15.1 Purpose and scope](../../../../docs/architecture/target-and-abi.md#151-purpose-and-scope),
   [§15.5.7 Tier 0](../../../../docs/architecture/target-and-abi.md#1557-tier-0-the-vm),
   [§15.7.2 The three mechanisms](../../../../docs/architecture/target-and-abi.md#1572-the-three-mechanisms),
   [§15.9 The `verse.target` file](../../../../docs/architecture/target-and-abi.md#159-the-versetarget-description-file).
6. [10 §10.3 Differential testing](../../../../docs/architecture/testing-and-verification.md#103-differential-testing).

## What it does

| Stage | Input | Output | Errors it owns |
|-------|-------|--------|----------------|
| Backends | reachable mono MIR | an artifact for the selected target | target limits that are not capabilities, such as no threads on this target |

Source: [02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts).

| Tier | Crate | Used by |
|------|-------|---------|
| 0 | `verse-bc`: bytecode compiler and interpreter | running a file, `verse test`, `verse repl`, debugging |
| 1 | `verse-cl`: Cranelift | `verse --fast app.vs`, the third and later runs of an unchanged script, `verse build --profile dev` |
| 2 | `verse-llvm`: LLVM, an optional feature | `verse build --release`, embedded and exotic targets |
| W | `verse-wasm`: Wasm emitter | browser, WASI, edge, GUI on web |

Sources: [04 §4.1](../../../../docs/architecture/backends-and-targets.md#41-tiers),
[02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout).

- Artifacts are an executable, a static or shared library with a C ABI, a
  Wasm module or component, or a bytecode bundle `.vsb`
  ([04 §4.4](../../../../docs/architecture/backends-and-targets.md#44-linking-and-artifacts)).
- Build order: VM in M1, Cranelift in M2, Wasm in M3, LLVM in M7
  ([ADR-0008](../../../../docs/decisions/0008-tiered-execution.md)). No code
  exists yet.

## How it works

**Tier 0 is the reference.** Where a compiled tier and the VM disagree, the VM
is presumed right until the spec says otherwise. It is the oracle of the
differential tests and the target of every debug session
([14 §14.1](../../../../docs/architecture/bytecode-and-vm.md#141-role)).
Peak speed and multi-threaded execution are not goals in 0.1.

**Bytecode.** Register-based, with 32-bit little-endian words in the forms
`ABC`, `AI` and `J`. `WIDE` extends operands to 16 bits. Instructions are
typed (`ADD_CHK`, `LDF8`), so there is no run-time type dispatch. Slots hold
raw bits with no tags. Names, types and ownership states come from side
tables ([14 §14.2](../../../../docs/architecture/bytecode-and-vm.md#142-design-rules),
[14 §14.4](../../../../docs/architecture/bytecode-and-vm.md#144-instruction-set-and-encoding)).
Why register-based: fewer instructions per operation than a stack machine
([04 §4.1](../../../../docs/architecture/backends-and-targets.md#tier-0-the-bytecode-vm)).
Dispatch starts as a `match` loop, measured before tail-call dispatch is added.

**Values and calls.** A slot is 8 bytes. An aggregate takes `ceil(size / 8)`
slots in the mono `layout`. The caller builds arguments in a window at the top
of its frame, and the callee's base is the window start. Call records live in
a separate vector per task. Task stacks are segments that never move, so a
borrow is a host address that stays valid
([14 §14.3](../../../../docs/architecture/bytecode-and-vm.md#143-value-representation-and-call-records)).

**Checks.** A `checked_*` plus its `switch` becomes one fused `*_CHK`+J
instruction. Bounds use `ULT i, len`
([14 §14.5](../../../../docs/architecture/bytecode-and-vm.md#145-checked-arithmetic-bounds-and-division)).

**Lazy bytecode generation.** Before the first instruction, the reachability
pre-pass monomorphizes and verifies every reachable instance, and the
capability check runs. A function table entry is `Unresolved`, `Compiled` or
`Compiling`. The first call compiles the verified callee to bytecode; code is
never patched. Protos are cached per instance under `.verse/cache/`
([14 §14.6](../../../../docs/architecture/bytecode-and-vm.md#146-calls-and-lazy-bytecode-generation)).
Why: code generation scales with what runs.

**Unwinding.** Each proto has an exception table built from `unwind bbN`
edges, read only while unwinding, so the success path pays nothing
([14 §14.8](../../../../docs/architecture/bytecode-and-vm.md#148-unwinding-and-cleanup)).
Compiled tiers use `native`, `wasm-eh`, `status-code` or `abort`, chosen by
the target file
([15 §15.7.2](../../../../docs/architecture/target-and-abi.md#1572-the-three-mechanisms)).

**Debugging and replay.** Two loop instances, `run::<false>` and
`run::<true>`, switch when hooks are armed. `MARK` exists only in debug
protos. Breakpoints patch an instruction to `BREAK`
([14 §14.11](../../../../docs/architecture/bytecode-and-vm.md#1411-debug-hooks)).
A recording from any tier replays in the VM at optimization level 0
([14 §14.10](../../../../docs/architecture/bytecode-and-vm.md#1410-record-and-replay)).

**Cranelift** is the default native backend: it compiles quickly, is written
in Rust and embeds as a JIT. It has no 32-bit ARM or 32-bit RISC-V, to be
re-verified in M2. **LLVM** covers Thumb, 32-bit RISC-V and Xtensa, and gives
the best release code. **Wasm** is a direct emitter using `wasm-encoder`,
because there is no Cranelift Wasm backend. It uses no Wasm GC
([04 §4.1](../../../../docs/architecture/backends-and-targets.md#tier-1-and-2-native-code)).

**Native runs from the cache.** The second run of an unchanged script also
builds it with Cranelift in the background. From the third run, the cached
native build runs. `[run] native_after`, `--fast` and `--no-cache` tune it,
and `--debug` always runs in the VM
([04 §4.1](../../../../docs/architecture/backends-and-targets.md#tier-1-and-2-native-code)).
A hot-function JIT is deferred past the first stable release.

**Targets.** A target is data plus a lowering and a platform layer. A new
target in a known backend family is only data
([04 §4.3](../../../../docs/architecture/backends-and-targets.md#43-what-a-target-is)).

## Rules that must hold

- **No backend holds ownership logic or picks an overflow or panic policy**
  ([13 §13.1](../../../../docs/architecture/mir-specification.md#131-purpose-and-relation-to-03)).
- **Same observables on every tier**: output bytes, exit status, panic record,
  `Cleanup` and drop order, task events for a seed, capability calls, `Float`
  bits, collection order. Timing, addresses, memory use and the depth of
  `stack_overflow` may differ
  ([14 §14.12](../../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract)).
- **`Float` is IEEE 754 binary64**, no fast-math, no implicit fused
  multiply-add, one canonical NaN where bits are visible
  ([14 §14.12](../../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract)).
- **The VM builds no user wording.** A panic record carries a kind and facts;
  text comes from 16 §16.6 and `events.toml`
  ([14 §14.2](../../../../docs/architecture/bytecode-and-vm.md#142-design-rules)).
- **MIR order decides which panic comes first**
  ([14 §14.5](../../../../docs/architecture/bytecode-and-vm.md#145-checked-arithmetic-bounds-and-division)).
- **Instrumentation changes nothing observable**: values, `Cleanup` order and
  scheduler choices stay the same with hooks on or off
  ([14 §14.11](../../../../docs/architecture/bytecode-and-vm.md#1411-debug-hooks)).
- **Every reachable instance is verified before the first instruction.**
  Verification after monomorphization never fails mid-run
  ([14 §14.6](../../../../docs/architecture/bytecode-and-vm.md#146-calls-and-lazy-bytecode-generation)).
  What a passing `verse check` promises about a run is the rule of
  [16 §16.6.1](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses).
- **Capabilities are checked before the first instruction.** The `CALLS`
  capability assertion exists only in debug builds of `verse` and adds no exit
  status of its own; a failed assertion is a trap inside the runtime, status 3
  ([14 §14.9](../../../../docs/architecture/bytecode-and-vm.md#149-sys-capability-calls-and-the-capability-check)).
- **A `.vsb` bundle is rejected** on a compiler version or pointer width
  mismatch ([14 §14.1](../../../../docs/architecture/bytecode-and-vm.md#141-role)).
- **The internal ABI may change in any release.** Only the C ABI and the Wasm
  component ABI are stable
  ([15 §15.1](../../../../docs/architecture/target-and-abi.md#151-purpose-and-scope)).
- **Builds are reproducible**: no embedded paths or timestamps
  ([04 §4.4](../../../../docs/architecture/backends-and-targets.md#44-linking-and-artifacts)).

## What depends on it

- Upstream, every backend trusts verified, reachable mono MIR with layout
  attached. See [mir-and-optimization.md](mir-and-optimization.md).
- The VM calls plugins with `CALLF`, through stubs from `verse-ffi`. See
  [native-plugins.md](native-plugins.md).
- All tiers share `verse-rt` for text formatting, hashing, collections and the
  seeded scheduler policy, so the differential test mainly checks code
  generation
  ([14 §14.12](../../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract)).
  See [runtime-and-tasks.md](runtime-and-tasks.md).
- `verse-dbg` depends on the VM's side tables, hooks and replay. See
  [debugger-and-language-server.md](debugger-and-language-server.md).
- `verse-cli` picks the tier per command and flag. See
  [packages-and-cli.md](packages-and-cli.md).
- Changing an opcode changes `.vsb` contents, which are versioned with the
  compiler. Changing an unwind mechanism touches the unwinding matrix of
  [15 §15.12](../../../../docs/architecture/target-and-abi.md#1512-conformance-and-tests).

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - differential tests with `verse difftest`: fast set every commit, full
     set nightly;
   - unit and golden tests;
   - fuzzing of the VM;
   - conformance tests for the observable behaviour;
   - the ABI torture programs, unwinding matrix, exit statuses and layout
     goldens of
     [15 §15.12](../../../../docs/architecture/target-and-abi.md#1512-conformance-and-tests);
   - benchmarks against the targets of
     [14 §14.13](../../../../docs/architecture/bytecode-and-vm.md#1413-performance-targets)
     and [09 §9.1](../../../../docs/architecture/performance.md#91-what-we-measure).
2. Update doc 14 for the VM, 04 for tiers and artifacts, 15 for layout, call
   convention, unwinding or target files.
3. Backends report target limits and V1005, V1007 to V1009, V1011, V1012
   ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
   Panic wording lives in
   [16 §16.6](../../../../docs/reference/diagnostics-catalog.md#166-run-time-panics),
   never in a backend. See [diagnostics.md](diagnostics.md).
4. Use the commit format of
   [CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format). The scope is
   `vm` for `verse-bc` and `targets` for target files. The docs give no
   scope for the Cranelift, LLVM and Wasm backends; the area list has
   `compiler` and `targets`.
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
