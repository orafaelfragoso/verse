## 4. Types

### 4.1 Built-in scalar types

| Type | Description | Examples |
|------|-------------|----------|
| `Int` | 64-bit signed integer, overflow-checked | `0`, `-12`, `1_000` |
| `Float` | 64-bit IEEE 754 floating point | `3.14`, `1e-9` |
| `Bool` | truth value | `true`, `false` |
| `Char` | one Unicode scalar value | `'a'`, `'\u{263A}'` |
| `Text` | immutable UTF-8 string | `"hello"` |
| `Bytes` | immutable sequence of bytes | `b"\x00\xFF"` |
| `Nothing` | the type of "no useful value" | (result of `print`) |

Sized numeric types (`Int8` … `Int64`, `UInt8` … `UInt64`, `Float32`) and
arbitrary-precision `BigInt` and `Decimal` are provided by `math` for code
that needs them.

`Text` is indexed by *characters*, not bytes: `"héllo".len()` is `5`. Because
positions count characters, indexing or slicing a `Text` by position
(`text[i]`, `text[2..5]`) may take time proportional to the length of the
text. For the raw bytes, `text.bytes()` gives a read-only view of the UTF-8
bytes with constant-time indexing: `"héllo".bytes().len()` is `6`, and
`text.bytes()[i]` is the `i`-th byte as an `Int` from 0 to 255. It follows the
rules for views (§9.3): it can be read, passed and bound to a name, but not
stored or returned; `.copy()` gives an owned `Bytes`.

### 4.2 Type annotations and inference

Verse is statically typed, but types are **visually optional**: annotations may
be left out almost everywhere — on local bindings, on function parameters, and
on return types. The compiler infers them, checks everything at compile time,
and never guesses: when it cannot decide, it stops and asks for one annotation
at that exact spot.

```verse
count = 0                           # Int
ratio = 0.5                         # Float
ratio2: Float = 1                   # error: 1 is an Int; write 1.0
mut tags = []                       # element type decided by later use…
tags.push("new")                    # …so tags is List<Text>

fn greet(name)                      # inferred: any `name` that is Show; returns Text
    "Hello, {name}!"

fn greet_typed(name: Text): Text    # the same function, spelled out
    "Hello, {name}!"

fn double(n)                        # numeric; Int unless a caller says otherwise
    n * 2
```

Both styles are equally checked. Annotate where it helps a reader — at
important boundaries, in teaching code, or to narrow a function on purpose —
and leave the rest to the compiler.

#### Inference rules

1. **Unannotated parameters are implicit generics.** Each one gets exactly the
   constraints its body needs, and nothing more:
   - trait constraints from operations — interpolation or `print` gives
     `Show`, `+` gives `Add`, `==` gives `Eq`, `<` gives `Order`, `for` gives
     `Iterable`, indexing `x[i]` gives `Index`, `.copy()` gives `Copyable`,
     calling a trait method gives that trait;
   - *callable* from calls — `f(x)` on a parameter `f` makes it a function
     type with matching parameters;
   - *structural* constraints from field access and inherent method calls —
     `person.name` gives "has a field `name`", `text.trim()` gives "has a
     method `trim()`";
   - a variant pattern (`Circle(r) -> …`) or passing the parameter to an
     annotated parameter fixes or narrows the type.

   Every call site is checked against those constraints at compile time, and
   the compiler generates specialized code for each concrete type used — but
   the **ownership check runs once, at the definition** (rule 9), so whether a
   body compiles never depends on which types its callers use. So
   `greet(42)`, `greet("Ana")` and `greet(ana)` all work if the argument is
   `Show`, while `greet(some_function)` is a compile error at the call site.
   Structural constraints are inferred only; they cannot be written by hand in
   0.1. To narrow a function on purpose, annotate a concrete type
   (`person: Person`).
2. **Numeric literals default to `Int` and `Float`.** `fn double(n)` with
   `n * 2` accepts any numeric type; where nothing else decides (for example
   `x = double(3)`), the literal makes it `Int`, and `double(2.5)` is a
   `Float`.
3. **Recursion needs an annotation.** A function that calls itself, directly or
   through other functions, must annotate its parameters and return type;
   otherwise the compiler reports an ambiguity at the recursive call.
