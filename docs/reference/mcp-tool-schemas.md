# 17. MCP tool schemas

Status: **draft**.
This file gives the exact JSON shapes behind the tools named in
[08](../architecture/debugging-and-mcp.md) §8.3 and §8.4, and points to the vocabulary file
[`events.toml`](events.toml). If this file and 08 disagree on a
name, the vocabulary file wins, then 08 §8.7, then this file.

## 17.1 Rules for every tool

1. **Names.** Tools are verbs (`debug_step`) or questions (`why_moved`). Fields
   are `snake_case` nouns. Event and stop names come from `events.toml`.
2. **One response shape.** Every response is an object with `ok`, a payload
   named after the tool family, and `next`. A failure has `ok: false` and an
   `error` (17.2), never prose alone.
3. **`next` is always present.** It lists at most five tool names that make
   sense now, most useful first. It is never empty after a stop.
4. **Positions** are always `{"file", "line", "column"}`; a span adds
   `"end_line"` and `"end_column"`. Lines and columns start at 1.
5. **Nothing is cut silently.** A response that was shortened carries
   `"cut": {"shown": 20, "total": 312, "more": "inspect_locals page=2"}`. The
   `more` field is a call the client can make.
6. **Unknown is `unknown`.** A field the runtime did not record is the string
   `"unknown"`, never a guess and never `null`. A missing optional input is
   omitted, never `null`.
7. **Budgets.** A value is cut after 200 characters, a list after 20 elements,
   a response after 4,000 characters. A client may raise a budget with
   `budget` on `inspect_value`; the server may refuse above 20,000 characters
   with error `D0109`.
8. **Idempotence.** Every tool except `debug_launch`, `debug_continue`,
   `debug_step`, `debug_back` and `debug_stop` is read-only and safe to repeat.
9. **Session.** Every tool except `debug_launch` takes `session` (a string such
   as `"s1"`). Ids (`v481`, `t3`, `b2`) are stable inside a session only.

## 17.2 Shared types

Written as JSON Schema fragments. `$defs` are referred to by name below.

```json
{
  "$defs": {
    "position": {
      "type": "object",
      "required": ["file", "line"],
      "properties": {
        "file": {"type": "string"},
        "line": {"type": "integer", "minimum": 1},
        "column": {"type": "integer", "minimum": 1}
      }
    },
    "cut": {
      "type": "object",
      "required": ["shown", "total", "more"],
      "properties": {
        "shown": {"type": "integer"},
        "total": {"type": "integer"},
        "more": {"type": "string"}
      }
    },
    "next": {
      "type": "array", "maxItems": 5,
      "items": {"type": "string"}
    },
    "error": {
      "type": "object",
      "required": ["code", "message"],
      "properties": {
        "code": {"type": "string", "pattern": "^D0[0-9]{3}$"},
        "message": {"type": "string"},
        "next": {"$ref": "#/$defs/next"}
      }
    },
    "stop": {
      "type": "object",
      "required": ["session", "step", "reason", "task", "call", "summary", "next"],
      "properties": {
        "session": {"type": "string"},
        "step": {"type": "integer"},
        "reason": {"enum": ["breakpoint", "step_done", "panicked", "assert_failed",
                            "propagated", "moved", "reassigned", "watch_changed",
                            "deadlocked", "exited", "paused"]},
        "task": {"type": "string"},
        "call": {
          "type": "object",
          "required": ["function", "file", "line"],
          "properties": {
            "function": {"type": "string"},
            "file": {"type": "string"},
            "line": {"type": "integer"},
            "column": {"type": "integer"}
          }
        },
        "summary": {"type": "string", "description": "one sentence, at most 25 words"},
        "source": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["line", "text"],
            "properties": {
              "line": {"type": "integer"},
              "text": {"type": "string"},
              "current": {"type": "boolean"}
            }
          }
        },
        "changed_since_last_stop": {"type": "array", "items": {"type": "string"}},
        "status": {"type": "integer", "description": "only when reason is exited"},
        "next": {"$ref": "#/$defs/next"}
      }
    },
    "value": {
      "type": "object",
      "required": ["id", "type", "state", "summary"],
      "dependentRequired": {"rendered": ["rendered_by"], "rendered_by": ["rendered"]},
      "properties": {
        "id": {"type": "string"},
        "name": {"type": "string"},
        "type": {"type": "string"},
        "state": {"enum": ["owned", "moved", "view", "borrowed", "exclusive",
                           "dropped", "not_yet_bound"]},
        "kind": {"enum": ["scalar", "text", "record", "variant", "list", "map",
                          "set", "tuple", "task", "channel", "function", "shared"]},
        "summary": {"type": "string"},
        "mutable": {"type": "boolean"},
        "presence": {"enum": ["present", "none"], "description": "optionals only"},
        "result": {"enum": ["ok", "error"], "description": "fallible values only"},
        "moved_at": {"$ref": "#/$defs/position"},
        "moved_to": {"type": "string"},
        "of": {"type": "string"},
        "until": {"$ref": "#/$defs/position"},
        "reassigned_at": {"$ref": "#/$defs/position"},
        "note": {"enum": ["from_foreign"]},
        "rendered": {"type": "string", "description": "plugin text, at most 25 words; summary is always present too"},
        "rendered_by": {"type": "string", "description": "the plugin name; present exactly when rendered is"},
        "fields": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["name"],
            "properties": {
              "name": {"type": "string"},
              "ref": {"type": "string", "description": "id of a value to expand"},
              "value": {"description": "a scalar shown inline"}
            }
          }
        },
        "truncated": {"type": "boolean"},
        "continue": {"type": "string", "description": "token for the rest, when truncated"}
      }
    },
    "event": {
      "type": "object",
      "required": ["step", "task", "event", "text"],
      "properties": {
        "step": {"type": "integer"},
        "task": {"type": "string"},
        "event": {"type": "string", "description": "a key of events in events.toml"},
        "text": {"type": "string", "description": "the template, filled in"},
        "at": {"$ref": "#/$defs/position"}
      },
      "additionalProperties": true,
      "description": "plus the fields that events.toml lists for this event"
    }
  }
}
```

