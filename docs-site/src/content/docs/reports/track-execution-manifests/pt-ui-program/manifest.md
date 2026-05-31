---
title: PT-UI-PROGRAM Track Execution Manifest
description: Explicit Stage 0-7 execution manifest for the UiProgram platform proof production track.
status: active
owner: ui
layer: workspace / domain/ui
canonical: false
last_reviewed: 2026-05-31
related_docs:
  - ../../../workspace/track-execution-manifest.md
  - ../../../workspace/track-execution-manifests/pt-ui-program.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
related_designs:
  - ../../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../../design/active/ui-program-architecture.md
  - ../../../design/active/ui-program-proof-slice-plan.md
related_reports:
  - ../../implementation-plans/wr-135-ui-program-platform-proof-track-governance-and-activation/plan.md
  - ../../closeouts/pm-ui-program-001-adr-design-promotion/closeout.md
  - ../../roadmap-intake/2026-05-31-activate-pt-ui-program-as-the-uiprogram-/proposal.md
---

# PT-UI-PROGRAM Track Execution Manifest

Machine-readable execution source:
`docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`.
This Markdown report is a human-readable mirror and is not parsed as execution
authority.

## Critical Review

The previous production track shape represented only Stage 6 proof slices. That
was too narrow for the accepted UiProgram architecture because the architecture
requires Stage 0 through Stage 5 design, contract, artifact, host, and migration
planning before runtime proof work can begin.

This manifest now maps the full path:

```text
Stage 0: ADR / Design Promotion
Stage 1: UI Program Contract Design
Stage 2: Control Package Proof Design
Stage 3: Compiler / Runtime Artifact Design
Stage 4: Evaluator / Host Design
Stage 5: Retained UI Migration Design
Stage 6: Runtime proof slices 6A through 6F
Stage 7: Runtime-proven closeout and MaterialProgram handoff
```

Current blockers after Stage 1 closeout:

- `PM-UI-PROGRAM-001` is completed as docs/governance-only bounded
  contract evidence.
- `WR-135` is archived as completed governance activation evidence.
- `WR-136` is archived as completed bounded-contract Stage 1 evidence after
  Manifest Runner V3 `agent_closeout` closeout.
- Manifest Runner V2 `agent_design` created the `WR-136` design/planning plan
  and updated the UI Program Architecture Stage 1 contract sections.
- `PM-UI-PROGRAM-003` through `PM-UI-PROGRAM-013` still name future WR
  candidates; those WR rows do not exist yet.
- `PM-UI-PROGRAM-002` / Stage 1 is completed as bounded-contract design evidence.
- `PM-UI-PROGRAM-003` / Stage 2 is the next legal milestone and is blocked on Track Expansion creating or linking its owning WR.
- no Stage 6 implementation is authorized.
- no shared `foundation/meta` extraction is authorized.

Conclusion: `/goal --track PT-UI-PROGRAM` can now see the full staged execution
path, but it may execute only one legal action at a time and must stop at
unmet gates.

## Track Authority

| Field | Value |
|---|---|
| Track id | `PT-UI-PROGRAM` |
| Title | UI Program Platform Proof |
| Authority level | Planning and sequencing only |
| Target completion quality | `runtime_proven` |
| Owner | `ui` |
| Current next legal action | Create or link the design WR for `PM-UI-PROGRAM-003` Control Package Proof Design; stop before Stage 2 design authoring until that WR and plan exist |

The manifest does not authorize product code, crate creation, runtime behavior,
MaterialProgram implementation, or shared extraction.

