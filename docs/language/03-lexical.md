## 3. Lexical structure and syntax

### 3.1 Source files

A Verse source file is UTF-8 text with the extension `.vs`. Each file is a
module (see [§12](12-modules.md#12-modules)). Line endings may be `\n` or `\r\n`.

### 3.2 Comments

```verse
# A line comment runs to the end of the line.

## A doc comment documents the item that follows it.
## It may span several lines and uses Markdown.
fn square(n: Int): Int
    n * n
```

- `#` starts an ordinary comment.
- `##` starts a doc comment. A block of `##` lines **directly above** a
  declaration (no blank line in between) documents that declaration (`fn`,
  `type`, `alias`, `trait`, `impl`, a module constant, a field, or a `test`).
- A `##` block at the very top of a file that is **followed by a blank line** is
  the *module* doc. `verse fmt` enforces the blank line, so the two cases always
  look different. See [§14.3](14-testing-docs.md#143-doc-comments).

There are no block comments; the formatter and editors handle commenting out
regions.

### 3.3 Indentation and blocks

Blocks are introduced by a line that ends a header (such as `if cond`,
`for x in xs`, `fn f()`, `type T`, `match v`) and consist of the following lines
that are indented further.

```verse
if temperature > 30
    print("Hot")
    print("Stay hydrated")
else
    print("Pleasant")
```

Rules:

- Indentation uses **spaces only**; a tab character in leading whitespace is a
  compile-time error. The canonical width is **4 spaces**.
- A block's lines must all share the same indentation.
- An empty block is not allowed; write `pass` when a block intentionally does
  nothing.

### 3.4 Line continuation

A statement normally ends at the end of a line. It continues onto the next line
when:

1. the line ends inside an open `(`, `[`, or `{`; or
2. the line ends with a binary operator (`+`, `and`, `or`, `otherwise`, `==`,
   `**`, …); or
3. the next line begins with `.` after a complete expression (a method
   chain); or
4. the next line begins with `|` (the next alternative of a variant list,
   §4.4).

A line that ends with `->` opens an indented block instead: the body of a match
arm, or — inside an open `(` — the body of a lambda. In the lambda case the
dedent that ends the block also closes the `(` opened on that line (the
*closing rule*, §5.4).

A leading dot that *starts* an expression (right after `(`, `,` or an
operator) is not a continuation but a key-path lambda such as `.name`
(§5.4).

A `type X =` header always continues on the same logical line; it never opens a
field block. Only a header of the form `type X` (or `type X<T>`) with nothing
after the name opens an indented block of fields.

```verse
report = orders
    .filter(o -> o.paid)
    .map(o -> o.total)
    .sum()

ok = user.active and
    user.verified

point = Point(
    x: 1.0,
    y: 2.0,
)
```

Trailing commas are allowed everywhere a comma-separated list appears.

### 3.5 Identifiers and naming

Identifiers start with a letter or `_` and continue with letters, digits or
`_`. Unicode letters are allowed. **The first character decides what kind of
name it is**, and the compiler relies on this:

- An identifier whose first letter is an **uppercase** letter (Unicode category
  Lu or Lt) is a *type-level name*: a type, variant, trait, or generic
  parameter.
- Every other identifier — starting with a lowercase letter, `_`, or a letter
  that has no case at all (for example `数量` or `مدينة`) — is a *value-level
  name*: a variable, constant, function, field, parameter, or module.

| Kind | Style | Example |
|------|-------|---------|
| Variables, functions, fields, modules | `snake_case` | `user_count`, `read_text` |
| Constants (module-level bindings) | `snake_case` (same as variables) | `max_retries` |
| Types, variants, traits, generic parameters | `PascalCase` | `HttpClient`, `NotFound`, `Speaker`, `T` |

There is no `SCREAMING_CASE`: `MAX_RETRIES = 3` is an error, because names
are `snake_case`; capitalised names are types and patterns. Within those two
kinds, `snake_case` and `PascalCase` are conventions enforced by
`verse fmt --check`.

A single `_` is a wildcard: it discards a value in patterns and bindings.

### 3.6 Keywords

```
alias      and        as         assert     break      continue   else
export     extern     false      fn         for        from       if
impl       import     in         is         let        match      mut
none       not        or         otherwise  pass       return     self
Self       set        spawn      take       test       then       trait
true       type       while      with
```

`let` is reserved but not used: Verse has no `let` (bindings are written
`x = …` and `mut x = …`, §3.9), and the compiler answers a `let` with the
error "`let` is not part of Verse" and the help "write `x = …`; use
`mut x = …` if it changes".

### 3.7 Literals

```verse
# Integers
42
1_000_000
0xFF
0o755
0b1010_0101

# Floats
3.14
6.02e23
1e-9                          # an exponent alone makes a Float
1_000.5
```

Right after a `.`, a digit sequence is always a single `INT`, never the start
of a `FLOAT`: `t.0.1` is the field `1` of the field `0` of `t` (nested tuple
access), not `t` followed by `.` and `0.1`. A number followed by `.` and an
identifier lexes as an `INT` and a method call (`10.seconds()`); a float needs
a digit after the dot (`10.0`), and `10.` alone is an error.

```verse

# Booleans
true
false

# Characters
'a'
'\n'
'é'

# Text
"Hello, world"
"Hello, {name}!"              # interpolation
"""
    Multi-line text.
    Leading indentation is stripped.
    """
r"C:\raw\path\{not interpolated}"   # raw text

# Bytes
b"GET / HTTP/1.1\r\n"

# Missing value
none

# Collections
[1, 2, 3]                     # List<Int>
{"ana": 30, "bruno": 17}      # Map<Text, Int>
{:}                           # empty map
set{"a", "b"}                 # Set<Text>
set{}                         # empty set
[0, ..rest]                   # spread a list into a list literal
{..defaults, "port": 80}      # spread a map; later keys win
(1, "one")                    # (Int, Text) tuple

# Ranges
1..10                         # 1 up to 9
1..=10                        # 1 up to 10
nums[2..]                     # open-ended: only inside a slice
nums[..3]
```

A bare `{}` is a compile-time error: it does not say whether a map or a set is
meant, and the message points to `{:}` (empty map) and `set{}` (empty set).

**Spread.** Inside a list, set or map literal, a *prefix* `..` spreads another
collection's elements into the new one: `[0, ..rest, 99]`,
`set{..admins, "root"}`, `{..defaults, "port": 80}`. In a map literal, when the
same key appears more than once, the **later entry wins**, so overrides go
after the spread. A prefix `..` (nothing on its left, right after `[`, `{` or
`,`) never clashes with the infix range operator `a..b`. Spreading **moves**
the elements out of the source, so the source must be at its last use (a
temporary, or a binding not used afterwards). If the source is used later, the
spread is a compile error that suggests spreading a copy, `..xs.copy()`;
nothing is copied silently (§9.5).

Open-ended ranges — `a..` (from `a` to the end) and `..b` / `..=b` (from the
start) — are allowed **only inside slice brackets**, such as `nums[1..]`,
`nums[..3]`, `name[..=2]`. Everywhere else a range needs both ends, so
`for i in 1..` is an error.

#### 3.7.1 Text escapes

| Escape | Meaning |
|--------|---------|
| `\n`, `\t`, `\r` | newline, tab, carriage return |
| `\\` | backslash |
| `\"` | double quote |
| `\{`, `\}` | literal braces (not interpolation) |
| `\0` | NUL character |
| `\u{1F600}` | Unicode scalar value by hex code |

#### 3.7.2 Interpolation

Inside an ordinary (non-raw) text literal, `{expr}` evaluates `expr` and inserts
its textual form (via the `Show` trait, [§10](10-traits-generics.md#10-traits-and-generics)). Any
expression is allowed. A text literal inside an interpolation is allowed one
level deep (`"{email otherwise "none"}"`); interpolation inside such a nested
literal is an error. An optional format
specifier follows a colon. **The format colon is the first `:` that is not
inside a bracket (`( )`, `[ ]`, `{ }`) or a nested text literal**, so
`"{f(x: 1)}"` and `"{ {"a": 1}["a"] }"` contain no format specifier, while
`"{price:.2}"` does. An expression whose own syntax has a top-level colon —
such as a typed pattern in `"{v is n: Int}"` — must be wrapped in parentheses,
`"{(v is n: Int)}"`; otherwise the colon is read as the format colon.

```verse
price = 1234.5
print("Total: {price:.2}")          # Total: 1234.50
print("Padded: {42:>6}")            # Padded:     42
print("Hex: {255:x}")               # Hex: ff
print("Contact: {email otherwise "none"}")
```

Style: keep nested literals short. A long expression or one with its own
text literals reads better bound to a name first
(`label = if count == 1 then "item" else "items"`, then `"{count} {label}"`),
and `verse fmt --check` suggests doing so.

#### 3.7.3 Multi-line and raw text

A triple-quoted literal `"""` begins a multi-line text. The opening `"""` must
end its line. The indentation of the closing `"""` is removed from every line,
so multi-line text can be indented naturally with the surrounding code.
Interpolation and escapes work as usual.

```verse
fn letter(name: Text): Text
    """
    Dear {name},

    Thank you for trying Verse.
    """
```

A raw literal `r"..."` or `r"""..."""` performs no escapes and no
interpolation, so `{` and `\` are ordinary characters — the form to use for
JSON, regular expressions and Windows paths. `r"""` follows the same layout
rules as `"""` (the opening quotes end their line; the closing indentation is
removed). A single-line `r"…"` cannot contain `"`.

### 3.8 Operators and precedence

From **highest** to **lowest** binding:

| Level | Operators | Associativity | Notes |
|-------|-----------|---------------|-------|
| 1 | `f(x)` `a[i]` `a.b` `a?` | left | postfix, applied left to right (notes below) |
| 2 | `**` | right | power; binds tighter than unary minus (§3.8.4) |
| 3 | `-x` | right | numeric negation |
| 4 | `*` `/` `%` | left | |
| 5 | `+` `-` | left | `+` also concatenates `Text` and lists |
| 6 | `..` `..=` | none | ranges |
| 7 | `==` `!=` `<` `<=` `>` `>=` `in` `not in` `is` | chained | see §3.8.3 |
| 8 | `not` | right | boolean negation |
| 9 | `and` | left | boolean, short-circuit |
| 10 | `or` | left | boolean, short-circuit |
| 11 | `otherwise` | left | fallback, short-circuit; restricted right operand (§3.8.2) |

Notes on level 1:

- A second postfix `?` needs brackets: `(x?)?`.
- `a.b` on an optional `a` follows the lifted-chain rule (§8.1).
- Postfix `?` is an ordinary postfix operator. A `?` immediately followed by
  `.` (`a?.b`) is a compile-time error: "`a?.b` is not part of Verse", with
  the help "write `(a?).b`, or `a.b otherwise …`". `(find_user(id)?).email` is valid: propagate a missing user out
  of the function, then read the field.

Three constructs are deliberately **not** operators in this table:

- `spawn` is a prefix that applies to exactly one call expression
  (`spawn fetch(url)`); its operand is never an arbitrary expression, so it has
  no precedence to compare ([§11](11-concurrency.md#11-concurrency)).
- `if a then b else c` is a complete expression form; each branch is a full
  expression, and the whole form extends as far to the right as possible. Wrap
  it in parentheses to use it inside a larger expression
  ([§6.1](06-control-flow.md#61-if--else)). Right after `return`, `break` or
  `continue`, an `if` is **always** a postfix guard (§6.7): `return if done`
  means "return when `done`". To return a conditional value, parenthesize it:
  `return (if c then a else b)`.
- A lambda `x -> expr` likewise extends as far to the right as possible. A
  key-path lambda (`.name`) is a primary expression (§5.4).

Prefix `..` (spread, §3.7) appears only directly inside a collection literal
and is not an operator either.

Assignment (`=`, `+=`, `-=`, `*=`, `/=`, `%=`) is a statement, not an
expression, so `if x = 3` is always a syntax error.

#### 3.8.1 Boolean operators: `and`, `or`, `not`

Boolean logic uses words, and **only** words: there is no `&&`, `||` or `!`.
`and`, `or` and `not` accept only `Bool` operands — there is no "truthiness",
so `if items` or `name or "x"` is a type error.

```verse
can_enter = is_member or has_ticket
valid = not name.is_empty() and age >= 18
```

`and` and `or` short-circuit.

#### 3.8.2 The fallback operator: `otherwise`

`left otherwise right` means **"use `left` if it is usable, otherwise
`right`."** The left operand must be optional or fallible:

| Left operand type | Fallback happens when… | Result type |
|-------------------|------------------------|-------------|
| `T?` | it is `none` | `T` if right is `T`; `T?` if right is `T?` |
| `T \| E` (`E` an error) | it is an error | `T` if right is `T`; `T \| E` if right is `T \| E` |

```verse
name = nickname otherwise "friend"                        # Text? → Text
timeout = settings["timeout"] otherwise 30                # Int? → Int
text = read_text(path) otherwise "default"                # Text | Error → Text
email = find_user(id).email otherwise "no email"          # User? → Text
source = read_text(primary) otherwise read_text(backup)       # still Text | Error
```

The right side is evaluated only when needed. Using `otherwise` on a value that
can never be missing or failed (a plain `Int`, a `Bool`) is a compile-time
error, because the fallback could never happen. The one exception is a
binding with a refutable pattern (below): there the fallback runs when the
pattern does not match, so the left side may be of any type
(`Circle(r) = shape otherwise return 0.0` with a plain `Shape`).

**Ownership of the result.** Either side of `otherwise` may be a view (a
borrow, an element from `find` or `m[k]`, §9.5) or an owned value. Text and
number literals and other static data count as views when needed, so they fit
either way. The result is:

| Left | Right | Result |
|------|-------|--------|
| a view | a view or a literal | a **view**: `args.first() otherwise "world"` |
| owned | owned, or a literal | **owned**: `read_text(path) otherwise "default"` |
| a view | an owned temporary (not static) | **owned**, so the left side must be copied: `m["k"].copy() otherwise make_default()` |
| owned | a non-static view | **owned**, so the right side must be copied: `read_text(path) otherwise fallback_text.copy()` |

Leaving out the `.copy()` in either of the last two rows is a compile-time
error that names the side to copy. A view result follows the usual rules: it
can be bound to a name as a local view or passed along, but storing or
returning it needs `.copy()`.

A literal in an owned position (the second row) is **materialised**: the
compiler creates an owned value from it. This is the one implicit creation
Verse allows, because it duplicates no user value. A lifted chain (§8.1) on an
owned temporary yields an owned field, so in
`find_user(id).email otherwise "no email"` (where `find_user` returns an owned
`User?`) the left side is owned and the result is an owned `Text`.

The same rule applies to every expression that chooses between branches:
`if c then a else b`, a block `if`/`else` used as a value, and the arms of a
`match`. The result is a view only if **every** branch is a view (or a
literal); otherwise it is owned, and each branch that is a view needs
`.copy()` — a compile-time error otherwise.

`otherwise` binds more loosely than every other binary operator. To keep that
from ever surprising a reader, **the right operand of `otherwise` must be** a
literal (including a negative number such as `-1`), a call, a postfix expression (a name, field access, index, or
a chain of them), a parenthesized expression, or another `otherwise`. Anything else is a
compile-time error:

```verse
n = counts["a"] otherwise 0 + 1          # error: write `(counts["a"] otherwise 0) + 1`
                                         #        or `counts["a"] otherwise (0 + 1)`
count = counts.get("a", default: 0) + 1  # clearer still, for maps (§9.2)
```

#### `otherwise return`, `otherwise break`, `otherwise continue`

As a deliberate exception to that rule, the right operand of `otherwise` may
also be one of the exits `return` (with or without a value), `break` or
`continue` — but only as the **whole** right operand, never nested inside a
larger expression:

```verse
fn label_for(id)
    user = find_user(id) otherwise return "unknown"         # User? → User
    "{user.name} ({user.age})"

for line in lines
    n = line.to_int() otherwise continue                    # skip non-numbers
    total += n

x = value otherwise (return 1) + 2                          # error: an exit must be the
                                                            #   whole right operand
```

When the left side is optional, the binding gets the unwrapped `T`. This is
the one form that also allows a **refutable pattern in a binding**, because the
failure case has somewhere to go:

```verse
fn radius_of(shape)
    Circle(r) = shape otherwise return 0.0
    r
```

With a **fallible** left side, `otherwise return …` (or `break`/`continue`)
*throws the error away* — the reason for the failure is lost. That is
sometimes exactly right (skip a malformed line); when the reason matters,
propagate it instead with `.context("…")?` (§8.4). There is no `guard`
keyword: `x = … otherwise return` is the guard.
#### 3.8.3 Comparison chaining

Comparisons can be chained and read like mathematics:

```verse
if 0 <= index < items.len()
    print(items[index])
```

There is no ambiguity with explicit type arguments: those are written only
after a capitalised type name and a dot (`Channel.new<Int>(capacity: 10)`,
§11.4), so `a.b < c > (d)` with a lowercase receiver is always a chained
comparison.

`a < b < c` means `a < b and b < c`, with `b` evaluated once.

#### 3.8.4 Arithmetic

- `Int` arithmetic is checked: overflow stops the program with a clear error.
  Wrapping and saturating arithmetic are available as methods
  (`a.wrapping_add(b)`, `a.saturating_mul(b)`).
- `/` on two `Int`s divides and truncates toward zero; `%` is the matching
  remainder. Dividing an `Int` by zero stops the program. `a.checked_div(b)`
  returns `Int?`.
- `/` on `Float`s follows IEEE 754.
- Wherever a program can observe the bits of a `Float` (reading them as an
  integer, serialization, `Show`), every `NaN` is the same quiet `NaN`, whatever
  the hardware produced, and `Show` prints it as `NaN`. The sign of zero is kept
  (`Show` prints `-0.0`); arithmetic itself is untouched.
- There are no implicit numeric conversions: write `n.to_float()` or
  `x.round().to_int()`.
- `a ** b` raises `a` to the power `b`. It is right-associative
  (`2 ** 3 ** 2` is `2 ** 9`) and binds tighter than unary minus, so
  `-2 ** 2` is `-4`, as in mathematics. On `Int` it is overflow-checked, and
  the exponent must not be negative: `2 ** -1` is a compile-time error (use
  `2.0 ** -1.0`), and a negative `Int` exponent computed at run time panics.
- A conversion never changes a value silently: converting a value that does not
  fit, such as `to_int` of a `NaN` or of a `Float` too large for `Int`, an `Int`
  or `Float` to a narrower type, or a non-Unicode value to `Char`, panics with
  "`{a}` cannot be converted to `{T}`" and a reason (too large, too small, not a
  number, or not a Unicode character).

### 3.9 Statements and expressions

Most constructs are expressions: `if … then … else …`, block `if`/`else`,
`match`, blocks, and calls all produce values. Declarations (bindings, `fn`,
`type`, `trait`, `impl`, `import`, `test`) and assignments are statements. The
value of a block is the value of its last expression.

#### Bindings and assignment

There is no `let`. A name is introduced by writing it, and `mut` marks a name
that can change:

```verse
total = 0                      # a new immutable binding
mut count = 0                  # a new mutable binding
count = count + 1              # `count` exists and is `mut`: reassignment
count += 1                     # the same
n: Int = 5                     # with a type annotation
mut ratio: Float = 0.0
(low, high) = bounds           # destructuring (§7.7)
Circle(r) = shape otherwise return 0.0   # refutable: only with `otherwise` (§3.8.2)
```

The rule for `name = expr` depends only on what is already visible:

| `name` is… | `name = expr` means |
|------------|---------------------|
| not yet bound in this function | a **new immutable binding** |
| a visible `mut` binding | **reassignment** |
| a visible immutable binding or immutable parameter | a compile-time error: "`total` is not `mut`, so it cannot be reassigned", with the help "write `mut total = …` at the first binding" |

`mut name = expr` always declares a new mutable binding. In a pattern, a name
starting with an uppercase letter is a constructor or variant and a lowercase
name is a new binding (§3.5), so `Circle(r) = shape` destructures and never
calls anything. A refutable pattern (one that might not match) is allowed only
with `otherwise return`/`break`/`continue`; without it, it is a compile-time
error.

**Destructuring always creates new bindings.** Every lowercase name in a
pattern on the left of `=` is a new binding, so a name that already exists is
an error ("`x` already exists in this scope"). To change `x`, declare it
`mut` and assign it on its own. The one exception is a
**parallel assignment**: a tuple made *only* of visible `mut` names assigns
them all at once, with the right side fully evaluated first, so
`(a, b) = (b, a)` swaps. Mixing new and existing names in one tuple is an
error.

```verse
mut a = 1
mut b = 2
(a, b) = (b, a)                # parallel assignment: a == 2, b == 1
(a, c) = (3, 4)                # error: `a` exists but `c` is new; do not mix
```

**No shadowing.** The rule is: **you cannot bind a name that is currently
visible**, where *visible* means **locals only**: parameters, earlier
bindings, and loop, pattern and lambda names in scope. Parameters (of functions and of lambdas), `for` loop variables,
pattern bindings in `match` arms and `if … is`, and `with … as` names all
count: each lives in its own block, but it may not reuse a name that is
already visible there. A name bound before a block cannot be bound again
inside it; a name bound inside a block may be bound again after that block
ends, because the inner one is out of scope. Sibling blocks — the branches of
one `if`/`else` or `match`, or two separate loops — may reuse a name, because
neither can see the other:

```verse
fn report(items)
    count = items.len()
    for item in items
        count = item.size          # error: `count` is visible here (immutable)
        size = item.size           # fine: new inside the loop
    size = 0                       # fine: the loop's `size` is out of scope
    items.map(count -> count + 1)  # error: lambda parameter reuses `count`
```

Without `let`, a second `x = …` would otherwise be impossible to tell apart
from an assignment, so Verse does not allow it at all; a transformed value
gets a new name (`raw = read_text(path)?` then `text = raw.trim()`).
Entry-file top-level bindings follow the script-local rules of §12.4.

Names from outside the function are not locals. A local may **hide a prelude
name** silently (`input = lines[0]`, `error = e.message()`) and may hide a
module constant or an imported name (`time = elapsed.seconds()` after
`import time`). Hiding an imported name
**warns** only when the same function also uses that import — an actual
collision ("`fs` hides the import `fs`").
The same holds one level up: a file-level import hides a prelude name of the
same spelling, so after `import info, error from log` the name `error`
means `log`'s function in that file, and the prelude `error` is not
reachable there (there is no qualifier for prelude names; import
`log` as a module, `log.error(…)`, to keep both). Because this changes a
name for the whole file, it **warns on the import line**: "`error` hides the
prelude `error`".
Names bound by `with … as` follow the same rule as other bindings: on a lock
guard, the bound value can be changed only when written `as mut value`
(§11.4).

A `mut` parameter is the caller's place under an exclusive borrow, so
assigning to it replaces the caller's value in place (§5.3):

```verse
fn reset(mut list)
    list = []                      # the caller's list is now empty
```

There is still one form, `x = …`, for both meanings, so tools carry the
difference: editors and `verse fmt --check` tooling must render a
reassignment distinctly from a new binding (for example an inlay hint
"reassign" before the name). This is a tooling requirement, not syntax.

Two warnings catch the usual slips:

- an **unused binding** warns (prefix the name with `_` to keep it on purpose);
  it is a warning only, and illustrative snippets in this document are exempt —
  the spec's examples are not compiled with warnings-as-errors;
- a **new binding whose name is one edit away from a visible `mut` binding**
  warns, because it is probably a typo for an assignment:

```
warning: new binding `totl` looks like the `mut` binding `total`
  --> sum.vs:6:5
   |
 2 |     mut total = 0
   |         ----- `total` is bound here
 6 |     totl = total + n
   |     ^^^^ this creates a new binding
   |
help: did you mean to reassign `total`? `total = total + n`
explain: V0313 (run `verse explain V0313`)
```

---

