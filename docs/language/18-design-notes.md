## 18. Design notes

Most of Verse's surface syntax (function and type declarations, `match`,
error types, optionals, modules, tests) is described in the chapters above. This section records the remaining decisions, and why.
Each follows the same tests: *Does it read like prose? Is there exactly one
reading? Is the safe behavior the default?*

### 18.1 Ranges

**Decision:** `a..b` excludes the end, `a..=b` includes it. Ranges are ordinary
values, work in `for`, `in`, slicing and patterns, and support `.step(n)` and
`.reverse()`. Open-ended forms (`a..`, `..b`, `..=b`) exist **only inside slice
brackets** (`items[2..]`, `items[..3]`), where the missing end is obvious.

**Why:** The half-open range is the right default for indexing
(`0..items.len()` never goes out of bounds) and splits cleanly
(`0..mid` and `mid..end` do not overlap). `..=` makes the inclusive case visible
instead of relying on a `- 1`. The notation is familiar from Rust, Swift and
Kotlin.

### 18.2 Text literals

**Decision:** Double quotes only; `{expr}` interpolation with optional `:format`;
triple-quoted multi-line text whose common indentation (set by the closing
`"""`) is stripped; raw text with an `r` prefix; escapes `\n \t \r \\ \" \{ \}
\0 \u{…}`. Single quotes are reserved for `Char`.

**Why:** One quoting style means no style debates. Indentation stripping lets
multi-line text sit naturally inside indented code. Raw text keeps regular
expressions and Windows paths readable. `\{` is needed because `{` always
starts interpolation in normal text.

### 18.3 Numeric and scalar types

**Decision:** Core types `Int` (64-bit, overflow-checked), `Float` (64-bit),
`Bool`, `Char`, `Text`, `Bytes`, and `Nothing`. Sized and arbitrary-precision
types live in `math`. No implicit conversions; `Int / Int` truncates toward
zero; division by zero and overflow panic.

**Why:** Most programs need one integer type and one floating type; fewer
choices means simpler code. Checked overflow is the safe default; the rare code
that wants wrapping says so by name. Implicit conversion is a classic source of
silent precision loss, so conversions are written out.

### 18.4 Booleans, fallback, and the word `otherwise`

**Decision:** Boolean operators are the words `and`, `or`, `not`, used only
with `Bool` operands (no `&&`, `||`, `!`, and no truthiness). The fallback
operator is a separate word, `otherwise`, which applies only to optional and
fallible values. Error and union types are written with `|`.

**Why:** One `or` for both "boolean or" and "fallback", resolved by the type
of the left operand, would be unambiguous to the compiler but not always to a
reader: in `a or b` you would have to know the type of `a` to know which
operation you were looking at. Giving each idea its own word removes the
guesswork: `or` is always logic, `otherwise` is always a fallback, and
`find_user(id).email otherwise "no email"` reads as a sentence. Using `|`
for fallible types (`Int | Error`) frees `or` from type position too and
matches the existing `|` in variant declarations — both say "one of these".
`otherwise` binds loosest of all binary operators because a fallback normally
applies to the whole expression to its left. Its right operand is restricted
to a literal, call, postfix expression, parenthesized expression or another
`otherwise`, so `x otherwise 0 + 1` — which would silently mean
`x otherwise 1` — is a compile-time error that asks for brackets.

### 18.5 Other operators

**Decision:** Comparison operators chain (`0 <= i < n`). `is` tests a pattern;
`in` / `not in` test membership. Assignment is a statement, never an
expression. No `?:` ternary operator (use the one-line `if a then b else c`, which reads as English), no `++`/`--`
(use `+= 1`), and no operator overloading except through the built-in traits
(`Eq`, `Order`, and the arithmetic traits `Add`, `Sub`, `Mul`, `Div`, `Rem`,
`Neg` listed in §10.4).

**Why:** Each removal eliminates a known source of bugs or puzzles
(`if (x = 3)`, `i++ + ++i`). Chained comparisons match how people write
mathematics.

