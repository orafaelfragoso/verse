# Verse spec review 1 (Michelangelo) — draft of /workspace/verse-spec.md (2,727 lines)

Legend: A = agreed, O = open (needs a decision or a fix before I can agree).

## Your four extra calls
1. `type X = A | B` always new variants, `alias` for unions of existing types — AGREED, plus one rule:
   `type Meters = Float` (one name, no `|`) must be a compile error that says "use alias",
   and any variant name that collides with an existing type (Int, Text, Error…) is also an error.
   The grammar (§17 type_decl) currently accepts both silently.
2. `otherwise` binds loosest — AGREED, plus a safety rule. `counts[w] otherwise 0 + 1` type-checks and
   silently means `counts[w] otherwise 1`. Rule: the right operand of `otherwise` must be a call, postfix
   expression, literal, parenthesized expression, or another `otherwise`; an unparenthesized `+ - * / and or
   == …` on the right is a compile error that suggests brackets.
3. Panic in a spawned task becomes `TaskPanicked` at `.wait()` — AGREED with two conditions:
   (a) `TaskPanicked` is a named type that `match` can tell apart from ordinary errors;
   (b) the panic message and trace are always printed to stderr at the moment of the panic, even if the
   waiter then handles it with `otherwise`, so `a.wait() otherwise "failed"` cannot hide a bug.
4. `with open(...) as file` checks acquisition without `?` — OPEN, I recommend requiring `?`:
   `with open("log.txt")? as file`. Reasons: §1/§8 promise that every early exit is marked with `?` or
   `return`; `with` is also used for infallible things (`counter.lock()`), so the word `with` does not
   signal failure; and the rule makes `with` mean cleanup plus propagation depending on the type, which is
   the same "word with two meanings that depends on types" problem we just removed from `or`. Cost: one character.

## Section status
1 Philosophy — O. §1.2 says each punctuation mark "has one job each", but `:` has about six (annotation, return
  type, field, named arg, map entry, format spec, supertrait/constraint, typed pattern), `{ }` three, `?` three.
  Say "one idea each" or list them honestly.
2 Quick tour — A (after §3.2 doc-comment adjacency rule below).
3 Lexical — O. See items L1–L8.
4 Types — O. T1–T3.
5 Functions — O. F1–F2.
6 Control flow — O. C1–C2.
7 Pattern matching — O. P1–P3.
8 Errors — O. E1–E4.
9 Collections — O. K1–K2.
10 Traits and generics — A (add `Add`/numeric traits to the §10.4 table, since §18.5 mentions them).
11 Concurrency — O. N1–N4.
12 Modules — O. M1–M3.
13 Resource management — O. Item 4 above.
14 Testing and docs — O. X1.
15 Standard library — O. S1–S2.
16 Tooling — A.
17 Grammar — O. G1–G9.
18 Design notes — O. Update 18.8, 18.11, 18.12 after the decisions below.
19 Comparison — A, but soften "fewer symbols than any of them" (not verifiable).
20 Examples — O. `print()` (S1), tests vs top-level code (X1), repeated `(m[k] otherwise 0) + 1` (K1).
   I checked the arithmetic and column layout of the 20.3 sample output; it is correct.
21 Review log — O. It is missing from the table of contents (which stops at 20), and it needs rows for
   everything decided below.

## Items

L1  `.with(age: 31)` (§4.3) uses `with`, which is a keyword (§3.6) and not an IDENT in the grammar.
    Proposal: rename to `.copy(age: 31)`; keep `with` only for resources.
L2  `set` is used as a literal prefix (`set{...}`) but is not a keyword. Make it reserved (or specify that
    `set{` with no space is the only reading).
L3  §3.5 allows SCREAMING_CASE constants, but §17 lexes an uppercase-first name as TYPE_IDENT, so
    `let MAX_RETRIES = 3` cannot parse. Also Unicode letters without case (e.g. Chinese) are neither.
    Proposal: constants are snake_case only; define TYPE_IDENT-vs-IDENT by convention enforced by the formatter,
    not by the lexer, or drop the lexer rule.
