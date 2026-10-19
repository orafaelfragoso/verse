## 21. Review log and resolved decisions

After the first draft, a review pass revised several surface details. The
revisions below are **final** and are applied throughout this document; they
take precedence over anything in earlier drafts.

| # | Topic | Resolved decision | Replaces |
|---|-------|-------------------|----------|
| R1 | Boolean operators | `and`, `or`, `not` are used **only** with `Bool` operands. There is no `&&`, `\|\|` or `!`. | Earlier draft let `or` also act as a fallback. |
| R2 | Fallback operator | The word **`otherwise`**: `find_user(id)?.email otherwise "no email"`, `read_text(p) otherwise "default"`. | `or` as fallback. |
| R3 | `else` | `else` belongs only to `if`: block `if … / else` and the one-line `if a then b else c`. | — |
| R4 | Fallible / union types | Written with `\|`: `Int \| Error`. Optional stays `T?`. | `Int or Error`. |
| R5 | Return types | A colon: `fn greet(name: Text): Text`. Function types: `fn(Int): Int`. | `-> Ret`. |
| R6 | Uses of `->` | Only match arms and lambdas. A lambda that is the body of a match arm must be parenthesized: `Double -> (n -> n * 2)`. | `->` also for return types. |
| R7 | The `?` family | Postfix `?` = stop early if error or missing; `T?` = a type that may be missing; `?.` = chain past a missing value. | — (clarified) |
| R8 | Indentation | 4 spaces, tabs rejected, a block starts on the next line after its header, with no colon. | — (clarified) |
| R9 | One-line conditional | `if a then b else c` (both branches required). | Multi-line `if` as the only expression form. |
| R10 | Early exit | Explicit `return`; otherwise the last expression is the result. | — (clarified) |
| R11 | Named arguments | `name: value`, the same shape as map entries and record construction. | — (clarified) |
| R12 | Empty literals | `{:}` is an empty map, `set{}` an empty set, `[]` an empty list. | `{}` for an empty map. |
| R13 | Task errors | Errors (and panics) inside a spawned task surface at `.wait()`, which returns `T \| Error`, to be handled with `?`, `otherwise`, or `match`. | Panics re-raised at `.wait()`. |
| R14 | Imported files | Contain declarations only; top-level statements run only in the entry file. | — (confirmed) |

Consequences worked through during the review:

- **Variant vs union declarations.** Both use `|`. In `type Shape = Circle(…) |
  Square(…)`, the `|` introduces *new* variants. A union of *existing* types is
  named with `alias`: `alias Loaded = Config | Error`. Anywhere else in type
  position, `A | B` is a union.
- **Pattern alternatives** (`"y" | "yes" -> …`) also use `|`; they appear only
  in patterns, so there is no ambiguity with types.
- **Which members are errors.** In a union, members whose types implement the
  `Error` trait are the *error side*; the rest are the *success side*. `?` and
  `otherwise` act on the error side.
- **Precedence.** `otherwise` binds more loosely than every other binary
  operator, including `or`, so `a or b otherwise c` means
  `(a or b) otherwise c` — and is rejected anyway unless `a or b` is optional.
  (Review 1 added a restriction on the right operand — R16 — and the
  `get(k, default: x)` form — R41.)
- **`then`** became a keyword for the one-line conditional.

### Review 1 (Michelangelo)

