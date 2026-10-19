# Review 9: `facts` keys for four diagnostics (Bob)

The `facts` record of a diagnostic ([02 §2.8](../architecture/02-compiler-architecture.md))
holds typed values by name. A placeholder `{x}` in 16 is filled from the key `x`.
Types are the ones of the diagnostic record: `Name`, `Type`, `Span`, `Count`,
`Text`, `Target`, `Capability`.

| Code | Key | Type | Meaning |
|------|-----|------|---------|
| V0419 | `f` | Name | the generic function that keeps growing |
| V0419 | `T` | Type | the argument type at the first call of the cycle |
| V0419 | `W` | Type | the wrapper that is added each time (for example `List<T>`) |
| V0419 | `cycle` | List of Span | the calls that form the cycle, shown as extra labels |
| V0419 | `depth` | Count | the depth at which the compiler stopped (the instance limit, default set in M2) |
| V0818 | `pkg` | Name | the Wasm-implemented package |
| V0818 | `feature` | Text | the Wasm feature, in the Wasm specification's own name |
| V0818 | `export` | Name | the first exported item that needs it |
| V0908 | `x` | Name | the binding whose cleanup would be skipped |
| V0908 | `panic_site` | Span | the panic site |
| V0908 | `cleanup_type` | Type | the `Cleanup` type of `x` |
| V0909 | `f` | Name | the function that waits and calls itself |
| V0909 | `target` | Target | the target whose tasks are state machines |
| V0909 | `wait` | Span | the wait inside `f` |
| V0909 | `cycle` | List of Span | the recursive calls, extra labels |

For each, the build check of 16.1 rule 9 fails if a placeholder in the message,
label or help has no key here, or a key here is never used in text. Keys the text
does not show (`depth`, `cycle`, `panic_site`, `cleanup_type`) still travel in
`check --json` for tools.

Note: V0818 in the current 16 is the Wasm-feature entry. My earlier request for
"`{plugin}` ran out of {limit}" (review 8) is one of V0819 to V0821; the keys for
those are `plugin` (Name), `limit` (Text: fuel, time, memory, output),
`value` (Text: the limit's size), and `kind` (Text: lint, generator, bindgen, tool).
