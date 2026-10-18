# 03. IR and pass specification

## 3.1 The representations

| IR | Purpose | Shape | Typed? |
|----|---------|-------|--------|
| Tokens | lexing | flat vector with trivia | no |
| CST | formatter, LSP, error recovery | green tree, lossless | no |
| AST | structure, spans | arena of enums | no |
| HIR | desugared core language | arena of enums, names resolved | no |
| THIR | typed core | HIR plus a type per expression | yes, generic |
| Generic MIR | ownership, drops, unwinding | CFG of basic blocks | yes, generic |
| Mono MIR | optimization, backends | same shape, no type parameters | yes, concrete |

## 3.2 HIR: the desugared core

HIR removes every piece of convenience syntax so later stages handle a small
language. The table gives the mapping. Source forms are from the spec.

| Surface form | HIR core |
|--------------|----------|
| `a otherwise b` (§3.8.2) | `match a { Present(x) -> x; Absent -> b }` over the optional or fallible type, typed later |
| `expr?` (§8.3) | early `return` of the absent or error case from the enclosing function or lambda |
| `if x is p` (§6.2) | single-arm match with fallthrough |
| `.name` key-path lambda (§5.4) | lambda with one parameter |
| `x if cond` postfix guards (§6.7) | `if cond` block |
| `"{a} and {b:.2}"` | concatenation of `Show` calls and format calls |
| `with e as y` (§13) | binding plus a scope with a cleanup edge |
| `for x in it` (§6.3) | loop over `Iterable.next` with a pattern |
| plain dot through optionals (§8.1) | `Lift(chain)` nodes at consuming positions only; other positions error |
| `a.b.c(args)` uniform call syntax (§5.7) | resolved call node (method or function) |

## 3.3 MIR

MIR is the only IR the backends and the optimizer see. It is a CFG in SSA-like
form for temporaries, with named, typed, **mutable locals** for user bindings
(as in Rust MIR). A later pass can promote locals to SSA values for the
backends that want it (Cranelift and LLVM build SSA themselves).

### Core definitions

```
Function { id, params: [Local], ret: Type, locals: [LocalDecl], blocks: [Block],
           unwind_cleanup: [Block], attrs }
Block     { stmts: [Stmt], term: Terminator }

Stmt =
  | Assign(Place, Rvalue)
  | StorageLive(Local) | StorageDead(Local)
  | Drop(Place, flag: Option<DropFlag>)         // run Cleanup and free
  | SetDiscriminant(Place, Variant)
  | Intrinsic(IntrinsicOp, [Operand])            // backend-visible primitives
  | DebugMark(SourceMark)                        // stepping and variable names

Rvalue =
  | Use(Operand)
  | Move(Place) | Copy(Place)                     // ownership made explicit
  | BorrowShared(Place) | BorrowMut(Place)        // only as call arguments
  | Binary(op, Operand, Operand)                  // wrapping ops only
  | CheckedBinary(op, Operand, Operand)           // returns (value, overflowed)
  | Cast(kind, Operand, Type)
  | Aggregate(Kind, [Operand]) | Len(Place) | Discriminant(Place)

Terminator =
  | Goto(Block) | Switch(Operand, [(Const, Block)], Block)
  | Call(Callee, [Operand], dest: Place, next: Block, unwind: Option<Block>)
  | CallDyn(vtable_slot, [Operand], dest, next, unwind)
  | Return | Unreachable
  | Panic(PanicKind, Message, unwind: Option<Block>)
  | Yield(YieldKind)                              // scheduler point, see 05
  | Spawn(Callee, [Operand], dest: Place)         // arguments already moved
```

### Invariants (checked by the verifier)

1. Every `Place` is typed and every operand matches the expected type.
2. A `Move(p)` is followed on every path by no read of `p` before it is
   reassigned (the verifier re-derives this independently of the ownership
   pass as a safety net).
3. Every `Drop` of a possibly-moved place has a drop flag.
4. `BorrowShared` and `BorrowMut` appear only as call operands, matching the
   second-class rule. They never reach a stored position.
5. Every block with a `Call`, `Panic` or `Yield` that can unwind names a
   cleanup block, and cleanup blocks drop in reverse declaration order.