Michelangelo's first review resolved the open items below. Each row names the review
item and the section where the change lives; these rows take precedence over
R1–R14 where they differ (notably R13 is refined by R17, and the implicit `?`
of `with` from the first draft is withdrawn by R18).

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R15 | 1, G7 | A variant name may not collide with a type in scope; `type Meters = Float` is an error that says "use `alias`". | §4.4, §17 |
| R16 | 2 | The right operand of `otherwise` must be a literal, call, postfix expression, parenthesized expression, or another `otherwise`; otherwise a compile error suggests brackets. | §3.8.2, §17 |
| R17 | 3, N3 | `TaskPanicked` is a named error type distinguishable in `match`; panic message and trace are always printed to stderr at the moment of the panic. | §8.5, §11.2 |
| R18 | 4 | `with open("log.txt")? as file`: `?` is required; `with` never propagates by itself. | §13.1–13.4, §18.12 |
| R19 | §1 | Punctuation stands for "one idea each"; multi-use marks (`:`, `{ }`, `?`) are listed honestly. | §1.2 |
| R20 | L1, G8 | Record copy-with-changes is `.copy(field: value)`. | §4.3 |
| R21 | L2 | `set` is a reserved word. | §3.6 |
| R22 | L3, G5 | Constants are `snake_case`; uppercase-first names are types/variants; names starting with any other letter (including uncased scripts) or `_` are values. | §3.5, §17 |
| R23 | L4 | A next line beginning with `\|` continues a variant list; a `type X =` header never opens a field block. | §3.4, §4.4, §17 |
| R24 | L5 | The interpolation format colon is the first colon outside any bracket or nested text. | §3.7.2, §17 |
| R25 | L6, G4 | `spawn` and `if … then … else` removed from the precedence table; `spawn` is a prefix on a call. | §3.8, §17 |
| R26 | L7, G5 | Open-ended ranges `a..`, `..b`, `..=b` only inside slices; `FLOAT` accepts `1e-9`. | §3.7, §6.4, §17 |
| R27 | L8 | A `##` block followed by a blank line is the module doc; directly above an item it documents the item; `verse fmt` enforces the blank line. | §3.2, §14.3, §17 |
| R28 | T1 | Map lookups and `list.get(i)` return a real `V?`; `T??` is not flattened except in `?.` chains. | §8.1, §9.3 |
| R29 | T2, E4 | `?` and `otherwise` act on the outermost layer in written order (`Text? \| Error`: error side first, then `none`). | §8.3 |
| R30 | T3, E4 | A specific error type in a union with `Error` is absorbed by `Error`; `?` cannot narrow `Error` back. | §4.6, §8.3, §11.1 |
| R31 | F1 | A lambda with *N* parameters may be called with an *N*-tuple (auto-spread). | §5.4, §18.7 |
| R32 | F2 | Field punning in construction and calls: `Dog(name:)`. | §4.3, §5.3, §17 |
| R33 | C1 | Loop labels are written before the loop: `outer: for row in grid`; `break outer`. | §6.5, §17, §18.8 |
| R34 | C2 | `if let` removed; use `if x is pattern`. | §6.2, §7.6 |
| R35 | P1, G3 | Inside parentheses `name: pattern` is always a field pattern; a typed binding is `field: n: Int`; top-level `n: Int` stays. | §7.3, §17 |
| R36 | P2, G9 | New pattern `p?` matches a present optional and binds it (`none -> …` / `user? -> …`, `Person(email: e?)`). | §7.3, §7.5, §17 |
| R37 | P3 | An unconditional lowercase catch-all arm that is not last makes later arms an "unreachable arm" error. | §7.2 |
| R38 | E1 | Discarding a fallible result (including containers such as `wait_all` results) is a compile error; `let _ =` is the escape hatch. | §1.3, §8.4, §11.2, §18.11, §17 |
| R39 | E2 | `?` returns from the innermost lambda, spawned function, test, or script. | §5.4, §8.3, §11.1, §14.1 |
| R40 | E3 | `?` on `T \| E` inside a function returning `U?` is an error. | §8.3 |
| R41 | K1 | Two-argument `get(key, default: x)` on maps and lists returns plain `V`; `default:` exists only on `get`. | §9.1–9.3, §20.1 |
| R42 | K2 | `m[k]` (`V?`) and `list[i]` (panics) shown side by side; bare `{}` is an error pointing to `{:}` and `set{}`. | §3.7, §9.3, §17 |
| R43 | N1 | `Cancelled` unwinds the task like a panic (running cleanup), cannot be caught by `otherwise` or `match` inside it, and is reported by `.wait()`; `sleep`, `receive`, `wait` have explicit signatures. | §11.5 |
| R44 | N2 | An unwaited failed task makes the process exit non-zero. | §11.3, §12.5 |
| R45 | N4 | `channel.receive()` returns `T?` (`none` when closed and drained). | §11.4 |
| R46 | M1 | Record fields are hidden unless marked `export`; `private` removed from the keywords. | §3.6, §4.3, §12.3, §17 |
| R47 | M2 | Imported-module `let` initializers must be infallible; entry-file top-level `let` is an ordinary statement. | §12.4 |
| R48 | M3 | `fn main` dropped for 0.1; top-level statements are the only entry mechanism. | §1.5, §12.5 |
| R49 | X1 | `verse test` never runs entry-file top-level statements. | §14.1, §20 |
| R50 | S1 | `read` removed from the prelude; `open`, `create`, `read_text` live only in `std.fs`; `print(value: Show = "")`, so `print()` prints an empty line. | §15.1 |
| R51 | S2 | The §8.2 example variant is renamed `NoResponse` to avoid clashing with `std.task.Timeout`. | §8.2 |
| R52 | G1 | `( "_" \| IDENT ) ":" type` typed patterns in the grammar. | §17 |
| R53 | G2 | `self.field = …` is a valid assignment place. | §5.6, §17 |
| R54 | G6 | `from m import x as y` exists. | §12.2, §17 |
| R55 | §10, §19, ToC | `Add`/`Sub`/`Mul`/`Div`/`Rem`/`Neg` rows in §10.4; softened the "fewer symbols" claim; §21 added to the contents. | §10.4, §19, Contents |
| R56 | addendum | Method calls are unmarked; the receiver of a `mut self` method must be a `let mut` binding (compile error otherwise). | §5.6 |

### Round 2 — follow-up decisions

| # | Topic | Resolution | Where |
|---|-------|------------|-------|
| Q1 | bare `spawn` | A bare `spawn worker(…)` statement that discards its `Task` is an accepted exception to "fallible results cannot be dropped": its outcome is still awaited and reported (§11.3). | §11.3 |
| Q2 | exit status | An explicit `exit(code)` always wins; the non-zero status for a failed unwaited task applies only on natural program end. | §11.3, §12.5, §18.13 |
| Q3 | `??` | Postfix `??` is not an expression (write `(x?)?`); `x??` exists only as a pattern meaning present-and-present. | §8.3, §7.3, §17 |

### Round 2 — user requirements

| # | Topic | Resolution | Where |
|---|-------|------------|-------|
| U1 | Visually optional types | Annotations are optional on parameters and return types (`fn greet(name)`); unannotated parameters are implicit generics with the constraints the body needs; numeric literals default to `Int`/`Float`; recursion needs an annotation; ambiguity is a compile error at the exact spot listing the inferred constraints, never a guess; fallible returns inferred from `?`; editors show inline hints and `verse fmt` never inserts annotations; record fields keep annotations unless a default implies the type; `mut`/`take` always explicit. The old "annotations required on all parameters" rule is removed. | §2, §4.2, §5, §16.1, §17, §18.17 |
| U1a | Exported signatures | May be inferred, only with (a) a committed `verse.api`, (b) a build that fails when an inferred exported signature differs until `verse api update` is run, and (c) cross-module checking against the interface file only. There is no "export requires explicit types" rule. | §4.2, §12.3, §16, §16.4, §17 |
| U2 | No garbage collection | Single ownership, deterministic freeing at end of scope, no references/pointers or lifetime syntax; parameters borrow read-only by default, `mut` (declaration and plain-call site) borrows exclusively, `take` consumes; borrows are second-class (not stored, returned, or sent to tasks); moves per control-flow path with use-after-move errors suggesting `.copy()`; scalars and records made only of copy types copy implicitly. RC, cycle collector and copy-on-write removed. | §1.3, §4.9, §4.10, §5.3, §18.9, §18.10, §19 |
| U2a | `.copy()` | `.copy()` with no arguments is the plain duplicate, the same method as `.copy(age: 31)`; there is no `clone`. | §4.3, §4.9, §18.15 |
| U2b | Lambdas | Non-escaping lambdas (`map`, `filter`, …) borrow captures read-only; escaping lambdas (returned, stored, `take`n, spawned) move or copy them. | §5.4 |
| U2c | Spawn | `spawn f(x)` moves `x`; borrowed values cannot cross into a task; spawned parameters are owned; reuse after spawn (also in a loop) is an error suggesting `.copy()`; handles copy cheaply; no borrowing from the spawner in 0.1. | §11.1 |
| U2d | Sharing | Opt-in only: `Shared<T>` (shared read-only, atomically reference-counted), `Guarded<T>` (mutable behind `.lock()`), `Weak<T>` (`upgrade(): T?`), `Channel<T>`, `Task<T>`. Cycles are possible only through these; graphs use `Shared` or arenas. Recursive types are boxed by the compiler. | §4.10, §11.4 |
| U2e | Cleanup | `Closeable` renamed `Cleanup`; `cleanup(mut self)` runs automatically at end of scope (also during a panic) and must not fail; `with … as` explicitly limits the scope. | §10.4, §13, §18.12 |
| U2f | `take` keyword | `take` is reserved; the collection method `take(n)` is renamed `prefix(n)` (`prefix_while(p)`). Assignment evaluates its right side before the place, so `m[k] = m.get(k, default: 0) + 1` is valid. | §3.6, §4.9, §9.5 |

