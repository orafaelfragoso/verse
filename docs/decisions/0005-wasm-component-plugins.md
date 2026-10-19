# ADR-0005: Plugins are sandboxed Wasm components

Status: accepted (2026-10-18, all three owners).

## Context

Requirement R3: extend the language with plugins. Rules agreed in the room:
plugins add libraries, compiler passes and tooling, never syntax or operators;
dependencies cannot run code at build time (spec §16.3).

## Decision

Plugins are **WebAssembly components** loaded by the toolchain in a sandbox
(wasmtime), described by WIT interfaces, with explicit capability grants in the
root project's `verse.toml`. Details in [07](../architecture/plugins.md).

## Alternatives

- **Native dynamic libraries.** Fast and simple, but unsandboxed, unportable
  across hosts, and a supply-chain risk.
- **Scripts in Verse itself run at build time.** Elegant, but conflicts with
  §16.3 ("building a dependency cannot run arbitrary code") unless sandboxed,
  and the language has no sandbox yet.
- **Rust procedural macros style.** Grammar and syntax extension, which the
  room rejected.
- **Process plugins over JSON-RPC.** Language-neutral, easy to debug, slower,
  and still unsandboxed without OS support.

## Consequences

- Plugin authors can write in any language that targets components, including
  Verse once M3 exists.
- Hosting wasmtime adds size to the toolchain; the plugin host is a separate
  Cargo feature and a separately downloaded component.
- Plugin interfaces are versioned (`verse:plugin/lint@0.1.0`).


## Addendum: libraries in native binaries

A Wasm-implemented package (an ordinary dependency, not a plugin) is translated ahead of time to mono MIR (a Wasm frontend), so native and script builds contain no Wasm engine. Only the compiler host embeds wasmtime, to run the four plugin kinds: `lint`, `generator`, `bindgen` and `tool`. See [07 §7.4](../architecture/plugins.md).
