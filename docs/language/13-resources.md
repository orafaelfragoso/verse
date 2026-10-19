## 13. Resource management

### 13.1 `with` blocks

External resources — files, network connections, locks, temporary directories —
must be released promptly and reliably. Because every value has one owner and
is freed at the end of its owner's scope (§4.10), a type that implements the
`Cleanup` trait is cleaned up **automatically** when its owner's scope ends,
however it ends: normally, by `return`, `break`, `continue`, `?`
propagation, or a panic.

```verse
trait Cleanup
    fn cleanup(mut self)       # must not fail
```

That automatic cleanup is a safety net: a file you open in a function is
closed when the function ends, even if you forget about it. The `with`
statement is the **explicit** way to say *exactly* where a resource's life
ends. It acquires the resource, binds it to a name for the indented block, and
cleans it up when the block ends:

```verse
import open from std.fs

with open("log.txt")? as file
    for line in file.lines()
        print(line)
# file is closed here, not at the end of the function
```

`with … as name` binds the resource itself, with one documented exception: a
**lock guard**. `with guarded.lock() as value` (§11.4) cleans up the guard
(releasing the lock) like any other resource, but binds `value` to the
protected value inside the guard, as an exclusive borrow (mutable only with
`as mut value`). Lock guards are the only type that behaves this way.

The same `mut` rule applies to every resource: `File`'s write methods
(`write_line`, `write`, `flush`) take `mut self`, so a file you write to is
bound with `as mut`: `with create("out.txt")? as mut target`.

`cleanup` must not fail: it has nowhere to report an error, because it may run
during a panic or while another error is propagating. Types whose closing can
fail (a buffered writer that must flush) also offer an explicit, fallible
method — `file.finish()?` — to call when the error matters; `cleanup` is the
quiet fallback.

`cleanup` may block, like any other code (a buffered `File` flushes when it is
closed), and nothing limits how long. A task blocked inside a `cleanup` still
counts as blocked when the program looks for a deadlock (§12.5); only `exit`
ignores a blocked cleanup. Cancellation is masked while a cleanup runs (§11.5).

### 13.2 Acquisition failure

Acquiring a resource can fail: `open` returns `File | Error`. **`with` never
propagates anything by itself.** Its resource expression must be a value whose
type implements `Cleanup`, so a fallible acquisition needs an explicit `?`,
exactly as anywhere else:

```verse
import open from std.fs

with open("log.txt")? as file          # `?` propagates a failed open
    ...

with open("log.txt") as file           # error: `File | Error` does not implement Cleanup
    ...                                #   help: add `?` to propagate the error
```

So `with` means one thing — *clean this up when the block ends* — and every early
exit is still marked by `?` or `return`. Infallible acquisitions such as
`counter.lock()` (§11.4) need no `?`.

To handle the failure locally, acquire first and decide:

```verse
match open("log.txt")                  # File | Error
    err: Error -> print("no log yet: {err.message()}")
    file: File ->
        with file as log               # `file` moves into the `with`
            print(log.read_all())
```

### 13.3 Several resources

Several resources can be acquired in one `with`, separated by commas. They are
cleaned up in reverse order of acquisition.

```verse
import open, create from std.fs

with open("input.txt")? as source, create("output.txt")? as mut target
    for line in source.lines()
        target.write_line(line.upper())?
```

### 13.4 Custom resources

```verse
import std.fs
import std.random

type TempDir
    path: Text

impl TempDir
    fn create()                            # inferred: TempDir | Error
        path = "/tmp/verse-{random.int(0..1_000_000)}"
        fs.make_dir(path)?
        TempDir(path:)

impl Cleanup for TempDir
    fn cleanup(mut self)
        _ = fs.remove_all(self.path)       # ignore failures during cleanup

with TempDir.create()? as dir
    fs.write_text("{dir.path}/scratch.txt", "draft")?
    ...
```

`_ = …` discards a result deliberately; `cleanup` must not fail, because
it may run during a panic.

---

