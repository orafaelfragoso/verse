# ADR-0012: Drop glue frees data of any depth in bounded stack

Status: accepted (2026-10-19).

## Context

A recursive enum's recursive field is a `Box<T>` freed by the drop glue
(15 §15.3.4, §15.3.6), and a nested `List` or `Map` is freed by its owner's
glue in the same way. Glue that calls glue recurses once per level of the
data, so dropping a 100,000-cell list nests 100,000 native frames. The call
limit counts only `recursive` MIR functions, so nothing bounds that recursion
except the stack itself.

## Decision

1. Glue never calls glue for a part in its own heap allocation (a `Box`, a
   collection or text buffer whose count reaches zero, an owned `dyn`'s
   data). It pushes the part on the task's drop stack, and the outermost
   glue drains the stack in a loop (15 §15.3.6). Glue drops inline parts in
   its own body, flattened, so it never calls another glue and a drop uses
   a fixed amount of native stack, whatever the types.
2. An entry holds the part's pointer, its glue and its next part index, so
   the drop order of spec §4.10 point 4 is unchanged.
3. When an entry's last part is the one being dropped, the entry is replaced
   rather than kept, so a chain needs one entry and the stack holds one entry
   per level that still has parts after it.
4. The drop stack belongs to the worker, because a drop never yields; a task
   costs no drop-stack memory. Its first 64 entries are static; the most a
   corpus of 56,603 real JSON documents needed was 20. Deeper drops grow the
   stack in chunks.
   A chunk that cannot be allocated aborts the process with the
   `out_of_memory` report, because glue cannot unwind.
5. The spec states the guarantee: dropping a value of any depth cannot
   overflow the stack (spec §4.10).

## Alternatives

- **Count glue depth and panic at a limit.** A drop would then fail on data
  the program built legally, and glue cannot unwind.
- **Leave it unbounded and document the limit.** The failure would depend on
  data size and tier.
- **Free children before parents with a pointer-reversing walk that uses the
  freed storage.** It needs no allocation, but it changes the drop order of
  spec §4.10 point 4 and needs a type tag in storage that has none.

## Consequences

- A `Cons` list or a degenerate tree of any length drops in bounded native
  stack, in every tier, with the documented order.
- Dropping a deep tree that is not a chain allocates entries for its
  pending siblings; the only new failure is running out of memory while
  freeing, which aborts.
- A `--no-heap` build never pushes, because none of its parts has its own
  allocation.
