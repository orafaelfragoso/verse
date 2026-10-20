# ADR-0013: Lossless syntax with Rowan and a handwritten parser

Status: proposed.

## Context

The stage contracts of [02 §2.2](../architecture/compiler-architecture.md#22-stage-contracts)
require a lossless CST, recovery without panics, and a trivia-free arena AST.
The formatter and language server need a lossless, immutable green tree that
is cheap to clone across edits.

## Decision

`verse-syntax` uses Rowan for the green tree. The parser builds green nodes
directly from a local checkpoint stack instead of using Rowan's hash-consing
builder, because hashing every nested node is recursive on a deep valid tree.
A handwritten scanner preserves physical tokens and trivia. A separate layout
pass inserts virtual indentation, logical newline and lambda-closing tokens.
Virtual tokens have empty text. Declarations and statements use recursive
descent. Expressions use binding powers from spec §3.8. Recovery stops at
logical line and block boundaries.

`verse-syntax::source` owns file IDs, byte spans and per-file line tables.
`verse-syntax::diagnostic` owns the shared diagnostic record and its terminal
renderer. Consumers import those records rather than depending on the CLI.
The arena AST interns token spellings and omits trivia and layout tokens.

Stacker grows the parser's call stack on demand. A deeply nested valid program
does not become a syntax error because of an arbitrary parser nesting limit.
The green tree's drop is iterative for pathological nesting, so a successful
parse also cannot overflow the stack while releasing the tree. Delimiter
lookahead is precomputed rather than repeatedly scanning nested expressions.
AST lowering uses an explicit work stack and flat edge and token arenas, with
no per-node vectors. Token interning records each token's original span.

Rowan, Stacker, Unicode general categories, Serde and the test dependencies are declared
at the workspace root. `Cargo.lock` records their resolved versions.

## Alternatives

- A custom green tree duplicates immutable-node representation, text ranges
  and traversal.
- A parser generator still needs custom indentation and lambda-closing logic.
- Formatting an AST loses comments and source grouping that the CST preserves.

## Consequences

The syntax crate has no target information and no dependency on later stages.
The formatter consumes its CST. The CLI consumes its diagnostics.
Round-trip, conformance, recovery, property and fuzz tests exercise the stage
contracts. The benchmark harness measures the representations rather than
assuming their performance.
