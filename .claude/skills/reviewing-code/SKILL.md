---
name: reviewing-code
description: Reviews code by verifying every finding against the actual code before reporting it. Use when reviewing a diff, pull request, milestone, or subsystem; when checking whether findings from another reviewer actually exist; or when a verdict is needed on whether something is correct and complete. Covers reading the contract first, reproducing claims with minimal inputs, classifying findings, and writing an evidence-backed report.
---

# Reviewing code

A review reports the truth about the code, with evidence. It does not fix, and
it does not accept a claim it has not reproduced.

## Read the contract first

Read what the code is supposed to do before reading the code: the spec,
architecture, ADRs, and CLI or diagnostics reference. Every finding cites the
rule it breaks; without the rule, you are reporting taste. In this repository,
load `understanding-verse` and `writing-tests` and follow the references they
name.

## Verify before you claim

A claim exists only when a minimal input reproduces it on the current commit.

- Cite `file:line` and confirm that line does what you say.
- Build a minimal repro in `/tmp`: a scratch crate for library claims, a CLI
  invocation for command claims, a small file for formatter claims. Do not
  modify the repository.
- Record observed vs expected output with the doc or spec citation.
- A missing file or wrong line or count makes the finding unproven; say so.

## Prefer executable evidence

Shrink every repro to the smallest input that still triggers the behavior. Run
the cheap gates to know the baseline: `cargo test --workspace`, `cargo fmt
--all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D
warnings`. Run a short fuzz budget only to confirm a specific suspicion; do not
run long fuzz or benchmark campaigns. For formatter or layout claims, dump the
CST first (`cargo run -p verse-syntax --example inspect`) so you know where
tokens land before asserting anything about indentation.

## Classify every finding

- **Confirmed** — a minimal input reproduces it on the current commit.
- **False positive** — it does not reproduce, or the cited code does not exist.
- **Spec-ambiguous** — the behavior exists, but the correct outcome is not
  decided by the docs. Report it as a decision to make, not as a bug.
- **Correct-per-spec** — the behavior matches the docs even though it looks
  wrong.
- **Unproven** — a suspicion you could not confirm; state what would confirm it.

## Write each finding with severity and evidence

Order by severity: BLOCKER / HIGH / MEDIUM / LOW / NIT. Each finding carries:
`file:line`, what is wrong, why it matters, a minimal repro (command or input,
observed, expected), and a suggested fix. Separate confirmed bugs from
suspicions. Do not report style unless it causes a real failure.

## Show negative evidence

Add a short "areas checked with no findings" section so coverage is explicit.
The attacks that failed are part of the result: they tell the reader where the
code was actually checked.

## Report format

Start with a one-paragraph verdict: is the change correct and complete for its
stated criteria, and what is the strongest counterexample or residual risk. Then
findings by severity, then areas checked with no findings. If there are no
blockers, say so plainly. Do not pad.
