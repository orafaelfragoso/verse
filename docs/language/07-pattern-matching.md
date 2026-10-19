## 7. Pattern matching

### 7.1 `match`

`match` compares a value against patterns, top to bottom, and evaluates the arm
of the first pattern that fits. Each arm is `pattern -> expression`, or
`pattern ->` followed by an indented block.

```verse
type Shape = Circle(radius: Float) | Square(side: Float)

fn area(s: Shape): Float
    match s
        Circle(r) -> 3.14 * r * r
        Square(side) -> side * side

fn describe(s: Shape): Text
    match s
        Circle(r) ->
            size = if r > 10.0 then "large" else "small"
            "a {size} circle"
        Square(_) -> "a square"
```

`match` is an expression; all arms must produce compatible types. An arm whose
body is a lambda must parenthesize it — `Double -> (n -> n * 2)` — because the
first `->` after a pattern always ends the pattern (see §5.4).

### 7.2 Exhaustiveness

The compiler checks that every possible value is handled. Forgetting a variant
is a compile-time error that names the missing case:

```
error: this `match` does not cover `Triangle(_, _, _)`
  --> shapes.vs:4:5
   |
 4 |     match s
   |     ^^^^^^^ add an arm for `Triangle(_, _, _)`
   |
help: add `Triangle(_, _, _) -> …` or a final `_ -> …`
explain: V0601 (run `verse explain V0601`)
```

Use `_` to explicitly handle "everything else".

**Unreachable arms.** An arm whose pattern is a single lowercase name (or `_`)
with no guard matches everything. If such an arm is **not the last arm**, every
arm after it is reported as an *unreachable arm* error. This catches a common
slip — writing `north ->` instead of `North ->`, which would otherwise bind a
new name and swallow every case:

```
error: this arm can never match
  --> compass.vs:5:9
   |
 4 |         north -> "up"
   |         ----- this arm matches every value (`north` is a new binding)
 5 |         South -> "down"
   |         ^^^^^ an earlier arm already matches everything it matches
   |
help: delete it
help: did you mean the case `North`?
explain: V0602 (run `verse explain V0602`)
```

Even in the **last** position, where a catch-all is legal, a binding whose
name is the lowercase spelling of a variant of the matched type (`north` when
`North` exists) gets a warning that suggests the variant or a different name.

### 7.3 Kinds of patterns

| Pattern | Matches | Example |
|---------|---------|---------|
| Literal | an equal value | `0`, `"quit"`, `'a'`, `true` |
| Binding | anything, naming it | `x` |
| Wildcard | anything, discarding it | `_` |
| Variant | a variant, destructuring fields by position | `Circle(r)` |
| Named fields | a variant or record by field name | `User(name:, age: 18)` |
| Tuple | a tuple of matching parts | `(0, y)` |
| List | a list by shape | `[]`, `[x]`, `[first, ..rest]` |
| Range | a number or char within range | `0..=9`, `'a'..='z'` |
| Alternatives | any of several patterns | `"y" \| "yes"` |
| Typed | a union member by type (top level only) | `n: Int`, `err: Error`, `_: Error` |
| Present | a present optional, binding its value | `user?`, `_?` |
| `none` | an absent optional | `none` |

**Field patterns.** Inside the parentheses of a variant or record pattern,
`name: pattern` **always** means "the field `name` must match `pattern`", and
`name:` alone is punning for `name: name` (bind the field to a variable of the
same name). `User(name:, age: 18)` binds `name` and requires `age` to equal 18.
Fields not mentioned are ignored.

**Typed patterns** (`n: Int`) are allowed only at the *top level* of an arm, to
pick a member of a union. Inside parentheses, a typed binding is written as a
field pattern whose pattern is itself typed: `Reply(body: b: Text)`. This
keeps `User(age: Int)` from having two readings — it can only mean "field
`age` matches the pattern `Int`", which is rejected because `Int` is a type, not
a variant.

**Present patterns.** `p?` matches an optional that is present and binds its
value to `p`; `_?` matches any present value. Together with `none` they cover
an optional exactly, and they nest: `Person(email: e?)` matches a person whose
email is present and binds it to `e`.

```verse
fn summary(items: List<Int>): Text
    match items
        [] -> "empty"
        [x] -> "just {x}"
        [first, ..rest] -> "{first} and {rest.len()} more"

fn grade(score: Int): Char
    match score
        90..=100 -> 'A'
        80..90 -> 'B'
        70..80 -> 'C'
        _ -> 'F'

fn answer(input: Text): Bool
    match input.lower()
        "y" | "yes" -> true
        _ -> false
```

### 7.4 Guards

An arm may add a condition with `if`:

```verse
match point
    (0, 0) -> "origin"
    (x, y) if x == y -> "on the diagonal"
    (x, _) if x < 0 -> "left half"
    _ -> "somewhere else"
```

### 7.5 Matching optionals and fallibles

```verse
match find_user(id)
    none -> print("not found")
    user? -> print("hello, {user.name}")     # user: User

match parse_age(input)
    age: Int -> print("age is {age}")
    err: Error -> print("invalid: {err.message()}")

match person
    Person(email: e?) -> send_welcome(e)
    Person(name:, email: none) -> print("{name} has no email")
```

For a nested optional such as `Int??` (§8.1), `x??` matches a present inner
value, `none?` matches "present but empty", and `none` matches the outer
absence.

### 7.6 `is` — a pattern as a condition

`value is Pattern` is a `Bool` expression. When used as an `if` condition, the
bindings in the pattern are available inside the block. This is the one way to
branch on a pattern without a full `match` (there is no separate
pattern-binding `if`):

```verse
if shape is Circle(r)
    print("radius {r}")

if settings["theme"] is theme?
    apply(theme)

circles = shapes.filter(s -> s is Circle(_))
```

Bindings from `is` are visible only in the `if` block, never in `else` or
after the `if`. `is` used outside an `if` condition binds nothing.

### 7.7 Destructuring in bindings and `for`

Patterns that always match (irrefutable patterns) may be used on the left of a
binding and in `for`. A pattern always binds new names; the one exception,
a parallel assignment to a tuple of `mut` names such as `(a, b) = (b, a)`,
is described in §3.9:

```verse
(x, y) = position
User(name:, age:) = current_user
[head, ..tail] = items               # error: refutable; the list might be empty
for (key, value) in settings
    print("{key} = {value}")
```

A refutable pattern in a binding is a compile-time error; use `match` or
`if … is` instead — or, when the failure case should leave the function or
loop, the `pattern = … otherwise return/break/continue` form (§3.8.2):

```verse
[head, ..tail] = items otherwise return none
```

**Ownership of bindings.** Names bound by a pattern borrow from the matched
value. If the matched value is owned and this is its last use, the bindings
take ownership of the parts instead, so they can be moved on without copies.

### 7.8 `match` without a subject

When the choice depends on conditions rather than on the shape of one value,
leave out the subject. Each arm is then a `Bool` condition, tried top to
bottom:

```verse
fn sign_label(n)
    match
        n < 0 -> "negative"
        n == 0 -> "zero"
        _ -> "positive"

fee = match
    order.total > 100.0 -> 0.0
    order.express -> 15.0
    _ -> 5.0
```

A subject-less `match` is an expression, uses the ordinary arm syntax
(including blocks after `->`), and **must end with a `_ ->` arm**; without it
the `match` is not exhaustive (§7.2), because no set of conditions can be
proved to cover every case. It replaces long `if … else if … else`
chains that choose a value.

---

