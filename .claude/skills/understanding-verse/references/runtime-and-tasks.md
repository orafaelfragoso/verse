# Runtime and tasks

`verse-rt`: memory, tasks and scheduling, panics and unwinding, collection
primitives, the `sys` capability interface, platform layers and exit
statuses.

## Read first

1. [ADR-0007 Task model](../../../../docs/decisions/0007-task-model.md).
2. [05 Runtime](../../../../docs/architecture/runtime.md), in order:
   [§5.1 Feature slices](../../../../docs/architecture/runtime.md#51-feature-slices),
   [§5.2 Memory](../../../../docs/architecture/runtime.md#52-memory),
   [§5.3 Tasks and scheduling](../../../../docs/architecture/runtime.md#53-tasks-and-scheduling),
   [§5.4 Panics and unwinding](../../../../docs/architecture/runtime.md#54-panics-and-unwinding),
   [§5.6 The `sys` capability interface](../../../../docs/architecture/runtime.md#56-the-sys-capability-interface),
   [§5.7 Embedded profile](../../../../docs/architecture/runtime.md#57-embedded-profile).
3. Spec [§11.1 Spawning tasks](../../../../docs/language/11-concurrency.md#111-spawning-tasks),
   [§11.3 Structured lifetimes](../../../../docs/language/11-concurrency.md#113-structured-lifetimes),
   [§11.5 Cancellation and time](../../../../docs/language/11-concurrency.md#115-cancellation-and-time).
4. [14 §14.7 Tasks, scheduler and seed](../../../../docs/architecture/bytecode-and-vm.md#147-tasks-scheduler-and-seed)
   and [§14.8 Unwinding and cleanup](../../../../docs/architecture/bytecode-and-vm.md#148-unwinding-and-cleanup).
5. [15 §15.6 Tasks across the ABI](../../../../docs/architecture/target-and-abi.md#156-tasks-across-the-abi),
   [§15.7 Unwinding and abort ABI](../../../../docs/architecture/target-and-abi.md#157-unwinding-and-abort-abi),
   [§15.8 Exit codes](../../../../docs/architecture/target-and-abi.md#158-exit-codes),
   [§15.10 Platform layers](../../../../docs/architecture/target-and-abi.md#1510-platform-layers-and-capability-mapping).
6. [15 §15.3.7 Handles](../../../../docs/architecture/target-and-abi.md#1537-handles-tasks-channels-shared-guarded)
   and [§15.4.2 Calls into the runtime](../../../../docs/architecture/target-and-abi.md#1542-calls-into-the-runtime).
7. [06 §6.5 Capabilities and runtime slices](../../../../docs/architecture/stdlib-and-shaking.md#65-capabilities-and-runtime-slices).

## What it does

- `verse-rt` is a Rust library compiled per target, split into feature
  slices. It has no garbage collector
  ([05](../../../../docs/architecture/runtime.md#05-runtime),
  [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).
- Slices: `core` (allocator, drop and panic machinery, collection primitives,
  checked arithmetic helpers), `unwind`, `tasks`, `debug`, and one `sys-*`
  slice per capability
  ([05 §5.1](../../../../docs/architecture/runtime.md#51-feature-slices)).
- A target names one platform layer: `std-os`, `std-wasi`, `std-web` or
  `std-embedded`. A layer is a menu of slices, and a build links only the
  slices reachability asks for
  ([15 §15.10](../../../../docs/architecture/target-and-abi.md#1510-platform-layers-and-capability-mapping)).
- Inputs: compiled code calls it through the runtime ABI, Rust
  `extern "C-unwind"` functions in the platform C convention
  ([15 §15.4.2](../../../../docs/architecture/target-and-abi.md#1542-calls-into-the-runtime)).
- `verse-rt` reports the run-time panics `R0xxx`
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
- The stackful scheduler, work stealing, channels, `Shared`, `Guarded` and
  cancellation are M4. The `core` slice starts in M1
  ([roadmap](../../../../docs/roadmap.md#11-roadmap)). No code exists yet.

## How it works

**Memory.** `std-os` uses a size-class allocator with thread-local caches,
starting with the `mimalloc` crate behind an interface. `std-embedded` uses a
fixed-region bump or TLSF allocator. The compiler inserts every drop; the
runtime only provides `alloc`, `realloc` and `dealloc`. `Shared<T>` is
reference counted, atomic only when the program can spawn. Cycles leak, and
`--leak-check` reports them in debug builds
([05 §5.2](../../../../docs/architecture/runtime.md#52-memory)).

**One task semantics, several implementations.** Compiled code has three
lowerings. Tier 0 has VM coroutines as a fourth implementation that is not a
lowering ([ADR-0007](../../../../docs/decisions/0007-task-model.md)).

| Strategy | Targets | Mechanism |
|----------|---------|-----------|
| stackful | native | own stack per task, lazily committed, guard page; assembly context switch; work-stealing scheduler, one worker per core; reactor on `epoll`, `kqueue` or IOCP |
| state machine | Wasm without stack switching | `task-lower` turns may-yield functions into resumable `F_State` plus `F_poll` |
| cooperative | microcontrollers | the same state machines on a single-core executor driven by interrupts and a timer |
| VM coroutines | tier 0 | stack segments and call records per task, switched by the interpreter loop, one worker thread |

Sources: [05 §5.3](../../../../docs/architecture/runtime.md#53-tasks-and-scheduling),
[15 §15.6.2](../../../../docs/architecture/target-and-abi.md#1562-resumable-functions-state-machine-and-cooperative-targets).
Why: the spec forbids `async` and function coloring, OS threads cannot reach
1 million tasks, and continuation passing slows ordinary code
([ADR-0007](../../../../docs/decisions/0007-task-model.md)).

**Seeded scheduling.** With `--seed N`, each switch point picks
`rng.below(len)` among runnable tasks sorted by id, with PCG-XSH-RR 64/32
seeded through SplitMix64. `verse-rt` implements the same policy in a
single-worker seeded mode. The multi-core scheduler runs only when no seed or
recording is requested
([14 §14.7](../../../../docs/architecture/bytecode-and-vm.md#147-tasks-scheduler-and-seed)).

**Deep recursion.** Each task counts the calls of `recursive` functions it has
in progress. The call past `--max-calls` (10,000 on every target) panics with
R0004, at the same call in the VM and in native code unless a native stack
fills first (it can on `cooperative` targets, small Wasm shadow stacks, and
hosted targets with frames over about 830 bytes). The stack is the
backstop: the prologue of each non-leaf function compares the stack pointer
with `stack_limit` in the TCB, set `headroom` above the real end, so cleanup
blocks run in the reserve, and a full stack prints the second form of R0004. The
guard page is the last backstop: it prints that second form and exits with
101, running no cleanup
([15 §15.6.4](../../../../docs/architecture/target-and-abi.md#1564-stack-bounds-and-overflow)).

**Panics.** `rt_panic(kind, site, a, b)` builds the panic record, prints the
report at once, sets the unwind state to `panic` and starts unwinding.
Cancellation has state `cancel`, no record, and prints nothing
([15 §15.7.1](../../../../docs/architecture/target-and-abi.md#1571-the-panic-record-and-the-unwind-state)).
Unwinding is `native`, `wasm-eh`, `status-code` or `abort` per target
([15 §15.7.2](../../../../docs/architecture/target-and-abi.md#1572-the-three-mechanisms)).

**Collections.** The runtime implements the primitives: the `Text` header,
growth, hashing with SipHash-1-3, UTF-8 validation. The rest of the
collections API is written in Verse, so it is monomorphized and inlined,
except an instance that is `recursive`
([05 §5.5](../../../../docs/architecture/runtime.md#55-strings-and-collections)).

**Capabilities.** All platform access goes through `sys` capabilities. Each
provider sits behind a Rust trait per capability, so tests and replay
substitute fakes such as a virtual clock
([05 §5.6](../../../../docs/architecture/runtime.md#56-the-sys-capability-interface),
[15 §15.10.3](../../../../docs/architecture/target-and-abi.md#15103-the-capability-table)).
Why: ports stay small, reachability gets exact, denying is natural.

**`--no-heap`.** Tasks and channels live in static memory. A channel capacity
must be a literal (V0916). A `spawn` site must run a bounded number of times,
counted on the call graph, or it is V0917
([15 §15.3.7](../../../../docs/architecture/target-and-abi.md#1537-handles-tasks-channels-shared-guarded),
[05 §5.7](../../../../docs/architecture/runtime.md#57-embedded-profile)).

## Rules that must hold

- **The same semantics on every strategy**: arguments are moved into a task
  before it starts; a panic becomes `TaskPanicked` at `.wait()`; cancellation
  unwinds like a panic and drops what the task owns; unwaited failed tasks set the exit
  status ([05 §5.3](../../../../docs/architecture/runtime.md#53-tasks-and-scheduling)).
- **Functions outside the may-yield set pay nothing** on any target
  ([ADR-0007](../../../../docs/decisions/0007-task-model.md)).
- **A drop never panics and never parks.** Drop glue runs no user code, so a
  cleanup block has no switch point: an unwinding task cannot be interrupted,
  a cancel request then changes nothing, and a task has at most one panic
  ([15 §15.7.4](../../../../docs/architecture/target-and-abi.md#1574-drops-while-unwinding),
  [14 §14.8](../../../../docs/architecture/bytecode-and-vm.md#148-unwinding-and-cleanup)).
- **The runtime owns every catch**, and there are exactly two: the task
  boundary and the program entry. A foreign frame is never
  unwound through
  ([15 §15.7.5](../../../../docs/architecture/target-and-abi.md#1575-catch-points)).
- **Intrinsics cannot unwind or yield.** A runtime function that can fail
  returns a status, and MIR already has the panic branch
  ([15 §15.4.2](../../../../docs/architecture/target-and-abi.md#1542-calls-into-the-runtime)).
- **Deadlock is one predicate on every tier**: no runnable task, no pending
  completion, no timer, no armed interrupt source, and blocked tasks exist.
  It raises R0008 and exits with 70
  ([15 §15.8.1](../../../../docs/architecture/target-and-abi.md#1581-statuses)).
- **Exit statuses and their precedence** are fixed: `exit(code)`, then 70,
  then 101, then 1, then 0. `exit(code)` drops nothing, and a code outside
  0 to 255 is the panic R0013
  ([15 §15.8.1](../../../../docs/architecture/target-and-abi.md#1581-statuses)).
- **`abort` only when no platform handle is live** at any panic site (V0908), and
  never with a reachable `spawn` (V0910)
  ([15 §15.7.6](../../../../docs/architecture/target-and-abi.md#1576-the-abort-profile-and-unreachable)).
- **Interrupts only wake tasks.** No Verse code runs in interrupt context
  ([05 §5.7](../../../../docs/architecture/runtime.md#57-embedded-profile)).
- **Wording comes from `events.toml`**, never from the ABI
  ([15 §15.7.1](../../../../docs/architecture/target-and-abi.md#1571-the-panic-record-and-the-unwind-state)).

## What depends on it

- Upstream, the runtime trusts MIR to hold every drop, unwind edge and yield,
  and the `layout` pass to fix header sizes. See
  [mir-and-optimization.md](mir-and-optimization.md).
- Reachability picks the slices: `tasks` when a `spawn` is reachable,
  `unwind` when a reachable unwind edge runs a drop on a `native` target
  ([06 §6.5](../../../../docs/architecture/stdlib-and-shaking.md#65-capabilities-and-runtime-slices)).
- Every tier links it. The VM shares its formatting, hashing, collections and
  seeded scheduler policy, so a change here shows in the differential tests.
  See [vm-and-backends.md](vm-and-backends.md).
- The standard library reaches the platform only through capability
  intrinsics. See [standard-library.md](standard-library.md).
- The `debug` slice records runs for replay. See
  [debugger-and-language-server.md](debugger-and-language-server.md).
- A change to a header layout is an ABI level A change, shared with every
  tier ([15 §15.1](../../../../docs/architecture/target-and-abi.md#151-purpose-and-scope)).

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - conformance tests for spec §11 task behaviour;
   - differential task tests with a seeded scheduler, against the VM
     ([ADR-0007](../../../../docs/decisions/0007-task-model.md));
   - the safety runs of
     [10 §10.4](../../../../docs/architecture/testing-and-verification.md#104-safety-properties-we-test-explicitly):
     poisoning debug allocator, Valgrind and sanitizers for the runtime crate,
     ThreadSanitizer, panic and drop order;
   - the exit status table and the unwinding matrix of
     [15 §15.12](../../../../docs/architecture/target-and-abi.md#1512-conformance-and-tests);
   - benchmarks for spawn, channels and memory per task
     ([09 §9.1](../../../../docs/architecture/performance.md#91-what-we-measure)).
2. Update doc 05, and doc 15 for the ABI, unwinding, exit codes or platform
   layers. A scheduler policy change also updates 14 §14.7.
3. A new run-time panic gets an `R` code in
   [16 §16.6](../../../../docs/reference/diagnostics-catalog.md#166-run-time-panics),
   never reused, with one sentence of at most 15 words that reads the same on
   every tier. See [diagnostics.md](diagnostics.md).
4. Use the commit format of
   [CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format) with the scope
   `runtime`, for example `fix(runtime): ...`.
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
