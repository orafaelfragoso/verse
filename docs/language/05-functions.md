## 5. Functions

### 5.1 Declaring functions

A function is declared with `fn`, its parameters, an optional return type after
a colon, and an indented body. **The last expression of the body is the return
value.** Parameter and return types may be written or left to inference
(§4.2):

```verse
fn greet(name)
    "Hello, {name}!"

fn hypotenuse(a: Float, b: Float): Float
    squares = a * a + b * b
    squares.sqrt()
```

A function whose body ends in a statement (or that is annotated without a
return type and never returns a value) returns `Nothing`:

```verse
fn log(message)
    print("[log] {message}")
```

### 5.2 Early return

`return` exits a function early. It is needed only when leaving before the last
line:

```verse
fn classify(n)
    if n < 0
        return "negative"
    if n == 0
        return "zero"
    "positive"
```

Two compact forms exist for the most common early exits. A postfix guard puts
the condition after the exit (§6.7), and `otherwise return` unwraps an optional
or leaves (§3.8.2):

```verse
fn describe_user(id)
    return "invalid id" if id <= 0
    user = find_user(id) otherwise return "no such user"
    "{user.name}, {user.age}"
```

### 5.3 Parameters and arguments

#### Borrow, `mut`, and `take`

How a function receives each argument is part of its signature and is always
written explicitly (it is never inferred):

| Parameter | Meaning | Call site |
|-----------|---------|-----------|
| `p` | **read-only borrow** (the default): the function may look, not change or keep | `f(x)` |
| `mut p` | **exclusive mutable borrow**: the function may change the caller's value in place | `f(mut x)` — `x` must be a `mut` binding |
| `take p` | **consume**: the function becomes the owner; the caller's value moves (copy types are copied) | `f(x)` — no marker; the move is visible because `x` must not be used afterwards |

`mut` means "can change" in every position: a `mut` binding can be reassigned
and changed in place (§3.9, §4.9), a `mut` parameter may change the caller's
value, and a `mut self` method may change its receiver (§5.6). Parameter names
count as bindings of the function, so a body cannot bind a parameter's name
again (§3.9). A `mut` parameter is the caller's place under an exclusive
borrow, so assigning to it replaces the caller's value in place:

```verse
fn reset(mut list)
    list = []                           # the caller's list is now empty

mut names = ["Ana", "Bruno"]
reset(mut names)                        # names == []
```

```verse
type Cart
    items: List<Item> = []

fn total(cart)                          # borrows `cart`
    cart.items.map(.price).sum()

fn add_item(mut cart, take item)        # changes `cart`, keeps `item`
    cart.items.push(item)

mut cart = Cart()
apple = Item(name: "apple", price: 0.5)
add_item(mut cart, apple)               # `mut` at the call site; `apple` moves
print(total(cart))
print(apple.name)                       # error: `apple` was moved and cannot be used
                                        #   help: use `apple.copy()` before the move, or move it last
```

The `mut` marker is written at the call site of a *plain function*, so a
reader can see which arguments may change. Method calls (`cart.add(item)`) do
not carry it (§5.6). A value cannot be passed as `mut` and also as another
argument of the same call (`merge(mut a, a)` is an error), so a function with a
`mut` parameter always has exclusive access.

**Arguments first, then the exclusive borrow.** All arguments of a call are
evaluated first, left to right; only then does the exclusive borrow for `mut`
parameters (and for a `mut self` receiver, §5.6) begin. So an argument may
read the value that is about to be borrowed mutably — the same idea as the
assignment rule in §4.9:

```verse
add_item(mut cart, Item(name: "extra", price: total(cart)))   # fine
history.push(history.len())                                   # fine
```

#### Borrows are second-class

A borrowed parameter is a view of the caller's value for the duration of the
call. It **cannot be stored** in a field or collection, **cannot be returned**,
and **cannot be sent to another task**. In particular, a function cannot return
a reference into its argument — it returns a copy, an owned value, or an
optional:

```verse
fn first_name(person: Person)
    person.name                  # error: a view cannot be returned
                                 #   (`person.name` is a view of `person`)
                                 #   help: return `person.name.copy()`, or take
                                 #   the parameter with `take`

fn first_name(person: Person)
    person.name.copy()           # fine: the caller owns the result

fn age_of(person)
    person.age                   # fine: a copy, with an inferred `Copy` bound
```

An unannotated getter such as `age_of` compiles with an inferred `Copy` bound
on the returned field (§4.2, rule 9). Errors about a function's body appear at
the definition. The only call-site errors are unmet bounds, written or
inferred, and they name the bound.

