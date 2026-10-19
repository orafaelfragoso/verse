# 16. Diagnostics catalog

This file assigns a stable code to every diagnostic the 0.1 toolchain gives,
and fixes its wording. It builds on 02 §2.8 (the `Diagnostic` record), 08 §8.7
(vocabulary and the "don't say" table), spec §16.2 (the style) and
`events.toml`.

## 16.1 Rules

1. **A code never changes meaning and is never reused.**
2. **Codes are `V` and four digits.** The first two digits are the family
   (16.2). Errors and warnings share the number space; the record's `severity`
   says which one it is. Debugger codes are `D0xxx` ([`events.toml`](events.toml), 08 §8.7). Run-time panic codes are `R0xxx` (16.6). Lints are
   compiler warnings and use `V` codes in the family of their topic (spec §16.5).
3. **The message says what is wrong, in the program's words.** It names the
   binding, the type or the rule, never an internal stage. It does not use any
   word from the "don't say" table of 08 §8.7.
4. **Every error has a place, a reason and, where one exists, a fix.** The
   place is the primary label. The reason is one short line under the label.
   The fix is a `help:` line; a machine-applicable fix also carries an edit.
5. **At most one primary label and three `help:` lines.** More than three
   fixes is a sign the message should be two diagnostics.
6. **The message is one sentence of at most 15 words**, in the form "`x` is
   not Y" or "cannot Z because W". Long explanation goes to `verse explain`.
7. **The same fact has the same words everywhere.** A move is "moved" in a
   compile error and in the debugger; a diagnostic that describes an event
   uses that event's template from `events.toml`.
8. **Where a rule number exists it is shown**, as `rule: §4.9`, never as a
   link. `verse explain Vxxxx` prints the long form: what happened, why the
   language works this way, two ways to fix it, and one correct example.
9. **Each code has three tests:** a golden test of the exact rendered text, a
   test that the `help:` edit applies and the program then compiles (when the
   fix is machine-applicable), and a test that `verse explain` has text.

10. **A placeholder is a fact key.** Every `{name}` in a message, label or help is
    exactly a key of the diagnostic's `facts` (02 §2.8). Facts are text, a
    position, or a whole number; a count is always a number, never text. A test
    fails if a placeholder has no fact or a fact has no placeholder (the
    positions that only a label uses are the exception and are listed with
    the entry).

## 16.2 Families

| Family | Range | About |
|--------|-------|-------|
| Source text | V01xx | tabs, indentation, text and number literals, comments |
| Syntax | V02xx | the grammar: blocks, parentheses, keywords |
| Names and imports | V03xx | unknown names, hidden names, import lines, visibility |
| Types | V04xx | mismatches, missing annotations, optionals, errors as values |
| Ownership | V05xx | moves, views, `mut`, borrows, `Copy` bounds |
| Patterns | V06xx | `match`, `is`, destructuring |
| Resources and tasks | V07xx | `Cleanup`, `with`, `spawn`, channels, locks |
| Packages and API | V08xx | `verse.toml`, `verse.lock`, `verse.api`, versions |
| Capabilities and targets | V09xx | grants, targets, `--no-heap`, reachability |
| Foreign calls | V10xx | `extern` blocks and plugins (07) |
| Command line | V11xx | the arguments of `verse` itself |
| Run time (panics) | R0xxx | overflow, bounds, division by zero, stack, memory (16.6) |
| Internal | V99xx | a compiler bug; the message asks for a report |

`V0412` (the optional-used-as-plain example in spec §16.2 and 02 §2.8) keeps
its number.

### 16.2.1 Families by pass

