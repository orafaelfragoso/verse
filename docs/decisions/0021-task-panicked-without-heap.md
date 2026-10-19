# ADR-0021: `TaskPanicked` without a heap

Status: accepted (2026-10-19).

## Context

`TaskPanicked.message()` and `trace()` return `Text` (spec §11.1). A
`--no-heap` build forbids building a `Text` (05 §5.7), and no doc said what
those methods do there, or whether a program that handles a task panic can be
built for `--no-heap` at all. 15 §15.7.1 left it as an **Open:** sentence.

## Decision

1. `message()` and `trace()` keep their meaning and keep building `Text`. A
   `--no-heap` build that can reach either one fails with V0903, the
   existing allocation error, also when the call goes through an `Error`
   that holds a `TaskPanicked` (a `dyn-call` of the slot, 06 §6.4). No new
   code is needed.
2. `TaskPanicked` gains `code()`, which returns the panic's run-time code,
   such as `"R0002"`, as a text literal. It needs no heap, so a `--no-heap`
   program can still tell panics apart.
3. A `--no-heap` runtime keeps no stored copy of the stack lines; the
   fixed-size panic record is all it keeps (15 §15.7.1).
4. The report printed at the moment of the panic is unchanged in every
   build, because it streams and needs no heap
   ([ADR-0012](0012-one-call-limit-and-trace.md)).

## Alternatives

- **Return a static message under `--no-heap`.** It covers the fixed part of
  each message but drops the facts (the index, the length), so one method
  would return different text depending on a build flag.
- **A fixed-size inline buffer.** It truncates, and any size is arbitrary.
- **Expose the span and task id as values.** No standard type for a source
  position exists yet; `code()` is enough to tell panics apart, and the
  printed report already has the rest.

## Consequences

- A `--no-heap` program can match `TaskPanicked` and branch on `code()`.
- Under `--no-heap`, calling `message()` on an `Error` that can hold a
  `TaskPanicked` is refused, as for any error type whose `message()`
  allocates.