### Sugar round

| # | Feature | Resolution | Where |
|---|---------|------------|-------|
| S-1 | `otherwise return` | `let v = x otherwise return X` (or `break`/`continue`): the exit must be the whole right operand (`x otherwise (return 1) + 2` is an error); unwraps `T?` to `T`; a refutable `let` pattern is allowed only in this form; with a fallible left side the error is discarded — use `.context(…)?` to keep it. A deliberate exception to the right-operand rule. No `guard`. | §3.8.2, §5.2, §7.7, §17 |
| S-2 | Uniform call syntax | `x.f(y)` calls an in-scope `f(x, y)` when the first parameter accepts `x`; a method always wins; ambiguity is a compile error; parameter modes carry over. No `\|>`. | §5.7, §17 |
| S-3 | Subject-less `match` | Arms are `Bool` conditions; a final `_ ->` arm is required ("not exhaustive" otherwise); it is an expression. | §7.8, §17 |
| S-4 | Spread in literals | `[0, ..rest]`, `set{..a, x}`, `{..defaults, "port": 80}`; prefix `..` never clashes with infix ranges; later map keys win. | §3.7, §17 |
| S-5 | `**` | Power operator: right-associative, tighter than unary minus (`-2 ** 2` is `-4`), overflow-checked for `Int`; a negative `Int` exponent is a compile error. | §3.8, §17 |
| S-6 | Key paths | A leading dot that starts an expression is a one-parameter lambda (`.name`, `.name.upper()`, `.0`); plain access chains only (`filter(.age >= 18)` is an error suggesting a lambda); after a complete expression a leading dot continues a chain; no leading-dot variants. | §3.4, §5.4, §17 |
| S-7 | Lambda closing rule | When a line inside an open `(` ends with `->`, the dedent ending the lambda body also closes that `(`; the lambda must be the last argument. No parenless trailing lambdas. | §3.4, §5.4, §17 |
| S-8 | Postfix guards | Only `return`, `break`, `continue` take a postfix `if cond`; one line, no `else`; no `unless`, no general postfix `if`. | §6.7, §17 |
| S-9 | Rejected | List comprehensions, `$0`/`it`, leading-dot variants, multi-clause function heads, `?` on method names, `\|>`, `guard` — with reasons. | §18.19 |

### Review 2 (Michelangelo)