The `error` codes used by the debugger are listed in `events.toml` (`D0101`
to `D0108`). Two more are defined here: `D0109` (a budget above the allowed
maximum) and `D0110` (the request is valid but the session is in the wrong
state, for example `debug_continue` while the program is running).

## 17.3 The core tools

Each entry shows the input, then the output. The blocks in this section are
annotated sketches (a field's value is its type, its allowed values or an
example), not strict JSON Schema; only the `$defs` block in 17.2 is. A strict
schema file is generated from them in the first implementation milestone. Unless noted, the output is
`{"ok": true, "stop": <stop>}` for tools that move the program and
`{"ok": true, ..., "next": [...]}` for the others.

### `debug_launch`

```json
{
  "input": {
    "type": "object", "required": ["file"],
    "properties": {
      "file": {"type": "string"},
      "args": {"type": "array", "items": {"type": "string"}},
      "record": {"type": "boolean", "default": false},
      "seed": {"type": "integer", "description": "fixed scheduler seed"},
      "deny": {"type": "array", "items": {"type": "string"},
               "description": "capabilities the program may not use (§5.6)"}
    }
  },
  "output": {"ok": true, "session": "s1", "stop": {"reason": "paused", "...": "first stop, before the first statement"}}
}
```

### `debug_continue`, `debug_step`, `debug_back`

```json
{
  "debug_continue": {"input": {"session": "s1"}},
  "debug_step": {
    "input": {
      "session": "s1",
      "mode": {"enum": ["into", "over", "out", "line", "task"], "default": "over"},
      "count": {"type": "integer", "minimum": 1, "default": 1}
    }
  },
  "debug_back": {
    "input": {"session": "s1", "mode": "as debug_step", "count": "as debug_step"},
    "note": "needs a recorded session, else D0103"
  }
}
```

All three return a `stop`. A `debug_step` that ends the program returns the
stop with `reason: "exited"` and `status`.

### `debug_break`

```json
{
  "input": {
    "session": "s1",
    "action": {"enum": ["set", "list", "clear"], "default": "set"},
    "at": {"type": "string", "description": "file:line"},
    "function": {"type": "string"},
    "condition": {"type": "string", "description": "a pure Verse expression"},
    "hit_count": {"type": "integer"},
    "logpoint": {"type": "string", "description": "a text template, as in a Verse text literal"},
    "id": {"type": "string", "description": "for action clear"}
  },
  "output": {"ok": true, "breakpoints": [{"id": "b1", "at": {"file": "ages.vs", "line": 12}, "condition": "age > 100"}], "next": ["debug_continue"]}
}
```

