---
title: UI Program Proof Slice Plan
description: Bounded Stage 6 proof-slice planning contract for validating UI Program Architecture without authorizing broad implementation.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-31
related:
  - ./ui-program-architecture.md
  - ./runenwerk-domain-workbench-north-star.md
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Program Proof Slice Plan

## 1. Status And Authority

This document is the bounded proof-slice planning contract for Stage 6 of
[UI Program Architecture](./ui-program-architecture.md), under the accepted
[Runenwerk Domain Workbench North Star](./runenwerk-domain-workbench-north-star.md).

It treats UI Program Architecture as accepted design direction. It does not
authorize broad implementation, new crates, crate renames, placeholder future
folders, shared `foundation/meta` extraction, or UI runtime implementation by
itself.

Each 6A-6F slice still needs its own accepted implementation task/WR before any
code changes. A slice WR must name its exact write scope, validation commands,
acceptance evidence, and rollback/compatibility expectations.

The Stage 6 goal is evidence, not architectural expansion:

```text
6A Label text proof
-> 6B Button route/event proof
-> 6C InspectorField binding/state proof
-> 6D ColorPicker package proof
-> 6E world-space host boundary proof
-> 6F headless fixture/source-map proof
```

The full Proof Gate is not satisfied until all six sub-proofs are complete.

## 2. Non-Goals

This plan does not authorize:

- implementation;
- new workspace crates;
- crate renames;
- placeholder future folders;
- shared `foundation/meta`;
- ECS-owned UI semantics;
- renderer-owned product truth;
- generic node soup;
- a giant `UiSemanticEvent` enum;
- generic graph interpretation in hot paths by default;
- replacing retained/current UI broadly;
- extracting shared schema, ID, graph, fixture, migration, or artifact
  primitives before the Second-Domain Extraction Gate.

Retained/current UI remains compatible until a specific proof slice
intentionally replaces a bounded surface through an accepted WR.

## 3. Proof-Slice Sequence

The proof order is intentional. Earlier slices establish small contracts that
later slices reuse.

| Slice | Proof | Purpose |
| --- | --- | --- |
| 6A | `Label` plus structural `UiFrame` text proof | Prove text/render boundary and structural visual output without renderer-owned UI truth. |
| 6B | `Button` plus route/event/host-command proof | Prove semantic UI event packets, stable routes, host route maps, and host commands without a central event enum. |
| 6C | `InspectorField` plus binding/state proof | Prove read/write binding, `UiSchemaValue`, preview state, committed state, dirty propagation, and authorization. |
| 6D | `ColorPicker` plus rich `ControlPackage` proof | Prove wheel-plus-triangle rich custom control packaging, kernels, schemas, fixtures, diagnostics, and migration. |
| 6E | World-space prompt plus ECS/host boundary proof | Prove anchored prompt integration where ECS may host identity, anchor, lifetime, visibility input, and data feed only. |
| 6F | Headless fixture plus diagnostics/source-map proof | Prove deterministic fixture/debug serialization, artifact evidence, diagnostics, source maps, migration evidence, and reproducibility. |

## 4. Exact Intended Write Scopes Per Slice

These are intended implementation write scopes for later slice WRs. They are
not active write authorization from this plan. A later WR may narrow a scope. A
later WR must not broaden a scope without review.

### 6A - Label Text Proof

Intended write scope after a 6A WR:

- `domain/ui/ui_widgets/src/label.rs`
- `domain/ui/ui_text/src/style.rs`
- `domain/ui/ui_text/src/layout.rs`
- `domain/ui/ui_text/src/font.rs`
- `domain/ui/ui_render_data/src/frame/ui_frame.rs`
- `domain/ui/ui_render_data/src/primitives/glyph_run.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`
- package-local tests or examples in the same existing crates only

### 6B - Button Route/Event Proof

Intended write scope after a 6B WR:

- `domain/ui/ui_widgets/src/button.rs`
- `domain/ui/ui_input/src/event.rs`
- `domain/ui/ui_input/src/routing.rs`
- `domain/ui/ui_surface/src/capability.rs`
- `domain/ui/ui_runtime/src/input/dispatch.rs`
- `domain/ui/ui_runtime/src/input/interaction_result.rs`
- `domain/ui/ui_runtime/src/input/outcome.rs`
- package-local tests or examples in the same existing crates only

### 6C - InspectorField Binding/State Proof

