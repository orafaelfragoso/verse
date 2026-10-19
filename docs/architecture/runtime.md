# 05. Runtime

The runtime is a Rust library, `verse-rt`, compiled per target. It has no
garbage collector, because the language has none (spec §4.10).

## 5.1 Feature slices

The runtime is not one blob. It is built as **slices**, and the linker gets only
the ones the reachability analysis asks for ([06](stdlib-and-shaking.md)).

| Slice | Provides | Needs OS? | Depends on |
|-------|----------|-----------|------------|
| `core` | allocator, drop and panic machinery (including the panic report: the stack walk, the trace map and folding, [15 §15.7.1](target-and-abi.md#1571-the-panic-record-and-the-unwind-state)), `Text`, `List`, `Map`, `Set` primitives, checked arithmetic helpers, formatting of scalars | no | none |
| `unwind` | landing pads and cleanup running on panic and cancellation | no | `core` |
| `tasks` | scheduler, `Task`, the `Sender` and `Receiver` made by `Channel.new<T>()`, `Shared`, `Guarded`, timers | scheduler primitives only | `core` |
| `sys-stdio` | `print`, `input`, error reports of a failed top-level `?` | yes, per target | `core` |
| `sys-clock`, `sys-random`, `sys-fs`, `sys-net`, `sys-process`, `sys-os`, `sys-env` | the platform capabilities of `time`, `random`, `fs`, `net` and `http`, `process`, `os`, `env` | yes, per target | `core` |
| `sys-gpio`, `sys-uart`, `sys-i2c`, `sys-spi` | microcontroller peripherals | board | `core` |
| `debug` | trace events, record and replay hooks, the `values` and `last_events` of a panic report | no | `core` |
| `sys-surface`, `sys-gpu` | windows, input and drawing; GPU compute (capabilities `surface` and `gpu`, [06 §6.5](stdlib-and-shaking.md#65-capabilities-and-runtime-slices)) | yes, per target | `core` |

**How platforms and slices combine.** Each target names one *platform layer* (`std-os`, `std-wasi`, `std-web`, `std-embedded`, see [04 §4.2](backends-and-targets.md#42-target-matrix)) that implements the allocator, the entry point and the `sys-*` capabilities that target provides. The feature slices above are the pieces of that layer, so a platform layer is a *menu* of slices and a build links only the slices reachability asks for, never the whole layer.

A hello-world links `core` and `sys-stdio` only (a single capability that
holds `print` and `input`). That is a requirement of R6, and it is tested ([09](performance.md)).

## 5.2 Memory

- **Allocator.** A size-class allocator with thread-local caches for the
  `std-os` slice (a design similar to mimalloc and snmalloc; we begin with the
  `mimalloc` crate behind an interface and replace it only if measurements
  justify it). `std-embedded` uses a fixed-region bump or TLSF allocator. Wasm
  uses a small allocator tuned for single-threaded use.
- **Frees are deterministic.** The compiler inserts drops (MIR `Drop`); the
  runtime provides only `alloc`, `realloc`, `dealloc`.
- **`Shared<T>`** is reference counted. Counts are atomic when the program can
  spawn tasks and non-atomic when whole-program analysis proves it cannot
  (`refcount-opt`, [03](ir-and-passes.md)). `Weak` uses a weak count.
- **Cycles** through `Shared` and `Guarded` leak, as the spec says (§4.10). In
  debug builds the runtime can run a leak report at exit
  (`verse --leak-check app.vs`), which finds leaked cycles by scanning live
  allocations from roots. This is a diagnostic, not collection.
- **Arenas** (index-based, in user code) need no runtime support.
- **Memory limits.** `verse --max-memory <size> app.vs` is enforced by the allocator
  hook. Out of memory is a panic, with a report of the largest allocations in
  debug mode.

## 5.3 Tasks and scheduling

Spec §11.1: lightweight tasks, many cores, no `async`/`await`, no function
coloring, blocking calls just work. The mechanism depends on the target, but
the observable behavior is identical ([ADR-0007](../decisions/0007-task-model.md)).
Compiled code has **three lowerings** (stackful, state machines, cooperative
executor); tier 0 has its own **VM coroutine tasks**, described last.

**Native targets (`stackful`)**

- Each task has its own stack: a lazily committed region with a guard page.
  The reservation is the target's `[tasks] max_stack`, 8 MiB of address space
  per task by default; physical memory is committed only as touched, so the
  reservation costs nothing until a task uses it.
- Context switch is a few register saves in assembly (the approach of Go,
  Boost.Context and Zig's old async design).
- **Work-stealing scheduler**, one worker thread per core, local run queues
  and a global injector queue (the Tokio and Go design).
- **I/O**: a reactor per worker or shared: `epoll` on Linux, `kqueue` on
  macOS, IOCP on Windows, with `io_uring` as a later optimization on Linux. A
  blocking call such as `get` registers interest and yields. File I/O that has
  no async primitive runs on a small blocking thread pool.
- **Deep recursion** stops at the **call limit**: each task counts the calls
  of `recursive` functions it has in progress, and the call that would pass
  `--max-calls` (10,000 by default, on every target) raises the panic
  `stack_overflow` (R0004, "recursion is too deep in task {n}: {limit} calls
  in progress, the limit, in `{f}`", [16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics)).
  The VM counts the same calls, so a native task whose stack does not fill
  first stops at the same call
  ([ADR-0012](../decisions/0012-one-call-limit-and-trace.md)).
- **The stack reservation is a backstop.** An explicit check in the prologue
  of each non-leaf function raises the second form of R0004, "task {n} ran out
  of stack in `{f}`", when the stack fills before the call limit. A fault
  cannot run cleanup, because cleanup needs stack that is exhausted, so the
  check fires `headroom` above the real end and `Cleanup` runs in the reserve
  ([15 §15.6.4](target-and-abi.md#1564-stack-bounds-and-overflow)). The
  **guard page** stays only as a backstop for code that does not check (a
  runtime function, foreign code): a fault there prints the same second form
  and ends the program with status 101, running no cleanup. Deep recursion is
  a bug and must not corrupt memory.

**State-machine targets (Wasm without stack switching)**

- The `task-lower` pass computes a **may-yield** set: functions that can
  reach a scheduler call (`wait`, channel receive, `sleep`, I/O). This is a
  compiler-internal effect analysis. **It never appears in the language**, so
  there is no function coloring for users.
- Functions in the may-yield set are transformed into resumable state
  machines (the way C# and Rust lower `async`, and Binaryen's Asyncify does
  at the Wasm level). Functions outside the set are untouched and pay
  nothing.
- Where the engine supports the Wasm stack-switching proposal, the stackful
  strategy is used instead.

**Cooperative targets (microcontrollers)**

- A single-core executor with a run queue, driven by interrupts and a timer.
  Same state-machine lowering. No threads, no atomics needed.

**Tier 0 (bytecode VM)**

- Tasks are VM coroutines, not a lowering: each task has a chain of stack
  segments and a vector of call records, and the interpreter loop switches
  between them. The loop never recurses, so no native stack switching is
  needed. One worker thread in 0.1.
- Switch points are only the MIR `yield` terminators that cannot proceed, a
  `DROPC` whose `cleanup` blocks, an explicit `pause`, and task exit. The scheduler policy with and without
  `--seed` is specified in [14 §14.7](bytecode-and-vm.md#147-tasks-scheduler-and-seed);
  the seeded single-worker mode of `verse-rt` implements the same policy so
  that differential tests compare the same task events.

**Semantics shared by all strategies** (checked by the conformance suite):

- A spawned call's arguments are evaluated and moved into the task before it
  starts (§11.1).
- A panic in a task becomes `TaskPanicked` at `.wait()`.
- Cancellation unwinds the task like a panic, running `Cleanup` (§11.5).
- Unwaited failed tasks set the exit status (§12.5).
- `exit(code)` ends the program **immediately**, from any task. It runs no
  `Cleanup`, does not cancel the other tasks, never raises R0009, and flushes
  standard output and error. A code outside 0 to 255 is a panic, R0013, on every
  platform. A program that needs its cleanups returns from `main` or ends with a
  failed `?` ([15 §15.8.1](target-and-abi.md#1581-statuses)).

## 5.4 Panics and unwinding

Spec: a panic stops the current task, running `Cleanup` for owners in scope,
prints to standard error, and exits with status 101 for the main program
(§8.5). Cancellation unwinds the same way.

MIR has explicit unwind edges. Each backend chooses how to realize them:

| Target family | Mechanism |
|---------------|-----------|
| native with `unwind = "native"` | DWARF unwind tables and a personality routine in `verse-rt` |
| Wasm with exception handling | `try_table` and `throw` |
| Wasm without it, and embedded | the compiler emits a status check after calls that can unwind and branches to the cleanup block (zero cost on the success path except the check) |
| `abort` (opt-in profile) | no cleanup runs; documented as a deviation from the specification. Panics clean up on every target, and `abort` is allowed only when the compiler proves no `Cleanup` value is live at any panic site; otherwise the build fails. An `abort` build also fails with V0910 when a `spawn` is reachable, because a task's panic could not reach its waiter ([15 §15.7.6](target-and-abi.md#1576-the-abort-profile-and-unreachable)) |

Panic output is a structured record first (message, task, source span, stack),
rendered as text for humans and emitted as JSON when `VERSE_PANIC=json` or when
a debugger is attached ([08](debugging.md)).

## 5.5 Strings and collections

- The runtime implements the primitives (`Text` header, growth policy, hashing
  with SipHash-1-3 by default and a keyed fast hash for internal tables,
  UTF-8 validation with SIMD where the target supports it).
- Everything else in the collections API (§9.5) is **written in Verse** on top
  of these primitives, so it is monomorphized and inlined into user code,
  except an instance that is `recursive` ([13 §13.3](mir-specification.md#133-types-and-layout-facts),
  [ADR-0012](../decisions/0012-one-call-limit-and-trace.md)). That is how
  iterator chains reach the performance of a hand-written loop.
- Hash iteration order is insertion order for `Map` (spec §9.2). `Set` also iterates in
  insertion order (spec §9.4).

## 5.6 The `sys` capability interface

All platform access goes through one small interface of **capabilities**.
`fs` calls `sys.fs.*`, never an OS function directly. A target provides the
capabilities it can.

This is a description of the interface, not Verse code.

```
capability fs      { open, read, write, close, stat, list, remove, ... }
capability net     { connect, listen, accept, send, recv, resolve, tls_* }
capability clock   { monotonic_now, wall_now, sleep_until }
capability random  { fill_bytes }
capability stdio   { write_stdout, write_stderr, read_stdin, read_stdin_line }
capability process { spawn, wait, exit }
capability os      { signal, pid, user, group, chmod, syscall }
capability env     { get, vars, args }
capability gpio    { configure, read, write, interrupt }
...
```

Reasons:

1. **Ports are small.** A new OS or board needs the capabilities, not a new
   standard library.
2. **Reachability gets exact.** A program that never reaches `sys.net` links
   none of it, and the build can say which function pulled `net` in.
3. **Denying is natural.** A Wasm host or a test harness can grant or deny
   capabilities (and `verse --deny net app.vs` becomes trivial). Foreign code in
   a plugin does not go through `sys`, so a denial does not reach it
   ([07 §7.1](plugins.md#71-what-a-plugin-is)).
4. **Testing.** Tests substitute fake capabilities (virtual clock, in-memory
   file system) for deterministic tests and for replay ([08](debugging.md)).

## 5.7 Embedded profile

Microcontrollers need explicit rules, because the language's defaults assume a
heap and a 64-bit `Int`.

- Heap is allowed but sized at build time (`[targets.<target>] heap = "32K"` in
  `verse.toml`, see [19](../reference/cli-and-manifest.md)). Out-of-memory is a panic.
- `verse build --no-heap` checks that no reachable function allocates, using
  the MIR (reachability plus allocation intrinsics). Records, tuples, variants,
  `Int`, `Float`, fixed ranges and borrowed `Text` literals work without a heap.
  `List`, `Map`, `Set`, `Text` building and `Shared` do not.
- Tasks and channels work without a heap. A channel's capacity is a literal,
  and its buffer is static. A `spawn` must run a bounded number of times: it
  is not in a loop, not in a recursive function, and not in a function called
  through a function value. The compiler then gives each possible task a
  static slot for its control block, its result and its state (a fixed-size
  stack on a stackful target). Any other `spawn` is a compile error (V0917).
  The exact rule is [15 §15.3.7](target-and-abi.md#1537-handles-tasks-channels-shared-guarded).
- `Int` stays 64-bit (spec §4.1). The optimizer uses range analysis to lower to
  32-bit operations where it can prove values fit, and sized types
  (`math.Int32`) are the explicit tool for hot code.
- **Interrupts only wake tasks.** No Verse code runs in interrupt context. The platform layer owns a tiny handler, written once in the runtime and not in Verse, that records the event (a pin change, a timer tick, a received byte) and marks the waiting task ready. The task then runs on the cooperative executor like any other, so ordinary rules (ownership, `Cleanup`, panics) hold in it. A program waits for an interrupt through the `gpio`, `uart` or `clock` capability, never by registering a function ([20 §20.6](first-party-packages.md#206-iot)).