This is the price of having no lifetimes: there is no lifetime syntax and no
elision rules to learn, because a borrow can never outlive the call that
created it. The cost, stated plainly: accessor-style functions copy, and code
that would return a view into a large structure in Rust or C++ is written
differently in Verse — it changes the structure in place through `mut`, takes
ownership with `take`, returns an index, or shares the data through
`Shared<T>` (§11.4).

#### Default values and named arguments

Parameters may have default values. At the call site, arguments can be passed
by position or by name. Positional arguments come first; once a named argument
appears, the rest must be named.

```verse
fn connect(host, port = 80, secure = false)
    ...

connect("a.com")
connect("a.com", port: 443, secure: true)
connect("a.com", secure: true)              # port uses its default
connect(host: "a.com", port: 8080)
```

A named argument is written `name: value` — the same shape as a map entry
(`{"name": value}`) or a record field. When the value is a variable with the
same name as the parameter, it can be left out: `connect(host:, port:)` means
`connect(host: host, port: port)` (*punning*, the same shorthand as in record
construction and patterns). Default values are evaluated at each call, never
shared between calls, and a default fixes the parameter's type when nothing
else does (`port = 80` makes `port` an `Int`). Named arguments are especially
encouraged for `Bool` parameters, and `verse fmt --check` warns on positional
`true`/`false` arguments.

### 5.4 Lambdas

A lambda is written `param -> expression`. Multiple parameters are wrapped in
parentheses; types are inferred from context but may be written.

```verse
double = n -> n * 2
add = (a, b) -> a + b
shout = (text: Text) -> text.upper() + "!"
greet_all = () -> print("Hello, everyone")
```

#### Multi-line lambdas: the closing rule

When a line inside an open `(` ends with `->`, the lambda's body is the
indented block that follows, and **the dedent that ends the block also closes
the parenthesis opened on that line**. The lambda must be the last argument of
that call:

```verse
labels = orders.map(order ->
    status = if order.paid then "paid" else "due"
    due = order.due_date.format("%d %b")
    "{order.id}: {status} (due {due})"
print(labels.join("\n"))
```

There is no separate "trailing lambda" syntax: a lambda is always an argument
inside the call's parentheses, and the closing rule only saves writing the
lonely `)`. In detail:

- One dedent closes **every** parenthesis opened by block lambdas on the lines
  it ends (for example a block lambda nested inside another).
- Nothing can follow the closed call on the dedent line, so a call cannot be
  chained after it; bind the result first (`labels = orders.map(order ->`
  … then `labels.sum()`).
- Writing the `)` explicitly on its own line is accepted; `verse fmt` removes
  it (§16.1).

#### Lambdas in match arms

`->` is used only by lambdas and match arms, so a lambda that forms the body of
a match arm must be wrapped in parentheses to keep the two arrows apart:

```verse
fn operation(op: Op): fn(Int): Int
    match op
        Double -> (n -> n * 2)
        Negate -> (n -> -n)
```

Lambdas that appear as call arguments are already inside parentheses and need
nothing extra: `Evens -> items.filter(n -> n % 2 == 0)`.

#### Key-path shorthand

A leading dot that *starts* an expression — right after `(`, `,` or an
operator — is a one-parameter lambda that reads from its argument:

```verse
ages = users.map(.age)                      # same as u -> u.age
names = users.map(.name.copy())             # u -> u.name.copy()  (owned results, §9.5)
loud = users.map(.name.upper())             # u -> u.name.upper()
firsts = pairs.map(.0)                      # p -> p.0
by_age = users.sort_by(.age)
```

Only plain access chains (fields, tuple positions, method calls) are allowed
after the dot. Anything with an operator is a compile error that suggests a
lambda: `users.filter(.age >= 18)` → write `users.filter(u -> u.age >= 18)`.
After a complete expression, a leading dot still continues a method chain
(§3.4). There are no leading-dot variant names (`.north` is not `North`).

#### Captures: borrowed or moved

A lambda uses variables from its surroundings in one of two ways, decided by
what happens to the lambda:

- A lambda that is **only called while the function it was passed to runs**
  (`map`, `filter`, `sort_by`, `any`, `each`, …) *borrows* its captures
  read-only. So `items.map(x -> x + offset)` does not consume `offset`, and
  `people.filter(p -> p.city == city)` leaves `city` usable. A function-typed
  parameter is non-escaping in this sense unless it is declared `take`. Like
  any borrow (§5.3), a non-escaping function value can be passed on to another
  call, but it cannot be stored, returned, sent to a task or captured by an
  escaping lambda. To keep it, declare the parameter `take`.