### 18.6 Equality

**Decision:** `==` is structural equality for all types, automatically derived.
There is no identity operator. A `T` may be compared with a `T?` (the `T` is
treated as present). Comparing unrelated types is a compile-time error.

**Why:** With single ownership and no surface references, identity is not
observable (handles such as `Shared<T>` compare by the value inside), so one
equality is enough. Rejecting `1 == "1"` at compile time catches real mistakes.

### 18.7 Tuples and destructuring

**Decision:** Tuples are written `(a, b)`, accessed with `.0`, `.1`, and may
carry labels: `(low: Int, high: Int)`. Bindings, `for` and `match` destructure
with patterns. Only irrefutable patterns are allowed in bindings and `for`
(a binding may use a refutable pattern only with `otherwise return`). A
lambda with *N* parameters called with an *N*-tuple spreads it across the
parameters (`pairs.map((a, b) -> a + b)`).

**Why:** Tuples are perfect for returning two things without declaring a type;
labels keep that readable as code grows. Refusing refutable binding patterns
avoids a hidden panic. Auto-spread makes `zip`, `enumerate` and `entries`
pleasant without a special lambda-pattern syntax; it applies only when the
arity matches exactly, so it never guesses.

### 18.8 Loops

**Decision:** `while cond` with `break` and `continue`; optional loop labels
written **before** the loop, `outer: for row in grid`, and used as
`break outer` / `continue outer`. No `do-while`, no C-style `for`, no separate
infinite-loop keyword (`while true`).

**Why:** `for … in` covers almost all loops; `while` covers the rest. A label
written as `for row in grid as outer` would misuse `as`, which elsewhere
(`import m as n`) names the thing on its *left*, whereas a loop
label names the loop. A leading `outer:` uses the colon in its "is named"
sense (§1.2), reads like a heading, and is what readers of other languages
expect.

### 18.9 Immutability, ownership, no shadowing

**Decision:** Immutability is deep and belongs to the binding. There is no
`let`: `x = expr` binds a new immutable name, `mut x = expr` a mutable one,
and `x = expr` on an existing `mut` name reassigns it; on an existing
immutable name it is an error. `mut` means "can change" everywhere — bindings,
parameters, `self`. Every value has exactly one owner; assignment and passing
either copy (copy types), borrow (function parameters, local views), or move
(new owners), and moves are checked per control-flow path. `.copy()` — with no
arguments, or with field changes — is the single way to duplicate a value.
**Shadowing is removed:** a name is bound once per function; parameters, loop
variables and pattern bindings may not reuse a visible name, though sibling
blocks may reuse one. Only locals count: a local may hide a prelude, imported
or module-level name. Patterns always bind new names, except that a tuple of
visible `mut` names is a parallel assignment, so `(a, b) = (b, a)` swaps. An
unused binding warns, and so does a new binding one
edit away from a visible `mut` name. `let` stays reserved, with a friendly
error.

**Why:** With deep immutability and single ownership, a reader can understand a
function by reading only that function: nothing it borrows can change behind
its back, and nothing it owns is visible elsewhere. One duplication method
keeps the vocabulary small. Dropping `let` removes the most frequent keyword
in ordinary code, and the one word left, `mut`, marks exactly the names a
reader has to watch. The price is that without a declaration keyword,
`x = …` on a visible name cannot say whether it means "a new `x`" or "change
`x`" — so shadowing had to go: with it, shadowing and reassignment would look
identical. Without it, the rule is simple (an existing name can only be
reassigned, and only if it is `mut`), each name means one thing for the whole
function, and the typo warning catches the main risk of binding by writing,
a misspelt assignment that silently creates a new name.

### 18.10 Why no garbage collector

