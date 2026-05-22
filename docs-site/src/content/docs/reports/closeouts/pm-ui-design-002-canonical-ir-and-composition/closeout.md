---
title: PM-UI-DESIGN-002 Canonical UI IR And Composition Closeout
description: Closeout evidence for the UI Designer Canonical UI IR and deterministic composition design milestone.
status: completed
owner: editor
layer: domain/ui-definition
canonical: true
last_reviewed: 2026-05-22
related:
  - ../../../workspace/production-tracks.yaml
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
  - ../../../domain/ui/architecture.md
  - ../../../domain/editor/editor-definition/current-architecture.md
---

# PM-UI-DESIGN-002 Canonical UI IR And Composition Closeout

## Result

PM-UI-DESIGN-002 is complete as a bounded design milestone.

The accepted design defines the Canonical UI IR, schema/version gate, migration
dry-run, deterministic composition order, inspectable conflict diagnostics, and
round-trip authoring contract for the UI Designer platform.

This closeout does not implement code, create schemas, add runtime behavior,
add UI surfaces, or promote WR execution state.

## Evidence

- `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`
  is the accepted PM-002 design contract.
- The accepted design names `domain/ui/ui_definition` as the current generic
  owner for authored UI definition, Canonical UI IR, migration, composition,
  diagnostics, and round-trip definition evidence.
- The accepted design names `domain/editor/editor_definition` as the owner of
  editor/workbench-specific authored extensions.
- The accepted design requires deterministic composition provenance with
  winning source, losing source, affected target profile, owning domain, and
  activation impact.
- The accepted design keeps app/runtime/provider layers as consumers of
  projection output instead of source truth.

## Architecture Governance

Architecture governance for this slice accepts the existing dependency
direction:

```text
foundation -> domain/ui and domain/editor contracts -> engine/runtime -> apps
```

No new ADR is required for PM-002 because no durable dependency direction,
runtime ownership, or crate boundary changes in this design-only slice.

A future ADR or accepted design update is required before extracting a
standalone `domain/ui_definition` crate, creating a game-runtime UI owner crate,
or changing the dependency direction between UI, editor, engine, and app layers.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- PM-UI-DESIGN-003 and later milestones remain separately governed and must not
  be inferred complete from this design closeout.
- PM-UI-DESIGN-004 through PM-UI-DESIGN-010 remain incomplete and must not be
  inferred complete from this design closeout.
- No Canonical UI IR implementation code changed in this slice.
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

Close PM-UI-DESIGN-002 as completed design evidence with the accepted Canonical
UI IR and composition design, this closeout, and production-track metadata
update. Continue the production track only through the next legal action
reported by `task ai:goal -- --track PT-UI-DESIGN`.
