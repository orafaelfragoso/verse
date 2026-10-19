# Review 10: architecture 13 (MIR specification) and 14 (bytecode and VM), by Picasso

Scope: internal consistency of 13 and 14; the language spec (moves and drops,
borrows, unwinding, Map/Set order, Text indexing, channels and tasks, overflow);
02, 03, 04, 05, 09, 10, ADR-0002/0003/0007/0008, 08 and `vocab/events.toml`;
16 (V9902, R0001..R0009); the Bob rulings Q13 to Q19 in 12; the four worked
lowerings; links. Files were re-read from disk after the Q13-Q19 and 03 updates.
I did not edit 13, 14 or any other file.

**Verdict: publish as DRAFT, not READY.** Both docs already say "draft 1" and
the design is sound (one MIR, explicit edges, the VM as oracle). But B1 to B3
make the verifier or the panic text unimplementable as written, so they must be
fixed before anyone writes `verse-verify` or the VM panic path. Counts:
**3 blockers, 19 should-fix, 8 nits.**

Checks that came out clean: **links** (24 links in 13 and 14, files and
anchors, 0 broken); all four examples' **spans** match their snippets column by
column; **block numbering** (P3) matches every listing; **V9902 exists** in 16
(see S9 for its wording); ADR-0003 and ADR-0007 now agree with 13.9 and 14.7;
14.10's step definition matches Q19; 14.6/14.9 capability flow matches P6-5.

## Blockers

**B1. A `panic` carries no operands, so R0001..R0006 cannot be printed.**
13.2 grammar line 87 `"panic" PKIND { atom } unwind`: `atom` is never defined.
13.8 prints `panic overflow add Int unwind continue` with no operands (13 Ex1
bb5, bb6). 16 §16.6 messages need values: R0001 `{a} {op} {b}`, R0002 `{i}` and
`{n}`, R0003 `{a}`, R0005 `{x}` and `{msg}`, R0006 `{expr}`. 14.5 says `PANIC`
takes "a pooled operand list" and the record has operands, so the VM expects
what the MIR does not provide. Neither 13 nor 14 mentions any R code.
Fix: define `atom = operand | STRING`; print `panic overflow add Int(copy _1, Int(1))`
style operand lists; add to 13.4 a table PKIND to R code and operands:
`overflow OP T` R0001 (a, b), `index_oob` R0002 (i, n), `div_zero` R0003 (a),
`expect_none` R0005 (expr text, optional msg), `assert` R0006 (expr text),
`stack_overflow` R0004 (task), `out_of_memory` R0007 (size). `slice_oob` and
`neg_exponent` (spec §3.8.4: negative run-time exponent panics) have no R code
in 16: ask Michelangelo for entries or fold them into R0002/R0001. Add R0008
(deadlock) and R0009 (panic in cleanup) as non-MIR kinds raised by the runtime.

**B2. V12 is unsound for negative indices, and the VM cannot implement it from MIR.**
13.5 V12 (line 238) accepts a dominating `lt(i, len p)`. MIR has only signed
`lt` (13.2 binop, line 77), so `i = -1` passes the fact and the unchecked load
reads out of bounds. 14.4/14.5 (lines 103, 133) say the VM uses `LT_U` for "a
negative index is a huge unsigned value", but a MIR `lt` cannot select `LT_U`
without changing the meaning of a user `lt`. Spec §9.1: negative indices are not
allowed (must panic at run time, R0002). Fix: add MIR binop `ult` (unsigned
compare, used only for bounds) and make V12 require `ult(i, len p)` (or both
`ge(i,0)` and `lt(i,len)`); update the V12 rejected example and 14.4 `LT_U`
mapping; compiled tiers then get the same check.

