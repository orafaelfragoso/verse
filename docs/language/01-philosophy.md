## 1. Philosophy and design principles

Verse is a high-level, statically typed, general-purpose language whose single
overriding goal is **readability**. A Verse program should read like a short,
clear piece of prose: you should be able to show it to a colleague who has never
seen the language and have them follow along.

Everything else in the language follows from a handful of principles.

### 1.1 Code is read far more often than it is written

Verse optimizes for the reader. Keywords are short English words (`mut`, `if`,
`for`, `match`, `take`, `and`, `or`, `not`). Punctuation is used only where it
carries real meaning. Blocks are marked by indentation, so the visual shape of
the code *is* its structure.

### 1.2 Minimal punctuation, maximal meaning

There are no semicolons, no braces around blocks, and no parentheses around
conditions. The punctuation that remains stands for **one idea each**. Some
marks are used in several places, but always for the same idea — the table
lists every use honestly:

| Symbol | Meaning |
|--------|---------|
| `( )`  | "group": calls, parameters, tuples, grouping, variant fields |
| `[ ]`  | "position": list literals, indexing, slicing, list patterns |
| `{ }`  | "contents": map literals (`{"a": 1}`, empty `{:}`), set literals (`set{…}`), and — inside text — interpolation (`"{name}"`) |
| `< >`  | "type parameter": generics |
| `->`   | "produces": lambdas and match arms (nothing else) |
| `\|`   | "one of": variants, union and error types (`Int \| Error`), pattern alternatives |
| `?`    | "maybe missing or failed": `T?` (a type that may be missing), postfix `expr?` (stop early on error or missing), and the pattern `x?` (match a present value) |
| `:`    | "has type" or "is named": type annotations, return types, record fields, named arguments, map entries, supertraits and generic constraints (`T: Order`), typed patterns (`n: Int`), field patterns (`age: 18`), loop labels (`outer:`), and interpolation format specifiers (`{price:.2}`) |

### 1.3 Safe by default

- There is **no null**. A value that might be absent has an optional type `T?`,
  and the compiler makes you handle the absence.
- Errors are **values in the type**: a function that can fail says so in its
  signature (`Int | Error`), and failure must be handled or explicitly
  propagated with `?`. Silently discarding a fallible result is a compile-time
  error; `_ = …` is the visible way to ignore one on purpose.
- Types are **static but visually optional**: the compiler infers them —
  including function parameters and return types — and when it cannot decide,
  it stops and asks for one annotation at that exact spot. It never guesses.
- Bindings are **immutable unless marked `mut`**: `x = 1` binds a name that
  never changes, `mut count = 0` one that can. A name is bound once per
  function — there is no shadowing — so `x = …` on a name that already exists
  can only be an assignment to a `mut` binding.
- Every value has **exactly one owner** and is freed deterministically right
  after its owner's last use. There is **no garbage collector**, no manual allocation,
  no pointer or reference you can write down, no dangling reference and no
  use-after-free. Functions *borrow* their arguments unless they say `mut`
  (exclusive, mutable) or `take` (consume).
- Integer overflow, out-of-bounds list indexing and division by zero **stop the
  program with a clear message** rather than silently producing garbage.
- Concurrency is **race-free by construction**: a task owns what it is given;
  sharing between tasks happens only through opt-in handles (channels,
  `Shared`, `Guarded`).

### 1.4 One obvious way

Verse prefers one clear construct per idea. There is one loop over collections
(`for ... in`), one general branching construct for data (`match`), one way to
mark public items (`export`), and one official formatter (`verse fmt`) whose
output is the canonical style.

### 1.5 Scripts grow into programs

A Verse file is a script: statements run from top to bottom, no ceremony and no
`main` function required. The same file can grow functions, types, tests and modules without
ever being rewritten into a "real program" shape.

### 1.6 Tests and documentation are part of the language

Doc comments (`##`) and test blocks (`test "name"`) are first-class syntax, not
conventions. The tooling understands them directly.

### 1.7 Notation in this document

Code blocks are Verse unless marked otherwise. Inside examples, `...` stands for
code that has been left out for brevity; it is not Verse syntax. Comments of
the form `# 3` show what a line prints or evaluates to.

---