L4  Multi-line variant lists (§4.4 `type Json = Null` / `    | Flag(...)`) are not covered by §3.4 or the
    lexer rule in §17, and an indented line after a `type` header is the record-field block form.
    Proposal: add "the next line begins with `|`" to the continuation rules, and say a header ending in
    `= Name` never opens a field block.
L5  Interpolation `{expr:format}` — say the format colon is the first colon outside any bracket or nested string.
    Otherwise `"{f(x: 1)}"` and `"{ {"a": 1} }"` are ambiguous.
L6  §3.8 precedence table and §17 disagree about `spawn` and `if…then…else`. The table has spawn tighter than
    `if`, the grammar has `if` tighter than `spawn`. Since spawn's operand must be a call, remove both from the
    table and describe `spawn` as a prefix on a call.
L7  Open-ended ranges (`nums[1..]`, grammar `range_expr`) are used but not documented in §3.7 or §6.4.
    Also `..3` is not in the grammar. Decide both. (And `1e-9` in §4.1 is not matched by the FLOAT rule.)
L8  Doc comments: a `##` block directly above a declaration documents it, so §14.3's example and §2's tour
    (doc then `from … import`) are ambiguous with "module doc". Rule: a `##` block followed by a blank line is
    the module doc; a `##` block with no blank line documents the next item. `verse fmt` enforces the blank line.

T1  `T??` is flattened to `T?` (§8.1). That loses information for `Map<Text, Int?>` and `list.get(i)` on a list of
    optionals: "key missing" and "present but none" look the same. Proposal: flatten only for `?.` chains,
    and let map and list lookups return a real `V??`; or forbid optional map values. Decide.
T2  What does `?` or `otherwise` do on `Text? | Error`? Proposal: they act on the outermost layer in
    the written order: the error side first (the result is `Text?`), then a second `otherwise` handles `none`.
    State this in §8.3.
T3  A union that contains `Error` and also a specific error type (`Text | FetchError | Error`, which is what
    `.wait()` returns for a function that returns `Text | FetchError`) overlaps. Define normalization: a specific
    error type is absorbed by `Error`, and `?` cannot narrow `Error` back into `FetchError`.

F1  `(a, b) -> a + b` is a two-parameter lambda (§5.4), but `zip`, `enumerate`, and `entries` produce one tuple.
    Does `pairs.map((a, b) -> a + b)` destructure? Proposal: a lambda with N parameters may be called with an
    N-tuple (auto-spread). State it.
F2  Patterns allow field punning (`User(name:, age:)`) but construction does not (`Dog(name: name)` in §5.7).
    Proposal: allow `Dog(name:)` in construction too, or remove the shorthand from patterns.

C1  `for row in grid as outer` — `as` here names the loop, but in `with x as y` and `import m as n` it names the
    thing on its left. Proposal: label before the loop, `outer: for row in grid`, which matches the
    "`:` means is named" idea of §1.2. `break outer` stays.
C2  `if let x = opt` and `if value is Pattern` overlap, against §1.4 "one obvious way", and neither can say
    "present" inside a nested pattern. See P2.

P1  §17 `sub_pat = pattern | IDENT ":" [pattern]` is ambiguous: `User(age: Int)` is both a typed pattern
    binding `age` and a field `age` matching pattern `Int`. Proposal: inside parentheses `name: pattern`
    always means field; a typed pattern needs `field: n: Int`; top-level `n: Int` stays for fallibles.
P2  There is no pattern for "present optional". `user ->` after a `none ->` arm relies on arm order.
    Proposal: reuse the `?` family as a pattern: `user?` matches a present value and binds it, so `match` on `T?`
    reads `none -> …` / `user? -> …`, `if x is user?` replaces most `if let`, and nested forms
    (`Person(email: e?)`) work. Then `if let` can be dropped.