**B3. `drop` of a `Cleanup` type is a statement, but Q14 makes it a possible yield and unwind point.**
13.4 (line 165) and 13.10 M7 (line 554, still "Open") keep `drop` a statement
that "does not change the CFG". Q14 (C, recommended) says `drop` of a type with
`Cleanup` becomes a terminator with an unwind edge, and `task-lower` treats it as
a yield. As written, V14 (only call, spawn, yield, panic name an unwind edge),
V15 (cleanup blocks: no `yield`) and V18 cannot hold for user cleanup code that
panics or flushes. Fix: add `drop p [flag d] -> next bbN, unwind u` as the form
for types with `cleanup = true` in layout facts (plain statement otherwise);
inside `cleanup` blocks it may yield (Q14) and its unwind edge is `none` because
a second panic is task-level (Q13); relax V15 and extend V14/V18 accordingly;
14.4 `DROP` row and 14.7 switch-point list follow (see S4). Re-print Example 3
(bb4, bb5, bb7).

## Should-fix

**S1. Panic inside `Cleanup` still says abort (Q13).** 13.10 M9 (line 556):
"abort the process with status 101"; 14.8 "Panic during cleanup" (lines 225-226):
"print both panics, abort with 101". Q13 (C): task-level, the task ends as
panicked with the first panic primary, remaining cleanups still run, the second
panic is printed at once as R0009 and attached to `TaskPanicked` (`trace()` shows
both). Fix: rewrite both; 14.5 panic record gets a `secondary` list; the pending
unwind state in 14.7/14.8 must hold a panic list; the walk in 14.8 continues
after a cleanup panic; `.vsrec` `Finish` (14.10) outcome must be able to carry
two panics. Keep V15 (cleanup blocks stay `nounwind` for the verifier).

**S2. Deadlock exit status (Q17).** 14.7 Deadlock bullet (lines 199-201) "Exit
status 1 (proposal; the spec is silent)"; P14-7 (line 402) "exit 1"; 14.12 exit
table (line 354) lists 0, 1, 101, `exit`. Fix: status 70, text is R0008 (16 §16.6,
one label per blocked task), delete P14-7 or mark it resolved, add 70 to the
table. State that a program waiting on stdin, a socket or a timer is not deadlocked.

**S3. M8/Q16 and the wait edge.** M8 (line 555) is still "Open"; 14.8 "Abandon"
(lines 222-224) says "assumption". Q16 (B) rules it: cancel children, then wait.
Update both to "ruled, Q16". Also a real gap: in 13 Ex4 `run`, `yield wait(move _3)`
has `unwind bb5` which does not abandon `_3` (it was consumed by the operand),
so a cancelled or panicking parent parked in `wait` leaves the child running,
which breaks Q16 and spec §11.3. Fix: define that `WAIT` itself requests
cancellation of its operand task when it takes the unwind edge (add to 13.4
`yield wait` row and 14.8), or give the wait edge a `task.abandon` in a cleanup
block, and fix V17's text and the explanation under Ex4 ("`task` unbound or consumed").

**S4. Q14 in the VM: cancellation mask and blocking `DROP`.** 14.4 Drop row
(line 109) says "none (cleanup must not fail)" and nothing about blocking; P14-6
(line 401) says "until then `cleanup` must not block". 14.3 call-record flags and
14.7 task fields (line 173) have no cancellation mask. Fix: resolve P14-6 per
Q14; add `DROP` of a `cleanup` type to the 14.7 switch-point list; define the mask
as "`cancel_requested` is not delivered while any call record has the
`cleanup-running` flag" (the flag already exists in 14.3), delivered at the first
yield after the outermost cleanup returns; add a differential test (cancel
during a blocking cleanup).

**S5. Drop order of fields, elements, entries (Q15).** 13.4 `drop` row (line 165):
"drop fields in declaration order (assumption: spec is silent)". Q15 rules
declaration order for fields and tuple elements, index order for lists,
insertion order for Map and Set. Fix: cite Q15 in that row and in 14.4 `DROP`
(drop glue), and add the list/map/set orders; V14's "reverse declaration order"
applies to locals only, say so.

**S6. NaN rule is the old proposal (Q18).** 14.12 (lines 359, 363-364) "May
differ: NaN payload and sign" and P14-4 (line 399) "bit-identical except NaN
payload and sign". Q18 (C): every NaN becomes `0x7FF8000000000000` where bits are
observable (`to_bits`, hashing, byte serialization, `Show`); arithmetic untouched.
Fix: replace both with the Q18 rule, keep "bit for bit" for all else, and list in
14 which `verse-rt` entry points canonicalize (so all four tiers call them). Add a
differential test: `(0.0/0.0).to_bits()` on every tier.