Every item of the second review was accepted as proposed. These rows take
precedence over earlier rows where they differ (notably X1 replaces the
"filter copies its elements" rule of §9.5).

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| U2f-1 | `prefix`/`skip` | `take(n)`/`take_while` renamed `prefix(n)`/`prefix_while` — agreed; the keyword aside moved from the §9.5 table to §18.15. | §9.5, §18.15 |
| U2f-2 | Assignment order | Right side first; receiver and index expressions on the left after it, left to right among themselves; everything else left to right. | §4.9 |
| X1 | Chains pass views | Adapters (`filter`, `sort`, `sort_by`, `prefix`, `skip`, `unique`, `zip`, `enumerate`, `reverse`, …) pass views; the chain ends in an owning step (`map`, a reduction, a move from a source at its last use, or `.copy()`); `find`/`first`/`last`/`min`/`max` return views; a `let` bound to a chain is a local view; implicitly duplicating a non-`Copy` element is a compile error suggesting `.copy()`. Tour line: `people.filter(p -> p.age >= 18).map(.name.copy())`. | §2, §4.9, §9.5, §18.10 |
| X2 | Flip errors | Every error caused by a view/owner flip names the later use that kept the source alive. | §4.9, §18.10 |
| X3 | Generic ownership; `Copyable` | The ownership check runs once, at the definition, treating unannotated/generic parameter types as opaque non-`Copy` types unless bounded. New built-in trait `Copyable` provides `.copy()`, derived structurally, absent for `Task`, `Cleanup` types and anything containing them; `Copy` stays the implicit-copy marker. `first<T: Copyable>` uses `items.get(0)?.copy()`. | §4.2, §4.3, §5.5, §10.4, §10.5, §10.6, §15.1, §18.10, §18.15 |
| X4 | Argument order | Arguments are evaluated first; then the exclusive borrow for `mut` parameters and `mut self` begins. The §4.10 `Tree.add` example is valid under this rule. | §4.10, §5.3, §5.6 |
| L-1 | Spread | Spread moves from a source at its last use; otherwise a compile error suggesting `..xs.copy()`. | §3.7 |
| L-2 | Refutable `let` | With a refutable pattern, the left side of `otherwise` may be any type. | §3.8.2 |
| L-3 | `t.0.1` | After `.`, a digit sequence is always one `INT`. | §3.7, §17 |
| L-4 | Format colon | An interpolated expression with its own top-level colon needs parentheses: `"{(v is n: Int)}"`. | §3.7.2, §17 |
| L-5 | `return if` | `if` right after `return`/`break`/`continue` is always a postfix guard; parenthesize a conditional value. | §3.8, §17 |
| T-1 | "small records" | Dropped: `Copy` is structural with no size limit. | §4.9, §9.3 |
| T-2 | Partial moves | Moving a field out of an owned value at its last use is allowed; the rest is freed. | §4.9, §11.1 |
| F-1 | Durations | `10.seconds()`, `5.millis()` everywhere. | §15.2, §20.2 |
| F-2 | Closing rule details | One dedent closes every block-lambda parenthesis on the lines it ends; nothing may follow on that line; an explicit `)` is accepted and removed by `verse fmt`. | §5.4 |
| E-1 | `ParseError` | Defined in `std.num`; `Text.to_int()` returns `Int \| ParseError`. | §8.2, §15.2 |
| K-1 | `take` leftover | §9.5 ownership text uses `prefix`; keyword aside removed from the table. | §9.5 |
| K-2 | Key functions | `sort_by`/`min_by`/`max_by` keys may be views; `group_by` keys must be owned. | §9.5 |
| K-3 | Laziness | No lazy iterator value can be stored or returned; a `let` bound to a chain is a local view. | §9.5 |
| K-4 | Slices | Slices are views; `.copy()` makes an owned list or text. | §9.3 |
| P-1 | Variant-like catch-all | Warning for a catch-all binding named like a lowercase variant, even in last position. | §7.2 |
| C-1 | Lock guards | The guard implements `Cleanup`; `with` on a guard binds the value inside as an exclusive borrow — the only such case. | §11.4, §13.1 |
| C-2 | `spawn f(mut x)` | A compile error. | §11.1 |
| M-1 | Field/method names | Fields and methods share one namespace per type; example field renamed `cents`. | §12.3 |
| M-2 | Script locals | Entry-file top-level `let`s are not visible inside `fn` bodies; pass them as arguments. | §12.4 |
| G-1 | One-line arms | `arm_body` allows `arm_expr`, an exit, `pass`, or an assignment. | §17 |
| G-2 | Lone-name guards | Guards and subject-less conditions use `expr - lambda`. | §17 |
| G-3 | Qualified names | `base_type` takes `{ IDENT "." } TYPE_IDENT`; `postfix` takes `"." TYPE_IDENT`. | §17 |
| G-4 | `let … otherwise` | The second `let_stmt` form takes `pattern`, not `top_pat`. | §17 |
| G-5 | Inferred constraints | Indexing gives `Index`; calling a parameter makes it callable. | §4.2, §10.4 |
| D-1 | "by vote" | Reworded to "settled in the design discussion". | §18 |
| D-2 | `?` meanings | "Four related meanings". | §18.19 |
| D-3 | Trade-offs | §18.10 states the chain rule, flip errors and the generic check. | §18.10 |
| D-4 | Log | This table; the Review 1 intro no longer calls it "the second review". | §21 |
| EX-1 | `largest` | `fn largest<T: Measurable>(take items: List<T>): T?`; `shapes` is at its last use. | §20.3 |
| EX-2 | Word keys | `counts[clean.copy()] = …`. | §20.1 |
| EX-3 | Boundary types | `raw: List<Text>`, `text: Text` annotated. | §20.1 |
| EX-4 | `zip(tasks)` | Comment: works because `tasks` is at its last use. | §20.2 |
| S | Prelude `Error` | `Error` listed only as a trait; a trait name in type position means "any implementing type". | §15.1 |
| N-1 | `otherwise` and ownership | Literals and other static data count as views when needed; `a otherwise b` is a view when both sides are views (`args.first() otherwise "world"`); with an owned non-static temporary on the right the result is owned and a view on the left needs `.copy()` (compile error otherwise, suggesting it). Tour and §12.5 examples already conform. | §3.8.2, §8.3 |
| N-2 | `get(k, default: x)` | Returns a view of the element when present, of the default otherwise (static literals fine); owned positions need `.copy()`. | §9.3, §9.5 |
| N-3 | Guard cross-reference | §6.7 states that an exit followed by `if` is always a guard; write `return (if c then a else b)`. | §6.7 |
| N-4 | `find_user` | The M-2 comment (`users` is a module constant) is kept as is. | §8.1 |
| N-5 | `Copyable` closed | `Copyable` cannot be implemented by hand in 0.1; `Cleanup` types are deliberately non-copyable; a type needing duplication offers an explicit method such as `duplicate()`. | §10.4 |

### Review 3 (Michelangelo)

Re-review of the round-3 changes: all of Review 2 landed; N-3, N-4 and N-5
were agreed as stated, and N-1 and N-2 with the amendments below. All items
were accepted.

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R3-1 | `otherwise` table | New row: owned left + non-static view right is an error asking for `.copy()` on the right. A literal in an owned position is materialised — the one allowed implicit creation, since it duplicates no user value. `?.` on an owned temporary yields an owned field. `if c then a else b`, block `if` values and `match` arms follow the same rule: a view only if every branch is a view; otherwise owned, and view branches need `.copy()`. | §3.8.2, §6.1, §8.3 |
| R3-2 | `get(k, default: x)` | Same table as `otherwise`: a view with a view or static default; an owned temporary default (`Text.new()`) makes the result owned and the element side needs `.copy()`. | §9.3 |
| R3-3 | Single element from an owned source | On an owned source at its last use (`take items`, a temporary) the single element moves out and the rest is freed — the owned counterpart of the view rule. | §9.5, §20.3 |
| R3-4 | Collecting steps | `to_list`, `to_set`, `to_map`, `flat_map`, `flatten` are owning steps (move at last use, else `.copy()`); `chunks` and `windows` are views of slices. | §9.5 |
| R3-5 | Handles | `Shared`, `Guarded`, `Channel` are `Copyable`; `.copy()` copies the handle, not the value. Passing a handle to `spawn` needs an explicit `.copy()`; examples fixed. | §10.4, §11.1, §11.4, §15.2 |
| R3-6 | Handles always `Copyable` | `Shared<T>`, `Guarded<T>`, `Channel<T>`, `Weak<T>` are `Copyable` whatever `T` is, including resources (`Guarded<File>`), because copying a handle never duplicates the value; the "anything containing a `Cleanup` type" exclusion applies only to containment by value. | §10.4, §11.4 |
| R3-7 | `Weak<T>` | `Weak<T>` is `Copyable` (copies the weak handle) and needs an explicit `.copy()`, listed with the other handles. | §10.4, §11.1, §11.4, §15.2 |
| R3-8 | Explicit handle copies | §4.10 point 5: copying a handle must be written as `.copy()`. | §4.10 |
| R3-9 | Owned element, owned default | `names.get(id).copy() otherwise Text.new()` is the intended form; `.copy()` on an optional view `V?` of a `Copyable` `V` yields an owned `V?`. | §9.3 |
| R3-10 | `Weak` type parameter | `Weak` is parameterised over the handle type: `Weak<Shared<T>>` or `Weak<Guarded<T>>`, created with `h.downgrade()`. Supersedes the `Weak<T>` spelling in rows U2d, R3-6 and R3-7 above. | §4.10, §10.4, §11.1, §11.4, §18.10 |
| R3-11 | `upgrade()` type | `upgrade()` returns `Shared<T>?` or `Guarded<T>?` respectively (`none` if the value is gone). | §11.4 |
| R3-12 | `Weak<Channel<T>>` | `Weak` accepts only a `Shared` or `Guarded` handle; `Weak<Channel<T>>` is a compile error (channels are closed by dropping the last sender, not by weak reference). | §11.4 |

