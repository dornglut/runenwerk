---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../specs/pt-runensdf-003-standalone-transfer.ron
  - ../../guidelines/programming-principles.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runensdf-repository-identity-decision.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

## Completed primary implementation

ID: `PT-RUNENSDF-003`

Title: Standalone RunenSDF Repository and Corrected Source Transfer

Lifecycle state: `complete`

Accepted standalone revision:

```text
repository: Crystonix/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
package: runen-sdf
crate: runen_sdf
```

The standalone repository now contains the corrected implementation, all nine integration-test modules, public downstream conformance, independent lockfile, framework documentation, licensing, security policy, repository-policy validation, and durable `cargo validate` workflow.

Automatic GitHub Actions evidence passed in runs `29845971330` and `29846386222`. The private target repositories failed before runner allocation; no manual owner validation was substituted.

Detailed evidence is recorded in `../../reports/closeouts/pt-runensdf-003-standalone-transfer-closeout.md`.

## Next bounded phase

ID: `PT-RUNENSDF-004`

Title: Runenwerk Clean Cutover to Standalone RunenSDF

Lifecycle state: `queued-planning`

Implementation authorization: **none until a complete PT-RUNENSDF-004 phase specification and final consumer audit are accepted**

The phase must:

1. repeat the complete Runenwerk consumer and manifest audit;
2. classify every real source, test, documentation, adapter, and persisted consumer;
3. pin real consumers to exact revision `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`;
4. remove `domain/sdf` from workspace membership and old lockfile authority;
5. delete internal source, tests, and stale framework-owned documentation;
6. prove no forwarding package, crate alias, source include, submodule, branch dependency, or duplicate implementation remains;
7. validate the complete Runenwerk integration automatically.

If the final audit still finds no production consumer, the phase removes the isolated internal package without adding an unused external dependency.

## Program allocation

```text
RunenSDF     PT-RUNENSDF-003 complete; PT-RUNENSDF-004 queued planning
RunenECS     R1 specified; no Rust implementation authorized
RunenRender  R1 specified; no Rust implementation authorized
RunenUI      independent workstream outside this program
```

RunenSDF module regrouping, publication, GPU work, renderer work, and unrelated repository cleanup remain outside PT-RUNENSDF-004.