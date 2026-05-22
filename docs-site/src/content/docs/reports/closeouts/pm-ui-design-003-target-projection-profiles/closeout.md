---
title: PM-UI-DESIGN-003 Target Projection Profiles Closeout
description: Closeout evidence for editor/workbench and game-runtime UI target projection profile design.
status: completed
owner: editor
layer: domain/ui-definition
canonical: true
last_reviewed: 2026-05-22
related:
  - ../../../workspace/production-tracks.yaml
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
---

# PM-UI-DESIGN-003 Target Projection Profiles Closeout

## Result

PM-UI-DESIGN-003 is complete as a bounded design milestone.

The accepted design defines editor/workbench and game-runtime UI target
projection profiles over Canonical UI IR, including reproducibility inputs,
explicit ownership, and fail-closed diagnostics.

This closeout does not implement code, add schemas, create game-runtime UI
crates, add UI surfaces, or promote WR execution state.

## Evidence

- `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md`
  is the accepted PM-003 design contract.
- The accepted design keeps editor/workbench projection on
  `domain/editor/editor_definition` and `domain/editor/editor_shell` contracts.
- The accepted design keeps game-runtime UI projection independent from editor
  shell ownership.
- Projection output is explicitly derived state and not source truth.
- Projection reproducibility inputs and fail-closed diagnostics are named.

## Architecture Governance

Architecture governance accepts the profile split and records that no new ADR is
required for this design-only slice. A future ADR or accepted design update is
required before adding a game-runtime UI owner crate or making projection output
authoritative state.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- PM-UI-DESIGN-004 through PM-UI-DESIGN-010 remain incomplete and must not be
  inferred complete from this design closeout.
- No projection implementation code changed in this slice.
- This is not `runtime_proven` or `perfectionist_verified`.

## Validation

Required validation for this bounded design slice:

```text
task production:render
task production:validate
task production:check
task roadmap:render
task roadmap:validate
task roadmap:check
task docs:validate
task planning:validate
task ai:goal -- --track PT-UI-DESIGN
```

The command outputs are recorded in the working session that completed this
closeout.

## Closeout Decision

Close PM-UI-DESIGN-003 as completed design evidence with the accepted target
projection profiles design, this closeout, and production-track metadata
update. Continue the production track only through the next legal action
reported by `task ai:goal -- --track PT-UI-DESIGN`.