### Review 4 / Rafael's changes

Three language changes proposed by Rafael and agreed by Michelangelo, applied
throughout the spec (every section, grammar, tour, examples, stdlib table).
Earlier rows above are kept as history; where they use `let`, `?.` or the
`from m import x` form, the rows below supersede them (in
particular the `?.` wording of R3-1).

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| RF-1 | Remove `let`; keep `mut` | `x = expr` binds a new immutable name, `mut x = expr` a mutable one; `x = expr` on a visible `mut` name reassigns, on a visible immutable name it is an error ("`x` is immutable; write `mut x = …` where it is declared, or pick a new name"). Typed: `n: Int = 5`, `mut n: Int = 0`. | §1.3, §3.9, §4.9, grammar |
| RF-1a | Destructuring | `(a, b) = pair`; refutable patterns only with `otherwise return/break/continue` (`Circle(r) = shape otherwise return 0.0`); constructor patterns are told apart from bindings by capitalisation. | §3.8.2, §3.9, §7.7, §18.7, grammar |
| RF-1b | `mut` means "can change" | Same meaning on bindings, parameters and `mut self`. | §3.9, §4.9, §5.3, §5.6, §18.9 |
| RF-1c | No shadowing | A name is bound once per function; parameters, loop variables and pattern bindings may not reuse a visible name; sibling blocks may reuse a name; entry-file top-level bindings keep the M-2 rules. §18.9 rewritten (shadowing would be indistinguishable from reassignment). Supersedes the earlier "inner block may shadow" rule. | §1.3, §3.9, §5.3, §12.4, §18.9, grammar |
| RF-1d | Diagnostics | Unused binding warns; a new binding within edit distance 1 of a visible `mut` name warns ("did you mean to reassign `total`?"). | §3.9, §18.9 |
| RF-1e | `let` reserved | Stays in the keyword list with the error "`let` is not used in Verse; write `x = …`". | §3.6, grammar |
| RF-1f | Rewrites | All `let`/`let mut` uses rewritten; one renamed binding in §3.8.2 (`n` → `count`). | all sections |
| RF-2 | Remove `?.` | Plain dots look through optionals only when the chain is immediately consumed by `otherwise`, postfix `?`, an `is` pattern / `match` subject, or an optional-typed position (annotated binding, `return` in an optional-returning function, optional parameter). Elsewhere a dot on `T?` is an error suggesting `otherwise`/`?`/`is` (and annotating `T?` for an unannotated binding). | §3.4, §3.8, §8.1, §16.2, grammar |
| RF-2a | Lifted-chain rules | Short-circuits entirely on the first `none`; one flattened optional level (lookups like `m.get(k)` not flattened); ownership follows the view rule; the optional's own methods (`.expect()`, `.copy()` on `V?`, …) win over lifting; a name that is both an optional method and a field of the inner type is an error. | §8.1, §3.8.2, §9.3, §9.5 |
| RF-2b | `?` has three jobs | The `T?` type, postfix propagation, the `x?` pattern. | §1.2, §8.1, §18 |
| RF-2c | Examples and rationale | All `?.` examples rewritten (including `x?.copy()` in §9.3); §18 explains the drop (less noise, same safety, the consumer decides). | §2, §3, §5.5, §8, §9, §10, §16, §18 |
| RF-3 | Imports | `import get, post from std.net`; `import parse as parse_json from std.json`; per-name `as` in lists; `import std.fs` and `import std.json as js` unchanged; every import line starts with `import`. | §12.2, §18.14, grammar |
| RF-3a | Examples | All item imports rewritten (sections, tour, §11/§20 examples, stdlib table, packages). | §2, §8, §11, §12, §13, §15, §16, §18, §20 |
| RF-1g | Confirmed | A leading `_` suppresses the unused-binding warning; parameters count as bindings. | §3.9, §5.3 |
| RF-4 | Import parsing | Module vs item import (`import a as b` / `import a as b from m`) is decided by the trailing `from` on the same logical line; an import fits on one line, or uses a parenthesised item list that may span lines. | §12.2, grammar |
| RF-5 | Reassignment visibility | One form `x = …` is kept; editors and `verse fmt --check` tooling must render reassignments distinctly from new bindings (inlay hint "reassign"). No syntax change. | §3.9 |
| RF-6 | Scope of the no-shadowing rule | You cannot bind a name that is currently visible: a name bound inside a block may be bound again after the block ends; a name bound before a block cannot be rebound inside it. Example added. | §3.9 |
| RF-7 | Lambda parameters | Count as bindings; they cannot reuse a visible name. | §3.9 |
| RF-8 | Assigning a `mut` parameter | `list = […]` replaces the caller's value in place (the parameter is the caller's place under an exclusive borrow). | §3.9, §5.3 |
| RF-9 | Unused-binding warning | Warning only; illustrative snippets are exempt; the spec's examples are not compiled with warnings-as-errors. | §3.9 |
| RF-10 | `.or_error` on a lifted chain | Applies to the final result: a `none` anywhere becomes the error. | §8.1 |
| RF-11 | Significant parentheses | `verse fmt` never removes semantically significant parentheses; `(find_user(id)?).email` stays; parentheses end a chain, so `?` inside them propagates there. | §8.1, §16.1 |
| RF-12 | `MAX_RETRIES = 3` | Error reworded: names are `snake_case`; capitalised names are types and patterns. | §3.5 |
| RF-13 | `fn port(config)` | Kept; note that entry-file locals are not visible in functions (M-2), so this is not shadowing. | §12.4 |
| RF-14 | Channel closing | Ruled as "`close()` or last handle dropped"; superseded before it was applied by RF-15. The spec already read "last handle" in §11.4; no "last sender" wording existed outside the R3-12 history row, which is kept. | — |
| RF-15 | Channel split | `Channel.new<T>()` returns `(tx, rx)`: `Sender<T>` and `Receiver<T>`. The channel closes when the last `Sender` is dropped; `tx.close()` is an early shortcut that closes for everyone. Supersedes the `Channel<T>` handle in R3-5…R3-7 and R3-12. | §11.4 |
| RF-15a | Copyability | `Sender<T>` is `Copyable` (handle copy, explicit `.copy()`, also for `spawn`); `Receiver<T>` is not `Copyable` in 0.1 (single consumer; fan-out uses several channels) and moves into a task with `spawn`. | §4.10, §10.4, §11.1, §11.4, §15.2, §18.10 |
| RF-15b | Operations | `for value in rx` ends when the channel is closed and drained; `rx.recv(): T?`; `tx.send(v)` returns the error `Closed` if the `Receiver` was dropped or the channel closed. `Closed` added to `std.task`. | §11.4, §11.5, §15.2, §18.13 |
| RF-15c | `Weak` | `Weak` accepts only `Shared`/`Guarded`; `Sender` and `Receiver` are not weakable. | §11.4 |
| RF-15d | Examples and lists | §11.4 channel example rewritten (Senders copied per task, `tx` moved into `start_workers`, `for square in rx`); "channels" → "channel receivers" in the `Iterable` and `for` lists; §1.3 handle list. | §1.3, §6.3, §10.4, §11.4 |
| RF-16 | §3.9 error row | "a visible immutable binding or immutable parameter" — `mut` parameters are assignable (RF-8). | §3.9 |
| RF-17 | `with … as mut` on a lock | The name bound by `with g.lock() as value` is mutable only when written `as mut value`; §11.4 example fixed (`with counter.lock() as mut value`); rule stated with C-1 and in §3.9; grammar `resource` takes optional `mut`. Amends C-1. | §3.9, §11.4, §13.1, grammar |
| RF-18 | Explicit type arguments | Allowed on associated-function calls through a type name (`Channel.new<Int>(capacity: 10)`); normally inferred from an annotation (`(tx, rx): (Sender<Int>, Receiver<Int>) = Channel.new(capacity: 10)`). | §11.4, grammar |
| RF-19 | Channel capacity | `Channel.new<T>(capacity: Int = 16)`; `capacity: 0` is unbuffered (rendezvous). | §11.4 |
| RF-20 | Safe worker example | `start_workers(take tx, count)` spawns workers (each with `tx.copy()`) and returns the tasks without waiting; the caller consumes `for square in rx`, then waits. 20 items with capacity 4. New sentence: waiting for producers before consuming can deadlock once the channel is full; always consume while producers run. | §11.4 |
| RF-21 | Blocked senders | A sender blocked on a full channel wakes with `Closed` when the `Receiver` is dropped or `close()` is called. | §11.4 |
| RF-22 | `send` is fallible | Discarding its result is the usual compile error (§8.4). | §11.4 |
| RF-23 | §4.10 point 5 | `Receiver<T>` removed from the "two owners share one value" list; `Sender<T>` kept. | §4.10 |
| RF-24 | `Closed` | A named error type in `std.task`, like `Timeout`. | §11.4, §15.2 |
| RF-25 | Postfix `?` then dot | Postfix `?` is an ordinary postfix operator applied left to right: `find_user(id)?.email` means `(find_user(id)?).email` (propagate, then read) and is not optional lifting. Parentheses are allowed and equivalent there; they matter only when they end a lifted chain, and `verse fmt` keeps those. Amends RF-11. | §3.8, §8.1, §16.1, grammar |
| RF-26 | Type arguments only after a type name | `TYPE_IDENT "." IDENT type_args call`; a lowercase receiver never takes type arguments, so there is no ambiguity with chained comparison. Amends RF-18. | §3.8.3, grammar |
| RF-27 | Spawning in a chain | The §11.4 worker and `Guarded` examples end in `.to_list()`; `to_list` on a mapped range yields an owned `List<…>`, and `spawn` side effects run when the lazy chain is collected. | §9.5, §11.4 |
| RF-28 | `Guarded` example import | `import Guarded, wait_all from std.task`. | §11.4 |
| RF-29 | File writes need `mut` | `write_line`, `write`, `flush` take `mut self`; written files are bound `as mut target`. | §13.1, §13.3 |
| RF-30 | §4.10 point 5 | `Task<T>` removed from the sharing list (tasks are moved, never shared). | §4.10 |
| RF-31 | No explicit type arguments on free functions | Inferred, or given by annotating the binding (`n: Int = parse(text)?`); `parse<Int>(…)` is a compile error with that hint. | §10.5, grammar |
| RF-32 | Eager chains | A chain that is bound or returned is evaluated eagerly and collected into a list at the end of the statement (an owned `List<…>` when it ends in an owning step), and lambda side effects such as `spawn` run then; only a `for` target or a mid-chain use stays lazy. `.to_list()` is needed only where the chain would otherwise be consumed as something else, not for binding. RF-27's `.to_list()` additions reverted. Amends RF-27. | §9.5, §11.4 |

