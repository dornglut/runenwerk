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

This file names the current planning focus. Detailed historical UI execution
records remain in closeouts, completed-work, pull requests, and Git history; they
are not current implementation authority.

## Current focus

ID: `PT-REPOSITORY-FAMILY-000`

Title: Repository Family Charter and Track Activation

State: `active-implementation` for one documentation/authority PR only

Owner: workspace architecture and planning

Current branch:

```text
docs/repository-family-charter
```

Goal:

```text
Establish one canonical repository-family architecture and activate three bounded
parallel tracks at different maturity levels:

RunenSDF    complete investigation/design, then extract first
RunenECS    complete investigation and architecture decisions
RunenRender complete semantic inventory and internal decomposition design
```

RunenUI is explicitly outside this workstream and remains governed by its
separate repository/thread.

## Authority

```text
docs-site/src/content/docs/architecture/repository-family-architecture.md
docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
docs-site/src/content/docs/reports/investigations/repository-family-current-state-investigation.md
docs-site/src/content/docs/design/active/runensdf-extraction-design.md
docs-site/src/content/docs/design/active/runenecs-extraction-boundary-design.md
docs-site/src/content/docs/design/active/runenrender-decomposition-design.md
```

## Decisions fixed by this phase

```text
Runenwerk remains the integration/product repository.
Framework repositories must not depend on Runenwerk.
No universal RunenCore/shared-meta repository is created.
RunenSDF is the first extraction candidate.
RunenECS does not retain ECS-owned spatial indexing or Runenwerk geometry.
The generic scheduler remains a separate package in the RunenECS repository.
RunenRender is internally decomposed before external extraction.
RunenRender must not depend on ECS, SDF, UI, scene, material authoring, or Runenwerk.
All extractions use one clean cutover with no source mirror or compatibility crate.
```

## Implementation contract

This phase changes architecture, ADR, investigation, design, and planning
Markdown only.

It must:

- replace obsolete `PT-UI-RUNTIME-PLATFORM-012` active implementation authority;
- record PR #107 as closed/unmerged historical evidence;
- record commit `b5e9624c...` as a rejected incomplete extraction attempt;
- make repository-family sequencing understandable from Markdown alone;
- keep historical closeouts and completed-work evidence intact;
- avoid Rust, Cargo, lockfile, workflow, tool, or generated-file changes;
- pass documentation build/validation and diff hygiene.

## Allowed files

```text
docs-site/src/content/docs/architecture/repository-family-architecture.md
docs-site/src/content/docs/adr/accepted/0014-repository-family-extraction-boundaries.md
docs-site/src/content/docs/reports/investigations/repository-family-current-state-investigation.md
docs-site/src/content/docs/design/active/runensdf-extraction-design.md
docs-site/src/content/docs/design/active/runenecs-extraction-boundary-design.md
docs-site/src/content/docs/design/active/runenrender-decomposition-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
root architecture/planning summaries only if required for current-truth alignment
```

## Forbidden files and scope

```text
Cargo.toml
Cargo.lock
engine/**
domain/**
apps/**
net/**
foundation/**
.github/**
tools/**
RunenUI repository changes
RunenSDF/RunenECS/RunenRender repository creation
any extraction implementation
any source move/delete
```

## Evidence and gates

Evidence classes:

```text
E2 GitHub repository/commit/PR metadata
E3 current source, manifest, and authority inspection
E8 accepted architecture/ADR/planning authority after this phase is reviewed
```

Complete investigation gate:

```text
Complete for repository-family direction and track ordering.
Not complete for RunenSDF consumer-level implementation authorization.
Not complete for RunenECS extraction.
Not complete for RunenRender decomposition implementation.
```

Complete design gate:

```text
Complete for the repository-family architecture and planning reset.
RunenSDF target direction is fixed but implementation awaits SDF-001 inventory.
RunenECS and RunenRender remain investigation/design tracks.
```

Validation envelope:

```text
pnpm --dir docs-site build
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Cargo validation is not required for this docs-only phase unless a non-document
file changes unexpectedly.

## Stop conditions

Stop this phase if:

- current docs schema rejects a new owner/layer/status value;
- a current accepted ADR contradicts the fixed dependency direction;
- source inspection reveals a dependency cycle that invalidates track ordering;
- the diff touches source, manifests, lockfiles, workflows, or tools;
- documentation validation cannot be run or fails for reasons introduced here.

## Next actions after merge

Exactly three bounded follow-ups become active:

1. `PT-RUNENSDF-001` — complete SDF source, test, consumer, and public API investigation;
2. `PT-RUNENECS-001` — complete ECS/macros/scheduler/spatial/network/replay investigation;
3. `PT-RUNENRENDER-001` — complete renderer module/shader/control-flow inventory.

Only RunenSDF may advance directly toward boundary-correction implementation once
its investigation and design gate are accepted.
