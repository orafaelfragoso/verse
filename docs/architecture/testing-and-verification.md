# 10. Testing and verification

The compiler is only useful if it is right. This document defines how we know.

## 10.1 Test layers

| Layer | What it checks | Tool | Runs |
|-------|----------------|------|------|
| Unit | one function or data structure | `cargo test` | every commit |
| Golden | token streams, CST dumps, HIR, THIR, MIR, diagnostics text | snapshot files under `tests/golden/` | every commit |
| **Spec conformance** | every normative sentence in the spec has a test | `tests/conformance/` | every commit |
| **Differential** | tier 0, Cranelift, LLVM and Wasm produce identical output on the same program, with optimization on and off | `verse difftest` | every commit (fast set), nightly (full) |
| Property based | formatter idempotence, parse and print round trip, `verse fmt` never changes meaning | `proptest` | every commit |
| Fuzzing | lexer, parser, type checker, ownership pass, VM, with grammar-aware generators | `cargo fuzz`, a Verse program generator | continuous |
| Soundness | programs that must be **rejected** (use after move, borrow conflicts, data races) | `tests/must-fail/` with expected diagnostic codes | every commit |
| Verifier | MIR invariants after MIR build, after monomorphization and after every optimizer pass | `verse-verify` | after MIR build and monomorphization: every build of `verse` (`build`, running a file, `verse check`); after every optimizer pass: debug builds and CI |
| Benchmarks | [09](performance.md) | `verse bench` | nightly and on request |

## 10.2 Specification conformance

- The spec's code examples are extracted by `tools/extract-examples` and run:
  every ```verse block is either a passing test or tagged with the reason it is
  a fragment (spec §1.7 says `...` is not Verse syntax).
- Every rule gets an identifier by section (for example `§4.9` plus a number)
  and a test file name that contains it. A coverage report lists spec
  sections that have no test. The target is 100%.
- A spec change that changes behavior must change a conformance test in the
  same commit. This keeps the spec and the compiler from drifting.
- The check-then-run test runs `verse check` on each conformance program and
  then runs it with the same inputs. It fails when the check exits 0 and the
  run then stops with V9902, or exits 2 for a reason the rule of
  [16 §16.6.1](../reference/diagnostics-catalog.md#1661-exit-statuses) does
  not except. Verification after an optimizer pass, which runs only in
  debug builds of `verse` and in CI, is outside this rule, but the test still
  fails on a V9902 from it, as a deliberate extra check of the optimizer.
- Ambiguities discovered while writing tests are raised against the spec,
  not settled in the code as silent choices.

## 10.3 Differential testing

Required by the rule that script and compiled modes behave
identically. The harness:

1. Compiles each test program for every available backend.
2. Runs them with the same input, a seeded scheduler (see [08](debugging.md))
   and a virtual clock.
3. Compares standard output, standard error, exit status and, for tasks, the
   sequence of observable events.
4. Repeats with the optimizer disabled, and with each pass disabled one at a
   time when a mismatch appears, to find the guilty pass automatically.

Floating-point behavior is IEEE 754 binary64 with no fast-math on every tier,
so results are bit-identical. Fused multiply-add is never introduced
implicitly.

## 10.4 Safety properties we test explicitly

- **Memory safety**: no use after free, no double free, no leaks in programs
  without `Shared` cycles. Run test suites under a debug allocator that
  poisons freed memory and reports leaks, and under Valgrind and sanitizers
  for the runtime crate.
- **Ownership soundness**: a generated-program fuzzer produces programs that
  pass the ownership pass, runs them under the poisoning allocator, and fails
  on any invalid access. Any such program is a soundness bug in the checker.
- **Data-race freedom**: ThreadSanitizer on the native runtime and on programs
  produced by the compiler for the concurrency tests.
- **Overflow, bounds, division**: every operation has tests at the boundary
  values, in all tiers.
- **Panic and drop order**: tests that panic or cancel while files, locks
  and channel ends are live, and check that they are released in the
  specified order (spec §13.2).

## 10.5 Diagnostics quality tests

Spec §18.10 makes ownership error messages a requirement. Each diagnostic
code has a golden test with the exact rendered text, and a rule that the
diagnostic must contain (a) the span, (b) the rule, (c) a fix when one exists.
A CI check fails if a diagnostic code has no test or no `verse explain` text.

## 10.6 Formal methods (selective)

We do not attempt to verify the whole compiler. We apply formal tools where
bugs are catastrophic and the logic is small:

- A **model of the ownership rules** (move, borrow, last use) written in a
  small executable specification (Rust or Alloy) and compared with the real
  pass on generated programs.
- **MIR verifier** as a standalone checker, written independently of the
  passes it checks.
- Later: translation validation for the optimizer on small functions (compare
  input and output MIR by symbolic execution), only if fuzzing finds
  optimizer bugs often enough to justify it.

## 10.7 Release gates

A compiler release requires: all conformance tests pass, the full differential
suite passes, the fuzzers ran for a fixed budget with no new crashes, no
benchmark gate regression, the spec coverage report shows no uncovered
section, and `verse.api` for the standard library is unchanged or has an
intentional diff.
