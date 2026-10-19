# Architecture 16. Diagnostics catalog

Owner: Michelangelo. Reviewer: Bob. Status: **draft 1**.
This file assigns a stable code to every diagnostic the 0.1 toolchain gives,
and fixes its wording. It builds on 02 §2.8 (the `Diagnostic` record), 08 §8.7
(vocabulary and the "don't say" table), spec §16.2 (the style) and
`vocab/events.toml`. Entries marked **0.2** belong to the `extern` section and
are reserved, not active.

## 16.1 Rules

1. **A code never changes meaning and is never reused.** A removed diagnostic
   keeps its code, marked *retired*.
2. **Codes are `V` and four digits.** The first two digits are the family
   (16.2). Errors and warnings share the number space; the record's `severity`
   says which one it is. Debugger codes are `D0xxx` (17.2). Plugin codes are
   `plugin/rule` and never start with `V` (07 §7.6).
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
| Foreign calls (0.2) | V10xx | `extern` and `bindgen` |
| Internal | V99xx | a compiler bug; the message asks for a report |

`V0412` (the optional-used-as-plain example in spec §16.2 and 02 §2.8) keeps
its number.

## 16.3 Catalog

Each entry: code, severity, message, label under the primary position, help
lines, rule. `{…}` is filled by the compiler. Examples show the label and the
`help:` text only; the full layout is the one of spec §16.2.

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
| V0108 | error | the number {text} does not fit in `Int` | the largest `Int` is 9,223,372,036,854,775,807 | use a `Float`, or a sized type from `std.num` | §4.1 |

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
| V0310 | error | modules `{a}` and `{b}` import each other at startup | the constants initialise in a cycle | move the shared constant to a third module | §12.2 |
| V0311 | error | `{case}` is a case of `{Type}` | cases are not imported with the type | write `{Type}.{case}(…)`, or add `import {case} from {module}` | §12.2, §4.4 |
| V0312 | error | a renamed type needs a capitalised name | `{x}` is a value name | write `as {X}` | §3.5 |

### V04xx Types

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0401 | error | expected `{T}`, found `{U}` | this is `{U}` | convert it with `{conversion}` (shown when one exists) | §4.2 |
| V0402 | error | cannot tell the type of `{name}` | nothing here decides it | annotate it: `{name}: {guess}` (shown when one candidate exists) | §4.2 rule 4 |
| V0403 | error | `{f}` needs an annotation because it calls itself | recursion hides the return type | write `fn {f}(…): {T}` | §4.2 rule 3 |
| V0404 | error | `{f}` was called with {n} arguments, but takes {m} | | the parameters are `{signature}` | §5.3 |
| V0405 | error | `{f}` has no parameter called `{p}` | the parameters are {list} | | §5.3 |
| V0406 | error | `{T}` has no field or method `{name}` | | did you mean `{near}`? | §4.3 |
| V0407 | error | `{T}` does not implement `{Trait}` | needed by `{f}` here | add `impl {Trait} for {T}` (shown for traits that cannot be derived) | §10.1 |
| V0408 | error | `{T}` cannot be compared with `{U}` | `==` needs two values of one type | convert one side | §9.6 |
| V0409 | error | `{x}` does not fit in `{T}` | the largest `{T}` is {max} | use a larger type | §4.1 |
| V0410 | error | `Int` and `Float` are not mixed implicitly | | write `{x}.to_float()` or `{y}.round().to_int()` | §4.1 |
| V0411 | error | a fallible value cannot be ignored | `{f}` returns `{T} \| Error` | handle it with `?`, `otherwise` or `match` | §8.4 |
| V0412 | error | `{name}` might be missing | this is `{T}?`, not `{T}` | provide a fallback: `… otherwise {default}`; stop early: `…?`; branch: `if … is {name}?` | §8.1 |
| V0413 | error | `?` can only be used in a function that returns an optional or `Error` | `{f}` returns `{T}` | change the return type to `{T}?` or `{T} \| Error` | §8.3 |
| V0414 | error | `otherwise` needs an optional or fallible value on its left | this is `{T}`, which is always there | remove `otherwise` | §3.8.2 |
| V0415 | error | `{x}` is `{T}?` and cannot be used as `{T}` here | the dot looks through an optional only when the chain ends in `otherwise`, `?`, `is`, `match`, or an optional-typed position | add one of those at the end of the chain | §8.1 |
| V0416 | error | the `Error` of `{f}` is not handled here | `{f}` can return `{E}` | add `?`, `otherwise` or a `match` arm for `{E}` | §8.4 |
| V0417 | error | `{x}` is not a `Text`, so it cannot be indexed by character | | use `.bytes()` for a byte view | §4.1 |
| V0418 | error | missing type for a public item | `export` items need types in `verse.api` | annotate the parameters and the result | §16.4 |

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

