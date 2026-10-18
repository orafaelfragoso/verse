# Changes 3 — Review 2 (Michelangelo)

Every item of `reviews/review-2.md` was accepted as proposed. This table maps
each item to the file and section where it is resolved. §17 lives in
`grammar.ebnf.md`. The same items are logged in `21-review-log.md` under
"Review 2 (Michelangelo)".

| Item | Resolution | Files (sections) |
|------|------------|------------------|
| U2f-1 `prefix` | `prefix`/`prefix_while` kept; keyword aside moved out of the §9.5 table into §18.15 | [09-collections.md](../09-collections.md) (§9.5), [18-design-notes.md](../18-design-notes.md) (§18.15) |
| U2f-2 assignment order | Right side first; left-side receiver and index expressions after it, left to right; everything else left to right | [04-types.md](../04-types.md) (§4.9) |
| X1 chains pass views | Adapters pass views; the chain ends in an owning step; `find`/`first`/`last`/`min`/`max` (and `min_by`/`max_by`) return views; a `let` bound to a chain is a local view; implicitly duplicating a non-`Copy` element is a compile error suggesting `.copy()`; tour line uses `.map(.name.copy())` | [09-collections.md](../09-collections.md) (§9.5), [02-tour.md](../02-tour.md) (§2), [04-types.md](../04-types.md) (§4.9), [18-design-notes.md](../18-design-notes.md) (§18.10) |
| X1 knock-ons | spread (L-1); slices (K-4); `find_user` → `?.copy()`; `first<T: Copyable>` → `items.get(0)?.copy()`; `largest` → `Order + Copyable` with `max()?.copy()` (§10.6) and `take items` (§20.3); tour claim reworded | [03-lexical.md](../03-lexical.md) (§3.7), [09-collections.md](../09-collections.md) (§9.3), [08-errors.md](../08-errors.md) (§8.1), [05-functions.md](../05-functions.md) (§5.5), [10-traits-generics.md](../10-traits-generics.md) (§10.5, §10.6), [20-examples.md](../20-examples.md) (§20.3), [02-tour.md](../02-tour.md) |
| X2 flip errors | Errors caused by a view/owner flip name the later use, with an example message | [04-types.md](../04-types.md) (§4.9), [18-design-notes.md](../18-design-notes.md) (§18.10) |
| X3 generic ownership, `Copyable` | Inference rule 9 (ownership checked once at the definition; generic/unannotated types opaque and non-`Copy` unless bounded); `.copy()` gives a `Copyable` constraint; `Copyable` row in the built-in traits; `Copy` vs `Copyable` explained; `first<T: Copyable>`; prelude lists `Copyable` | [04-types.md](../04-types.md) (§4.2, §4.3), [05-functions.md](../05-functions.md) (§5.5), [10-traits-generics.md](../10-traits-generics.md) (§10.4, §10.5, §10.6), [15-stdlib.md](../15-stdlib.md) (§15.1), [18-design-notes.md](../18-design-notes.md) (§18.10, §18.15) |
| X4 argument order | Arguments first, then the `mut` / `mut self` borrow begins; `Tree.add` example annotated as valid | [05-functions.md](../05-functions.md) (§5.3, §5.6), [04-types.md](../04-types.md) (§4.10) |
| L-1 spread | Moves from a source at its last use; otherwise an error suggesting `..xs.copy()` | [03-lexical.md](../03-lexical.md) (§3.7) |
| L-2 refutable `let` | Left side of `otherwise` may be any type with a refutable pattern | [03-lexical.md](../03-lexical.md) (§3.8.2) |
| L-3 `t.0.1` | After `.`, digits are always one `INT` | [03-lexical.md](../03-lexical.md) (§3.7), [grammar.ebnf.md](../grammar.ebnf.md) |
| L-4 format colon | `"{(v is n: Int)}"` needs parentheses | [03-lexical.md](../03-lexical.md) (§3.7.2), [grammar.ebnf.md](../grammar.ebnf.md) |
| L-5 `return if` | `if` after an exit is always a guard; parenthesize a conditional value | [03-lexical.md](../03-lexical.md) (§3.8), [grammar.ebnf.md](../grammar.ebnf.md) |
| T-1 "small" | Dropped; `Copy` has no size limit | [04-types.md](../04-types.md) (§4.9), [09-collections.md](../09-collections.md) (§9.3), [21-review-log.md](../21-review-log.md) (U2 row wording) |
| T-2 partial moves | Field moved out of an owned value at its last use; rest freed | [04-types.md](../04-types.md) (§4.9), [11-concurrency.md](../11-concurrency.md) (§11.1 comment) |
| F-1 durations | `10.seconds()`, `5.millis()`, `2.hours()` | [15-stdlib.md](../15-stdlib.md) (§15.2), [20-examples.md](../20-examples.md) (§20.2) |
| F-2 closing rule | One dedent closes all; nothing follows; explicit `)` accepted, removed by `verse fmt` | [05-functions.md](../05-functions.md) (§5.4) |
| E-1 `ParseError` | Defined in `std.num`; `Text.to_int(): Int \| ParseError` | [08-errors.md](../08-errors.md) (§8.2), [15-stdlib.md](../15-stdlib.md) (§15.2) |
| K-1 `take` leftover | §9.5 text uses `prefix`; table aside removed | [09-collections.md](../09-collections.md) (§9.5) |
| K-2 key functions | `sort_by`/`min_by`/`max_by` keys may be views; `group_by` keys owned (and `group_by` is an owning step) | [09-collections.md](../09-collections.md) (§9.5) |
| K-3 laziness | No lazy iterator stored or returned; `let` on a chain is a local view | [09-collections.md](../09-collections.md) (§9.5) |
| K-4 slices | Slices are views; `.copy()` for owned | [09-collections.md](../09-collections.md) (§9.3) |
| P-1 variant-like name | Warning even in last position | [07-pattern-matching.md](../07-pattern-matching.md) (§7.2) |
| C-1 lock guards | Guard implements `Cleanup`; `with` binds the value inside; the only such case | [11-concurrency.md](../11-concurrency.md) (§11.4), [13-resources.md](../13-resources.md) (§13.1) |
| C-2 `spawn f(mut x)` | Compile error | [11-concurrency.md](../11-concurrency.md) (§11.1) |
| M-1 one namespace | Fields and methods share a namespace; field renamed `cents` | [12-modules.md](../12-modules.md) (§12.3) |
| M-2 script locals | Entry-file `let`s not visible in `fn` bodies; example | [12-modules.md](../12-modules.md) (§12.4) |
| G-1 one-line arms | `arm_body` allows exit, `pass`, assignment | [grammar.ebnf.md](../grammar.ebnf.md) |
| G-2 lone-name guards | Guard and `cond_arm` use `arm_expr` (`expr - lambda`) | [grammar.ebnf.md](../grammar.ebnf.md) |
| G-3 qualified names | `{ IDENT "." } TYPE_IDENT`; `"." TYPE_IDENT` in `postfix` | [grammar.ebnf.md](../grammar.ebnf.md) |
| G-4 `let … otherwise` | Second form takes `pattern` | [grammar.ebnf.md](../grammar.ebnf.md) |
| G-5 constraints | `Index` from indexing, "callable" from calls; `Index` trait row | [04-types.md](../04-types.md) (§4.2), [10-traits-generics.md](../10-traits-generics.md) (§10.4), [15-stdlib.md](../15-stdlib.md) (§15.1) |
| D-1 "by vote" | "settled in the design discussion" | [18-design-notes.md](../18-design-notes.md) (§18 intro) |
| D-2 four meanings | "four related meanings (§1.2)" | [18-design-notes.md](../18-design-notes.md) (§18.19) |
| D-3 trade-offs | Chain rule, flip errors, generic check in §18.10 | [18-design-notes.md](../18-design-notes.md) (§18.10) |
| D-4 log | Review 2 table; Review 1 intro fixed | [21-review-log.md](../21-review-log.md) |
| EX-1 `largest` | `take items: List<T>`; `shapes` at its last use | [20-examples.md](../20-examples.md) (§20.3) |
| EX-2 word keys | `counts[clean.copy()]` | [20-examples.md](../20-examples.md) (§20.1) |
| EX-3 boundary types | `raw: List<Text>`, `text: Text` | [20-examples.md](../20-examples.md) (§20.1) |
| EX-4 `zip(tasks)` | Comment explaining the last-use move | [20-examples.md](../20-examples.md) (§20.2) |
| Prelude `Error` | Listed only as a trait; note on trait names in type position | [15-stdlib.md](../15-stdlib.md) (§15.1) |
| N-1 `otherwise` ownership | View when both sides are views (literals count as views); owned with an owned temporary on the right, then a left view needs `.copy()` | [03-lexical.md](../03-lexical.md) (§3.8.2), [08-errors.md](../08-errors.md) (§8.3); tour and §12.5 examples unchanged (already valid) |
| N-2 `get(k, default:)` | Returns a view of the element or of the default; `.copy()` for owned positions | [09-collections.md](../09-collections.md) (§9.3, §9.5) |
| N-3 guard cross-reference | Exit + `if` is always a guard; `return (if c then a else b)` | [06-control-flow.md](../06-control-flow.md) (§6.7) |
| N-4 `find_user` | M-2 comment kept; no change | [08-errors.md](../08-errors.md) (§8.1) |
| N-5 `Copyable` closed | Not implementable by hand in 0.1; `Cleanup` types deliberately non-copyable; explicit `duplicate()`-style method instead | [10-traits-generics.md](../10-traits-generics.md) (§10.4) |

`00-README.md` was not edited: Michelangelo had already marked §1, §6, §7,
§14, §16 and §19 as agreed, which matches his review.
