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
text. For the raw bytes, `text.bytes()` gives the UTF-8
bytes with constant-time indexing: `"héllo".bytes().len()` is `6`, and
`text.bytes()[i]` is the `i`-th byte as an `Int` from 0 to 255. The result is
a `Bytes` value that shares the text's storage until one of them changes
(§4.9).

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

5. **Raise sets are inferred with the return type.** A function whose return
   type is inferred and whose body uses `?` on fallible values (or returns an
   error value) gets the raise set of the errors that can escape, exact as in
   §4.6. A `fn load(path)` whose body is `text = read_text(path)?` followed by
   `text.trim()` returns `Text raises Error`. A function whose return type is
   written declares its raise set too; with no `raises` clause it cannot fail
   (§8.2). A `?` on an optional gives an optional return type.
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

   first_name(ana)            # fine: `Text` is `Copy`
   log_of(service)            # error at the call site: `log_of` needs `File`
                              #   to be `Copy`, because it returns `service.log`
                              #   help: take the parameter with `take`
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

Every value has one owner (§4.9), and assigning it gives the new owner its
own copy. `copy()` with named arguments makes a copy with some fields
changed:

```verse
twin = ana                        # an independent User
older = ana.copy(age: 31)         # ana is unchanged
```

`.copy()` is provided by the built-in trait `Copyable` (§10.4). On a `Copy`
value, `.copy()` with no arguments is the same as the value itself. On a
handle (`Shared<T>`, `Guarded<T>`, `Sender<T>`, `Weak`), which is `Copyable`
but not `Copy`, `.copy()` is the one way to get a second handle to the same
shared value (§4.10). `Task<T>`, `Receiver<T>`, lock guards, platform handles
such as files (§13.4), types that implement `Unique` (§10.4), and anything
that contains them have no `.copy()`.

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
type Meters = Float            # error: `Float` is already a type, so it cannot be a variant
type Value = Int | Text        # error: help: to give it another name, write `alias Value = Int | Text`
type Reply = Text(body: Text) | Empty   # error: `Text` is already a type, so it cannot be a variant
```

```
error: `Float` is already a type, so it cannot be a variant
  --> units.vs:2:15
   |
 2 | type Meters = Float
   |               ^^^^^ a variant of `Meters` cannot reuse the name of a type in scope
   |
help: to give it another name, write `alias Meters = Float`
explain: V0314 (run `verse explain V0314`)
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
- `T raises E` is **fallible**: a `T`, or an error of one of the types in the
  **raise set** `E`, written `raises FetchError | Timeout`. Every type in a
  raise set implements the `Error` trait, and an error type is written only
  there, never as a member of a union. `raises Error` is the **open set**:
  any error type. A set that names `Error` names nothing else.

`?` binds tighter than `|` and `|` tighter than `raises`:
`Text? | Int raises Error` is "an optional text or an int, or any error".
Optionals nest: `Int??` is a different type from `Int?` (§8.1).

**Union normalization.** Unions and raise sets are flattened and
de-duplicated (`(A | B) | A` is `A | B`). A raise set is **exact**: it keeps
every member through `?`, tasks and function types (§8.3, §11.1, §4.7), so a
caller sees what can fail as precisely as the function's body does. Only an
open set hides the concrete type; a typed pattern in `match`
(`err: FetchError -> …`) inspects it there.