**Decision:** No garbage collector and no manual memory management. Memory is
managed by single ownership with **second-class borrows**: parameters borrow
by default, `mut` borrows exclusively, `take` consumes; borrows cannot be
stored, returned, or sent to another task; there is no reference or pointer
type and no lifetime syntax. Values are freed deterministically right after
their owner's last use, which also releases the files, locks and other
runtime-owned handles in them (§18.12); recursive types are boxed by the
compiler. Sharing is opt-in through `Shared<T>`, `Guarded<T>`,
`Weak<Shared<T>>` / `Weak<Guarded<T>>`, the channel ends `Sender<T>`
(copyable) and `Receiver<T>` (not copyable, single consumer), and `Task<T>`.

**Why:** Deterministic, pause-free memory and resource management, with the
same safety guarantees as Rust — no use-after-free, no data races — but
without lifetime annotations or elision rules, because a borrow can never
outlive the call that made it. Most everyday code (transforming data, calling
functions, building results) never mentions ownership at all.

**The honest trade-offs:**

- **Some programs need `.copy()`.** Because a borrow cannot be returned or
  stored, code that would hand out a view into a structure must copy instead
  (`p.name.copy()`), restructure to work in place with `mut`, or return an
  index. Copies of large values cost time and memory, and the compiler only
  removes copies it can prove unnecessary (a source at its last use is moved).
- **Collection chains pass views; the end of the chain decides.** Adapters
  such as `filter`, `sort_by` or `zip` hand along views of the elements and
  copy nothing; the chain ends in an owning step — `map` (the lambda must
  return an owned value), a reduction, a move from a source at its last use,
  or an explicit `.copy()`. `find`, `first`, `last`, `min` and `max` return
  views, and a name bound to a chain is a local view. A non-`Copy` element is
  never duplicated implicitly: where that would be needed, the compiler asks
  for `.copy()` (§9.5). The cost is that keeping a filtered list while the
  source is still in use needs a visible `.copy()`.