### `debug_break_on`

```json
{
  "input": {
    "session": "s1",
    "kind": {"enum": ["panicked", "propagated", "moved", "reassigned", "dropped",
                      "spawned", "waited", "sent", "overflowed"]},
    "name": {"type": "string", "description": "for moved, reassigned, dropped"},
    "error_type": {"type": "string", "description": "for propagated"}
  },
  "output": {"ok": true, "breakpoints": [{"id": "b2", "kind": "propagated"}], "next": ["debug_continue"]}
}
```

`kind` takes the event names of `events.toml`, so the list grows only when the
vocabulary grows.

### `inspect_locals`

```json
{
  "input": {
    "session": "s1", "task": "t3",
    "call": {"type": "integer", "default": 0, "description": "0 is the innermost call"},
    "page": {"type": "integer", "default": 1}
  },
  "output": {
    "ok": true, "task": "t3",
    "call": {"function": "parse_age", "file": "ages.vs", "line": 12},
    "locals": [{"id": "v481", "name": "person", "type": "Person", "state": "owned", "summary": "..."}],
    "cut": "present only when shortened",
    "next": ["inspect_value", "why_moved"]
  }
}
```

### `inspect_value`

```json
{
  "input": {"session": "s1", "id": "v481", "depth": {"default": 2}, "budget": {"type": "integer", "default": 4000}, "continue": {"type": "string"}},
  "output": {"ok": true, "value": "<value>", "next": ["why_moved", "why_changed", "history"]}
}
```

### `inspect_stack`

```json
{
  "input": {"session": "s1", "task": "t3"},
  "output": {
    "ok": true, "task": "t3",
    "stack": [
      {"call": 0, "function": "best", "at": {"file": "scores.vs", "line": 18}, "summary": "`best` called with `scores` and 7"},
      {"call": 1, "function": "<script>", "at": {"file": "scores.vs", "line": 40}, "summary": "script body"}
    ],
    "next": ["inspect_locals"]
  }
}
```

The innermost call is first. A foreign call carries `"foreign": true` and has
no locals.

### `inspect_tasks`

```json
{
  "input": {"session": "s1"},
  "output": {
    "ok": true,
    "tasks": [{"id": "t3", "parent": "t1", "function": "fetch", "state": "waiting",
               "waiting_for": "channel c2", "at": {"file": "net.vs", "line": 30}}],
    "deadlock_candidates": ["t3", "t4"],
    "next": ["inspect_stack", "why_blocked"]
  },
  "task_state": ["running", "ready", "waiting", "finished", "panicked", "cancelled"]
}
```

### `evaluate`

```json
{
  "input": {"session": "s1", "expression": {"type": "string"}, "task": "t3", "call": {"default": 0}},
  "output": {"ok": true, "value": "<value>", "next": ["inspect_value"]},
  "errors": ["D0104 when the expression is not pure"]
}
```

### `why_moved`

```json
{
  "input": {"session": "s1", "name": "guest", "task": "t3"},
  "output": {
    "ok": true,
    "answer": {
      "moved_at": {"file": "ages.vs", "line": 21},
      "moved_to": "add_guest",
      "step": 1841,
      "now_used_at": {"file": "ages.vs", "line": 24},
      "summary": "`guest` moved to `add_guest` at line 21 and is used again at line 24."
    },
    "next": ["debug_goto", "why_view"]
  }
}
```

If `guest` was never moved, `answer` is `{"moved_at": "none", "summary": "`guest` has not moved."}`.

### `why_view`

```json
{
  "input": {"session": "s1", "name": "b"},
  "output": {
    "ok": true,
    "answer": {
      "view_of": "a",
      "kept_alive_by": {"file": "ages.vs", "line": 15},
      "rule": "§4.9",
      "summary": "`b` is a view of `a` because `a` is used again at line 15."
    },
    "next": ["why_moved"]
  }
}
```

### `why_changed`

