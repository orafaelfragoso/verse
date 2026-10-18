## 9. Collections

### 9.1 Lists

`List<T>` is an ordered, growable sequence.

```verse
nums = [1, 2, 3]
print(nums[0])                    # 1      — Int; out of range panics
print(nums.get(10))               # none   — Int?
print(nums.get(10, default: 0))   # 0      — Int
print(nums.len())                 # 3
print(nums[1..])                  # [2, 3]
print(nums[..2])                  # [1, 2]
print(nums + [4, 5])              # [1, 2, 3, 4, 5]

mut queue: List<Text> = []
queue.push("first")
queue.push("second")
next = queue.pop_front()       # Text?
queue[0] = "changed"
```

Negative indices are not allowed; use `nums.last()` (which returns `T?`).

### 9.2 Maps

`Map<K, V>` associates keys with values. Insertion order is preserved when
iterating.

```verse
ages = {"ana": 30, "bruno": 17}
print(ages["ana"])                    # 30 — Int? (present)
print(ages["zoe"])                    # none
print(ages.get("zoe", default: 0))    # 0  — Int

mut stock: Map<Text, Int> = {:}
stock["apples"] = 10                              # insert or replace
stock["apples"] = stock.get("apples", default: 0) + 5
_ = stock.remove("pears")                         # remove returns the old value, Int?
print("apples" in stock)                          # true
```

Writing `map[key] = value` inserts or replaces. Keys must implement `Hash` and
`Eq`. Iterating a map — or calling `map.entries()` — yields labeled pairs
`(key: K, value: V)`, which can be destructured (`for (name, age) in ages`),
passed to a two-parameter lambda (§5.4), or read by label (`entry.key`,
`entry.value`). An empty map is written `{:}`; a bare `{}` is a compile-time
error that points to `{:}` and `set{}`.

### 9.3 Lookups side by side

Lists and maps answer "what is at this position/key?" differently, on purpose:

| Expression | On a `List<T>` | On a `Map<K, V>` |
|------------|----------------|------------------|
| `c[i]` / `m[k]` | `T` — **panics** if out of range | `V?` — `none` if the key is missing |
| `c.get(i)` / `m.get(k)` | `T?` | `V?` (same as `m[k]`) |
| `c.get(i, default: x)` / `m.get(k, default: x)` | `T` — `x` if out of range | `V` — `x` if missing |
| `c[i] = v` / `m[k] = v` | replace (panics if out of range) | insert or replace |

A bad list index is almost always a bug (and loops over `0..items.len()` should
not need unwrapping), while a missing map key is ordinary data — see §18.11.

The one-argument `get` returns `V?` **without flattening**: on a
`Map<Text, Int?>` it returns `Int??` (§8.1). The two-argument
`get(key, default: x)` always returns a plain `V`, never an optional, and is
the idiomatic way to count or accumulate:

```verse
counts[word] = counts.get(word, default: 0) + 1
```

`default:` is a parameter of `get` only; it is not a general language feature.
`get(k, default: x)` follows the same ownership table as `otherwise`
(§3.8.2), with the element as the left side and the default as the right:

| Element | Default | Result |
|---------|---------|--------|
| a view (always, from a borrowed collection) | a view or a static literal (`0`, `""`) | a **view**: `names.get(id, default: "anonymous")` |
| a view | an owned temporary (`Text.new()`, `make_label()`) | **owned**, so the element side needs `.copy()`: write `names.get(id, default: Text.new())` as `names.get(id).copy() otherwise Text.new()` |