P3  A mistyped unit variant (`north ->` instead of `North ->`) becomes a catch-all binding. Add: a single
    lowercase identifier arm that is not last triggers an "unreachable arm" error for the arms after it.
    (Exhaustiveness already helps; say it explicitly.)

E1  §18.11 says discarding a fallible result is a warning. §1.3 promises "failure must be handled or
    explicitly propagated". Make it an error (the spec already has `let _ =` as the escape hatch).
    Also cover containers: `wait_all(tasks)` (§11.4) throws away a `List<T | Error>` and loses every error.
E2  State that `?` inside a lambda returns from the lambda, not from the enclosing function, and what
    `?` inside `spawn`'d functions and `test` blocks does (the last is already stated).
E3  `?` on `T | E` inside a function that returns `U?` is not covered by the §8.3 table. Add the row (error).
E4  See T2 and T3.

K1  `(counts[w] otherwise 0) + 1` appears three times (§3.8.2, §9.2, §20.1). It is the most common map
    update and the least readable line in the examples. Proposal: `counts.get(w, default: 0) + 1`
    (a two-argument `get` on maps and lists) and keep `otherwise` for real optionals.
K2  `m[k]` returns `V?` but `list[i]` panics. §18.11 explains why; I accept it, but §9.1 and §9.2 should show
    the two side by side. `{}` on its own should be a compile error whose message points to `{:}` and `set{}`.

N1  Cancellation (§11.5) is a `Cancelled` error from blocking operations, so `get(url) otherwise "x"` swallows
    it and the task keeps running. Proposal: `Cancelled` is not catchable by `otherwise` or `match`; it unwinds
    the task like a panic (running `with` cleanup) and `.wait()` reports it. Also give `sleep`, `receive`, and
    `wait` explicit return types.
N2  §11.3: if an unwaited task fails, its error goes to stderr. Define the process exit status: non-zero.
N3  See item 3 above (always print panics).
N4  `channel.receive()` on a closed channel: return type is not stated (`T?` is the obvious choice).

M1  §12.3: items are private by default and need `export`, but fields of an exported record are public by
    default and need `private`. Inverted defaults. Proposal: fields are hidden unless marked `export`
    (same word, same default), and remove `private` from the keyword list.
M2  §12.4: imported modules have lazily initialized `let` constants, but the entry file's `let` is eager and
    ordered. Same syntax, two meanings. Also a lazy initializer that fails has nowhere to return to.
    Proposal: imported-module `let` must have an infallible initializer (no `?`), and the spec says plainly
    that entry-file top-level `let` is an ordinary statement.
M3  `fn main` plus top-level statements is two entry mechanisms (§12.5). Not blocking; consider dropping
    `fn main` for 0.1.

X1  `verse test` on an entry file such as §20.1 or §20.2: do top-level statements run? §20.1 reads files and
    §20.2 hits the network and calls `exit(1)`. Rule: `verse test` never runs entry-file top-level statements;
    tests see declarations and module-level constants only.

S1  The prelude has `read`, `open`, and `create`, and `std.fs` has `read_text`, `open`, `create`. Two names for
    one idea (§1.4). Proposal: remove `read` from the prelude (keep `open`/`create` only in `std.fs`).
    Also §20.1 calls `print()` with no argument, but the prelude signature has a required `value`.
S2  `Timeout` is a variant in the §8.2 example and a type in `std.task`. Rename the example variant.

G1  `single_pat` has no `_: Error` (used 4 times in tests); add `( "_" | IDENT ) ":" type`.
G2  `place = IDENT {…}` cannot start with `self`, so `self.age = 3` in a `mut self` method does not parse.
G3  See P1 (sub_pat ambiguity).
G4  See L6 (spawn and cond order).
G5  See L3 (TYPE_IDENT lexing) and L7 (FLOAT exponent).
G6  `import` grammar lacks `from … import x as y`; decide whether it exists.
G7  `type_decl` accepts `type X = Foo`; see item 1.
G8  `.` followed by a keyword (`.with`) — see L1.
G9  Add the new `?` pattern if P2 is accepted.
