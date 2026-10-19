# Diagnostics

The `Diagnostic` record, the code catalog, how a message is worded and
shown, `verse explain`, run-time panic text and exit statuses.

## Read first

1. [02 §2.8 Diagnostics](../../../../docs/architecture/compiler-architecture.md#28-diagnostics):
   the record.
2. [16 §16.1 Rules](../../../../docs/reference/diagnostics-catalog.md#161-rules):
   the ten catalog rules.
3. [16 §16.2 Families](../../../../docs/reference/diagnostics-catalog.md#162-families)
   and [16 §16.2.1 Families by pass](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass).
4. [16 §16.4 How a code is shown](../../../../docs/reference/diagnostics-catalog.md#164-how-a-code-is-shown)
   and [16 §16.5 `verse explain`](../../../../docs/reference/diagnostics-catalog.md#165-verse-explain).
5. [16 §16.6 Run-time panics](../../../../docs/reference/diagnostics-catalog.md#166-run-time-panics)
   and [16 §16.6.1 Exit statuses](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses).
6. [08 §8.7 Vocabulary and wording](../../../../docs/architecture/debugging.md#87-vocabulary-and-wording)
   and [`events.toml`](../../../../docs/reference/events.toml).
7. Spec [§16.2 Compiler messages](../../../../docs/language/16-tooling.md#162-compiler-messages)
   and [§16.5 Lints](../../../../docs/language/16-tooling.md#165-lints).
8. [10 §10.5 Diagnostics quality tests](../../../../docs/architecture/testing-and-verification.md#105-diagnostics-quality-tests).

## What it does

Diagnostics are data first, text second. Every stage produces one record
([02 §2.8](../../../../docs/architecture/compiler-architecture.md#28-diagnostics)):

| Field | Holds |
|-------|-------|
| `code` | stable, documented, never reused, such as `V0412` |
| `severity` | `error`, `warning` or `note` |
| `message` | plain language, in the style of spec §16.2 |
| `primary` | a span and a label |
| `secondary` | more spans with labels |
| `rule` | the spec section that defines the rule |
| `help` | suggestions, with machine-applicable edits when safe |
| `facts` | structured data: move site, branch condition, bound name |

- The terminal renderer, the language server, `verse check --json` and the
  debug adapter all consume the same record
  ([02 §2.8](../../../../docs/architecture/compiler-architecture.md#28-diagnostics)).
- `verse explain V0412` prints the long form
  ([16 §16.5](../../../../docs/reference/diagnostics-catalog.md#165-verse-explain)).
- `verse-rt` reports the `R0xxx` panics
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
- The crate layout of
  [02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)
  names no crate for the record or the renderer.

## How it works

**Codes.** `V` and four digits; the first two digits are the family. Errors
and warnings share the number space. Debugger codes are `D0xxx`, panic codes
`R0xxx`. Lints are warnings with `V` codes in the family of their topic
([16 §16.1](../../../../docs/reference/diagnostics-catalog.md#161-rules)).

| Family | About |
|--------|-------|
| V01xx, V02xx | source text, syntax |
| V03xx, V04xx | names and imports, types |
| V05xx, V06xx, V07xx | ownership, patterns, resources and tasks |
| V08xx, V09xx, V10xx, V11xx | packages, capabilities, foreign calls, command line |
| V99xx | a compiler bug: V9901 a pass failed, V9902 the verifier rejected its output |

Source: [16 §16.2](../../../../docs/reference/diagnostics-catalog.md#162-families).
A new code goes in the family of the pass that finds the problem
([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
V0701, V0704 and V0706 are intentionally unassigned
([16 §16.5](../../../../docs/reference/diagnostics-catalog.md#165-verse-explain)).

**Wording.** The message is one sentence of at most 15 words, in the form
"`x` is not Y" or "cannot Z because W", in the program's words, never naming
an internal stage. At most one primary label and three `help:` lines. A rule
number is shown as `rule: §4.9`, never as a link. Every `{name}` placeholder
is a key of `facts`
([16 §16.1](../../../../docs/reference/diagnostics-catalog.md#161-rules)).
Why: the same fact gets the same words in the compiler and the debugger
([08 §8.7](../../../../docs/architecture/debugging.md#rules)).

**Vocabulary.** Say "binding", "moved", "copied", "dropped",
"`none`", "error value", "task". Do not say "variable", "reference",
"cloned", "freed", "null", "exception", "thread". The full table is
[say and don't say](../../../../docs/architecture/debugging.md#words-say-and-dont-say).
A diagnostic that describes an event uses that event's template from
`events.toml`, and the compiler fails to build if an event has no entry
([08 §8.7](../../../../docs/architecture/debugging.md#rules)).

**Layout.** The first line is the message with no code in it. The last line
is ``explain: V0412 (run `verse explain V0412`)``, after every `help:` line.
JSON carries the code as a separate `code` field. A typo suggestion uses edit
distance 2 and shows only when exactly one name is that close. The V0511 note
shows only with `--explain-ownership`
([16 §16.4](../../../../docs/reference/diagnostics-catalog.md#164-how-a-code-is-shown)).

**Long form.** `verse explain` prints four headings in order: what happened,
why Verse works this way, ways to fix it, example. At most 25 lines, one
example that compiles
([16 §16.5](../../../../docs/reference/diagnostics-catalog.md#165-verse-explain)).

**Lints** are compiler warnings with the same codes and layout. A lint never
stops a build, has a machine-applicable fix or none, and has no
configuration ([§16.5](../../../../docs/language/16-tooling.md#165-lints)).

**Panics.** The text is fixed in the catalog, so the VM and every compiled
tier print identical words. A panic never shows an address, a stage name or
a tier. Each MIR panic kind maps to one `R` code and its facts; a kind with no
row is a compiler bug reported as V9902
([16 §16.6](../../../../docs/reference/diagnostics-catalog.md#166-run-time-panics)).

**Exit statuses.** 0 normal; 1 a top-level `try` or an unwaited task failed;
2 source errors, a capability check failed before start, or a plugin
library that is missing, fails to build, lacks a symbol or is static only
(V1007 to V1009, V1011), found before the first statement; 3 a bug in Verse; 70 a deadlock (R0008); 101 a panic
([16 §16.6.1](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses)).
After `verse check` exits 0 on a file, running that file with the same
sources, `verse.toml`, `verse.lock`, compiler and target never stops with
V9902, and never exits 2 except for a capability refused by a `--deny` flag
of that run, or a plugin library that is missing, fails to build, lacks a
symbol or is static only (V1007 to V1009, V1011). Verification after an
optimizer pass, which runs only in debug builds of `verse` and in CI, is
outside this rule.

## Rules that must hold

- A code never changes meaning and is never reused
  ([16 §16.1 rule 1](../../../../docs/reference/diagnostics-catalog.md#161-rules)).
- Every error has a place, a reason and, where one exists, a fix (rule 4).
- Each pass reports only the codes of its row in the families-by-pass table.
  A pass with no row, or a family with no pass, is an error in that table
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
- Every ownership error carries the move site, the branch condition of a
  one-path move, the rule and
  the fix ([02 §2.4](../../../../docs/architecture/compiler-architecture.md#24-the-ownership-pass),
  [§18.10](../../../../docs/language/18-design-notes.md#1810-why-no-garbage-collector)).
- A `V99xx` message ends with "this is a bug in Verse, not in your program"
  ([16 §16.6.1](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses)).
- MIR verifier rule ids `V1`..`V24` are not diagnostic codes
  ([13 §13.5](../../../../docs/architecture/mir-specification.md#135-verifier-rules)).
- An `exit(code)` the program calls itself is never changed
  ([16 §16.6.1](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses)).

## What depends on it

- Every stage emits records. See
  [parser-and-formatter.md](parser-and-formatter.md),
  [names-and-types.md](names-and-types.md) and
  [ownership-checking.md](ownership-checking.md) for the codes each owns.
- Editors and tools apply machine-applicable edits without parsing prose
  ([02 §2.8](../../../../docs/architecture/compiler-architecture.md#28-diagnostics)).
  A tool that reads text matches `^explain: (\S+)`
  ([16 §16.4](../../../../docs/reference/diagnostics-catalog.md#164-how-a-code-is-shown)).
- Differential tests compare panic text and exit status across tiers
  ([10 §10.3](../../../../docs/architecture/testing-and-verification.md#103-differential-testing)).
- Scripts read exit statuses to tell causes apart
  ([16 §16.6.1](../../../../docs/reference/diagnostics-catalog.md#1661-exit-statuses)).
- [19 §19.5](../../../../docs/reference/cli-and-manifest.md#195-where-each-code-appears)
  lists where each package and capability code appears; keep it in step.

## After a change

1. Give each code three tests: a golden test of the exact rendered text, a
   test that the `help:` edit applies and the program then compiles (when it
   is machine-applicable), and a test that `verse explain` has text
   ([16 §16.1 rule 9](../../../../docs/reference/diagnostics-catalog.md#161-rules)).
   CI fails if a code has no test or no `verse explain` text
   ([10 §10.5](../../../../docs/architecture/testing-and-verification.md#105-diagnostics-quality-tests)).
2. Add a must-fail program under `tests/must-fail/` with the expected code
   when the code rejects a program
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)).
3. Add the code to the catalog table of its family, with message, label,
   help and rule. A code that describes an event uses that event's template
   from `events.toml`
   ([16 §16.1 rule 7](../../../../docs/reference/diagnostics-catalog.md#161-rules)).
4. Wording or code changes are docs changes: type `spec` when they change
   the design, `docs` otherwise
   ([CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format)).
5. Run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
