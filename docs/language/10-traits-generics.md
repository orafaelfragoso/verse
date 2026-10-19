## 10. Traits and generics

### 10.1 Traits

A trait names a set of methods that several types can share. A method without a
body is **required**; a method with a body is a **default** that implementations
may override. Because a required method has no body to infer from, its
parameter and return types are always written in the trait; implementations
may leave them out, since the trait already fixes them.

```verse
trait Speaker
    fn speak(self): Text

    fn speak_twice(self): Text
        "{self.speak()} {self.speak()}"
```

A type implements a trait with `impl Trait for Type`:

```verse
type Dog
    name: Text

type Robot
    model: Int

impl Speaker for Dog
    fn speak(self): Text
        "{self.name} says woof"

impl Speaker for Robot
    fn speak(self): Text
        "Unit {self.model} says beep"
```

A type's own methods go in a plain `impl Type` block; trait methods go in
`impl Trait for Type`. An implementation must appear in the module that defines
either the trait or the type (this keeps implementations easy to find and
prevents conflicts between libraries).

### 10.2 Supertraits

A trait can require another trait:

```verse
trait Describe: Show
    fn describe(self): Text
        "<{self}>"
```

Every type implementing `Describe` must also implement `Show`.

### 10.3 Traits as types

A trait name used as a type means "any value whose type implements this trait".
This allows collections of different types that share behavior:

```verse
speakers: List<Speaker> = [Dog(name: "Rex"), Robot(model: 7)]
for s in speakers
    print(s.speak())
```

Calls through a trait type are dispatched at run time.

### 10.4 Built-in traits

| Trait | Provides | Automatically implemented? |
|-------|----------|----------------------------|
| `Show` | `show(self): Text`, used by interpolation and `print` | yes, structurally, for all types (override for custom text) |
| `Eq` | `==`, `!=` | yes, when all parts are `Eq` |
| `Order` | `<`, `<=`, `>`, `>=`, `sort()` | yes, when all parts are `Order` |
| `Hash` | use as map key / set element | yes, when all parts are `Hash` |
| `Iterable<T>` | `for … in`, and all the methods of §9.5 | lists, maps, sets, ranges, text, channel receivers |
| `Default` | `Type.default()` | when all fields have defaults |
| `Copy` | implicit copying on assignment, passing, storing and returning (§4.9); every `Copy` type is also `Copyable` | yes, for scalars, `Text`, `Bytes`, and for `List`, `Map`, `Set`, optionals, unions, records, variants and tuples made only of `Copy` parts; absent for handles, `Task<T>`, `Receiver<T>`, lock guards, platform handles, `Unique` types, function values that borrow their captures (§5.4), and anything that contains them; cannot be implemented by hand |
| `Copyable` | the `.copy()` method, with no arguments (the same value, or for a handle a second handle) or with field arguments on records (a copy with changes, §4.3) | yes, structurally, for every type whose parts are all `Copyable`; **absent** for `Task<T>`, for lock guards and platform handles such as `File` (§13.4), for types that implement `Unique`, and for anything that contains them by value (a record field, variant, tuple or collection element); cannot be implemented by hand in 0.1. The handles `Shared<T>`, `Guarded<T>`, `Sender<T>`, `Weak<Shared<T>>` and `Weak<Guarded<T>>` are always `Copyable`, whatever `T` is (including resources, as in `Guarded<File>`), because their `.copy()` copies the handle (a new reference to the same shared value) and never duplicates the value inside (§11.4); `Receiver<T>` is not `Copyable` in 0.1 (single consumer) |
| `Unique` | no methods: a type that implements it, and anything that contains it by value, is neither `Copy` nor `Copyable` | no; written as `impl Unique for T` with an empty body, for a type that owns something the runtime does not, such as a temporary directory or a foreign handle (§13.4, §12.6) |
| `Index<K, V>` | `x[k]` reads (§9.3) | lists, maps, text and slices; cannot be implemented by hand in 0.1 |
| `Error` | `message(self)`, `cause(self)` | no |
| `Add`, `Sub`, `Mul`, `Div`, `Rem` | `+`, `-`, `*`, `/`, `%` | `Int`, `Float`, `math` types; `Add` also for `Text` and `List` (concatenation) |
| `Neg` | unary `-` | `Int`, `Float`, `math` types |

