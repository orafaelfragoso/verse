# 07. Plugins

Requirement R3: *the language can be extended with native libraries*. A
plugin is how a Verse program calls code written in another language. This
document says what a plugin is, how a project declares it, how values cross
the boundary, and how script mode and compiled mode load it. The language
rules for `extern` are in spec [§12.6](../language/12-modules.md#126-foreign-functions-extern).
The decision and its alternatives are in [ADR-0005](../decisions/0005-native-c-abi-plugins.md).

## 7.1 What a plugin is

A plugin is a compiled library with a C interface. Any language that can
export C functions can write one:

| Language | How it exports a function |
|----------|---------------------------|
| C | an ordinary function in a header |
| C++ | `extern "C"` |
| Zig | `export fn` |
| Rust | `#[no_mangle] pub extern "C" fn`, in a crate built as `cdylib` or `staticlib` |

There is one mechanism for every language: the platform C ABI. A plugin
never changes the language. It adds no keyword, operator, annotation or
grammar rule, and it cannot rewrite the program. A `.vs` file means the same
thing whatever plugins a project declares. From Verse, a plugin is a set of
functions declared in an `extern` block and called like any other function.

**Verse cannot check foreign code.** The compiler checks the Verse side of
every call: the types, the ownership words, the declared library. It cannot
see inside the library. A plugin can read freed memory, keep a pointer it
was told not to keep, open a socket while the program runs with
`--deny net`, or crash the process. Ownership, bounds checks, capability
grants and panics stop at the boundary. Declaring a library in `verse.toml`
is the decision to trust it. `verse add` and the size report name every
plugin a program uses, so that decision stays visible.

Lints and generators are not plugins. Lints are built into the compiler, and
generators are ordinary Verse packages run by `verse generate` (spec
[§16.5, §16.6](../language/16-tooling.md#165-lints)).

## 7.2 Declaring a plugin

Every plugin is a section `[plugins.<name>]` in `verse.toml`. The name is an
identifier (`[a-z][a-z0-9_]*`), and `extern <name>` in source refers to it.

```toml
[plugins.sqlite]
lib = "sqlite3"                                 # a system library

[plugins.fastmath]
path  = "native/fastmath/target/release"        # a library built in this repository
build = "cargo build --release --manifest-path native/fastmath/Cargo.toml"
```

| Key | Type | Default | Meaning |
|-----|------|---------|---------|
| `lib` | text | the plugin name | the library's file name without platform prefix and suffix: `sqlite3` finds `libsqlite3.so`, `libsqlite3.dylib`, `sqlite3.dll` or the static forms |
| `path` | path | none | the folder that holds the library. Without it, the system linker paths and `pkg-config` are searched |
| `build` | text | none | a command run before linking or loading, from the package root, through the system shell |
| `link` | `"dynamic"` \| `"static"` | `"dynamic"` | how compiled mode links the library (7.6) |
| `targets` | list of target names | every target | where the plugin is available (7.8) |

Rules:

- **Declaring is the opt-in.** An `extern` block whose name has no
  `[plugins.<name>]` section is an error (V1006). No other grant exists.
- **`build` runs before each build or run that loads the library.** The tool
  it calls (Cargo, Zig, `make`) decides whether anything needs rebuilding, so
  Verse keeps no state of its own about it. It never runs during
  `verse check`. Its output is shown only when it fails (V1009).
- **A dependency may declare `lib` plugins only.** A published package can
  wrap a system library, because `lib` is data. It cannot use `path` or
  `build`, because building a dependency never runs code (spec §16.3). A
  dependency with either key is rejected at resolution (V0834). `verse add`
  and `verse update` list the plugins a package declares, next to its
  capabilities.
- **Unknown keys are errors** (V0801), as everywhere in `verse.toml`.

## 7.3 `extern` blocks

An `extern` block names a plugin and lists the C functions the program uses,
each with a full signature and no body:

```verse
## Bindings for SQLite. Written by `verse bind`, then edited.

extern sqlite
    export fn sqlite3_libversion_number(): Int32
    export fn sqlite3_open(filename: Bytes, mut db: Pointer?): Int32
    export fn sqlite3_close(db: Pointer): Int32
    export fn sqlite3_errmsg(db: Pointer): Pointer
```

The functions are items of the module that holds the block, imported and
exported like any other function. A C name that is not a Verse value name is
renamed with `as`, in the order of an import: `fn SDL_Init as sdl_init(flags:
UInt32): Int32`. The call site is a normal call. There is no `unsafe`
marker and no special block. The full rules are spec
[§12.6](../language/12-modules.md#126-foreign-functions-extern).

## 7.4 Values across the boundary

Only the types below cross. Any other type in an `extern` signature is an
error that names the type and lists these (V1002).

| Verse | C | Notes |
|-------|---|-------|
| `Int8` … `Int64`, `UInt8` … `UInt64` | `int8_t` … `uint64_t` | by value |
| `Int` | `int64_t` | by value |
| `Float`, `Float32` | `double`, `float` | by value |
| `Bool` | `bool` | one byte; foreign code must return 0 or 1 |
| `Char` | `uint32_t` | a Unicode scalar value; any other value is a panic (R0012) |
| `Pointer` | any `T *` that is never null | opaque: Verse can store it, pass it and compare it, never read through it except with the methods below |
| `Pointer?` | any `T *` that may be null | null is `none` |
| `Bytes`, a byte view (`"…".bytes()`) | `const uint8_t *` | a parameter only; a zero byte follows the last byte, so a C string function can read it |
| a `Copy` record of the types above | a `struct` with the same fields in the same order | by value, with the target's C layout |
| `Nothing` | `void` | a result only |

- **`Text` never crosses.** Write `name.bytes()`. The conversion stays
  visible, and it is the one spelling. A text that contains a zero character
  ends early for C, and Verse does not check for one.
- **A length is passed explicitly.** C cannot learn the length of a `Bytes`
  argument. A C function that takes a buffer and a length is declared with both,
  and called as `write(data, data.len())`.
- **`mut` means a pointer to the caller's value.** `mut db: Pointer?` is
  `sqlite3 **`, and `mut n: Int32` is `int32_t *`. Foreign code may write a
  new value of the declared type during the call. Only a scalar, `Pointer`,
  `Pointer?` or `Copy` record may be `mut`.
- **`take` is not allowed** (V1010). Verse memory is never handed to foreign
  code, because foreign code cannot free it.
- **No callbacks, no variadic functions.** A function value cannot cross,
  and `printf`-style signatures cannot be declared.
- **Reading foreign memory.** `p.read_bytes(count: Int): Bytes` copies `count`
  bytes starting at `p`. `p.read_c_bytes(): Bytes` copies the bytes up to the
  first zero byte. Both copy into a new owned `Bytes`; Verse cannot check that
  the memory is valid.
- **A null where `Pointer` was declared panics** (R0014). Verse checks every
  `Pointer` result, because the check is one comparison and the bug would
  otherwise surface far from its cause. Declare the result `Pointer?` when
  null is a normal answer.

The width of C's `int`, `long` and `size_t` depends on the target. `verse
bind` writes the sized type for the target it runs on. A project that builds
for targets with different C data models keeps one bindings file per data
model and selects it with `[targets.<t>.files]`.

## 7.5 Ownership and lifetime

| Direction | Rule |
|-----------|------|
| a plain parameter | a borrow for the duration of the call. Foreign code must not keep a pointer into a `Bytes` argument after it returns |
| a `mut` parameter | an exclusive borrow for the duration of the call, as in Verse (§5.3) |
| a result | owned by the caller. A `Pointer` result is only an address: Verse never frees what it points to |

Foreign memory is released by calling the library's own function. A drop runs
no user code and never calls foreign code (spec §13.4,
[ADR-0013](../decisions/0013-drop-glue-runs-no-user-code.md)), so the usual
shape is a Verse record that holds the `Pointer` and offers a named close
method; a value dropped without it leaks the foreign resource:

```verse
import sqlite3_open, sqlite3_close from sqlite_c

export type Database
    handle: Pointer

impl Database
    ## Closes the database. A `Database` dropped without `close` stays open.
    export fn close(take self)
        sqlite3_close(self.handle)

export fn open(path: Text): Database | Error
    mut db: Pointer? = none
    status = sqlite3_open(path.bytes(), mut db)
    handle = db otherwise return error("cannot open {path}: out of memory")
    database = Database(handle:)
    if status != 0
        database.close()                  # SQLite returns a handle even when the open fails
        return error("cannot open {path}: SQLite status {status}")
    database
```

Errors stay values: a C function returns a status code, and a Verse
function turns it into `T | Error` (§8). The binding file declares the C
functions as they are; the wrapper module is where errors and closing live.

Other rules at the boundary:

- **No unwinding through foreign frames.** A foreign function must return
  normally. A C++ exception or a Rust panic that reaches the boundary is
  undefined. Rust aborts on a panic that leaves an `extern "C"` function, which
  is the behaviour Verse needs. A Verse panic never crosses into foreign code,
  because foreign code never calls Verse.
- **Tasks.** A foreign call runs on the thread of the calling task and holds
  it until it returns. It is not a blocking point for cancellation (§11.5) and
  does not yield. A long call belongs in its own task. Foreign code may start
  its own threads; they cannot call into Verse.
- **Debugger.** A foreign call shows as `called` with `foreign: true`, and the
  debugger does not step into it. A value returned from it carries the note
  `from_foreign`, worded "came from a foreign call" ([`events.toml`](../reference/events.toml)).

## 7.6 Loading in script mode and compiled mode

Both modes run the same checks, the same type mapping and the same machine
code inside the library. Only the moment the library is attached differs.

| | Script mode (`verse app.vs`, tier 0) | Compiled mode (`verse build`) |
|---|---|---|
| when | before the first statement runs | at link time |
| how | the VM opens the shared library (`dlopen`, `LoadLibrary`) and resolves each reachable symbol | the linker links the library, dynamically or statically per `link` |
| the call | the `CALLF` instruction ([14 §14.4](bytecode-and-vm.md#144-instruction-set-and-encoding)), through a call stub generated per signature by `verse-ffi` | a direct C-ABI call |
| a missing library or symbol | the program does not start: V1007 or V1008, exit status 2 | the build fails: V1007 or V1008 |

- **Only reached functions count.** Reachability ([06](stdlib-and-shaking.md))
  adds an edge from each reachable call to the plugin and its symbol. A
  plugin with no reachable call is not loaded and not linked. The size report
  lists each linked plugin with the chain that reached it.
- **Script mode needs a shared library.** A plugin with `link = "static"` must
  also provide a shared build for script mode; with only a static file,
  running the program stops with V1011. A Rust crate gets both with
  `crate-type = ["cdylib", "staticlib"]`.
- **A failure to load is a start-up failure, never a late one.** Every
  reachable symbol is resolved before the first statement, in both modes, so a
  program never fails halfway through because a symbol is missing.

## 7.7 `verse bind`

`verse bind <header.h> --plugin <name>` reads a C header and writes a `.vs`
file with one `extern` block for it. The file is ordinary source that the
project owns and edits. Nothing regenerates it behind the author's back.

- C types map by 7.4, using the target the command runs for (`--target`).
- A C name that is not a Verse value name gets an `as` rename in snake case.
- What a header cannot say, the author decides by editing: whether a pointer
  may be null (`Pointer` or `Pointer?`), which functions to keep, and the
  wrapper module of 7.5.
- A declaration that cannot be expressed (a callback, a variadic function, a
  `union`, a struct with a field of another type) is written as a comment
  that says why, so nothing is dropped silently.
- `verse bind` writes to `--out <file>` and refuses to overwrite an existing
  file. To see what changed in a new header, write to a new file and compare.

## 7.8 Targets

A plugin is available on the targets in its `targets` key, and only where
its library exists for that target.

| Target | Plugins |
|--------|---------|
| native (Linux, macOS, Windows) | dynamic or static |
| WASI | static only, built for the same Wasm target and linked into the module |
| browser Wasm | none: a reachable foreign call is V1005 |
| microcontrollers | static only |

`verse check --target <T>` reports every reachable call into a plugin that
`T` cannot provide, before anything is built (V1005, or V1012 for a plugin
whose `targets` list leaves `T` out).

## 7.9 Diagnostics

The V10xx family of the [diagnostics catalog](../reference/diagnostics-catalog.md#v10xx-foreign-calls)
covers plugins and `extern`:

| Code | When |
|------|------|
| V1002 | a type that cannot cross a foreign call |
| V1003 | an `extern` function without a full signature |
| V1005 | a foreign call reachable on a target that cannot link it |
| V1006 | `extern <name>` with no `[plugins.<name>]` |
| V1007 | the library was not found |
| V1008 | the library has no such symbol |
| V1009 | the `build` command failed |
| V1010 | `take` on an `extern` parameter |
| V1011 | script mode found only a static library |
| V1012 | a plugin whose `targets` list leaves out the target |

A dependency that declares `path` or `build` is V0834. A null `Pointer` result
is the panic R0014.
