## 15. Standard library overview

The standard library is small, consistent, and readable. Every module follows
the same conventions: functions that can fail return `T | Error` (or a
module-specific error type); functions that may find nothing return `T?`;
resources implement `Cleanup`.

### 15.1 The prelude

These names are available in every file without importing. A local or a
file-level import may hide one (an import warns); see §3.9.

| Name | Description |
|------|-------------|
| `print(value: Show = "", end: Text = "\n")` | write `value` to standard output followed by `end`; `print()` with no argument writes an empty line |
| `input(prompt: ""): Text?` | read a line from standard input; `none` at end of input |
| `args` | `List<Text>` of command-line arguments |
| `env(name): Text?` | read an environment variable |
| `exit(code: Int)` | end the program immediately with a status code |
| `error(message, cause: none): Error` | build a simple error |
| Types | `Int`, `Float`, `Bool`, `Char`, `Text`, `Bytes`, `Nothing`, `List`, `Map`, `Set`, `Range`, `Task` |
| Traits | `Show`, `Eq`, `Order`, `Hash`, `Iterable`, `Index`, `Default`, `Cleanup`, `Copy`, `Copyable`, `Error`, `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` |

`Error` is a trait, not a type. Written where a type is expected, as in
`T | Error`, a trait name means "any type that implements it" (§8.2, §10.3),
which is why `Error` appears in return types throughout this spec.

File access is **not** in the prelude: there is exactly one name for each idea,
and it lives in `std.fs` — `open(path): File | Error` (read),
`create(path): File | Error` (write), and `read_text(path): Text | Error`
(whole file at once). Import them with `import open, read_text from std.fs`.

### 15.2 Core modules

| Module | Purpose | Highlights |
|--------|---------|------------|
| `std.fs` | files and directories | `read_text`, `write_text`, `append_text`, `read_bytes`, `exists`, `list_dir`, `walk`, `make_dir`, `remove`, `remove_all`, `open`, `create`, `File` |
| `std.net` | networking and HTTP | `get`, `post`, `put`, `delete`, `Request`, `Response(status, headers, body)`, `serve`, `TcpStream`, `Url` |
| `std.text` | text processing | `words`, `lines`, `pad`, `wrap`, `similarity`, `Regex`, `Builder`, Unicode normalization |
| `std.time` | dates, times and durations | `now()`, `today()`, `sleep(d)`, `Duration` (`5.seconds()`, `200.millis()`, `2.hours()`), `Instant`, `Date`, `DateTime`, time zones, `format` |
| `std.json` | JSON | `parse(text): Json \| Error`, `stringify(value, pretty: false)`, the `Json` variant type, `ToJson` / `FromJson` traits with automatic implementations for records and variants |
| `std.task` | concurrency | `Channel` (`Channel.new<T>()` returns a `Sender<T>` and a `Receiver<T>`), `Shared`, `Guarded`, `Weak`, `wait_all`, `wait_any`, and the named error types `Timeout`, `Cancelled`, `TaskPanicked` and `Closed` (returned by `Sender.send` once the channel is closed); `.copy()` on `Sender`, `Shared`, `Guarded` and `Weak` copies the handle, explicitly, also when passing it to `spawn`; `Receiver` is not copyable and moves (§11.4) |
| `std.math` | mathematics | `pi`, `e`, `sqrt`, `sin`, `cos`, `log`, `clamp` |
| `std.num` | numeric types | sized integers, `BigInt`, `Decimal`, `OrderedFloat`, parsing, and `ParseError`, the error returned by `Text.to_int()` (`Int \| ParseError`) and `Text.to_float()` (`Float \| ParseError`) |
| `std.random` | randomness | `int(range)`, `float()`, `choice(list)`, `shuffle(list)`, seeded generators |
| `std.process` | child processes | `run(cmd, args)`, `Output(status, stdout, stderr)` |
| `std.env` | environment | `vars()`, `current_dir()`, `home_dir()`, `platform` |
| `std.cli` | argument parsing | declarative flags from a record type |
| `std.log` | structured logging | `info`, `warn`, `error`, levels, sinks |

Domain libraries (`finance`, `ml`, `web`, `gui`, `iot`) ship as optional
packages, not in `std` (§16.3).

A few examples:

```verse
import std.fs
import std.json
import std.time
import get from std.net

started = time.now()

response = get("https://api.example.com/items")?
items = json.parse(response.body)?

fs.write_text("items.json", json.stringify(items, pretty: true))?

names = fs.list_dir(".")?
for name in names.filter(n -> n.ends_with(".vs"))
    print(name)

print("done in {time.now() - started}")
```

`json` values can be converted to and from records directly:

```verse
type Item
    id: Int
    title: Text

raw = r"""
    {"id": 1, "title": "Pen"}
    """                                   # raw: `{` is not interpolation
item = Item.from_json(json.parse(raw)?)?
print(json.stringify(item))
```

---

