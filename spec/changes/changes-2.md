# Changes 2 — Round 2 decisions and sugar round

Where each round-2 decision is resolved in the spec. Section numbers follow
`21-review-log.md`; files are the per-section parts in `/workspace/verse/`
(§17 lives in `grammar.ebnf.md`). The *Main file* is where the rule is
stated; *Also touches* lists files that apply it, repeat it, or log it.

## Follow-up decisions (Q)

| # | Decision | Main file | Also touches |
|---|----------|-----------|--------------|
| Q1 | A bare `spawn worker(…)` that discards its `Task` is an accepted exception to "fallible results cannot be dropped" | [11-concurrency.md](../11-concurrency.md) (§11.3) | [21-review-log.md](../21-review-log.md) (Where corrected from §8.4, §11.3 to §11.3) |
| Q2 | `exit(code)` always wins; the failed-unwaited-task status applies only on natural program end | [11-concurrency.md](../11-concurrency.md) (§11.3) | [18-design-notes.md](../18-design-notes.md) (§18.13); [12-modules.md](../12-modules.md) (§12.5 now states that `exit(code)` overrides the failed-task status); [21-review-log.md](../21-review-log.md) |
| Q3 | No postfix `??` in expressions (write `(x?)?`); `x??` only as a pattern | [08-errors.md](../08-errors.md) (§8.3) | [03-lexical.md](../03-lexical.md) (§3.8); [07-pattern-matching.md](../07-pattern-matching.md) (§7.3); [grammar.ebnf.md](../grammar.ebnf.md); [21-review-log.md](../21-review-log.md) |

## User requirements (U)

| # | Decision | Main file | Also touches |
|---|----------|-----------|--------------|
| U1 | Types visually optional: implicit generics, literal defaults, recursion needs annotations, ambiguity errors, inferred fallible returns, inline hints, field annotation rule, `mut`/`take` explicit | [04-types.md](../04-types.md) (§4.2) | [02-tour.md](../02-tour.md) (both styles); [05-functions.md](../05-functions.md) (§5.1, §5.5); [16-tooling.md](../16-tooling.md) (§16.1); [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.17); [20-examples.md](../20-examples.md); [21-review-log.md](../21-review-log.md) |
| U1a | Exported signatures may be inferred only with a committed `verse.api`, build-time drift check, and cross-module checking against the interface file | [12-modules.md](../12-modules.md) (§12.3) | [04-types.md](../04-types.md) (§4.2); [16-tooling.md](../16-tooling.md) (command table, §16.3, §16.4); [18-design-notes.md](../18-design-notes.md) (§18.17); [21-review-log.md](../21-review-log.md) |
| U2 | No garbage collection: single ownership, deterministic freeing, borrow / `mut` / `take`, second-class borrows, per-path moves, use-after-move errors | [04-types.md](../04-types.md) (§4.9, §4.10) | [01-philosophy.md](../01-philosophy.md) (§1.3); [05-functions.md](../05-functions.md) (§5.3, §5.6); [06-control-flow.md](../06-control-flow.md) (§6.3 loop ownership); [18-design-notes.md](../18-design-notes.md) (§18.9, §18.10); [19-comparison.md](../19-comparison.md); [grammar.ebnf.md](../grammar.ebnf.md); [21-review-log.md](../21-review-log.md) |
| U2a | `.copy()` (no arguments) is the one duplication method; no `clone` | [04-types.md](../04-types.md) (§4.3, §4.9) | [02-tour.md](../02-tour.md); [05-functions.md](../05-functions.md); [09-collections.md](../09-collections.md); [11-concurrency.md](../11-concurrency.md); [18-design-notes.md](../18-design-notes.md) (§18.15); [20-examples.md](../20-examples.md); [21-review-log.md](../21-review-log.md) |
| U2b | Non-escaping lambdas borrow captures; escaping lambdas move or copy them | [05-functions.md](../05-functions.md) (§5.4) | [04-types.md](../04-types.md); [21-review-log.md](../21-review-log.md) |
| U2c | `spawn` moves arguments; borrows cannot cross into tasks; reuse after spawn is an error; handles copy cheaply | [11-concurrency.md](../11-concurrency.md) (§11.1) | [18-design-notes.md](../18-design-notes.md) (§18.13); [20-examples.md](../20-examples.md) (§20.2); [21-review-log.md](../21-review-log.md) |
| U2d | Opt-in sharing: `Shared<T>`, `Guarded<T>`, `Weak<T>`, `Channel<T>`, `Task<T>`; cycles only through these; recursive types boxed | [11-concurrency.md](../11-concurrency.md) (§11.4) | [04-types.md](../04-types.md) (§4.10); [15-stdlib.md](../15-stdlib.md) (§15.2); [18-design-notes.md](../18-design-notes.md) (§18.10); [19-comparison.md](../19-comparison.md); [21-review-log.md](../21-review-log.md) |
| U2e | `Closeable` → `Cleanup`; automatic at end of scope, also on panic, must not fail; `with` limits scope | [13-resources.md](../13-resources.md) (§13.1–§13.4) | [04-types.md](../04-types.md); [10-traits-generics.md](../10-traits-generics.md) (§10.4); [15-stdlib.md](../15-stdlib.md); [18-design-notes.md](../18-design-notes.md) (§18.12); [grammar.ebnf.md](../grammar.ebnf.md); [21-review-log.md](../21-review-log.md) |
| U2f | `take` reserved; `take(n)` → `prefix(n)`; assignment evaluates the right side before the place | [09-collections.md](../09-collections.md) (§9.5) and [04-types.md](../04-types.md) (§4.9) | [03-lexical.md](../03-lexical.md) (§3.6 keywords); [20-examples.md](../20-examples.md) (§20.1); [21-review-log.md](../21-review-log.md) |