### Review 4 (Michelangelo full read)

Rulings on `reviews/review-4.md` (plus R4-14, added by the reviewer). Where
a row amends an RF row above, it supersedes it.

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R4-1 | Scope of "visible" | Locals only (parameters, earlier bindings, loop/pattern/lambda names in scope). A local may hide a prelude name silently and may hide a module constant or imported name; hiding an imported name that the same function also uses warns. A file-level import hides a prelude name of the same spelling (`import error from std.log`); there is no prelude qualifier, so import `std.log` as a module to keep both. Amends RF-6. | §3.9, grammar |
| R4-2 | `a?.b` | A `?` immediately followed by `.` is a compile error ("`?.` is not Verse; write `(a?).b` to propagate then read, or `a.b otherwise …` / `a.b?` to look through the optional"). `(a?).b` stays valid. Supersedes RF-25's "bare `x?.y` means `(x?).y`". | §3.8, §8.1, §16.1, §18.11, grammar |
| R4-3 | Getters | An unannotated function that returns a field, element or lookup read through a borrowed parameter gets an inferred `Copy` bound on the returned type, recorded in `verse.api`; an unmet bound is a call-site error naming it and suggesting `.copy()` inside, an annotation, or `take`. | §4.2 rule 9, §5.3, §18.10 |
| R4-4 | Raw text in the `std.json` example | Example uses a multi-line `r"""…"""` literal; §3.7.3 expands the raw-text rule (no escapes, no interpolation; layout as `"""`; single-line `r"…"` cannot contain `"`). Raw text was already in the grammar (`RAW_TEXT`). | §3.7.3, §15.2 |
| R4-5 | Destructuring | A pattern always creates new bindings (an existing name is an error pointing to `mut`); a tuple made only of visible `mut` names is a parallel assignment, right side first, so `(a, b) = (b, a)` swaps; mixing is an error. Swap added to the tour. | §2, §3.9, §7.7, grammar |
| R4-6 | Nested text in interpolation | One level deep; interpolation inside the nested literal is an error; style note: bind long ones to a name first (`verse fmt --check` suggests it). | §3.7.2, grammar |
| R4-7 | §4.9 copy/move sentence | Rewritten: `Copy` depends on the kinds of fields, not size; a copy is never inserted silently except for `Copy` types and literals; moves happen at last use; adding a `Text` field makes a record non-`Copy` and the compiler reports the places that relied on the copy. | §4.9 |
| R4-8 | `10.seconds()` | A number followed by `.` and an identifier is an `INT` and a method call; a float needs a digit after the dot; `10.` alone is an error. | §3.7, grammar |
| R4-9 | §8.1 ways out | "four ways out" (fallback, `?`, `is`, annotation); `.or_error` added to the tool table. | §8.1 |
| R4-10 | §3.8 row 1 | Notes moved to a list below the table; meaning unchanged. | §3.8 |
| R4-11 | `Map.with_default` | Declined. The counting idiom `counts[word] = counts.get(word, default: 0) + 1` is already in §9.2; no edit. | §9.2 |
| R4-12 | Nested optionals | Style note: avoid `T??` in public APIs; prefer a variant; `none?` / `score??` patterns are for generic code. | §8.1 |
| R4-13 | `.to_list()` in §11.4 | Confirmed absent from the worker example (removed by RF-32, which made it unnecessary); no edit. | §11.4 |
| R4-14 | Where errors appear | One sentence, stated identically in §4.2 rule 9, §5.3 (by the getter example) and §10.6: "Errors about a function's body appear at the definition. The only call-site errors are unmet bounds, written or inferred, and they name the bound." | §4.2, §5.3, §10.6 |
| R5-1 | §4.2 rule 9 help text | Help now says "return `person.name.copy()`, or take the parameter with `take`" (annotating would move the error to the definition). Supersedes "an annotation" in R4-3. | §4.2 |
| R5-2 | §16.1 formatter sentence | Rewritten as one list. | §16.1 |
| R5-3 | Import hiding a prelude name | Warns on the import line: "`error` hides the prelude `error`" (e.g. `import info, error from std.log`). §16 has no diagnostics list, so it is stated in §3.9 only. | §3.9 |
| R5-4 | Pointers to the hiding rule | One line each in §12.2 and §15.1 pointing to §3.9. | §12.2, §15.1 |
| R5-5 | §18.9 summary | Clauses added: only locals count; tuple of `mut` names is a parallel assignment (swap). | §18.9 |

