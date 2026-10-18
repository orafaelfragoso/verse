# Verse spec — changes for review 1 (Michelangelo)

File revised: `/workspace/verse-spec.md` (HTML and PDF regenerated). Every item
is also logged as a numbered row (R15–R56) in **§21 → "Review 1"**.

## The four extra calls
| Item | Resolution | Where |
|------|------------|-------|
| 1 | A variant name may not collide with a type already in scope; `type Meters = Float` and `type Value = Int \| Text` are errors whose message points to `alias`. | §4.4, §4.8, §17 `type_decl` (semantic note), §18.15, R15 |
| 2 | The right operand of `otherwise` must be a literal (incl. negative number), call, postfix expr, parenthesized expr, or another `otherwise`; `x otherwise 0 + 1` is an error suggesting brackets. | §3.8 table, §3.8.2, §17 `fallback`, §18.4, R16 |
| 3 | `TaskPanicked` is a named type (`std.task`) that `match` can tell apart; the panic message and trace are always printed to stderr at the moment of the panic. | §8.5, §11.2 (example), §15.2, §20.2, §18.13, R17 |
| 4 | `with open("log.txt")? as file`: `?` is required; `with` never propagates; non-Closeable `File \| Error` is a compile error. All `with` examples updated. | §13.1–13.4, §17 `resource`, §18.12 (rewritten), R18 |

## Section 1
| Item | Resolution | Where |
|------|------------|-------|
| §1 punctuation | "one idea each"; every use of `:`, `{ }`, `?` (incl. the new `x?` pattern, labels, format specs) listed. | §1.2, R19 |
| E1 promise | §1.3 now states discarding a fallible result is a compile error, `let _ =` is the escape hatch. | §1.3 |

## Lexical (L1–L8)
| Item | Resolution | Where |
|------|------------|-------|
| L1 | `.with(...)` renamed `.copy(...)`. | §4.3, §18.15, R20 |
| L2 | `set` is a reserved word. | §3.6, §17 `set`, R21 |
| L3 | No SCREAMING_CASE; first character decides: uppercase (Lu/Lt) = type-level, anything else (lowercase, `_`, uncased letters) = value-level. | §3.5, §17 `TYPE_IDENT`/`IDENT`, §12.2 (renames keep kind), R22 |
| L4 | Next line beginning with `\|` continues; `type X =` never opens a field block. | §3.4, §4.4, §17 intro + `type_decl`, R23 |
| L5 | Format colon = first colon outside brackets or nested text. | §3.7.2, §17 `interp`, R24 |
| L6 | `spawn` and `if…then…else` removed from the precedence table; `spawn` is a prefix on a call (grammar: `unary = "spawn" postfix`). | §3.8, §17, R25 |
| L7 | `a..`, `..b`, `..=b` only inside slices; documented; `slice` rule added; `FLOAT` matches `1e-9`. | §3.7, §6.4, §9.1, §17 `index`/`slice`/`FLOAT`, §18.1, R26 |
| L8 | `##` + blank line = module doc; `##` directly above = item doc; `verse fmt` enforces. Tour and §14.3 fixed. | §2, §3.2, §14.3, §17 `file`, §18.15, R27 |

## Types (T1–T3)
| Item | Resolution | Where |
|------|------------|-------|
| T1 | `map[k]` and `list.get(i)` return a real `V?` (`Int??` on `Map<Text, Int?>`); only `?.` chains flatten; `x??` / `none?` patterns shown. | §4.6, §8.1, §9.3, §18.11, R28 |
| T2 | `?`/`otherwise` act on the outermost layer in written order (`Text? \| Error`). | §8.3, R29 |
| T3 | Specific error absorbed by `Error`; `?` cannot narrow back; `.wait()` case worked through. | §4.6, §8.3 table, §11.1, R30 |

## Functions (F1–F2)
| Item | Resolution | Where |
|------|------------|-------|
| F1 | Tuple auto-spread for N-parameter lambdas (exact arity only). | §5.4, §9.2, §18.7, R31 |
| F2 | Punning in construction and calls: `Dog(name:)`, `connect(host:)`. Examples updated (§5.6, §20.2, §20.3). | §4.3, §5.3, §17 `arg`, R32 |

## Control flow (C1–C2)
| Item | Resolution | Where |
|------|------------|-------|
| C1 | `outer: for row in grid`; `break outer` / `continue outer`. | §6.5, §17 `loop_stmt`, §18.8, R33 |
| C2 | `if let` removed; `if x is pattern` (with `user?`). | §6.2, §7.6, §8.1 table, §20.3, §17 `condition` removed, R34 |

