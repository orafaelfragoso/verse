# 13. MIR specification

This document is the formal specification of **generic MIR** and **mono MIR**:
the text format, the types and layout facts, every statement and terminator,
the verifier rules, and the debug-info contract. [03](03-ir-and-passes.md)
introduces MIR and lists the passes; this document is the detail. The
bytecode VM that executes mono MIR is specified in [14](14-bytecode-and-vm.md).

Status: **draft 3**. Labels follow the [README](README.md):
*decision*, *target*, *assumption*, *proposal*. The changes this document made
to 03 §3.3 (M1 to M9) are **applied** in 03; [13.10](#1310-changes-against-03-and-open-items)
lists them for the record and keeps the rows that are still proposals. Where a
rule follows a question of [12](12-open-questions.md) (Q13 to Q19), it names the
question; all of them are **accepted** (2026-10-18), so those rules are final.

## 13.1 Purpose and relation to 03

- **Decision** ([ADR-0003](adr/0003-one-mir-many-backends.md)): every
  execution tier consumes one mono MIR in which moves, drops, checked
  arithmetic, panics, unwind edges and yield points are explicit. A backend
  holds no ownership logic and chooses no overflow or panic policy.
- **Decision** ([ADR-0002](adr/0002-ownership-on-generic-code.md)): ownership
  is analysed once, on the generic definition ([02 §2.4](02-compiler-architecture.md#24-the-ownership-pass)).
  Generic MIR is its output; monomorphization ([02 §2.6](02-compiler-architecture.md#26-monomorphization))
  substitutes types and never re-runs ownership.
- **Generic MIR** may mention type parameters and unresolved trait calls and
  has no layout. **Mono MIR** has every parameter substituted, every trait call
  resolved (or a `dyn` slot), and layout attached; the optimizer
  ([03 §3.4](03-ir-and-passes.md#34-pass-list)), reachability and all backends
  start here.
- Goals: **G1** a backend needs no ownership logic; **G2** every panic, unwind
  and yield point is a visible edge; **G3** a stable printed form for golden
  tests and `verse dump mir` (the command name is a proposal).
- Non-goals: surface sugar (gone in HIR), inference, a format for third parties
  ([13.9](#139-versioning-policy)).

## 13.2 Textual format

The printed form is canonical: the printer is the only writer, the parser
accepts exactly what the printer writes, and `parse(print(m)) == m` is a unit
test. Terminals are in quotes; `NL` is a line feed.

```ebnf
file      = "mir" , ( "generic" | "mono" ) , "rev" , INT , NL ,
            { "file" , FID , STRING , NL } , { layout } , { function } ;
layout    = "layout" , type , "size" , INT , "align" , INT ,
            { "copy" | "needs_drop" | "cleanup" } ,
            [ "tag" , ( "none" | "byte" | "niche" , "@" , INT ) ] , NL ;
function  = "fn" , path , [ "<" , targs , ">" ] ,
            "(" , [ local , { "," , local } ] , ")" , "->" , type ,
            { "caps" , "(" , IDENT , { "," , IDENT } , ")" | "mayyield" | "nounwind" } ,
            "@" , span , "{" , NL , { decl } , { block } , "}" , NL ;
targs     = tparam , { "," , tparam }          (* in a `mir generic` file *)
          | type , { "," , type } ;            (* in a `mir mono` file: the instance's type arguments *)
tparam    = IDENT , [ ":" , bound , { "+" , bound } ] ;
bound     = path | "has" , "." , IDENT ;       (* inferred structural bound, §4.2 *)
decl      = "local" , local , ":" , type , "=" , ( "ret" | "param" | "binding" ) ,
            [ "owned" | "borrowed" | "view" | "exclusive" ] , [ "mut" ] ,
            [ STRING ] , [ "@" , span ] , NL
          | "temps" , local , ":" , type , { "," , local , ":" , type } , NL
          | "flag" , FLAG , STRING , NL ;
block     = [ "cleanup" ] , BB , ":" , NL , { stmt } , term ;
stmt      = sbody , ";" , dbg , "@" , span , NL ;
term      = tbody , ";" , dbg , "@" , span , NL ;
dbg       = [ EVENT ] , [ name ] , [ "->" , name ] ;      (* never empty, V20 *)
name      = STRING | "<" , IDENT , ">" ;   (* "x" is a user name, <mul> is internal *)
sbody     = place , "=" , rvalue
          | "drop" , place , [ "flag" , FLAG ]        (* statement form: type has no `cleanup` *)
          | "set_flag" , FLAG , "=" , BOOL
          | "set_discriminant" , place , "," , INT
          | "storage_live" , local | "storage_dead" , local
          | "intrinsic" , path , "(" , [ operands ] , ")" ;
rvalue    = operand | "ref" , [ "mut" ] , place | unop , operand
          | binop , "(" , operand , "," , operand , ")"
          | ( "cast" | "checked_cast" ) , CASTKIND , operand , "as" , type
          | "aggregate" , ( "tuple" | "record" , path | "case" , type , "#" , INT
                          | "list" , type | "closure" , path ) , "(" , [ operands ] , ")"
          | "len" , "(" , place , ")" | "discriminant" , "(" , place , ")" ;
unop      = "neg" | "not" | "checked_neg" ;
binop     = "wadd" | "wsub" | "wmul" | "band" | "bor" | "bxor" | "shl" | "shr"
          | "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "ult" | "ule" | "ugt" | "uge"
          | "fadd" | "fsub" | "fmul" | "fdiv" | "frem"
          | "checked_" , ( "add" | "sub" | "mul" | "div" | "rem" | "pow" ) ;
operands  = operand , { "," , operand } ;
operand   = "move" , place | "copy" , place | const ;
const     = TYNAME , [ "(" , literal , ")" ] ;               (* Int(7), Bool(true), Nothing *)
place     = pbase , { "." , ( IDENT | INT ) | "[" , local , "]" } ;
pbase     = local | FLAG | "(" , "*" , place , ")" | "(" , place , "as" , "#" , INT , ")" ;
          (* a FLAG place is a `Bool`: read with `copy dN`, written only by `set_flag` *)
tbody     = "goto" , BB
          | "switch" , operand , "[" , [ case , { "," , case } ] , "]" , "otherwise" , BB
          | "return" , [ "propagated" ] | "unreachable" | "resume_unwind"
          | "panic" , PKIND , "{" , [ fact , { "," , fact } ] , "}" , unwind
          | "drop" , place , [ "flag" , FLAG ] , "," , "next" , BB , "," , unwind
          | "call" , callee , "(" , [ operands ] , ")" , "->" , place , "," , "next" , BB , "," , unwind
          | "spawn" , callee , "(" , [ operands ] , ")" , "->" , place , "," , "next" , BB , "," , unwind
          | "yield" , YKIND , "(" , [ operands ] , ")" , "->" , place , "," , "resume" , BB , "," , unwind ;
unwind    = "unwind" , ( BB | "continue" | "none" ) ;
callee    = path , [ "<" , type , { "," , type } , ">" ]           (* function or instance *)
          | "<" , type , "as" , path , ">" , "::" , IDENT         (* trait call, generic only *)
          | "dyn" , "#" , INT | "intrinsic" , path | "sys" , IDENT , "." , IDENT ;
span      = [ FID , ":" ] , INT , ":" , INT , "-" , INT , ":" , INT , [ "in" , span ] ;   (* inlined: call-site span *)
type      = path , [ "<" , type , { "," , type } , ">" ] | "(" , type , { "," , type } , ")"
          | "(" , type , ")" | type , "?" | type , "|" , type | "&" , [ "mut" ] , type
          | "dyn" , path | TPARAM ;
          (* `( type )` is grouping: MIR has no 1-tuple. `?` binds tighter than `|` *)
case      = literal , "->" , BB ;
literal   = INT | BOOL | STRING | "0x" , HEX , 16 * HEX ;    (* a Float prints its raw bits *)
atom      = operand | STRING | IDENT ;
fact      = IDENT , ":" , atom ;
path      = IDENT , { ( "." | "::" ) , IDENT } ;
```

Lexical forms: `_N` local, `bbN` block, `dN` drop flag, `fN` file (`FID`),
`INT` decimal, `BOOL` `true` or `false`, `STRING` with JSON escapes, `IDENT` a
name, `TYNAME` the name of a scalar type, `HEX` a hex digit, `EVENT` a word of
`vocab/events.toml`. `#` starts a comment that the parser ignores and the
printer never writes.
`PKIND` is one of `overflow OP TYPE`, `div_zero`, `index_oob`, `slice_oob`,
`assert`, `expect_none`, `neg_exponent`, `stack_overflow`, `out_of_memory`; the
facts each carries are in [13.4](#134-statements-and-terminators).
`YKIND` is `send`, `recv`, `wait`, `join`, `sleep`, `lock`, `sys`, `pause`.
`CASTKIND` is `widen`, `i2f`, `char_int`, `niche`, `narrow`, `f2i`, `int_char`.
An `atom` after `panic` is a fact value: an operand (a `copy` of an initialised
scalar, or a constant), a text, or a word.

**Stable printing rules** (all golden tests depend on them):

| Rule | Text |
|------|------|
| P1 | One item per line, two-space indent, LF, no trailing spaces; blocks end at the next label or `}` |
| P2 | `_0` is the return place, then parameters in order, then the other locals in the order the MIR builder declares them (bindings and temporaries interleave; the bindings of a pattern are declared before the temporary that holds the matched value). The order is part of the golden text. Numbers are never reused |
| P3 | Blocks are numbered by depth-first preorder from `bb0`, following terminator targets in printed order; `cleanup` blocks follow all normal blocks, in order of first reference |
| P4 | Record fields print by name, tuple fields and enum cases by index; type arguments always print in full; no ids, addresses or hashes |
| P5 | Spans are `L:C-L:C`, 1-based, end exclusive, columns count Unicode scalar values; the file id is omitted when it equals the function's file |
| P6 | Integers decimal; `Float(0x...)` prints the 64 raw bits; text uses JSON escapes; functions sort by module path, name, instance key; no timestamps (02 §2.7) |

## 13.3 Types and layout facts

| MIR type | Meaning | Notes |
|----------|---------|-------|
| `Int`, `Float`, `Bool`, `Char`, `Nothing` | scalars (spec §4.1) | `Int` is 64-bit on every target; `Float` is IEEE 754 binary64 |
| `Int8`..`UInt64`, `Float32` | sized numerics of `std.num` | same ops, own checked-overflow ranges |
| `Text`, `Bytes`, `List<T>`, `Map<K,V>`, `Set<T>` | runtime types with primitives in `verse-rt` | opaque to MIR; handled by intrinsics, calls and `len` (see below) |
| `Slice<T>` | a view of a range of a list or of bytes: `(ptr, len)` | only as `&Slice<T>`, produced by `call intrinsic slice(...)`; V7 applies |
| `(A, B)` | tuple | fields by index |
| record `P<T>` | named fields | fields by name |
| enum `P<T>` | variant type; cases numbered in declaration order | boxed recursion is explicit in the field type |
| `T?` | enum with case `#0` = none, `#1` = present | niche packed by `layout` |
| `A \| B \| E` | fallible or union type, cases in normalized written order (§4.6) | `?` and `match` read the case with `discriminant` |
| `Task<T>`, `Sender<T>`, `Receiver<T>`, `Shared<T>`, `Guarded<T>` | runtime handles | `Task` and `Receiver` are not `Copy` |
| `dyn Trait` | `(data ptr, vtable ptr)` (03 §3.5) | only callee form is `dyn #slot` |
| `&T`, `&mut T` | shared and exclusive borrow | **MIR only**, second-class, see V7 |
| `T` (parameter) | type parameter | generic MIR only |

**Collections in MIR.** `len(p)` is the number of items of a `List<T>`, `Bytes`
or `Slice<T>`, and of *characters* of a `Text` (spec §9). Only `List<T>`,
`Bytes` and `Slice<T>` places can be indexed with `p[i]`, and only after V12's
fact. A `Text` index by character is `call intrinsic text.char_at(&p, i)`, which
ends in its own `panic index_oob {what: text, unit: characters}`. `Map` and
`Set` lookups are ordinary calls that return an option and cannot panic.
A slice `a[i..j]` is `call intrinsic slice(&p, from, to)`, whose failure edge is
`panic slice_oob`; the result is a borrow, so it can be stored only as V7
allows, and a user function never returns one.

Facts a pass may query. "Unknown" means the pass must be conservative.

| Fact | Generic MIR | Mono MIR | Used by |
|------|-------------|----------|---------|
| `copy` (implicit bitwise copy, spec §4.9) | only from a `Copy` bound; else false | exact | V1, `move-elim`, `copy-elide` |
| `needs_drop` | unknown (true) | exact | `drop-opt`, V6 |
| `cleanup` (drop glue runs user code) | from a `Cleanup` bound | exact | task-lower, 14 |
| `size`, `align`, field offsets, tag kind | absent | present, per target ([03 §3.5](03-ir-and-passes.md#35-layout-rules)) | backends, `layout` pass |
| `mayyield` | from callees, conservative for trait calls | exact | `task-lower` |
| `caps(...)` | from non-trait callees (lower bound) | exact | reachability, capability check |
| field projection on a type parameter | allowed only with a `has.field` bound | resolved to an offset | V21 |

Before the `layout` pass runs, mono MIR has sizes but field order is
declaration order (assumption). `layout` is the only pass that may change
order, and it must not change any printed name. It must not reorder the result
of a checked operation: `(T, Bool)` of `checked_*` and `checked_cast` has the
value at offset 0 and the flag in the next slot, because backends and the VM
read it that way ([14.4](14-bytecode-and-vm.md#144-instruction-set-and-encoding)).

## 13.4 Statements and terminators

**Statements** do not change the CFG. Operands are `move p`, `copy p` or a
constant. `copy` is only the implicit bitwise copy of a `Copy` type; the
explicit `.copy()` of spec §4.3 is an ordinary `call`.

| Statement | Operands | Semantics | Invariants |
|-----------|----------|-----------|------------|
| `p = op` | operand | store; `move` leaves the source uninitialised, `copy` does not | V1 V4 V9 |
| `p = ref q` / `ref mut q` | place | shared or exclusive borrow of `q`; a pointer to `q` that cannot be stored (03 `BorrowShared`/`BorrowMut`) | V7 V8 V9 |
| `p = wadd(a,b)` etc. | two scalars | wrapping integer, bit, comparison, IEEE float ops; no fused multiply-add. `ult`, `ule`, `ugt`, `uge` compare two `Int` read as unsigned (a negative value is a huge one; used for bounds, V12). `shl`, `shr` mask the count to 0..63; the 0.1 surface has no shift operator, so they come from `std.num` only (assumption) | V1 |
| `p = checked_add(a,b)` etc. | two scalars | result is `(T, Bool)`: value and overflowed. `checked_div`/`checked_rem` overflow only for `MIN / -1`; division by zero is V11's job; `checked_pow` with a negative exponent is V11's job | V10 V11 |
| `p = aggregate K(ops)` | operands | build tuple, record, enum case, empty list, closure; operands are moved or copied in | V1 V4 V7 |
| `p = len(q)`, `discriminant(q)` | place | read-only queries, `Int` result | V4 V13 |
| `p = cast k op as T` | operand | casts that cannot fail: `widen` (to a wider sized integer), `i2f` (nearest, ties to even), `char_int`, `niche` (inserted by `layout`). Never changes ownership | V1 |
| `p = checked_cast k op as T` | operand | casts that can fail: `narrow` (to a narrower integer), `f2i` (truncates toward zero; NaN or out of range fails), `int_char` (not a Unicode scalar value fails). Result `(T, Bool)` as `checked_*`; the failure is `panic convert T` (R0012), not an overflow. `to_int` of a NaN or a huge `Float` takes this path | V10 |
| `drop p [flag d]` | place | **statement form**, only when the `cleanup` fact of the type is false: if the flag is false skip; else drop the parts ([order](#drop-order)) and free. Cannot unwind or yield | V5 V6 V15 |
| `set_flag d = b` | constant | drop-flag write | V5 |
| `set_discriminant p, k` | place, case | in-place case change; only emitted by `layout` and SROA | V13 |
| `storage_live l`, `storage_dead l` | local | scope markers; `storage_dead` of a borrowed source ends it | V9 |
| `intrinsic f(ops)` | operands | backend-visible primitive that cannot unwind or yield (allocation, `list_push_unchecked`, `task.abandon`, ...) | V23 |

**Terminators** end a block. `panic`, `call`, `spawn` and `yield` can unwind and
name where unwinding goes: `unwind bbN` (a `cleanup` block), `unwind continue`
(nothing to clean, keep unwinding into the caller) or `unwind none` (cannot
unwind).

| Terminator | Operands | Semantics | Invariants |
|------------|----------|-----------|------------|
| `goto bb` | block | jump | V2 |
| `switch op [c -> bb, ...] otherwise bb` | scalar, `Bool`, flag or discriminant | multi-way branch; `otherwise` is mandatory; a case is a literal | V2 V10 V11 V12 V13 |
| `return [propagated]` | none | return `_0`; `propagated` marks a `?` exit (event `propagated`) | V2 V6 |
| `panic KIND {facts} unwind u` | kind and facts ([table](#panic-kinds)) | stop the task, record a panic, start unwinding | V10 V14 V24 |
| `drop p [flag d], next bb, unwind u` | place | **terminator form**, when the `cleanup` fact is true or unknown (always, for a value of type `T` in generic MIR): if the flag is false go to `next`; else call `cleanup` (it may block and may panic, Q13 and Q14), drop the parts, free, go to `next`. `u` cleans up the *other* locals; the parts of `p` are still dropped by the glue. Cancellation is masked while it runs; a task blocked in it counts as blocked for deadlock detection (Q14, Q17) | V5 V6 V14 V15 V18 |
| `call f(ops) -> p, next bb, unwind u` | callee, operands | call, store result, continue; borrow operands end at the call | V1 V7 V8 V14 V19 V23 |
| `spawn f(ops) -> p, next bb, unwind u` | callee, owned operands | evaluate arguments, move them into a new task, store `Task<T>`; the task starts after the move (spec §11.1) | V16 V17 |
| `yield send/recv/wait/join/sleep/lock/sys/pause(ops) -> p, resume bb, unwind u` | operands | a point where the scheduler may switch tasks; may block; **cancellation is delivered here** (not in a `cleanup` block, Q14) and takes the unwind edge | V14 V16 V17 V18 V19 |
| `unreachable` | none | the verifier or optimizer proved it cannot run; backends trap | V2 |
| `resume_unwind` | none | last terminator of a cleanup chain: continue unwinding into the caller | V15 |

`send` takes its value by move (`take value`, spec §11.4); `recv` and `lock`
take a borrow of the handle; `wait` takes the `Task` and hands its outcome to
the program. Non-blocking capability calls use `call sys cap.op(...)`; blocking
ones use `yield sys`. `lock` blocks and is a cancellation point (assumption:
spec §11.5 lists I/O, wait, sleep, send and recv but not lock). `pause` has no
surface syntax in 0.1; it is emitted only for the polling rule P14-2 and test
harnesses.

**Implicit waits and conditional spawns.** At every exit of the scope that
holds a `Task` local (normal end, `?` exit, `return`), MIR build emits one
`yield join(move _t)` per task not yet waited, in reverse declaration order
(spec §11.3). `join` behaves like `wait`, except that the runtime reports a
failed outcome on standard error and counts it for the exit status (spec
§11.3); a `wait` gives the outcome to the program. If the `spawn` is in a
branch, the task local has a drop flag: `set_flag dN = true` after the `spawn`,
`set_flag dN = false` where it is waited, and each exit reads it with
`switch copy dN [true -> bbJ] otherwise bbNext` before the `join`; the
`cleanup` path does the same switch before `task.abandon`.

**Unwind edge of `wait` and `join`.** It is taken when the *waiter* is
cancelled or panics while waiting. The runtime then cancels the awaited task
and hands its join to the scheduler (the rule of Q16, the same as `intrinsic
task.abandon`), so the `Task` operand is consumed on the normal edge and on the
unwind edge alike (V17). Only a child cancelled by an unwinding parent prints
nothing for that cancellation (spec §11.3); an explicit `cancel()` keeps the
§11.3 behaviour (an unwaited task that failed, including by cancellation, is
reported and makes the exit non-zero). A child that fails with its own panic or
error while unwinding is still reported on standard error and counts for the
exit status (Q16, accepted).

<a id="drop-order"></a>**Which `drop` form and in which order.** The form
follows the `cleanup` fact: terminator when it is true or unknown, statement
when it is false. Generic MIR therefore uses the terminator for every
`needs_drop` value of type `T`. Monomorphization keeps the form (it never
changes block structure); `drop-opt` then rewrites the terminator of a
cleanup-free type to the statement form, or deletes it, and `simplify-cfg`
merges the blocks. Locals drop in reverse declaration order (V14). The parts of
one value drop in declaration order for record fields, tuple elements and enum
payload fields, index order for list elements, and insertion order for map and
set entries (Q15, accepted; spec §4.10 is silent). Locals drop in reverse
because a later local may borrow an earlier one; the fields of one value cannot
borrow each other, so declaration order is safe and plain. A `Cleanup` type runs
its `cleanup` first and then drops its parts.

<a id="panic-kinds"></a>**Panic kinds.** The kind words and the facts come from
[16 §16.6](16-diagnostics-catalog.md#166-run-time-panics); the fact names below are the placeholder names
there, so the debugger and every tier fill the same message. Operands are
`copy` of an initialised scalar or a constant, so a panic block can be read
without owning anything.

| Printed kind | Code | Facts, in this order |
|--------------|------|----------------------|
| `overflow OP T` (`OP` is `add`, `sub`, `mul`, `div`, `rem`, `pow` or `neg`) | R0001 | `a`, `b`: the operands of the failed operation (`neg` has only `a`) |
| `convert T` (a failed `checked_cast`) | R0012 | `a`: the value, operand; `reason`: `too_large`, `too_small`, `not_a_number` or `not_a_character`, or `derived`, which the runtime resolves from `a`, the cast kind and `T` (`f2i`: NaN is `not_a_number`, else the sign decides; `narrow`: the sign decides; `int_char`: `not_a_character`). `T` is in the kind words |
| `index_oob` | R0002 | `i`, `n` (the length): operands; `what`: `list` or `text`; `unit`: `items` or `characters` |
| `div_zero` | R0003 | `a`: operand; `op`: text `"/"` or `"%"` |
| `slice_oob` | R0010 | `from`, `to`, `n`: operands; `what`; `unit` as above |
| `neg_exponent` | R0011 | `n`: the exponent, operand |
| `expect_none` | R0005 | `x`: text of the expression; `msg`: text, left out when the program gave none |
| `assert` | R0006 | `expr`: text of the condition |
| `stack_overflow` | R0004 | none in MIR; the runtime supplies `n` (the task) |
| `out_of_memory` | R0007 | none in MIR; the runtime supplies `size` |

R0008 (deadlock) and R0009 (a `Cleanup` panicked during unwinding) are not MIR
kinds: the runtime raises them (Q17, Q13). For an `assert`, the `switch` on the
condition carries the `asserted` event and its failure edge is `panic assert`.

**Statement to debug event** (vocabulary of
[08 §8.7](08-debugging-and-mcp.md#87-vocabulary-and-wording-owned-by-michelangelo)). The
`dbg` clause on each line names the event; the VM and record mode emit it.

| MIR | Event | Value state afterwards |
|-----|-------|------------------------|
| first store to a user `binding` | `bound` | `owned`, or `view` when the local has mode `view` |
| store to a `mut` binding that is already live | `reassigned` | `owned` |
| `move` of a user local | `moved` (`to` is the `->` name) | `moved` |
| `copy` of a user local | `copied` | unchanged |
| `ref q` into a call | `borrowed` (`by` is the callee, written `-> "callee"`) | `borrowed` for the call |
| `ref mut q` into a call, or the guard value of `with g.lock() as mut value` | none (the line carries the name only); state is shown as `exclusive` while it lasts | `exclusive` |
| `drop p`, either form | `dropped`; if the glue runs `cleanup`, `cleaned_up` comes first | `dropped` |
| `switch` on an `assert` condition | `asserted` (`passed` is the edge taken) | |
| `call` / `return` | `called` / `returned`; `return propagated` gives `propagated` | |
| `panic overflow ...` / other panics | `overflowed` / `panicked` | |
| `spawn`, `yield wait`, `yield send/recv`, channel close | `spawned`, `waited`, `sent`, `received`, `closed` | |

Events are data derived from MIR; in compiled tiers they exist only in record
mode and may not change behavior (Q7 in [12](12-open-questions.md)).

## 13.5 Verifier rules

`verse-verify` checks these after MIR build, after monomorphization, and after
every optimizer pass in debug builds and CI. It is written independently of the
passes ([10 §10.6](10-testing-and-verification.md)). A failure is an internal
compiler error, `V9902` ([16 §16.2](16-diagnostics-catalog.md#162-families)), whose label reads "rule {rule} failed in
`{function}`" and whose primary span is the span of the offending line. In
script mode a failure at the first call of an instance stops the run with that
diagnostic before the instance executes, and the exit status is 3, a compiler
bug ([16 §16.6.1](16-diagnostics-catalog.md#1661-exit-statuses)), in `run` as in `build`. Rule ids `V1`..`V24` are **not** diagnostic
codes (those are `V` plus four digits, [02 §2.8](02-compiler-architecture.md#28-diagnostics)).
Rules V3 to V9 re-derive what the ownership pass decided; they are a safety net
and the oracle for the soundness fuzzer ([10 §10.4](10-testing-and-verification.md)).
In the examples, `...` stands for unrelated lines.

**Fact rule** (V10 to V13). A fact about a place, such as "`d` is not zero" or
"`i` is below `len p`", starts at the edge of a `switch` whose operand is a
`copy` of the temporary that holds the test. It holds on every path from there
until the place, a prefix of it or a part of it is stored to, borrowed with
`ref mut`, moved or dropped. "Unchanged since" in the rules means exactly this.

| ID | Rule | Protects | Rejected example |
|----|------|----------|------------------|
| V1 | Operand and place types match the rvalue, call signature and destination; `copy` only of a `Copy` type; the fact types of a `panic` are those of its kind (V24) | type safety, no implicit duplicate of an owner | `_2 = copy _1` with `_1: Text` |
| V2 | `bb0` is the entry; each block has one terminator; targets exist; normal edges never enter `cleanup` blocks; `switch` cases are distinct and its `otherwise` is present; `_0` is initialised on every path to `return` | well-formed CFG | `goto bb9` with no `bb9` |
| V3 | A `temps` local is assigned once and the assignment dominates every use | SSA-like temporaries, optimizer assumptions | `_5 = Int(1)` in two blocks that both reach a use of `_5` |
| V4 | Path-sensitive initialisation: a place is read, moved or borrowed only when definitely initialised; `move` makes it uninitialised; moving all fields of an aggregate uninitialises it; a field of a `Cleanup` type is never moved out | no use after move, no partial move of a type whose `cleanup` needs the whole value | `_2 = move _1` then `_3 = copy _1` |
| V5 | `drop p` (either form) needs `p` initialised, or maybe-initialised with a drop flag; flags are written on every path that changes the state; a drop of a definitely-uninitialised place is rejected; the statement form is only for a type whose `cleanup` fact is false; a store into a place that is maybe-initialised and has `needs_drop` is rejected, so reassigning a live owner is: new value into a temporary, `drop p` under its flag, then the store (spec §4.9; Example 5) | no double drop, no drop of moved value, no leak on reassignment, `Cleanup` always reachable through a terminator | `drop _1` after `_2 = move _1` and no flag; `_1 = move _2` while `_1: Text` is owned |
| V6 | At `return` and at `resume_unwind`, every local with `needs_drop` is uninitialised (moved or dropped), except `_0`, and except an enum whose case, selected by a dominating `switch`, has no `needs_drop` payload | no leak, deterministic frees | `return` with `_1: Text` still owned |
| V7 | Borrow values (`&T`, `&mut T`) appear only as call or `yield` operands, as the source of a store into a local of borrow type (a temporary, or a `view` or `exclusive` binding), and as parameters; never in `aggregate`, in a store to a field, element or `_0`, in `spawn`, in `yield send`, or captured by `closure` | second-class borrows (spec §5.3) | `_0 = aggregate tuple(move _2)` with `_2: &Text` |
| V8 | An exclusive borrow of `p` conflicts with any other access to `p` or an overlapping place (prefix or extension) while it is live; it starts after every other operand of its call has been evaluated; no operand pair of one call may include `ref mut p` and any use of `p` | exclusive access, args first then borrow (§5.3) | `call merge(move _3, copy _1)` with `_3 = ref mut _1` |
| V9 | While a borrow is live its source is not moved, dropped, reassigned (shared borrow) or `storage_dead`; a borrow's live range ends at its last use | no dangling pointer, views stay valid | `_2 = ref _1` ... `drop _1` ... `call f(move _2)` |
| V10 | The `.1` flag of every `checked_*` and `checked_cast` result is tested by a `switch` before `.0` is read, and the failure edge reaches `panic overflow OP TYPE` (for `checked_*`, with the operands of the operation as facts) or `panic convert T` (for `checked_cast`, with `a`) | one overflow policy for all tiers (03 invariant 6) | `_5 = checked_add(...)` then `copy _5.0` with no `switch` |
| V11 | `checked_div` and `checked_rem` need a dominating fact that the divisor is not zero: a constant, or the false edge of a `switch` on `eq(d, 0)`. `checked_pow` needs the same for the exponent: a constant that is not negative, or the true edge of `ge(e, Int(0))` whose other edge reaches `panic neg_exponent {n: e}` | division and power panics are explicit data | `checked_div(copy _1, copy _2)` with no zero test |
| V12 | `p[i]` (on a `List<T>`, `Bytes` or `Slice<T>`) needs a dominating fact `ult(i, len(p))` on the true edge, or both `ge(i, Int(0))` and `lt(i, len(p))`; a signed `lt` alone does not qualify. The other edge reaches `panic index_oob {i, n: len(p), ...}`, or user code that decides what to do | bounds checks are explicit data and a negative index cannot pass | `copy _1[_2]` straight after a call that grew `_1`; `lt(_2, len(_1))` alone |
| V13 | `(p as #k)` is read or moved only on a path where a `switch` on `discriminant p` selected case `k`; moving it uninitialises all of `p` | enum payload safety | `move (_4 as #1)` in the block for case 0 |
| V14 | Every `call`, `spawn`, `yield`, `panic` and terminator `drop` that can unwind names `unwind bbN` whenever any local with `needs_drop` (other than the place being dropped) is maybe-initialised there, and `bbN` drops exactly those locals in reverse declaration order (honouring flags) before `resume_unwind`; otherwise it names `continue`. In a cleanup block the edge is `none` | `Cleanup` runs on panic and cancellation, in the specified order (spec §8.5, §11.5) | a `call` with `_2` owned and `unwind continue` |
| V15 | A `cleanup` block is reached only by `unwind` edges and by the `next` or `goto`/`switch` edges of other cleanup blocks; together they form an acyclic region. A block of it contains `drop` (either form, `unwind none`), `set_flag`, `storage_dead`, `intrinsic`, `call` and `yield` with `unwind none`, `goto`, and `switch` on a flag; no `spawn`, `panic` or `return`; every path ends in `resume_unwind`. A panic or a cancel request inside it does not start a second unwinding: the runtime handles it (Q13, Q14) | unwinding always terminates; `Cleanup` may block (a flush) but runs to completion | a `spawn` inside `cleanup bb7` |
| V16 | `spawn` operands and the value of `yield send` are `move` of an owned value or `copy` of a `Copy` value; no borrow, no `mut` | tasks own their arguments (spec §11.1) | `spawn f(move _3)` with `_3 = ref mut _1` |
| V17 | A `Task<T>` local is never the operand of `drop`; on every path it is consumed by `yield wait`, `yield join` (whose unwind edges hand the child to the runtime, Q16) or `intrinsic task.abandon` in a cleanup block. A `Task` inside a list or record is open (M10) | structured lifetimes (spec §11.3) | `drop _3` with `_3: Task<Int>` |
| V18 | `yield` may appear in normal and `cleanup` blocks. A function containing a `yield`, a call to a `mayyield` function, or a terminator `drop` of a type whose `cleanup` may yield carries `mayyield`; in generic MIR a terminator `drop` of an unknown type counts as `mayyield` | `task-lower` finds every switch point, including a blocking `Cleanup` (Q14) | `yield recv(...)` in a function without `mayyield` |
| V19 | A `sys cap.op` callee names a capability in the function's `caps(...)`, and `caps` of a function include those of its callees (in script mode, those the pre-pass of [13.6](#136-generic-mir-and-mono-mir) computed for the callee instance) | exact capability check before run (P6-5 in [12](12-open-questions.md)) | `call sys net.connect(...)` with `caps(fs)` |
| V20 | Every statement and terminator has a span inside a known file and a non-empty `dbg`; every `binding` and `param` has a name and a mode word; event words exist in `vocab/events.toml` | debugger and diagnostics speak the same words (Q6) | a `drop` line with no `;` clause |
| V21 | Generic only: an operation on a value of type `T` must be licensed by a bound of the function: trait calls need the trait bound, `copy` of `T` needs `Copy`, field projection needs `has.field` | generics are checked once, at the definition (spec §10.6) | `_2 = copy _1` with `_1: T` and bound `Copyable` |
| V22 | Mono only: no type parameter or `trait` callee remains; every `call` target exists as an instance; `layout` exists for every type used; instance keys are unique | backends can lower without resolving anything | `call <T as Show>::show(...)` in `mir mono` |
| V23 | `intrinsic` and `call intrinsic` match the intrinsic's declared signature and effects: an `intrinsic` statement cannot unwind or yield | optimizer may reorder pure intrinsics | an allocating `intrinsic` with `unwind bb3` |
| V24 | Every `panic` names a kind of the [table in 13.4](#panic-kinds) and gives exactly the facts of that row, with the stated types; operands are `copy` of an initialised scalar or constants | one panic text per kind, filled the same way by every tier (16 §16.6) | `panic index_oob {i: copy _2}` (no `n`, `what`, `unit`) |

## 13.6 Generic MIR and mono MIR

| Aspect | Generic MIR | Mono MIR |
|--------|-------------|----------|
| Header | `mir generic rev N` | `mir mono rev N` |
| Types | may contain `T`; unannotated parameters carry inferred bounds, including `has.field` | none contain a parameter |
| Callees | function, `<T as Trait>::m`, `dyn #n`, intrinsic, sys | function **instance**, `dyn #n`, intrinsic, sys |
| Drops | `drop` of `T` is the terminator form (`cleanup` unknown); `needs_drop` unknown | glue resolved, form unchanged; `drop-opt` rewrites a cleanup-free terminator to a statement or deletes it, and can delete cleanup blocks |
| Moves and flags | decided by the ownership pass | identical structure; `monomorphize` never adds or removes a move, drop or panic |
| Parameters | a plain `p` of **any** type, `Int` included, is `&T` (borrowed) and is read as `copy (*_1)` | unchanged. A later `param-by-value` pass (proposal, M11) passes small `Copy` types by value and rewrites the reads (04 §4.5) |
| Layout | none | `layout` lines, per target |
| Verifier | V1 to V21, V23, V24 | V1 to V20, V22 to V24 (V21 replaced by V22) |
| Names | source names and spans | same, plus the instance key `pair_of<Text>` in the function header |

**Monomorphization** is a function from `(generic fn, [TypeId])` to a mono
function (instance key as in 02 §2.6). It (1) substitutes types in locals,
operands and callees, (2) resolves each `<T as Trait>::m` to a concrete
function through the impl table, or leaves a `dyn` slot, (3) renames the
function with its type arguments, (4) attaches layouts, (5) recomputes `caps`
and `mayyield`. It never changes block structure, spans, `dbg` clauses or
panic sites. Ownership facts are not recomputed (spec §4.2 rule 9); the
verifier re-checks V3 to V9 on the result as a safety net.

A moved `T` stays a `move` after substitution even when `T = Int`. Turning a
`move` of a `Copy` type into a `copy` (a job for `move-elim`, 03 §3.4 pass 6;
M11), or rewriting a `drop` of a type with no glue, is the optimizer's job, so
that monomorphization stays a pure substitution and its output is predictable in
golden tests.

**Facts in script mode.** `verse run` instantiates code lazily
([02 §2.6](02-compiler-architecture.md#26-monomorphization)), so a callee is not
yet compiled when its caller is. The reachability pre-pass computes, per
instance key, the exact `caps(...)` and `mayyield` from the generic MIR with the
type arguments substituted, and caches them per module. V19 and V18 are checked
on a lazily built instance against those facts. Tier 0 never runs `task-lower`,
so it needs `mayyield` only for the verifier; the capability check needs `caps`
before the first instruction.

## 13.7 Debug info and source spans

- **Every statement and terminator** has a span and a `dbg` clause (V20).
  The clause carries the event word (08 §8.7), the user name when one exists
  (`"avg"`), else an internal name in angle brackets (`<mul>`), and for moves
  and calls the destination (`-> "add_item"`).
- **Every `param` and `binding`** local has its user name, its mode word and
  its span. The mode word uses the 08 vocabulary: `owned`, `borrowed` (a plain
  parameter seen by a call), `view` (a binding that only looks at a source that
  is used later), `exclusive` (a `mut` parameter, or the guard value of
  `with g.lock() as mut value`). Temporaries have no name.
- **Value states** (`owned`, `moved`, `view`, `borrowed`, `exclusive`,
  `dropped`, `not_yet_bound`) are computed by the VM from these words and the
  executed statements.
- **Source marks.** Stepping marks are derived from spans (a new source line
  starts at the first statement whose span begins on a different line). 03's
  `DebugMark` statement is therefore not stored (applied in 03). **Optimization
  level 0** runs no pass except the read-only `verify`; debug sessions and
  replays use it, so marks stay where users expect them and steps (next bullet)
  mean the same thing in every session.
- **Step** (Q19, accepted; [08 §8.5](08-debugging-and-mcp.md#85-deterministic-record-and-replay)). A step is one executed
  statement or terminator of mono MIR at level 0. A `drop` terminator is one
  step; the steps of its `cleanup` call are counted on their own. Steps exist
  only in tier 0.
- **Inlining** keeps the spans of the inlined body and adds the call site as a
  parent span, written `@f1:3:5-3:12 in 8:9-8:20`, so a user-level stack can be
  rebuilt (the `inspect_stack` contract in 08).
- **Panics** keep one `panic` terminator per source site; the optimizer may
  merge two only when kind, operands and span are equal.
- **Wording.** Event words come from `vocab/events.toml`. "Frame" never appears
  in debug output; the VM's internal call record ([14 §14.3](14-bytecode-and-vm.md#143-value-representation-and-call-records)) is shown as a "call".

## 13.8 Worked examples

Spans refer to the snippet as shown, starting at line 1. Each listing is
complete unless it says "fragment". Every listing is a golden test: the parser
reads it and the printer writes it back byte for byte (the `...` lines of a
fragment are removed first).

### Example 1: checked arithmetic, a postfix guard and a division

```verse
fn average(total: Int, count: Int): Int
    return 0 if count == 0
    total / count + 1
```

```mir
mir mono rev 1
fn average(_1, _2) -> Int @1:1-3:22 {
  local _0: Int = ret
  local _1: &Int = param borrowed "total" @1:12-1:22
  local _2: &Int = param borrowed "count" @1:24-1:34
  temps _3: Bool, _4: (Int, Bool), _5: (Int, Bool)
bb0:
  _3 = eq(copy (*_2), Int(0))                     ; <eq> @2:17-2:27
  switch copy _3 [true -> bb1] otherwise bb2      ; <guard> @2:5-2:27
bb1:
  _0 = Int(0)                                     ; <value> @2:5-2:13
  return                                          ; returned @2:5-2:13
bb2:
  _4 = checked_div(copy (*_1), copy (*_2))        ; <div> @3:5-3:18
  switch copy _4.1 [false -> bb3] otherwise bb6   ; <div> @3:5-3:18
bb3:
  _5 = checked_add(copy _4.0, Int(1))             ; <add> @3:5-3:22
  switch copy _5.1 [false -> bb4] otherwise bb5   ; <add> @3:5-3:22
bb4:
  _0 = copy _5.0                                  ; <value> @3:5-3:22
  return                                          ; returned @3:5-3:22
bb5:
  panic overflow add Int {a: copy _4.0, b: Int(1)} unwind continue ; overflowed <add> @3:5-3:22
bb6:
  panic overflow div Int {a: copy (*_1), b: copy (*_2)} unwind continue ; overflowed <div> @3:5-3:18
}
```

A plain parameter is borrowed even when it is an `Int`, so each read is
`copy (*_1)` (13.6). V10 holds for both checked operations, and the panic facts
are the operands of the failed operation (the table in 13.4). V11 accepts
`checked_div` because `bb2` is the false edge of the switch on `eq(*_2, 0)` and
`*_2` is not stored to in between (the fact rule): the source guard doubles as
the zero check. Without the guard, MIR build inserts a `switch` to
`panic div_zero {a: copy (*_1), op: "/"}`. No local needs drop, so every panic
says `unwind continue`.

### Example 2: generic MIR, then mono MIR, with a cleanup edge

```verse
fn pair_of<T: Copyable>(take x: T): (T, T)
    (x.copy(), x)
```

```mir
mir generic rev 1
fn pair_of<T: Copyable>(_1) -> (T, T) @1:1-2:18 {
  local _0: (T, T) = ret
  local _1: T = param owned "x" @1:25-1:34
  temps _2: &T, _3: T, _4: T
bb0:
  _2 = ref _1                                     ; borrowed "x" @2:6-2:14
  call <T as Copyable>::copy(move _2) -> _3, next bb1, unwind bb2 ; called "copy" @2:6-2:14
bb1:
  _4 = move _1                                    ; moved "x" -> <tuple> @2:16-2:17
  _0 = aggregate tuple(move _3, move _4)          ; <tuple> @2:5-2:18
  return                                          ; returned @2:5-2:18
cleanup bb2:
  drop _1, next bb3, unwind none                  ; dropped "x" @1:25-1:34
cleanup bb3:
  resume_unwind                                   ; <unwind> @1:25-1:34
}
```

`x.copy()` borrows `x` and ends the borrow at the call (V9). The move of `x` is
its last use, so no drop follows on the normal path (V6). If `copy` panics,
`_1` is still owned, so `bb2` drops it (V14). The drop is a terminator because
`T` might have `Cleanup` (13.4); in a cleanup block its own edge is `none`.
Instance `pair_of<Text>` is the same text with `T` replaced by `Text`, the
callee `Text::copy`, and a line `layout Text size 24 align 8 needs_drop tag none`
(sizes from 03 §3.5, an assumption). The header reads `mir mono rev 1` and the
function is `pair_of<Text>` (V22). `monomorphize` keeps the terminator; `Text`
has no `cleanup`, so `drop-opt` makes it the statement `drop _1` and
`simplify-cfg` merges `bb2` and `bb3`. For `pair_of<Int>` the callee is
`Int::copy`, `layout Int size 8 align 8 copy`, `drop-opt` deletes the drop, the
cleanup blocks become empty, and the edge is rewritten to `unwind continue`. A
later pass, not monomorphization, turns `move _1` into `copy`
([13.6](#136-generic-mir-and-mono-mir)).

### Example 3: `?`, `with`, an exclusive borrow and a cleanup block

```verse
import create from std.fs

fn save(path: Text, line: Text): Nothing | Error
    with create(path)? as mut target
        target.write_line(line)?
```

```mir
mir mono rev 1
fn save(_1, _2) -> Nothing | Error caps(fs) @3:1-5:33 {
  local _0: Nothing | Error = ret
  local _1: &Text = param borrowed "path" @3:9-3:19
  local _2: &Text = param borrowed "line" @3:21-3:31
  temps _3: &Text, _4: File | Error, _5: Int
  local _6: File = binding owned mut "target" @4:31-4:37
  temps _7: &Text, _8: &mut File, _9: Nothing | Error, _10: Int
bb0:
  _3 = ref (*_1)                                  ; borrowed "path" -> "create" @4:17-4:21
  call std.fs.create(move _3) -> _4, next bb1, unwind continue ; called "create" @4:10-4:22
bb1:
  _5 = discriminant(_4)                           ; <?> @4:10-4:23
  switch copy _5 [0 -> bb2] otherwise bb8         ; <?> @4:10-4:23
bb2:
  _6 = move (_4 as #0)                            ; bound "target" @4:31-4:37
  _7 = ref (*_2)                                  ; borrowed "line" -> "write_line" @5:27-5:31
  _8 = ref mut _6                                 ; "target" @5:9-5:15
  call File::write_line(move _8, move _7) -> _9, next bb3, unwind bb9 ; called "write_line" @5:9-5:32
bb3:
  _10 = discriminant(_9)                          ; <?> @5:9-5:33
  switch copy _10 [0 -> bb4] otherwise bb6        ; <?> @5:9-5:33
bb4:
  drop _6, next bb5, unwind continue              ; dropped "target" @4:5-4:37
bb5:
  _0 = aggregate case (Nothing | Error)#0(Nothing) ; <ok> @4:5-4:37
  return                                          ; returned @4:5-4:37
bb6:
  _0 = aggregate case (Nothing | Error)#1(move (_9 as #1)) ; <?> @5:9-5:33
  drop _6, next bb7, unwind continue              ; dropped "target" @4:5-4:37
bb7:
  return propagated                               ; propagated @5:9-5:33
bb8:
  _0 = aggregate case (Nothing | Error)#1(move (_4 as #1)) ; <?> @4:10-4:23
  return propagated                               ; propagated @4:10-4:23
cleanup bb9:
  drop _6, next bb10, unwind none                 ; dropped "target" @4:5-4:37
cleanup bb10:
  resume_unwind                                   ; <unwind> @4:5-4:37
}
```

`line` is borrowed first and `target` second: all operands are evaluated, then
the exclusive borrow starts (V8; spec §5.3). The payload of `_4` is moved only
on the edge for case 0 (V13). `File` has `Cleanup`, so every drop of `_6` is a
terminator (V5): `cleanup` may block and may panic, so the drop has a `next`
block and an unwind edge of its own. On the normal exit, the `?` exit and the
unwind edge the `with` guarantees the drop and `cleaned_up`, in that block
order. After the drop nothing else is live, so those edges are `continue`; in
the cleanup block the edge is `none`. `bb8` needs no drop: `target` is not
bound yet (V6 passes).

### Example 4: tasks and channels

```verse
import Channel, Sender, Closed from std.task

fn square_one(id: Int, take out: Sender<Int>): Nothing | Closed
    out.send(id * id)?

fn run(): Int?
    (tx, rx) = Channel.new<Int>(capacity: 1)
    task = spawn square_one(7, tx)
    value = rx.recv()
    _ = task.wait()
    value
```

```mir
mir mono rev 1
fn square_one(_1, _2) -> Nothing | Closed mayyield @3:1-4:23 {      # fragment
  ...
bb1:
  _4 = ref _2                                     ; borrowed "out" -> "send" @4:5-4:13
  yield send(move _4, copy _3.0) -> _5, resume bb2, unwind bb6 ; sent "out" @4:5-4:22
  ...
cleanup bb6:
  drop _2                                         ; dropped "out" @3:24-3:45
  resume_unwind                                   ; <unwind> @3:24-3:45
}
```

`yield send` is where cancellation arrives: `bb6` drops the `Sender`, which
closes the channel if it is the last one (the runtime's glue, not user code, so
the statement form). `send` takes its value by move; a
`copy` of an `Int` is an owned value (V16). The rest of `square_one` is
Example 1 (`checked_mul`, an overflow `panic ... unwind bb6`) and the `?`
pattern of Example 3. The listing of `run` uses the same file:

```mir
fn run() -> Int? mayyield @6:1-11:10 {
  local _0: Int? = ret
  local _1: Sender<Int> = binding owned "tx" @7:6-7:8
  local _2: Receiver<Int> = binding owned "rx" @7:10-7:12
  local _3: Task<Nothing | Closed> = binding owned "task" @8:5-8:9
  local _4: Int? = binding owned "value" @9:5-9:10
  temps _5: (Sender<Int>, Receiver<Int>), _6: Sender<Int>, _7: &Receiver<Int>, _8: Nothing | Closed | Error
bb0:
  call Channel::new<Int>(Int(1)) -> _5, next bb1, unwind continue ; called "new" @7:16-7:45
bb1:
  _1 = move _5.0                                  ; bound "tx" @7:5-7:13
  _2 = move _5.1                                  ; bound "rx" @7:5-7:13
  _6 = move _1                                    ; moved "tx" -> "square_one" @8:32-8:34
  spawn square_one(Int(7), move _6) -> _3, next bb2, unwind bb5 ; spawned "square_one" @8:12-8:35
bb2:
  _7 = ref _2                                     ; borrowed "rx" -> "recv" @9:13-9:22
  yield recv(move _7) -> _4, resume bb3, unwind bb6 ; received "rx" @9:13-9:22
bb3:
  yield wait(move _3) -> _8, resume bb4, unwind bb5 ; waited "task" @10:9-10:20
bb4:
  drop _8                                         ; dropped <discard> @10:5-10:20
  _0 = move _4                                    ; moved "value" -> <return> @11:5-11:10
  drop _2                                         ; dropped "rx" @7:10-7:12
  return                                          ; returned @11:5-11:10
cleanup bb5:
  drop _2                                         ; dropped "rx" @7:10-7:12
  resume_unwind                                   ; <unwind> @7:10-7:12
cleanup bb6:
  intrinsic task.abandon(move _3)                 ; <abandon> @8:5-8:9
  drop _2                                         ; dropped "rx" @7:10-7:12
  resume_unwind                                   ; <unwind> @7:10-7:12
}
```

The `spawn` and `wait` edges need only `rx` (`tx` moved; `task` unbound or
consumed). If the waiter is cancelled or panics inside `wait`, the runtime
cancels the child and joins it itself (13.4, Q16), so `bb5` only drops `rx`. The
`recv` edge also owns the task, so `bb6` abandons it first (reverse declaration
order, V14, V17): cancel the child, then let the runtime wait for it before the
unwinding of `run` completes (Q16). The verifier rejects a second `move _1`
(V4), `spawn` of a `ref mut` (V16) and `drop _3` (V17).

### Example 5: a bounds check, a drop flag and a reassignment

```verse
fn archive_if(names: List<Text>, i: Int, keep: Bool, mut store: List<Text>)
    mut text = names[i].copy()
    if keep
        store.push(text)
    else
        text = "none"
```

```mir
mir mono rev 1
fn archive_if(_1, _2, _3, _4) -> Nothing @1:1-6:22 {
  local _0: Nothing = ret
  local _1: &List<Text> = param borrowed "names" @1:15-1:32
  local _2: &Int = param borrowed "i" @1:34-1:40
  local _3: &Bool = param borrowed "keep" @1:42-1:52
  local _4: &mut List<Text> = param exclusive mut "store" @1:54-1:75
  temps _5: Int, _6: Int, _7: Bool, _8: &Text
  local _9: Text = binding owned mut "text" @2:5-2:13
  flag d0 "text"
  temps _10: Text, _11: &mut List<Text>, _12: Nothing, _13: Text
bb0:
  _5 = len((*_1))                                 ; <len> @2:16-2:24
  _6 = copy (*_2)                                 ; <index> @2:22-2:23
  _7 = ult(copy _6, copy _5)                      ; <bounds> @2:16-2:24
  switch copy _7 [true -> bb1] otherwise bb6      ; <bounds> @2:16-2:24
bb1:
  _8 = ref (*_1)[_6]                              ; borrowed "names" -> "copy" @2:16-2:24
  call Text::copy(move _8) -> _9, next bb2, unwind continue ; called "copy" @2:16-2:31
bb2:
  set_flag d0 = true                              ; bound "text" @2:5-2:13
  switch copy (*_3) [true -> bb3] otherwise bb5   ; <if> @3:5-3:12
bb3:
  _10 = move _9                                   ; moved "text" -> "push" @4:20-4:24
  set_flag d0 = false                             ; <flag> @4:20-4:24
  _11 = ref mut (*_4)                             ; "store" @4:9-4:14
  call List<Text>::push(move _11, move _10) -> _12, next bb4, unwind continue ; called "push" @4:9-4:25
bb4:
  drop _9 flag d0                                 ; dropped "text" @2:5-2:13
  _0 = Nothing                                    ; <value> @3:5-6:22
  return                                          ; returned @3:5-6:22
bb5:
  _13 = Text("none")                              ; <value> @6:16-6:22
  drop _9                                         ; dropped "text" @6:9-6:13
  _9 = move _13                                   ; reassigned "text" @6:9-6:22
  goto bb4                                        ; <join> @3:5-6:22
bb6:
  panic index_oob {i: copy _6, n: copy _5, what: list, unit: items} unwind continue ; panicked <index> @2:16-2:24
}
```

`names[i]` is checked with `ult`, so a negative `i` fails the test (V12); the
failure edge is `panic index_oob` with `i`, `n`, `what` and `unit` (V24, R0002).
`i` is copied into `_6` first because an index in a place is a local. The
branch `keep` moves `text` away, so at the join `text` is moved on one path
and owned on the other: `d0` records that, and `bb4` drops it under the flag
(V5, V6). `Text` has no `cleanup`, so these drops are statements. The `else`
branch reassigns an owned `text`: the new value is built first (`_13`), the old
one is dropped, then the store (V5; spec §4.9). The `push` call needs no unwind
cleanup: `_9` is moved and `_10` is consumed by the call. The exclusive borrow
`_11` starts after `_10` is evaluated (V8).

## 13.9 Versioning policy

- **Decision** (Q11 in [12](12-open-questions.md)): in 0.1 the MIR
  serialization, text and binary, is **internal and unversioned for third
  parties**. No plugin, tool or backend outside the repository may depend on
  it. Transforming plugins are out of 0.1.
- The header `rev N` is an internal revision. Any change to the grammar, to a
  statement or terminator, or to a printing rule increments `rev` and
  regenerates golden files in the same commit.
- A reader accepts only its own `rev`. The persistent cache key already
  includes the compiler version ([02 §2.7](02-compiler-architecture.md#27-incremental-and-parallel-compilation)),
  so a cached MIR is never read by another compiler build.
- Bytecode bundles (`.vsb`) have their own compiler-version rule
  ([04 §4.4](04-backends-and-targets.md#44-linking-and-artifacts)); they are a
  different artifact and do not carry MIR.
- Wasm library plugins are translated to the in-memory mono MIR, not to a text
  form ([04 §4.4](04-backends-and-targets.md#44-linking-and-artifacts)).
- A stable MIR form, if ever offered, needs a new document, a version scheme
  with a compatibility promise, and a ruling (proposal; not in 0.1).

## 13.10 Changes against 03 and open items

Rows M1 to M6 were proposals in draft 1 and are **applied** in 03, and so are
the `DropT` terminator of M7, the `join` kind and `CheckedCast`. Rows M7 to M9 follow Q14, Q16 and Q13 of
[12](12-open-questions.md), all accepted (2026-10-18), so they are final.
03's `CallDyn` is the `call dyn #n` form here.

| # | Change | Why | Status |
|---|--------|-----|--------|
| M1 | `Move`/`Copy` are operand forms, not rvalues; `Use` disappears | one way to write a read | applied |
| M2 | Borrows may be stored in `view`/`exclusive` locals (V7) | `b = a` view, `for x in items`, `with g.lock() as value` are bindings that look at a source | applied |
| M3 | `spawn` and `yield` get `next`/`resume` and `unwind` edges; `yield` gets operands and a destination | send, recv, wait must carry values and be cancellable | applied |
| M4 | New `resume_unwind`, `set_flag`, `intrinsic task.abandon` | end of cleanup chains, drop flags, unwinding past unwaited tasks | applied |
| M5 | `DebugMark` statement dropped; marks derive from spans | one source of truth | applied |
| M6 | `Panic` takes structured kinds and named facts, not a message string | wording lives in the vocabulary file (Q6) and 16 §16.6 | applied |
| M7 | `drop` of a type with `Cleanup` (or of unknown `cleanup`) is a terminator with `next` and `unwind`; the statement form stays for cleanup-free types. Cleanup may block; cancellation is masked while it runs (Q14) | `task-lower` must see that a `drop` may yield | accepted (Q14 C, M7) |
| M8 | Unwinding past unwaited tasks: cancel the children, then the runtime joins them (`task.abandon`, the unwind edge of `wait` and `join`) | Q16 | accepted (Q16 B, M8) |
| M9 | A panic inside `Cleanup` during unwinding is task-level: R0009, the first panic stays primary, remaining cleanups still run; cleanup blocks stay `nounwind` for the verifier | Q13 | accepted (Q13 C, M9) |
| M10 | Open: a `Task` stored in a list or record. Proposal: the drop glue of any type that contains a `Task` is a terminator `drop` that joins it, as if it had `Cleanup` | V17 covers locals only | proposal |
| M11 | Proposal for 03 §3.4: `move-elim` also turns a `move` of a `Copy` type into a `copy`; a new `param-by-value` pass passes small `Copy` parameters by value and rewrites `copy (*_1)` to `copy _1` | 13.6 leaves both to the optimizer | proposal |
