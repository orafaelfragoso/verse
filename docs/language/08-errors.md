## 8. Errors and missing values

Verse has two related, but distinct, ideas:

- **Missing values** — "there may be nothing here" — use optional types `T?`.
- **Errors** — "something went wrong, and here is why" — use fallible types
  `T raises E`.

Neither involves exceptions or null. One type form of `?`, one pattern, and
two words, `try` and `otherwise`, cover almost everything:

| Form | Where | Meaning |
|------|-------|---------|
| `T?` | type | a value that may be missing |
| `T raises E` | type | a value, or an error of one of the types in `E` (§8.2) |
| `try expr` | prefix | stop early: if `expr` is an error or missing, return it from the current function |
| `x?` | pattern | match a present value (§7.3) |
| `a otherwise b` | expression | fallback: `a` if usable, else `b` |

A plain dot looks through an optional when the chain is consumed by one of
these (§8.1); there is no separate safe-access operator.

### 8.1 Optional types `T?`

There is no `null`. A value of type `T` is always a real `T`. A value that might
be absent has type `T?`, and is either a `T` or `none`.

```verse
fn find_user(id: Int): User?
    users.find(u -> u.id == id)             # `users`: a module constant (§12.4)

maybe: Text? = none
present: Text? = "here"         # a T is automatically a T?
```

You cannot use a `T?` where a `T` is expected; the compiler insists you decide
what to do about `none`. The tools for this are:

| Tool | Meaning | Example |
|------|---------|---------|
| `otherwise` | fallback value | `nickname otherwise "friend"` |
| `if … is x?` | branch on presence | `if find_user(id) is u?` |
| `match` | full control | `none -> …` / `u? -> …` |
| `try` | propagate `none` out of a function returning an optional | `try user` |
| `.expect(msg)` | assert presence; stops the program with `msg` otherwise | `config.expect("config required")` |
| `.or_error(msg)` | turn `none` into an error, usually after `try` (§8.3) | `try find_user(id).or_error("no such user")` |

#### Plain dots through optionals

There is no safe-access operator. A plain dot (a field, a method call, a chain of
them) may look *through* an optional, but only where the chain is
**immediately consumed** by something that deals with `none`:

| The chain is consumed by… | Example |
|---------------------------|---------|
| (a) `otherwise` | `city = order.customer.address.city otherwise "unknown"` |
| (b) `try` | `email = try find_user(id).email` (returns `none` from the function) |
| (c) an `is` pattern or a `match` subject | `if order.customer.address is a?` |
| (d) a position whose expected type is optional | `city: Text? = order.customer.address.city`; `return user.email` in a function returning `Text?`; an argument to a `Text?` parameter |

Anywhere else, a dot on a `T?` is a compile-time error that names the
consumers a chain may end in:

```
error: `order.customer` is `Customer?` and cannot be used as `Customer` here
  --> orders.vs:3:8
   |
 3 | city = order.customer.address.city
   |        ^^^^^^^^^^^^^^ the dot looks through an optional only when the chain is in `try`, ends in `otherwise`, `is` or `match`, or is in an optional-typed position
   |
help: add one of those at the end of the chain
explain: V0415 (run `verse explain V0415`)
```

The four ways out are a fallback with `otherwise`
(`city = order.customer.address.city otherwise "unknown"`), propagation with
`try` (`city = try order.customer.address.city`), a branch with `is`
(`if order.customer is c?`), and, for an unannotated binding, annotating the
optional type (`city: Text? = order.customer.address.city`).

The rules of a lifted chain:

- **It short-circuits completely.** At the first `none`, the rest of the chain
  is not evaluated — later hops, method calls, and their arguments.
- **It yields one level of optional.** A hop that itself returns `X?` does not
  create `X??`: "something along the chain was missing" is the only question a
  chain answers, so `order.customer.nickname` (with `nickname: Text?`) is a
  plain `Text?`. Lookups keep their own types: `m.get(k)` on a
  `Map<Text, Int?>` is still `Int??` (below).
- **The result is a value**, as for chains without optionals (§9.5).
- **The optional's own methods win.** `.expect(msg)`, `.or_error(msg)` and
  `.copy()` on a `V?` act on the optional itself, not on the value inside:
  `shared_user.copy()` on a `Shared<User>?` is a second handle, or `none`. If
  a name is both a method of
  the optional and a field or method of the inner type, the call is a
  compile-time error that asks you to choose with `is` or `otherwise`.