**S7. "Step" is defined on unoptimized mono MIR, but debug builds run `simplify-cfg`.**
13.7 (lines 295-296): "Debug builds run no optimizer pass except `verify` and
`simplify-cfg`". Q19 and 14.10 (lines 295-298): a step is one statement or
terminator of the level-0 mono MIR, and replay runs at level 0. `simplify-cfg`
merges blocks and folds constant switches, so step numbers differ from a
"no pass" run. Fix: define "level 0" once in 13.7 as exactly `{verify,
simplify-cfg}` (and say Q19's "unoptimized" means that), or drop `simplify-cfg`
from debug builds. Say whether a live (non-replay) debug session uses level 0 too.

**S8. 14 does not reference R0001..R0009 and misquotes the source of panic text.**
14.2 (line 40) and 14.5 (line 140) say wording is rendered "from
`vocab/events.toml`"; that file has no panic templates (only `panicked` with a
`text` field); the wording is fixed in 16 §16.6. 14.3 (line 60) says
`stack_overflow` is "reported as 'recursion too deep in task N'" but R0004 is
"recursion is too deep in task {n}". 14.12 (line 353) excludes "the rendered
stack text of a panic" from the byte comparison, but 16 §16.6 says the VM and
every tier print identical text including the `in main` stack lines. Fix: point
14.2/14.5 to 16 §16.6, quote R0004 exactly, and align 14.12 with 16 (compare the
whole stderr text, with inlined frames reconstructed from spans per 13.7), or
ask 16 to say the stack lines are structured-only for comparison.

**S9. V9902 wording does not carry what 13.5 promises.** 13.5 (lines 216-219):
the error "names the rule, the function and the span". 16 V9902: "the verifier
found a broken rule in {stage}", with no rule or function placeholder; 16.1 rule
3 also forbids naming an internal stage and requires every fact to have a
placeholder. Fix: add `{rule}` (V1..V23) and `{function}` to V9902's message or
label in 16 (and keep `{stage}` as "after monomorphization" etc.), and say in
13.5 what happens when the verifier fails at run time in script mode (14.6
verifies at first call): V-code, exit status.

**S10. The 13.2 grammar does not match the printed form.** (a) Undefined
nonterminals: `case`, `atom`, `literal`, `path` (and how `::` and `.` mix:
13.8 prints `std.fs.create`, `File::write_line`, `Channel::new<Int>`). (b) Switch
labels print as `true -> bb1` and `0 -> bb2` but `const` is `TYNAME ( literal )`
(`Bool(true)`). (c) `discriminant` is `"discriminant" place` in the grammar but
Ex3 prints `discriminant(_4)`. (d) Mono headers: `tparam = IDENT [...]` cannot
hold `List<Int>` as the instance's type arguments (comment at line 50). (e) `type`
has no parenthesised grouping, yet Ex3 prints `aggregate case (Nothing | Error)#0`;
`( type )` parses as a 1-tuple. Fix: add the missing productions, pick one form
for each of (b)(c), add `"(" type ")"` grouping or print `aggregate case
#0 of Nothing | Error`, and add a parse-print test per listing in 13.8 (13.2
already requires `parse(print(m)) == m`, so run it on the examples).

