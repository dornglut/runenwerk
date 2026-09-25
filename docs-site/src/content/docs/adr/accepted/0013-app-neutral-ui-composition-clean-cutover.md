---
title: App-Neutral UI Composition Clean Cutover
description: Accepted decision for app-neutral structural composition and the clean consumer-by-consumer retirement of predecessor Runenwerk UI runtime authority during standalone RunenUI adoption.
status: accepted
owner: ui
layer: domain/app
canonical: true
last_reviewed: 2026-09-25
publication: reference
pagefind: false
supersedes:
  - ../superseded/0006-editor-surface-provider-plugin-seam.md
  - ../superseded/0012-capability-workbench-clean-break.md
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/accepted/adaptive-ui-composition-design.md
related_roadmaps:
  - ../../domain/ui/roadmap.md
---

# ADR 0013: App-Neutral UI Composition Clean Cutover

## Decision

`domain/ui/ui_composition` is the app-neutral structural composition authority
and `domain/ui/ui_adaptive_composition` is the derived adaptive-mechanism owner.
Those authorities remain Runenwerk-local and are not part of standalone RunenUI.

Standalone RunenUI is now the Dornglut reusable UI-framework authority for
transient View/Element authoring, mounted runtime identity and reconciliation,
framework-local state/lifecycle, input/focus/interaction, style/layout/text,
renderer-neutral paint and semantic publication, accessibility, deterministic
headless execution, and its public platform/renderer edges.

Runenwerk therefore adopts a **clean consumer-by-consumer cutover** rather than a
second all-at-once local UI rewrite. For each accepted consumer migration:

1. Runenwerk-owned authored/product/composition meaning is projected explicitly
   into public RunenUI contracts;
2. that consumer has exactly one mounted/runtime/interaction authority;
3. the replaced local runtime authority for that consumer is deleted in the same
   accepted cut;
4. no forwarding facade, writable mirror, compatibility namespace, or dual
   runtime is retained.

This revises the original delivery assumption that every local UI predecessor
would disappear in one repository-wide branch. The clean-cutover invariant is
retained; the unit of cutover is now one decision-complete consumer boundary.

## Retained Runenwerk authority

The following semantics remain Runenwerk-owned unless a later accepted decision
changes them:

- `ui_composition` structural application composition and persistence;
- `ui_adaptive_composition` transient structural projection/proposals;
- authored UI definition/schema semantics;
- UiProgram, lowering, compiler, artifact, source-map and diagnostic semantics;
- Story V2 orchestration and Runenwerk-local proof policy;
- editor/product commands, provider/session/content policy, self-authoring and
  workspace/product semantics;
- app/domain mutation and authorization;
- Runenwerk renderer integration and product publication policy.

RunenUI does not acquire those semantics merely because it becomes the runtime
consumer.

## Predecessor runtime authority

Runenwerk-local reusable framework-shaped runtime families are predecessor
authority once a named consumer moves to RunenUI. This includes the local
retained tree/runtime/widget path and overlapping math/input/layout/text/theme,
geometry/state/accessibility/testing and renderer-neutral runtime machinery.

These packages remain current implementation authority for consumers not yet
migrated. They must not be expanded as a competing reusable-framework roadmap.
Their deletion point is the accepted consumer cut that removes their last
maintained use.

Mixed packages such as `ui_controls`, `ui_binding`, `ui_render_data`,
`ui_evaluator`, `ui_runtime_view`, `ui_hosts`, `ui_surface`, and
`engine::plugins::ui` must be split by responsibility rather than classified
by filename.

## `ui_surface` supersession

The original decision to remove `ui_surface` is retained.

Current source still uses it, so this ADR no longer describes that removal as
already complete. Until its final consumer cut:

- it is a temporary predecessor/compatibility boundary only;
- no new generic mounted-runtime, composition, product, or world-space authority
  may be added to it;
- reusable mounted/runtime/input/accessibility responsibilities move to RunenUI
  when their consumers migrate;
- app/domain-specific capability, ratification, session, content or mutation
  meaning moves to the actual Runenwerk app/domain owner;
- renderer/product `Surface` vocabulary remains separate from this UI
  compatibility package.

Concrete world-space prompt semantics are not retained as generic
`ui_surface` authority.

The package is deleted when the exact-source census proves every maintained
responsibility has moved to its real owner or has an accepted deferred non-goal.

## `ui_hosts` supersession

