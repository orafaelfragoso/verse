## 6. Control flow

### 6.1 `if` / `else`

```verse
if balance < 0
    print("Overdrawn")
else if balance == 0
    print("Empty")
else
    print("In credit")
```

A block always starts on the line after its header — there is no colon at the
end of `if balance < 0`. Conditions must have type `Bool`; there is no
truthiness.

#### One-line form: `if … then … else …`

When choosing between two values, use the one-line expression form. Both
branches are required, and each is a single expression:

```verse
label = if count == 1 then "item" else "items"
sign = if n < 0 then -1 else if n == 0 then 0 else 1
print("{count} {if count == 1 then "item" else "items"}")
```

The block form is also an expression when it has an `else` and each branch ends
in a value:

```verse
greeting = if hour < 12
    part = "morning"
    "Good {part}"
else
    "Hello"
```

`then` appears only in the one-line form; `else` appears only with `if`.

### 6.2 `if … is` — testing a pattern

To run a block only when a value has a certain shape, test it with `is`
(§7.6). The names bound by the pattern are available inside the block. The
pattern `user?` matches a *present* optional and binds the value inside it:

```verse
if find_user(id) is user?
    print("Found {user.name}")
else
    print("No such user")

if shape is Circle(r)
    print("A circle of radius {r}")
```

There is no separate pattern-binding form of `if`; `is` is the one way to
branch on a pattern.

### 6.3 `for` loops

`for` iterates over anything that implements the `Iterable` trait: lists, sets,
maps, ranges, text (characters), channel receivers, and more.

```verse
for name in ["Ana", "Bruno", "Carla"]
    print("Hi, {name}")

for i in 0..3
    print(i)                       # 0, 1, 2

for (word, count) in word_counts   # maps yield (key: K, value: V) pairs
    print("{word}: {count}")

for (index, item) in items.enumerate()
    print("{index + 1}. {item}")
```

The loop variable is a fresh immutable binding on every iteration. Patterns may
be used to destructure ([§7](07-pattern-matching.md#7-pattern-matching)).

**Ownership in loops.** Each `x` of `for x in items` is a value of its own,
copied from the element (§4.9), so storing it elsewhere is always allowed.
When the loop is the collection's last use, the loop takes the collection and
each element moves into `x`, so nothing is counted. A collection of a type
that is not `Copy` can be iterated only at its last use:

```verse
mut adults: List<Person> = []
for person in people
    if person.age >= 18
        adults.push(person)          # `adults` owns its own copy
```

### 6.4 Ranges

| Range | Contains |
|-------|----------|
| `1..10` | 1, 2, …, 9 (end excluded) |
| `1..=10` | 1, 2, …, 10 (end included) |
| `(1..10).step(2)` | 1, 3, 5, 7, 9 |
| `(1..=10).reverse()` | 10, 9, …, 1 |

Ranges are values of type `Range<Int>` (also `Range<Char>`), support `in`
(`if age in 13..=19`), and can slice lists and text: `items[1..3]`,
`name[0..=2]`.

Inside slice brackets only, either end may be left open:

| Slice | Meaning |
|-------|---------|
| `items[2..]` | from index 2 to the end |
| `items[..3]` | the first three elements (indices 0, 1, 2) |
| `items[..=3]` | indices 0 through 3 |

Open-ended ranges are not values: `r = 2..` and `for i in 1..` are
compile-time errors. Slicing past the end of a list panics, like indexing.

### 6.5 `while`, `break`, and `continue`

```verse
mut attempts = 0
while attempts < 3
    attempts += 1
    if try_connect()
        break
    print("Retrying…")

for line in lines
    continue if line.starts_with("#")     # postfix guard, §6.7
    process(line)
```

`break` leaves the innermost loop; `continue` skips to its next iteration.
A loop may be given a label — written **before** the loop as `label:`, in the
"`:` means *is named*" sense — when you need to leave or continue an outer
loop:

```verse
outer: for row in grid
    for cell in row
        if cell == target
            break outer
```

`continue outer` works the same way. Labels may be placed on `for` and `while`
loops only.

A `while true` loop with `break` is the idiom for an unbounded loop.

### 6.6 `pass`

`pass` is an explicit "do nothing" for blocks that must exist syntactically:

```verse
match event
    Click(x, y) -> handle_click(x, y)
    _ -> pass
```

### 6.7 Postfix guards

The three exits — `return`, `break` and `continue` — may be followed by
`if condition` on the **same line**. The exit happens only when the condition
is true:

```verse
fn safe_ratio(a, b)
    return 0.0 if b == 0.0
    a / b

for line in lines
    continue if line.is_empty()
    break if line == "END"
    process(line)
```

The rules are deliberately narrow: only `return`, `break` and `continue` take
a postfix `if`; it fits on one line; there is no `else`; and there is no
`unless`. Everything else uses an ordinary `if` block. The guard reads as a
sentence ("continue if the line is empty") and keeps the main path of a loop
or function unindented. For unwrapping an optional while exiting, use
`x = … otherwise return` (§3.8.2). `return`, `break` or `continue`
followed by `if` is always a guard; to return a conditional expression,
parenthesize it: `return (if c then a else b)` (§3.8).

---

