## 12. Modules

### 12.1 Files are modules

Every `.vs` file is a module named after its path relative to the project's
`src/` directory (or the directory of the entry file for single-file scripts):

```
my_app/
├── verse.toml
└── src/
    ├── main.vs            # module main
    ├── shapes.vs          # module shapes
    └── geometry/
        └── vectors.vs     # module geometry.vectors
```

### 12.2 Importing

```verse
import fs                           # use as fs.read_text(...)
import json as js                   # use as js.parse(...)
import geometry.vectors             # use as vectors.add(...)
import get, post from http          # use get(...) and post(...) directly
import Shape, area from shapes
import parse as parse_json, stringify from json   # rename one item
```

`import a.b.c` binds the last segment (`c`) as a name; `import a.b.c as d`
binds it as `d`. `import x, y from M` binds individual items, and
`import x as y from M` binds the item `x` under the name `y` (useful when two
modules export the same name); `as` can follow any name in the list, as in
`import parse as parse_json, stringify from json`. The line always starts
with `import`, so every import reads the same way: what you get first, where
it comes from last. Whether a line imports a module or items is decided by
the trailing `from M` on the same logical line: `import a as b` imports the
module `a` as `b`, while `import a as b from m` imports the item `a` of `m` as
`b`; without `from`, the names are module paths. An import must therefore fit
on one line, or put a long item list in parentheses, which may span lines:

```verse
import (
    open, read_text, write_text,
    create as create_file,
) from fs
```

**Where a module comes from.** The first segment of a module path names one
of three things: a module of this program (`geometry`, §12.1), a dependency in
`verse.toml` (§16.3), or a standard library module (`json`, `http`, §15.2).
There is no prefix for the standard library. The three sets never overlap.
Giving a module of the program, a dependency or the package itself a standard
library name is a compile error, and the registry refuses such a package at
publish. So `import json` always means the standard library's `json`.

The names are reserved only as module paths. A local may still be called
`text` or `json`, and if the file imports that module, the local hides it
under the rules of §3.9.

A renamed item keeps its kind: a type must be
renamed to an uppercase-first name, a value to a lowercase one (§3.5). Wildcard imports do not exist; every name in a file can
be traced to a declaration or an import. Imports must appear at the top of a
file, after the module doc comment. Circular imports between modules are
allowed for declarations but not for module-level constant initialization.

An imported name may be hidden by a local, and an import may hide a prelude
name (with a warning); the rules are in §3.9.

**Unused imports.** An imported name that the file never uses is a warning on
the import line. Unlike an unused binding (§3.9), it cannot be silenced, not
even with a leading `_`: the fix is to delete the name, or the whole line. An
import never adds code by itself; it only makes names visible (§12.4), and a
build includes only what the program actually uses.

**Variants of an imported type.** Importing a type does not import its
variants, because every name in a file traces to an import line. *Matching*
needs no import: in a pattern, a variant name is looked up in the type of the
value being matched. *Building* a value needs the variant in scope: import it,
or qualify it with the type name as §4.4 allows.

```verse
import Shape from shapes

fn area(shape: Shape): Float
    match shape
        Circle(r) -> 3.14159 * r * r      # a pattern: no import needed
        Square(s) -> s * s

unit = Shape.Circle(1.0)                  # building: qualified, or `import Circle from shapes`
```

A bare `Circle(1.0)` without the import is an error that names both fixes:
"`Circle` is a case of `Shape`; write `Shape.Circle(…)` or add
`import Circle from shapes`".

### 12.3 Visibility with `export`

Everything is **private to its module by default**. Mark items public with
`export`:

```verse
## Geometry helpers.

export type Vector
    export x: Float
    export y: Float

export type Account
    export owner: Text
    cents: Int = 0                  # hidden: only this module can see it

impl Account
    export fn balance(self): Int
        self.cents

export fn add(a: Vector, b: Vector): Vector
    Vector(x: a.x + b.x, y: a.y + b.y)

export origin = Vector(x: 0.0, y: 0.0)

fn helper(): Int          # not exported: visible only in this module
    42
```