**S11. `dbg` event words in the examples do not match the 13.4 table, V20 or `events.toml`.**
Ex3 bb2 `ref mut _6 ; exclusive "target"`: the 13.4 table (line 203) says no
event, `exclusive` is a state (`[states.exclusive]`), so V20 ("event words exist in
events.toml") rejects it. `borrowed` needs field `by` (the callee) but the `dbg`
clause for `ref` prints only the source name (Ex2, Ex3, Ex4: no `-> "copy"`).
The table says `drop` emits `cleaned_up` then `dropped`, but a `dbg` clause holds
one EVENT. `assert` has no row (`asserted` exists in events.toml). Fix: print
`ref mut` lines with an internal name only (`<excl> "target"`), add `-> callee` to
`borrowed`, say `cleaned_up` is derived from the `cleanup` layout fact at the
same pc, and add the `asserted` row.

**S12. Plain parameters: Ex1 contradicts 13.6.** 13.6 (line 259) and 13.7: a plain
parameter is `&T`, `borrowed`, and mono "unchanged". Ex3 follows it (`_1: &Text
= param borrowed`), but Ex1 prints `_1: Int = param owned "total"` for a plain
`total: Int`. Fix: pick one rule. Recommended: plain params are `&P` in all MIR
including `Int` (Ex1 reads `copy (*_2)`), and a later `param-by-value` pass (add
to 03 §3.4) turns small `Copy` params into by-value; otherwise generic-to-mono
would change `&T` into `T` for `T = Int`, which 13.6 forbids.

**S13. Capability and may-yield facts in lazy script mode, and stale 02 §2.6 text.**
V18/V19 need callee `mayyield` and `caps` (transitive), but 14.6 (lines 146-156)
compiles one instance at its first call and says "Startup cost scales with what
runs, not with what is reachable (02 §2.6)". 02 §2.6 now says `verse run` does a
reachability pre-pass first (cost scales with reachability) and 14.14 P14-3 is
marked resolved. 14.9 (line 238) still defines `required` as "over the code that
can run, P14-3". Fix: state that `caps` and `mayyield` of an instance come from
the pre-pass summary keyed by instance key (and that V18/V19 in lazy compile
check against it), fix the 14.6 sentence, and rewrite 14.9 to cite 02 §2.6.
Mention the pre-pass in the 14.13 hello-world row (20 ms includes it, warm cache).

**S14. Recording size: 14.10 contradicts 08 and is stale.** 14.10 (line 281):
"under 100 bytes per event, compressed (08 §8.5)... (08 does not say this; see the
inconsistency list)". 08 §8.5 now says "before compression" and puts results over
64 bytes in blob blocks. Fix: "before compression", drop the parenthesis, and
change the 14.13 row (line 388) the same way.

**S15. Implicit waits and conditional spawns have no MIR form.** Spec §11.3 waits
unwaited tasks at function exit. 14.7 says "compiler-emitted `WAIT`s", 13.5 V17
forbids `drop` of a `Task`, and `flag` is only defined for `drop`. A task spawned
in one branch needs a flag-controlled `yield wait` at every exit and on every
cleanup path. Fix: define in 13.4 how the MIR builder emits the exit waits (a
`yield wait` per live `Task` local, reverse declaration order, discard of the
`T | Error`; failure sets exit status per §11.3), and let `flag` guard a `wait`
or `task.abandon`. Add an example (a `spawn` in an `if`, or a bare `spawn
worker(...)` statement).

**S16. Text, Map, Set and slices in MIR.** 13.3 calls them "opaque", but `len q`
and `p[i]` (13.4, V12) are place operations. Spec §4.1: Text indexes by
character (`"héllo".len()` is 5, `.bytes().len()` is 6), and `text[i]` is not a
constant-time load; Map `m[k]` returns `V?`, not a place; slices `nums[1..]` are
views (§9.3) and `slice_oob` exists with no operation. 14.5 says "element access
after it is an unchecked `LD*` through the list pointer". Fix: say V12 and `p[i]`
apply only to `List<T>` and `Bytes` (incl. `text.bytes()`); `len` of `Text` is the
character count (with `bytes().len()` the byte count); Text char access, Map and
Set access and slicing are calls to `verse-rt` intrinsics with their own panic
sites; define the MIR type of a slice view.

**S17. Reassignment and drop flags have no rule and no example.** Spec §4.9:
"Reassigning frees the old value first" (after the right side is evaluated).
13.4 `p = op` row says only "store". Is a store into an initialised `needs_drop`
place an error unless a `drop` precedes it (V4/V5), or an implicit drop? Add the
rule, the `reassigned` lowering, and an example with `mut x` reassigned inside a
branch (drop flags, V5), a `List` index (V12), and a `match` (V13): none of
the rules V5, V12 and V13 has a worked example, and `flag`, `len`, `aggregate
closure`, `dyn #n` and loops are never printed. A fifth example (a `for` over a
list with an early `?`) would cover most.