6. `CheckedBinary` is followed by a `Switch` to a `Panic(Overflow)` block, so
   no backend decides overflow policy on its own.
7. In **mono** MIR no type contains a type parameter, and every callee is a
   concrete function or an intrinsic or a `dyn` slot.

### Why this shape

- **Explicit moves and drops** let every backend, including the plain
  bytecode interpreter, run identical semantics with no ownership logic of
  its own. The interpreter is a few hundred lines of dispatch because the hard
  part is already done.
- **Checked arithmetic as data** makes overflow policy a MIR concern, so
  optimizations (range analysis) can delete checks in one place for all
  targets.
- **Unwind edges as data** lets each backend choose the mechanism: native
  unwind tables, WebAssembly exception handling, or status-code propagation
  (see [05](05-runtime.md), section 5.4).

## 3.4 Pass list

Mono MIR optimizer passes, in order. Each pass states what it must preserve.
Every pass is individually switchable (`--no-opt-pass=NAME`) so the
differential tester can isolate a miscompile.

| # | Pass | Effect | Must preserve |
|---|------|--------|---------------|
| 1 | `verify` | check invariants | none (read-only) |
| 2 | `simplify-cfg` | merge and remove blocks, fold constant switches | control flow semantics |
| 3 | `inline` | inline small and single-call functions; always-inline for intrinsics | unwind and drop order |
| 4 | `sroa` | split records and tuples into scalars when they do not escape | field-level move state |
| 5 | `const-prop` and `fold` | evaluate constants, fold branches | overflow panics (folding a checked op that overflows stays a panic, not a wrap) |
| 6 | `move-elim` | remove copies of values at last use (spec: "a source at its last use is moved") | observable `Cleanup` order |
| 7 | `copy-elide` | avoid copying immutable data when the copy is never mutated and the source outlives it | no aliasing visible to `mut` |
| 8 | `range-analysis` | prove bounds and arithmetic checks redundant | panics that can still occur |
| 9 | `devirtualize` | turn `dyn` calls into direct calls when the concrete type is known | semantics |
| 10 | `drop-opt` | remove drops of types without `Cleanup` and with no heap, merge flags | order of `Cleanup` calls |
| 11 | `dce` | delete unreachable blocks and unused pure computations | side effects and panics |
| 12 | `layout` | choose field order, niche optimization (`T?` of a pointer type costs no extra byte) | none observable |
| 13 | `refcount-opt` | elide atomic operations on `Shared` when whole-program analysis shows the program never spawns, or elide the pair on a clone and drop in one block | counts at any observable point |
| 14 | `task-lower` | mark may-yield functions, build state machines or insert stack-switch points depending on target | task semantics |

Rules for adding a pass:

1. A pass has a one-paragraph specification of what it changes and preserves.
2. It has a golden test and a differential test (optimized equals unoptimized
   on the conformance suite).
3. It has a benchmark showing it helps, or it is deleted (spec principle:
   "no unnecessary complexity").

## 3.5 Layout rules

Decisions that backends and the runtime share:

- `Int` is 64-bit on every target, including 32-bit microcontrollers (spec
  §4.1). Cost on 32-bit cores is noted in [09](09-performance.md).
- Records are laid out in the order that minimizes padding unless marked
  otherwise; the layout is not observable from Verse.
- `T?` uses a niche when `T` has a spare bit pattern (pointers, `Bool`,
  `Char`, enums with spare discriminants); otherwise a tag byte.
- Variant types use a tag plus the largest payload, with niche packing for
  single-payload variants.
- `Text` is a UTF-8 buffer with a small-string form of up to 22 bytes held
  inline in a 24-byte header, on 64-bit targets (assumption, benchmarked in
  M1). The header also caches whether the text is pure ASCII, and its
  character count when known, to make `len()` and indexing cheap (see
  [09](09-performance.md), risk P1).
- `List<T>` is `(ptr, len, cap)`, contiguous. `Map` preserves insertion
  order: an indices table plus a dense entries array (the "compact dict"
  design). `Set` uses the same structure.
- Recursive variants are boxed by the compiler (spec §4.10 point 3).
- Trait objects are `(data ptr, vtable ptr)`.
