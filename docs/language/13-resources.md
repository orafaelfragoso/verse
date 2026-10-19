## 13. Resource management

### 13.1 Release at last use

External resources, such as files, network connections, locks and channel
ends, must be released promptly and reliably. Every value has one owner (§4.9), and a
value is **dropped right after its owner's last use**. Dropping a resource
releases it, however the code goes on from there: normally, by `return`,
`break`, `continue`, `?` propagation, a panic or a cancellation. A use of
something that reads the value, such as the iterator of `file.lines()`,
counts as a use of the value. A temporary that no name owns is dropped at the end of
the statement that made it. A `Task` owned by a binding, or discarded by a bare `spawn`, is the one
exception: it is not dropped at its last use but waited for when its scope
ends (§11.3).

```verse
import open from fs

file = open("log.txt")?
for line in file.lines()
    print(line)
# `file` is closed here: the loop was its last use
```

There is no statement that marks where a resource's life ends. To release a
resource earlier, use it for the last time earlier; to keep it open, use it
later.

A resource is an ordinary value. `File`'s write methods (`write_line`,
`write`, `flush`) take `mut self`, so a file you write to is bound with
`mut`: `mut target = create("out.txt")?`. Acquiring a resource can fail, and
the failure is handled like any other: `open` returns `File | Error`, so it
needs `?`, `otherwise` or `match` (§8).

### 13.2 Order of release

Values whose last use is the same point are dropped in **reverse declaration
order**, because a later value may have been made from an earlier one. The
parts of one value drop in the order they were written (§4.10).

```verse
import open, create from fs

source = open("input.txt")?
mut target = create("output.txt")?
for line in source.lines()
    target.write_line(line.upper())?
# the loop is the last use of both: `target` closes first, then `source`
```

### 13.3 Closing explicitly

Dropping a resource cannot fail: a drop has nowhere to report an error,
because it may run during a panic or while another error is propagating. A
type whose closing can fail therefore offers a **named method** to call when
the error matters. A `File` you wrote to writes what is still buffered when it
is dropped, and loses an error there; `finish` writes it, closes the file and
returns the error:

```verse
import create from fs

mut writer = create("out.txt")?
writer.write_line("done")?
writer.finish()?                        # the fallible close: `writer` is consumed
```

| Operation | Signature | Meaning |
|-----------|-----------|---------|
| `file.finish()` | `finish(take self): Nothing \| Error` | write what is buffered and close the file; consumes it |

When a task of the same program reads the other end of the file, as with a
pipe, call `finish()` before the last use. The write in a drop is a system
call that blocks the whole worker thread while the pipe is full, so the
reading task may never run; `finish()` is an ordinary I/O call that pauses
only its own task while it waits.

### 13.4 What a drop does

A drop runs no code of the program. It frees the value's memory and releases
the **runtime-owned handles** in it:

| Handle | Released by its drop |
|--------|----------------------|
| platform handles: `File`, network connections, processes, devices | the handle is closed; a `File` first writes what is still buffered |
| a lock guard (§11.4) | the lock is released |
| `Sender<T>`, `Receiver<T>` | the channel end is closed; the channel closes when its last `Sender` is dropped |
| `Shared<T>`, `Guarded<T>`, `Weak` | the count goes down; the value inside is dropped with the last `Shared` or `Guarded` handle |
| a `Task<T>` held in another value | the task is cancelled, and it finishes before the task that dropped it ends (§11.3) |

A drop never fails, never panics, and never pauses the task, so no other task
runs in the middle of one and a cancellation cannot interrupt it (§11.5). A
value that contains a runtime-owned handle releases it when the value is
dropped. A platform handle or a lock guard is not `Copyable`, and neither is a
value that contains one (§10.4). A type the program writes for something the
runtime does not own implements the marker trait `Unique` to get the same
rule, so no copy can release the resource a second time.

Something the runtime does not own, such as a temporary directory or a handle
from a foreign library (§12.6), is released by a named method that the type
offers. A value dropped without it is not released:

```verse
import fs
import random

type TempDir
    path: Text

impl Unique for TempDir                    # one owner: no `.copy()`, so `remove` runs once

impl TempDir
    fn create()                            # inferred: TempDir | Error
        path = "/tmp/verse-{random.int(0..1_000_000)}"
        fs.make_dir(path)?
        TempDir(path:)

    fn remove(take self)                   # inferred: Nothing | Error
        fs.remove_all(self.path)

dir = TempDir.create()?
fs.write_text("{dir.path}/scratch.txt", "draft")?
dir.remove()?
```

---
