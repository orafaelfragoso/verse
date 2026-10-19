# Verse implementation architecture

This folder is the engineering counterpart of the language specification
(`../00-README.md` and sections 1–21). The specification says **what Verse
means**. These documents say **how we build it**. When the two disagree, the
specification wins and the disagreement is filed in
[12-open-questions.md](12-open-questions.md).

Status: **draft 1**, written against spec 0.1 (5,093 lines). Nothing here
has been implemented yet, so every number is a *target*, not a measurement.
Facts, assumptions and hypotheses are labelled as such.

| # | Document | What it answers |
|---|----------|-----------------|
| 00 | [Overview](00-overview.md) | Plain-language summary, one diagram, how R1–R7 are met, principles, glossary |
| 01 | [Requirements](01-requirements.md) | Rafael's seven requirements, made testable, and where each one is satisfied |
| 02 | [Compiler architecture](02-compiler-architecture.md) | The pipeline, stage contracts, invariants, incremental compilation |
| 03 | [IR and pass specification](03-ir-and-passes.md) | CST, AST, HIR, THIR, generic MIR, mono MIR, and each pass |
| 04 | [Execution tiers and backends](04-backends-and-targets.md) | Script mode, native, WebAssembly, ARM, x86, RISC-V, microcontrollers |
| 05 | [Runtime](05-runtime.md) | Memory, tasks, panics and unwinding, feature slices, embedded profile |
| 06 | [Standard library and tree-shaking](06-stdlib-and-shaking.md) | Optional domain libraries, how builds include only what they use |
| 07 | [Plugins](07-plugins.md) | Sandboxed extension points and their safety model |
| 08 | [Debugging and MCP](08-debugging-and-mcp.md) | A debugger and compiler that LLMs can drive step by step |
| 09 | [Performance](09-performance.md) | Budgets, benchmark suite, known spec-level performance risks |
| 10 | [Testing and verification](10-testing-and-verification.md) | Conformance, differential testing, fuzzing, soundness checks |
| 11 | [Roadmap](11-roadmap.md) | Milestones with dependencies and exit criteria |
| 12 | [Open questions](12-open-questions.md) | Spec-level decisions that need an explicit ruling before we rely on them |
| 19 | [CLI, manifests and package manager](19-cli-and-manifest.md) | The `verse` commands, `verse.toml`, `verse.lock`, `verse.api` versioning, registry, resolution |
| 20 | [First-party packages](20-first-party-packages.md) | API sketches for `finance`, `ml`, `web`, `gui`, `iot` |
| adr/ | [Architecture decision records](adr/) | One file per major decision, with alternatives and trade-offs |
| adr/0001 | [Implementation language: Rust](adr/0001-implementation-language.md) | Why Rust for compiler, runtime and tooling |
| adr/0004 | [Whole-program reachability](adr/0004-whole-program-reachability.md) | Why tree-shaking runs on optimized mono MIR |

## Reading order

Start with 01 and 02. Read ADR-0001 to ADR-0004 if you want to challenge a
decision. Read 12 last: it lists everything that needs a ruling from the
language owners.

## Conventions

- **Decision**: chosen and recorded as an ADR.
- **Target**: a measurable goal we have not yet measured.
- **Assumption**: believed true, not yet verified. Each one has an owner
  milestone in the roadmap that verifies it.
- **Proposal**: needs a ruling before it is treated as established. These are
  collected in document 12.
