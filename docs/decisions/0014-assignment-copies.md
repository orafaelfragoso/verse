# ADR-0014: Assignment copies, and storage is shared until written

Status: accepted (2026-10-19).

## Context

Whether `b = a`, a `for` variable, a pattern binding or a chain result owned
its value or only viewed it depended on whether the source was used later.
Adding a line below could turn working code into an error, and every such
error had to name the later use. That contradicts spec §1.2, "exactly one
reading": a reader could not tell what a line meant without reading the rest
of the function. It also made `.copy()` the everyday way to keep a `Text`
(`.map(.name.copy())`, `counts[clean.copy()]`) and needed five diagnostics
(V0502, V0503, V0504, V0510, V0512) and a debugger value state, `view`, to
explain itself.

## Decision

1. **Assignment copies.** Binding, assigning, passing, storing, returning,
   sending to a task, a `for` variable, a pattern binding, a lookup, a slice
   and a chain result all give the new owner its own value (spec §4.9).
   Changing one owner's value never changes another's. No binding is a view.
2. **`Copy` covers every value type.** Scalars, `Text`, `Bytes`, `List`,
   `Map`, `Set`, optionals, unions, records, variants and tuples of `Copy`
   parts are `Copy`. The handles `Shared<T>`, `Guarded<T>`, `Weak<…>` and
   `Sender<T>` are `Copyable` but not `Copy`: they move, and a second handle
   is written `.copy()`, so every place that shares a changing value is
   visible. `Task<T>`, `Receiver<T>`, lock guards, platform handles, `Unique`
   types, function values that borrow their captures, and anything containing
   them move and are never copied (spec §10.4).
3. **Shared storage with copy on write.** `Text`, `Bytes` and the
   collections keep their contents in heap storage that starts with a count.
   A copy adds one; a drop takes one away and frees the storage at zero. A
   write into storage whose count is above one copies it first, inside the
   `verse-rt` primitive that writes, so no program can observe the sharing
   (spec §4.10, 05 §5.5, 15 §15.3). The count is atomic when a `spawn` is
   reachable and a plain word otherwise, the rule `Shared` already follows
   (09 P5). Inline `Text`, static literals and storage that never allocated
   have no count.
4. **Last use is an optimization, not a meaning.** A copy at the source's
   last use is compiled as a move (`move-elim`), and a copy that is never
   written and never outlives its source is read in place (`copy-elide`,
   including slices through `slice_ref`). Last use decides cost, never which
   programs compile.
5. **Parameters keep their words.** A plain parameter is read in place for
   the call, and what the function keeps is a copy. `mut` is an exclusive
   borrow. `take` makes the function the owner: a `Copy` value is copied,
   which costs nothing at the caller's last use, and any other value moves
   (spec §5.3).
6. **MIR.** The `copy` layout fact is `bitwise` or `counted` (13 §13.3). A
   `counted` copy adds one to each count the value holds, lowered as a
   `verse-rt` call or the VM's `RETAIN`. Copy on write lives in `verse-rt`,
   so MIR has no new statement. Binding mode and value state `view` do not
   exist.
7. **Codes.** V0502, V0503, V0504, V0510, V0512 and V0703 stay unassigned and
   are never reused (16 §16.3). V0501 carries the branch condition that V0510
   gave.

## Alternatives

- **Keep the view/move rule and improve the messages.** The messages already
  named the later use; the non-locality is the problem, not the wording.
- **Rust's model: explicit references with lifetimes.** Rejected by spec
  §18.10: a second-class borrow is what keeps the analysis local and the
  language free of lifetime syntax.
- **Views as syntax (`&a`).** Keeps locality but adds a sigil to every
  borrow. A copy by default gives the same locality with no sigil.
- **Deep copies with no shared storage.** Simple, but every `b = a` of a
  large list would duplicate it, and code would need `take` or `mut` to stay
  fast.

Prior art: Swift's value types, whose `Array`, `String` and `Dictionary` share
storage with copy on write; Hylo's mutable value semantics.

## Consequences

- What a line means never depends on the lines after it, and the examples of
  spec §2, §9 and §20 lose their `.copy()` calls.
- A copy that is not a last use costs an atomic count update when the program
  can reach a `spawn`. A program that keeps a copy and then writes to the
  original pays one duplicate of the storage (assumption: both are below the
  noise of the M1 and M2 benchmarks for the 09 §9.2 suites, measured there).
- Storage written by one task while another holds a copy is copied first, so
  shared storage needs no lock and is never a data race.
- `.copy()` with no arguments means "a second handle" on a handle, and is the
  value itself on a `Copy` type; `.copy(field: …)` stays the way to copy a
  record with changes (spec §4.3).
- [ADR-0002](0002-ownership-on-generic-code.md) holds unchanged: ownership
  stays a local dataflow pass on generic code. Its liveness evidence now
  places moves and drops and no longer explains a view.
- A slice of a large list that is stored keeps a copy of the range only; one
  that is only read copies nothing.