V0511 is a `note`, not an error (its severity is `note`, per 02 §2.8).

### V06xx Patterns

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0601 | error | this `match` does not cover {missing} | add an arm for {missing} | add `{pattern} -> …` or a final `_ -> …` | §7.2 |
| V0602 | warning | this arm can never match | an earlier arm already matches everything it matches | delete it | §7.1 |
| V0603 | warning | `{name}` looks like the case `{Name}` | in the last arm a lowercase name binds anything | write `{Name}` if you meant the case | §7.2 |
| V0604 | error | `{pattern}` is not a case of `{T}` | the cases are {list} | | §7 |
| V0605 | error | a typed pattern is only allowed at the top of an arm | `{x}: {T}` is inside parentheses | write `field: {x}: {T}` | §7.3 |
| V0606 | error | this destructuring has {n} parts, but the value has {m} | | | §7.7 |
| V0607 | error | `{pattern}` is a type, not a case | `{Type}` cannot be used as a pattern here | write `field: {x}: {Type}` | §7.3 |

### V07xx Resources and tasks

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0701 | error | `{x}` must be cleaned up | `{T}` is a resource and is not used with `with` | write `with {expr} as {x}` | §13 |
| V0702 | error | `{expr}` is `{T} \| Error`, which cannot be cleaned up | `with` needs the resource itself | add `?` to propagate the error | §13 |
| V0703 | error | a task cannot use `{name}` | it is a view | send a copy with `.copy()`, or move it in | §11.1 |
| V0705 | error | a `Receiver` has one owner | `{name}` was moved at {position} | give the receiver to one task | §11.4 |
| V0707 | error | `{name}` is locked here and cannot be used outside `with` | `Guarded` values are read inside the block | read it as `with {g}.lock() as {value}` | §11.4 |

### V08xx Packages and API

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0801 | error | `{key}` is not a key of `[{section}]` | unknown keys are errors | did you mean `{near}`? | 19 §19.3 |
| V0802 | error | the package name `{name}` cannot be imported | a name uses letters, digits and `_`, starting with a letter | write `{suggested}` | 19 §19.4 |
| V0803 | error | version {have} is too small for these changes; publish as {need} | {change summary} | set `version = "{need}"`, or keep the old signature and add a new function | 19 §19.5 |
| V0804 | error | version {req} of `{pkg}` does not exist | the newest release is {newest} | `{pkg} = "{near}"` | 19 §19.7 |
| V0805 | error | `{pkg}` lists plugins, but only the root project may | a dependency cannot run code while your project builds | add the plugin to your own `verse.toml` under `[plugins]` | 07 §7.5 |
| V0806 | error | `{pkg}` {ver} does not match its checksum | the archive hashes to `{got}` | `verse lock --refetch {pkg}` | 19 §19.12 |
| V0807 | error | `verse.lock` is out of date | `{pkg}` asks for {req}, the lock has {have} | run `verse lock` (not allowed with `--locked`) | 19 §19.6 |
| V0808 | error | `{pkg}` needs {T}, which `{other}` does not provide | two packages need different major versions of `{pkg}` | pin both to one major | 19 §19.7 |
| V0809 | error | a path dependency cannot be published | `{pkg}` is `{ path = … }` | name a version as well | 19 §19.10 |
| V0810 | warning | `verse.api` changed | the new API adds {n} items | run `verse api update` | §16.4 |
| V0811 | error | `verse.api` differs from the source | `{item}`: file says `{a}`, source says `{b}` | run `verse api update` | §16.4 |
| V0812 | error | `{name}` is not a valid license | use an SPDX name such as `MIT` | | 19 §19.3 |
| V0813 | error | a yanked version cannot be chosen | `{pkg}` {ver} was yanked: {reason} | use {near} | 19 §19.8 |
| V0814 | error | plugin `{name}` is a `{k1}`, but verse.toml says `{k2}` | the plugin's own manifest says `{k1}` | change `kind` in verse.toml | 18 §18.2 |
| V0815 | error | plugin `{name}` asks for more than verse.toml grants | it asks to {ask}; the grant is {grant} | grant it in `[plugins]`, or do not use this plugin | 07 §7.5 |
| V0816 | warning | plugin `{name}` reported a code that is not its own | `{code}` was dropped | tell the plugin's author | 18 §18.7 |
| V0817 | error | plugin `{name}` needs plugin interface {n}, and this compiler supports {m} | | update the compiler or use an older plugin | 18 §18.9 |

