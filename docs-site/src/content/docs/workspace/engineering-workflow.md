---
title: Engineering Workflow
description: Canonical risk-scaled workflow and validation authority for Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ./start-here.md
  - ./authority-model.md
  - ./documentation-structure.md
---

# Engineering Workflow

This document is the canonical workflow authority for Runenwerk.

The repository uses ordinary engineering artifacts:

```text
GitHub issues and pull requests manage work
ADRs and design documents own durable architecture decisions
code and tests own current behavior
cargo validate owns the required validation baseline
CI proves the same baseline at the reviewed commit
```

Repository scripts may automate checks. They do not create implementation authority, grant permissions, certify truth, or replace review.

## Core rules

1. Identify the owner and boundary before changing shared behavior.
2. Scale investigation and documentation to the risk of the change.
3. Keep one source of authority for each decision or state.
4. Use focused checks while iterating and the baseline before merge.
5. Do not claim validation that was not run.
6. Do not preserve obsolete paths without an explicit compatibility need and removal condition.
7. Prefer a coherent bounded slice over a broad change with mixed ownership.

## Work classes

### Routine

Use for local fixes and behavior-preserving refactors with a clear owner and unchanged dependency direction.

Required record:

```text
problem
changed behavior or structure
focused tests
baseline status before merge
```

A separate design document is not required.

### Significant

Use when a change affects several modules, durable behavior, a public surface, persistence, host integration, or a major product path.

Required record:

```text
problem and intended outcome
owner and affected boundaries
important alternatives and selected approach
acceptance criteria
migration or compatibility impact
validation evidence
```

This record may live in an issue, accepted design, ADR, or PR body. Do not duplicate it across all four.

### Architectural or extraction

Use for new repositories or crates, framework extraction, reusable platform contracts, dependency-direction changes, renderer/host boundaries, durable public APIs, or authority changes.

Required before implementation:

```text
current-state investigation grounded in code and tests
complete target ownership and dependency direction
alternatives and trade-offs
public and internal vocabulary
migration, cutover, and deletion plan
conformance and acceptance evidence
explicit non-owned responsibilities
```

The target design must be complete enough to prevent ownership ambiguity. Delivery may still use bounded implementation PRs.

## Lifecycle

Operational work uses five states:

```text
proposed
active
blocked
done
deferred
```

Durable documents may additionally be:

```text
accepted
superseded
archived
```

Do not create additional states unless they cause a distinct review, release, or compatibility decision.

## Standard flow

### Routine change

```text
branch
-> implement
-> focused checks
-> cargo validate
-> pull request
-> review
-> squash merge
```

### Significant change

```text
issue or accepted design
-> bounded implementation branch
-> tests and documentation
-> cargo validate
-> pull request
-> review against acceptance criteria
-> squash merge and close issue
```

### Architecture or extraction

```text
investigation and design or ADR
-> explicit acceptance
-> implementation issue
-> bounded implementation PRs
-> conformance and cutover proof
-> old-authority deletion
```

A separate planning or closeout PR is required only when it carries independently reviewable truth. Do not create process-only PRs by default.

## Validation profiles

Runenwerk does not use `quick`, `full`, or `quiet` gates. Those names are ambiguous and drift over time.

### Focused

Run the smallest checks that exercise the changed owner while implementing.

Examples:

```text
cargo test -p <package>
cargo clippy -p <package> --all-targets -- -D warnings
python tools/docs/validate_docs.py
```

Focused checks accelerate iteration. They are not the merge baseline.

### Baseline

Every merge requires:

```text
cargo validate
git diff --check
```

`cargo validate` is read-only and lockfile-safe. It runs:

```text
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
repository documentation validation
deterministic repository audit
```

The same command runs locally and in GitHub Actions. CI is authoritative for the reviewed commit.

### Extended

Run manually or on a schedule when the change or release needs broader evidence:

```text
cargo xtask validate --extended
```

The extended profile may include dependency policy, unused-dependency analysis, external links, structural AST rules, and the docs-site production build. These checks are not ordinary PR blockers unless an accepted policy promotes a specific check into the baseline.

Output verbosity never changes which checks are required.

## Pull-request contract

A PR must state:

```text
what changed
why it changed
owner and boundary impact
validation actually run
known risks or deferred work
```

Before recommending merge, verify:

```text
scope matches the issue or design
no unrelated changes are included
acceptance criteria are met
public and dependency impacts are explicit
required tests and documentation are present
cargo validate passes at the reviewed head
CI passes at the reviewed head
no unresolved correctness or ownership findings remain
post-merge state is truthful
```

Use squash merge for bounded feature branches unless preserving individual commits has a specific value.

## Evidence language

Use concrete evidence labels in prose when they matter:

```text
connector inspection
manual source inspection
local command result
CI result
user-reported result
not verified
```

Confidence matrices and certificates are not required. A claim must simply identify the evidence that supports it and any important gap.

## Repository authority

- Code, tests, fixtures, and runtime evidence own current behavior.
- Accepted ADRs and designs own durable decisions.
- GitHub issues own live work; the manually maintained roadmap owns high-level sequencing.
- Pull requests own review and delivery evidence, not long-term architecture.
- Historical generated views are evidence only and are not maintained.
- Historical reports and superseded documents do not authorize new work.

See [Authority Model](authority-model.md) for conflict resolution.

## Retired workflow systems

Runenwerk does not use production-track state machines, execution contract packs, track locks, truth certificates, batch execution, generated worker prompts, or local workflow ledgers. These systems were retired under issue `#122`.

Historical reports may mention them, but those references provide context only. New work uses GitHub issues and pull requests, accepted ADRs/designs, the maintained roadmap, tests, `cargo validate`, and exact-head CI.

## Stop conditions

Stop and resolve the issue before implementation or merge when any of these remain unknown:

```text
owner or dependency direction
public API or persistence impact
migration or deletion path for an architectural change
acceptance criteria
required validation
reviewed-head CI status
```

Unknowns should be recorded as blockers, not converted into assumptions.