Each stage of [02 §2.2](../architecture/compiler-architecture.md#22-stage-contracts)
and each optimizer pass of [03 §3.4](../architecture/ir-and-passes.md#34-pass-list)
reports the codes of its row. V99xx belongs to every pass: V9901 when the pass
fails, V9902 when the verifier rejects its output. A pass whose only row entry
is V99xx reports nothing else. The last three rows are not compiler passes;
they name the crate that reports the codes (02 §2.9). A new code goes in the
family of the pass that finds the problem; a pass with no row, or a family
with no pass, is an error in this table.

| Pass | Codes |
|------|-------|
| Lexer | V01xx |
| Parser | V02xx |
| AST lowering, HIR lowering, MIR build, Monomorphization | V99xx |
| Name resolution | V03xx; V1006 (an `extern` block names no plugin) |
| Type inference | V04xx, V06xx; V1002, V1003, V1010 (foreign signatures) |
| Ownership | V05xx, V07xx |
| Optimizer, `verify`, `simplify-cfg`, `inline`, `sroa`, `const-prop`, `fold`, `move-elim`, `copy-elide`, `range-analysis`, `devirtualize`, `drop-opt`, `dce`, `layout`, `refcount-opt` | V99xx |
| `task-lower` | V0909 |
| Reachability and capability check | V0901 to V0905, V0908, V0910, V0916, V0917 |
| Backends | V1005, V1007 to V1009, V1011, V1012 (linking foreign code, 07) |
| `verse-pkg` | V08xx; V0906, V0907 (target variants in the manifest) |
| `verse-cli` | V11xx; V0911 to V0915 (target files) |
| `verse-rt` | R0xxx (run time, 16.6) |

## 16.3 Catalog

Each entry: code, severity, message, label under the primary position, help
lines, rule. `{…}` is filled by the compiler. Examples show the label and the
`help:` text only; the full layout is the one of spec §16.2.

A rule given as a section only (for example `§7`) names the section, not a
numbered rule. It is refined when the conformance suite
([10 §10.2](../architecture/testing-and-verification.md#102-specification-conformance))
gives every rule an identifier.

### V01xx Source text

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0101 | error | a tab is not allowed in indentation | this line starts with a tab | indent with four spaces (applied by `verse fmt`) | §3.3 |
| V0102 | error | this line is indented by {n} spaces, not a multiple of 4 | expected {m} or {k} spaces | indent with {m} spaces | §3.3 |
| V0103 | error | this text is never closed | the text starts here | add the closing `"` | §3.7 |
| V0104 | error | `\{c}` is not an escape | not one of `\n \t \\ \" \{ \}` | write `\\` for a backslash | §3.7 |
| V0105 | error | a raw text must start on its own line | the opening `"""` ends its line, but this one is followed by text | move the text to the next line | §3.7 |
| V0106 | error | this interpolation is nested too deeply | text inside `{…}` inside `{…}` | bind the inner text to a name first | §3.7 |
| V0107 | error | `{text}` is not a number | cannot read this as {Int or Float} | write digits, with `_` between groups | §3.7 |
| V0108 | error | the number {text} does not fit in `Int` | the largest `Int` is 9,223,372,036,854,775,807 | use a `Float`, or a sized type from `math` | §4.1 |
| V0109 | error | this file is not valid UTF-8 | the first bad byte is here | save the file as UTF-8 | §3.1 |

### V02xx Syntax

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0201 | error | expected {what} here | found {found} | | §3 |
| V0202 | error | this block is empty | a block needs at least one line | write `pass`, or remove the line | §6.6 |
| V0203 | error | `{kw}` is a keyword and cannot be a name | rename it | try `{kw}_` or a more specific name | §3.6 |
| V0204 | error | `let` is not part of Verse | a new binding is written `x = value` | write `{name} = {value}`; use `mut {name} = {value}` if it changes | §3.9 |
| V0205 | error | `a?.b` is not part of Verse | a plain dot already looks through an optional where it is used | write `(a?).b`, or `a.b otherwise {fallback}` | §8.1 |
| V0206 | error | `from` must come last in an import | the item list goes first | write `import {names} from {module}` | §12.2 |
| V0207 | error | an import must be at the top of the file | imports come right after the doc comment | move this line up | §12.2 |
| V0208 | error | a long import needs parentheses | the list continues on the next line | wrap the names in `( … )` | §12.2 |
| V0209 | error | `{op}` cannot be in the right operand of `otherwise` without parentheses | `otherwise` binds more loosely than `{op}` (second form, when the operand holds `return`, `break` or `continue`: "an exit must be the whole right operand") | write `({left} otherwise {right}) {op} {rest}` or `{left} otherwise ({right} {op} {rest})` (shown when the operand holds no exit) | §3.8.2 |
| V0210 | error | `{}` does not say whether it is a map or a set | this could be an empty map or an empty set | write `{:}` for an empty map, or `set{}` for an empty set | §3.7 |

### V03xx Names and imports

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0301 | error | `{name}` is not defined | no binding, function, type or import has this name | did you mean `{near}`? (shown only when one name is within edit distance 2) | §3.9 |
| V0302 | error | `{name}` already exists in this scope | first bound at {position} | use a different name, or write `{name} = …` without `mut` only if the first one is `mut` | §3.9 |
| V0303 | error | `{name}` is not `mut`, so it cannot be reassigned | bound without `mut` at {position} | write `mut {name} = …` at the first binding | §3.9 |
| V0304 | warning | `{name}` hides the prelude `{name}` | this import is on the line below the doc comment | import it `as {other}`, or keep it if you meant to replace the prelude name | §3.9 |
| V0305 | warning | `{name}` hides the import `{name}` | the import is used in this function | rename the local | §3.9 |
| V0306 | warning | `{name}` is never used | bound here | delete it, or start the name with `_` | §3.9 |
| V0307 | warning | `{name}` is never used | imported here; unused imports cannot be silenced | delete the name or the line | §12.2 |
| V0308 | error | `{name}` is private to `{module}` | `{module}` does not `export` it | ask the owner to `export` it, or use `{public_name}` | §12.3 |
| V0309 | error | `{module}` has no item called `{name}` | it exports {list, up to 5} | did you mean `{near}`? | §12.2 |
| V0310 | error | modules `{a}` and `{b}` import each other (other forms: "module `{a}` imports itself through `{b}`" for a cycle of three or more modules, "module `{a}` imports itself" for a self-import) | `{back_edge}` imports `{a}` | move what they share into a module none of them imports | §12.2 |
| V0311 | error | `{case}` is a case of `{Type}` | cases are not imported with the type | write `{Type}.{case}(…)`, or add `import {case} from {module}` | §12.2, §4.4 |
| V0312 | error | a renamed type needs a capitalised name | `{x}` is a value name | write `as {X}` | §3.5 |
| V0313 | warning | new binding `{name}` looks like the `mut` binding `{near}` | this creates a new binding | did you mean to reassign `{near}`? `{near} = {value}` | §3.9 |
| V0314 | error | `{name}` is already a type, so it cannot be a variant | a variant of `{type}` cannot reuse the name of a type in scope | to give it another name, write `alias {type} = {rhs}` (shown when every variant is an existing type); otherwise rename the variant | §4.4 |
| V0315 | error | cannot mix the existing `{existing}` and the new `{new}` in one tuple | a tuple of existing names reassigns them all, so every name must be a `mut` binding | bind `mut {new} = …` before this line | §3.9 |

### V04xx Types

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0401 | error | expected `{T}`, found `{U}` | this is `{U}` | convert it with `{conversion}` (shown when one exists) | §4.2 |
| V0402 | error | cannot tell the type of `{name}` | nothing here decides it | annotate it: `{name}: {guess}` (shown when one candidate exists) | §4.2 rule 4 |
| V0403 | error | `{f}` needs an annotation because it calls itself (second form: "calls itself through `{g}`"; the label marks every call in the cycle) | recursion hides the return type | write `fn {f}(…): {T}` | §4.2 rule 3 |
| V0404 | error | `{f}` was called with {n} arguments, but takes {m} | | the parameters are `{signature}` | §5.3 |
| V0405 | error | `{f}` has no parameter called `{p}` | the parameters are {list} | | §5.3 |
| V0406 | error | `{T}` has no field or method `{name}` | in a generic call: "needed by `{f}`, called here", with a second label on the line inside `{f}` | did you mean `{near}`? | §4.3 |
| V0407 | error | `{T}` does not implement `{Trait}` | needed by `{f}`, called here; second label on the line inside `{f}` | add `impl {Trait} for {T}` (shown for traits that cannot be derived) | §10.1 |
| V0408 | error | `{T}` cannot be compared with `{U}` | `==` needs two values of one type | convert one side | §9.6 |
| V0409 | error | `{x}` does not fit in `{T}` | the largest `{T}` is {max} | use a larger type | §4.1 |
| V0410 | error | `Int` and `Float` are not mixed implicitly | | write `{x}.to_float()` or `{y}.round().to_int()` | §4.1 |
| V0411 | error | a fallible value cannot be ignored | `{f}` returns `{T} \| Error` | handle it with `?`, `otherwise` or `match`, or discard it with `_ = …` | §8.4 |
| V0412 | error | `{name}` might be missing | this is `{T}?`, not `{T}` | provide a fallback: `… otherwise {default}`; stop early: `…?`; branch: `if … is {name}?` | §8.1 |
| V0413 | error | `?` can only be used in a function that returns an optional or `Error` | `{f}` returns `{T}` | change the return type to `{T}?` or `{T} \| Error` | §8.3 |
| V0414 | error | `otherwise` needs an optional or fallible value on its left | this is `{T}`, which is always there | remove `otherwise` | §3.8.2 |
| V0415 | error | `{x}` is `{T}?` and cannot be used as `{T}` here | the dot looks through an optional only when the chain ends in `otherwise`, `?`, `is`, `match`, or an optional-typed position | add one of those at the end of the chain | §8.1 |
| V0416 | error | the `Error` of `{f}` is not handled here | `{f}` can return `{E}` | add `?`, `otherwise` or a `match` arm for `{E}` | §8.4 |
| V0417 | error | `{x}` is not a `Text`, so it cannot be indexed by character | | use `.bytes()` for a byte view | §4.1 |
| V0418 | error | missing type for a public item | `export` items need types in `verse.api` | annotate the parameters and the result | §16.4 |
| V0419 | error | `{f}` needs a new copy for every depth of this call | `{f}` calls itself with `{T}` wrapped in `{W}` | annotate the parameter with one concrete type | §4.2 rule 1 |

### V05xx Ownership

Messages carry the structured explanation required by spec §18.10. The labels
below are the minimum; the compiler adds the second label ("moved here") whenever the
fact has a position.

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0501 | error | `{name}` was moved and cannot be used | moved to {to} at {position} | use `{name}.copy()` before the move, or move it last | §4.9 |
| V0502 | error | `{name}` is a view and cannot be stored | it is a view of `{of}`, which {reason} | store `{name}.copy()` | §4.9 |
| V0503 | error | a view cannot be returned | `{name}` is a view of `{of}` | return `{name}.copy()`, or take the parameter with `take` | §4.9 |
| V0504 | error | a view cannot be sent to a task | `{name}` is a view of `{of}` | send `{name}.copy()`, or move it with `take` | §11.1 |
| V0505 | error | `{name}` is borrowed exclusively by this call and cannot be used in it again | the first use is here | call it with a copy | §4.10 |
| V0506 | error | `{name}` is not `mut`, but `{f}` needs to change it | `{f}` takes `mut {param}` | bind it as `mut {name}` | §4.10 |
| V0507 | error | `{T}` cannot be copied | `{why}` | move it instead | §4.9 |
| V0508 | error | `{f}` needs `{T}` to be `Copyable`, because it returns `{x}` | the bound is inferred from the body | return `{x}.copy()`, or take the parameter with `take` | §4.2 rule 9 |
| V0509 | error | this call needs `{T}` to be `Copyable` | `{T}` contains `{part}`, which is not | pass a type that can be copied | §4.2 rule 9 |
| V0510 | error | `{name}` is moved in one branch and used after it | moved at {position} only when `{cond}` | move it in both branches, or copy it | §4.9 |
| V0511 | error | a chain bound to a name is collected here | `{name}` holds the whole result | no change needed; this note explains why `{name}` is owned | §9.5 |
| V0512 | warning | `{name}` is only a view because `{other}` uses it later | | move the later use first, or copy | §4.9 |
| V0513 | error | `{name}` is a borrowed function and cannot outlive this call | `{name}` is a parameter without `take`; it is {use: stored, returned, sent to a task, or captured by an escaping lambda} here | declare the parameter as `take {name}` | §5.4 |
| V0514 | error | `{name}` is not `mut`, so `{place}` cannot be changed | bound without `mut` at {position} | write `mut {name} = …` at the first binding | §4.9 |

V0511 is a `note`, not an error (its severity is `note`, per 02 §2.8).

### V06xx Patterns

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0601 | error | this `match` does not cover {missing} | add an arm for {missing} | add `{pattern} -> …` or a final `_ -> …` | §7.2 |
| V0602 | error | this arm can never match | an earlier arm already matches everything it matches | delete it; did you mean the case `{Name}`? (shown when the earlier arm's name is a case of the matched type in lowercase) | §7.2 |
| V0603 | warning | `{name}` looks like the case `{Name}` | in the last arm a lowercase name binds anything | write `{Name}` if you meant the case | §7.2 |
| V0604 | error | `{pattern}` is not a case of `{T}` | the cases are {list} | | §7 |
| V0605 | error | a typed pattern is only allowed at the top of an arm | `{x}: {T}` is inside parentheses | write `field: {x}: {T}` | §7.3 |
| V0606 | error | this destructuring has {n} parts, but the value has {m} | | | §7.7 |
| V0607 | error | `{pattern}` is a type, not a case | `{Type}` cannot be used as a pattern here | write `field: {x}: {Type}` | §7.3 |

### V07xx Resources and tasks

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0702 | error | `{expr}` is `{T} \| Error`, which cannot be cleaned up | `with` needs the resource itself | add `?` to propagate the error | §13 |
| V0703 | error | a task cannot use `{name}` | it is a view | send a copy with `.copy()`, or move it in | §11.1 |
| V0705 | error | a `Receiver` has one owner | `{name}` was moved at {position} | give the receiver to one task | §11.4 |
| V0707 | error | `{name}` is locked here and cannot be used outside `with` | `Guarded` values are read inside the block | read it as `with {g}.lock() as {value}` | §11.4 |
| V0708 | error | `spawn` cannot pass `{name}` as `mut` | a task cannot give an exclusive borrow back to the spawner | move `{name}` in and get it back from `wait()`, or share it through a `Guarded` | §11.1 |

### V08xx Packages and API

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0801 | error | `{key}` is not a key of `[{section}]` | unknown keys are errors | did you mean `{near}`? | 19 §19.3 |
| V0802 | error | the package name `{name}` cannot be imported | a name uses letters, digits and `_`, starting with a letter | write `{suggested}` | 21 §21.3 |
| V0803 | error | version {have} is too small for these changes; publish as {need} | {change summary} | set `version = "{need}"`, or keep the old signature and add a new function | 21 §21.6 |
| V0804 | error | version {req} of `{pkg}` does not exist | the newest release is {newest} | `{pkg} = "{near}"` | 21 §21.13 |
| V0806 | error | `{pkg}` {ver} does not match its checksum | the archive hashes to `{got}` | `verse lock --refetch {pkg}` | 21 §21.13 |
| V0807 | error | `verse.lock` is out of date | `{pkg}` asks for {req}, the lock has {have} | run `verse lock` (not allowed with `--locked`) | 21 §21.7 |
| V0809 | error | a path dependency cannot be published | `{pkg}` is `{ path = … }` | name a version as well | 21 §21.12 |
| V0810 | warning | `verse.api` changed | the new API adds {n} items | run `verse api update` | §16.4 |
| V0811 | error | `verse.api` differs from the source | `{item}`: file says `{a}`, source says `{b}` | run `verse api update` | §16.4 |
| V0812 | error | `{name}` is not a valid license | use an SPDX name such as `MIT` | | 19 §19.3 |
| V0813 | error | a yanked version cannot be chosen | `{pkg}` {ver} was yanked: {reason} | use {near} | 21 §21.9 |
| V0823 | warning | this lock was made with {old}, and you run {new} | the toolchain differs | run `verse lock` to record {new}; the build continues and does not reuse old caches | 21 §21.7 |
| V0824 | error | `{pkg}` {ver} has a different API from the one in your lock | the lock says `{a}`, the package says `{b}` | `verse lock --refetch {pkg}`; if it persists, the version was republished | 21 §21.7, §21.13 |
| V0825 | error | `{T}` from `{pkg}` {a} is not `{T}` from `{pkg}` {b} | two major versions of `{pkg}` are in this build | update `{other}`, or pin one version of `{pkg}` | 21 §21.8 |
| V0826 | warning | `{pkg}` {ver}, in your lock, was yanked: {reason} | | `verse update {pkg}` to move to {near} | 21 §21.9 |
| V0827 | warning | updating `{pkg}` adds the `{cap}` capability | {ver_old} did not reach `{cap}`; {ver_new} does | grant it in `[capabilities]`, or stay on {ver_old} | 21 §21.8 |
| V0828 | error | `{pkg}` {ver} is not in the cache or in `vendor/`, and `--offline` is on | | run `verse vendor` while connected, or remove `--offline` | 21 §21.9 |
| V0829 | error | `{file}` in `vendor/` was edited | it does not match the lock's checksum for `{pkg}` | `verse vendor --restore {pkg}`, or copy the package into your own source | 21 §21.9 |
| V0830 | error | `branch` is not accepted | a branch changes without a new version | use `tag = "…"` or `rev = "…"` | 19 §19.3 |
| V0831 | error | `{req}` has no upper bound | any new major version would be accepted | write `"{near}"`, which means {range} | 19 §19.3 |
| V0832 | error | two clean builds of this project differ | `{file}` has different bytes | run `verse why {file}`; the usual cause is a generator that is not deterministic | 21 §21.10 |
| V0833 | error | the package name `{name}` is not available | {reason: taken, reserved, too close to `{near}`, used before by another owner, named like a capability such as `clock`, or named like a standard library module such as `json`} | choose another name | 21 §21.3 |
| V0834 | error | `{pkg}` declares a plugin with `{key}`, which only the root project may use | a dependency cannot build or ship native code | declare the plugin in your own `verse.toml` | 07 §7.2, 21 §21.13 |
| V0835 | error | `{name}` is both a module of this package and a dependency | declared here; `{path}` is also a module of this package | rename the module | 21 §21.4 |
| V0836 | error | no module or package is named `{name}` | not in `src/`, `verse.toml` or the standard library | `verse add {name}`, if it is a package | 21 §21.4 |
| V0837 | error | `{name}` is the name of a standard library module | {what: a module of this program, a dependency, or this package} uses it | choose another name; standard library names are reserved | 21 §21.4 |

### V09xx Capabilities and targets

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0901 | error | this program reaches the `{cap}` capability, which verse.toml does not grant | `{f}` calls `{g}` | add `"{cap}"` to `[capabilities] grant`; or see the whole chain: `verse why {cap}` | 19 §19.3 |
| V0902 | error | this program calls `{f}`, which `{target}` cannot provide | the target has no `{cap}` | build for another target, or avoid `{f}` | 04 §4.3 |
| V0903 | error | this function allocates, and `--no-heap` is on | `{expr}` builds a `{T}` | use a fixed-size type, or turn `--no-heap` off | 05 §5.7 |
| V0904 | error | `{pkg}` reaches `{cap}`, which it does not declare | its `[library] capabilities` lists {list} | add `{cap}` there, or avoid `{f}` | 06 |
| V0905 | error | this package uses {n} KB, but its size budget is {m} KB | the biggest part is `{item}` | see `verse build --size-report` | 06 |
| V0906 | error | `{module}` has different public items on `{target}` | the target variant exports `{item}` only here | make every variant match `verse.api` | 06 §6.9 |
| V0907 | error | the file `{f}` for target `{t}` does not exist | listed in `[targets.{t}.files]` | create it or remove the line | 19 §19.3 |
| V0908 | error | this panic would skip the cleanup of `{x}` | `{x}` is alive here | turn the `abort` profile off, or move the panic site after the cleanup | 05 §5.4 |
| V0909 | error | `{f}` waits and calls itself, which `{target}` cannot run | the wait is here | build for a native target, or make the recursive part not wait | 03 pass 14 (`task-lower`), 04 §4.2 |
| V0910 | error | this `abort` build cannot report a task's panic to its waiter | `spawn` is reachable here | turn the `abort` profile off, or remove the `spawn` | 15 §15.7.6 |
| V0911 | error | `{key}` cannot be used with `{other}` in target `{t}` | {reason} | change one of them in `{file}` | 15 §15.9.4 |
| V0912 | error | `{key}` needs {feature}, which target `{t}` does not provide | {reason} | {fix} | 15 §15.9.4 |
| V0913 | error | target `{t}` has no provider for the `{cap}` capability | its platform layer offers {list} | remove `{cap}` from `{file}`, or pick another platform layer | 15 §15.9.4 |
| V0914 | error | the `[memory]` section of target `{t}` is not valid | {reason: `heap` is not allowed on this layer, or the sizes do not fit in `ram`} | change `{key}` in `{file}` | 15 §15.9.4 |
| V0915 | error | no target is named `{t}` | looked in {list of places} | check the name, or add `targets/{t}.target` | 15 §15.9.1 |
| V0916 | error | this channel's capacity is not a constant, and `--no-heap` is on | `{expr}` is computed when the program runs | write the capacity as a number, or turn `--no-heap` off | 15 §15.3.7 |
| V0917 | error | this `spawn` can run without bound, and `--no-heap` is on | {reason: it is inside a loop, `{f}` calls itself, or `{f}` is called through a function value} | spawn the task once, before the loop, and send it work over a channel; or turn `--no-heap` off | 15 §15.3.7 |

### V10xx Foreign calls

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V1002 | error | `{T}` cannot cross a foreign call | allowed types: {list} | pass the text's `.bytes()` | §12.6, 07 §7.4 |
| V1003 | error | `{f}` needs a written signature | every parameter and the result need types | annotate them | §12.6 |
| V1005 | error | `{target}` cannot link foreign code | {reason} | build for a native target | 07 §7.8 |
| V1006 | error | no plugin is named `{name}` | `extern {name}` needs `[plugins.{name}]` in verse.toml | add `[plugins.{name}]` with `lib` or `path` | §12.6, 07 §7.2 |
| V1007 | error | the library for plugin `{name}` was not found | looked in {places} | install it, or set `path` in `[plugins.{name}]` | 07 §7.6 |
| V1008 | error | the library for plugin `{name}` has no `{symbol}` | declared here | check the C name, or rebuild the library | 07 §7.6 |
| V1009 | error | the build command of plugin `{name}` failed | it exited with status {status} | run `{command}` yourself to see the full output | 07 §7.2 |
| V1010 | error | a foreign function cannot take `{x}` | Verse memory is never handed to foreign code | remove `take`; the argument is borrowed for the call | §12.6, 07 §7.4 |
| V1011 | error | script mode needs a shared library for plugin `{name}` | only `{file}` was found | build a shared library as well as the static one | 07 §7.6 |
| V1012 | error | plugin `{name}` is not available on `{target}` | its `targets` list is {list} | add `{target}` to `targets` in `[plugins.{name}]`, or avoid `{f}` | 07 §7.8 |

### V11xx Command line

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V1101 | error | no command `{word}`; to run a file, write `{word}.vs` | | run `verse` with no arguments to see the commands | 19 §19.2.1 |

### V99xx Internal

| Code | Sev | Message | Label | Help |
|------|-----|---------|-------|------|
| V9901 | error | the compiler failed in {stage}; this is a bug in Verse, not in your program | | run `verse check --report` and send the file |
| V9902 | error | the verifier found a broken rule in {stage} | rule {rule} failed in `{function}` | as above |

## 16.4 How a code is shown

The first line is the message exactly as spec §16.2 shows it, with no code in
it. The code goes on the last line of the block, which also says how to read
more, so the line a person scans first stays plain and the code is easy to copy.

```
error: `email` might be missing
  --> users.vs:12:11
   |
12 |     print(user.email.lower())
   |           ^^^^^^^^^^ this is `Text?`, not `Text`
   |
help: provide a fallback:   user.email.lower() otherwise ""
explain: V0412 (run `verse explain V0412`)
```

- The `explain:` line is always last, after every `help:` line. A `note` and a
  `warning` use it the same way.
- JSON output carries the code as `code`, a separate field (02 §2.8). A tool that reads
  the text can match `^explain: (\S+)` for the code.
- A run-time panic (16.6) uses the same last line with its `R` code.
- Documents show a message block with a bold label above it only when the
  block is not copied from real output. A block copied from real output
  includes its `explain:` line and needs no label.
- A typo suggestion ("did you mean") uses edit distance 2 and is shown only
  when exactly one name is that close.
- The `V0511` note shows how chain ownership works. It is shown only with
  `--explain-ownership` and is never printed by default.

## 16.5 `verse explain`

```
$ verse explain V0412
V0412  `x` might be missing

What happened
  `user.email` has the type `Text?`: a text that may be missing. `lower()`
  needs a text, so Verse stops before the program could meet a missing one.

Why Verse works this way
  There is no null. A value that may be absent says so in its type (§8.1).

Ways to fix it
  1. Use a fallback:   user.email.lower() otherwise ""
  2. Stop early:       user.email.lower()?     (the function must return `Text?` or `Error`)

Example
  fn shout(user: User): Text
      user.email.upper() otherwise "(no email)"
```

Format of every long form: the four headings above, in this order, at most 25
lines, one example that compiles. The long form is part of the catalog and has
its own test (16.1 rule 9).

Codes V0701, V0704 and V0706 are intentionally unassigned: a resource not
used with `with` is cleaned up when its owner's scope ends (§13.1), the spec
has no `Sendable` rule (sharing uses `Shared` and `Guarded`, §11.4), and an
unwaited task is waited for automatically (§11.3), so none needs a
diagnostic.

## 16.6 Run-time panics

A panic is not a compiler diagnostic, but it is the same voice. Its text is
fixed here, so the bytecode VM and every compiled tier print identical words;
the differential tests ([10](../architecture/testing-and-verification.md)) compare them.
Codes are `R` and four digits and follow rules 1 and 2 of 16.1. `verse explain R0002` works like it does for a `V` code. A panic is
printed at the moment it happens, to standard error, and the main program
then exits with status 101 (spec §8.5). R0008 is the exception: a deadlock exits with status 70, so scripts can tell it from a failed `?` and from a panic.

```
panic: index 7 is out of range, the list has 3 items
  --> src/main.vs:12:11
12 |     print(nums[7])
   |           ^^^^^^^ this index
  in main
explain: R0002 (run `verse explain R0002`)
```

The first line is the message; the rest is the position, the label, and the
stack, one function per line. With `VERSE_PANIC=json` or a debugger attached
the same facts are one record (05 §5.4); the message is the same.

| Code | Message | Label | Spec |
|------|---------|-------|------|
| R0001 | `{a} {op} {b}` overflows `{T}` | the result does not fit | spec §3.8 |
| R0002 | index {i} is out of range, the {what} has {n} {unit} | this index | spec §9 |
| R0003 | cannot divide {a} by zero (second form for `%`: "cannot take the remainder of {a} by zero") | this divisor is 0 | spec §8.5 |
| R0004 | recursion is too deep in task {n} | this call made the stack full | 05 §5.3 |
| R0005 | `{x}` is none, and the program expected a value | `{msg}` | spec §8.5 |
| R0006 | assertion failed: `{expr}` | this expression is false | spec §8.5 |
| R0007 | the program ran out of memory | {size} requested here | 05 §5.2 |
| R0008 | every task is waiting and none can continue | {task}: waiting for {what} (one label per task); a task blocked inside a `Cleanup` says "waiting inside the cleanup of {cleanup_of}", where the fact `cleanup_of` is the type name and replaces `what` for that task | spec §11 |
| R0009 | the cleanup of `{x}` panicked while the task was stopping | the first panic was {first} | 15 §15.7.4 |
| R0010 | the range {from}..{to} is out of range, the {what} has {n} {unit} | this range | spec §9.3 |
| R0011 | the exponent {n} is negative, and an `Int` power needs 0 or more | this exponent | spec §3.8 |
| R0012 | `{a}` cannot be converted to `{T}` | {reason: it is too large, it is too small, it is not a number, or it is not a Unicode character} | spec §3.8 |
| R0013 | the exit code {n} is outside 0 to 255 | this call | spec §12.5 and §15.1 `exit`, 15 §15.8 |
| R0014 | `{f}` returned a null pointer, and its signature says `Pointer` | this call | spec §12.6, 07 §7.4 |

**From MIR panic kind to code** (13 §13.4 names the kinds; the operands a kind
carries are the facts below, and the debugger and the compiled tiers fill the
same placeholders from them).

| MIR panic kind | Code | Facts (placeholders) |
|----------------|------|----------------------|
| `overflow OP TYPE` | R0001 | `a`, `op`, `b`, `T` (for a unary operation `b` is absent and the message reads "`{op}{a}` overflows `{T}`") |
| `index_oob` | R0002 | `i`, `n`, `what`, `unit` |
| `div_zero` | R0003 | `a`, and `op` (`/` or `%`) |
| `stack_overflow` | R0004 | `n` (the task) |
| `expect_none` | R0005 | `x`, `msg` (absent when the program gave none) |
| `assert` | R0006 | `expr` |
| `out_of_memory` | R0007 | `size` |
| `slice_oob` | R0010 | `from`, `to`, `n`, `what`, `unit` |
| `neg_exponent` | R0011 | `n` |
| `convert T` (a `checked_cast` that failed: `narrow`, `f2i`, `int_char`) | R0012 | `a`, `T`, `reason` |
| `foreign_null` | R0014 | `f` |

A panic kind with no row here is a bug in the compiler, and the build's
verifier reports it as V9902. `what` is `list` or `text`; `unit` is `items` or
`characters`. R0008, R0009 and R0013 are not MIR panic kinds: the runtime raises them
(`exit` is 15 §15.8). A stack overflow found by the guard page, not by the prologue check, prints the same R0004 text.

Rules for panic wording:

1. The first line is one sentence of at most 15 words, with no word from the
   "don't say" table (08 §8.7).
2. A panic never shows an address, a stage name or a tier. It shows the same
   text on the VM, the native tiers, Wasm and a microcontroller.
3. `R0005` shows the program's own message when `.expect(msg)` gave one, and
   the default text otherwise.
4. A panic in a spawned task prints as above, and its waiter then sees
   `TaskPanicked` (spec §11.2); the waiter prints nothing more by itself.
5. A text that holds a value (`{i}`, `{n}`, `{a}`) writes the number as the
   program would print it.

### 16.6.1 Exit statuses

The spec fixes 0, 1, 101 and `exit(code)` for a running program (spec §12.5).
These are the others, so a script can tell the causes apart.

| Status | Meaning |
|--------|---------|
| 0 | normal completion |
| 1 | a top-level `?` failed, or an unwaited task failed (spec §12.5) |
| 2 | `verse check` or `build`, or running a file, found errors in the source, or the command line names no command (V1101), so the program did not start. This includes a capability the target does not offer or the manifest does not grant (V0901, V0902, a `--deny`), found by the build-time check (15 §15.8.1), and a plugin library that is missing, fails to build, lacks a symbol or is static only (V1007 to V1009, V1011), found before the first statement |
| 3 | a bug in Verse: a compiler bug (V9901 or V9902), or a trap inside the runtime that Verse itself caused. A verifier failure (V9902) is found before the program starts, in `build`, in `verse check` and when running a file; verification after an optimizer pass, which runs only in debug builds of `verse` and in CI, is outside this rule |
| 70 | a deadlock (R0008) |
| 101 | a panic (16.6) |

A `V99xx` message always ends with "this is a bug in Verse, not in your program"
and the `explain:` line (16.4), so the cause is clear even when the status is
all a script sees. An `exit(code)` that the program calls itself is never
changed.

**A checked file starts.** `verse check` makes every check that running a file
makes before its first statement, with the `[run] deny` list of `verse.toml`,
except loading plugin libraries
([19 §19.2.3](cli-and-manifest.md#1923-building-and-checking)).
After `verse check` exits 0 on a file, running that file with the same
sources, `verse.toml`, `verse.lock`, compiler and target never stops with
V9902, and never exits 2 except for a capability refused by a `--deny` flag
of that run, or a plugin library that is missing, fails to build, lacks a
symbol or is static only (V1007 to V1009, V1011).
Verification after an optimizer pass, which runs only in debug builds of
`verse` and in CI, is outside this rule. An instance is optimized at its first
call, so a failure there is V9902 at a first call
([13 §13.5](../architecture/mir-specification.md#135-verifier-rules)).
After a passing check, status 3 means a bug in code generation (V9901 from a backend), a trap inside the runtime,
or, in debug builds of `verse` and in CI, V9902 from verification after an optimizer pass
([ADR-0011](../decisions/0011-verify-before-first-statement.md)).