Intended write scope after a 6C WR:

- `domain/editor/editor_inspector/src/`
- `domain/ui/ui_definition/src/value.rs`
- `domain/ui/ui_definition/src/view_binding/mod.rs`
- `domain/ui/ui_runtime/src/state/runtime_state.rs`
- `domain/ui/ui_runtime/src/state/mod.rs`
- `domain/ui/ui_surface/src/diagnostics/`
- package-local tests or examples in the same existing crates only

### 6D - ColorPicker ControlPackage Proof

Intended write scope after a 6D WR:

- `domain/ui/ui_widgets/src/lib.rs`
- `domain/ui/ui_widgets/src/color_picker.rs` if the 6D WR accepts a new file
  inside the existing `ui_widgets` crate
- `domain/ui/ui_layout/src/`
- `domain/ui/ui_math/src/`
- `domain/ui/ui_render_data/src/primitives/`
- `domain/ui/ui_definition/src/value.rs`
- package-local tests or examples in the same existing crates only

No future crate folder may be created for `ColorPicker` during this proof.

### 6E - World-Space Prompt Boundary Proof

Intended write scope after a 6E WR:

- `domain/ui/ui_runtime/src/`
- `domain/ui/ui_input/src/`
- `domain/ui/ui_surface/src/`
- `domain/editor/editor_viewport/src/`
- `apps/runenwerk_runtime_preview/src/` only if the accepted WR needs an app
  host proof
- package-local tests or examples in the same existing crates only

ECS integration may pass identity, anchor, lifetime, visibility input, and data
feed only. ECS must not own UI semantics.

### 6F - Headless Fixture / Diagnostics / Source-Map Proof

Intended write scope after a 6F WR:

- `domain/ui/ui_definition/src/preview_fixture/`
- `domain/ui/ui_definition/src/source.rs`
- `domain/ui/ui_definition/src/diagnostic.rs`
- `domain/ui/ui_definition/src/migration.rs`
- `domain/ui/ui_surface/src/diagnostics/`
- `domain/ui/ui_runtime/src/`
- package-local tests or examples in the same existing crates only

No 6F work may create a shared fixture, diagnostic, schema, or source-map
foundation layer.

## 5. Contracts Touched Per Slice

| Slice | Contracts |
| --- | --- |
| 6A | `Label`, font/style intent, text layout request, structural `UiFrame`, `UiRuntimeArtifactManifest`, `UiRuntimeArtifactTables`, text/render boundary. |
| 6B | `Button`, `UiEventPacket`, `RouteId`, `RouteSchemaVersion`, `RouteCapability`, `HostRouteMapVersion`, `HostCommand`, optional `DomainCommand`. |
| 6C | `InspectorField`, read model, write model, `UiSchemaValue`, value snapshot, dirty propagation, preview state, committed state, authorization/capability checks. |
| 6D | `ColorPicker`, `ControlPackage`, package ID, control kind ID, property schema, state schema, event payload schema, layout kernel ID, interaction kernel ID, visual kernel ID, fixture IDs, diagnostic IDs, migration story. |
| 6E | world-space host contract, prompt anchor, projection, lifetime, visibility, host data feed, ECS boundary. |
| 6F | deterministic fixture/debug serialization, source maps, diagnostics, migration evidence, artifact reproducibility, headless host contract. |

## 6. Fixtures Required Per Slice

| Slice | Required Fixtures |
| --- | --- |
| 6A | `Label` fixture with font/style intent, text layout request, structural `UiFrame` output, and no renderer resource dependency. |
| 6B | `Button` fixture with route emission, route capability, host route-map entry, `HostCommand` proof, and invalid route proof. |
| 6C | `InspectorField` fixture with read snapshot, preview edit, committed write, dirty propagation, binding failure, and denied authorization. |
| 6D | `ColorPicker` fixture for wheel-plus-triangle selection, package property schema, package state schema, event payload schema, visual output, accessibility metadata, and migration. |
| 6E | world-space prompt fixture with anchor, projection, lifetime, visibility input, data feed, and ECS boundary assertion. |
| 6F | headless fixture pack that runs without renderer or app process and captures artifact manifest, table summary, diagnostics, source maps, migration evidence, and reproducibility hash. |

## 7. Diagnostics Required Per Slice