- **Ownership can flip with later code.** `b = a`, `for x in items`,
  pattern bindings and chain results are views when the source is used later
  and owners when it is the last use, so adding a line below can turn working
  code into an error. Every such error must name the later use that kept the
  source alive ("`person` is a view and cannot be stored", labelled "it is
  a view of `people`, which is used again on line 12"), §4.9.
- **Generic code is checked once.** Unannotated and generic parameter types
  are treated as opaque, non-`Copy` types during the ownership check, so a
  generic body sometimes needs a `Copy` or `Copyable` bound (or `.copy()`)
  where a concrete type would not (§4.2, §10.6). Unannotated getters are the
  exception that keeps types visually optional: returning a field read
  through a borrowed parameter infers a `Copy` bound, recorded in
  `verse.api`, and an unmet bound is a call-site error that names it.
- **Some designs need `Shared<T>` or arenas.** Graphs, caches and observer
  lists that would rely on many references in a GC language use `Shared`/`Weak`
  handles or index-based arenas. `Shared` handles are reference-counted, and a
  cycle of them leaks unless one link is `Weak`; the spec says so plainly
  (§4.10).
- **Tasks cannot borrow from their spawner** in 0.1; they receive owned values,
  copies, or handles.
- **Error messages carry the burden.** Every ownership error must point at the
  move and the later use, explain the rule in one sentence, and offer the fix
  (usually `.copy()`, `mut`, or `take`). This is a requirement on the
  compiler, not a nicety.

### 18.11 Missing values and errors in detail

**Decisions:**
- **Lookups.** Map indexing `map[key]` and `list.get(i)` return a real `V?`,
  while list indexing `list[i]` returns `T` and panics when out of range.
  Two-argument `get(key, default: x)` returns a plain `V` on both maps and
  lists; `default:` exists only on `get`. A bare `{}` is an error pointing to
  `{:}` and `set{}`. *Why:* a missing map key is ordinary data, but a bad list
  index is almost always a bug — and `list[i]` in a loop over `0..list.len()`
  should not need unwrapping. `get(k, default: 0) + 1` replaces the clumsy
  `(m[k] otherwise 0) + 1` for the most common update.
- **No silent flattening.** `T??` is a distinct type. Only lifted chains
  (plain dots through optionals, §8.1) flatten, because a chain only asks "was
  anything missing?". *Why:* on a
  `Map<Text, Int?>`, "key missing" and "present but `none`" must stay
  distinguishable.
- **Layers in written order.** `?` and `otherwise` act on the outermost layer:
  for `Text? | Error` the error side first, then a second `otherwise` (or `?`)
  handles `none`.
- **Union normalization.** A specific error type in a union with `Error` is
  absorbed by `Error`, and `?` never narrows `Error` back; `match` with a typed
  pattern recovers the specific type.
- **Patterns.** `match` on a union uses typed patterns (`n: Int`,
  `err: Error`) at the top level only; inside parentheses `name: pattern` is
  always a field. `p?` matches a present optional. There is no `if let`,
  and no `let` (§18.9). A catch-all lowercase
  arm that is not last makes later arms an *unreachable arm* error.
- **No `?.` operator; the consumer decides.** A plain dot looks through an
  optional only when the chain is immediately consumed by `otherwise`, a
  postfix `?`, an `is`/`match`, or a position whose type is optional; anywhere
  else it is an error. *Why:* `?.` added a symbol to every hop
  (`order?.customer?.address?.city`) while the safety came from the place that
  finally handles `none` — so that place now carries the decision, and the
  chain reads as plain prose. The safety is the same: a missing value can
  still never be used as if it were present, and the compiler points at the
  hop that may be missing. The cost is that the meaning of a dot depends on
  how its result is used, which the error messages explain. A `?`
  immediately followed by a dot is an error, since readers from other
  languages would take `a?.b` for safe access; "propagate, then access" is
  written `(a?).b`.
- `?` returns from the innermost function-like body: a lambda, a spawned
  function, a test, or the script. `?` on `T | E` in a function returning `U?`
  is an error, as is `?` on an optional in a function returning an error union
  (`.or_error("why")` converts). *Why:* errors should carry a message, and an
  error should never silently become `none`.
- **Discarding a fallible result is a compile-time error**, including values
  that contain fallible results (such as the list from `wait_all`). `_ = …`
  is the one visible escape hatch. *Why:* §1.3 promises that failure is
  handled or explicitly propagated; a warning would break that promise.

### 18.12 Resources close at their last use

**Decision:** A resource is an ordinary value. It is released when it is
dropped, right after its owner's last use, and no statement marks that point.
A drop runs no code of the program: it frees memory and releases
runtime-owned handles, and it never fails, panics or pauses the task. A close
that can fail is a named method, such as `file.finish()?`. A lock guard is an
ordinary value too, read with `get()` and changed with `put()` (§13, §11.4).

**Why:** One rule decides when every value ends, so a resource needs no second
spelling for "release here" (§1.4): using a file for the last time is what
closes it. Keeping the program's code out of drops keeps every drop
infallible and non-blocking, so a panic during unwinding cannot start a second
panic, and a cancellation never interrupts a drop and needs no mask. The cost
is that a resource the runtime does not own, such as a temporary directory,
is released only by its own method.

### 18.13 Concurrency details

**Decision:** Structured task lifetimes (unwaited tasks are awaited when their
spawning function ends), owned arguments moved into each task (borrows cannot
cross; handles copy cheaply), communication and sharing only through
channels (`Sender`/`Receiver`), `Shared`, `Guarded` and `Weak`, cooperative cancellation, and every task outcome — success, error, or
panic — delivered by `.wait()` as `T | Error`. A panic is always printed when
it happens and reaches the waiter as the named type `TaskPanicked`.
Cancellation unwinds the cancelled task like a panic (dropping what it owns)
and cannot be caught inside it; the waiter sees `Cancelled`. An unwaited task
that fails makes the process exit non-zero on natural program end (an explicit
`exit(code)` always wins). `rx.recv()` returns `T?`; a channel closes when
its last `Sender` is dropped (or early with `tx.close()`), and `tx.send(v)`
returns `Closed` once the receiver is gone.

**Why:** These rules make "no async/await" safe: tasks cannot leak, race, or
lose results silently, and neither `otherwise` nor `match` can hide a crash or
swallow a cancellation.

### 18.14 Modules without side effects

**Decision:** Only the entry file runs top-level statements, and top-level
statements are the *only* entry mechanism (no `fn main` in 0.1). Imported
modules contain declarations and lazily initialized constants whose
initializers must be infallible; in the entry file, a top-level binding is an
ordinary statement. `verse test` never runs entry-file statements. Everything
— including record fields — is hidden unless marked `export`. No wildcard
imports; `import x as y from m` renames, and every import line starts with
`import` (`import get, post from http`). Imports form no cycle.
Implementations (`impl Trait for Type`) live with the trait or the type.

**Why:** Importing a module or running its tests should never print, connect,
or write files. One entry mechanism and one visibility default (`export`, for
items and fields alike) leave nothing to remember. A lazy constant has no
caller, so it must not be able to fail. Without cycles, no module that a file
imports depends back on it, and the order in which constants are first read
never decides whether a program runs. Python and TypeScript allow cycles and
give a half-initialised module to whichever side loads second.

### 18.15 Smaller decisions

- `pass` for intentionally empty blocks (indentation needs *something*).
- `{:}` for an empty map: a bare `{}` would not say whether it is a map or a
  set; the colon is the map's signature. (`set{}` is the empty set.)
- `then` exists only for the one-line `if a then b else c`.
- `alias` for type aliases and unions of existing types; a variant name that
  collides with an existing type is an error, so `type Meters = Float` is
  rejected with a pointer to `alias`.
- `set` is a reserved word, so `set{…}` has exactly one reading.
- `Map` and `Set` both iterate in insertion order, so printing or looping over
  them gives the same result on every run and every target.
- `.copy()` is the one duplication method: with no arguments it makes a plain
  copy, with field arguments a copy with changes (`.copy(age: 31)`). There is
  no `clone`. `.copy()` comes from the
  built-in `Copyable` trait, which every type has except `Task`, lock guards,
  platform handles such as `File`, and anything containing them; `Copy` (implicit copying) is a separate,
  narrower trait.
- The collection methods are `prefix(n)` / `prefix_while(p)` rather than
  `take(n)` / `take_while(p)`, because `take` is a keyword (§5.3); `prefix`
  and `skip` read as a pair.
- Field punning works in construction, calls and patterns alike:
  `Dog(name:)`.
- Names are classified by their first character: uppercase-first names are
  types and variants; all others (lowercase, `_`, or uncased scripts) are
  values. Constants are `snake_case`.
- `print()` with no argument prints an empty line (`value` defaults to `""`).
- Doc comments: a `##` block followed by a blank line is the module doc; one
  directly above an item documents that item.
- Generic `impl` blocks declare their parameters: `impl<T> Box<T>`.
- A trait type such as `List<Speaker>` uses run-time dispatch, and the trait
  type itself implements the trait, so it can be passed to generic functions.
- **Foreign functions are plain calls.** An `extern` block (§12.6) declares C
  functions from a plugin named in `verse.toml`, and they are called like any
  other function. There is no capability gate, no special block and no `unsafe`
  marker, because Verse cannot see what foreign code does and a marker would
  promise a check that does not exist. Declaring the library is the opt-in.
  Only plain data crosses, and `Text` crosses as its `.bytes()`, so the
  conversion stays visible.
- **Reserved for 0.2: a fixed-size mutable byte buffer.** `Bytes` is
  immutable and `List<Int>` allocates, so bulk I/O on boards without a heap
  (SPI, UART, I2C reads) needs a buffer whose size is fixed when it is made.
  The same buffer would let a foreign function fill Verse memory. 0.1 adds no
  type for it.

### 18.16 Standard library and tooling

**Decision:** A compact, consistently named standard library of top-level
modules (`fs`, `net`, `http`, `text`, `time`, `json`, `task`, …) and a single
`verse` tool with a zero-configuration formatter, built-in test runner, doc
generator and package manager with a lock file and no install scripts.

**Why:** Consistency across the ecosystem is part of readability: every Verse
project is formatted, tested and built the same way.

### 18.17 Visually optional types

**Decision:** Static typing with annotations optional almost everywhere,
including parameters and return types. Unannotated parameters become implicit
generics with exactly the constraints their bodies need (trait and structural);
numeric literals default to `Int`/`Float`; recursion requires annotations;
fallible return types are inferred from `?`; ambiguity is a compile error at
the exact spot, listing the inferred constraints. Record fields keep
annotations unless a default implies the type; `mut`/`take` are always
written. Exported signatures may be inferred only with a committed `verse.api`
that the build checks, and inference never crosses a module boundary.

**Why:** Readers of prose do not want type noise, but programs still deserve a
compiler that catches mistakes before they run. Implicit generics give the
brevity of a dynamic language with static checking; the "never guess" rule and
the exact-spot errors keep inference predictable; and `verse.api` gives
libraries a stable, reviewable public surface even when the source omits types.
Editors show inferred types inline, so the information is one glance away.

### 18.18 Borrowed sugar

Verse borrows a handful of small conveniences from other languages, each
narrowed so that it has exactly one reading:

| Sugar | Form in Verse | Narrowed how | Inspired by |
|-------|---------------|--------------|-------------|
| Early exit on missing | `v = x otherwise return 0` (also `break`, `continue`) | exit only as the whole right operand; refutable binding patterns only in this form; with a fallible left side the error is discarded (use `.context(…)?` to keep it) | Swift `guard let`, Rust `let … else` |
| Uniform call syntax | `x.f(y)` calls `f(x, y)` | a method always wins; ambiguity is a compile error | D, Nim, Koka |
| Subject-less `match` | `match` + `n < 0 -> …` arms | final `_ ->` required; an expression | Kotlin `when`, Lisp `cond` |
| Spread in literals | `[0, ..rest]`, `{..defaults, "port": 80}` | prefix only; later map keys win | JavaScript, Python |
| Power operator | `a ** b` | right-assoc, tighter than unary minus; Int overflow-checked; negative Int exponent rejected | Python, Fortran |
| Key paths | `users.map(.age)` | plain access chains only; no operators; no leading-dot variants | Swift key paths |
| Lambda closing rule | `orders.map(order ->` + indented body | only when the lambda is the last argument; the dedent closes the `(` | — |
| Postfix guards | `continue if line.is_empty()` | only `return`/`break`/`continue`; one line; no `else`, no `unless` | Ruby, Perl |

**Why:** each one removes a common bit of noise (a nested `if`, a temporary
variable, a lonely `)`), and none introduces a second meaning for existing
syntax.

### 18.19 Rejected sugar

| Proposal | Why it was rejected |
|----------|---------------------|
| List comprehensions (`[x * 2 for x in xs if x > 0]`) | a second way to write `filter`/`map` chains, which already read left to right; nested comprehensions read inside out |
| Implicit lambda parameters (`$0`, `it`) | anonymous names hurt reading; key paths cover the common short case and named parameters cover the rest |
| Leading-dot variant names (`.north`) | would make a leading dot mean two things (key path or variant); variants are short enough to write |
| Multi-clause function heads (`fn fact(0) = 1`, `fn fact(n) = …`) | spreads one function over several declarations; `match` in one body is clearer and keeps one signature for inference and `verse.api` |
| `?` on method names (`items.empty?()`) | `?` already has three related meanings about missing or failed values (§1.2); a fourth, unrelated one would undo that |
| A pipe operator (`\|>`) | uniform call syntax already gives left-to-right chains with the existing dot |
| A `guard` keyword | `x = … otherwise return` does the same job with existing words |

---

