# Review of `extern-semantics.md` (Michelangelo)

Verdict: **accepted for 0.2 with the four rulings below.** E1 to E11 meet C1 to C7.

## Rulings on the open points

1. **Open point 1, how a `with` block holds a capability.** Use the existing
   resource form (§13): the block binds a value and the binding is used. In the
   sketch, `pins` is bound and never used, which would trip the unused-binding
   rule and hides why the block is there. Pass the bound value as the first
   argument of a foreign call, or make the capability a resource that the
   declared function takes by name. A reader then sees where the capability
   goes. Spelling is left to the 0.2 section; the rule is "the bound name must
   appear in the calls it permits".
2. **Open point 2, pure libraries.** Allow a `pure` marker in `[library.native]`.
   A call to `libm` that needs `with` is ceremony with no information. Pure
   libraries have no capability, so there is nothing to name.
3. **Open point 3, `Text` across the boundary.** No. Keep `text.bytes()`.
   One spelling, and the cost of the conversion stays visible.
4. **Open point 4, Wasm component imports.** Same rules, as the second foreign
   kind. Agreed.

## Wording to add

- E10: `from_foreign` is a new debugger note. It needs an entry in
  `vocab/events.toml`, or the build fails (08 §8.7). Suggested wording:
  "came from a foreign call".
- E3: the error names the capability and shows the `with` line to add. Keep
  that exact shape: "calling `write_pin` needs the `gpio` capability; wrap
  the call in `with gpio() as …`".
- One word for the idea: **capability** names what a program or plugin needs;
  **grant** is the verb for giving it (`verse.toml`, `--deny`). Do not use
  "permission" or "privilege" anywhere.

## Picasso's three decisions

- `text.bytes()[i]` is an `Int` from 0 to 255: accepted.
- `Map` and `Set` keep a key's place on replace and move it to the end on
  remove and re-insert: accepted. It is the order a reader expects, and it
  needs one sentence in §9.4 that says so.
- Define "capability" in the 0.2 `extern` item: agreed, in one sentence, in
  plain words.
