# Verse implementation architecture

This folder is the engineering counterpart of the language specification
([language](../language/README.md)). The specification says **what Verse
means**. These documents say **how we build it**. When the two disagree, the
specification wins.

Status: **draft**, written against spec 0.1. Nothing here
has been implemented yet, so every number is a *target*, not a measurement.
Facts, assumptions and hypotheses are labelled as such.

| Document | What it answers |
|----------|-----------------|
| [Overview](overview.md) | Plain-language summary, one diagram, how R1–R7 are met, principles, glossary |
| [Requirements](requirements.md) | The seven requirements, made testable, and where each one is satisfied |
| [Compiler architecture](compiler-architecture.md) | The pipeline, stage contracts, invariants, incremental compilation |
| [IR and pass specification](ir-and-passes.md) | CST, AST, HIR, THIR, generic MIR, mono MIR, and each pass |
| [MIR specification](mir-specification.md) | Generic and mono MIR: text format, types, statements, verifier rules, debug info |
| [Bytecode and VM](bytecode-and-vm.md) | Tier 0: the bytecode format, the interpreter, record and replay |
| [Execution tiers and backends](backends-and-targets.md) | Script mode, native, WebAssembly, ARM, x86, RISC-V, microcontrollers |
| [Target and ABI](target-and-abi.md) | Data layout, calling conventions, unwinding, exit statuses, platform layers |
| [Runtime](runtime.md) | Memory, tasks, panics and unwinding, feature slices, embedded profile |
| [Standard library and tree-shaking](stdlib-and-shaking.md) | Optional domain libraries, how builds include only what they use |
| [First-party packages](first-party-packages.md) | API sketches for `finance`, `ml`, `web`, `gui`, `iot` |
| [Packages](packages.md) | What a package is, how `import` finds one, naming, versioning from `verse.api`, `verse.lock`, resolution, the registry, workspaces |
| [Plugins](plugins.md) | Native C-ABI libraries: `[plugins]`, `extern`, the type mapping, loading in script and compiled mode, `verse bind` |
| [Debugging](debugging.md) | The `verse debug` adapter over DAP, record and replay, panic reports, the debugger's wording |
| [Performance](performance.md) | Budgets, benchmark suite, known spec-level performance risks |
| [Testing and verification](testing-and-verification.md) | Conformance, differential testing, fuzzing, soundness checks |

Reference material (CLI and `verse.toml`, diagnostics, the event
vocabulary) is in [reference](../reference/). Decision records are in
[decisions](../decisions/README.md). Milestones are in the [roadmap](../roadmap.md).

## Reading order

Start with the overview and the compiler architecture. Read ADR-0001 to
ADR-0004 if you want to challenge a decision.

## Conventions

- **Decision**: chosen and recorded as an ADR.
- **Target**: a measurable goal we have not yet measured.
- **Assumption**: believed true, not yet verified. Each one has a milestone
  in the roadmap that verifies it.
- **Proposal**: needs a ruling before it is treated as established.
