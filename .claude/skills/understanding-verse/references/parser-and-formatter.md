# Parser and formatter

The front of the pipeline: the lexer, the lossless CST, the parser, AST
lowering, and `verse fmt`, which formats from the CST.

## Read first

1. [02 §2.1 Pipeline](../../../../docs/architecture/compiler-architecture.md#21-pipeline)
   and [02 §2.2 Stage contracts](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts),
   the Lexer, Parser and AST lowering rows.
2. [02 §2.3 Data structures](../../../../docs/architecture/compiler-architecture.md#23-data-structures):
   the CST, arenas, spans.
3. [03 §3.1 The representations](../../../../docs/architecture/ir-and-passes.md#31-the-representations):
   tokens, CST and AST side by side.
4. Spec [§3.1](../../../../docs/language/03-lexical.md#31-source-files) to
   [§3.6](../../../../docs/language/03-lexical.md#36-keywords): source files,
   comments, indentation, line continuation, identifiers, keywords.
5. Spec [§3.7 Literals](../../../../docs/language/03-lexical.md#37-literals),
   with escapes, interpolation and raw text.
6. Spec [§17 EBNF grammar sketch](../../../../docs/language/17-grammar.md#17-ebnf-grammar-sketch).
7. Spec [§16.1 The formatter](../../../../docs/language/16-tooling.md#161-the-formatter).
8. The V01xx and V02xx tables of the
   [diagnostics catalog](../../../../docs/reference/diagnostics-catalog.md#v01xx-source-text).

## What it does

| Stage | Input | Output |
|-------|-------|--------|
| Lexer | valid UTF-8 | balanced `INDENT`/`DEDENT`, tabs rejected, every byte covered by a token or trivia |
| Parser | token stream | a CST whose text round-trips byte for byte; error nodes where parsing failed, never a panic |
| AST lowering | CST | each node has a stable `NodeId` and span; trivia discarded |

Source: [02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts).

- A source file is UTF-8 text with the extension `.vs`, and each file is a
  module. Line endings may be `\n` or `\r\n`
  ([§3.1](../../../../docs/language/03-lexical.md#31-source-files)).
- The crate `verse-syntax` owns the lexer, CST, parser and AST. The crate
  `verse-fmt` owns the formatter, which is CST based
  ([02 §2.9](../../../../docs/architecture/compiler-architecture.md#29-crate-layout)).
- The CST serves the formatter, the language server and error recovery. The
  compiler proper uses the AST
  ([03 §3.1](../../../../docs/architecture/ir-and-passes.md#31-the-representations),
  [02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)).
- `verse fmt [path]` formats in the one canonical style. `--check` is for CI
  ([§16 Tooling](../../../../docs/language/16-tooling.md#16-tooling)).
- No code exists yet. The roadmap puts the lexer, CST, parser for the whole
  grammar and the `verse fmt` skeleton in M0
  ([roadmap](../../../../docs/roadmap.md#11-roadmap)).

## How it works

**Lexer.** It turns leading whitespace into `INDENT` and `DEDENT` and ends
logical lines with `NEWLINE`. It suppresses `NEWLINE` inside brackets, after a
trailing binary operator, and before a line that begins with `.` or `|`.
`BLANK` is an empty line, significant only after a module doc
([§17](../../../../docs/language/17-grammar.md#17-ebnf-grammar-sketch)).
Why: the syntax is indentation-significant and must stay friendly to
lossless tools
([01 constraint 1](../../../../docs/architecture/requirements.md#constraints-taken-from-the-specification)).

**Indentation.** Spaces only. A tab in leading whitespace is an error. The
canonical width is 4 spaces. A block's lines share one indentation. An empty
block is an error; `pass` marks an intentionally empty one
([§3.3](../../../../docs/language/03-lexical.md#33-indentation-and-blocks)).

**Line continuation.** A statement continues when the line ends inside an
open bracket, ends with a binary operator, or the next line starts with `.`
or `|`. A line that ends with `->` opens a block. Inside an open `(`, the
dedent that ends a lambda block also closes that `(` (the closing rule). A
leading dot that starts an expression is a key-path lambda, not a
continuation. `type X =` never opens a field block; only `type X` with
nothing after the name does
([§3.4](../../../../docs/language/03-lexical.md#34-line-continuation)).

**Identifiers.** The first character decides the kind. An uppercase first
letter (Unicode Lu or Lt) is a type-level name. Every other identifier is a
value-level name. A single `_` is a wildcard
([§3.5](../../../../docs/language/03-lexical.md#35-identifiers-and-naming)).

**Comments.** `#` starts a comment. `##` starts a doc comment. A `##` block at
the top of a file followed by a blank line is the module doc. There are no
block comments ([§3.2](../../../../docs/language/03-lexical.md#32-comments)).

**Grammar.** The grammar is a readable sketch, not a complete formal
definition. Rules marked *(semantic)* are checked after parsing
([§17](../../../../docs/language/17-grammar.md#17-ebnf-grammar-sketch)).
Examples of parser-relevant choices in the sketch:

- `import a as b` imports a module, while `import a as b from m` imports an
  item. The trailing `from` on the same logical line decides.
- A lowercase receiver never takes explicit type arguments, so
  `a.b < c > (d)` is always a chained comparison.
- `try` is a prefix word below the postfix chain and above `**`, so
  `try a.b(c)` parses as `try (a.b(c))`; `?` appears only in types and
  patterns.

**CST.** A green tree, structurally shared and cheap to re-parse after an
edit. Printing it gives back the file byte for byte. Why: the formatter and
language server need trivia, and the hot path of the compiler must never
touch it, so the compiler works on the AST
([02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures),
[00 Glossary](../../../../docs/architecture/overview.md#glossary)).

**AST.** Nodes live in per-module arenas (`Vec<Node>` addressed by
`NodeId(u32)`), with no `Rc` and no per-node heap allocation. Identifiers and
text literals are interned to `u32` ids. Spans are
`(file_id: u32, start: u32, end: u32)`; line and column come from a per-file
line table on demand. These choices are assumptions until the M0 benchmarks
run ([02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)).

**Parallelism and caching.** The module is the unit of parallelism for
parsing. Every pure stage is a memoized query keyed by content hash.
`.verse/cache/` stores parsed modules
([02 §2.7](../../../../docs/architecture/compiler-architecture.md#27-incremental-and-parallel-compilation)).

**Formatter.** `verse fmt` has no configuration options. It normalizes
indentation to four spaces, spacing around operators, blank lines between
declarations and trailing commas in multi-line lists. It wraps long method
chains one call per line, sorts imports, enforces the blank line after a
module doc, and drops the redundant `)` that the lambda closing rule makes
unnecessary ([§16.1](../../../../docs/language/16-tooling.md#161-the-formatter)).
Why no options: every Verse file looks the same
([§16.1](../../../../docs/language/16-tooling.md#161-the-formatter)).

## Rules that must hold

- The parser never panics. It produces error nodes and recovers at line and
  block boundaries
  ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
- The CST round-trips the source byte for byte
  ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
- Every byte is covered by a token or trivia, and `INDENT`/`DEDENT` are
  balanced ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
- Nothing in the front end ever sees a target triple
  ([02 §2.1](../../../../docs/architecture/compiler-architecture.md#21-pipeline)).
- The formatter never removes semantically significant parentheses: those
  that end a lifted chain, and those in `(try a).b`
  ([§16.1](../../../../docs/language/16-tooling.md#161-the-formatter)).
- The formatter never inserts or removes type annotations, and never touches
  `verse.api`
  ([§16.1](../../../../docs/language/16-tooling.md#161-the-formatter),
  [§16.4](../../../../docs/language/16-tooling.md#164-interface-files-verseapi)).
- `verse fmt` is idempotent and never changes meaning
  ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers),
  [roadmap M0](../../../../docs/roadmap.md#11-roadmap)).
- Tools must render a reassignment differently from a new binding, because
  both are written `x = …`. This is a tooling requirement, not syntax
  ([§3.9](../../../../docs/language/03-lexical.md#bindings-and-assignment)).
- `verse fmt --check` enforces `snake_case` and `PascalCase` naming
  ([§3.5](../../../../docs/language/03-lexical.md#35-identifiers-and-naming)).
- The lexer and parser report only V01xx and V02xx codes
  ([16 §16.2.1](../../../../docs/reference/diagnostics-catalog.md#1621-families-by-pass)).
  AST lowering reports only V99xx.

## What depends on it

- Name resolution consumes the AST of all modules in a package. It trusts
  stable `NodeId`s and spans
  ([02 §2.2](../../../../docs/architecture/compiler-architecture.md#22-stage-contracts)).
  See [names-and-types.md](names-and-types.md).
- The language server uses the CST
  ([00 Glossary](../../../../docs/architecture/overview.md#glossary)).
- Every diagnostic span and every debug position comes from AST spans
  ([02 §2.3](../../../../docs/architecture/compiler-architecture.md#23-data-structures)).
- `tools/extract-examples` runs every ```` ```verse ```` block of the spec, so
  a grammar change can break spec examples
  ([10 §10.2](../../../../docs/architecture/testing-and-verification.md#102-specification-conformance)).
- Golden snapshots of token streams and CST dumps live under `tests/golden/`
  ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)).
- Upstream, the lexer trusts nothing: its input invariant is only valid UTF-8,
  and V0109 reports a file that is not
  ([catalog V01xx](../../../../docs/reference/diagnostics-catalog.md#v01xx-source-text)).

## After a change

1. Add the test layers that apply
   ([10 §10.1](../../../../docs/architecture/testing-and-verification.md#101-test-layers)):
   - unit tests with `cargo test`;
   - golden snapshots of token streams and CST dumps under `tests/golden/`;
   - property tests with `proptest`: formatter idempotence, parse and print
     round trip, `verse fmt` never changes meaning;
   - fuzzing of the lexer and parser with grammar-aware generators;
   - a conformance test for every normative sentence you implement
     ([10 §10.2](../../../../docs/architecture/testing-and-verification.md#102-specification-conformance)).
2. If the change alters syntax, change the spec chapter, the
   [§17 grammar](../../../../docs/language/17-grammar.md#17-ebnf-grammar-sketch)
   and a conformance test in the same commit
   ([10 §10.2](../../../../docs/architecture/testing-and-verification.md#102-specification-conformance)).
3. A new error goes in V01xx (lexer) or V02xx (parser), follows the catalog
   rules, and gets its three tests. See [diagnostics.md](diagnostics.md).
4. Use the commit format of
   [CONTRIBUTING](../../../../CONTRIBUTING.md#commit-format), for example
   `feat(compiler): parse extern blocks`. A syntax change to the language is
   type `spec`. The docs give no scope for `verse-fmt`; the area list has
   `compiler` and `tooling`.
5. If you changed docs, run `tools/docs/check.sh`.

If the code and the docs disagree, the docs win until a `spec` pull request changes them.
