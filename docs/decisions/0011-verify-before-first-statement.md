# ADR-0011: Verify every reachable instance before the first statement

Status: accepted (2026-10-19).

## Context

`verse check` is the command a person or a CI job runs to ask "is this program
fine?". Its answer is only useful if running the program agrees with it. If
script mode verified each instance at its first call, a program that
`verse check` accepted could print half its output and then stop with a
verifier failure (V9902, exit status 3). The status a script sees would depend
on which path ran first.

Script mode runs a whole-program reachability pre-pass before the first
instruction, so the capability check can refuse a program before it starts
([02 §2.6](../architecture/compiler-architecture.md#26-monomorphization)).
A pre-pass that only walks the generic MIR with type arguments substituted can
compute `caps` and `mayyield` per instance key, but it builds no mono MIR: it
attaches no layouts and produces no renamed instance for the mono verifier
rules to check (V22 needs a `layout` for every type used). Verifying in the
pre-pass therefore means monomorphizing there, not only computing facts.

The verifier rule ids run from V1 to V24
([13 §13.5](../architecture/mir-specification.md#135-verifier-rules)), but a
mono instance is checked against the mono rule set of
[13 §13.6](../architecture/mir-specification.md#136-generic-mir-and-mono-mir),
which leaves out V21 (generic only, checked after MIR build) and adds V22.

## Decision

1. **The pre-pass monomorphizes and verifies.** Before the first statement,
   script mode monomorphizes every reachable instance, with the five steps of
   13 §13.6, and verifies each one with the mono rules of 13 §13.6. It runs no
   optimizer pass and builds no bytecode. Its result is cached per module
   ([02 §2.7](../architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)).
2. **Bytecode generation stays lazy.** The first call of an instance builds its
   proto from the verified mono MIR
   ([14 §14.6](../architecture/bytecode-and-vm.md#146-calls-and-lazy-bytecode-generation)).
   A verifier failure therefore never happens after the first statement.
3. **`verse check` runs the same pre-pass and the same capability check.** It
   makes every check that running a file makes before the first statement,
   except loading plugin libraries, and generates no code
   ([19 §19.2.3](../reference/cli-and-manifest.md#1923-building-and-checking)).
4. **The promise.** After `verse check` exits 0 on a file, running that file
   with the same sources, `verse.lock`, compiler and target never exits 2 and
   never stops with V9902
   ([16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses)).
5. **The capability check before run is the only capability check.** In debug
   builds of `verse` itself, `CALLS` asserts the capability bit of the proto.
   The assertion checks the VM's own code and is never an exit status
   ([14 §14.9](../architecture/bytecode-and-vm.md#149-sys-capability-calls-and-the-capability-check)).

Points the proposal left open, decided here:

- **What the promise excludes.** Two inputs exist only when the program runs:
  the capabilities it is granted (`--deny`, `[run] deny`) and the plugin
  libraries on the machine (V1007 to V1009, V1011,
  [07 §7.6](../architecture/plugins.md#76-loading-in-script-mode-and-compiled-mode)).
  `verse check` cannot decide them, so either can still stop a checked
  program before it starts, with status 2. `verse check` does not load plugin
  libraries because a plugin's `build` never runs during `verse check`, so the
  library it would load may not exist yet.
- **Status 3 stays possible.** Bytecode generation stays lazy by decision 2,
  so a bug in it is reported at a first call (V9901). A trap inside the
  runtime is a bug no pre-run check can rule out. The promise covers every
  check `verse check` runs, not every bug in Verse.
- **The verifier runs in every build of `verse`.** Verification after
  monomorphization is part of running a file and of `verse check`, not a
  debug-build or CI check. The checks after every optimizer pass stay in debug
  builds and CI.
- **Performance target.** The cold pre-pass target for 10,000 reachable
  instances is under 150 ms on 8 cores
  ([14 §14.13](../architecture/bytecode-and-vm.md#1413-performance-targets)).
  The first-call target of the same table allows 100 microseconds to compile
  a verified 50-statement instance to bytecode. Taking the same bound for
  monomorphizing and verifying one instance gives 1 s of work for 10,000
  instances, about 125 ms over 8 cores, since the instance is the unit of
  parallelism (02 §2.7). The warm target stays under
  5 ms, because the cache holds verified instances.

## Alternatives

- **Verify each instance at its first call.** Start-up pays only for what
  runs, but the exit status table then has a row (3) that a checked program
  can still produce halfway through, and `verse check` promises nothing about
  a run. Python reports most errors only when the failing line runs; that is
  the behaviour this decision rules out for checked programs.
- **Verify every instance, reachable or not.** Not needed. The promise is
  about the program that runs, and reachability already bounds it. Zig takes
  the same position: it analyses only the code a program references.
- **Make bytecode generation eager too.** It would remove V9901 at first call,
  but start-up would scale with all reachable code instead of what runs, for a
  failure that only a bug in the bytecode compiler can cause. A verified
  instance gives the bytecode compiler a valid input.
- **Keep `verse check` to the front end.** Rust's `cargo check` stops before
  monomorphization, so errors found only after monomorphization appear in
  `cargo build` and not in `cargo check`. Verse would then have the same gap
  between check and run.

## Consequences

- `verse check` and the first run of a file pay the verifier for reachable code
  that never executes. The per-module cache keeps that cost to the first run
  after a change.
- Script mode and a compiled build verify the same mono MIR with the same
  rules, so the verifier sees the same `caps` and `mayyield` in both modes.
- The exit status table has no status that exists only because one check runs
  late.
- The promise is a spec conformance rule
  ([10 §10.2](../architecture/testing-and-verification.md#102-specification-conformance)):
  a test runs `verse check` and then the file, and compares the statuses.