Without the copy, the second row is a compile-time error: a view of an owned
temporary default would not outlive the call. `names.get(id).copy() otherwise
Text.new()` is the intended way to get an owned element with an owned
default. It works because `.copy()` on an optional view `V?` of a `Copyable`
`V` yields an owned `V?` (`none` stays `none`), which then meets the owned
default as in the second row of the `otherwise` table (`.copy()` is one of
the optional's own methods, §8.1). Where an owned value is needed —
storing it, returning it, sending it to a task — copy the view result:
`names.get(id, default: "anonymous").copy()`.

**Ownership of lookups.** `list[i]`, `m[k]` and `get` give a *view* of the
element — a second-class borrow that can be read, passed to functions, or bound
to a name, but not stored in a field or collection, returned, or sent to a
task. To keep an element beyond that, copy it (`m["ana"].copy()`) or take it
out of a `mut` collection with `remove`/`pop`, which return owned values.
`Copy` types (numbers, `Bool`, `Char`, and records made only of them) are
simply copied.

**Slices are views too.** `nums[1..]`, `nums[2..5]` and `name[0..=2]` give a
view of part of the list or text, with the same rules as `list[i]`. Write
`nums[1..].copy()` or `name[0..=2].copy()` to get an owned list or text.

### 9.4 Sets

`Set<T>` is an unordered collection of unique values.

```verse
vowels = set{'a', 'e', 'i', 'o', 'u'}
seen: Set<Text> = set{}
print('e' in vowels)                 # true
both = set{"a", "b"}.union(set{"b", "c"})       # set{"a", "b", "c"}
```

Set operations: `union`, `intersection`, `difference`, `is_subset`, plus `+`
(union) and `-` (difference) as conveniences.

### 9.5 Transforming collections

Collections share a rich family of methods (from the `Iterable` trait) that
chain with dots and take lambdas. Chains read top to bottom like a recipe.
A chain that is bound to a name or returned is evaluated eagerly and collected
into a `List` at the end of the statement; only a chain used as a `for` target
or consumed in the middle of a longer chain stays lazy, so a lazy iterator
value is never stored, bound or returned. A collected chain that ends in an
owning step is an owned `List<…>`; one that ends in adapters only is a local
view (see *Ownership* below):

```verse
total = numbers.filter(n -> n % 2 == 0).map(n -> n * 2).sum()

top_scorers = players
    .filter(p -> p.active)
    .sort_by(.score, descending: true)
    .prefix(3)
    .map(.name.copy())
```

| Method | Result |
|--------|--------|
| `map(f)` | apply `f` to each element |
| `filter(pred)` | keep elements where `pred` is true |
| `reduce(initial, f)` | combine into one value |
| `sum()`, `product()`, `average()` | numeric summaries |
| `min()`, `max()`, `min_by(key)`, `max_by(key)` | the smallest or largest element, `T?` (a view) |
| `count()`, `count(pred)` | number of elements (matching) |
| `any(pred)`, `all(pred)` | `Bool` |
| `find(pred)` | first match, `T?` (a view) |
| `first()`, `last()` | `T?` (a view) |
| `sort()`, `sort_by(key, descending: false)` | a sorted list |
| `group_by(key)` | `Map<K, List<T>>` |
| `prefix(n)`, `skip(n)`, `prefix_while(p)`, `skip_while(p)` | prefixes and suffixes |
| `enumerate()` | `(index, item)` pairs |
| `zip(other)` | pairs from two collections |
| `flat_map(f)`, `flatten()` | flatten nested collections (an owning step) |
| `unique()` | remove duplicates, keeping first occurrences |
| `reverse()` | the elements in reverse order |
| `chunks(n)`, `windows(n)` | groups of elements, each a view of a slice (§9.3) |
| `join(sep)` | `Text` from a collection of `Show` values |
| `to_list()`, `to_set()`, `to_map()` | collect into a concrete collection (an owning step) |

**Ownership: views flow through a chain; the end of the chain decides.** These
methods borrow the collection they are called on and pass each element to the
lambda as a borrow. Nothing is ever copied silently:

- **Adapters pass views.** Within one expression chain, the methods that hand
  elements along (`filter`, `sort`, `sort_by`, `prefix`, `skip`,
  `prefix_while`, `skip_while`, `unique`, `reverse`, `zip`, `enumerate`, …)
  pass second-class views of the source's elements. Nothing is copied.
- **The chain ends in an owning step**, which decides what the result owns:
  - `map(f)` — the result holds what the lambda returns, which must be owned:
    `.map(p -> p.age)` copies an `Int` (a `Copy` type), while
    `.map(p -> p.name)` is an error that asks for `.map(p -> p.name.copy())`
    (or the key path `.map(.name.copy())`);
  - a reduction (`sum`, `count`, `join`, `any`, `all`, `reduce`, …), which
    produces a new value;
  - a move out of an owned source at its last use (a temporary, or a binding
    that is not used afterwards), which moves the elements instead of copying
    them;
  - an explicit `.copy()`, which duplicates the remaining elements:
    `people.filter(p -> p.active).copy()` is an owned `List<Person>`;
  - a collecting step — `to_list()`, `to_set()`, `to_map()`, `flat_map(f)`,
    `flatten()` — which builds a new owned collection: it moves the elements
    from a source at its last use, and otherwise needs `.copy()` before it.
    A bound or returned chain is collected into an owned list at the end of
    the statement, and side effects in its lambdas run then:
    `tasks = urls.map(url -> spawn fetch(url.copy()))` is a `List<Task<…>>`
    whose tasks are spawned when that statement executes. `to_list()` is
    needed only where the chain would otherwise be consumed as something
    else — as a `for` target, or to name the element type — not for binding.
    (`chunks(n)` and `windows(n)` are not owning steps: each group is a view
    of a slice of the source, §9.3.)
- **No implicit duplication of non-`Copy` elements.** If a chain would have
  to duplicate an element that is not `Copy` to produce an owned result — for
  example `adults: List<Person> = people.filter(p -> p.age >= 18)` while
  `people` is still used later — that is a compile error with the usual
  suggestion: add `.copy()` (which needs `Copyable`, §10.4) or move the later
  use. `Copy` elements (numbers, `Bool`, `Char`, …) are copied implicitly as
  everywhere else.
- **Single-element results are views.** `find`, `first`, `last`, `min`, `max`,
  `min_by` and `max_by` return a view of the element, as `list[i]`, `m[k]`
  and `get(k, default: x)` do (§9.3). So
  `people.find(p -> p.id == id).email otherwise "none"` copies nothing, and
  keeping the element is written `people.find(p -> p.id == id).copy()` (an
  owned `Person?`; `.copy()` acts on the optional, §8.1).
  That is the rule for a *borrowed* source. On an **owned source at its last
  use** — a `take` parameter or a temporary — the single element **moves
  out** and the rest of the collection is freed; this is the owned
  counterpart of the view rule (see `largest` in §20.3).
- **A binding to a chain is a local view.** `adults = people.filter(…)`
  is allowed when `people` is used later: `adults` is a view that lives only in
  the current scope, exactly like `b = a` (§4.9). It can be read, iterated
  and passed to functions, but not stored, returned or sent to a task.

```verse
adults = people.filter(p -> p.age >= 18)           # a local view; nothing copied
print(adults.len())
names = adults.map(.name.copy())                   # the one visible copy
oldest = people.max_by(.age).copy()                # an owned Person?
print(people.len())                                # `people` is still usable
```

**Key functions.** Comparison key functions (`sort_by`, `min_by`, `max_by`)
may return views, so `people.sort_by(.name)` needs no copy of each name.
`group_by` stores its keys in the resulting map, so its key function must
return an owned value: `people.group_by(.city.copy())`. Its groups hold the
elements themselves, so `group_by` is an owning step: it moves from a source
at its last use, or needs `.copy()` before it.

`min()`, `max()` and `average()` on a possibly empty collection return `T?`;
on a list known non-empty by the caller, combine with `otherwise` or `.expect()`.

### 9.6 Equality and ordering

`==` and `!=` compare **structurally**: two values are equal when they have the
same type and equal contents. Because every value has a single owner and there
are no references in the language, identity is never observable, so there is
no separate identity comparison.

```verse
assert [1, 2] == [1, 2]
assert User(name: "A", age: 1) == User(name: "A", age: 1)
assert {"a": 1} == {"a": 1}
```

Built-in types, tuples, lists, records and variants whose parts are comparable
automatically implement `Eq`; types whose parts are ordered automatically
implement `Order` (lexicographic for tuples and lists; declaration order for
variants; field order for records). Comparing values of different types is a
compile-time error, with one convenience: a `T` may be compared with a `T?`
(so `counts["the"] == 3` works on a map lookup). `Float` equality follows IEEE 754 (`nan != nan`), so `Float`
implements `Eq` only partially and cannot be a map key or set element; use
`std.num.OrderedFloat` if needed.

---