### Round 7 (accepted rulings)

Rulings carried over from the architecture work (architecture/12 Q1, Q2, Q4
and 06 P6-2…P6-5). No section is reopened.

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R7-1 | `Set` order | `Set` iterates in insertion order, exactly like `Map`; a present value keeps its place, a removed-and-re-added one moves to the end (stated for `Map` too); set operations keep the left set's order, `union` appends the right set's new values. Replaces "unordered". | §9.2, §9.4, §18.15 |
| R7-2 | `Text` indexing cost and byte view | Indexing or slicing `Text` by position may take time linear in its length. `text.bytes()` is a read-only view of the UTF-8 bytes with constant-time indexing; each byte is an `Int` 0–255; view rules of §9.3; `.copy()` gives owned `Bytes`. | §4.1 |
| R7-3 | Unused imports | An unused imported name warns on the import line and cannot be silenced (no `_` escape); delete the name or the line. An import never adds code by itself. | §12.2 |
| R7-4 | Size and capability tools | Tool table rows: `verse build --size-report`, `verse why <item>`, `verse run --deny <capability>`, `verse check --target <T>`. | §16 |
| R7-5 | `[library]` and first-party packages | Optional `[library]` section (modules, capabilities, size budget, per-target availability); `[library.native]` link data reserved for 0.2. `finance`, `ml`, `web`, `gui`, `iot` are first-party optional packages, not `std`; `std.net.serve` stays in `std`. | §16.3 |
| R7-6 | Domain libraries | One sentence: they ship as optional packages, not in `std`. | §15.2 |
| R7-7 | `extern` | Reserved for 0.2: callable only inside a `with` block holding the matching system capability, plain-data types only; no syntax or word in 0.1. Draft semantics: `reviews/extern-semantics.md` (proposal). | §18.15 |
| R7-8 | Order rule stated | One sentence in §9.4 for `Map` and `Set` alike: replacing the value of an existing key keeps its position; removing a key and inserting it again puts it at the end. §9.2 uses the same words. Makes R7-1 explicit. | §9.2, §9.4 |

