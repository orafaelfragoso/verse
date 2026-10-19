---
name: writing-tests
description: Covers how Verse is tested and which tests a change needs. The test layers of testing-and-verification §10.1 (unit, golden, spec conformance, differential, property, fuzz, must-fail soundness, MIR verifier, benchmarks), where each lives (tests/golden, tests/conformance, tests/must-fail, `verse difftest`, proptest, `cargo fuzz`, `verse-verify`, bench/), conformance tests tied to spec rules, expected diagnostic codes, and the release gates of §10.7. Use when adding or changing tests, when a change to the lexer, parser, type checker, ownership pass, MIR, a backend, the VM, the runtime or a diagnostic needs tests, or when asked which tests to write.
---

# Writing tests

No code exists yet. The layout below is the plan from the docs, not a tree
you can browse. Where a file format or naming scheme is not decided, this
skill says so. Do not invent one in a test; raise it against the docs (see
the `editing-docs` skill).

## Read first

1. [10 §10.1 Test layers](../../../docs/architecture/testing-and-verification.md#101-test-layers)
2. [10 §10.2 Specification conformance](../../../docs/architecture/testing-and-verification.md#102-specification-conformance)
3. [10 §10.3 Differential testing](../../../docs/architecture/testing-and-verification.md#103-differential-testing)
   and the contract in [14 §14.12](../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract)
4. [16 §16.1 rule 9](../../../docs/reference/diagnostics-catalog.md#161-rules):
   the three tests every diagnostic code needs
5. [10 §10.7 Release gates](../../../docs/architecture/testing-and-verification.md#107-release-gates)

## The layers

| Layer | Checks | Where (planned) | Runs |
|-------|--------|-----------------|------|
| Unit | one function or data structure | `cargo test` in each crate under `crates/` | every commit |
| Golden | token streams, CST dumps, HIR, THIR, MIR, diagnostics text | snapshot files under `tests/golden/` | every commit |
| Spec conformance | every normative sentence in the spec | `tests/conformance/` | every commit |
| Differential | tier 0, Cranelift, LLVM and Wasm give identical output, optimizer on and off | `verse difftest` | fast set every commit, full set nightly |
| Property | formatter idempotence, parse and print round trip, `verse fmt` never changes meaning | `proptest` | every commit |
| Fuzzing | lexer, parser, type checker, ownership pass, VM | `cargo fuzz` and a Verse program generator; corpora in `tests/` | continuous |
| Soundness | programs that must be rejected | `tests/must-fail/` with expected diagnostic codes | every commit |
| Verifier | MIR invariants after every pass | `verse-verify` | debug builds and CI |
| Benchmarks | the budgets of [09](../../../docs/architecture/performance.md#92-benchmark-suite) | `verse bench`, programs in `bench/` | nightly and on request |

The crate list and the `tests/` and `bench/` folders are in
[02 §2.9](../../../docs/architecture/compiler-architecture.md#29-crate-layout).
Verse code in `std/` tests itself with `test` blocks run by `verse test`
([spec §14.1](../../../docs/language/14-testing-docs.md#141-test-blocks)).

## Which layers a change needs

Pick from this table. Add only the layers the change can break.

| Change | Add or update |
|--------|---------------|
| Lexer or parser (`verse-syntax`) | golden token or CST dump; conformance test for the spec rule; the property round trip must still pass; a fuzz corpus entry for any crash found |
| Grammar change (spec chapter 17) | conformance tests for each new production; golden CST |
| Name resolution, HIR (`verse-hir`) | golden HIR; conformance; must-fail for each new rejection |
| Type inference (`verse-types`) | conformance; must-fail with the expected `V04xx` code; golden diagnostic text |
| Ownership pass (`verse-own`) | must-fail with the expected `V05xx` code; conformance; the soundness fuzzer must stay clean |
| MIR build or an optimizer pass (`verse-mir`) | golden MIR; differential with the pass on and off; `verse-verify` runs after the pass |
| A new MIR verifier rule | a MIR input that breaks the rule and must report `V9902` |
| Backend (`verse-bc`, `verse-cl`, `verse-llvm`, `verse-wasm`) | differential against tier 0 on the conformance suite |
| Runtime (`verse-rt`) | unit tests; the safety runs of [10 §10.4](../../../docs/architecture/testing-and-verification.md#104-safety-properties-we-test-explicitly) (debug allocator, sanitizers, ThreadSanitizer for tasks) |
| Formatter (`verse-fmt`) | property: idempotence and meaning preserved; golden output |
| A diagnostic code | the three tests of 16.1 rule 9 |
| A spec change that changes behavior | change a conformance test in the same commit |
| A performance change | a benchmark delta in the pull request ([09 §9.2](../../../docs/architecture/performance.md#92-benchmark-suite) rule 1) |

## Conformance tests

[10 §10.2](../../../docs/architecture/testing-and-verification.md#102-specification-conformance)
ties each test to a normative spec sentence.

- Every rule gets an identifier by section, for example `§4.9` plus a
  number, and the test file name contains it. The exact numbering scheme
  and file name format are not decided yet.
- A coverage report lists spec sections with no test. The target is 100%.
- Every ```verse block in the spec is extracted by `tools/extract-examples`
  and either runs as a passing test or is tagged with the reason it is a
  fragment. `...` in an example is not Verse syntax
  ([spec §1.7](../../../docs/language/01-philosophy.md#17-notation-in-this-document)).
- When writing a test exposes an ambiguity in the spec, raise it against the
  spec. Do not settle it in the code as a silent choice.

## Must-fail tests

A must-fail test is a program the compiler must reject, with the diagnostic
codes it must report: use after move, borrow conflicts, data races
([10 §10.1](../../../docs/architecture/testing-and-verification.md#101-test-layers)).

- Assert the code, for example `V0501`, not only that compilation failed. A
  test that passes on any error proves nothing.
- One rejected rule per file, so a failure names one rule.
- Take the codes from the [catalog](../../../docs/reference/diagnostics-catalog.md#163-catalog).
  A program that needs a code the catalog lacks needs a catalog change first.
- How a file states its expected codes is not decided yet.

## Diagnostic tests

Each code has three tests
([16 §16.1](../../../docs/reference/diagnostics-catalog.md#161-rules) rule 9):

1. a golden test of the exact rendered text
   ([16 §16.4](../../../docs/reference/diagnostics-catalog.md#164-how-a-code-is-shown));
2. a test that the `help:` edit applies and the program then compiles, when
   the fix is machine-applicable;
3. a test that `verse explain` has text
   ([16 §16.5](../../../docs/reference/diagnostics-catalog.md#165-verse-explain)).

A test also fails when a placeholder has no fact or a fact has no placeholder
(rule 10). A CI check fails when a code has no test or no `verse explain`
text ([10 §10.5](../../../docs/architecture/testing-and-verification.md#105-diagnostics-quality-tests)).

## Differential tests

`verse difftest` runs each program on every available backend with the same
input, a seeded scheduler and a virtual clock, and compares standard output,
standard error, the exit status and the sequence of task events
([10 §10.3](../../../docs/architecture/testing-and-verification.md#103-differential-testing)).
The full list of observables is in
[14 §14.12](../../../docs/architecture/bytecode-and-vm.md#1412-differential-testing-contract).

- Tier 0, the bytecode VM, is the reference. Native backends are checked
  against it ([roadmap principle 2](../../../docs/roadmap.md#principles-for-sequencing)).
- On a mismatch the harness disables passes one at a time
  (`--no-opt-pass=NAME`, [03 §3.4](../../../docs/architecture/ir-and-passes.md#34-pass-list))
  to find the guilty pass.
- Floating point is IEEE 754 binary64 with no fast-math on every tier, so
  results must be bit-identical. Do not compare floats with a tolerance.
- A program that reads time, randomness or I/O must get them through
  capabilities so the seed and the fakes make it deterministic.

## Release gates

A compiler release requires
([10 §10.7](../../../docs/architecture/testing-and-verification.md#107-release-gates)):

1. all conformance tests pass;
2. the full differential suite passes;
3. the fuzzers ran for a fixed budget with no new crashes;
4. no benchmark gate regression (5% on a gated benchmark,
   [09 §9.2](../../../docs/architecture/performance.md#92-benchmark-suite) rule 4);
5. the spec coverage report shows no uncovered section, and `verse.api` for
   the standard library is unchanged or has an intentional diff.

## Focused tests

- Test what a user observes: the program's output, the exit status, the
  diagnostic code and text. Not the internal shape of a pass.
- One behavior per test. The test name says the behavior.
- Assert exact expected values. A golden file is an exact value; review its
  diff like code.
- No smoke tests that only check "it runs". No tests that only prove a
  deleted feature stays deleted.
- A crash found by a fuzzer becomes a minimal regression test at the layer
  that owns the bug, plus a corpus entry.

## After writing tests

1. Run the layers you touched. Paste the commands and their summary lines in
   the pull request's Verification section.
2. Use type `test` for a pull request with only tests. Otherwise the test
   ships in the same pull request as the change it covers.
3. If the docs changed, run `tools/docs/check.sh`.
