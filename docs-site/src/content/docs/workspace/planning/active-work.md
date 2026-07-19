---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names current execution authority. Historical UI work remains in
completed-work, closeouts, merged pull requests, and Git history.

## Current focus

ID: `PT-REPOSITORY-FAMILY-000`

Title: Repository Family Charter and Track Activation

Lifecycle state: `active-planning`

Implementation authorization: documentation and authority changes only

Owner: workspace architecture and planning

Branch:

```text
docs/repository-family-charter
```

## Goal

Establish one repository-family architecture and replace obsolete active UI
product authority with three bounded tracks at different maturity levels:

```text
RunenSDF     architecture ready; executable investigation verification pending
RunenECS     architectural investigation in progress
RunenRender  semantic investigation in progress
RunenUI      independent workstream; repository relationship fixed only
```

## Decisions fixed by this phase

- Runenwerk remains the integration and product repository.
- Framework repositories do not depend on Runenwerk.
- RunenSDF is the first extraction candidate.
- RunenECS does not retain Runenwerk geometry or general spatial policy.
- RunenRender is decomposed and proven internally before external extraction.
- RunenUI and RunenRender remain independent peers; Runenwerk owns their future
  integration.
- No universal shared-core repository, source mirror, submodule, or long-lived
  compatibility package is introduced.
- Completed extraction uses one clean cutover.

Track-specific APIs, package details, and implementation mechanisms are not fixed
by this planning phase unless the accepted ADR or architecture document states
them explicitly.

## Allowed scope

```text
root architecture summaries
docs-site architecture and ADR authority
repository-family current-state investigation
track design documents
active-work, roadmap, production-tracks, and decision-register
```

## Forbidden scope

```text
Rust source
Cargo manifests or Cargo.lock
workflows or validation tools
external repository creation
source movement or deletion
RunenUI implementation
SDF, ECS, or renderer implementation
```

## Evidence and gates

Evidence currently available:

```text
E2 GitHub commit and pull-request metadata
E3 connector-backed source, manifest, and authority inspection
```

Gate status:

```text
Repository-family direction investigation: complete for track ordering
Repository-family design: complete after owner review and docs validation
RunenSDF source/API investigation: substantially complete; consumer/command verification pending
RunenECS complete investigation: not complete
RunenRender complete investigation: not complete
Implementation authorization: none
Merge readiness: blocked by owner review and local docs validation
```

The words “complete” and “decision-complete” may be used only for the scope whose
required evidence has actually passed.

## Validation envelope

```text
pnpm --dir docs-site build
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Cargo validation is unnecessary for this docs-only phase unless a non-document
file enters the diff.

## Historical truth

PR #107 is closed and unmerged. It is not current implementation authority.

Commit `b5e9624c594c9f1e3f2a0929bf84028f13fde860` is a rejected incomplete
extraction attempt and is not an implementation base.

Planning-file pruning must not remove unique completion evidence. Completed UI
facts remain in `completed-work.md`, closeout reports, merged PRs, and Git history.

## Next actions after merge

1. Correct and verify `PT-RUNENSDF-001`, then activate one bounded SDF boundary
   correction phase if its local gates pass.
2. Continue `PT-RUNENECS-001` until source, consumer, and safety inventory is
   complete; prepare only the next executable repair spec.
3. Continue `PT-RUNENRENDER-001` until module, shader, consumer, and runtime
   inventory is complete; prepare only the next executable repair spec.

Only RunenSDF may advance directly toward implementation after its verification
and design corrections pass.