- **`.or_error(msg)` applies to the final result of the chain.** In
  `try order.customer.address.city.or_error("no city")`, a `none` anywhere along
  the chain becomes the error.

`try` covers the whole postfix chain to its right (§3.8). So
`try find_user(id).email` reads through the optional and propagates if
anything along the chain is missing, and the bracketed
`(try find_user(id)).email` propagates a missing user out of the function and
then reads `.email` from the `User`. Both read as they run, so there is no
operator that could be mistaken for safe access.

#### Nested optionals are real

*Style:* avoid nested optionals such as `T??` in public APIs; prefer a variant
that names each case (`type Reply = NoSuchKey | Pending | Value(v: Int)`). The
`none?` and `score??` patterns below exist for generic code.

Outside lifted chains, optionals are **never** flattened: `T??` is a different
type from `T?`. This matters for lookups. `map[key]` and `list.get(i)` return
`V?`, where `V` is the element type — so on a `Map<Text, Int?>` a lookup has
type `Int??`, and "the key is missing" (`none`) stays distinguishable from "the
key is present and its value is `none`" (a present `none`):

```verse
scores: Map<Text, Int?> = {"ana": 7, "bruno": none}

match scores["bruno"]          # Int??
    none -> print("no such player")
    none? -> print("bruno has not played yet")
    score?? -> print("score {score}")
```

On ordinary maps and lists (whose elements are not optional), lookups are plain
`V?` and none of this comes up.

### 8.2 Fallible types `T raises E`

A function that can fail declares what it can raise after its return type:

```verse
fn read_text(path: Text): Text raises Error        # signatures only
fn parse_port(text: Text): Int raises ParseError
fn fetch(url: Text): Text raises FetchError | Timeout
```

`ParseError` is in the prelude (§15.1); it is the error type of `Text.to_int()`,
which returns `Int raises ParseError` (and of `Text.to_float()`, which returns
`Float raises ParseError`).

The body returns either a `T` (success) or an error value. A value of an error
type returned as the last expression, or with `return`, is the failure:

```verse
fn divide(a: Float, b: Float): Float raises Error
    if b == 0.0
        return error("division by zero")
    a / b
```

**The clause is checked both ways.** Every error the body can raise, by
`return` or by `try` (§8.3), must be in the clause; one that is not is an error
that names it. A member of the clause that nothing in the body can raise is a
warning:

```verse
fn load(): Report raises FetchError | Timeout | ParseError
    page = try fetch(u)              # keeps FetchError | Timeout
    try parse(page)                  # adds ParseError; the clause lists it

fn quiet(): Report raises Timeout
    try parse(fetch(u))              # error: `fetch` can raise `FetchError`, which `quiet` does not declare

fn careful(): Report raises Timeout | ParseError
    try parse(cached)                # warning: `Timeout` is declared, but nothing in `careful` raises it
```

A function whose return type is written and has no clause cannot fail. A
function whose return type is left to inference also has its raise set
inferred: the errors its body can raise (§4.2).

#### The `Error` trait

Error types implement the built-in `Error` trait:

```verse
trait Error: Show
    fn message(self): Text
    fn cause(self): Error?
        none
```

`raises Error` (with the trait name itself) accepts *any* error type, the most
common choice for application code. The `error(…)` function below makes an
`Error`, so a function that uses it raises the open set. Libraries often define specific error
types:

```verse
type FetchError = NoResponse(seconds: Int) | BadStatus(code: Int) | Offline

impl Error for FetchError
    fn message(self): Text
        match self
            NoResponse(s) -> "no response after {s}s"
            BadStatus(c) -> "server answered {c}"
            Offline -> "no network connection"
```

The built-in function `error(message: Text, cause: Error? = none): Error`
creates a simple error value.

### 8.3 Propagation with `try`

The prefix word `try` means **"stop early: if this is an error or missing,
return it from the current function; otherwise give me the value."** It
covers the whole postfix chain to its right, calls inside that chain's
arguments included, so one `try` covers every call in
`try json.parse(read_text(path))`. It binds looser than a call or a dot and
tighter than `**` and every binary operator (§3.8), so
`try nickname(7) otherwise "anonymous"` is
`(try nickname(7)) otherwise "anonymous"`.

```verse
import json
import read_text from fs

fn load_settings(path: Text): Settings raises Error
    text = try read_text(path)
    data = try json.parse(text)
    Settings.from_json(data)
```

Rules for `try expr`:

