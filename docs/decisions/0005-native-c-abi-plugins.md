# ADR-0005: Plugins are native C-ABI libraries

Status: accepted (2026-10-18).

## Context

Requirement R3: extend the language. Programs need code that is not Verse:
system libraries (SQLite, OpenSSL, a board's SDK), numeric kernels for the
`ml` package, and existing code in C, C++, Zig or Rust. Plugins must never
add syntax or change what a program means, and building a dependency must
never run code (spec §16.3).

## Decision

A plugin is a **compiled library with a C interface**, declared in
`verse.toml` as `[plugins.<name>]` and called through `extern` blocks in
source (spec §12.6). Foreign calls are plain calls: no capability gate, no
`with` block, no `unsafe` marker. Declaring the library is the opt-in, and
the documentation says plainly that Verse cannot check foreign code. Script
mode opens the library at start-up; compiled mode links it. Both run the same
checks and the same type mapping. Details in [07](../architecture/plugins.md).

Two extension needs are met without plugins. Lints are built into the
compiler. Code generators are ordinary Verse packages run by `verse generate`
(spec §16.5, §16.6).

## Alternatives

- **Sandboxed WebAssembly components** (lint, generator, bindgen and tool
  kinds, run by an embedded Wasm engine with WIT interfaces and capability
  grants). Rejected. It covered tooling, not the main need: a program could
  still not call SQLite or a vendor SDK. It added a Wasm engine to the
  toolchain, a versioned host interface, fuel and memory budgets, and a
  second package kind translated from Wasm to MIR. Lints and generators are
  served better by built-in lints and generator packages, which need none of
  that.
- **Capability-gated foreign calls** (each library declares a capability, and
  every call sits inside a `with` block that holds it). Rejected. Verse cannot
  see what foreign code does, so the gate would claim a guarantee it cannot
  keep, and it adds ceremony to every call into `libm`.
- **Process plugins over JSON-RPC.** Language-neutral and isolated, but every
  call pays a process round trip, and the boundary is a protocol instead of
  a function call.
- **A Verse-specific plugin ABI.** Would let plugins pass `Text` and
  collections, but every plugin language would need an adapter, and the
  internal ABI could no longer change between compiler versions.

## Consequences

- Any language that exports C functions can write a plugin, with no Verse
  tooling on its side.
- Foreign code runs with the program's full access. Ownership, bounds checks
  and `--deny` do not reach inside it.
- The toolchain embeds no Wasm engine. The VM needs a small loader and call
  stubs (`verse-ffi`).
- The browser Wasm target cannot use plugins; WASI and microcontrollers link
  them statically.
