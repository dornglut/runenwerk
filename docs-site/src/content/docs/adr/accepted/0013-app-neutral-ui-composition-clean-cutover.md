---
title: App-Neutral UI Composition Clean Cutover
description: Accepted decision to make domain/ui composition the structural authority, separate adaptive projection, retain app-owned content semantics, and remove workspace and ui_surface structural authority through one governed cutover.
status: accepted
owner: ui
layer: domain/app
canonical: true
last_reviewed: 2026-06-19
supersedes:
  - ../superseded/0006-editor-surface-provider-plugin-seam.md
  - ../superseded/0012-capability-workbench-clean-break.md
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/accepted/adaptive-ui-composition-design.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# ADR 0013: App-Neutral UI Composition Clean Cutover

## Decision

Create `domain/ui/ui_composition` as the app-neutral structural composition
authority and `domain/ui/ui_adaptive_composition` as the derived adaptive
mechanism owner.

Rename `ui_hosts` to `ui_program_hosts` and prove it owns only UiProgram
lifecycle, event routing, and output consumption.

Remove `ui_surface` after every responsibility maps to a replacement contract,
proof, historical status, or accepted deferred non-goal. Delete its concrete
world-space prompt contracts and defer real world-space semantics to
`PT-GAME-WORLDSPACE-UI`.

Replace editor workspace structural authority with generic composition. Editor
providers, sessions, extension state, policies, content actions, and UI wording
remain editor-owned. Draw becomes the non-editor runtime proof. Other app
classes remain executable headless conformance fixtures.

Use one clean-cutover branch and one final merge. Temporary compatibility code
may read old state on the branch, but no aliases, writable parallel authorities,
or legacy persistence survive cleanup.

## State And Persistence

Use the accepted four-part model:

- `CompositionDefinitionV1`: authored/saved layout;
- `CompositionState`: ratified mutable structure;
- `AdaptiveProjectionState`: transient derived presentation;
- `LayoutPromotion`: explicit state plus app-extension snapshot to a new saved
  definition bundle.

Persistence is deterministic canonical RON. Core and extension documents share
layout/revision/schema/compatibility metadata and BLAKE3 payload hashes. Loading,
promotion, validation, and generation-pointer activation are atomic.

Legacy V1-V5 workspace files are unsupported, left untouched, and never
automatically migrated.

## Authority Boundaries

`MountedContentRef` is opaque and typed. Provider/session/content meaning stays
app/domain-owned and keyed by `MountedUnitId`.

Composition owns structural transactions and structural undo/redo only.
Adaptive composition emits proposals only. App and windowing layers own native
windows, OS vetoes, monitor/DPI/restore behavior, and concrete policy.

Narrow lifecycle, capability, proposal-acceptance, content-resolution,
extension-snapshot, and target/window coordination ports replace any universal
`AppHost` trait.

## Consequences

- ADR 0006 provider ownership is retained in generic form, but its
  `ToolSurfaceInstanceId` and `WorkspaceState` structural assumptions are
  superseded.
- ADR 0012 clean-break principles are retained, but stable workbench surface
  identity is superseded by `MountedUnitId`, content/profile references, and
  generic composition persistence.
- User-facing editor wording may still say workspace or panel, but core APIs and
  schemas do not.
- `Surface` is reserved for render/product output vocabulary.
- Extraction from `domain/ui` requires two real non-UI bounded-context proofs
  and a later ADR; fixtures do not qualify.

## Rejected Alternatives

Compatibility aliases or a gradual main-branch dual API were rejected because
they preserve multiple authorities.

Extending `ui_surface` was rejected because it combines structure, content
semantics, observation, presentation, and world-space proof under one boundary.

A universal app-host trait was rejected because lifecycle, policy, windowing,
content, and persistence change for different reasons.

Persisting adaptive projection state was rejected because responsive state is
derived and must restore automatically.

## Fitness Functions

The production track must prove deterministic serialization, graph invariants,
atomic transactions, structural inverse validation, unavailable-content
fallbacks, linked persistence bundles, headless fixtures, dependency direction,
input/accessibility parity, native-window ownership, measured performance, full
`ui_surface` supersession, and current documentation/truth evidence.

Final completion requires no unowned, unexplained, or unaccepted risk.
