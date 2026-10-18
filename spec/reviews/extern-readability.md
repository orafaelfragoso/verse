# Readability constraints for `extern` (spec 0.2, needed for bindgen)

Author: Michelangelo. Semantics (callable only inside a `with` block holding the
matching `sys` capability; plain data in and out) are Picasso's draft. These are
the constraints any form of `extern` must meet.

1. **No new keyword-shaped surprises.** Declaring a foreign function reuses
   the existing import form: the foreign module is named after `from`, so it
   reads like every other import (`import sqrt, pow from c "libm"` is the
   shape to evaluate; the exact spelling is for the spec section).
2. **The signature is always written.** Foreign code has no body to infer from,
   so parameters and return type are annotated, like required trait methods
   (§10.1). Ownership words are written too: `take`, `mut`, or a plain borrow.
3. **The call site looks like a normal call.** `sqrt(x)` inside the `with`
   block, no `unsafe` marker, no cast syntax. The `with` block is the one
   visible marker that foreign code runs here.
4. **Fallible by type, never by convention.** A foreign function that can fail
   returns `T | Error`. A status code is turned into an error value by the
   generated binding, not by the caller.
5. **Plain data only, named in the error.** A call that passes a non-plain type
   is a compile error that names the type and says which types are allowed.
6. **Generated bindings are ordinary files** (07 §7.3): header comment, in
   `generated/`, all normal rules apply.
7. **Debugger wording.** A foreign call appears as `called` with
   `foreign: true`; its inside is not stepped, and `inspect` of a value that
   crossed the boundary says `state: owned` with a `from_foreign` note.
