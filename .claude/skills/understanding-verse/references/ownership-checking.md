# Ownership checking

The ownership pass: it classifies every use as a move, copy, borrow, mut
borrow or view, finds every drop point, and explains every ownership error.

## Read first

1. [02 §2.4 The ownership pass](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass).
   This is the pass's contract.
2. [ADR-0002](../../../../docs/decisions/0002-ownership-on-generic-code.md):
   why a local dataflow pass on generic code.
3. Spec [§4.9 Ownership, moves and copies](../../../../docs/language/04-types.md#49-ownership-moves-and-copies)
   and [§4.10 Memory model](../../../../docs/language/04-types.md#410-memory-model).
4. Spec [§4.2 rule 9](../../../../docs/language/04-types.md#inference-rules):
   ownership is checked once, at the definition.
5. Spec [§5.3 Borrow, `mut`, and `take`](../../../../docs/language/05-functions.md#borrow-mut-and-take),
   [Borrows are second-class](../../../../docs/language/05-functions.md#borrows-are-second-class)
   and [§5.4 Captures](../../../../docs/language/05-functions.md#captures-borrowed-or-moved).
6. Spec [§9.5 Transforming collections](../../../../docs/language/09-collections.md#95-transforming-collections):
   chains pass views, the end decides.
7. Spec [§11.1 Ownership across tasks](../../../../docs/language/11-concurrency.md#ownership-across-tasks)
   and [§13.1 `with` blocks](../../../../docs/language/13-resources.md#131-with-blocks).
8. Spec [§18.10 Why no garbage collector](../../../../docs/language/18-design-notes.md#1810-why-no-garbage-collector):
   the trade-offs and the error-message requirement.
9. The V05xx and V07xx tables of the
   [diagnostics catalog](../../../../docs/reference/diagnostics-catalog.md#v05xx-ownership).

## What it does

| Input invariant | Output invariant | Errors it owns |
|-----------------|------------------|----------------|
| THIR | every use is classified as move, copy, borrow, mut borrow, or view; every drop point is known | use after move, borrow rules, `Copy` bound errors, resource and task rules (`with`, views sent to tasks) |

Source: [02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts).

- The crate is `verse-own`
  ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).
- It runs on THIR of the generic definition, once per function, and never
  again for specializations
  ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)).
- Its result drives drop elaboration and explains errors
  ([ADR-0002](../../../../docs/decisions/0002-ownership-on-generic-code.md)).
- It reports V05xx and V07xx
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).

## How it works

The pass computes six things
([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)):

1. **Liveness and last use** for every local, by backward dataflow over the
   function's CFG. Why: `b = a`, `for x in items`, pattern bindings and chain
   results are views when the source is used later and owners at the source's
   last use.
2. **Place states.** Each place (local, field path, element-in-place) is
   `Owned`, `MovedAt(site)`, `Borrowed(kind)` or `View(of, until)`.
   Conditional moves produce drop flags, as in Rust MIR drop elaboration.
3. **Second-class borrow rules.** A borrow cannot be stored, returned or sent
   to a task, so its lifetime is the dynamic extent of one call. The check is
   a local rule, not a lifetime solver: a `mut` argument is exclusive for the
   call, with arguments evaluated first; a view cannot flow into a stored
   position, a return, or `spawn`.
4. **Chain classification.** Adapters such as `filter` and `zip` pass views;
   the chain end decides. The pass records the owning step that collects each
   chain.
5. **Inferred `Copy` bounds** for unannotated getters, written to the
   function's interface record and to `verse.api` for exports.
6. **Explanations.** Every error carries the move site, the later use that
   kept a source alive, the rule number, and the suggested fix.

**Why a custom pass.** Rust's borrow checker solves lifetimes across stored
references. Verse has none, so region inference would be unused complexity
([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)).
ADR-0002 also rejects checking after monomorphization (errors would move off
the definition and differ per instantiation) and reference counting
everywhere (it contradicts deterministic frees)
([ADR-0002](../../../../docs/decisions/0002-ownership-on-generic-code.md)).
The analysis is linear-time per function and parallel across functions.

**Language rules the pass enforces:**

- Copy types are scalars and records, variants and tuples made only of copy
  types. Everything else moves when it gets a new owner, and is never
  duplicated implicitly; the compiler asks for `.copy()`
  ([§4.9](../../../../docs/language/04-types.md#49-ownership-moves-and-copies)).
- A value moves when bound with `mut b = a` (or `b = a` at `a`'s last use),
  stored, returned, passed to `take` or `spawn`, or captured by an escaping
  lambda. Moves are tracked per control-flow path; a move in a loop counts as
  moved on the next iteration
  ([§4.9](../../../../docs/language/04-types.md#49-ownership-moves-and-copies)).
- A parameter is a read-only borrow by default; `mut p` is an exclusive
  borrow; `take p` consumes. `merge(mut a, a)` is an error
  ([§5.3](../../../../docs/language/05-functions.md#borrow-mut-and-take)).
- In `place = value`, the right side is evaluated completely first
  ([§4.9](../../../../docs/language/04-types.md#49-ownership-moves-and-copies)).
- A non-escaping lambda borrows its captures; an escaping one moves them. A
  function-typed parameter is non-escaping unless declared `take`
  ([§5.4](../../../../docs/language/05-functions.md#captures-borrowed-or-moved)).
- `spawn f(x)` moves `x`; views (V0504, V0703) and `mut` arguments (V0708)
  cannot cross into a task
  ([§11.1](../../../../docs/language/11-concurrency.md#ownership-across-tasks)).
- Inside generic or unannotated code, a parameter type is opaque and non-`Copy`
  unless a `Copy` bound says otherwise
  ([§4.2 rule 9](../../../../docs/language/04-types.md#inference-rules),
  [§10.6](../../../../docs/language/10-traits-generics.md#106-constraints)).
- Locals are freed in reverse order of declaration; the parts of one value in
  the order they were written
  ([§4.10](../../../../docs/language/04-types.md#410-memory-model)).

## Rules that must hold

- Ownership runs once, on generic code, before specialization. This is the
  most important ordering constraint in the pipeline
  ([01 constraint 3](../../../../docs/architecture/requirements.md#constraints-taken-from-the-specification)).
- Errors appear at the definition. A call site fails only on an unmet
  written or inferred bound
  ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)).
- Every error that a later use caused must name that later use
  ([§4.9](../../../../docs/language/04-types.md#49-ownership-moves-and-copies),
  [§18.10](../../../../docs/language/18-design-notes.md#1810-why-no-garbage-collector)).
- Monomorphization never adds or removes a move, drop or panic, and does not
  recompute ownership facts
  ([13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir)).
- MIR verifier rules V3 to V9 re-derive what this pass decided: V4
  initialisation, V5 drops and flags, V6 no leak at return, V7 borrows are
  second-class, V8 exclusive borrows, V9 no move of a borrowed source
  ([13 §13.5](../../../../docs/architecture/mir-specification.md#135-verifier-rules)).
- The value states the debugger shows are exactly the states this pass uses
  ([08 §8.4](../../../../docs/architecture/debugging.md#values),
  [08 §8.7](../../../../docs/architecture/debugging.md#value-states)).
- A bug here is a soundness bug in the language
  ([ADR-0002](../../../../docs/decisions/0002-ownership-on-generic-code.md)).

## What depends on it

- MIR build takes THIR plus ownership facts and emits explicit `Move`,
  `Copy`, `Drop` and unwind edges
  ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
  Moves and flags in generic MIR are decided by this pass
  ([13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir)).
- `verse.api` records inferred `Copy` bounds, so a change here can change a
  public signature
  ([ADR-0002](../../../../docs/decisions/0002-ownership-on-generic-code.md)).
- The debugger shows ownership state and the same explanations
  ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass),
  [08 §8.4](../../../../docs/architecture/debugging.md#values)).
- The language server shows ownership state inline
  ([08 §8.8](../../../../docs/architecture/debugging.md#88-verse-extensions)).
- Upstream, the pass trusts THIR: every expression has a type. See
  [names-and-types.md](names-and-types.md).

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - must-fail programs under `tests/must-fail/` with expected codes (use
     after move, borrow conflicts, data races);
   - golden snapshots of the diagnostics text;
   - fuzzing of the ownership pass;
   - the verifier (`verse-verify`) on the MIR it produces.
2. Run the soundness checks of
   [10 §10.4](../../../../docs/architecture/testing-and-verification.md#104-safety-properties-we-test-explicitly):
   the generated-program fuzzer under the poisoning allocator. Compare with
   the executable model of the ownership rules
   ([10 §10.6](../../../../docs/architecture/testing-and-verification.md#106-formal-methods-selective)).
3. Every ownership code needs a golden test with span, rule and fix
   ([10 §10.5](../../../../docs/architecture/testing-and-verification.md#105-diagnostics-quality-tests)).
   See [diagnostics.md](diagnostics.md).
4. A change to an ownership rule is a language change: type `spec`, with a
   conformance test in the same commit. Code changes use scope `compiler`
   ([CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format)).
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
