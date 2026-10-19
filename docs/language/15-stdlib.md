## 15. Standard library overview

The standard library is small, consistent, and readable. Every module follows
the same conventions: functions that can fail return `T raises Error` (or a
module-specific error type); functions that may find nothing return `T?`;
resources close when they are dropped, and a close that can fail is a named
method (§13).

### 15.1 The prelude

These names are available in every file without importing. A local or a
file-level import may hide one (an import warns); see §3.9.

| Name | Description |
|------|-------------|
| `print(value: Show = "", end: Text = "\n")` | write `value` to standard output followed by `end`; `print()` with no argument writes an empty line |
| `input(prompt: ""): Text?` | read a line from standard input; `none` at end of input |
| `args` | `List<Text>` of command-line arguments |
| `env(name): Text?` | read an environment variable |
| `exit(code: Int)` | end the program immediately with a status code from 0 to 255; it flushes standard output and error but drops nothing (§12.5) |
| `error(message, cause: none): Error` | build a simple error |
| Types | `Int`, `Float`, `Bool`, `Char`, `Text`, `Bytes`, `Nothing`, `List`, `Map`, `Set`, `Range`, `Task`, `ParseError` |
| Traits | `Show`, `Eq`, `Order`, `Hash`, `Iterable`, `Index`, `Default`, `Copy`, `Copyable`, `Unique`, `Error`, `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg` |

`Error` is a trait, not a type. Written where a type is expected, as in
`T raises Error`, a trait name means "any type that implements it" (§8.2, §10.3),
which is why `Error` appears in return types throughout this spec.

`ParseError` is the error returned by `Text.to_int()` (`Int raises ParseError`)
and `Text.to_float()` (`Float raises ParseError`). It is in the prelude because
those methods are.

`print` and `input` are the everyday way to use standard output and input.
The `io` module has the streams themselves, for writing to standard error,
reading bytes, and buffering.

File access is **not** in the prelude: there is exactly one name for each idea,
and it lives in `fs` — `open(path): File raises Error` (read),
`create(path): File raises Error` (write), and `read_text(path): Text raises Error`
(whole file at once). Import them with `import open, read_text from fs`.

### 15.2 Core modules

Standard library modules are top-level: `import json`, `import get from http`.
Their names are reserved. A package, a dependency or a module of your own
cannot use one (§12.2).

| Module | Purpose | Highlights |
|--------|---------|------------|
| `fs` | files and directories | `read_text`, `write_text`, `append_text`, `read_bytes`, `exists`, `list_dir`, `walk`, `make_dir`, `remove`, `remove_all`, `open`, `create`, `File` |
| `io` | standard streams and byte I/O | `stdin`, `stdout`, `stderr`, `Reader`, `Writer`, `BufReader`, `BufWriter`, `read_line`, `read_all`, `copy` |
| `net` | networking | `TcpStream`, `TcpListener`, `UdpSocket`, `resolve` (DNS), `Url` |
| `http` | HTTP client and server | `get`, `post`, `put`, `delete`, `Request`, `Response(status, headers, body)` with a `Bytes` body, `serve(port, handler)` (§15.3) |
| `text` | text processing | `words`, `lines`, `pad`, `wrap`, `similarity`, `Regex`, `Builder`, Unicode normalization |
| `time` | dates, times and durations | `now()`, `today()`, `sleep(d)`, `Duration` (`5.seconds()`, `200.millis()`, `2.hours()`), `Instant`, `Date`, `DateTime`, time zones, `format` |
| `json` | JSON | `parse(source: Text \| Bytes): Json raises Error` (bytes are read as UTF-8), `stringify(value, pretty: false)`, the `Json` variant type, `ToJson` / `FromJson` traits with automatic implementations for records and variants |
| `task` | concurrency | `Channel` (`Channel.new<T>()` returns a `Sender<T>` and a `Receiver<T>`), `Shared`, `Guarded` (whose `lock()` returns a `Guard`), `Weak`, `wait_all`, `wait_any`, and the named error types `Timeout`, `Cancelled`, `TaskPanicked` and `Closed` (returned by `Sender.send` once the channel is closed); `.copy()` on `Sender`, `Shared`, `Guarded` and `Weak` copies the handle, explicitly, also when passing it to `spawn`; `Receiver` is not copyable and moves (§11.4) |
| `math` | mathematics and numeric types | `pi`, `e`, `sqrt`, `sin`, `cos`, `log`, `clamp`, sized integers, `BigInt`, `Decimal`, `OrderedFloat`, parsing |
| `random` | randomness | `int(range)`, `float()`, `choice(list)`, `shuffle(list)`, seeded generators |
| `process` | child processes | `run(cmd, args)`, `Output(status, stdout, stderr)` |
| `os` | operating system | signals, `pid()`, users and groups, permissions, low-level system calls |
| `env` | environment | `vars()`, `current_dir()`, `home_dir()`, `platform` |
| `cli` | argument parsing | declarative flags from a record type |
| `log` | structured logging | `info`, `warn`, `error`, levels, sinks |

The standard library gives full access to the system. A program reaches
signals, users, permissions and system calls through `os`, never through a
plugin.

Domain libraries (`finance`, `ml`, `web`, `gui`, `iot`) ship as optional
packages, not in the standard library (§16.3).

A few examples:

```verse
import fs
import json
import time
import get from http

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

### 15.3 HTTP

`http` is the one HTTP client and server. A server is a function from a
request to a response:

```verse
import serve, Request, Response from http

fn hello(_request: Request): Response raises Error
    Response(status: 200, headers: {:}, body: b"hello")

serve(port: 8080, handler: hello)?
```

| Name | Signature | Meaning |
|------|-----------|---------|
| `serve` | `serve(port: Int, handler: fn(Request): Response raises Error): Nothing raises Error` | listen on `port` and answer every request with `handler`; returns only when the server cannot go on, with the error (for example, the port is taken) |
| `Response` | `Response(status: Int, headers: Map<Text, Text>, body: Bytes)` | the record a handler returns and `get` gives back |
| `body.text()` | `text(self): Text raises Error` | a `Bytes` method: the body read as UTF-8; an error if it is not valid UTF-8 |

Each request runs in a task of its own (§11.1), so a slow request does not hold
up the others. An `Error` returned by the handler becomes a response with status
500, and the error is written to the log with `log.error`; the client does not
see its message. A panic in the handler reaches `serve` as a `TaskPanicked`
error (§11.2), so it is printed when it happens and also becomes a 500.

A body is `Bytes`, because a response does not have to be text. Read it with
`.text()`, or give it to `json.parse`, which accepts bytes:

```verse
import get from http
import json

response = get("https://api.example.com/items")?
print(response.body.text()?)
items = json.parse(response.body)?
```

---

