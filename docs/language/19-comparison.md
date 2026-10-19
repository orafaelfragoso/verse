## 19. Verse vs other languages

| | **Verse** | Python | Rust | Go | Swift | Kotlin |
|---|---|---|---|---|---|---|
| Blocks | indentation | indentation | braces | braces | braces | braces |
| Typing | static; annotations optional, even on parameters | dynamic (optional hints) | static, inferred | static, light inference | static, inferred | static, inferred |
| Missing values | `T?`, no null | `None` anywhere | `Option<T>` | `nil` pointers | `T?` | `T?` |
| Errors | `T \| E` + `?` | exceptions | `Result<T, E>` + `?` | `(T, error)` returns | `throws` / `try` | exceptions / `Result` |
| Default mutability | immutable (`x = …` / `mut x = …`) | mutable | immutable (`let` / `let mut`) | mutable | `let` / `var` | `val` / `var` |
| Data semantics | values: assignment copies, storage shared until written; `mut` / `take`; handles shared with `.copy()` | references | ownership & borrowing | values + pointers | structs by value, classes by ref | references |
| Memory | ownership, no GC; second-class borrows, no lifetimes | automatic (RC + GC) | ownership, no GC | tracing GC | ARC | tracing GC (JVM) |
| Concurrency | `spawn` / `.wait()`, no coloring | threads, `async`/`await` | threads, `async`/`await` | goroutines, channels | `async`/`await`, actors | coroutines (`suspend`) |
| Generics | `<T: Trait>` | duck typing / hints | `<T: Trait>` | `[T any]` | `<T: Protocol>` | `<T : Bound>` |
| Interfaces | `trait` + `impl … for` | classes, protocols | `trait` + `impl … for` | implicit interfaces | protocols | interfaces |
| Built-in tests | `test "name"` blocks | library (`unittest`) | `#[test]` functions | `_test.go` files | XCTest library | JUnit library |
| Official formatter | `verse fmt`, no options | external (`black`) | `rustfmt` | `gofmt` | `swift-format` | `ktlint` (external) |
| Script style | yes | yes | no | no | yes (scripts) | scripts (`.kts`) |

In one sentence: Verse aims for **Python's readability, Rust's safety model,
Go's simple concurrency, and Swift's optionals**, while keeping punctuation to a
minimum and spelling out logic and fallbacks in words.

**Compared with Rust** specifically: Verse gives the same core guarantees — no
garbage collector, deterministic freeing, no use-after-free, no data races —
with no lifetime syntax, no reference types and no elision rules to learn. The
price is that there are no references: a function cannot return or store one,
so some Rust patterns (returning a view into a structure, structs that hold
references, tasks borrowing from their parent) become a copy, a `mut`
operation in place, an index, or a `Shared<T>` handle in Verse. A copy shares
its storage until one side writes, as Swift's arrays and strings do, so it
costs an atomic count rather than a duplicate.

---

