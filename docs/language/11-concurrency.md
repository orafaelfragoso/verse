## 11. Concurrency

### 11.1 Spawning tasks

`spawn` starts a function call running **concurrently** in a lightweight task
and immediately returns a `Task<T>`, where `T` is the call's return type.
`.wait()` blocks the current task until the result is ready.

**Errors in a task surface at `.wait()`.** Because a task can panic or be
cancelled, `.wait()` always returns a fallible value, which you handle like
any other: with `try`, `otherwise`, or `match`. Its raise set is the spawned
function's set plus `TaskPanicked` and `Cancelled`, kept exact (§4.6): for a
function returning `Text raises FetchError`, `.wait()` returns
`Text raises FetchError | TaskPanicked | Cancelled`. `try` inside the spawned
function returns from that function, and the error is what `.wait()` later
reports.

```verse
import get from http

fn fetch(url)
    response = try get("https://{url}")
    response.body.text()   # the body is `Bytes`; this reads it as UTF-8 (§15.3)

a = spawn fetch("a.com")
b = spawn fetch("b.com")

print(a.wait() otherwise "a failed")    # handle the failure here
page_b = try b.wait()                   # or stop early and pass it up
print(page_b)
```

There is **no `async`/`await` and no function coloring**. Any function can be
spawned, and any function can call blocking operations like `get` or `read_text`:
while one task waits for I/O, the runtime runs other tasks. Tasks are
multiplexed over all CPU cores.

The arguments of a spawned call are evaluated *before* the task starts, in the
spawning task. `spawn` applies to exactly one call expression; to spawn a small
block of work, give it a name with `fn` and spawn the call.

#### Ownership across tasks

- **`spawn f(x)` gives the task its own `x`.** The task owns its arguments
  for the whole call, so a spawned function's parameters are always owned
  values, even when the function declares them as plain borrows. A `Copy`
  value is copied into the task (§4.9), and any other value moves. A copy
  shares storage with the spawner's value only under the atomic count, and a
  change on either side copies the storage first (§4.10), so neither task sees
  the other's changes. Tasks cannot borrow from the spawner in 0.1.
- **No `mut` arguments.** `spawn f(mut x)` is a compile error: an exclusive
  borrow must end when the call returns, and a task cannot hand a `mut` borrow
  back to the spawner. Move the value in and get the changed value back from
  `wait()`, or share it through a `Guarded<T>`.
- **Using a moved value after spawning it is a use-after-move error.** In a
  loop, the error is reported on the second iteration:

  ```verse
  (tx, rx) = Channel.new<Page>()
  for url in urls
      spawn crawl(url, tx)              # error: `tx` was moved and cannot be used
                                        #   (moved into the task in one iteration, used in the next)
                                        #   help: pass `tx.copy()`, a second handle
  ```

- **Handles are copied explicitly, and cheaply.** `Sender<T>`, `Shared<T>`,
  `Guarded<T>` and `Weak<Shared<T>>` / `Weak<Guarded<T>>` values are handles. A handle is
  not `Copy` (§4.9): passing one to `spawn` moves it, so to keep using it, pass
  `handle.copy()`: a new handle to the same channel or shared value, not a copy
  of the value inside (§11.4). There is no implicit copy on `spawn`. A
  `Receiver<T>` is not `Copyable`: it is moved into the one task that consumes
  the channel.

### 11.2 Working with many tasks

```verse
import wait_all, wait_any from task

tasks = urls.map(url -> spawn fetch(url))             # each task gets its own `url`
pages = wait_all(tasks)                  # takes the tasks; a List of fallible results, in order

mirrors = mirror_urls.map(url -> spawn fetch(url))
fastest = try wait_any(mirrors)          # the first to finish; the others are cancelled
```

| Operation | Signature | Meaning |
|-----------|-----------|---------|
| `task.wait()` | `wait(take self): T raises E \| TaskPanicked \| Cancelled` | block until done; consumes the task |
| `task.wait(timeout: d)` | `wait(take self, timeout: Duration): T raises E \| TaskPanicked \| Cancelled \| Timeout` | the same, failing with a `Timeout` error if it takes too long |
| `task.is_done()` | `is_done(self): Bool` | never blocks |
| `task.cancel()` | `cancel(self)` | request cancellation (§11.5) |
| `wait_all(tasks)` | `wait_all(take tasks: List<Task<T raises E>>): List<T raises E \| TaskPanicked \| Cancelled>` | results in task order |
| `wait_any(tasks)` | `wait_any(take tasks: List<Task<T raises E>>): T raises E \| TaskPanicked \| Cancelled` | the first result to complete; the rest are cancelled |