### V09xx Capabilities and targets

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V0901 | error | this program reaches the `{cap}` capability, which verse.toml does not grant | `{f}` calls `{g}` | add `"{cap}"` to `[capabilities] grant`; or see the whole chain: `verse why {cap}` | 19 §19.3 |
| V0902 | error | this program calls `{f}`, which `{target}` cannot provide | the target has no `{cap}` | build for another target, or avoid `{f}` | 04 §4.3 |
| V0903 | error | this function allocates, and `--no-heap` is on | `{expr}` builds a `{T}` | use a fixed-size type, or turn `--no-heap` off | 05 §5.7 |
| V0904 | error | `{pkg}` reaches `{cap}`, which it does not declare | its `[library] capabilities` lists {list} | add `{cap}` there, or avoid `{f}` | 06 |
| V0905 | error | this package uses {n} KB, but its size budget is {m} KB | the biggest part is `{item}` | see `verse build --size-report` | 06 |
| V0906 | error | `{module}` has different public items on `{target}` | the target variant exports `{item}` only here | make every variant match `verse.api` | 06 P6-7 |
| V0907 | error | the file `{f}` for target `{t}` does not exist | listed in `[targets.{t}.files]` | create it or remove the line | 19 §19.3 |

### V10xx Foreign calls (0.2, reserved)

| Code | Sev | Message | Label | Help | Rule |
|------|-----|---------|-------|------|------|
| V1001 | error | calling `{f}` needs the `{cap}` capability | foreign code runs here | wrap the call in `with {cap}() as {x}` | extern E3 |
| V1002 | error | `{T}` cannot cross a foreign call | allowed types: {list} | pass `text.bytes()` for text | extern E5 |
| V1003 | error | `{f}` needs a written signature | every parameter and the result need types | annotate them | extern E2 |
| V1004 | error | the bound name `{x}` is never used in this `with` block | the block is there to permit foreign calls | pass `{x}` to the calls it permits | extern review R1 |
| V1005 | error | `{target}` cannot link foreign code | the browser target has no native linking | build for a native target | extern E11 |

### V99xx Internal

| Code | Sev | Message | Label | Help |
|------|-----|---------|-------|------|
| V9901 | error | the compiler failed in {stage}; this is a bug in Verse, not in your program | | run `verse check --report` and send the file |
| V9902 | error | the verifier found a broken rule in {stage} | | as above |

## 16.4 Plugin diagnostics

A plugin writes its own codes as `plugin/rule` (for example `style/prefer-map`).
The tool adds `source: "plugin"` and the plugin name. A plugin diagnostic is a
warning unless the root `verse.toml` promotes it with `promote = […]`. It
cannot reuse a `V` code, suppress a compiler diagnostic, or lower one.

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

Codes V0704 and V0706 are intentionally unassigned: the spec has no `Sendable`
rule (sharing uses `Shared` and `Guarded`, §11.4), and an unwaited task is
waited for automatically (§11.3), so neither needs a diagnostic.

## 16.6 Open points

1. **Exact spec section numbers** in the "Rule" column marked with a section
   only (for example `§7`) are the section, not a numbered rule. They are
   refined when the conformance suite (10 §10.2) gives every rule an identifier.
2. **Bob:** the list of compiler diagnostic families you offered would let me
   check this catalog against the passes. Anything in the passes with no entry
   here needs a code; anything here with no pass is a candidate to remove.
3. **Typo suggestions** ("did you mean") use edit distance 2 and are shown only
   when exactly one name is that close.
4. **The `V0511` note** shows how chain ownership works. It is a `note` shown
   only with `--explain-ownership` and is never printed by default.
