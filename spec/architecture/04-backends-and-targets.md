# 04. Execution tiers, backends and targets

## 4.1 Tiers

| Tier | Name | Used by | Compile cost | Run speed | Introspection |
|------|------|---------|--------------|-----------|---------------|
| 0 | **Bytecode VM** | `verse run`, `verse test`, `verse repl`, debugging | lowest: lazy, per function | baseline | full: every local, every step |
| 1 | **Cranelift** | `verse run --fast`, hot-function JIT (later), `verse build --profile dev` | low | typically within 2x of optimized native (assumption) | DWARF plus Verse debug map |
| 2 | **LLVM** | `verse build --release`, embedded and exotic targets | high | best | DWARF plus Verse debug map |
| W | **Wasm emitter** | browser, WASI, edge, plugins, GUI on web | low | engine dependent | DWARF in `.debug_*`, source maps |

All tiers consume the same **mono MIR** ([ADR-0003](adr/0003-one-mir-many-backends.md)).
That is how requirement R1 is met: scripting and compiling are the same
compiler with a different last stage, so they cannot drift apart semantically.

### Tier 0: the bytecode VM

- **Register-based** bytecode (fewer instructions per operation than a stack
  machine; the Lua 5 and Dalvik experience), with 16-bit register operands and
  fixed-width 32-bit instruction words where possible.
- **Input is mono MIR**, so the VM contains no ownership logic, no generics
  and no trait resolution. It executes moves, drops, checked arithmetic and
  calls.
- **Typed instructions** (`ADD_I64_CHK`, `LOAD_FIELD_8`) so there is no run-time
  type dispatch.
- **Direct-threaded dispatch** where the host supports computed goto, with a
  `match` fallback. On Rust stable we start with a `match` loop in a hot
  function and measure before adding tail-call dispatch.
- **Debug mode** adds a `MARK` instruction per source line (derived from MIR
  spans; MIR has no mark statement); release mode emits none. Breakpoints patch
  an instruction to `BREAK`. The full instruction set, frame layout and task
  scheduler are in [14](14-bytecode-and-vm.md).
