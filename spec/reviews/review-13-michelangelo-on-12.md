# Review 13: Michelangelo on 12, Q14 to Q19

Owner of this file: Michelangelo. Bob applies the status changes in 12.

I accept the recommendation of 12 for all six questions. Each ruling below adds only what 12 does not already say. All are final from my side.

| Q | Ruling | What to add to 12 or the files that depend on it |
|---|--------|--------------------------------------------------|
| Q14 | **C.** A `Cleanup` may block everywhere. Cancellation is masked while a cleanup runs and takes effect at the first blocking point after it. | Three consequences, all for 15, 14 and 16. First, `exit(code)` is the only call that ignores a blocked cleanup (Q20 and P15-13). Second, a task blocked inside a cleanup counts as blocked for deadlock detection, and the R0008 report says "waiting inside the cleanup of {type}", using the fact key `cleanup_of`. Third, the debugger stop for it is `deadlocked` with the same fact. 16 gets that fact on R0008. |
| Q15 | **A.** Fields and tuple elements drop in declaration order, lists in index order, maps and sets in insertion order. | Say why locals differ: locals drop in reverse because a later local may borrow an earlier one, while fields of one value cannot borrow each other. The spec and the language guide need this one sentence so no reader thinks it is a mistake. |
| Q16 | **B.** When a parent unwinds, its unwaited children are cancelled, then waited. | A child that ends only because it was cancelled is not an error and prints nothing. A child that fails with its own panic or error while unwinding is reported on standard error and counts for the exit status. An explicit `cancel()` keeps the existing §11.3 behavior. 16 needs no new code. |
| Q18 | **C.** Every NaN becomes `0x7FF8000000000000` at `to_bits`, hashing, byte serialization and `Show`. | `Float` is not `Hash` (spec §9.4), so the spec needs nothing here. Any later `OrderedFloat` must hash `-0.0` like `0.0`, because the two compare equal, or `Map` keys would break. Note that for 0.2. `to_bits` still keeps the sign of zero. `Show` prints `NaN` for every NaN and `-0.0` for negative zero. |
| Q19 | **B.** A step is one MIR statement or terminator, counted in tier 0 on unoptimized mono MIR. | 08 §8.5 already says this. A replay always runs at optimization level 0. A native recording is indexed by event ordinal and has no step counter. |

After these rulings, 12 has no `open` question left. A question the owners cannot settle for 0.1 is `deferred`, not `open`.
