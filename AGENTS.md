# AGENTS.md

This is the root entrypoint for AI coding agents working in Runenwerk through a local checkout, GitHub connector, ChatGPT context tooling, or patch-based workflow.

## Read first

For non-trivial work, read:

```text
docs-site/src/content/docs/workspace/engineering-workflow.md
```

Then inspect the authorities relevant to the change:

```text
ARCHITECTURE.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
CRATES.md
TESTING.md
```

Use accepted ADRs and design documents for durable subsystem decisions. Use GitHub issues and the active roadmap for current work state.

## Operating model

Work from repository evidence rather than memory:

```text
identify the owner and boundary
-> inspect current code, tests, and accepted docs
-> classify the change as routine, significant, or architectural
-> implement a bounded coherent slice
-> run focused checks
-> run cargo validate before merge
-> report exact evidence and remaining gaps
```

Do not require generated prompts, contract packs, execution locks, truth certificates, batch manifests, or local workflow state to understand or authorize work.

## Change classes

### Routine

Local fix or behavior-preserving refactor with a clear owner and unchanged dependency direction. A separate design is not required.

### Significant

Cross-module or durable behavior change. Record the owner, boundary, alternatives, acceptance criteria, migration impact, and validation in one appropriate authority: issue, design, ADR, or PR body.

### Architectural or extraction

New repository/crate, reusable platform contract, public API, dependency-direction change, host/renderer boundary, or workflow-authority change. Inspect current reality and establish the complete target ownership, alternatives, migration, cutover, deletion, and conformance plan before implementation.

## Repository conventions

- Organize code by domain responsibility.
- Preserve one-way dependency direction.
- Prefer explicit types and discoverable public APIs.
- Reuse nearby patterns before adding abstractions.
- Do not add compatibility layers without a real consumer and removal condition.
- Do not create new crates or shared foundations without accepted architectural ownership.
- Keep product policy in Runenwerk and reusable framework semantics in the owning peer repository.
- Do not place repository workflow policy in application or domain tests.

## Validation

Use focused commands while implementing, for example:

```text
cargo test -p <package>
cargo clippy -p <package> --all-targets -- -D warnings
python tools/docs/validate_docs.py
```

The required baseline is:

```text
cargo validate
git diff --check
```

`cargo validate` is the same locked, read-only implementation used by GitHub Actions. See `TESTING.md`.

Use the extended profile only when broader evidence is required:

```text
cargo xtask validate --extended
```

Runenwerk does not use quick, full, or quiet gates. Output verbosity never changes validation semantics.

## Retired workflow systems

Track locks, execution contract packs, truth certificates, batch execution, and generated prompts are not part of the repository workflow. Historical references are context only.

## Reporting

Report:

```text
files and exact owners changed
behavior or authority changed
focused validation run
cargo validate and CI status
public API or dependency impact
remaining risks or blockers
next concrete action
```

Do not claim commands, CI, local branch cleanup, or runtime behavior that were not actually observed.