4. **When inference cannot decide, it says so — exactly there.** The error
   names the spot, says which annotation to add, and lists the constraints it
   did infer:

   ```
   error: cannot infer the element type of this list
     --> stats.vs:2:5
      |
    1 | fn empty_scores()
      |                  - add a return type here, for example `: List<Int>`
    2 |     []
      |     ^^ a List of something, but nothing says of what
   ```

5. **Fallible return types are inferred from `?`.** A function whose body uses
   `?` on fallible values (or returns an `error(…)`) gets a fallible return
   type made of its success type and the errors that can escape, normalized as
   in §4.6. A `fn load(path)` whose body is `text = read_text(path)?`
   followed by `text.trim()` returns `Text | Error`. A `?` on an optional gives an optional return type.
6. **Inferred types are visible on demand.** Editors (through `verse lsp`) show
   inferred parameter and return types as inline hints. `verse fmt` never
   inserts or removes annotations.
7. **Record fields keep their annotations** unless a default value implies the
   type (`active = true` is a `Bool` field; `tags: List<Text> = []` still needs
   its annotation because `[]` says nothing about the elements).
8. **`mut` and `take` are always written.** How a function treats its argument
   (borrow, exclusive `mut` borrow, or `take`) is never inferred (§5.3).
9. **Ownership is checked once, at the definition.** Inside a generic or
   unannotated function, every parameter type that is unannotated or generic
   is treated as an opaque, **non-`Copy`** type, unless a `Copy` bound says
   otherwise (`<T: Copy>`). One bound is inferred instead of rejected, so that
   simple getters need no annotation: an unannotated function that returns a
   field, element or lookup read through a borrowed parameter gets an
   **inferred `Copy` bound** on the returned type, recorded in `verse.api`
   like any other inferred constraint:

   ```verse
   fn age_of(person)
       person.age             # fine: inferred bound "`person.age` is Copy"

   fn first_name(person)
       person.name            # fine here too: inferred bound "`person.name` is Copy"

   first_name(ana)            # error at the call site: `first_name` returns
                              #   `person.name` by value, so the field must be
                              #   Copy, and `Person.name` is `Text`
                              #   help: return `person.name.copy()`, or take
                              #   the parameter with `take`
   ```

   Errors about a function's body appear at the definition. The only
   call-site errors are unmet bounds, written or inferred, and they name the
   bound. Because the check never looks at callers, `verse.api` is enough to
   check other modules (§12.3).

#### Inference and module boundaries

Inference never crosses a module boundary. Exported functions may have
inferred signatures **only** because every package commits a generated
interface file, `verse.api`, that records each exported signature as the
compiler inferred it. Other modules are type-checked against `verse.api`, never
against another module's body, and the build fails if an inferred exported
signature drifts from the recorded one until someone runs `verse api update`
on purpose. A body edit therefore cannot silently change the public API. See
§12.3 and §16.4.

### 4.3 Record types

A record type groups named fields. Fields are listed on indented lines.

```verse
type User
    name: Text
    age: Int
    email: Text? = none
    roles: List<Text> = []
    active = true                 # type implied by the default: Bool
```

Fields keep their type annotations unless a default value implies the type
(§4.2). Fields may have default values. Construct a record by calling the type name with
named arguments; positional arguments are allowed in field order. When a
variable has the same name as the field, the value may be left out —
`User(name:, age:)` means `User(name: name, age: age)`. This *field punning*
works the same way in construction and in patterns (§7.3).

```verse
ana = User(name: "Ana", age: 30)
bruno = User("Bruno", 17)
name = "Carla"
age = 41
carla = User(name:, age:)
print(ana.name)
```

Fields are visible inside the module that defines the type. To make a field
visible to other modules, mark it `export` (§12.3).

#### Copies

Every value has one owner (§4.9). `copy()` with no arguments makes an
independent duplicate; the same method with named arguments makes a duplicate
with some fields changed:

```verse
twin = ana.copy()                 # an independent User
older = ana.copy(age: 31)         # ana is unchanged
```