## Accepted Design Dependencies

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-135-ui-program-platform-proof-track-governance-and-activation/plan.md`

## Global Forbidden Scope

- no product code from this manifest alone;
- no new crates;
- no crate renames;
- no placeholder future folders;
- no 6A implementation before a dedicated 6A WR and production plan exist;
- no UI runtime implementation before the owning implementation milestone;
- no shared `foundation/meta`;
- no MaterialProgram implementation;
- no RenderPlan substitution for the MaterialProgram second-domain proof;
- no renderer-owned UI or product truth;
- no ECS-owned UI semantics;
- no generic node soup;
- no giant `UiSemanticEvent` enum.

## Global Validation Commands

Run after manifest, roadmap, or production metadata changes:

```text
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
task planning:validate
```

Run before continuing goal execution:

```text
task ai:goal -- --track PT-UI-PROGRAM
task production:plan -- --milestone PM-UI-PROGRAM-001 --roadmap WR-135
```

## Milestone Sequence

Future WR candidate labels are planning labels only. Track Expansion must
allocate real WR rows before those milestones can become implementation or
closeout work.

| Order | Milestone | Stage | Owning WR / candidate | Dependency | Next legal action |
|---|---|---|---|---|---|
| 1 | `PM-UI-PROGRAM-001` ADR / Design Promotion | Stage 0 | `WR-135` | none | Completed governance closeout. |
| 2 | `PM-UI-PROGRAM-002` UI Program Contract Design | Stage 1 | `WR-136` | PM-001 | Completed bounded-contract closeout. |
| 3 | `PM-UI-PROGRAM-003` Control Package Proof Design | Stage 2 | `WR-TBD-UI-PROGRAM-003` | PM-002 | Create/link design WR after PM-002. |
| 4 | `PM-UI-PROGRAM-004` Compiler / Runtime Artifact Design | Stage 3 | `WR-TBD-UI-PROGRAM-004` | PM-003 | Create/link design WR after PM-003. |
| 5 | `PM-UI-PROGRAM-005` Evaluator / Host Design | Stage 4 | `WR-TBD-UI-PROGRAM-005` | PM-004 | Create/link design WR after PM-004. |
| 6 | `PM-UI-PROGRAM-006` Retained UI Migration Design | Stage 5 | `WR-TBD-UI-PROGRAM-006` | PM-005 | Create/link design WR after PM-005. |
| 7 | `PM-UI-PROGRAM-007` 6A Label Structural UiFrame Text Proof | Stage 6A | `WR-TBD-UI-PROGRAM-007` | PM-006 | Create/link 6A WR after PM-006. |
| 8 | `PM-UI-PROGRAM-008` 6B Button Route Event Host Command Proof | Stage 6B | `WR-TBD-UI-PROGRAM-008` | PM-007 | Create/link 6B WR after 6A closeout. |
| 9 | `PM-UI-PROGRAM-009` 6C InspectorField Binding State Proof | Stage 6C | `WR-TBD-UI-PROGRAM-009` | PM-008 | Create/link 6C WR after 6B closeout. |
| 10 | `PM-UI-PROGRAM-010` 6D ColorPicker ControlPackage Proof | Stage 6D | `WR-TBD-UI-PROGRAM-010` | PM-009 | Create/link 6D WR after 6C closeout. |
| 11 | `PM-UI-PROGRAM-011` 6E World Space Host Boundary Proof | Stage 6E | `WR-TBD-UI-PROGRAM-011` | PM-010 | Create/link 6E WR after 6D closeout. |
| 12 | `PM-UI-PROGRAM-012` 6F Headless Fixture Diagnostics Source Map Proof | Stage 6F | `WR-TBD-UI-PROGRAM-012` | PM-011 | Create/link 6F WR after 6E closeout. |
| 13 | `PM-UI-PROGRAM-013` Runtime Proven Closeout And MaterialProgram Handoff | Stage 7 | `WR-TBD-UI-PROGRAM-013` | PM-012 | Create/link closeout WR after 6F closeout. |

## Milestone Details

### PM-UI-PROGRAM-001 - ADR / Design Promotion

| Field | Value |
|---|---|
| Authority level | docs/governance only |
| Predecessor dependency | none |
| Owning WR | `WR-135` |
| Write scope | production metadata, generated production docs, roadmap metadata, generated roadmap docs, WR-135 contract, this manifest, PM-001 closeout |
| Forbidden scope | product code, crates, placeholder folders, runtime implementation, MaterialProgram implementation, shared extraction |
| Required contracts | WR-135 governance contract |
| Evidence gates | completed PM-001 closeout before completion |
| Validation commands | production render/validate/check; roadmap render/validate/check; docs validate; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-001-adr-design-promotion/closeout.md` |
| Stop conditions | completed; stop if any future action tries to use PM-001 as implementation authority |
| Code allowed | no |
| Crate creation allowed | no |
| Production behavior may change | no |

### PM-UI-PROGRAM-002 - UI Program Contract Design

