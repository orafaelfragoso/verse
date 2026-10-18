# Review 4 (Michelangelo) — full read-through as a reader

Read front to back after RF-1..RF-25 (RF-26..31 still landing). Focus: beauty, ergonomics, contradictions.
RF-1..RF-25 themselves are accepted (imports, `let` removal, lifted chains, Sender/Receiver split all landed correctly).
Blockers must be settled before the sections they touch go back to *agreed*.

## Blockers

| # | Where | Problem | Proposal |
|---|-------|---------|----------|
| R4-1 | §3.9, §12.2, §15.1, §15.2 | "You cannot bind a name that is currently visible" is not scoped. Visible could include the prelude (`input`, `error`, `env`, `args`, `exit`), imports (`fs`, `time`, `e`, `pi`) and module constants. Then `input = read_line()` or `error = e.message()` in a function would be an error, and `std.log` exports `error`, which collides with the prelude `error`. | Scope the rule to **locals only**: a name can be bound once per function (parameters, lambda parameters, loop variables, pattern bindings, `with … as`, plain bindings). A local may **hide** a prelude, imported or module-level name, with a warning "hides the import `fs`" for imports and prelude names. Table row 1 of §3.9 already says "not yet bound in this function"; make §3.9's prose say the same. Rename or alias `std.log.error`. |
| R4-2 | §3.8 row 1, §8.1, §18 | `a?.b` is legal and means "propagate, then read". Swift, Kotlin, JS and Rafael's old spec all read `?.` as safe access. In a function returning an optional, `city = order?.customer?.address?.city` compiles and silently **returns early** instead of leaving a local `none`. | Make `?` immediately followed by `.` a compile error: "`?.` is not Verse. To propagate then read, write `(a?).b`; to look through an optional, write `a.b` and consume the chain with `otherwise`, `?` or `is`". One spelling per meaning (§1.4). |
| R4-3 | §4.2 rule 9, §5.3 | `fn age_of(person)` / `person.age` is an error, because the field type is unknown and treated as non-`Copy`. The simplest getter needs an annotation or `.copy()`, which contradicts "types are visually optional" and the tour. | Do not error. An unannotated function that returns a borrowed field, element or lookup gets an **inferred `Copy` bound** on that type, recorded in `verse.api` like any other constraint. A call with a non-`Copy` type is a call-site error naming the bound: "`first_name` returns `person.name` by value, so the field must be `Copy`; use `.copy()` inside, or annotate". Ownership is still checked once at the definition. |
| R4-4 | §15.2 (`std.json` example) | `json.parse("""{"id": 1, "title": "Pen"}""")` is a non-raw triple-quoted literal, so `{"id": 1 …}` is read as interpolation. | Use `r"""…"""`. Add a one-line lint for a `{` that cannot parse as an interpolation. |
| R4-5 | §7.7, §3.9 | Destructuring on the left of `=` has no rule when a name already exists: `(x, y) = position` with `mut x` visible is a new binding, a reassignment, or an error? There is also no swap: `(a, b) = (b, a)`. | Patterns **always create new bindings**, and a visible name is the usual error. Add one exception: a tuple of names that are *all* visible `mut` bindings is a parallel assignment, so `(a, b) = (b, a)` swaps. Mixing new and existing names is an error. |

## Polish (non-blocking, but please fix in the same pass)

| # | Where | Note |
|---|-------|------|
| R4-6 | §2, §3.7.2, §6.1 | Text literals nested in interpolation read badly: `"{person.email otherwise "no email"}"` and `{if count == 1 then "item" else "items"}`. Keep the grammar. Add a `verse fmt --check` hint that suggests binding a name first, and prefer the bound form in the tour. |
| R4-7 | §4.9 | "adding a field can never silently turn a copy into a move" is false: adding a `Text` field to an all-`Float` record does exactly that. Say instead "there is no size limit; only the kinds of the fields matter". |
| R4-8 | §3.7 | `10.seconds()`: say that a digit sequence followed by `.` and a letter is an `Int` and a method call (a float needs digits after the dot). |
| R4-9 | §8.1 | "the three ways out" lists four. `.or_error()` is used in §8.1 and §8.3 but is not in the §8.1 tool table. |
| R4-10 | §3.8 table row 1 | One cell carries three paragraphs of rule. Move the `?` and lifting prose below the table. |
| R4-11 | §9.2 | `stock["apples"] = stock.get("apples", default: 0) + 5` is the most repeated line in the spec. Optional: `Map.with_default(0)` so `counts[word] += 1` works (Python's `defaultdict`), reading better than the `get` idiom. |
| R4-12 | §7.3, §8.1 | `Reply(body: b: Text)`, `none?` and `score??` are hard to read, and so is `x otherwise none otherwise "anonymous"`. Add a style note: prefer `match` with named arms for nested optionals. |
| R4-13 | §11.4 | `start_workers` must end in `.to_list()` (RF-27 is pending). Check after it lands. |

## Section status after this review

Reopened: 3 (R4-1, R4-2, R4-5, R4-8), 4 (R4-3, R4-7), 5 (R4-3), 7 (R4-5), 8 (R4-2, R4-9), 15 (R4-1, R4-4), 12 (R4-1).
All other sections stay agreed.

## Re-verification (after changes-6)

Verified as landed and correct: R4-1 (§3.9), R4-2 (§3.8, §8.1, grammar, §16.1, §18.11), R4-3 (§4.2 r9, §5.3, §18.10), R4-4 (§15.2, §3.7.3), R4-5 (§3.9, §7.7, tour), R4-6 to R4-10, R4-12, R4-13, R4-14.
Set back to **agreed**: §3, §5, §7, §8.

New findings (all small):

| # | Where | Note |
|---|-------|------|
| R5-1 | §4.2 rule 9 example | The error help says "annotate the parameter". Annotating `person: Person` makes `person.name` an error at the definition (§5.3), so it is not a fix. Replace with "return `person.name.copy()`, or take the parameter with `take`". |
| R5-2 | §16.1 | The list sentence has two "and"s ("…, and sorts imports, enforces the blank line…, and drops…"). Reword as one list. |
| R5-3 | §3.9 | A file-level `import … error from std.log` silently changes what `error(…)` means for the whole file. Add a warning on the import line: "`error` hides the prelude `error`". |
| R5-4 | §12.2, §15.1 | Readers look for the hiding rule where imports and the prelude live. Add a one-line pointer to §3.9 in each. |
| R5-5 | §18.9 | The summary does not mention that only locals count, nor the swap exception. Add one clause for each. |

§4, §12 and §15 stay open until R5-1, R5-3 and R5-4 land. §10, §13, §14, §16, §18, §19 and the §20 examples are otherwise accepted.

Final: R5-1 to R5-5 verified in text. §4, §12, §15 set to agreed. All 21 sections agreed; nothing open.