`.copy()` is the one way to duplicate a value, with or without changes. It is
provided by the built-in trait `Copyable` (§10.4), which almost every type has
automatically; the exceptions are `Task<T>`, types that implement `Cleanup`
(files, locks, temporary directories), and anything that contains them.

### 4.4 Variant types

A variant type (also called a sum type or tagged union) is one of several named
alternatives, each with its own fields.

```verse
type Shape = Circle(radius: Float) | Square(side: Float)

type Direction = North | East | South | West

type Json = Null
    | Flag(value: Bool)
    | Number(value: Float)
    | Str(value: Text)
    | Array(items: List<Json>)
    | Object(fields: Map<Text, Json>)
```

Long variant lists may continue across lines when each line starts with `|`
(§3.4). A `type X =` header never opens a field block.

**A variant name may not collide with a type that is already in scope**
(`Int`, `Float`, `Text`, `Error`, an imported type, …). This one rule keeps
`type` and `alias` apart:

```verse
type Direction = North | East | South | West   # fine: four new unit variants
type Meters = Float            # error: `Float` is an existing type
                               #   help: to give it another name, write `alias Meters = Float`
type Value = Int | Text        # error: help: write `alias Value = Int | Text`
type Reply = Text(body: Text) | Empty   # error: variant name `Text` is already a type
```

Variants are constructed like records — `Circle(radius: 2.0)` or
`Circle(2.0)` — and may be qualified with the type name when a name is
ambiguous: `Shape.Circle(2.0)`. Variant types may be recursive; the compiler
handles the indirection.

### 4.5 Tuples

A tuple groups a fixed number of values of possibly different types.

```verse
pair = (3, "three")                 # (Int, Text)
print(pair.0)                       # 3
(number, word) = pair               # destructuring
```

Tuple elements may optionally be **labeled**, in which case they can be read
by name as well as by position:

```verse
fn divide(a: Int, b: Int): (quotient: Int, remainder: Int)
    (quotient: a / b, remainder: a % b)

result = divide(17, 5)
print("{result.quotient} remainder {result.remainder}")   # 3 remainder 2
(q, r) = result
```

### 4.6 Optional, union and fallible types

- `T?` is an **optional**: a `T` or `none`.
- `A | B` is a **union**: a value that is either an `A` or a `B`. The member
  types must be distinct.
- A union with a member that implements the `Error` trait is **fallible**:
  `Int | Error` is "an `Int`, or an error". Members implementing `Error` form
  the *error side*; the others form the *success side*.

`?` binds tighter than `|` in types: `Text? | Error` is
"an optional text, or an error". Optionals nest: `Int??` is a different type
from `Int?` (§8.1).

**Union normalization.** Unions are flattened and de-duplicated
(`(A | B) | A` is `A | B`), and a specific error type in a union that also
contains `Error` is **absorbed** by `Error`: `Text | FetchError | Error`
normalizes to `Text | Error`. Once a value has been widened to `Error`, `?`
cannot narrow it back to `FetchError`; use a typed pattern in `match`
(`err: FetchError -> …`) to inspect the concrete error.