| Slice | Required Diagnostics |
| --- | --- |
| 6A | missing font/style intent, unsupported text layout request, invalid structural `UiFrame`, renderer-owned handle leakage. |
| 6B | unknown `RouteId`, unsupported `RouteSchemaVersion`, missing `RouteCapability`, missing `HostRouteMapVersion`, invalid event payload, hidden route-string behavior. |
| 6C | invalid `UiSchemaValue`, missing read binding, failed write binding, stale value snapshot, dirty propagation failure, denied capability, binding failure diagnostic. |
| 6D | duplicate package/control/kernel/capability IDs, invalid property/state/event schema, missing fixture ID, missing diagnostic ID, unsupported migration, RGB cube use in first proof. |
| 6E | ECS-owned UI meaning, missing anchor, invalid projection, expired lifetime, hidden world mutation, missing visibility input, invalid host data feed. |
| 6F | nondeterministic fixture/debug serialization, missing source map, stale migration, artifact manifest mismatch, artifact table mismatch, diagnostic ID collision, reproducibility failure. |

Diagnostics must use stable diagnostic IDs and source-map attachments when
source context exists.

## 8. Source-Map Evidence Required Per Slice

| Slice | Source-Map Evidence |
| --- | --- |
| 6A | authored `Label` source maps to text layout request and structural `UiFrame` text primitive. |
| 6B | authored `Button` source maps to route declaration, emitted `UiEventPacket`, and consumed host route-map entry. |
| 6C | authored `InspectorField` source maps to read binding, write binding, value snapshot, state transition, and binding diagnostic. |
| 6D | authored `ColorPicker` source maps to package descriptor, property/state/event schemas, kernel IDs, fixture IDs, and visual output. |
| 6E | authored prompt source maps to host anchor/projection/lifetime declarations and world-space host evidence. |
| 6F | headless report proves source-map round trip for all prior slice artifacts that participate in the fixture pack. |

## 9. Runtime Artifact Evidence Required Per Slice

| Slice | Runtime Artifact Evidence |
| --- | --- |
| 6A | `UiRuntimeArtifactManifest` records text/schema/artifact IDs; `UiRuntimeArtifactTables` include text layout request and structural frame output plan. |
| 6B | manifest records route IDs, route schema versions, capabilities, and host route-map version; tables include event dispatch or route dispatch data. |
| 6C | manifest records binding schemas, state schema IDs, capabilities, and diagnostics; tables include binding snapshot layout, dirty dependency plan, and state layout. |
| 6D | manifest records package/control/schema/kernel/fixture/diagnostic IDs; tables include layout, interaction, visual, and state data for `ColorPicker`. |
| 6E | manifest records world-space host profile and capabilities; tables include host-facing anchor/projection/lifetime input layout without ECS-owned semantics. |
| 6F | manifest and table summaries are serialized deterministically for headless fixture evidence and reproducibility checks. |

Hot paths must use runtime artifacts. Generic graph interpretation is allowed
only for debugging, fixture proof, migration tools, compiler validation, or
intentionally low-frequency authoring work.

## 10. Validation Commands Per Slice

Each slice WR may add stricter commands. These are the minimum expected command
families for implementation evidence.

| Slice | Minimum Validation |
| --- | --- |
| 6A | `cargo test -p ui_text`; `cargo test -p ui_render_data`; `cargo test -p ui_widgets`; `cargo test -p ui_runtime`; `task docs:validate` |
| 6B | `cargo test -p ui_input`; `cargo test -p ui_surface`; `cargo test -p ui_widgets`; `cargo test -p ui_runtime`; `task docs:validate` |
| 6C | `cargo test -p ui_definition`; `cargo test -p ui_surface`; `cargo test -p ui_runtime`; `cargo test -p editor_inspector`; `task docs:validate` |
| 6D | `cargo test -p ui_widgets`; `cargo test -p ui_layout`; `cargo test -p ui_math`; `cargo test -p ui_render_data`; `cargo test -p ui_runtime`; `task docs:validate` |
| 6E | `cargo test -p ui_runtime`; `cargo test -p ui_input`; `cargo test -p ui_surface`; `cargo test -p editor_viewport`; `cargo test -p runenwerk_runtime_preview` if the accepted WR touches the runtime preview app; `task docs:validate` |
| 6F | `cargo test -p ui_definition`; `cargo test -p ui_surface`; `cargo test -p ui_runtime`; `task docs:validate` |

## 11. Rollback / Compatibility Expectations