## Patterns (P1–P3)
| Item | Resolution | Where |
|------|------------|-------|
| P1 | Inside parentheses `name: pattern` is always a field; typed binding `field: n: Int`; top-level `n: Int` kept. | §7.3, §17 `top_pat`/`sub_pat`/`field_pat`, R35 |
| P2 | `p?` pattern (present optional); `none -> …` / `user? -> …`; `Person(email: e?)`. | §7.3, §7.5, §7.6, §17 `present_pat`, R36 |
| P3 | Unconditional lowercase/`_` arm not last ⇒ later arms are "unreachable arm" errors (example message). | §7.2, R37 |

## Errors (E1–E4)
| Item | Resolution | Where |
|------|------------|-------|
| E1 | Discarding a fallible result (incl. containers, e.g. `wait_all`) is a compile **error**; `let _ =`. §11.4 example now loops over `wait_all`. | §1.3, §8.4, §11.2, §11.4, §17 `statement` note, §18.11, R38 |
| E2 | `?` returns from the innermost lambda / spawned function / test / script (table). | §5.4, §8.3, §11.1, §14.1, R39 |
| E3 | Row added: `?` on `T \| E` in a function returning `U?` is an error. | §8.3 table, R40 |
| E4 | See T2, T3. | — |

## Collections (K1–K2)
| Item | Resolution | Where |
|------|------------|-------|
| K1 | `get(key, default: x)` on maps and lists returns plain `V`; `default:` only on `get`; all `(m[k] otherwise 0) + 1` rewritten. | §3.8.2, §9.1–9.3, §20.1, §18.11, R41 |
| K2 | Side-by-side table of `list[i]` / `m[k]` / `get` / `get(…, default:)`; bare `{}` error. | §3.7, §9.2, §9.3, §17 `map`, R42 |

## Concurrency (N1–N4)
| Item | Resolution | Where |
|------|------------|-------|
| N1 | `Cancelled` unwinds like a panic (runs `with` cleanup), cannot be caught by `otherwise`/`match`/`?` inside the task, reported by `.wait()`; explicit signatures for `sleep`, `receive`, `wait`. | §11.2 table, §11.5, §18.13, R43 |
| N2 | Unwaited failed task ⇒ non-zero exit (1, or 101 for a panic). | §11.3, §12.5, R44 |
| N3 | See item 3. | — |
| N4 | `receive(self): T?`; channel operations table. | §11.4, R45 |

## Modules (M1–M3), tests (X1), stdlib (S1–S2)
| Item | Resolution | Where |
|------|------------|-------|
| M1 | Fields hidden unless `export`; `private` removed from keywords/grammar. | §3.6, §4.3, §12.3, §17 `field`, R46 |
| M2 | Imported-module `let` must be infallible (no `?`, no fallible type, no `let mut`); entry-file `let` is an ordinary statement. | §12.4, §18.14, R47 |
| M3 | `fn main` dropped; top-level statements only. Old §5.6 removed (Methods is now §5.6). | §1.5, §12.5, §18.14, R48 |
| X1 | `verse test` never runs entry-file top-level statements. | §14.1, §20 intro, R49 |
| S1 | `read` removed from prelude; `open`/`create`/`read_text` only in `std.fs` (all examples import them); `print(value: Show = "")` so `print()` prints an empty line. | §3.8.2, §8.2–8.4, §13, §15.1, R50 |
| S2 | §8.2 variant renamed `NoResponse(seconds:)`. | §8.2, R51 |

## Grammar (G1–G9)
| Item | Resolution | Where |
|------|------------|-------|
| G1 | `top_pat = ( "_" \| IDENT ) ":" type`. | §17, R52 |
| G2 | `place = ( IDENT \| "self" ) …`. | §17, §5.6, R53 |
| G3 | See P1. | §17 |
| G4 | See L6. | §17 |
| G5 | See L3 and L7. | §17 |
| G6 | `from m import x as y` (`import_item`). | §12.2, §17, R54 |
| G7 | See item 1. | §17 |
| G8 | See L1 (`.with` gone). | — |
| G9 | `present_pat = single_pat { "?" }`. | §17, R36 |

## Other requested fixes
| Item | Resolution | Where |
|------|------------|-------|
| §10.4 | Rows for `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`, with an example; §18.5 now points to them. | §10.4, §15.1, §18.5, R55 |
| §19 | "fewer symbols than any of them" softened. | §19, R55 |
| ToC | §21 added. | Contents, R55 |
| §18 | 18.8, 18.11, 18.12 rewritten; 18.1, 18.4, 18.5, 18.7, 18.13–18.15 updated. | §18 |
| Addendum | Method calls are unmarked; a `mut self` method's receiver must be a `let mut` binding (compile error otherwise). | §5.6, R56 |
