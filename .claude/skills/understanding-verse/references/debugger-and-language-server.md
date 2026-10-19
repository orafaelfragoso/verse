# Debugger and language server

The `verse debug` adapter, record and replay, panic reports and `verse lsp`:
`verse-dbg` and `verse-lsp`. Nothing is implemented yet; the docs are the
design.

## Read first

1. [ADR-0006](../../../../docs/decisions/0006-debugging-over-dap.md): why the
   debugger speaks DAP and runs on the VM.
2. [08 Debugging](../../../../docs/architecture/debugging.md), all of it:
   [§8.2 architecture](../../../../docs/architecture/debugging.md#82-architecture),
   [§8.3 the adapter](../../../../docs/architecture/debugging.md#83-the-debug-adapter),
   [§8.4 values and stops](../../../../docs/architecture/debugging.md#84-values-and-stops),
   [§8.5 record and replay](../../../../docs/architecture/debugging.md#85-deterministic-record-and-replay),
   [§8.6 panic reports](../../../../docs/architecture/debugging.md#86-panics-and-crashes-as-first-class-data),
   [§8.7 vocabulary](../../../../docs/architecture/debugging.md#87-vocabulary-and-wording),
   [§8.8 extensions](../../../../docs/architecture/debugging.md#88-verse-extensions),
   [§8.9 safety](../../../../docs/architecture/debugging.md#89-safety).
3. [14 §14.10 Record and replay](../../../../docs/architecture/bytecode-and-vm.md#1410-record-and-replay)
   and [§14.11 Debug hooks](../../../../docs/architecture/bytecode-and-vm.md#1411-debug-hooks):
   the `.vsrec` format and how the VM implements stops.
4. [`events.toml`](../../../../docs/reference/events.toml): the one source of
   event names, value states, stop reasons, `D0xxx` errors and `verse why`
   openers.
5. For the language server:
   [02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)
   (the CST), [02 §2.7](../../../../docs/architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)
   (queries), [02 §2.8](../../../../docs/architecture/compiler-architecture.md#28-diagnostics)
   (the diagnostic record) and
   [spec §16.5 Lints](../../../../docs/language/16-tooling.md#165-lints).

## What it does

| Crate | Owns | Source |
|-------|------|--------|
| `verse-dbg` | debugger core, record and replay, DAP adapter | [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout) |
| `verse-lsp` | language server | [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout) |

**Debugger.** A DAP client (editor, terminal, script) talks to `verse debug`.
The adapter drives the `verse-dbg` core. The core runs the program in the
bytecode VM, or replays a `.vsrec` recording in the VM
([08 §8.2](../../../../docs/architecture/debugging.md#82-architecture)).

- Inputs: DAP requests; a `.vs` file or package folder; a recording made by
  any tier with `--record` or `VERSE_RECORD=file`.
- Outputs: DAP responses and events. Values carry ownership state. Stops carry
  a Verse stop reason. Panics produce a panic report
  ([08 §8.4](../../../../docs/architecture/debugging.md#84-values-and-stops),
  [§8.6](../../../../docs/architecture/debugging.md#86-panics-and-crashes-as-first-class-data)).
- `verse debug` serves DAP on standard input and output. `--port n` serves it
  on a local port, one session per connection
  ([08 §8.3](../../../../docs/architecture/debugging.md#83-the-debug-adapter)).
- `attach` takes `pid` and works only for VM programs started with
  `verse --debug app.vs`. A compiled program is debugged by replay.

**Panic reports.** Any native or VM run writes one on a panic, with no
debugger attached. `VERSE_PANIC=json` prints it as JSON to standard error
([08 §8.6](../../../../docs/architecture/debugging.md#86-panics-and-crashes-as-first-class-data)).

**Language server.** `verse lsp` shows inferred types and ownership state
inline. It uses the same compiler queries as the compiler, not DAP
([08 §8.8](../../../../docs/architecture/debugging.md#88-verse-extensions)).
It reads the lossless CST, a green tree that is cheap to re-parse after an
edit ([02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)).
It consumes the same diagnostic record as the terminal and `verse check --json`
([02 §2.8](../../../../docs/architecture/compiler-architecture.md#28-diagnostics)),
and lints appear in it
([spec §16.5](../../../../docs/language/16-tooling.md#165-lints)).

## How it works

**Everything debuggable runs in the VM.** The VM is the semantic reference
([ADR-0008](../../../../docs/decisions/0008-tiered-execution.md)) and can expose
every local and every step cheaply. Native debuggers still work through DWARF
and a Verse debug map, as a secondary path
([08 §8.2](../../../../docs/architecture/debugging.md#82-architecture)).

**Compiled programs are debugged by replay.** The runtime logs only
nondeterministic inputs: capability results, scheduler choices, lock order,
random seeds and signals. Replaying the log in the VM reproduces the run.
This works because script and compiled modes behave identically and all
outside access goes through capabilities
([05 §5.6](../../../../docs/architecture/runtime.md#56-the-sys-capability-interface)).

**Replay.** The VM recompiles from the same source at optimization level 0. A
log-driven scheduler consumes `Pick` and `Wake` events. Each capability call
consumes a `Cap` event and returns the logged bytes with no real I/O. Each
output write is checked against `Out`. A mismatch stops with a divergence
report and never guesses
([14 §14.10](../../../../docs/architecture/bytecode-and-vm.md#1410-record-and-replay)).

**Steps.** A step is one MIR statement or terminator of mono MIR at level 0.
Steps exist only in the VM. A recording is indexed by event ordinal, so a
compiled tier needs no step counter. `verse/goto` with a `step` works on
recordings replayed in the VM; with an `event` it works everywhere
([08 §8.5](../../../../docs/architecture/debugging.md#85-deterministic-record-and-replay)).

**Time travel.** Snapshots every K steps, then forward re-execution from the
nearest one, the approach of rr and Pernosco scaled down. `stepBack` and
`reverseContinue` are standard DAP. A backward step costs at most one snapshot
interval ([14 §14.10](../../../../docs/architecture/bytecode-and-vm.md#1410-record-and-replay)).

**Hooks.** The VM has two dispatch loops, `run::<false>` and `run::<true>`, and
switches at an instruction boundary when hooks are armed. Breakpoints patch
the first word of an instruction with `BREAK`. `MARK` is emitted only for
debug builds. Disabled hooks cost nothing
([14 §14.11](../../../../docs/architecture/bytecode-and-vm.md#1411-debug-hooks)).

**Verse extensions.** A `verse` field on `Variable`, on the `stopped` event and
on the `exceptionInfo` response. Two requests: `verse/goto` and
`verse/history`. A client that ignores them still has a full debugger
([08 §8.8](../../../../docs/architecture/debugging.md#88-verse-extensions)).

**One vocabulary.** Debug events and compiler diagnostics use the same names
and wording. Events are past-tense `snake_case` verbs. Value states are exactly
the place states of the ownership pass
([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass),
[08 §8.7](../../../../docs/architecture/debugging.md#87-vocabulary-and-wording)).

## Rules that must hold

- The adapter never changes a program's state and never writes its files.
  `goto`, `setVariable`, `setExpression` and `restartFrame` are not supported,
  because they would break replay
  ([08 §8.3](../../../../docs/architecture/debugging.md#83-the-debug-adapter),
  [§8.9](../../../../docs/architecture/debugging.md#89-safety)).
- `evaluate`, breakpoint conditions and log points accept only expressions the
  compiler proves pure: no capability calls, no spawning.
- Capabilities in a debugged program are the same as in a normal run.
  `--deny` and the `deny` launch argument apply.
- `--port` listens on loopback only. Recordings are owner-only files.
  `verse record redact` removes chosen capability results.
- The program's output reaches the client as `output` events, never on the
  adapter's standard output.
- `allThreadsStopped` is always `true`. A stop halts every task.
- Nothing is cut silently. A value never claims more than the runtime knows:
  an unrecorded field is `unknown`
  ([08 §8.7](../../../../docs/architecture/debugging.md#87-vocabulary-and-wording)).
- `events.toml` is the single source of wording. The compiler fails to build if
  an event has no entry. No text uses a word from the "don't say" table.
- A stop summary is one sentence of at most 25 words, present tense, with a
  position and a cause. It does not say "paused" or "you".
- Debugger errors use `D0xxx` codes, never reused, listed in `events.toml`.
- Instrumentation must not change values, `Cleanup` order or scheduler
  choices ([14 §14.11](../../../../docs/architecture/bytecode-and-vm.md#1411-debug-hooks)).
- Replay depends on deterministic `Set` order, fixed hash seeds in record mode,
  no behavior that depends on addresses, and bit-identical floats
  ([08 §8.5](../../../../docs/architecture/debugging.md#85-deterministic-record-and-replay)).

Not decided in the docs:

- The snapshot interval K is an assumption to tune
  ([14 §14.10](../../../../docs/architecture/bytecode-and-vm.md#1410-record-and-replay)).
- The `.vsrec` format is internal and unversioned for third parties.
- The size of an event record is a target: under 100 bytes before compression.
- The language server has no design document. The docs do not list its LSP
  requests, features or protocol version. Its delivery is milestone M6, with a
  protocol test suite as the exit criterion
  ([roadmap](../../../../docs/roadmap.md)).

## What depends on it

Upstream inputs it trusts:

- The bytecode VM and its hooks, `MARK` and the event table
  ([14 §14.11](../../../../docs/architecture/bytecode-and-vm.md#1411-debug-hooks)).
- The ownership pass's place states and explanations
  ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)).
- The runtime's `debug` slice: trace events, record and replay hooks, panic
  reports ([05 §5.1](../../../../docs/architecture/runtime.md#51-feature-slices)).
- Capability-mediated I/O and the seeded scheduler
  ([14 §14.7](../../../../docs/architecture/bytecode-and-vm.md#147-tasks-scheduler-and-seed)).
- The diagnostic record and the compiler queries, for the language server.

Downstream:

- DAP clients and editors depend on the `initialize` capabilities, the
  exception filters (`panicked`, `propagated`, `assert_failed`) and the
  `verse` fields.
- CI reads panic reports through `VERSE_PANIC=json`.
- `verse difftest` has a replay leg: a native run recorded with `--record` and
  replayed in the VM must reproduce every observable. A divergence is a bug in
  the recorder, the VM or a tier
  ([14 §14.12](../../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract)).
- A change to an event name or template changes compiler diagnostics that use
  the same event ([16 §16.1](../../../../docs/reference/diagnostics-catalog.md#161-rules) rule 7).

## After a change

1. Add tests by layer
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - Golden wording tests that render each stop, value and error from fixed
     records. They fail on any "don't say" word and check that a diagnostic
     and an event with the same name share a template
     ([08 §8.7](../../../../docs/architecture/debugging.md#87-vocabulary-and-wording)).
   - Differential tests with recording on and off, and hooks on and off
     ([14 §14.12](../../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract)).
   - Unit tests for the core and for the `.vsrec` reader and writer.
   - Fuzzing covers the VM
     ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)).
2. Check the milestone exit criteria you touch
   ([roadmap](../../../../docs/roadmap.md)). M1: `verse debug` can launch, stop
   and inspect over DAP. M6: the LSP passes a protocol test suite, and replay
   reproduces a recorded task-heavy run exactly.
3. Update the docs in the same pull request:
   - Adapter behavior in [08](../../../../docs/architecture/debugging.md).
   - Recording format and hooks in [14](../../../../docs/architecture/bytecode-and-vm.md).
   - Names, templates and `D0xxx` codes in
     [`events.toml`](../../../../docs/reference/events.toml).
   - A change to the decision itself is a new ADR, and
     [ADR-0006](../../../../docs/decisions/0006-debugging-over-dap.md) becomes
     `superseded` ([decisions](../../../../docs/decisions/README.md)).
   - The `verse debug` and `verse lsp` rows of
     [19 §19.2](../../../../docs/reference/cli-and-manifest.md#192-commands).
4. Commit as `<type>(tooling): …`. Use `vm` when the change is in the VM's
   hooks or recorder. A design change is `spec(tooling)`.
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
