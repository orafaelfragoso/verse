## 14. Testing and docs

### 14.1 Test blocks

Tests are written next to the code they test, using `test` blocks:

```verse
fn slugify(title: Text): Text
    title.lower().words().join("-")

test "slugify joins lowercase words with dashes"
    assert slugify("Hello Big World") == "hello-big-world"

test "slugify of empty text is empty"
    assert slugify("") == ""
```

- A `test` block has a descriptive text name and an indented body.
- `test` blocks are ignored when a file runs and by `verse build`, and compiled only
  by `verse test`.
- **`verse test` never runs the entry file's top-level statements.** Tests see
  the module's declarations (functions, types, traits, impls) and its
  module-level constants only. So testing a script such as §20.1 does not read
  files, and testing §20.2 does not touch the network or call `exit`. Code
  that a test needs must therefore live in a function, not in top-level
  statements.
- A test passes if its body finishes without a failed `assert`, a panic, or a
  failed `?`. Inside a test, `?` returns from the test (which then fails with
  the error's message): test bodies behave like functions returning
  `Nothing | Error`.
- Tests can access private items of their module.
- Tests run in parallel, each in its own task; they must not depend on each
  other's order.

### 14.2 `assert`

`assert condition` checks a `Bool`. On failure it reports the source text and,
for comparisons, the values of both sides:

```
FAIL  slugify joins lowercase words with dashes   (text.vs:4)
    assert slugify("Hello Big World") == "hello-big-world"
    left:  "hello-big--world"
    right: "hello-big-world"
```

An optional message follows a comma: `assert total > 0, "cart cannot be empty"`.
Outside tests, `assert` is also allowed in ordinary code, where a failure
panics. Assertions are never removed from release builds.

Patterns combine with `is` for expressive checks:

```verse
test "parsing rejects letters"
    assert parse_age("abc") is _: Error
    assert parse_age("42")? == 42
```

### 14.3 Doc comments

`##` comments are Markdown. Where a `##` block belongs is decided by one
visible rule:

- A `##` block **directly above** a declaration, with no blank line in between,
  documents that declaration.
- A `##` block at the top of a file **followed by a blank line** documents the
  module.

`verse fmt` enforces the blank line after a module doc, so the two never look
alike:

```verse
## Geometry helpers for 2-D drawings.     ← module doc (blank line follows)

import std.math

## The distance between two points.       ← documents `distance`
export fn distance(a: Point, b: Point): Float
    ...
```

Code blocks inside doc comments are **doc tests**: `verse test` runs them too.

````verse
## Returns the area of a shape.
##
## ```verse
## assert area(Square(side: 2.0)) == 4.0
## ```
export fn area(shape: Shape): Float
    ...
````

`verse doc` turns doc comments into a browsable HTML site, with every type,
function, trait and implementation cross-linked.

---

