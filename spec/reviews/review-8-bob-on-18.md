# Review 8: architecture 18 (plugin interface), checked against the host design (Bob)

Verdict: **no blockers**. Two changes requested in 18.10, one decision for 18.11 point 2.

## 18.10 limits against the host design

The host runs plugins with Wasmtime ([ADR-0005](../architecture/adr/0005-wasm-component-plugins.md)).
Numbers are defaults to confirm in M5, not measurements.

| Limit | Verdict | Note |
|-------|---------|------|
| fuel, 100 million | keep | Wasmtime fuel is counted in roughly one unit per Wasm instruction. Say "fuel units (about one per instruction)", not "instructions", so the text stays true if the engine changes. Fuel is **deterministic**, so it is the primary limit. |
| time, 10 s | keep, but change what it means (R8-1) | Time depends on the machine. |
| memory, 128 MiB | keep | Wasm memory is limited in 64 KiB pages, so the limit is exact and deterministic. The host may run plugins in parallel, so total use can be several times 128 MiB; the host runs at most as many plugins at once as `verse build` has jobs. |
| output, 4 MiB and 256 files | keep | Checked by the host as it receives `emit`, so a plugin cannot exceed it by buffering. |
| diagnostics, 200 | keep | |

**R8-1. Build determinism and the time limit.** The language promise is that the
same inputs and the same plugin versions give the same bytes (spec §16.3,
[ADR-0004](../architecture/adr/0004-whole-program-reachability.md)). A plugin
stopped by the clock on a slow machine and finished on a fast one would break
that promise. So: **fuel and memory stops are part of the result** and are
cached like any other result; a **time stop is a host failure, not a result**.
It is reported as an error that says "this limit depends on the machine; raise
`time` or lower the work", and it is never cached.

**R8-2. A stopped plugin and the build.** 18.10 says a stopped plugin "never
fails the build by itself". That is right for a `lint` (a warning is enough). For
a `generator` or `bindgen` it hides the real problem, because the files the
program imports are missing and the user then sees a confusing "unknown name".
Proposal: a stopped `lint` is a warning; a stopped `generator`, `bindgen` or
`tool` that the build needs is an error with the limit's name. This is a
catalog entry for 16 (suggest V0818, "`{plugin}` ran out of {limit}").

**R8-3. Cache key.** Plugin results are cached by the hash of the plugin file, the
interface version, the granted inputs and the limits, so changing a limit
invalidates the cache. Add one sentence to 18.10.

## 18.11 point 2: renderers for `tool` plugins (decision)

A renderer is a small, pure function that runs only inside the debugger host
(`verse mcp`). It never runs in the program and never changes how the program
behaves, so script and compile modes stay identical.

| Part | Decision |
|------|----------|
| Entry point | `render(value)`, listed in 18.3 as a fifth entry point for kind `tool` with `renders = ["TypeName", ...]` in `plugin.toml` |
| Input | the **value record of [08 §8.4](../architecture/08-debugging-and-mcp.md)**: type name, state (`owned`, `moved`, `borrowed`, `exclusive`, `view`), presence, and fields, cut to the depth and size the debug call asked for. No addresses, no host pointers. |
| Output | one string of at most 25 words, plain, present tense, in the words of 08 §8.7 |
| Check | the host runs the output through the "don't say" table of §8.7. A string that fails, or is longer than 25 words, is dropped and the built-in summary is used. The user sees a warning once per type |
| Both always shown | the record carries `summary` (built in) and `rendered` (plugin text) with `rendered_by`, so an LLM can always read the built-in line |
| Limits | 1 million fuel units, 50 ms, 16 MiB **per call**; a stop falls back to the built-in summary |
| Replay | record and replay stores raw values only. Rendering happens on request, so a plugin changing never invalidates a recording |
| Capabilities | none. A renderer asks for no inputs and no files |

Why this shape: it keeps the debugger's vocabulary rule enforced by the host
instead of by trust, keeps traces independent of plugins, and gives small
budgets so a slow renderer cannot slow stepping.

## Owners

Michelangelo owns 18, 16 and 08 and applies these. I can write the
`render` paragraph for 08 §8.4 if he prefers.

## Response (Michelangelo)

All three changes and the renderer decision are applied in 18.10 and a new 18.12 (the renderer rules, with the `render` entry point in 18.3). Codes: V0818 was already taken by "Wasm feature not supported" (review 7, R7-5). Stopped plugins are V0819 (trap), V0820 (fuel, memory or output) and V0821 (rejected output), a warning for a `lint` and an error for the other kinds; the clock stop is V0822, an error, never cached. Accepted: please write the 08 §8.4 paragraph for `summary`, `rendered` and `rendered_by`; I will add the fields to the `value` type in 17 §17.2 once it is in.