Both are covered in depth in [§8](08-errors.md#8-errors-and-missing-values).

### 4.7 Function types

The type of a function is written `fn(ParamTypes): Ret`:

```verse
double: fn(Int): Int = n -> n * 2
fn apply_twice(f: fn(Int): Int, x: Int): Int
    f(f(x))
```

### 4.8 Type aliases

An alias gives an existing type a second, shorter name. It does not create a
new type; the two names are interchangeable.

```verse
alias Headers = Map<Text, Text>
alias Handler = fn(Request): Response | Error
alias Loaded = Config | Error
```

`alias` is required to name a union of *existing* types. In a `type`
declaration, `|` always introduces *new* variants (§4.4), so `type X = A | B`
and `alias X = A | B` can never be confused.

To create a genuinely *distinct* type (so that a `UserId` cannot be mixed up
with an ordinary `Int`), declare a one-field record:

```verse
type UserId
    value: Int
```

### 4.9 Ownership, moves and copies

Every value has **exactly one owner**: a binding, a field of a record, an
element of a collection, or a task. When the owner's scope ends, the value is
freed — immediately and deterministically.

**Copy types.** Scalars (`Int`, `Float`, `Bool`, `Char`, `Nothing`) and
records, variants and tuples made *only* of copy types (a `Point(x: Float, y:
Float)`, a `Shape`, an `(Int, Int)`) are copied implicitly whenever they are
assigned or passed. There is no size limit: whether a type is `Copy` depends
only on the kinds of its fields, never on its size. A copy is never inserted
silently except for `Copy` types and literals (§3.8.2), and moves happen at a
value's last use; adding a non-`Copy` field (say a `Text`) to an all-scalar
record makes it non-`Copy`, and the compiler then reports every place that
relied on the implicit copy. Everything else — `Text`, `Bytes`, collections, and records that
contain them — is **moved** when it gets a new owner. A non-`Copy` value is
never duplicated implicitly; anywhere an implicit duplicate would be needed,
the compiler asks for `.copy()`.

**Moves.** A value moves when it is:

- bound with `mut b = a`, or with `b = a` when `a` is not used
  afterwards;
- stored in a field or a collection (record construction, `push`, `map[k] = v`);
- returned from a function;
- passed to a `take` parameter (§5.3) or into a task with `spawn` (§11.1);
- captured by an escaping lambda (§5.4).

When `b = a` appears and `a` *is* used afterwards, nothing moves: `b` is a
read-only local view of `a` (a borrow, §5.3), and `a` cannot be changed while
`b` is in use. A binding to a collection chain is a local view in the same
way (§9.5).

**Moving a field out.** Moving a field out of an owned value at the value's
last use is allowed: the field gets its new owner and the rest of the value is
freed. If the value is used afterwards (or is only borrowed), moving the field
is the usual move error, with a `.copy()` suggestion:

```verse
fn body_of(url)
    response = get(url)?
    response.body             # fine: `response` is owned and not used again
```

Moves are tracked **per control-flow path**. A value moved in one branch of an
`if` or `match` counts as moved after it on that path, and a value moved inside
a loop counts as moved on the next iteration.

```verse
mut a = [1, 2, 3]
mut b = a.copy()              # an independent list
b.push(4)
print(a)                      # [1, 2, 3]
print(b)                      # [1, 2, 3, 4]

names = ["Ana", "Bruno"]
mut roster = names            # moves: `names` is not used afterwards
roster.push("Carla")

p = Point(x: 1.0, y: 2.0)     # an all-Float record: copied implicitly
q = p
print(p.x + q.x)              # fine: q is a copy
```

Using a moved value is a compile-time error that says where the move happened
and suggests `.copy()`:

```verse
mut inbox: List<Text> = []
note = "remember the milk"
inbox.push(note)
print(note)
```

```
error: `note` is used after it was moved
  --> notes.vs:4:7
   |
 3 | inbox.push(note)
   |            ---- `note` moved into `inbox` here
 4 | print(note)
   |       ^^^^ used here after the move
   |
help: to keep using `note`, store a copy:   inbox.push(note.copy())
```

**Errors name the later use.** Whether `b = a`, a `for` loop variable
(§6.3), a pattern binding (§7.7) or a chain result (§9.5) owns its value or
only views it depends on whether the source is used later. So adding a line
below can turn working code into an error. Every error caused by such a flip
must name the later use that kept the source alive:

```
error: `person` is only a view, so it cannot be stored in `adults`
  --> people.vs:8:16
   |
 7 | for person in people
   |     ------ `person` is a view because `people` is used again on line 12
 8 |     adults.push(person)
   |                 ^^^^^^ stored here
...
12 | print(people.len())
   |       ------ the later use that keeps `people` alive
   |
help: write `person.copy()`, or move the use on line 12 above the loop
```

Inside a loop, the error points at both the move and the later use — which is
the same line, one iteration later:

```verse
greeting = "hi"
mut all: List<Text> = []
for name in names
    all.push(greeting)        # error: `greeting` moved here in one iteration
                              #   and used again here in the next
                              #   help: all.push(greeting.copy())
```

**Mutability** is a property of the **binding**, and it is deep. `mut` means
"can change" everywhere it appears — on a binding, a parameter (§5.3) or
`self` (§5.6):

- A plain binding (`user = …`) can never be reassigned, and the value it owns
  can never be changed — not its fields, not its elements.
- A `mut` binding (`mut guest = …`) can be reassigned with `guest = …`, and
  its fields and elements can be changed in place. Reassigning frees the old
  value first.

Because there is no `let` and no shadowing (§3.9), `guest = …` on an existing
name is always a reassignment, and on an immutable name always an error.

```verse
user = User(name: "Ana", age: 30)
user.age = 31                     # error: `user` is immutable; write
                                  #   `mut user = …` where it is declared

mut guest = User(name: "Guest", age: 0)
guest.age = 18                    # fine
guest = User(name: "Other", age: 5)   # fine; the old guest is freed
```

Methods that change their receiver declare `mut self` and can only be called on
mutable bindings ([§5.6](05-functions.md#56-methods)).

**Order inside an assignment.** In `place = value` (and `place += value`), the
right side is evaluated completely before the place, so a value can be read on
the right and moved into the place on the left in the same statement. The
receiver and index expressions on the left are evaluated after the right side,
left to right among themselves; everything else is evaluated left to right:

```verse
counts[word] = counts.get(word, default: 0) + 1   # borrows `word`, then moves it
```

### 4.10 Memory model

Verse has **no garbage collector** and no manual memory management:

1. **Single ownership.** Every value has one owner and is freed at the end of
   its owner's scope (or when a `mut` owner is reassigned). There is no
   collector running in the background and no pause.
2. **No references in the surface language.** There are no pointer types, no
   reference types, and no lifetime syntax. Functions *borrow* their
   arguments, but borrows are **second-class**: they cannot be stored in a
   field, returned, or sent to another task (§5.3). This is what lets the
   compiler prove, without annotations, that nothing is ever used after it is
   freed.
3. **Recursive types are boxed for you.** Variant types such as `Json` or
   `Tree<T>` are stored with the indirection they need; the boxes are owned
   like any other value and freed with their owner.
4. **Cleanup is deterministic.** A type that implements `Cleanup` has its
   `cleanup(mut self)` run automatically when its owner's scope ends — also
   during a panic (§13). Locals are freed in reverse order of declaration,
   because a later local may borrow an earlier one. The parts of one value
   cannot borrow each other, so they are freed in the order they were written:
   fields and tuple elements in declaration order, list elements by index, and
   `Map` and `Set` entries in insertion order.
5. **Sharing is opt-in.** The only ways for two owners to share one value are
   the handle types `Shared<T>` (shared read-only), `Guarded<T>` (shared,
   mutable behind a lock), `Weak<Shared<T>>` / `Weak<Guarded<T>>` (non-owning),
   and the channel sender `Sender<T>` (§11.4). Copying a handle is cheap and
   shares the value behind it; the copy must be written explicitly as
   `.copy()`, like every other copy (a `Task<T>` and a `Receiver<T>` are never
   shared: they cannot be copied, and they move; `wait` takes a task).

Consequences:

- No use-after-free, no double free, no dangling reference, and no data race,
  in any program that compiles.
- **Cycles are possible only through `Shared`/`Guarded` handles** (for example
  two `Guarded` nodes pointing at each other). Such a cycle is never freed; use
  a `Weak<Shared<T>>` or `Weak<Guarded<T>>` for back-pointers to avoid it.
- Arbitrary graphs (trees with parent links, graphs with cycles) are built
  either with `Shared`/`Weak` handles or — usually simpler and faster — with an
  **index-based arena**: store the nodes in a `List<Node>` and refer to them by
  `Int` index.

```verse
type Node
    name: Text
    children: List<Int> = []      # indices into the arena
    parent: Int? = none

type Tree
    nodes: List<Node> = []

impl Tree
    fn add(mut self, take name, parent: Int?): Int
        self.nodes.push(Node(name:, parent:))
        if parent is p?
            # The argument is evaluated first, then the `mut` borrow of the
            # receiver begins (§5.3), so reading `self.nodes` here is fine.
            self.nodes[p].children.push(self.nodes.len() - 1)
        self.nodes.len() - 1
```

The trade-offs of this model are discussed honestly in §18.10.

---