**One word, one default, everywhere.** `export` can precede `fn`, `type`,
`alias`, `trait`, a module-level constant, record **fields**, and functions inside
an `impl`. Anything not marked `export` — including a field of an exported
record — is visible only inside its own module. Other modules can construct a
record directly only if every field without a default is exported; otherwise
they use an exported function such as `Account.open(owner:)`. Trait
implementations are public whenever both the trait and the type are visible.

Fields and methods share **one namespace per type**: a type cannot have a
field and a method with the same name, whether or not either is exported
(`balance` the field and `balance()` the method would be a compile error,
which is why the field above is named `cents`).

#### Exported signatures and `verse.api`

Exported functions may leave their types to inference, just like private ones
— there is no rule that exported functions must annotate their parameters.
This is safe because of three guarantees:

1. **(a) A generated interface file is committed.** `verse api update` writes
   `verse.api` at the package root, recording every exported item with its
   full inferred signature (including inferred constraints and parameter
   modes). The file is committed with the code.
2. **(b) The public API cannot drift silently.** Every build re-infers the
   exported signatures and compares them with `verse.api`. If one differs, the
   build fails and shows the old and new signature, until someone runs
   `verse api update` on purpose (and the change shows up in review as a diff
   of `verse.api`). Editing a function body can therefore never change the
   public API by accident.