| Field | Value |
|---|---|
| Authority level | design only |
| Predecessor dependency | `PM-UI-PROGRAM-001` |
| Owning WR | `WR-136` |
| Write scope | exact PM-002 design/planning scope in `WR-136`, including UiProgram contract design docs, implementation-plan report, roadmap/production metadata, generated planning docs, manifest source/report, and closeout path |
| Forbidden scope | product code, crates, placeholder folders, runtime implementation, shared extraction |
| Required contracts | UI Program Contract Design implementation plan |
| Evidence gates | completed design closeout before completion |
| Validation commands | docs validate; production render/validate/check; roadmap render/validate/check; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-002-ui-program-contract-design/closeout.md` |
| Stop conditions | completed; no further PM-002 action is legal through this milestone |
| Code allowed | no |
| Crate creation allowed | no |
| Production behavior may change | no |

### PM-UI-PROGRAM-003 - Control Package Proof Design

| Field | Value |
|---|---|
| Authority level | design only |
| Predecessor dependency | `PM-UI-PROGRAM-002` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-003` |
| Write scope | active design docs, implementation-plan report, roadmap/production metadata, generated planning docs, closeout |
| Forbidden scope | product code, crates, placeholder folders, runtime implementation, shared extraction |
| Required contracts | Control Package Proof Design implementation plan |
| Evidence gates | completed design closeout before completion |
| Validation commands | docs validate; production render/validate/check; roadmap render/validate/check; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-003-control-package-proof-design/closeout.md` |
| Stop conditions | stop if package registry, ID, schema, kernel, fixture, diagnostic, or migration policy remains implicit |
| Code allowed | no |
| Crate creation allowed | no |
| Production behavior may change | no |

### PM-UI-PROGRAM-004 - Compiler / Runtime Artifact Design

| Field | Value |
|---|---|
| Authority level | design only |
| Predecessor dependency | `PM-UI-PROGRAM-003` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-004` |
| Write scope | active design docs, implementation-plan report, roadmap/production metadata, generated planning docs, closeout |
| Forbidden scope | product code, crates, placeholder folders, runtime implementation, shared extraction |
| Required contracts | Compiler / Runtime Artifact Design implementation plan |
| Evidence gates | completed design closeout before completion |
| Validation commands | docs validate; production render/validate/check; roadmap render/validate/check; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-004-compiler-runtime-artifact-design/closeout.md` |
| Stop conditions | stop if manifest/table split, artifact IDs, source maps, cache invalidation, or hot-path artifact strategy remains undecided |
| Code allowed | no |
| Crate creation allowed | no |
| Production behavior may change | no |

### PM-UI-PROGRAM-005 - Evaluator / Host Design

| Field | Value |
|---|---|
| Authority level | design only |
| Predecessor dependency | `PM-UI-PROGRAM-004` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-005` |
| Write scope | active design docs, implementation-plan report, roadmap/production metadata, generated planning docs, closeout |
| Forbidden scope | product code, crates, placeholder folders, runtime implementation, shared extraction |
| Required contracts | Evaluator / Host Design implementation plan |
| Evidence gates | completed design closeout before completion |
| Validation commands | docs validate; production render/validate/check; roadmap render/validate/check; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-005-evaluator-host-design/closeout.md` |
| Stop conditions | stop if host boundaries, event/route/command boundaries, capability checks, or headless execution remain implicit |
| Code allowed | no |
| Crate creation allowed | no |
| Production behavior may change | no |

### PM-UI-PROGRAM-006 - Retained UI Migration Design

| Field | Value |
|---|---|
| Authority level | design only |
| Predecessor dependency | `PM-UI-PROGRAM-005` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-006` |
| Write scope | active design docs, implementation-plan report, roadmap/production metadata, generated planning docs, closeout |
| Forbidden scope | product code, crates, placeholder folders, runtime implementation, shared extraction |
| Required contracts | Retained UI Migration Design implementation plan |
| Evidence gates | completed design closeout before completion |
| Validation commands | docs validate; production render/validate/check; roadmap render/validate/check; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-006-retained-ui-migration-design/closeout.md` |
| Stop conditions | stop if compatibility, dual-lowering, rollback, proof-surface replacement, or drift guard policy remains undecided |
| Code allowed | no |
| Crate creation allowed | no |
| Production behavior may change | no |

### PM-UI-PROGRAM-007 - 6A Label Structural UiFrame Text Proof

| Field | Value |
|---|---|
| Authority level | implementation slice only after active WR and production plan |
| Predecessor dependency | `PM-UI-PROGRAM-006` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-007` |
| Write scope | bounded Label, text/render boundary, structural UiFrame, artifact, fixture, diagnostics, tests, docs, closeout paths named by 6A contract |
| Forbidden scope | unrelated UI runtime, broad retained UI replacement, crates, shared extraction, renderer-owned UI truth |
| Required contracts | 6A production implementation contract |
| Evidence gates | completed 6A closeout with runtime/artifact proof before completion |
| Validation commands | 6A focused tests named by contract; docs validate; production/roadmap/planning validation |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-007-6a-label-structural-uiframe-text-proof/closeout.md` |
| Stop conditions | stop if text/render ownership, structural UiFrame evidence, or retained compatibility cannot be proven |
| Code allowed | yes, only after active 6A WR and production plan |
| Crate creation allowed | no |
| Production behavior may change | only a bounded named proof surface |

### PM-UI-PROGRAM-008 - 6B Button Route Event Host Command Proof

| Field | Value |
|---|---|
| Authority level | implementation slice only after active WR and production plan |
| Predecessor dependency | `PM-UI-PROGRAM-007` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-008` |
| Write scope | bounded Button, route/event/host-command contracts, fixture, diagnostics, tests, docs, closeout paths named by 6B contract |
| Forbidden scope | giant event enum, hidden route strings, broad host rewrite, crates, shared extraction |
| Required contracts | 6B production implementation contract |
| Evidence gates | completed 6B closeout with event/route/host-command proof before completion |
| Validation commands | 6B focused tests named by contract; docs validate; production/roadmap/planning validation |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-008-6b-button-route-event-host-command-proof/closeout.md` |
| Stop conditions | stop if route IDs, route schema versions, route capabilities, or host route maps remain implicit |
| Code allowed | yes, only after active 6B WR and production plan |
| Crate creation allowed | no |
| Production behavior may change | only a bounded named proof surface |

### PM-UI-PROGRAM-009 - 6C InspectorField Binding State Proof

| Field | Value |
|---|---|
| Authority level | implementation slice only after active WR and production plan |
| Predecessor dependency | `PM-UI-PROGRAM-008` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-009` |
| Write scope | bounded InspectorField, binding/state contracts, fixture, diagnostics, tests, docs, closeout paths named by 6C contract |
| Forbidden scope | direct editor/provider mutation, broad binding rewrite, crates, shared extraction |
| Required contracts | 6C production implementation contract |
| Evidence gates | completed 6C closeout with binding/state proof before completion |
| Validation commands | 6C focused tests named by contract; docs validate; production/roadmap/planning validation |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-009-6c-inspectorfield-binding-state-proof/closeout.md` |
| Stop conditions | stop if read/write model, UiSchemaValue, dirty propagation, preview/committed state, or authorization policy remains implicit |
| Code allowed | yes, only after active 6C WR and production plan |
| Crate creation allowed | no |
| Production behavior may change | only a bounded named proof surface |

### PM-UI-PROGRAM-010 - 6D ColorPicker ControlPackage Proof

| Field | Value |
|---|---|
| Authority level | implementation slice only after active WR and production plan |
| Predecessor dependency | `PM-UI-PROGRAM-009` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-010` |
| Write scope | bounded wheel-plus-triangle ColorPicker package proof, schemas, kernels, fixture, diagnostics, tests, docs, closeout paths named by 6D contract |
| Forbidden scope | RGB cube projection, broad package framework, crates, shared extraction |
| Required contracts | 6D production implementation contract |
| Evidence gates | completed 6D closeout with rich ControlPackage proof before completion |
| Validation commands | 6D focused tests named by contract; docs validate; production/roadmap/planning validation |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-010-6d-colorpicker-controlpackage-proof/closeout.md` |
| Stop conditions | stop if package/control/schema/kernel/fixture/diagnostic IDs or migration story remain implicit |
| Code allowed | yes, only after active 6D WR and production plan |
| Crate creation allowed | no |
| Production behavior may change | only a bounded named proof surface |

### PM-UI-PROGRAM-011 - 6E World Space Host Boundary Proof

| Field | Value |
|---|---|
| Authority level | implementation slice only after active WR and production plan |
| Predecessor dependency | `PM-UI-PROGRAM-010` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-011` |
| Write scope | bounded world-space prompt, host boundary fixture, diagnostics, tests, docs, closeout paths named by 6E contract |
| Forbidden scope | ECS-owned UI semantics, broad world UI framework, crates, shared extraction |
| Required contracts | 6E production implementation contract |
| Evidence gates | completed 6E closeout with world-space host boundary proof before completion |
| Validation commands | 6E focused tests named by contract; docs validate; production/roadmap/planning validation |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-011-6e-world-space-host-boundary-proof/closeout.md` |
| Stop conditions | stop if anchor/projection/lifetime/visibility/data-feed ownership remains ambiguous |
| Code allowed | yes, only after active 6E WR and production plan |
| Crate creation allowed | no |
| Production behavior may change | only a bounded named proof surface |

### PM-UI-PROGRAM-012 - 6F Headless Fixture Diagnostics Source Map Proof

| Field | Value |
|---|---|
| Authority level | hardening slice only after active WR and production plan |
| Predecessor dependency | `PM-UI-PROGRAM-011` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-012` |
| Write scope | bounded headless fixture pack, diagnostics, source maps, artifact reproducibility, migration evidence, tests, docs, closeout paths named by 6F contract |
| Forbidden scope | implementing missing 6A-6E behavior, broad fixture framework, crates, shared extraction |
| Required contracts | 6F production implementation contract |
| Evidence gates | completed 6F closeout with aggregate accepted 6A-6E proof before completion |
| Validation commands | 6F focused tests named by contract; docs validate; production/roadmap/planning validation |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-012-6f-headless-fixture-diagnostics-source-map-proof/closeout.md` |
| Stop conditions | stop if prior slice behavior is missing; missing behavior returns to the owning milestone |
| Code allowed | yes, only after active 6F WR and production plan |
| Crate creation allowed | no |
| Production behavior may change | only bounded headless/evidence behavior |

### PM-UI-PROGRAM-013 - Runtime Proven Closeout And MaterialProgram Handoff

| Field | Value |
|---|---|
| Authority level | closeout only |
| Predecessor dependency | `PM-UI-PROGRAM-012` |
| Future WR candidate | `WR-TBD-UI-PROGRAM-013` |
| Write scope | closeout reports, roadmap/production metadata, generated planning docs, MaterialProgram proof-path link |
| Forbidden scope | new implementation, MaterialProgram implementation, shared extraction, crates |
| Required contracts | runtime-proven closeout and MaterialProgram handoff contract |
| Evidence gates | completed Stage 6A-6F closeouts plus final runtime-proven closeout |
| Validation commands | production render/validate/check; roadmap render/validate/check; docs validate; planning validate |
| Expected closeout path | `docs-site/src/content/docs/reports/closeouts/pm-ui-program-013-runtime-proven-closeout-and-materialprogram-handoff/closeout.md` |
| Stop conditions | stop if any Stage 6 evidence is missing, if MaterialProgram implementation would start, or if extraction is implied |
| Code allowed | no, unless a separate accepted gap-fix WR is created |
| Crate creation allowed | no |
| Production behavior may change | no |

## Command Results

Repository tooling now implements the manifest commands for this track:

- `production:plan-track`: this manifest.
- `production:expand-track`: prints remaining future WR candidates without
  mutating roadmap rows.
- `production:run-track -- --allow auto_safe`: created and linked `WR-136` for
  `PM-UI-PROGRAM-002`, then stopped before design authoring.
- `production:run-track -- --allow auto_safe --allow agent_design --deny
  product_code`: created the `WR-136` design/planning plan, updated the bounded
  Stage 1 contract in the UI Program Architecture design, updated planning
  metadata, and stopped before closeout.
- `production:run-track -- --allow auto_safe --allow agent_design --allow
  agent_closeout --deny product_code`: closed `PM-UI-PROGRAM-002` / `WR-136`
  as `bounded_contract`, archived the WR, and stopped before Stage 2 authoring.
- `production:next`: reports `PM-UI-PROGRAM-003` / Stage 2 as the current legal
  milestone and points to Track Expansion for the future WR candidate.
- `production:audit-track`: sees the full staged path and completed PM-002
  closeout evidence; it still blocks before Stage 2 authoring until Track
  Expansion creates or links the PM-003 WR.

## `/goal` Use

`/goal --track PT-UI-PROGRAM` may use this manifest to avoid inference. It must
execute exactly one legal milestone action at a time:

```text
PM-UI-PROGRAM-002 is closed as bounded-contract design evidence. The next legal action is PM-UI-PROGRAM-003 Track Expansion; stop before Stage 2 design authoring.
```

It must not implement Stage 6 or create code until Track Expansion, a dedicated
owning WR, and a production plan exist for the selected milestone.
