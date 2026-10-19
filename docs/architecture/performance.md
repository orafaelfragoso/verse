# 09. Performance

Requirement R4: really fast and efficient. This document turns that into
budgets, a benchmark suite, and a list of places where the **language
specification itself** creates performance risk.

Everything numeric below is a **target**. No measurement exists yet. M0 builds
the harness, and each target becomes a regression gate once it has a baseline.

## 9.1 What we measure

| Dimension | Metric | Initial target | Gate |
|-----------|--------|----------------|------|
| Compile speed | `verse check` on 10k lines, cold | under 300 ms on 8 cores | M1 |
| Compile speed | `verse check` after a one-line edit | under 30 ms | M1 |
| Compile speed | `verse build` (Cranelift) 10k lines | under 2 s | M2 |
| Startup | `verse hello.vs`, warm cache | under 20 ms | M1 |
| Script speed | tier 0 on the benchmark suite | within 30x of optimized native, aiming at 10x to 20x | M1 |
| Native speed | Cranelift tier | within 2x of the LLVM tier | M2 |
| Native speed | LLVM tier | within 1.3x of equivalent Rust on the suite | M7 |
| Memory | peak RSS of hello world | under 2 MB | M2 |
| Binary size | hello world, native, stripped | under 200 KB | M2 |
| Binary size | hello world, Wasm | under 30 KB | M3 |
| Binary size | Cortex-M blinky, flash | under 16 KB | M7 |
| Task cost | spawn plus wait of an empty task | under 300 ns, 1 million live tasks fit in 2 GB | M4 |
| Channel | send plus receive between two tasks | under 150 ns | M4 |

The comparison baselines are Rust, Go and Python for run speed (a language of
the same safety level, a language with the same lightweight tasks, and a
scripting language), and Lua and CPython for startup.

## 9.2 Benchmark suite

Kept in `bench/`, run by `verse bench`, results stored as JSON and compared
against the committed baseline.

| Group | Programs | What they stress |
|-------|----------|------------------|
| Micro | arithmetic loops, field access, calls, checked overflow, list index | the cost of checks, calls and moves |
| Algorithms | n-body, spectral norm, fannkuch, binary trees, mandelbrot, fasta | classic cross-language comparison |
| Data structures | map insert and lookup, set operations (ordered set vs a plain hash set baseline, to confirm the P8 cost), sort, text building, JSON parse | collections and the allocator |
| Chains | `filter`, `map`, `sum` chains over large lists | that iterator chains compile to loops |
| Concurrency | spawn storm, ping-pong over channels, parallel map, `Guarded` contention | scheduler and synchronization |
| Real programs | the three programs in spec §20, a small HTTP server, a CSV aggregator | end-to-end behavior |
| Compiler | self-generated 1k, 10k, 100k-line projects | compile speed and memory |

Rules:

1. **Profile before optimizing.** A change motivated by performance needs a
   profile or a benchmark delta in its description.
2. **Report medians and spread**, from at least 10 runs, pinned CPU frequency
   where possible. Noise above 3% means the benchmark is not yet a gate.
3. **Explain the number.** A reported result states what the limiter is (CPU,
   memory bandwidth, allocator, locks), the tuning used, and the repeat
   variance.
4. **CI gate**: a 5% regression on any gated benchmark fails the build unless
   the author records an accepted trade-off.

## 9.3 Where speed comes from (design levers)

1. **Monomorphization and inlining.** Unannotated generics are specialized, so
   `fn double(n)` over `Int` is just a multiply. Iterator chains inline into
   loops because the collection API is written in Verse ([05](runtime.md),
   section 5.5).
2. **No garbage collector.** No pauses, no write barriers, no tracing.
   Deterministic frees map to `free` calls the allocator handles in a few
   nanoseconds.
3. **Move at last use.** The ownership pass turns copies into moves, so the
   price of the safety model is mostly paid at compile time.
4. **Checks as data.** Overflow and bounds checks are explicit MIR, so range
   analysis can remove those it proves redundant (for example, loops over
   `0..list.len()` and constant indices).
5. **Static dispatch by default.** Only trait types (`Speaker` used as a type)
   dispatch dynamically, and `devirtualize` removes those when the concrete
   type is known.
6. **Niche and layout optimization** shrink `T?`, `T | Error` and variants.
7. **Non-atomic reference counts** when the program provably never spawns.
8. **Cheap tasks** via lazily committed stacks or state machines.
9. **Compile-time work moved off the critical path**: incremental queries,
   function-level parallelism, and a persistent cache.

## 9.4 Spec-level performance risks

These are not bugs. They are places where the specified semantics cost
something real, and where the specification should state the price. Each has a
proposed mitigation that **does not change semantics**. Where semantics had
to change, the last column records the ruling.

| ID | Risk | Where | Mitigation inside the spec | Needs a ruling? |
|----|------|-------|----------------------------|-----------------|
| P1 | `Text` is indexed by characters, so `text[i]`, `slice` and `len()` are O(n) on non-ASCII text | §4.1 | cache an "is ASCII" flag and a character count in the header; ASCII text indexes in O(1); document the complexity in the spec | ruled: §4.1 states the cost and a byte-view API is documented |
| P2 | `Int` is 64-bit everywhere, with overflow checks | §4.1 | range analysis; wide ops lowered to 32-bit when provable; sized types exist | no |
| P3 | A view cannot be returned or stored, so some APIs copy | §18.10 | `move-elim` and `copy-elide`; chains collect once; arenas and indices for graph data | no |
| P4 | Ownership flips with later code | §4.9 | none needed at run time; the cost is diagnostic quality, handled in [02](compiler-architecture.md) section 2.4 | no |
| P5 | `Shared` is reference counted, atomic when tasks exist | §4.10 | non-atomic when no spawn is reachable; copy elision; `Guarded` uses a lock with spin-then-park | no |
| P6 | Unannotated generics are specialized per type, so compile time and code size can grow | §4.2 | identical code folding; lazy instantiation in script mode; size report | no |
| P7 | Every task can unwind with `Cleanup` | §11.5 | unwind tables cost no run time on the success path; status-code mode costs a branch per call that can unwind | ruled: panics clean up on every target |
| P8 | `Set` is "unordered": implementation freedom helps speed but breaks replay and reproducibility | §9.4 | **ruled: insertion order**; same compact layout as `Map`, one index array, about 4 bytes per element more than an unordered table (assumption to measure in M1) | ruled |
| P9 | `spawn` moves all arguments in, so large values are moved or copied at spawn | §11.1 | moves are pointer-sized for heap data; copy only where the program says `.copy()` | no |
| P10 | Fallible results cannot be dropped and `T \| Error` is a tagged value | §8 | niche packing and a pointer-sized error for the common case | no |

## 9.5 Compile-time budget discipline

- Every pass records time and memory with `--timings` (JSON output for
  tools).
- No quadratic algorithm over module size is allowed in a stage that runs on
  every edit. Type inference and ownership are per function, so a 100k-line
  file with small functions stays linear.
- A pathological case, such as one function with 50k statements, is a test
  (`bench/compiler/pathological/`), not an afterthought.