```json
{
  "input": {"session": "s1", "place": "person.age", "task": "t3"},
  "output": {
    "ok": true,
    "answer": {"event": "<event>", "step": 1840, "from": 29, "to": 30,
               "summary": "`person.age` reassigned from 29 to 30 at line 9."},
    "next": ["debug_goto", "history"]
  }
}
```

### `why_panic`

```json
{
  "input": {"session": "s1", "last_events": {"type": "integer", "default": 10}},
  "output": {
    "ok": true,
    "answer": {
      "kind": "index_out_of_bounds",
      "text": "list index 7 is out of bounds for a list of length 3",
      "at": {"file": "scores.vs", "line": 18, "column": 14},
      "values": ["<value>"],
      "events": ["<event>"],
      "summary": "Panicked at line 18: list index 7 is out of bounds for a list of length 3."
    },
    "next": ["why_changed", "inspect_locals"]
  }
}
```

It has the same fields as the panic report of 08 §8.6, plus `summary`.

### `debug_stop`

```json
{
  "input": {"session": "s1"},
  "output": {"ok": true, "summary": "Session s1 ended after 1,842 steps; the program had not finished.", "next": []}
}
```

This is the one response where `next` may be empty.

## 17.4 Panic report and event line

Both are the `event` shape of 17.2 with the fields that `events.toml` lists.
The panic report of 08 §8.6 is an `event` with `event: "panicked"` plus
`values`, `stack` and `last_events`. A recording (08 §8.5) is a stream of
`event` objects, one JSON object per line, in step order, compressed with zstd.

## 17.5 Resources

| URI | Content type | Notes |
|-----|--------------|-------|
| `verse://spec/<section>` | `text/markdown` | one section of the specification |
| `verse://diag/<code>` | `text/markdown` | the long form of a diagnostic (see 18) |
| `verse://api/<module>` | `application/json` | the module's `verse.api` |
| `verse://session/<id>/trace` | `application/x-ndjson` | events so far, one per line |
| `verse://vocab` | `application/toml` | `events.toml` itself, so a client can learn the words |

## 17.6 Versioning

`events.toml` has `version`. A new event or field is a minor change. Renaming
or removing one is a major change and needs a change to 08.
The server reports the version in the MCP `initialize` result as
`verse_vocab_version`.

## 17.7 Checks run by the build

1. Every event name in the compiler and VM has a key in `events.toml`.
2. Every key in `events.toml` is emitted by at least one test.
3. Every template contains no word from the "don't say" table (08 §8.7).
4. Every `summary` produced in golden tests has at most 25 words.
5. These schemas validate every JSON response in the golden tests.

## 17.8 Package-manager tools

These tools answer through the same server and use the rules of 17.1. Their
payloads are the `--json` records that 19 §19.2 defines for the commands of the
same name; this section fixes only the wrapper. All are read-only except
`lock`, which writes `verse.lock` and returns a diff.

| Tool | Input | Payload | Next |
|------|-------|---------|------|
| `lock` | `project`, `check: bool` | `{"changed": [{"package", "from", "to", "capabilities_added": []}], "stale": bool}` | `why_package` |
| `why_package` | `project`, `package` | `{"package", "version", "chain": [{"by", "asks"}]}`; the first line of `summary` is the `why.package` template | `lock` |
| `why_item` | `project`, `item` | `{"item", "chain": [{"from", "to"}]}`; first line is the `why.item` template | `size_report` |
| `why_capability` | `project`, `capability` | `{"capability", "chain": [{"from", "to"}]}` | `why_item` |
| `api_diff` | `project`, `against` (version, optional) | `{"required_bump": "major or minor or patch", "changes": [{"item", "change", "bump"}]}` | `apply_fix` |
| `update_preview` | `project`, `packages` (optional) | `{"updates": [{"package", "from", "to", "capabilities_added", "advisories": []}]}` | `lock` |

Their error codes are the V08xx family of the diagnostics catalog (16).

## 17.9 Open points

1. **`debug_goto`, `history`, `why_alive`, `why_blocked`, `diff_stops`,
   `source`** are optional tools and have no schema here yet. They get one in
   the next revision once the core set is agreed.
2. **Pagination token format** is opaque to clients; the server may change it.
