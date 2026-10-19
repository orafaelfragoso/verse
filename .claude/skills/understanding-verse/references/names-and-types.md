# Names and types

Name resolution, HIR lowering, type inference and THIR: everything between
the AST and the ownership pass.

## Read first

1. [02 §2.2 Stage contracts](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts),
   the Name resolution, HIR lowering and Type inference rows.
2. [02 §2.5 Type inference](../../../../docs/architecture/compiler-architecture.md#25-type-inference).
3. [03 §3.2 HIR: the desugared core](../../../../docs/architecture/ir-and-passes.md#32-hir-the-desugared-core).
4. Spec [§4.2 Type annotations and inference](../../../../docs/language/04-types.md#42-type-annotations-and-inference),
   all nine [inference rules](../../../../docs/language/04-types.md#inference-rules).
5. Spec [§3.9 Bindings and assignment](../../../../docs/language/03-lexical.md#bindings-and-assignment):
   no `let`, no shadowing.
6. Spec [§12.2 Importing](../../../../docs/language/12-modules.md#122-importing),
   [§12.3 Visibility](../../../../docs/language/12-modules.md#123-visibility-with-export)
   and [§12.4 Module contents](../../../../docs/language/12-modules.md#124-module-contents).
7. Spec [§10.3 Traits as types](../../../../docs/language/10-traits-generics.md#103-traits-as-types)
   and [§10.6 Constraints](../../../../docs/language/10-traits-generics.md#106-constraints).
8. Spec [§16.4 Interface files](../../../../docs/language/16-tooling.md#164-interface-files-verseapi).
9. The V03xx, V04xx and V06xx tables of the
   [diagnostics catalog](../../../../docs/reference/diagnostics-catalog.md#v03xx-names-and-imports).

## What it does

| Stage | Input | Output |
|-------|-------|--------|
| Name resolution | AST of all modules in a package | every identifier is a `Def` id or a local id; the no-shadowing and refinement rules enforced; import graph acyclic (V0310) |
| HIR lowering | resolved AST | no sugar; optional-lifting positions marked but not yet typed |
| Type inference | HIR | every expression has a type or a recorded unresolved variable; an unresolved variable becomes the "cannot infer, annotate here" error |

Source: [02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts).
The output of inference is THIR, HIR with a type per expression, still generic
([03 §3.1](../../../../docs/architecture/ir-and-passes.md#31-the-representations)).

Crates ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)):

- `verse-hir`: resolution, HIR, THIR.
- `verse-types`: inference, trait solving, constraints.

## How it works

**Resolution and scopes.** A name is introduced by writing it. `name = expr`
is a new immutable binding when the name is not yet bound in the function,
a reassignment when it names a visible `mut` binding, a refinement when it
names a visible immutable binding of the same block and `expr` uses the name
(a new local that consumes the old value), and an error otherwise. You cannot
bind a name that is currently visible, except by a refinement, where visible
means locals only. Sibling blocks may reuse a name. A
local may hide a prelude, imported or module name; hiding an import warns
only when the function also uses that import
([§3.9](../../../../docs/language/03-lexical.md#bindings-and-assignment)).
Why: without `let`, shadowing and reassignment would look identical
([§18.9](../../../../docs/language/18-design-notes.md#189-immutability-ownership-no-shadowing)).

**Imports.** The first segment of a module path names a module of the
program, a dependency, or a standard library module, and the three sets never
overlap. There are no wildcard imports. Importing a type does not import its
variants; matching needs no import, building does. Imports sit at the top of
the file ([§12.2](../../../../docs/language/12-modules.md#122-importing)).

**Module contents.** Only the entry file runs top-level statements. A
top-level binding in an imported module is a constant, initialized once on
first use, and must be infallible. The constants of one module never read
each other in a cycle, through functions or not (V0426). In the entry file
it is a script local,
not visible inside `fn` bodies
([§12.4](../../../../docs/language/12-modules.md#124-module-contents)).

**HIR lowering.** It expands `otherwise`, `try`, `if … is`, key-path lambdas,
postfix guards, string interpolation, `for`, plain dots through
optionals and uniform call syntax into a small core
([03 §3.2](../../../../docs/architecture/ir-and-passes.md#32-hir-the-desugared-core)).
Why: later stages then handle a small language.

**Inference.** Hindley–Milner style unification with trait and callable
constraints, solved per function and per module
([02 §2.5](../../../../docs/architecture/compiler-architecture.md#25-type-inference)):

- Exported items are written in full: parameter types, return type and
  `raises` clause (V0418).
- Unannotated parameters of private functions used only through trait
  operations become implicit generics with `Trait(T)` and
  `Callable(T, args, ret)` constraints.
- A parameter whose body reads a field or calls an inherent method takes the
  one concrete type the calls in its module pass (V0425 if they disagree).
  There are no structural constraints.
- Recursion requires annotations, so inference never needs a fix-point
  across a call cycle. Strongly connected components of the call graph are
  checked for an annotated entry.
- Numeric literals default to `Int` or `Float` after solving.
- Raise sets are inferred from `try` with the return type, exact as in §4.6, and
  checked against a written `raises` clause in both directions (V0420, V0421).
- Interned types make type equality integer equality
  ([02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)).

**Module boundary.** An exported function's written signature is compared
with `verse.api`. Importers use only `verse.api`, so a module is checked
without the source of its dependencies
([02 §2.5](../../../../docs/architecture/compiler-architecture.md#25-type-inference),
[§12.3](../../../../docs/language/12-modules.md#exported-signatures-and-verseapi)).
Why: packages compile independently and in parallel, and a body edit cannot
silently change the public API
([§4.2](../../../../docs/language/04-types.md#inference-and-module-boundaries)).

**Traits as types.** A trait name used as a type is dispatched at run time
([§10.3](../../../../docs/language/10-traits-generics.md#103-traits-as-types)).
Monomorphization leaves calls through it unresolved, as one shared body
([02 §2.6](../../../../docs/architecture/compiler-architecture.md#26-monomorphization)).

**Parallelism.** The function body is the unit for inference; the module is
the unit for resolution
([02 §2.7](../../../../docs/architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)).

## Rules that must hold

- Every identifier resolves to a `Def` id or a local id
  ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
- No shadowing among locals; a refinement (`x = f(x)` on an immutable `x` of
  the same block) is the one rebinding; `let` is reserved, with a friendly error
  ([§3.9](../../../../docs/language/03-lexical.md#bindings-and-assignment),
  [§18.9](../../../../docs/language/18-design-notes.md#189-immutability-ownership-no-shadowing)).
- Inference never crosses a module boundary
  ([§4.2](../../../../docs/language/04-types.md#inference-and-module-boundaries)).
- `build`, `check` and `test` fail when an exported signature differs
  from `verse.api`
  ([§16.4](../../../../docs/language/16-tooling.md#164-interface-files-verseapi)).
- When inference cannot decide, it stops at that spot and asks for one
  annotation; it never guesses (rule 4,
  [§4.2](../../../../docs/language/04-types.md#inference-rules)).
- `mut` and `take` are never inferred (rule 8).
- Structural constraints cannot be written by hand in 0.1 (rule 1).
- Errors about a body appear at the definition. The only call-site errors are
  unmet bounds, and they name the bound (rule 9).
- Name resolution reports V03xx and V1006. Type inference reports V04xx,
  V06xx, V1002, V1003 and V1010. HIR lowering reports only V99xx
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).

## What depends on it

- The ownership pass runs on THIR and trusts that every expression has a
  type ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
  It infers no bound, so it writes nothing into the interface record or
  `verse.api`
  ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass)).
  See [ownership-checking.md](ownership-checking.md).
- Monomorphization keys instances by `(DefId, [TypeId])` and resolves trait
  calls through the impl table
  ([02 §2.6](../../../../docs/architecture/compiler-architecture.md#26-monomorphization),
  [13 §13.6](../../../../docs/architecture/mir-specification.md#136-generic-mir-and-mono-mir)).
- `verse-pkg` owns `verse.api`, and V0810 and V0811 report its drift
  ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout),
  [catalog V08xx](../../../../docs/reference/diagnostics-catalog.md#v08xx-packages-and-api)).
- The language server shows inferred types as inline hints, from the same
  compiler queries ([§4.2 rule 6](../../../../docs/language/04-types.md#inference-rules),
  [08 §8.8](../../../../docs/architecture/debugging.md#88-verse-extensions)).
- Upstream, resolution trusts the AST's stable `NodeId`s and spans. See
  [parser-and-formatter.md](parser-and-formatter.md).

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - golden snapshots of HIR and THIR under `tests/golden/`;
   - a conformance test named after the spec rule
     ([10 §10.2](../../../../docs/architecture/testing-and-verification.md#102-specification-conformance));
   - fuzzing of the type checker;
   - must-fail programs with the expected code under `tests/must-fail/`.
2. The roadmap retires the inference risk with every spec example plus 50
   adversarial programs ([roadmap risks](../../../../docs/roadmap.md#risks-and-their-milestones)).
   Keep both passing.
3. Check the compile budget: no quadratic algorithm over module size in a
   stage that runs on every edit
   ([09 §9.5](../../../../docs/architecture/performance.md#95-compile-time-budget-discipline)).
4. A new error goes in V03xx (resolution) or V04xx/V06xx (inference). See
   [diagnostics.md](diagnostics.md).
5. A change to inference rules is a language change: type `spec`, with the
   spec and a conformance test in the same commit. Code changes use scope
   `compiler` ([CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format)).
6. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