A task keeps the exact raise set of the function it runs: `spawn fetch(url)`
with `fetch(url): Text raises FetchError | Timeout` is a
`Task<Text raises FetchError | Timeout>`, and its `wait()` returns
`Text raises FetchError | Timeout | TaskPanicked | Cancelled` (§4.6). A task
that cannot fail waits as `T raises TaskPanicked | Cancelled`.

Remember that a `List<T raises E>` contains fallible results, so the value of
`wait_all` cannot be discarded (§8.4): loop over it, or `_ =` it on
purpose.

**Panics in tasks.** If a task panics, the panic message and stack trace are
**printed to standard error immediately, at the moment of the panic** —
whether or not anyone handles it later. The waiter then receives a value of
the named error type `TaskPanicked` (from the `task` module, implementing `Error`,
with `message()`, `trace()` and `code()`), which `match` can tell apart from
ordinary errors:

```verse
import TaskPanicked, Cancelled from task

match task.wait()                    # `task` runs a function that raises `FetchError`
    page: Text -> show(page)
    crash: TaskPanicked -> alert_on_call(crash.message())
    err: FetchError -> print("fetch failed: {err.message()}")
    _: Cancelled -> print("fetch was cancelled")
```

The task boundary is the one place where a panic becomes a value, so one
failing request cannot bring down a whole server — but because the panic was
already printed, `task.wait() otherwise "failed"` can never hide a bug.