Both are covered in depth in [§8](08-errors.md#8-errors-and-missing-values).

### 4.7 Function types

The type of a function is written `fn(ParamTypes): Ret`:

```verse
double: fn(Int): Int = n -> n * 2
fn apply_twice(f: fn(Int): Int, x: Int): Int
    f(f(x))
```

A fallible function's type carries its raise set:
`handler: fn(Request): Response raises HttpError`. A function value with a
smaller set fits a type with a larger one, and one that cannot fail fits any
fallible function type of the same signature.

A parameter in a function type may carry the mode word of its declaration,
`mut` or `take` (§5.3), and a name. The mode word is part of the type; the name
only documents it. `fn update(mut model: Model, msg: Msg): Nothing` has the
type `fn(mut model: Model, msg: Msg): Nothing`, which is the same type as
`fn(mut Model, Msg): Nothing`.

### 4.8 Type aliases

An alias gives an existing type a second, shorter name. It does not create a
new type; the two names are interchangeable.

```verse
alias Headers = Map<Text, Text>
alias Handler = fn(Request): Response raises Error
alias Loaded = Config raises Error
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
element of a collection, or a task. Right after the owner's last use, the
value is dropped, immediately and deterministically (§4.10). A `Task` owned by a binding, or discarded by a bare `spawn`, is the one
exception: it is not dropped at its last use but waited for when its scope
ends (§11.3).

**Assignment copies.** Binding a value to a name, assigning it, passing it,
storing it in a field or a collection, returning it, binding it in a `for`
loop (§6.3) or a pattern (§7.7), and taking it out of a collection (§9.3,
§9.5) all give the new owner **its own value**. Changing one never changes
another, and what a line means never depends on the lines after it:

```verse
people = load_people()
mut adults: List<Person> = []
for person in people
    if person.age >= 18
        adults.push(person)      # `adults` owns its own copy
print(people.len())              # adding or removing this line changes nothing above

names = ["Ana", "Bruno"]
mut roster = names               # a copy; `names` is unchanged whatever follows
roster.push("Carla")
print(names)                     # ["Ana", "Bruno"]
```

**Copy types.** A type whose values are copied this way is `Copy` (§10.4):
scalars (`Int`, `Float`, `Bool`, `Char`, `Nothing`), `Text`, `Bytes`,
`List<T>`, `Map<K, V>` and `Set<T>` of `Copy` types, optionals and unions of
`Copy` types, and records, variants and tuples made only of `Copy` parts. A
copy costs no more than a count: `Text`, `Bytes` and the collections keep
their contents in **shared storage** that copies share until one of them
changes. The first change to shared storage copies it (**copy on write**),
so a copy that is never changed is never duplicated (§4.10). When the source
is not used again, the compiler moves it instead, which costs nothing.

**Moves.** A value of a type that is not `Copy` moves when it gets a new
owner, and the old owner cannot be used again. These types are the handles
`Shared<T>`, `Guarded<T>`, `Weak<…>` and `Sender<T>` (§11.4), which move so
that sharing is always written as `.copy()`; `Task<T>`, `Receiver<T>`, lock
guards and platform handles such as `File` (§13.4); types that implement
`Unique` (§10.4); and anything that contains one of them.

**Moving a field out.** Moving a non-`Copy` field out of an owned value at the
value's last use is allowed: the field gets its new owner and the rest of the
value is dropped. If the value is used afterwards, moving the field is the
usual move error:

```verse
fn log_of(take service: Service)
    service.log               # fine: `service` is owned and not used again
```

Moves are tracked **per control-flow path**. A value moved in one branch of an
`if` or `match` counts as moved after it on that path, and a value moved inside
a loop counts as moved on the next iteration.

Using a moved value is a compile-time error that says where the move happened:

```verse
mut workers: List<Task<Report>> = []
job = spawn build_report()
workers.push(job)
print(job.wait())
```

```
error: `job` was moved and cannot be used
  --> reports.vs:4:7
   |
 3 | workers.push(job)
   |              --- moved here
 4 | print(job.wait())
   |       ^^^ moved to `workers` at line 3
   |
help: use `job` before the move, or move it last
explain: V0501 (run `verse explain V0501`)
```

When the move happens in only one branch, the label names the condition:
"moved at line 3 only when `urgent`".

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
user.age = 31                     # error: `user` is not `mut`, so `user.age` cannot be changed

mut guest = User(name: "Guest", age: 0)
guest.age = 18                    # fine
guest = User(name: "Other", age: 5)   # fine; the old guest is dropped
```

```
error: `user` is not `mut`, so `user.age` cannot be changed
  --> users.vs:2:1
   |
 2 | user.age = 31
   | ^^^^^^^^ bound without `mut` at line 1
   |
help: write `mut user = …` at the first binding
explain: V0514 (run `verse explain V0514`)
```

Methods that change their receiver declare `mut self` and can only be called on
mutable bindings ([§5.6](05-functions.md#56-methods)).

**Order inside an assignment.** In `place = value` (and `place += value`), the
right side is evaluated completely before the place, so a value can be read on
the right and moved into the place on the left in the same statement. The
receiver and index expressions on the left are evaluated after the right side,
left to right among themselves; everything else is evaluated left to right:

```verse
counts[word] = counts.get(word, default: 0) + 1   # reads `word`, then stores it
```

### 4.10 Memory model

Verse has **no garbage collector** and no manual memory management:

1. **Single ownership.** Every value has one owner and is freed right after
   its owner's last use (or when a `mut` owner is reassigned). There is no
   collector running in the background and no pause.
2. **No references in the surface language.** There are no pointer types, no
   reference types, and no lifetime syntax. A function reads a plain argument
   in place for the duration of the call, and a value it stores, returns or
   sends to another task is a copy (§5.3). This is what lets the compiler
   prove, without annotations, that nothing is ever used after it is freed.
3. **Recursive types are boxed for you.** Variant types such as `Json` or
   `Tree<T>` are stored with the indirection they need; the boxes are owned
   like any other value and freed with their owner.
4. **Release is deterministic.** Dropping a value frees its memory and
   releases the runtime-owned handles in it, such as an open file or a lock,
   on every path, a panic included (§13). A drop runs no code of the program.
   Dropping a copy of shared storage counts the storage down, and the last
   copy frees it. Locals whose last use is the same point are freed in reverse
   order of declaration, because a later local may have been made from an
   earlier one. The parts of one value are freed in the order they were
   written:
   fields and tuple elements in declaration order, list elements by index, and
   `Map` and `Set` entries in insertion order.
5. **Shared storage is invisible.** Copies of a `Text`, `Bytes` or collection
   share storage under an atomic count until one of them changes, and a change
   to shared storage first copies it. So no program can observe the sharing:
   every owner sees its own value, on every task.
6. **Sharing is opt-in.** The only ways for two owners to reach one value are
   the handle types `Shared<T>` (shared read-only), `Guarded<T>` (shared,
   mutable behind a lock), `Weak<Shared<T>>` / `Weak<Guarded<T>>` (non-owning),
   and the channel sender `Sender<T>` (§11.4). A handle is not `Copy`: it
   moves, and a second handle to the same value is written `.copy()`, so
   every place that shares is visible (a `Task<T>` and a `Receiver<T>` are
   never shared: they cannot be copied, and they move; `wait` takes a task).

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

