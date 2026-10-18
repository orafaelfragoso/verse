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
| `verse build` | produce an optimized native executable |
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
name = "word-tools"
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
  `import red, bold from colors`.
- Resolution chooses a single version of each package per major version.
- Builds are reproducible: the same `verse.lock` gives the same build.
- Packages declare no install scripts; building a dependency cannot run
  arbitrary code.
- A published package includes its `verse.api`; dependents are checked against
  it, never against the package's source.

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

