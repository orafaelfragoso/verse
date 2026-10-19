# ADR-0017: Propagation is the prefix word `try`

Status: accepted (2026-10-19).

## Context

`?` had three meanings: the type `T?`, the pattern `x?`, and postfix `expr?`
for propagation (spec §1.2). The overload needed three rules to keep readers
from misreading it: `a?.b` was refused because readers take it for safe access
(V0205), `(x?)?` needed brackets, and two postfix `?` in a row were banned. It
was also the reason `empty?()` was rejected (spec §18.19).

## Decision

1. **`try expr`** means what postfix `?` meant: stop early on an error or a
   `none`, with the table of spec §8.3. `?` keeps only the type `T?` and the
   pattern `x?`.
2. **Scope.** `try` covers the whole postfix chain to its right, including
   calls inside that chain's arguments, as in Swift: one `try` covers both
   calls of `try json.parse(read_text(path))`. "Propagate, then read" is
   bracketed: `(try find_user(id)).email`.
3. **Precedence.** `try` binds looser than a call, an index or a dot and
   tighter than `**` and every binary operator (spec §3.8), so
   `try nickname(7) otherwise "anonymous"` is
   `(try nickname(7)) otherwise "anonymous"`, the order Swift and Zig use.
   `try try x` peels two layers.
4. **Lambdas.** A lambda body is a function of its own and needs its own
   `try`; its return type becomes fallible (spec §5.4).
5. **Top level.** In a script, `try` on a failed value prints the error and
   exits with status 1, as before.
6. **Codes.** V0205 is unassigned and never reused. V0412 suggests
   `try …`, and V0413 says "`try` can only be used in a function that returns
   an optional or raises".
7. **Keyword.** `try` joins the keywords of spec §3.6.

## Alternatives

- **Keep postfix `?`.** The three rules above exist only to keep `?` from
  being misread.
- **Rust's postfix `?` with the other meanings renamed** (`Option<T>`,
  `Some(x)`). `T?` and `x?` read well and are the common case.
- **A `try!(…)` macro.** Rust replaced it because it could not cover a chain;
  a prefix word that covers the whole chain has no nesting problem.

Prior art: Swift's `try`, which covers everything to its right; Zig's prefix
`try`; Midori's required call-site `try`.

## Consequences

- A reader sees at the start of a line that it can stop early.
- `?` means "may be missing" and nothing else, in types and patterns.
- Every example in the docs is rewritten from `expr?` to `try expr`, with one
  `try` per chain.