| `expr` has type | enclosing function returns | effect |
|-----------------|----------------------------|--------|
| `T raises E` | `U raises C`, where `C` contains every member of `E` | on error, return that error, keeping its type; otherwise yields `T` |
| `T raises E` | `U raises Error` | on error, return it; otherwise yields `T` |
| `T raises E` | `U?` (no clause) | compile-time error: an error cannot become `none`; handle it, or add `raises E` |
| `T raises E` | `U raises C`, where `C` lacks a member of `E` | compile-time error that names the member: add it to the clause, convert with `.context("…")` (gives `Error`), or `match` |
| `T?` | `U?` | on `none`, return `none`; otherwise yields `T` |
| `T?` | `U raises C` | compile-time error: say why with `.or_error("…")` |
| `T raises Error` | `U raises FetchError` | compile-time error: the open set is not a member of a specific one |

**Layers are peeled from the outside, in written order.** For a type with
several layers, `try` and `otherwise` act on the *outermost* layer only. In
`Text? raises Error`, the outermost layer is the error side:

```verse
fn nickname(id: Int): Text? raises Error
    ...

a = try nickname(7)                          # Text?  (errors propagated)
b = nickname(7) otherwise none               # Text?  (errors replaced by none)
c = nickname(7) otherwise "anonymous"        # error: expected `Text?`, found `Text`
d = try nickname(7) otherwise "anonymous"    # Text   (error propagated, then none handled)
e = nickname(7) otherwise none otherwise "anonymous"       # Text
```

The same rule makes an `Int??` need two steps: `try try scores[name]` peels
both layers, and `x otherwise none otherwise 0` handles both. (`x??` exists
only as a *pattern*, meaning "present and present", §7.5.)

**Where `try` returns to.** `try` always returns from the *innermost*
function-like body that contains it:

| `try` appears in… | it returns from… |
|-----------------|------------------|
| a named function | that function |
| a lambda | the lambda, which needs its own `try` (its return type becomes fallible or optional; §5.4) |
| a function run with `spawn` | the spawned function; the error is delivered by `.wait()` (§11.1) |
| a `test` block | the test, which then fails with the error's message (§14.1) |
| entry-file top-level code | the script, which exits (see below) |

To turn a missing value into an error explicitly:

```verse
user = try find_user(id).or_error("no user with id {id}")
```

At the **top level of a script** (outside any function), `try` on a failed value
prints the error message to standard error and exits with status 1. On `none`
it prints `error: missing value at file.vs:LINE` and exits with status 1.

### 8.4 Handling errors

```verse
import fs
import read_text from fs

# Fallback value
notes = read_text("notes.txt") otherwise ""

# Branch on success or failure
match read_text("notes.txt")
    text: Text -> print(text)
    err: Error -> print("could not read notes: {err.message()}")

# Add context while propagating
config_text = try read_text(path).context("loading {path}")

# Ignore a result on purpose (the only way; see below)
_ = fs.remove("cache.tmp")
```

`.context(msg)` wraps the error in a new error whose `cause()` is the original,
so its result raises the open set: `T raises E` becomes `T raises Error`.
`.or_error(msg)` turns a `T?` into a `T raises Error` in the same way.

#### Fallible results cannot be dropped

**Discarding a fallible value is a compile-time error.** A call whose result
is `T raises E` cannot stand alone as a statement, and neither can a value
that *contains* fallible results: a `List<T raises Error>` from `wait_all`, a
`Map<K, V raises E>`, a tuple with a fallible part. Either handle it (`try`,
`otherwise`, `match`, a loop over the results) or discard it visibly with
`_ = …`:

```verse
fs.remove("cache.tmp")                 # error: a fallible value cannot be ignored
                                       #   help: handle it with `try`, `otherwise` or `match`, or discard it with `_ = …`
try fs.remove("cache.tmp")             # propagate
_ = fs.remove("cache.tmp")             # deliberately ignored
```

### 8.5 Panics

Some failures are bugs, not conditions to handle: indexing a list out of range,
integer overflow, dividing an integer by zero, a failed `assert`, or
`.expect(...)` on `none`. These **panic**: the current task stops (dropping
every value it still owns, §13), the message and a stack trace are **always printed to
standard error at the moment of the panic**, and — if the task is the main
program — the program exits with status 101. A panic in a spawned task is
delivered to its waiter as a `TaskPanicked` error (§11.2). Panics are not caught by `try` and are not meant
for normal control flow. Code that wants to avoid them uses the checked
alternatives (`items.get(i)`, `a.checked_div(b)`).

---