- An **escaping** lambda — returned, stored in a field or collection, passed to
  a `take` parameter, or used in `spawn` — *moves* its captures (copy types
  are copied). Using a captured value after the lambda escaped is the usual
  use-after-move error, with a `.copy()` suggestion.

A lambda cannot reassign a captured `mut` variable; use a fold, a loop, or
a returned value instead.

```verse
fn make_greeter(greeting)                   # returns an escaping lambda
    words = greeting.copy()                 # the lambda needs its own copy
    name -> "{words}, {name}!"

hello = make_greeter("Hello")
print(hello("Ana"))
```

Named functions are values too: `shapes.map(area)` passes the function `area`.

#### Tuple auto-spread

A lambda (or function) with *N* parameters may be called with a single
*N*-tuple; the tuple is spread across the parameters. This is what makes
two-parameter lambdas work with `zip`, `enumerate` and `entries`, which produce
pairs:

```verse
totals = prices.zip(quantities).map((price, qty) -> price * qty)
lines = names.enumerate().map((i, name) -> "{i + 1}. {name}")
report = ages.entries().map((name, age) -> "{name} is {age}")
```

Spreading happens only when the arity matches exactly; a one-parameter lambda
receives the whole tuple (`pair -> pair.0`).

#### `?` inside a lambda

`?` inside a lambda returns from the *lambda*, not from the enclosing function,
so the lambda's own return type becomes fallible
(see [§8.3](08-errors.md#83-propagation-with-)):

```verse
doubled = inputs.map(text -> text.trim().to_int()? * 2)       # List<Int | ParseError>
```

### 5.5 Generic functions

Every unannotated parameter is already an implicit generic (§4.2). Explicit
generic parameters name the type so it can be used in several places:

```verse
fn first<T: Copyable>(items: List<T>): T?
    items.get(0).copy()           # `.copy()` on the optional: an owned `T?`
```

The `Copyable` bound is what allows `.copy()` on a `T` (§10.4): the ownership
check runs once, at the definition, and treats `T` as an opaque, non-`Copy`
type unless a bound says otherwise (§4.2, rule 9). Without the bound,
`items.get(0).copy()` is a compile error that suggests adding `T: Copyable`.

See [§10](10-traits-generics.md#10-traits-and-generics).

### 5.6 Methods

Methods are functions declared inside an `impl` block for a type. The first
parameter is `self`.

```verse
type Dog
    name: Text
    tricks: List<Text> = []

impl Dog
    fn speak(self)
        "{self.name} says woof"

    fn learn(mut self, take trick)
        self.tricks.push(trick)

    ## A constructor-style function with no `self`.
    fn puppy(take name)
        Dog(name:)

mut rex = Dog.puppy("Rex")
rex.learn("sit")
print(rex.speak())
```

- `self` borrows the receiver read-only.
- `mut self` borrows it exclusively and may change it — including assigning
  its fields directly, as in `self.name = new_name` or `self.age += 1`. As
  with `mut` parameters, the arguments are evaluated first and only then does
  the exclusive borrow of the receiver begin (§5.3), so
  `self.nodes[p].children.push(self.nodes.len() - 1)` is fine.
- `take self` consumes the receiver (for example `task.wait()`); the receiver
  moves, so it must not be used afterwards.
- Method calls look the same whether or not they mutate — `cart.add(item)`
  carries no `mut` marker, deliberately: the receiver is the dotted name in
  front, so it is already visible. Instead, the **receiver of a `mut self`
  method must be a `mut` binding** (or a mutable field/element of one);
  calling it on an immutable binding is a compile-time error that names the
  binding.
- A function in an `impl` block without `self` is an **associated function**,
  called as `Type.name(...)`.
- `Self` inside an `impl` names the type being implemented.

A type can have several `impl` blocks, in the same module as the type.

### 5.7 Uniform call syntax

Any function in scope can also be called with dot syntax, `x.f(y)`, when its
first parameter accepts `x`. Then `x.f(y)` means `f(x, y)`:

```verse
fn shout(text: Text)
    text.upper() + "!"

print("hello".shout())                 # same as shout("hello")

report = orders.filter(o -> o.paid).total_by_customer().format_table()
```

- **A method always wins.** If the type of `x` has a method `f` (inherent or
  from a trait in scope), `x.f(y)` calls the method.
- **Ambiguity is an error.** If two functions in scope could both be called as
  `x.f(…)`, the call is a compile-time error that lists both and asks you to
  call one as `module.f(x, …)`.
- Parameter modes carry over: if the first parameter is `mut`, the receiver
  must be a `mut` binding (and, as with methods, no marker is written); if
  it is `take`, the receiver moves.

Uniform call syntax is how Verse spells left-to-right pipelines; there is no
separate `|>` operator.

---

