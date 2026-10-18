# Review 3 (Michelangelo)

Re-review of the changed lines from `changes/changes-3.md`. All of review 2 landed
correctly. Rulings N-1..N-5: N-3, N-4, N-5 agreed as stated; N-1 and N-2 agreed with the amendments below.

Agreed now: 1, 2, 4, 5, 6, 7, 8, 12, 13, 14, 16, 17, 18, 19, 21.
Open (pending R3 items): 3 (R3-1), 9 (R3-2, R3-3, R3-4), 10, 11, 15 (R3-5), 20 (R3-3).

| Item | Where | Request |
|------|-------|---------|
| R3-1 | §3.8.2 table | Add the two missing rows: owned left + non-static view right is an error asking for `.copy()` on the right; and say a literal in an owned position is materialised (the one allowed implicit creation, since it duplicates no user value). State that `?.` on an owned temporary yields an owned field (line `find_user(id)?.email otherwise "no email"`). Give `if c then a else b` and `match` arms the same rule (view only if all branches are views). |
| R3-2 | §9.2 `get(k, default: x)` | Same table as `otherwise`: a view only if both the element and the default are views or static; an owned temporary default (`Text.new()`) makes the result owned and the element side needs `.copy()`. Otherwise the view of the default dangles. |
| R3-3 | §9.5, §20.3 | `first()`/`max()`/etc. are views of a *borrowed* source. On an owned source at its last use (`take items`, or a temporary), the single element moves out and the rest is freed. Without this sentence `largest` in §20.3 returns a view, contradicting the single-element rule. |
| R3-4 | §9.5 table | Classify `to_list`, `to_set`, `to_map`, `flat_map`, `flatten`, `chunks`, `windows`: owning steps (move at last use, else `.copy()`) except `chunks`/`windows`, which are views of slices (K-4). |
| R3-5 | §10.4, §11.1 | `Shared`, `Guarded`, `Channel` are `Copyable`, but `.copy()` on a handle must be defined: it copies the handle, not the value. Say whether passing a handle to `spawn` is implicit (§11.1 "copies cheaply") or needs `.copy()`. Recommended: explicit `.copy()` everywhere, for consistency with the rest of the spec. |

## Round 3 close-out

R3-1..R3-12 verified against the built text (§3.8.2, §9.2, §9.3, §9.5, §10.4, §11.1, §11.4, §15.1, §18.10, §20.3). Rulings N-1..N-5 and R3-6..R3-12 agreed.
Sections 3, 9, 10, 11, 15 and 20 are now **agreed**. All 21 sections are agreed; nothing is open.
