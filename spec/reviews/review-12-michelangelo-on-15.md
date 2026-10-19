# Review 12: architecture 15 (target and ABI), wording, codes and user-visible rules (Michelangelo)

Verdict: **accepted as a draft**, with the rulings below. The layout, calling
convention and unwinding sections are Bob's to judge and I do not question them.
I checked what a user can see: messages, exit statuses, the target file and its errors.

## Rulings on the open items

| Item | Ruling |
|------|--------|
| P15-4 `abort` loses `TaskPanicked` | Accepted: the build fails when a `spawn` is reachable under `abort`. Code **V0910** (16). |
| P15-11 codes for target-file errors | Allocated: V0911 (two keys that cannot be used together: backend and object format, tasks and threads, atomics and threads, unwind and backend, pointer width, endian and abi against the triple), V0912 (a key needs a feature the target lacks: stackful tasks without a context-switch row, `wasm-eh` without the feature), V0913 (no provider for a capability), V0914 (`[memory]` invalid), V0915 (no target of that name). 15.9.4 should name the code on each row. |
| P15-13 `exit(code)` range | A code outside 0 to 255 is a panic, **R0013**, on every platform. Silent truncation would make `exit(256)` report success, and the same call must mean the same on every target. **Settled (final, three-way tiebreak by Bob):** `exit(code)` runs no `Cleanup`. It ends the process immediately from any task, flushes standard output and error, does not cancel other tasks, and takes 0 to 255 (R0013 outside). It is the one call that must work from a cancelled or deadlocked task; a program that needs its cleanups returns from `main` or ends with a failed `?`. A cleanup-aware exit, or `abort_now` as the immediate one, is a 0.2 candidate listed in 12. Spec §12.5 and §15.1 say "runs no `Cleanup`". |
| P15-16 status for a trap or `unreachable` | Where the runtime is the cause, status **3** ("a bug in Verse", 16 §16.6.1). A host trap that Verse cannot control stays host-defined. |
| P15-23 stack check in the prologue | Accepted. The message is R0004 in every case, including the guard-page backstop ("recursion is too deep in task N"; exit 101); change 15.6.4 to say R0004, not its own text. Bob fixes 05 §5.3. |
| P15-24 precedence of statuses | Accepted as written. |
| P15-10, P15-14 | Accepted; the `exit(err)` fallback warning needs a code if it is shown to users: V0816-style plugin warnings are not the right family, so add it as a `note` in 16 when 15 says its wording. |
| Naming of Wasm targets | Use `wasm32-wasip1`, `wasm32-wasip2` and `wasm32-unknown-unknown` everywhere. 04 and 19 (Picasso) follow 15. |

## Fixes to make in 15

1. 15.8.1 row `1` for a failed capability check (V0901, V0902, `--deny`): this is source or configuration error before start, so status **2** (16 §16.6.1), not 1. Add rows for **2** and **3**.
2. 15.8.1 row `101`: "a guard-page fault" must say it prints the R0004 text.
3. Every user-visible sentence in 15 that quotes a message should quote the catalog (16) text and code instead of its own wording.
4. 15.9.4: add the V09xx code to each rule.
5. 15.3.3 field order: L4 reorders record fields by alignment. Say in one sentence that the debugger and `verse.api` always show fields in declaration order, so layout never leaks into what a user sees.

## What I did not review

15.3 to 15.7 mechanics, 15.10 capability table and 15.12 conformance plan: no user-visible wording in them beyond the items above.


Confirmed (Bob's item): a VM `TRAP` or a failed run-time capability re-check is a bug in Verse and exits 3 (16 §16.6.1).

## Final check of 15 (Michelangelo)

15, 05 and 04 pass my check. `exit`, statuses 2 and 3, R0004, R0013 and V0910 to V0915 agree with 16 §16.6.1, spec §12.5 and §15.1.

**Rulings, final:**

- **Q13 is C.** A panic in a `Cleanup` is reported as R0009. The first panic stays primary, the remaining `Cleanup` calls of that task still run, and the task ends with status 101 as in 15.8.1.
- **Q17 is C, status 70.** A deadlock is R0008 and exits 70. Precedence is the one in P15-24.

**Follow-ups, none of them blocking:**

- 12 marks Q13 and Q17 as closed (Picasso).
- 15: the `70` row of 15.8.1 changes its Label from *proposal* to *fact*. The `Q13 recommended` and `Q17 ... still open` cells change to "closed". The P15-2 row is closed.
- 15 (the item at about line 1444) says 19 §19.3 uses `wasm32-wasi`. 19 now uses `wasm32-wasip1`, so the note is deleted.
- 12 carries the cleanup-aware exit as a 0.2 candidate (P15-25).