The original decision to narrow `ui_hosts` to UiProgram-host responsibility is
also retained, but the current package name is acknowledged as unfinished
predecessor state.

Only UiProgram-specific lifecycle/event/output contracts may survive as a
Runenwerk-local host vocabulary. Generic mounted runtime, windowing, input,
renderer, application lifecycle, or product-mutation authority does not belong
there. If a retained UiProgram-specific package is still justified at cutover,
it should use an owner-accurate name such as `ui_program_hosts`; otherwise the
predecessor package is deleted.

## Engine and host boundary

`engine::plugins::ui` is currently a real Runenwerk mounted/session/runtime
owner. During RunenUI adoption it becomes an integration adapter, not another UI
framework.

The target direction is:

```text
Runenwerk authored/product/composition semantics
        -> explicit projection/adapter
            -> runenui_core + runenui_runtime + runenui_text
                 -> public RunenUI publication
                      -> Runenwerk Render adapter and/or accepted RunenUI renderer edge

Native Host -> runenui_winit translation where adopted
RunenUI actions -> Runenwerk app/domain mutation owner
```

Native window/event-loop policy remains with Native Host. RunenUI does not own
Runenwerk product mutation. Runenwerk Render does not become UI semantic
authority.

## State And Persistence

Use the accepted four-part composition model:

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

RunenUI mounted state is runtime state and must not replace
`ui_composition` persistence or become a second saved structural authority.

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

- Editor workspace structural authority remains superseded by
  `ui_composition`; user-facing wording may still say workspace or panel.
- `ui_surface` and generic `ui_hosts` authority are explicit predecessor
  debt, not accepted end-state framework boundaries.
- Existing local runtime packages remain valid only for not-yet-migrated
  consumers.
- Standalone RunenUI adoption is consumer-owned and does not authorize source
  copying from either repository.
- `Surface` remains reserved for render/product output vocabulary outside the
  temporary `ui_surface` compatibility package.
- No RunenApp, generic Host framework, shared registry, or UI meta-framework is
  introduced by this decision.

## First adoption proof

The first RunenUI consumer cut should be a bounded **headless execution proof**
over one maintained Runenwerk authored/UI-program path, not Editor-wide or
native-window migration.

That cut must prove:

```text
retained Runenwerk authored definition / UiProgram facts
    -> explicit target-neutral projection
        -> public RunenUI View/Element/action/semantic contracts
            -> ordinary runenui_runtime + runenui_testing execution
```

Story V2 may orchestrate and record that proof, but Story V2 remains Runenwerk
authority. The cut must identify and delete the local evaluator/runtime-view or
retained-runtime execution authority that becomes redundant **for that exact
proof consumer**. It must not add RunenUI merely as a second execution path.

No source-migration issue is authorized by this ADR alone; the concrete cut is
separately issue-owned after its projection contract and deletion set are
reviewed.

## Rejected Alternatives

A repository-wide immediate RunenUI replacement is rejected because current
Runenwerk authored/program/composition semantics have no one-to-one RunenUI
owner and would create an unnecessarily broad migration.

A gradual dual-runtime consumer is rejected because it creates two mounted,
interaction, focus, and lifecycle authorities for the same UI.

Keeping `ui_surface` as a permanent generic UI authority is rejected because
it combines responsibilities whose real owners are RunenUI, Runenwerk
app/domains, composition, or renderer/product integration.

Moving `ui_composition` into RunenUI is rejected because application
composition/persistence and UI framework visual/runtime composition are distinct
semantic authorities.

A universal app-host trait is rejected because lifecycle, policy, windowing,
content, and persistence change for different reasons.

## Fitness Functions

Any implementation issue that activates this decision must prove:

- one mounted/runtime/interaction authority per migrated consumer;
- no local/RunenUI writable mirror or forwarding namespace;
- target-neutral projection from retained Runenwerk source/program meaning;
- app/domain mutation remains outside RunenUI;
- `ui_composition` persistence and structural transactions remain authoritative;
- exact RunenUI public-contract use with no private framework reach-through;
- input, focus, text, accessibility, renderer-publication and headless behavior
  preserved for the selected consumer;
- replaced predecessor code deleted at acceptance;
- current documentation and dependency direction reconciled;
- exact-head repository validation.

Acceptance belongs to the reviewed pull request and its exact-head validation.
Final migration completion requires no unowned, unexplained, or unaccepted
predecessor authority.
