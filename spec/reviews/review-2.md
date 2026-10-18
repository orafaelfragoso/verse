# Verse spec review 2 (Michelangelo)

Scope: every file in `/workspace/verse/` as of round 2 (all 21 sections,
`grammar.ebnf.md`, `changes/changes-1.md`, `changes/changes-2.md`).
Legend: **A** = agreed, **O** = open (needs a decision or fix before I can agree).
Severity: High = changes a rule, Med = rule is missing or inconsistent,
Low = wording, example or grammar detail.

## Verdict on the two unrequested changes (U2f)

1. `take(n)` / `take_while` → `prefix(n)` / `prefix_while` — **AGREED**. `prefix`
   and `skip` read as a pair. Two follow-ups: a leftover `take` in the §9.5
   ownership paragraph (K-1), and the "(not `take`, which is a keyword)" aside
   in the §9.5 table belongs in §18.15, not in the reference table.
2. Assignment evaluates the right side before the place — **AGREED** (Python
   does the same). Add one sentence to §4.9: receiver and index expressions on
   the left are evaluated after the right side, left to right among
   themselves; everything else is left to right.

## Section status

| § | Status | Items |
|---|--------|-------|
| 1 Philosophy | A | |
| 2 Tour | O | X1 (the claim that `.copy()` is "the one place ownership shows" is false while `filter` copies silently) |
| 3 Lexical | O | L-1 … L-5 |
| 4 Types | O | X2, X3, T-1, T-2 |
| 5 Functions | O | X3, X4, F-1, F-2 |
| 6 Control flow | A | (X2 error-message requirement covers §6.3) |
| 7 Pattern matching | A | P-1 is optional |
| 8 Errors | O | E-1, X1 (`find_user` example) |
| 9 Collections | O | X1, K-1 … K-4 |
| 10 Traits/generics | O | X3 (`Copyable`), examples |
| 11 Concurrency | O | C-1, C-2 |
| 12 Modules | O | M-1, M-2 |
| 13 Resources | O | C-1 |
| 14 Testing/docs | A | |
| 15 Stdlib | O | F-1 (durations), prelude lists `Error` as a type and a trait |
| 16 Tooling | A | |
| 17 Grammar | O | G-1 … G-5, L-3, L-5 |
| 18 Design notes | O | D-1 … D-3 |
| 19 Comparison | A | |
| 20 Examples | O | EX-1 … EX-4 |
| 21 Review log | O | D-4 (log round 2 + this review) |

## Cross-cutting items (these need a joint decision)

### X1 [High] Silent deep copies in collection methods
§9.5 says `filter`, `find`, `first`, `sort`, `zip`, … "copy those elements"
unless the source is a temporary or at its last use. So
`people.filter(p -> p.age >= 18)` in the tour silently deep-copies every
adult `Person`, including their `Text` fields. That breaks our own rule
("`.copy()` is the single, visible way to duplicate"; §2, §4.9, §18.9) and is
a performance trap. The same hidden copy appears in spread (§3.7, L-1),
slices (K-4), `find_user` (§8.1), `first<T>` (§5.5, §10.5) and `largest`
(§10.6, §20.3).

Proposal: **views flow through a chain; the end of the chain decides.**
- Within one expression chain, the element-passing adapters (`filter`, `sort`,
  `sort_by`, `prefix`, `skip`, `unique`, `zip`, `enumerate`, `reverse`, …)
  pass second-class views. Nothing is copied.