3. **(c) Inference never crosses a module boundary.** Other modules — and other
   packages — are type-checked against `verse.api` (and the interface records
   of the package's own modules), never against another module's body.

```verse
## Geometry helpers.

export fn scale(v, factor)        # recorded in verse.api as:
    v.copy(x: v.x * factor, y: v.y * factor)
```

```
# verse.api (generated by `verse api update`; do not edit)
module geometry
fn scale<V, F>(v: V, factor: F): V
    where V has copy(x:, y:), V has field x: F, V has field y: F, F: Mul
```

The interface file uses a documented notation for inferred structural
constraints (`has field`, `has method`) that cannot be written in source in
0.1 (§4.2). Annotating an exported function narrows what `verse.api` records,
which is often what a library author wants.

### 12.4 Module contents

Only the **entry file**, the file passed to `verse`, runs top-level
statements. Imported modules may contain only declarations (`fn`, `type`,
`alias`, `trait`, `impl`, `extern`, `test`, `import`) and module-level constants
(`name = expr`).
Importing a module therefore never has side effects.

A top-level binding `name = expr` behaves in the only way that makes sense in
each place:

- **In an imported module**, a top-level binding is a *constant*. It is
  initialized once, on first use, and its initializer **must be infallible**:
  `?` is not allowed, and its type may not be fallible, because a lazy
  initializer has no caller to return an error to. (`mut` is not allowed
  at module level.)

  ```verse
  export max_retries = 3                            # fine
  export default_port = 8080                        # fine
  config = read_text("config.toml")?                # error: module constants cannot fail
  ```

- **In the entry file**, a top-level binding is an *ordinary statement*: it runs
  in order, top to bottom, like every other statement there, and may use `?`
  (see §8.3 for what a failed top-level `?` does). Such a binding is a **script
  local**: it is visible to the top-level statements after it, but **not
  inside `fn` bodies** (nor in `impl` methods or `test` blocks). A function
  that needs the value takes it as an argument. This keeps every function
  meaningful on its own — it never depends on whether a binding below it has run
  yet — and matches §14.1, where tests never run top-level statements. Module
  constants in imported modules are the only module-wide values.

  ```verse
  config = load_config("app.toml")?

  fn port(config)                     # takes it as an argument
      config.port

  print(port(config))
  ```

  Entry-file locals are not visible in functions (M-2), so the parameter
  `config` is not shadowing.

### 12.5 Program entry

A Verse program is a script: the entry file's top-level statements run from top
to bottom.

```verse
print("Hello, {args.first() otherwise "world"}!")
```

- `args` is a built-in `List<Text>` of command-line arguments, *not* including
  the program name (`verse hello.vs Ana` gives `args == ["Ana"]`).
- **Top-level statements are the only entry mechanism.** Verse 0.1 has no
  `fn main`; a function named `main` is just an ordinary function that runs
  only if the script calls it.
- Exit status, so a script can tell the causes apart:

  | Status | Meaning |
  |--------|---------|
  | `0` | normal completion |
  | `1` | a top-level `?` failed (the error message is printed to standard error), or an unwaited task failed (§11.3) |
  | `2` | the compiler found errors in the source, or a capability check failed, so the program did not start (when running a file as in `verse build`) |
  | `3` | a bug in Verse, not in the program: a compiler bug found before the program started (when running a file as in `verse build`), or a trap inside the runtime that Verse itself caused |
  | `70` | a deadlock: every task is waiting and none can continue |
  | `101` | a panic (§8.5) |

  `exit(code)` ends the program immediately with the status `code`, which must
  be 0 to 255; any other value panics with "the exit code {n} is outside 0 to
  255" on every platform, so `exit(256)` never reports success. `exit` may be
  called from any task. It flushes standard output and standard error, but
  **`exit` runs no `Cleanup`** and does not wait for or cancel the other tasks:
  the process just ends, so a file's unwritten buffer is lost and a temporary
  directory stays. A program whose cleanups must run ends by finishing the
  entry file or with a failed `?` instead. An explicit `exit(code)` always
  overrides the failed-unwaited-task status, which applies only when the
  program ends naturally.


### 12.6 Foreign functions (`extern`)

An `extern` block declares functions from a native library, a **plugin**,
so Verse code can call them. The plugin is named in `verse.toml` under
`[plugins.<name>]`, and the block names it:

```verse
extern sqlite
    export fn sqlite3_open(filename: Bytes, mut db: Pointer?): Int32
    export fn sqlite3_close(db: Pointer): Int32
    fn SDL_Init as sdl_init(flags: UInt32): Int32

mut db: Pointer? = none
status = sqlite3_open("app.db".bytes(), mut db)
```

- **The block names a declared plugin.** `extern sqlite` with no
  `[plugins.sqlite]` in `verse.toml` is an error. Declaring the library is the
  only opt-in.
- **Every signature is written in full.** Each parameter and the result have
  a type, and nothing is inferred, because a foreign function has no body to
  infer from. A function without a result type returns `Nothing`.
- **The C name comes first.** `fn SDL_Init as sdl_init(…)` calls the C symbol
  `SDL_Init` and binds it as `sdl_init`, in the order of an import (§12.2).
  Without `as`, the C name must be a valid value name.
- **The functions are module items.** They are exported with `export` and
  imported like any other function. An `extern` block may appear only at the
  top level of a module.
- **A call is a normal call.** There is no `unsafe` marker and no `with`
  block.
- **Only plain data crosses.** Parameters and results are numbers
  (`Int`, `Float`, `Int8` … `UInt64`, `Float32`), `Bool`, `Char`,
  `Pointer`, `Pointer?`, `Bytes` and byte views (parameters only), and `Copy`
  records made only of these. `Text` does not cross: call `.bytes()` on it.
  Collections, other optionals, variants, trait types, function values, `Task`
  and `Cleanup` types do not cross.
- **Ownership words keep their meaning.** A plain parameter is borrowed for the
  call. A `mut` parameter is an exclusive borrow that the foreign function may
  write. `take` is not allowed: Verse memory is never handed to foreign code.

`Pointer` is an opaque foreign address. It is `Copy` and can be stored,
passed and compared with `==`. It never holds null; a foreign pointer that
may be null is `Pointer?`, and a null returned where `Pointer` was declared is
a panic. `p.read_bytes(count)` copies `count` bytes from foreign memory into a
new `Bytes`, and `p.read_c_bytes()` copies the bytes before the first zero
byte.

**Verse cannot check foreign code.** The rules above are checked on the
Verse side of the call. What the library does inside is outside the
language: it can corrupt memory, keep a pointer it was given, or reach the
network under `--deny net`. A foreign function must not unwind, and it
cannot call back into Verse. The type mapping to C, loading in script and
compiled mode, and `verse bind` are described in architecture
[07](../architecture/plugins.md).

---

