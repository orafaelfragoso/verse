# ADR-0013: Values close at their last use, and drop glue runs no user code

Status: accepted (2026-10-19).

## Context

A value has one owner, and dropping it releases the runtime-owned handles in
it (spec §4.10). A second statement that ends a resource's life at a written
place gives two spellings for one meaning (spec §1.4), and binding the value
inside a lock guard instead of the guard is a third rule.

If a type's drop can run code the program writes, the drop can block and can
panic. Each of those needs machinery in every tier: a cancel mask so that a
cancel request does not interrupt a drop, a second-panic rule and its run-time
panic code, a `drop` terminator with an unwind edge, a VM switch point inside a
drop, and the verifier rules that keep them consistent. All of it exists to
support code inside drop glue.

## Decision

1. **Release at last use.** A value is dropped right after its owner's last
   use, on every path: normal flow, `?`, `return`, `break`, a panic and a
   cancellation. Values whose last use is the same point drop in reverse
   declaration order, and the parts of one value in declaration order (spec
   §4.10, §13). A use through a view of the value counts as a use of it, and
   a temporary that no name owns is dropped at the end of the statement that
   made it. A `Task` owned by a binding, or discarded by a bare `spawn`, is the
   one exception: it is waited for when its scope ends (spec §11.3). There is
   no statement that ends a resource's life at a written place.
2. **Drop glue runs no user code.** There is no trait a program implements to
   run code at a drop. Drop glue frees memory and releases runtime-owned
   handles: it closes a file or a socket (a `File` writes what is still
   buffered first, and an error there is lost), releases a lock, closes a
   channel end, counts a `Shared` down, and cancels a task held in a value and
   hands its join to the runtime. Glue never fails, never panics and never
   pauses the task.
3. **Named close methods.** A close that can fail, or that must be explicit,
   is a method with a name: `file.finish()?` reports the error of the final
   write. A type the program writes that holds something to release, such as
   a temporary directory or a foreign pointer, offers such a method, and a
   value dropped without it is not released. Such a type implements the
   marker trait `Unique` (spec §10.4), so it is neither `Copy` nor
   `Copyable` and its close method runs at most once per resource.
4. **Lock guards are ordinary values.** `g.lock()` returns a `Guard<T>`. The
   protected value is read with `guard.get()` and changed with
   `guard.put(value)` or `guard.replace(value)`; the lock is released at the
   guard's last use (spec §11.4). `put` and `replace` take `mut self`, so a
   view from `get()` cannot outlive a change. The setter is `put`, not `set`,
   because `set` is a reserved word (spec §3.6).
5. **Cancellation needs no mask.** Cancellation is delivered only at a
   `yield`, and cleanup blocks contain no `yield` and no drop that pauses, so
   unwinding has no switch point and a cancel request cannot interrupt a drop
   (spec §11.5).
6. **One MIR `drop`.** `drop` is a statement only: it cannot unwind or yield.
   The `cleanup` layout fact means that the glue releases a runtime-owned
   handle; `drop-opt` never moves such a drop (13 §13.3). The `abort` profile
   is allowed only when no platform handle, whose release reaches outside the
   process, is live at a panic site (V0908). The VM has one drop
   instruction, `DROP`, and no switch point in a drop (14 §14.4).
8. **`with` stays reserved.** `with` is a keyword with no statement, like
   `let`: the compiler answers it with the error "`with` is not part of
   Verse" and the help "a value closes at its last use; call a close method
   such as `finish()` to close it at a written place" (spec §3.6).
7. **Codes.** V0702, V0707 and R0009 stay unassigned and are never reused
   (16 §16.5, §16.6). The verifier rules V4, V5, V7, V14, V15, V17 and V18
   keep their numbers and state only what a drop that runs no user code needs
   (13 §13.5).

## Alternatives

- **Keep a scoped resource statement as the explicit marker, with the drop at
  the end of scope as the safety net.** This is Python's `with` beside
  reference counting. It is two spellings for one meaning, and the lock guard
  needs a special binding rule.
- **Keep user-written drop code, but forbid it to block or panic.** This
  removes the second-panic rule but keeps the terminator form of `drop`, and
  neither property can be checked at compile time, because the code may call
  any function.
- **User-written drop code with full power, as Rust's `Drop`.** Rust needs
  unwinding through drops, a double-panic abort and `ManuallyDrop`; Verse
  would need the cancel mask too, because a drop could block.
- **Explicit release only, as Zig's `defer`.** Every resource then needs a
  written release, and forgetting one is a leak in the common case, not only
  for the uncommon resource the runtime does not own.

Prior art: Swift ends an object's lifetime after its last use rather than at
the end of its scope. Rust's `BufWriter` flushes on drop and discards the
error, which is why a fallible close is a named method there too.

## Consequences

- 05, 13, 14 and 15 need no cancel mask, no second-panic rule, no `drop`
  terminator, no switch point inside a drop and no catch point in drop glue.
  A task cannot block inside a drop, so the deadlock report lists only tasks
  parked at a `yield`.
- Moving a release earlier needs no new syntax: the value's last use is the
  release point, and a named close method makes it explicit.
- A plugin wrapper around a foreign handle needs a named close method, because
  no glue can call foreign code ([07 §7.5](../architecture/plugins.md#75-ownership-and-lifetime)).
  A wrapper dropped without it leaks the foreign resource.
- Tasks a function abandons, by unwinding or by dropping a value that holds
  them, finish before the task that drops or abandons them ends, not before
  the function returns (spec §11.3). A function that returns normally still waits
  for the tasks it holds.
- A `Receiver` dropped at its last use closes its channel, so a producer that
  sends later gets `Closed` (spec §11.4).
- A `File`'s final write in its drop is a synchronous system call. A program
  that writes to a pipe that one of its own tasks reads calls `finish()`
  first, or the blocked write holds the worker thread (15 §15.3.6).
- A value whose last use is earlier than its scope end is freed earlier. Only
  runtime-owned handles make the difference visible, and the order is still
  fixed, so all tiers compare equal in the differential tests (14 §14.12).
