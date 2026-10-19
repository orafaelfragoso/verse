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
iterating: replacing the value of an existing key keeps its position, and
removing a key and inserting it again puts it at the end (the order rule,
§9.4).

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

**Ownership of lookups.** `list[i]`, `m[k]` and `get` give a copy of the
element, which costs a count (§4.9), so the result can be stored, returned or
sent to a task like any value. `remove` and `pop` on a `mut` collection take
the element out, which moves it. An element of a type that is not `Copy`,
such as a `Task`, can only be taken out this way.

**Slices.** `nums[1..]`, `nums[2..5]` and `name[0..=2]` are a list or a text
of their own with the elements of the range. A slice that is only read, as in
`for n in nums[1..]` or `total(nums[1..])`, is read in place, and storing it
copies the range.

### 9.4 Sets

`Set<T>` is a collection of unique values. It iterates in insertion order,
exactly like `Map` (§9.2). **The order rule, for `Map` and `Set` alike:**
replacing the value of an existing key keeps its position (for a set, adding a
value that is already present changes nothing), and removing a key and
inserting it again puts it at the end.

```verse
vowels = set{'a', 'e', 'i', 'o', 'u'}
seen: Set<Text> = set{}
print('e' in vowels)                 # true
both = set{"a", "b"}.union(set{"b", "c"})       # set{"a", "b", "c"}
```

Set operations: `union`, `intersection`, `difference`, `is_subset`, plus `+`
(union) and `-` (difference) as conveniences. Their results keep the order of
the left set; `union` then adds the right set's new values in its order.

### 9.5 Transforming collections

Collections share a rich family of methods (from the `Iterable` trait) that
chain with dots and take lambdas. Chains read top to bottom like a recipe.
A chain that is bound to a name or returned is evaluated eagerly and collected
into a `List` at the end of the statement; only a chain used as a `for` target
or consumed in the middle of a longer chain stays lazy, so a lazy iterator
value is never stored, bound or returned. A collected chain is a `List<…>`
of its own (see *Ownership* below):

```verse
total = numbers.filter(n -> n % 2 == 0).map(n -> n * 2).sum()

top_scorers = players
    .filter(p -> p.active)
    .sort_by(.score, descending: true)
    .prefix(3)
    .map(.name)
```

| Method | Result |
|--------|--------|
| `map(f)` | apply `f` to each element |
| `filter(pred)` | keep elements where `pred` is true |
| `reduce(initial, f)` | combine into one value |
| `sum()`, `product()`, `average()` | numeric summaries |
| `min()`, `max()`, `min_by(key)`, `max_by(key)` | the smallest or largest element, `T?` |
| `count()`, `count(pred)` | number of elements (matching) |
| `any(pred)`, `all(pred)` | `Bool` |
| `find(pred)` | first match, `T?` |
| `first()`, `last()` | `T?` |
| `sort()`, `sort_by(key, descending: false)` | a sorted list |
| `group_by(key)` | `Map<K, List<T>>` |
| `prefix(n)`, `skip(n)`, `prefix_while(p)`, `skip_while(p)` | prefixes and suffixes |
| `enumerate()` | `(index, item)` pairs |
| `zip(other)` | pairs from two collections |
| `flat_map(f)`, `flatten()` | flatten nested collections (an owning step) |
| `unique()` | remove duplicates, keeping first occurrences |
| `reverse()` | the elements in reverse order |
| `chunks(n)`, `windows(n)` | groups of elements, each a slice of the source (§9.3) |
| `join(sep)` | `Text` from a collection of `Show` values |
| `to_list()`, `to_set()`, `to_map()` | collect into a concrete collection (an owning step) |

**Ownership: every result is a value.** These methods read the collection
they are called on and pass each element to the lambda. What a chain produces
is a value of its own, like the result of any expression (§4.9):

- **Adapters** (`filter`, `sort`, `sort_by`, `prefix`, `skip`,
  `prefix_while`, `skip_while`, `unique`, `reverse`, `zip`, `enumerate`, …)
  hand elements along without copying them.
- **The end of the chain** builds the result. `map(f)` holds what the lambda
  returns, a reduction (`sum`, `count`, `join`, `any`, `all`, `reduce`, …)
  produces a new value, and a collecting step (`to_list()`, `to_set()`,
  `to_map()`, `flat_map(f)`, `flatten()`) builds a new collection. Elements
  of `Copy` types are copied into it, which costs a count each, and move
  instead when the source is at its last use (a temporary, or a binding that
  is not used afterwards).
- **A bound or returned chain** is collected into a `List` at the end of the
  statement, and side effects in its lambdas run then:
  `tasks = urls.map(url -> spawn fetch(url))` is a `List<Task<…>>` whose
  tasks are spawned when that statement executes. `to_list()` is needed only
  where the chain would otherwise be consumed as something else, as a `for`
  target or to name the element type, not for binding.
- **Single-element results** (`find`, `first`, `last`, `min`, `max`, `min_by`,
  `max_by`) are a copy of the element, as `list[i]`, `m[k]` and
  `get(k, default: x)` are (§9.3). On a source at its last use, such as a
  `take` parameter or a temporary, the element moves out and the rest of the
  collection is dropped (see `largest` in §20.3).
- **Elements that are not `Copy`.** A chain over a collection of `Task`s,
  `File`s or handles (§4.9) can only move its elements, so its source must be
  at its last use. Otherwise it is a compile error that names the element
  type and the later use of the source.

```verse
adults = people.filter(p -> p.age >= 18)           # a list of its own
print(adults.len())
names = adults.map(.name)
oldest = people.max_by(.age)                       # a Person?
print(people.len())                                # `people` is unchanged
```

**Key functions.** Comparison key functions (`sort_by`, `min_by`, `max_by`)
read their keys in place, so `people.sort_by(.name)` copies no name.
`group_by` stores its keys and groups in the resulting map, which owns copies
of them: `people.group_by(.city)`.

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
`math.OrderedFloat` if needed.

---