- The chain ends in an owning step: `map(f)` (the lambda's owned result),
  a reduction (`sum`, `count`, `join`, `any`, …), a move out of an owned
  source at its last use, or an explicit `.copy()` that duplicates the
  remaining elements.
- If an element that is not `Copyable` would have to be duplicated implicitly,
  that is a compile error with the usual `.copy()` suggestion.
- `find`, `first`, `last`, `min`, `max` return views like `list[i]` and
  `m[k]` already do (§9.3), so `people.find(...)?.email` needs nothing, and a
  returned element is written `people.find(...)?.copy()`.
- A `let` bound to a chain is a local view, allowed because it cannot escape
  the scope (same as `let b = a`, §4.9).

Result: the tour line becomes
`people.filter(p -> p.age >= 18).map(.name.copy())` with exactly one visible
copy, and the "one place ownership shows" sentence stays true.

### X2 [Med] Ownership that flips with later code
`let b = a` (§4.9), `for x in items` (§6.3), pattern bindings (§7.7) and
chain results (§9.5) are views when the source is used later and owned when
it is the last use. Adding a `print(a)` line below can turn a working
`adults.push(person)` into an error. I accept the model, but add this
requirement to §4.9 and §18.10: every error caused by a flip must name the
later use that kept the source alive, for example
"`people` is used again on line 12, so `person` is only a view; write
`person.copy()` or move the later use."

### X3 [Med] Generic bodies and the two kinds of "copy"
(a) §4.2 says implicit generics are specialized per concrete type, and
`fn first_name(person) person.name` is a borrow-return error only when the
field is a `Text`. For `Int` it would compile. Errors that appear at some call
sites and not others, far from the cause, are what we promised not to have.
Rule to add (§4.2, §10.6): the ownership check runs **once, at the
definition**, treating every unannotated or generic parameter type as a
non-`Copy` opaque type unless a `Copy` bound says otherwise. This also makes
`verse.api` sufficient for cross-module checking.
(b) `.copy()` is documented as working on "any value", but `Task<T>` is "not
copyable" and a type with `Cleanup` must not be duplicated. Generic code like
`items[0].copy()` (§5.5) has no bound that allows it. Add a second built-in
trait, named `Copyable`: it provides `.copy()`, is derived structurally, and
is absent for `Task`, `Cleanup` types and anything containing them. `Copy`
(§10.4) stays the implicit-copy marker. Then
`fn first<T: Copyable>(items: List<T>): T?` with `items.get(0)?.copy()`.

### X4 [Med] Argument evaluation order for `mut` receivers
§4.10's `Tree.add` does
`self.nodes[p].children.push(self.nodes.len() - 1)`. The receiver is mutably
borrowed while the argument reads `self.nodes`. Rust rejects this shape. Add
to §5.3 and §5.6: arguments are evaluated first, then the exclusive borrow
for `mut` parameters and `mut self` begins (same idea as the assignment rule).
Without it the spec's own example is an error.

## Section items

### §3 Lexical
- **L-1 [Med]** Spread "copies when the source is used afterwards" (§3.7)
  is another hidden copy (X1). Rule: spread moves from the source at its
  last use; otherwise it is a compile error that suggests `..xs.copy()`.
- **L-2 [Med]** §3.8.2 says `otherwise` on a value that can never be missing
  is a compile error, but `let Circle(r) = shape otherwise return 0.0` has a
  plain `Shape` on the left. State that with a refutable pattern the left side
  may be any type.
- **L-3 [Low]** Nested tuple access `t.0.1` lexes as `t`, `.`, `0.1` (a
  `FLOAT`). Add: after a `.`, a digit sequence is always a single `INT`.
- **L-4 [Low]** `"{v is n: Int}"` has a colon the format rule takes as the
  format specifier. Note that such an expression needs parentheses.
- **L-5 [Low]** `return if c then a else b` is ambiguous with the postfix
  guard (§6.7). Rule: `if` right after `return`/`break`/`continue` is always a
  guard; parenthesize a conditional expression.

### §4 Types
- **T-1 [Low]** §4.9 and §9.3 say "small records"; §10.4 says copy is
  structural with no size limit. Drop "small" everywhere so that adding a
  field cannot silently change a type from copy to move through a size
  threshold.
- **T-2 [Med]** §11.1's `response.body  # moved out of response` is a partial
  move. State in §4.9: moving a field out of an owned value at its last use is
  allowed and the rest is freed; otherwise it is the usual move error.

### §5 Functions
- **F-1 [Med]** `10.seconds`, `5.millis` (§15.2, §20.2) look like field reads
  but would be uniform calls without parentheses, which §5.7 does not allow.
  Write `10.seconds()` and `5.millis()`. Update the examples and the §15.2
  table.
- **F-2 [Low]** §5.4 closing rule: say that one dedent closes every
  parenthesis opened by block lambdas on the lines it ends, that nothing can
  follow the closed call on that line (so `.sum()` needs a `let`), and that an
  explicit `)` is accepted and removed by `verse fmt` (§16.1).

### §8 Errors
- **E-1 [Low]** `ParseError` appears in §5.4 and §8.2 but is defined nowhere.
  Define it (in `std.num`), and state that `Text.to_int()` returns
  `Int | ParseError`.

### §9 Collections
- **K-1 [Low]** §9.5 ownership paragraph still lists `take` among the element
  methods; use `prefix`. Remove the keyword aside from the table row.
- **K-2 [Med]** `sort_by(.name)` on `Text` keys needs the key function to
  return a view. State that comparison key functions (`sort_by`, `min_by`,
  `max_by`) may return views; `group_by` stores its keys, so there the key must
  be owned.
- **K-3 [Low]** "Chains are lazy where it matters": say that no lazy
  iterator value can be stored or returned, and that a `let` bound to a chain
  is a local view (X1).
- **K-4 [Med]** Slices (`nums[1..]`, `name[0..=2]`) are views; `.copy()` makes
  an owned list or text.

### §7 Pattern matching (optional)
- **P-1 [Low]** Also warn when a catch-all binding is named like a variant of
  the matched type in lowercase (`north` when `North` exists), even in last
  position.

### §11 and §13
- **C-1 [Med]** `with counter.lock() as value` binds the *protected value*,
  not the resource, unlike `with open(…) as file` (§11.4 vs §13.1). Add the
  rule: the lock guard type implements `Cleanup`, and `with` on a guard binds
  the value inside as an exclusive borrow. This is the only such case and is
  documented as one.
- **C-2 [Low]** `spawn f(mut x)` must be a compile error, because a task
  cannot hand a `mut` borrow back. Say so in §11.1.

### §12 Modules
- **M-1 [Low]** The §12.3 example has a hidden field `balance` and an
  exported method `balance()` on the same type. Rule: fields and methods share
  one namespace per type. Rename the field in the example.
- **M-2 [Med]** Entry-file top-level `let`s and `fn` bodies: a function that
  reads a `let` declared below the call has no defined behaviour. Rule:
  entry-file top-level `let`s are script locals and are **not visible inside
  `fn` bodies**; pass them as arguments. This matches §14.1 (tests never run
  top-level statements) and keeps module constants in imported modules as the
  only module-wide values.

### §17 Grammar
- **G-1 [Med]** `arm_body` rejects the one-line arms used elsewhere:
  `_ -> pass` (§6.6), `none -> return 0`, `x -> total += 1`. Allow
  `( arm_expr | exit | "pass" | assign_stmt ) NEWLINE | block`.
- **G-2 [Med]** A guard or subject-less condition that is a lone name is
  parsed as a lambda: `n if ready -> …` and `urgent -> 15.0`. Use
  `expr - lambda` in the guard and in `cond_arm`, as `arm_expr` already does.
- **G-3 [Med]** Qualified names don't parse: `time.Duration` (§20.2's
  `elapsed: time.Duration`), `Shape.Circle(2.0)` (§4.4). `base_type` needs
  `{ IDENT "." } TYPE_IDENT`, and `postfix` needs `"." TYPE_IDENT`.
