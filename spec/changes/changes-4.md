# Changes 4 — Review 3 (Michelangelo)

Every item of `reviews/review-3.md` was applied as decided. Line numbers are
in the files after the change. `00-README.md` was not edited (section status
is set by the reviewer).

| Item | Resolution | File: lines |
|------|------------|-------------|
| R3-1 `otherwise` table | New fourth table row (owned left + non-static view right → copy the right side); error sentence covers both copy rows; literal materialisation as the one allowed implicit creation; `?.` on an owned temporary yields an owned field (`find_user(id)?.email otherwise "no email"`); `if`/`match` branches follow the same rule | `03-lexical.md`: 350 (new row), 352–368 (rewritten paragraph + two new paragraphs) |
| R3-1 pointer in §6 | Branch rule for `if` (both forms) and `match` values, pointing to §3.8.2 | `06-control-flow.md`: 42–45 (§6.1) |
| R3-1 pointer in §8.3 | Pointer to the full table in §3.8.2 | `08-errors.md`: 194–197 |
| R3-2 `get(k, default: x)` | Same table as `otherwise`; owned temporary default (`Text.new()`) makes the result owned and the element side needs `.copy()`; static literal default stays a view | `09-collections.md`: 75–85 (§9.3) |
| R3-3 owned source | Single element moves out of an owned source at its last use; the rest is freed | `09-collections.md`: 191–194 (§9.5); `20-examples.md`: 243–244 (§20.3 comment) |
| R3-4 collecting steps | Table rows marked; new owning-step bullet; `chunks`/`windows` as views of slices | `09-collections.md`: 148, 151, 153 (table), 174–179 (owning-step list) |
| R3-5 handles | `Copyable` row: handle `.copy()` copies the handle | `10-traits-generics.md`: 79 (§10.4) |
| R3-5 handles | §11.1 bullet: explicit `.copy()` to pass a handle to `spawn`; no implicit copy | `11-concurrency.md`: 67–71 |
| R3-5 handles | §11.4: handle `.copy()` defined; examples now copy explicitly (channel worker, `dictionary`, `hits`) | `11-concurrency.md`: 149–153, 171, 198–199, 216 |
| R3-5 handles | §15.2 `std.task` row made consistent | `15-stdlib.md`: 41 |
| Log | Review 3 table | `21-review-log.md`: 182–196 |
| R3-6 handles always `Copyable` | §10.4 `Copyable` row: exclusion limited to containment by value; all four handles `Copyable` whatever `T`, including `Guarded<File>` | `10-traits-generics.md`: 79; `11-concurrency.md`: 149–152 (§11.4 paragraph) |
| R3-7 `Weak<T>` | Listed with the other handles; `.copy()` required; table column filled in | `11-concurrency.md`: 67–68 (§11.1 bullet), 146 (§11.4 table), 149–152; `15-stdlib.md`: 41; `10-traits-generics.md`: 79 |
| R3-8 explicit `.copy()` | §4.10 point 5 clause | `04-types.md`: 469–470 |
| R3-9 owned element + owned default | Confirmed form; `.copy()` on `V?` yields an owned `V?` | `09-collections.md`: 84–89 (§9.3) |
| Log | Rows R3-6…R3-9 appended to the Review 3 table | `21-review-log.md`: 196–199 |
| R3-10 `Weak` over the handle type | `Weak<T>` → `Weak<Shared<T>>` / `Weak<Guarded<T>>`; creation with `h.downgrade()` | `04-types.md`: 468–469, 480 (§4.10); `10-traits-generics.md`: 79 (§10.4); `11-concurrency.md`: 68 (§11.1), 146 (§11.4 table); `18-design-notes.md`: 143 (§18.10) |
| R3-11 `upgrade()` type | Returns `Shared<T>?` or `Guarded<T>?` | `11-concurrency.md`: 146 (§11.4 table) |
| Log | Rows R3-10, R3-11 appended to the Review 3 table | `21-review-log.md`: 200–201 |
| R3-12 `Weak<Channel<T>>` | One sentence in §11.4: `Weak` takes only `Shared`/`Guarded`; `Weak<Channel<T>>` is a compile error | `11-concurrency.md`: 153–155 (§11.4) |
| Log | Row R3-12 appended to the Review 3 table | `21-review-log.md`: 202 |