- Each proof slice must preserve retained/current UI compatibility unless its
  accepted WR explicitly names a bounded replacement surface.
- Slice implementations must be reversible by disabling the bounded proof path
  without deleting retained UI behavior.
- New proof data must not become the only production path until the slice
  acceptance criteria and validation evidence pass.
- Host-facing behavior must remain explicit and inspectable.
- Renderer resources, ECS components, app commands, and editor providers must
  not become hidden UI truth.
- Migration evidence must accompany any schema, route, package, fixture, or
  artifact version change.

## 12. Acceptance Criteria Per Slice

### 6A Acceptance

- A `Label` proof produces a structural `UiFrame` text output.
- Font/style intent and text layout request are inspectable.
- Text backend and renderer responsibilities remain separate.
- No renderer handle is treated as UI truth.

### 6B Acceptance

- A `Button` proof emits `UiEventPacket`.
- `RouteId`, `RouteSchemaVersion`, `RouteCapability`, and
  `HostRouteMapVersion` are visible in proof evidence.
- The host maps the UI event to `HostCommand` and optional `DomainCommand`.
- No giant `UiSemanticEvent` enum or hidden route-string behavior exists.

### 6C Acceptance

- An `InspectorField` proof demonstrates read model, write model,
  `UiSchemaValue`, value snapshot, dirty propagation, preview state, and
  committed state.
- Binding failure and authorization/capability denial produce diagnostics.
- UI does not reach into private editor/provider state.

### 6D Acceptance

- A wheel-plus-triangle `ColorPicker` proof demonstrates a rich
  `ControlPackage`.
- RGB cube projection is absent from the first proof except as deferred
  migration/design context.
- Package/control/schema/kernel/capability/fixture/diagnostic IDs are stable
  and collision-checked.
- Property, state, event payload, layout, interaction, visual, fixture,
  diagnostic, and migration evidence is present.

### 6E Acceptance

- A world-space prompt proof demonstrates anchor, projection, lifetime,
  visibility input, and host-owned data feed.
- ECS hosts only identity, anchor, lifetime, visibility input, and data feed.
- ECS does not own UI semantics.

### 6F Acceptance

- Headless fixture evaluation runs without renderer or app process.
- Fixture/debug serialization is deterministic.
- `UiRuntimeArtifactManifest`, `UiRuntimeArtifactTables`, source maps,
  diagnostics, migration evidence, and artifact reproducibility evidence are
  present.

## 13. Risks And Open Decisions

- The current UI crates are retained-runtime oriented. Each slice must avoid
  turning compatibility infrastructure into the future semantic core.
- Exact internal module/file names for new proof code must be accepted in each
  slice WR before code changes. This plan names intended scopes, not file
  creation instructions.
- `UiSchemaValue`, stable IDs, route contracts, artifact manifests, and
  artifact tables have architecture-level contracts, but concrete Rust APIs
  still need slice-specific implementation contracts.
- 6D may pressure the design toward too much package infrastructure too early.
  The `ColorPicker` proof must stay domain-owned and bounded.
- 6E may pressure ECS integration to own semantics. The proof must keep ECS at
  identity, anchor, lifetime, visibility input, and data feed only.
- 6F may pressure fixture, diagnostic, and source-map code toward shared
  foundation extraction. That remains forbidden.

## 14. Forbidden Until The Second-Domain Extraction Gate

The following remain forbidden until UI and `MaterialProgram` both prove the
same domain-agnostic primitive and a separate extraction design accepts it:

- shared `foundation/meta`;
- shared schema values outside UI ownership;
- shared graph primitives;
- shared stable ID registries;
- shared route/event systems;
- shared compiler/evaluator framework;
- shared runtime artifact framework;
- shared fixture framework;
- shared diagnostics/source-map framework;
- shared migration framework.

`RenderPlan` follows after `MaterialProgram`; it is not a substitute for the
second platform proof.

## 15. Implementation Task / WR Requirement

This document completes Stage 6 proof-slice planning only. It does not start
implementation.

Before code changes for 6A, create and accept a dedicated 6A implementation
task/WR that includes:

- final exact write scope;
- intended behavior;
- contracts touched;
- fixture evidence;
- diagnostics evidence;
- source-map evidence;
- runtime artifact evidence;
- validation commands;
- compatibility and rollback plan;
- closeout evidence requirements.

The same requirement applies to 6B, 6C, 6D, 6E, and 6F.