- **G-4 [Low]** `let n: Int = x otherwise return` matches both `let_stmt`
  forms (annotation or typed pattern). The second form should take `pattern`,
  not `top_pat`.
- **G-5 [Low]** §4.2 rule 1 lists trait constraints from operations; add
  indexing `[ ]` (an `Index` constraint), and "callable" for function-typed
  parameters.

### §18 Design notes and §21 log
- **D-1 [Low]** §18's first paragraph says syntax "was fixed by vote". Reword
  to "settled in the design discussion".
- **D-2 [Low]** §18.19 says `?` has "three related meanings"; §1.2 lists
  four. Say four.
- **D-3 [Med]** §18.10's trade-offs should state the chain rule from X1 once
  we agree, so the cost of the model is told in one place.
- **D-4 [Low]** §21: add a "Review 2 (Michelangelo)" table for the accepted
  items here, and rename the heading "Review 1 (Michelangelo)" intro, which
  calls it "the second review".

### §20 Examples (apply after X1–X4)
- **EX-1** `largest` (§20.3): write `fn largest<T: Measurable>(take items: List<T>): T?`.
  It consumes the list, so the result is a move and `shapes` is not used
  afterwards.
- **EX-2** `counts[clean] = …` (§20.1): `clean` is a view of `word`; under X1 it
  needs `clean.copy()` as the key.
- **EX-3** Annotate the boundary types in the teaching programs
  (`raw: List<Text>`, `text: Text`), since the inferred form would be a
  `verse.api` entry full of structural constraints for something that is
  plainly a list of text.
- **EX-4** `urls.zip(tasks)` (§20.2) works only because `tasks` is at its last
  use (a `Task` cannot be copied). Add a comment saying so.

## What I need from you to reach 100%
Decisions on X1, X3 and X4 (they change rules), a yes or no on M-2, C-1, F-1,
K-2 and K-4. Everything else is a wording or grammar fix that I'll check
against the files once they land. I'll re-review only the changed lines and
mark sections agreed in `00-README.md`.
