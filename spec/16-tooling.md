## 16. Tooling

Verse ships as a single command, `verse`, that contains the compiler, runner,
formatter, test runner, documentation generator, language server and package
manager.

| Command | Purpose |
|---------|---------|
| `verse run file.vs [args…]` | compile (cached) and run a script or project |
| `verse test [path]` | run `test` blocks and doc tests; `--filter "name"`, `--watch` |
| `verse fmt [path]` | format code in the one canonical style; `--check` for CI |
| `verse check` | type-check without running; fast feedback for editors |
| `verse build` | build for the selected profile and target: a dev or release executable, a WebAssembly module, or a bytecode bundle |
| `verse build --size-report` | build, and list every included item with its size and why it was included |
| `verse why <item-or-path>` | one command, told apart by the form of its argument: for an item (`std.net.get`), the chain of uses from the program's entry statements; for a file path (`generated/item.vs`), the plugin that wrote it |
| `verse run --deny <capability>` | run, refusing a system capability named after its `std` module (`net`, `fs`, `process`, …); a program that can reach it fails before it starts |
| `verse check --target <T>` | check that everything the program can reach is available on target `T` (no `std.fs` on a browser WebAssembly target, for example) |
| `verse doc` | generate HTML documentation from doc comments |
| `verse repl` | interactive prompt |
| `verse new name` | create a project skeleton |
| `verse add pkg` / `verse remove pkg` | manage dependencies |
| `verse update` | update dependencies within allowed versions |
| `verse publish` | publish a package to the registry |
| `verse lsp` | language server for editors, including inline hints for inferred types |
| `verse api update` | regenerate `verse.api`, the committed interface file of exported signatures (§16.4) |
| `verse api check` | compare inferred exported signatures with `verse.api` (run by every build) |

### 16.1 The formatter

`verse fmt` has **no configuration options**. It normalizes indentation to four
spaces, spacing around operators, blank lines between declarations and
trailing commas in multi-line lists; wraps long method chains one call per
line; sorts imports; enforces the blank line after a module doc (§14.3); and
drops the redundant `)` that the lambda closing rule makes unnecessary (§5.4). It
never removes semantically significant parentheses — those that end a lifted
chain, and those in `(a?).b`, where `a?.b` would be an error (§8.1). It
**never inserts or removes type annotations**: whether to spell out a type is
the author's choice, and inferred types are shown by the editor as inline hints
instead. A file that passes `verse fmt --check` looks the same as every other
Verse file in the world.

### 16.2 Compiler messages

Error messages are written in plain language, point at the exact code, and
suggest a fix where possible:

```
error: `email` might be missing
  --> users.vs:12:11
   |
12 |     print(user.email.lower())
   |           ^^^^^^^^^^ this is `Text?`, not `Text`
   |
help: provide a fallback:   user.email.lower() otherwise ""
help: or stop early:        print(user.email.lower()?)
help: or branch:            if user.email is email?  …
```

### 16.3 Packages

A project is described by `verse.toml`:

```toml
[package]
name = "word_tools"
version = "0.3.0"
description = "Small text utilities"
license = "MIT"
verse = "0.1"

[dependencies]
colors = "1.2"              # semantic versioning: >= 1.2.0, < 2.0.0
http_extra = { git = "https://example.com/http_extra.git", tag = "v0.4.1" }

[dev-dependencies]
snapshot = "0.9"
```

- Packages are fetched from the central registry (or git/path sources) and
  pinned in `verse.lock`, which is committed for applications.
- A package's modules are imported by package name:
  `import red, bold from colors`. A package name therefore looks like an
  identifier: lowercase letters, digits and `_`, starting with a letter
  (`[a-z][a-z0-9_]*`), so `word_tools`, never `word-tools`.
- Resolution chooses a single version of each package per major version.
- Builds are reproducible: the same `verse.lock` gives the same build.
- Packages declare no install scripts; building a dependency cannot run
  arbitrary code.
- A published package includes its `verse.api`; dependents are checked against
  it, never against the package's source.

A library package may add a `[library]` section that describes what it offers:

```toml
[library]
modules      = ["colors", "colors.term"]   # the modules dependents may import
capabilities = ["stdio"]                   # the system capabilities its code may reach
size_budget  = "40K"                       # `verse publish` fails if its example build exceeds it

[library.targets]                          # where the package is available
default      = "all"
unavailable  = []                          # targets it does not support

[library.native]                           # reserved for 0.2, with `extern` (§18.15)
links        = []                          # native libraries to link: data, never scripts
```

Domain libraries are first-party optional packages, not part of `std`:
`finance`, `ml`, `web`, `gui` and `iot` are added with `verse add` like any
other package. `std.net.serve` remains in `std`.

### 16.4 Interface files (`verse.api`)

`verse.api` is what makes inferred exported signatures safe (§12.3):

| Step | What happens |
|------|--------------|
| `verse api update` | infers every exported signature and writes `verse.api`; commit it |
| `verse build`, `verse check`, `verse test` | re-infer exported signatures; **fail** if any differs from `verse.api`, printing the recorded and the new signature |
| other modules and packages | are checked against `verse.api` only — inference never crosses a module boundary |

So the only way to change a public signature is to change the code *and* run
`verse api update`, which turns the change into a visible diff in review.
`verse fmt` never touches `verse.api` and never inserts annotations into
source.

---