**`Copyable` is deliberately closed in 0.1.** It cannot be implemented by
hand, and platform handles and lock guards are deliberately **not**
`Copyable`: duplicating a file handle or a lock guard would leave two owners
of one external resource, and each drop would release it (§13.4). A
type that does need duplication offers an explicit method with its own name
and its own meaning, such as `duplicate()` (for example, reopening the same
file), so the call site says what really happens.

The arithmetic traits are how numeric types in `math` (such as `Decimal`
and `BigInt`) and user types like vectors or money get operators. They are the
only operator overloading Verse allows (§18.5); there is no way to define new
operators or to overload `and`, `or`, `not`, `otherwise`, `==` (which comes
from `Eq`) or indexing.

```verse
type Vector
    x: Float
    y: Float

impl Add for Vector
    fn add(self, other: Vector): Vector
        Vector(x: self.x + other.x, y: self.y + other.y)

v = Vector(x: 1.0, y: 2.0) + Vector(x: 0.5, y: 0.5)
```

### 10.5 Generic functions and types

Generic parameters go in angle brackets after the name.

```verse
fn first<T: Copy>(items: List<T>): T?
    items.get(0)                  # a copy of the first element, or `none`

type Box<T>
    value: T

type Pair<A, B>
    left: A
    right: B

type Tree<T> = Leaf | Node(left: Tree<T>, value: T, right: Tree<T>)
```

Generic arguments are usually inferred: `first([1, 2, 3])` is `Int?`, and
`Box(value: "hi")` is a `Box<Text>`. Write them explicitly when needed:
`Box<Int>(value: 0)`, `names: List<Text> = []`.

Free generic functions do not take explicit type arguments in 0.1: their type
arguments are inferred, or given by annotating the binding
(`n: Int = parse(text)?`). `parse<Int>(text)` is a compile-time error whose
help suggests that annotation. Explicit type arguments are allowed only on an
associated function called through a type name, such as
`Channel.new<Int>(capacity: 10)` (§11.4).

Methods on a generic type declare the parameters on the `impl`:

```verse
impl<T> Box<T>
    fn map<U>(self, f: fn(T): U): Box<U>
        Box(value: f(self.value))
```

### 10.6 Constraints

A generic parameter can require traits with `:`; several traits are joined
with `+`.

```verse
fn largest<T: Order + Copy>(items: List<T>): T?
    items.max()                   # a copy of the largest element (§9.5)

fn announce<T: Speaker + Show>(who: T)
    print("{who}: {who.speak()}")

impl<T: Show> Show for Box<T>
    fn show(self): Text
        "Box({self.value})"
```

Generics are checked once, at definition: inside `largest`, only operations
promised by `Order` and `Copy` may be used on `T`. That includes the
**ownership check**: a generic (or unannotated) parameter type is treated as
an opaque, non-`Copy` type unless it has a `Copy` bound, so a `T` is copied
only with that bound, and `.copy()` needs `Copyable` (§4.2, rule 9).
Errors about a function's body appear at the definition. The only call-site
errors are unmet bounds, written or inferred, and they name the bound. The
compiler specializes generic code for speed;
this is invisible to the programmer.

`Copy` and `Copyable` are different promises: `T: Copy` means a `T` is
copied implicitly on assignment (every value type: numbers, `Text`,
collections, and records of them); `T: Copyable` means a `T` has `.copy()`,
which every `Copy` type has and so do the handles `Shared<T>`, `Guarded<T>`,
`Sender<T>` and `Weak`, whose `.copy()` is a second handle (not a `Task` or
an open `File`).

---