- **Cache.** Bytecode is cached under `.verse/cache/` so a re-run of an
  unchanged script starts without parsing. The cache key is per **function
  instance** (`DefId` plus type arguments, [02 §2.6](02-compiler-architecture.md#26-monomorphization)), because generic
  instances cross module boundaries; instances are grouped per module on disk
  for storage, and a module's file indexes its instances.
- **Startup target**: hello world in under 20 ms wall time, including process
  start (target; measured in M1).

### Tier 1 and 2: native code

- **Cranelift** is the default native backend because it compiles quickly,
  is written in Rust, and is designed to be embedded (also as a JIT). It
  supports x86-64, AArch64, RISC-V 64 and s390x. It does **not** support
  32-bit ARM or 32-bit RISC-V (fact, as of its current documentation; we
  verify this again in M2 because it affects the embedded plan).
- **LLVM** is the release and embedded backend. It is an optional Cargo
  feature of the toolchain, because it is large and slows toolchain builds. It
  covers everything Cranelift does not: Thumb (Cortex-M), 32-bit RISC-V
  (for example ESP32-C3), Xtensa through the vendor fork, and the strongest
  optimizer for release builds.
- **No cross-backend divergence in semantics.** Overflow checks, bounds checks
  and panics are already in MIR as explicit control flow, so neither backend
  can quietly choose a different policy.
- **Hot-function JIT** (Tier 0 to Tier 1 inside `verse run`) is deferred past
  the first stable release. It needs the call-count and type feedback that
  Verse mostly does not need, because the program is statically typed; the
  simpler path is to compile the whole script natively behind the cache when it
  is run twice (`--fast` as default after the second run is a proposal, not a
  decision).

### Tier W: WebAssembly

- **A direct emitter** (using `wasm-encoder`), not a Cranelift backend: there
  is no Cranelift Wasm backend, and a direct emitter keeps output small
  (assumption: MIR to Wasm is a near one-to-one structured-control-flow
  conversion using the relooper algorithm, as Binaryen and Emscripten do).
- **Targets**: `wasm32-wasip1`/`wasip2` (command and component),
  `wasm32-unknown` (browser with a JavaScript glue file). Memory is linear
  memory managed by the Verse allocator. There is **no use of Wasm GC**: Verse
  has no collector, and ownership gives deterministic frees.
- **Exceptions for unwinding**: use Wasm exception handling where the engine
  supports it, otherwise propagate a status code after each call that can
  unwind (the MIR already has the unwind edges).
- **Tasks**: see [05](05-runtime.md), section 5.3. Wasm has no stack
  switching in all engines, so the default is a state-machine transform.

## 4.2 Target matrix

The columns say how each target is reached and what the runtime provides.
"Planned" means a milestone owns it; nothing here is implemented yet.

| Target | Triple (example) | Backend | Runtime slice | Threads | Tasks | Milestone |
|--------|------------------|---------|---------------|---------|-------|-----------|
| Linux x86-64 | `x86_64-unknown-linux-gnu`, `-musl` | Cranelift, LLVM | `std-os` | yes | stackful, work-stealing | M2 |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Cranelift, LLVM | `std-os` | yes | stackful, work-stealing | M2 |
| macOS x86-64 and ARM64 | `*-apple-darwin` | Cranelift, LLVM | `std-os` | yes | stackful | M2 |
| Windows x86-64 and ARM64 | `*-pc-windows-msvc` | Cranelift, LLVM | `std-os` | yes | stackful (fibers) | M5 |
| WASI | `wasm32-wasip1`, `wasip2` | Wasm emitter | `std-wasi` | no (single thread first) | state machines | M3 |
| Browser | `wasm32-unknown-unknown` | Wasm emitter | `std-web` | no (workers later) | state machines | M3 |
| RISC-V 64 Linux | `riscv64gc-unknown-linux-gnu` | Cranelift, LLVM | `std-os` | yes | stackful | M7 |
| Cortex-M | `thumbv7em-none-eabihf` | LLVM | `std-embedded` | no | cooperative executor | M7 |
| ESP32 (RISC-V) | `riscv32imc-unknown-none-elf` | LLVM | `std-embedded` | no | cooperative executor | M7 |
| ESP32 (Xtensa) | `xtensa-esp32-none-elf` | LLVM (vendor fork) | `std-embedded` | no | cooperative executor | later |
| Bare metal x86-64 | `x86_64-unknown-none` | LLVM | `core` only | no | none | later |

## 4.3 What a target is

A target is data plus two small implementations. This is the whole interface
that adding a target must not exceed (requirement R2).

```toml
# targets/thumbv7em-none-eabihf.toml
[target]
triple          = "thumbv7em-none-eabihf"
pointer_width   = 32
endian          = "little"
backend         = "llvm"
unwind          = "status-code"      # status-code | native | wasm-eh | abort
tasks           = "cooperative"      # stackful | state-machine | cooperative
atomics         = "native"           # native | none (single-core, interrupts masked)
float           = "hard"
runtime_slice   = "std-embedded"
object_format   = "elf"
linker          = "rust-lld"

[capabilities]                       # what this target can provide to `sys`
clock = true
random = "hardware"
fs = false
net = false
gpio = true
uart = true
```

1. **Lowering**: a function from mono MIR to the backend. For existing
   backends (Cranelift, LLVM, Wasm) a new target in the same family is only
   data.
2. **Platform layer**: the implementation of the `sys` capability interface
   ([05](05-runtime.md), section 5.6) for that target.

If `[capabilities]` lacks one the program uses, the build fails with a plain
message: "this program calls `std.net.get`, which `thumbv7em-none-eabihf`
cannot provide" (V0902). The **capability check** that reports it runs right
after reachability ([02 §2.2](02-compiler-architecture.md#22-stage-contracts)), and in script mode before tier 0 starts.

## 4.4 Linking and artifacts

- **Executable** (`verse build`): object code for reachable functions plus a
  prebuilt runtime slice for the target, linked with `lld` (bundled) and
  section garbage collection.
- **Static or shared library** with a C ABI for exported items (foreign interfaces
  are specified in the 0.2 `extern` section, [12](12-open-questions.md) Q2).
- **Wasm module or component** with WIT generated from `export` items.
- **Bytecode bundle** (`.vsb`): a single file the VM runs. Useful for shipping
  a script without recompiling. Versioned with the compiler version.
- **Reproducible builds**: no embedded paths or timestamps; builds from the
  same `verse.lock` produce identical bytes.

**Wasm-implemented packages.** A package shipped as a Wasm component (an ordinary dependency, not a plugin) is translated to mono MIR before linking on every target except Wasm, where it is linked as a component. No engine is embedded in a native binary ([07 §7.4](07-plugins.md)).

## 4.5 Calling convention and ABI

- **Internal ABI**: free to change between compiler versions. Verse-to-Verse
  calls use a compiler-chosen convention: small values in registers, large
  values by hidden pointer, `mut` and borrowed arguments by pointer.
- **Stable ABI**: only the C ABI for explicit `extern` exports and imports
  (0.2, [12](12-open-questions.md) Q2) and the Wasm component ABI for components.
- **Interop** with C, Rust and Wasm components goes through generated
  bindings (`verse bindgen`, a plugin type: [07](07-plugins.md)).