### Round 8 (architecture follow-ups)

Fixes carried over from architecture 19 (P19-8, P19-9), 20 (P20-6) and
Michelangelo's `reviews/review-6.md` (R6-2). No section is reopened.

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R8-1 | Package names | A package is imported by name, so its name looks like an identifier: `[a-z][a-z0-9_]*`. The §16.3 example is `word_tools`. | §16.3 |
| R8-2 | `verse why` | One command, `verse why <item-or-path>`, told apart by the form of its argument: an item gives the chain of uses from the entry statements; a file path gives the plugin that wrote it (architecture 07 §7.7). | §16 |
| R8-3 | `verse build` | Builds for the selected profile and target (dev or release executable, WebAssembly module, bytecode bundle), not only "an optimized native executable". | §16 |
| R8-4 | Variants of an imported type | Importing a type does not import its variants. A pattern looks a variant name up in the type being matched, so matching needs no import; building a value needs `import Circle from shapes` or the qualified `Shape.Circle(2.0)` (§4.4). The error names both fixes. Ruled in review-6 R6-2. | §12.2 |
| R8-5 | No-heap byte buffer | A fixed-size mutable byte buffer for bulk I/O on boards without a heap is reserved for 0.2, together with `extern`; 0.1 adds no type. | §18.15 |

### Round 9 (architecture follow-ups: conversions and exit statuses)

Fixes carried over from architecture 16 (§16.6, §16.6.1, run-time panic codes `R0012` and `R0013`
there, not to be confused with the `R` rows of this log) and architecture 15 (P15-13, P15-16, review-12).
No section is reopened. The spec itself names no catalog codes, so the sections below describe the behavior
in words.

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R9-1 | Failed numeric conversions | A conversion that does not fit panics: `to_int` of a `NaN` or of a `Float` too large for `Int`, an `Int` or `Float` to a narrower type, a non-Unicode value to `Char`. The message is "`{a}` cannot be converted to `{T}`" with a reason: too large, too small, not a number, or not a Unicode character. One sentence; the earlier wording of §3.8.4 said nothing about these cases, so nothing is replaced. | §3.8.4 |
| R9-2 | Exit statuses | The §12.5 list is a table aligned with architecture 16 §16.6.1: `0` normal; `1` a top-level `?` failed or an unwaited task failed; `2` errors in the source or a failed capability check, so the program did not start; `3` a bug in Verse (a compiler bug before the program started, or a trap inside the runtime that Verse caused), in `run` as in `build`; `70` a deadlock; `101` a panic. | §12.5 |
| R9-3 | `exit(code)` | The code is 0 to 255; any other value panics with "the exit code {n} is outside 0 to 255" on every platform, so `exit(256)` never reports success. `exit` may be called from any task and ends the program immediately: it flushes standard output and standard error, runs **no `Cleanup`** and does not wait for or cancel the other tasks. A program whose cleanups must run finishes the entry file or ends with a failed `?`. Ruling tie-broken by Bob, agreed by Picasso and Michelangelo (P15-13, `reviews/review-12-michelangelo-on-15.md`); Picasso's earlier proposal that `exit` run `Cleanup` and Michelangelo's interim amendment are withdrawn. A cleanup-aware exit is a 0.2 candidate (architecture 12). | §12.5, §15.1 |

### Round 10 (accepted rulings: cleanup, drop order, unwinding, NaN)

Rulings carried over from architecture 12 (Q14, Q15, Q16, Q18) and Michelangelo's
`reviews/review-13-michelangelo-on-12.md`. Q19 (the debugger step) is
architecture-only and changes no spec text. No section is reopened.

| # | Item | Resolved decision | Where |
|---|------|-------------------|-------|
| R10-1 | A `cleanup` may block | `cleanup` may block, like any other code, and nothing limits how long. A task blocked inside a cleanup still counts as blocked for deadlock detection; only `exit` ignores a blocked cleanup. Before this round the spec said only that `cleanup` must not fail. | §13.1 |
| R10-2 | Cancellation masked during a cleanup | A cancel request that arrives while a cleanup runs takes effect at the first blocking point after the cleanup finishes, so a cleanup always runs to completion. Before this round §11.5 did not say what cancellation does to a running cleanup. | §11.5 |
| R10-3 | Drop order | At the end of a scope, locals are freed in reverse order of declaration because a later local may borrow an earlier one; the parts of one value cannot borrow each other, so they are freed in declaration order (fields, tuple elements), by index (list elements) and in insertion order (`Map`, `Set`). Before this round §4.10 gave no order. | §4.10 |
| R10-4 | A parent that unwinds | A function that panics or is cancelled cancels its unwaited tasks and then waits for them. A child that ends only because of that cancellation is not an error and prints nothing; a child that fails with its own panic or error while unwinding is reported on standard error and counts for the exit status. The normal-return rule of §11.3 is unchanged: a child cancelled by an explicit `cancel()` and left unwaited is still reported, as before. | §11.3 |
| R10-5 | `NaN` and the sign of zero | Wherever the bits of a `Float` can be observed (reading them as an integer, serialization, `Show`) every `NaN` is the same quiet `NaN`, and `Show` prints it as `NaN`; the sign of zero is kept (`Show` prints `-0.0`); arithmetic is untouched. Hashing `-0.0` like `0.0` needs no wording: §9.4 already says `Float` is not `Hash` and cannot be a map key or set element (`OrderedFloat` is the escape), so the spec has no `Float` hash to define. | §3.8.4 |
