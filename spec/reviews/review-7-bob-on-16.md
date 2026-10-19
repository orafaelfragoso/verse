# Review 7: architecture 16 (diagnostics catalog), checked against the compiler passes (Bob)

Scope: does every pass in [02 §2.2](../architecture/02-compiler-architecture.md),
[03](../architecture/03-ir-and-passes.md) and [04](../architecture/04-backends-and-targets.md)
have a code in 16 for each way it can fail on valid input to the compiler? Wording
and style are Michelangelo's and are not questioned here. Verdict: **no blockers**.
The catalog covers the lexer, parser, name resolution, inference and ownership
well. Eight failure modes in the later stages have no entry.

## Stage by stage

| Stage | Errors it owns (02 §2.2) | Covered in 16 |
|-------|--------------------------|---------------|
| Lexer | bad indentation, unterminated text, invalid escapes | V0101 to V0108 (gap: invalid UTF-8, R7-1) |
| Parser | syntax errors | V02xx |
| Name resolution | unknown name, hidden name, cyclic import, visibility | V03xx |
| Type inference | type errors, missing annotations, optional misuse | V04xx (see R7-6) |
| Ownership | use after move, borrow rules, `Copy` bounds | V05xx |
| Monomorphization | none in 02, but see R7-2 | gap |
| Backends | target limits | V0902, V0903 (gaps R7-3, R7-4) |
| Wasm frontend (07 §7.4) | rejected Wasm features | gap, R7-5 |
| Plugin host (07) | plugin failures at build time | gap, R7-7 |

## Requested entries

| # | Needed code | Where it comes from | Suggested family and wording |
|---|-------------|---------------------|------------------------------|
| R7-1 | A source file that is not valid UTF-8 | Lexer input invariant, 02 §2.2 | V01xx: "this file is not valid UTF-8", label "the first bad byte is here", help "save the file as UTF-8". |
| R7-2 | Monomorphization does not terminate: an unannotated generic function whose instances keep growing (for example a function that calls itself with a wrapped copy of its argument), or an instance count over the build limit | 02 §2.6, instance key `(DefId, [TypeId])` | V04xx: "`{f}` needs a new copy for every depth of this call", label at the call, help "annotate the parameter with one concrete type". The limit is a compiler setting, default to be set in M2. |
| R7-3 | The `abort` profile with a `Cleanup` value live at a panic site | Q5 ruling: the build fails | V09xx: "this panic would skip the cleanup of `{x}`", label "`{x}` is alive here", help "turn the `abort` profile off, or move the panic site after the cleanup". |
| R7-4 | A function that waits and calls itself, built for a target whose tasks are state machines (Wasm, microcontrollers) | 04 task lowering, 03 pass 14 `task-lower`: the compiler needs a bounded frame | V09xx: "`{f}` waits and calls itself, which `{target}` cannot run", label at the wait, help "build for a native target, or make the recursive part not wait". |
| R7-5 | A Wasm-implemented package uses a Wasm feature the frontend cannot translate | 07 §7.4: rejected at build time naming the feature | V08xx: "`{pkg}` uses {feature}, which Verse cannot build in", label "first used in `{export}`", help "use a version of `{pkg}` without it". |
| R7-6 | Recursion that goes through another function (`f` calls `g`, `g` calls `f`) | 02 §2.5: "every cycle has an annotated entry" | Not a new code: V0403 says "`{f}` calls itself". Add a second message form "`{f}` calls itself through `{g}`" and make the label point at the whole cycle. |
| R7-7 | Plugin host failures: the plugin trapped, ran out of its time or memory budget, returned output that does not follow its schema, or asked for a capability it was not granted | 07 §7.5 and §7.6 | A V08xx family or its own `P` family (your choice): one code per failure, with the plugin name and the grant that would allow it. 16.4 covers diagnostics a plugin writes, not failures of the plugin itself. |
| R7-8 | A generic function called with a type that lacks what its body uses (field, method, trait) | 02 §2.5 constraints from the body | V0406 and V0407 already exist. Add to their label: "needed by `{f}`, called here", so the user sees the call and the line inside `{f}`. |

## Runtime panics are not in the catalog

Overflow, index out of range, division by zero, a failed `Error`-to-panic
conversion and stack overflow are not compiler diagnostics, but `why_panic`
([08](../architecture/08-debugging-and-mcp.md)) and a plain run both print their
wording. They need a fixed list of messages and stop reasons (one place, with the
same rule 7: same fact, same words). Proposal: a short section 16.7 "Panic
messages" with a `P`-less code such as `R0001` (run-time family `R0xxx`), so the VM
and the compiled tiers print identical text, which the differential tests check
([10](../architecture/10-testing-and-verification.md)).

## Candidates to remove

None. Every entry in V01xx to V07xx has a pass that can produce it. V0511
(chain ownership note) comes from the ownership pass facts of 02 §2.4. V9902
(verifier failure) is the MIR verifier of 13.

## Machine-readable facts for the catalog

Every compiler diagnostic is one `Diagnostic` record (02 §2.8): `code`, `severity`,
`message`, `labels`, `help`, `rule`, `facts`. The catalog's `{…}` placeholders must
be exactly the names of the `facts` keys, so a test can check that every placeholder
is filled. Suggest one table in 16.1: placeholder name, fact key, type.

## Owners

Michelangelo owns 16 and decides each request. I can supply the facts keys for
R7-2 to R7-5 when he assigns the codes.