`code()` returns the panic's run-time code, such as `"R0002"`
([16 §16.6](../reference/diagnostics-catalog.md#166-run-time-panics)), as a
text literal. `message()` and `trace()` build `Text`, so a `--no-heap` build
that can reach either one, also through an `Error` that holds a
`TaskPanicked`, fails with V0903 like any other allocation; `code()` needs no heap
and works there. The report printed at the moment of the panic is the same in
every build.

### 11.3 Structured lifetimes

Tasks belong to the function (or script) that spawned them. When that function
finishes, any of its tasks that were never waited on are **waited for
automatically** before it returns. If such an unwaited task failed (with an
error, a panic, or cancellation), its error is reported on standard error and
the **process will exit with a non-zero status** (1, or 101 for a panic) when
it finishes, even if everything else succeeded. This status applies only when
the program ends naturally: an explicit `exit(code)` always wins, and its code
is the one the process returns. Discarding a `Task` value (a bare
`spawn worker(…)` statement) is therefore allowed — an accepted exception to
the rule that fallible results cannot be dropped, because its outcome cannot be
lost. A function that returns normally therefore waits for every task it
started and still holds, results are never silently lost, and a program ends
only after all of its work is done. To deliberately run something in the background beyond the
current function, return its `Task` or pass it to someone who will wait on it.

A task held inside another value (a list, a record, a tuple) follows that
value. When the value is dropped and the task was never waited on, the task is
**cancelled**, and it finishes before the task that dropped the value ends; the
drop itself does not wait (§13.4). A child that ends only because of that
cancellation prints nothing; a child that fails with its own panic or error is
reported as above. To keep the results, give the tasks to `wait_all` or wait on
each one before the value is dropped.

When the function does not finish normally, because it panics or it is
cancelled and unwinds (§11.5), it **cancels** its unwaited tasks and does not
wait for them: the runtime finishes them before the task that started them
ends, so a panicking parent never blocks behind a long child, and unwinding
never pauses (§11.5). A child that ends
only because that cancellation reached it is not an error and prints nothing; a
child that fails with its own panic or error while unwinding is reported on
standard error and counts for the exit status, as above.

### 11.4 Sharing data safely

A task owns what it was given (§11.1), so two tasks can never touch the same
value by accident, and data races are impossible. Sharing is **opt-in** and
always goes through one of these handle types from the `task` module:

| Handle | Meaning | Copying the handle |
|--------|---------|--------------------|
| `Sender<T>` | the sending end of a channel: send owned values to the task that holds the `Receiver` | a new sender to the same channel |
| `Receiver<T>` | the receiving end of a channel; single consumer in 0.1 | not copyable; it moves (fan-out uses several channels) |
| `Shared<T>` | **shared read-only ownership**: the value is freed when the last handle goes away (atomically reference-counted) | shares the value; nobody can change it |
| `Guarded<T>` | **shared mutable state behind a lock**: `g.lock()` returns a guard that reads and changes the value | shares the value |
| `Weak<Shared<T>>`, `Weak<Guarded<T>>` | a **non-owning** handle, parameterised over the handle it was made from: `h.downgrade()` on a `Shared<T>` gives a `Weak<Shared<T>>`, and on a `Guarded<T>` a `Weak<Guarded<T>>`; `upgrade()` returns `Shared<T>?` or `Guarded<T>?` respectively (`none` if the value is gone) | a new weak handle to the same value; keeps nothing alive |
| `Task<T>` | the result of `spawn` (§11.1) | not copyable; `wait` takes it |

`Sender`, `Shared`, `Guarded` and `Weak` are `Copyable` (§10.4) whatever
their inner type — even `Guarded<File>` — and their `.copy()` copies the
**handle**: it returns a new reference to the same shared value and never
duplicates the value inside. A handle is not `Copy`, so a second handle is
always written `.copy()`, including when a handle is passed to `spawn`. `Receiver<T>` is
not `Copyable` in 0.1. `Weak` accepts only a `Shared` or `Guarded` handle;
`Sender` and `Receiver` are not weakable, so `Weak<Sender<T>>` and
`Weak<Receiver<T>>` are compile errors (a channel closes when its last
`Sender` is dropped, never by weak reference).

These are the only types whose copies refer to the same value; everything else
in Verse has exactly one owner. They are also the only way to build a cycle
(§4.10): a cycle of `Shared`/`Guarded` handles is never freed, so back-pointers
should be `Weak`.

**Channels** send values from one task to another. `Channel.new<T>()` returns
a pair `(tx, rx)`: a `Sender<T>` and a `Receiver<T>`. `send` takes its value,
so ownership moves to the task that receives it. Senders are copied (one per
producing task); the receiver is moved to the single consuming task. The
channel closes when its last `Sender` is dropped. The capacity defaults to 16;
`capacity: 0` makes an unbuffered (rendezvous) channel, where each `send`
waits for a matching `recv`. The type argument is usually inferred from an
annotation instead of written: `(tx, rx): (Sender<Int>, Receiver<Int>) =
Channel.new(capacity: 10)`. A program built without a heap (`--no-heap`, for
microcontrollers) writes the capacity as a number, as in
`Channel.new<Event>(capacity: 8)`, so the buffer can be placed in static
memory; a capacity computed at run time is then a compile error. Such a
program can also `spawn`, as long as each `spawn` runs a bounded number of
times: not in a loop, not in a recursive function, and not in a function
called through a function value. Each task then gets a slot in static memory.
A `spawn` that can run without bound is a compile error.

```verse
import Channel, wait_all from task

fn worker(id, out)
    try out.send(id * id)                # `Closed` if the receiver is gone

fn start_workers(take tx, count)         # spawns and returns, without waiting
    (1..=count).map(id -> spawn worker(id, tx.copy()))   # a Sender per task

(tx, rx) = Channel.new<Int>(capacity: 4)
tasks = start_workers(tx, 20)            # `tx` moves; 20 values, room for 4
for square in rx                         # consume while the producers run;
    print(square)                        #   ends when every Sender is gone
for outcome in wait_all(tasks)
    try outcome                          # stop if any worker failed
```

Waiting for the producers before consuming can deadlock once the channel is
full, so always consume while the producers run, and wait for them afterwards.
`send` is fallible: discarding its result is the usual compile-time error for
an unhandled fallible value (§8.4). `Closed` is a named error type in
the `task` module, like `Timeout`.

A `Receiver` given to another task moves, like any non-copyable value:
`spawn consume(rx)`. Like every value, a `Receiver` is dropped at its last use
(§13.1), which closes the channel: a later `send` returns `Closed`. Keep
receiving until the producers are done, as the loop above does.

| Operation | Signature | Meaning |
|-----------|-----------|---------|
| `Channel.new<T>(…)` | `new<T>(capacity: Int = 16): (Sender<T>, Receiver<T>)` | a new channel and its two ends; `capacity: 0` is unbuffered (rendezvous) |
| `tx.send(v)` | `send(self, take value: T): Nothing raises Closed` | blocks while the channel is full; returns the error `Closed` if the `Receiver` was dropped or the channel was closed — including for a sender already blocked on a full channel, which wakes with `Closed` |
| `tx.close()` | `close(self)` | an early shortcut: closes the channel for everyone, even while other `Sender`s exist; the receiver drains what is left |
| `rx.recv()` | `recv(self): T?` | blocks until a value arrives; `none` once the channel is closed and drained |

`for value in rx` receives until the channel is closed and drained (that is,
until `recv()` returns `none`).

**Shared read-only data** — a large configuration, a dictionary, a parsed
model — is wrapped once in `Shared<T>` and handed to as many tasks as needed
without copying the data:

```verse
import Shared from task

dictionary = Shared(load_words("words.txt"))

fn check(text, words)
    text.words().filter(w -> not words.get().contains(w)).len()

a = spawn check(chapter_one, dictionary.copy())       # a new handle, same words
b = spawn check(chapter_two, dictionary)              # last use: the handle moves
```

`shared.get()` reads the value inside in place, as a borrowed parameter is
read (§5.3). Binding or storing the result copies it, which needs `T: Copy`.

**Shared mutable state** is wrapped in `Guarded<T>` and can only be touched
through a **lock guard**. `g.lock()` waits until no other guard of the same
value is alive and returns a `Guard<T>`; the lock is released when the guard is
dropped, right after its last use (§13.1), even if the code leaves early:

```verse
import Guarded, wait_all from task

hits = Guarded(0)

fn record(counter)
    mut guard = counter.lock()          # the lock is released at `guard`'s last use
    guard.put(guard.get() + 1)

tasks = (1..=100).map(_ -> spawn record(hits.copy()))       # one handle per task
for outcome in wait_all(tasks)
    try outcome                    # stop if any task failed
print("hits: {hits.lock().get()}")      # the guard is dropped after this line
```

| Operation | Signature | Meaning |
|-----------|-----------|---------|
| `g.lock()` | `lock(self): Guard<T>` | block until the lock is free, then hold it |
| `guard.get()` | `get(self): T` | reads the value inside in place, like `shared.get()`; binding the result copies it (§4.9) |
| `guard.put(v)` | `put(mut self, take value: T)` | replace the value inside; the old one is dropped |
| `guard.replace(v)` | `replace(mut self, take value: T): T` | replace the value inside and return the old one |

A guard is an ordinary value with one owner, and it is not `Copyable`. `put`
and `replace` take `mut self`, so the guard is bound with `mut` to change the
value. A copy taken with `get()` keeps the value it had. Changing a copy of a
large value copies its storage on the first change (§4.10); to change it
without that copy, take it out with `replace`, change it, and `put` it back:

```verse
fn add_name(names, take name)            # `names` is a `Guarded<List<Text>>`
    mut guard = names.lock()
    mut list = guard.replace([])
    list.push(name)
    guard.put(list)
```

### 11.5 Cancellation and time

`task.cancel()` asks a task to stop. Cancellation is cooperative: the task is
stopped at its next *blocking point* — I/O, `wait`, `sleep`, `send`, or
`recv`.

**Cancellation cannot be caught inside the cancelled task.** At the blocking
point, the task *unwinds* exactly as it would for a panic: it stops running
ordinary code, every value it still owns is dropped in reverse declaration
order, which releases its resources (§13.4), and nothing can intercept it:
not `otherwise`, not `match`, not `try`. So
`get(url) otherwise "x"` cannot swallow a cancellation and keep going. Unlike a
panic, a cancellation prints nothing. The *waiter* sees the outcome as a
`Cancelled` error (from the `task` module) returned by `.wait()`, which it may handle
normally.

A drop never pauses the task (§13.4), so unwinding has no blocking point:
once a task starts to unwind, for a panic or a cancellation, it runs to its
end, and a cancel request that arrives then changes nothing.

Because cancellation never appears as a value inside the task, blocking
operations keep simple signatures:

| Operation | Signature |
|-----------|-----------|
| `time.sleep(d)` | `sleep(duration: Duration)` — returns `Nothing`; pauses only the current task |
| `rx.recv()` | `recv(self): T?` |
| `task.wait()` | `wait(take self): T raises E \| TaskPanicked \| Cancelled` (a cancelled *child* shows up here as `Cancelled`) |

---