## Sugar round (S)

| # | Decision | Main file | Also touches |
|---|----------|-----------|--------------|
| S-1 | `let v = x otherwise return/break/continue`; exit only as the whole right operand; refutable `let` only in this form; fallible left side discards the error | [03-lexical.md](../03-lexical.md) (§3.8.2) | [05-functions.md](../05-functions.md) (§5.2); [06-control-flow.md](../06-control-flow.md); [07-pattern-matching.md](../07-pattern-matching.md) (§7.7); [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-2 | Uniform call syntax `x.f(y)`; method wins; ambiguity is an error; no `\|>` | [05-functions.md](../05-functions.md) (§5.7) | [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-3 | Subject-less `match` with Bool arms and a required final `_ ->` | [07-pattern-matching.md](../07-pattern-matching.md) (§7.8) | [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-4 | Spread `..` in list, set and map literals; later map keys win | [03-lexical.md](../03-lexical.md) (§3.7) | [07-pattern-matching.md](../07-pattern-matching.md) (`..tail` patterns); [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-5 | `**` power: right-assoc, tighter than unary minus, Int overflow-checked, negative Int exponent rejected | [03-lexical.md](../03-lexical.md) (§3.8, §3.8.4) | [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.5, §18.18); [21-review-log.md](../21-review-log.md) |
| S-6 | Key-path shorthand `.name`, `.0`; plain access chains only; no leading-dot variants | [05-functions.md](../05-functions.md) (§5.4) | [03-lexical.md](../03-lexical.md) (§3.4); [09-collections.md](../09-collections.md); [11-concurrency.md](../11-concurrency.md); [20-examples.md](../20-examples.md); [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-7 | Lambda closing rule: the dedent closes the `(` opened on the `->` line; lambda must be last | [05-functions.md](../05-functions.md) (§5.4) | [03-lexical.md](../03-lexical.md) (§3.4); [16-tooling.md](../16-tooling.md) (§16.1); [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-8 | Postfix guards on `return`/`break`/`continue` only; one line, no `else` | [06-control-flow.md](../06-control-flow.md) (§6.7) | [05-functions.md](../05-functions.md) (§5.2); [20-examples.md](../20-examples.md) (§20.1); [grammar.ebnf.md](../grammar.ebnf.md); [18-design-notes.md](../18-design-notes.md) (§18.18); [21-review-log.md](../21-review-log.md) |
| S-9 | Rejected: list comprehensions, `$0`/`it`, leading-dot variants, multi-clause heads, `?` on method names, `\|>`, `guard` | [18-design-notes.md](../18-design-notes.md) (§18.19) | [21-review-log.md](../21-review-log.md) |

## Gaps found while mapping (fixed)

- Q1: the review log cited §8.4 for the bare-`spawn` exception, but only §11.3
  states it. Fixed: the log's "Where" column now reads §11.3.
- Q2: §12.5 listed `exit(code)` as an exit status without saying it overrides
  the failed-task status. Fixed: §12.5 now has one sentence saying so.
