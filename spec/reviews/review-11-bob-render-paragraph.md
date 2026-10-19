# Review 11: paragraph for 08 §8.4, "Rendered values" (Bob)

For Michelangelo to place at the end of §8.4 (value shapes). The decision is
[18 §18.12](../architecture/18-plugin-interface.md).

---

### Rendered values

A `tool` plugin that declares `renders = ["TypeName"]` in `plugin.toml` can
supply a short description of values of that type. The description is added to
the value record next to the built-in one. It never replaces it.

```json
{
  "name": "scores",
  "type": "Histogram",
  "state": "owned",
  "summary": "Histogram(buckets: 12, total: 340)",
  "rendered": "340 samples in 12 buckets, most in 40 to 50",
  "rendered_by": "stats-view"
}
```

- **Where it runs.** Only inside the debugger (`verse mcp`). It never runs in
  the program and cannot change what the program does, so script and compile
  modes stay identical.
- **Input.** The value record of this section, cut to the depth and size that
  the debug call asked for. It has no addresses and no pointers.
- **Output.** One string of at most 25 words, plain and in the present tense.
  The host checks it against the "don't say" table of §8.7. A string that fails
  the check or is too long is dropped, the built-in `summary` is used, and the
  user sees one warning for that type.
- **Budget.** 1 million fuel units, 50 ms and 16 MiB per call. A call that stops
  falls back to the built-in `summary`.
- **No capabilities.** A renderer can read no file and no input.
- **Recordings.** Record and replay stores raw values only, so a recording does
  not depend on any renderer. Rendering happens when a value is asked for.
- **Both lines are always present.** `summary` is always there, so a client that
  does not know about renderers still reads a full description.

Fields added to the value record: `rendered` (text, optional) and
`rendered_by` (the plugin name, present exactly when `rendered` is).