**S18. 13 still calls its changes proposals; 03 has applied them.** 13 header
(lines 9-11), 13.1 and 13.10 (line 544: "All rows are proposals until 03 is
updated"). 03 §3.3 now has operands, `Unwind`, `ResumeUnwind`, `SetFlag`,
`Return(propagated)` and says 13 wins. Fix: mark M1..M6 "applied in 03", keep
M7..M9 open or ruled (S1, S3, B3). 03 still lists `CallDyn(...)` as a terminator
while 13 uses `call dyn #n`: change one.

**S19. `cast` kinds and a few ops are undefined.** `cast IDENT operand as type`
(13.2) names no kinds; 14.4 says `F2I` panics "per cast kind in MIR". Spec §3.8.4
only defines `to_int`/`round().to_int()` use. Define the kinds (`int_widen`,
`int_narrow_checked`, `f2i`, `i2f`, niche) and their panic or saturate rule, and
the effect of `shl`/`shr` with a count at or above the width. For `checked_pow`
with a negative exponent add a verifier fact like V11's (13.5 has none for
`neg_exponent`).

## Nits

- **N1.** 14.6 (lines 164-165): "04 §4.1 says 'per module'". 04 §4.1 now says per
  instance; delete the parenthesis.
- **N2.** 14.4 (lines 118-120): "a constraint that 03 and 13 do not yet state".
  03 pass 12 now states it; 13.3 and V10 still do. Add it to 13.3.
- **N3.** P2 (line 111) says locals are numbered in creation order. Ex3 interleaves
  temps and bindings (`_6` after `_3`.._5), but Ex4 `run` lists all four bindings
  before its temps although `_5` is created before `_1`. Renumber Ex4 or reword P2.
- **N4.** V11 and V13 speak of a `switch` "on `eq(d, 0)`" and "on `discriminant p`",
  but the listings switch on a temp (`_3`, `_5`, `_10`). Say "a temp assigned once
  from the test, unchanged since" (the V3 temp rule makes this checkable).
- **N5.** 14.7 (line 169) "a fourth lowering strategy"; ADR-0007 and 05 §5.3
  say "a fourth implementation, not a lowering". Use their wording.
- **N6.** 14.12 "Must be identical" is conditional on the seed in its lead-in but
  the rows read as unconditional; say that task-event order is compared only with
  `--seed` or a replay (05 §5.3 multi-core scheduler otherwise).
- **N7.** `yield pause` and `yield lock` are cancellation points in 13.4, but spec
  §11.5 lists only I/O, `wait`, `sleep`, `send`, `recv`; and no Verse operation
  produces `pause` (14.7 P14-2 is the only source). Say who emits it, and confirm
  `lock` with the spec owners.
- **N8.** Two wrong cross-references: 13.5 cites 10 §10.6 for "written independently
  of the passes" (10.6 is formal methods; the verifier row is in 10.1, the
  soundness fuzzer in 10.4); 13.6 (lines 273-276) says a later pass turns `move`
  of a `Copy` type into `copy`, but no pass in 03 §3.4 does that (`move-elim`
  removes copies). Name the pass or extend `move-elim`.

## Notes on the worked examples (item 4)

All four come from valid Verse per spec §3.8.4, §5.2, §6.7, §8.3, §11.1, §11.4
and §13.1 (`return 0 if count == 0`, `take x: T` with `T: Copyable`,
`with create(path)? as mut target`, `Channel.new<Int>(capacity: 1)`, `spawn`,
`rx.recv()` returning `Int?`, `_ = task.wait()`). Remaining mismatches with the
doc's own rules are S10 (grammar), S11 (events), S12 (param mode), N3, N4, S3
(`wait` edge) and B3 (Ex3 `drop`). Ex4's `square_one` is a fragment by its own
label, so the rest is not checkable. One open check for Bob: Ex3 uses a `with`
statement as the function's final value; confirm spec §5.1 lets a block-ended
`with` supply the implicit `Nothing`.

## Top findings to fix first

B1 (panic operands and R-code mapping), B2 (V12 unsigned), B3 (drop terminator),
then S1+S2 (Q13/Q17 still say abort and exit 1), S3 (wait edge).
