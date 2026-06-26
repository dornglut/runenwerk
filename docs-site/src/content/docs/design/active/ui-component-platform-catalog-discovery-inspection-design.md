---
title: UI Component Platform Catalog Discovery And Inspection Design
description: Phase 4 design for reusable ControlPackage catalog, discovery, and inspection contracts that downstream tools consume without owning control semantics.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-26
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./ui-component-platform-control-kernel-design.md
  - ./ui-component-platform-authoring-kit-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# UI Component Platform Catalog Discovery And Inspection Design

## Status

This is the Phase 4 planning and acceptance design for `PT-UI-COMPONENT-PLATFORM-004`.

It follows Phase 1 `ControlPackage` / `ControlKernel` contracts, Phase 2 authoring ergonomics, and Phase 3 story-proof envelopes. It defines reusable catalog, discovery, and inspection contracts for control packages. It does not authorize catalog UI, Gallery previews, Designer UX, Workbench behavior, runtime widget behavior, runtime mount eligibility, text editing, canvas surfaces, transitions, renderer-owned semantics, ECS-owned semantics, or app/editor/game mutation.

## Existing Authority

`ui_controls` owns reusable control semantics, package descriptors, validation, authoring helpers, and story-proof requirements.

`ui_artifacts` owns read-only exported package snapshots for downstream inspection.

Gallery, Workbench, UI Designer, docs, and agents consume catalog/discovery/inspection facts. They do not own reusable control semantics and must not hardcode durable package behavior.

## Problem

After Phase 3, reusable controls can be described, authored, validated, and associated with story-proof requirements. Downstream consumers still need a shared way to discover and inspect packages without rebuilding their own partial catalog logic.

Without a component catalog contract, later consumers could duplicate authority across Gallery tiles, Designer pickers, docs pages, Workbench inspectors, and agent prompts. That would make package maturity, compatibility, diagnostics, stories, routes, fixtures, target profiles, and evidence status inconsistent across tools.

## Decision

Add a reusable catalog/discovery/inspection layer for the UI Component Platform.

The catalog layer owns derived, read-only views over package descriptors and proof summaries. It must not own runtime behavior, product-specific presentation, or host mutation.

Correct ownership split:

```text
ui_controls
  owns catalog entry meaning, filters, inspection DTOs, validation, and package-derived facts.

ui_artifacts
  owns exported read-only package/catalog snapshots.

Gallery / Workbench / UI Designer / docs / agents
  consume catalog entries and inspection facts; they do not define reusable control semantics.
```

## Proposed Contract Shape

The first implementation should prefer one focused module:

```text
domain/ui/ui_controls/src/catalog.rs
```

Split later only when stable responsibilities require it.

Candidate public concepts:

```text
ControlCatalogIndex
ControlCatalogEntryDescriptor
ControlCatalogFilter
ControlCatalogQuery
ControlCatalogQueryResult
ControlInspectionDescriptor
ControlInspectionSection
ControlInspectionFact
ControlDiagnosticBadge
ControlCompatibilitySummary
ControlStoryProofBadge
```

The final names may differ after inspecting nearby conventions, but the responsibilities should remain:

- derive catalog entries from `ControlPackageDescriptor` and `ControlKindDescriptor`;
- expose searchable IDs, display names, descriptions, categories, tags, target profiles, capabilities, routes, fixtures, stories, diagnostics, compatibility flags, proof status, and mount status;
- support deterministic filtering by package, control kind, category, tag, target profile, capability, story/proof status, expected failure, diagnostic presence, and compatibility;
- produce inspection DTOs that explain why a control is descriptor-only, proof-satisfied, proof-unsatisfied, deprecated, or not mount eligible;
- preserve source IDs so consumers can copy package/story/route/fixture/diagnostic IDs;
- remain read-only and rebuildable from package/proof inputs;
- avoid product-specific UI layout, preview execution, or mutable selection state.

## Minimum Phase 4 Scope

The first implementation pass should prove the contract with a small derived index:

```text
ControlCatalogIndex::from_packages(...)
ControlCatalogQuery::matches(...)
ControlInspectionDescriptor::from_control_kind(...)
```

Minimum filters:

```text
package_id
control_kind_id
category
tag
target_profile
capability
story_required
mount_eligible
has_diagnostics
```

Minimum inspection facts:

```text
identity
metadata
compatibility
schemas
kernels
routes
fixtures
stories
story proof summary refs
diagnostics
mount eligibility explanation
```

## Non-Goals

Do not implement:

- Gallery catalog UI;
- Gallery preview execution;
- Designer picker UX;
- Workbench inspector UX;
- docs rendering widgets;
- mutable selection state;
- runtime widget behavior;
- runtime mount eligibility;
- story runner behavior;
- CLI story execution;
- app/editor/game mutation;
- Surface2D;
- SpatialCanvas;
- NodeCanvas;
- PortGraphCanvas;
- ProgressionTreeView;
- TrackSurface;
- Timeline;
- CurveEditor;
- transitions;
- text editing;
- renderer-owned UI semantics;
- ECS-owned UI semantics.

## Boundary Rules

- Catalog entries are derived projections, not new authority.
- Catalog inspection must not mutate packages, registries, stories, or hosts.
- Catalog filters must operate on explicit descriptor/proof fields only.
- Catalog inspection must explain mount ineligibility without changing it.
- Gallery, Workbench, UI Designer, docs, and agents remain consumers.
- Runtime mount eligibility remains future-gated.
- Do not introduce hidden global package registries.
- Do not duplicate package validation logic; call or consume existing validation reports.

## Acceptance Criteria

Phase 4 is implementation-complete only when:

- a reusable catalog/discovery/inspection contract exists in `ui_controls`;
- catalog entries can be derived deterministically from package descriptors;
- filters work for identity, category, tag, target profile, capability, story requirement, mount eligibility, and diagnostics;
- inspection DTOs expose package/control IDs, schemas, kernels, routes, fixtures, stories, diagnostics, compatibility, proof refs, and mount explanation;
- invalid or missing catalog metadata fails closed where validation owns that invariant;
- exported artifacts can include read-only catalog data if needed;
- focused tests prove deterministic output, filter behavior, inspection facts, and no runtime-mount upgrade;
- no Gallery, Designer, Workbench, runtime widget, story runner, text, canvas, transition, renderer, or ECS behavior is implemented.

## Candidate Implementation Scope

The first implementation pass may touch:

```text
domain/ui/ui_controls/src/catalog.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/package/metadata.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/tests/control_catalog_contract.rs
domain/ui/ui_artifacts/src/control_packages.rs
domain/ui/ui_artifacts/tests/control_package_artifact.rs
```

Use `ui_artifacts` only if read-only catalog snapshots need export support. Do not add app/editor/gallery code in Phase 4.

## Test Plan

Required focused tests for the future implementation pass:

```text
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
```

Required static checks:

```text
cargo fmt --all --check
cargo check -p ui_controls
git diff --check
```

Recommended test cases:

- catalog index derives deterministic entries from `runenwerk_control_package()`;
- category/tag/target-profile/capability filters return expected controls;
- story-required and diagnostic filters return expected controls;
- mount eligibility inspection remains not eligible for descriptor-only controls;
- inspection descriptor exposes route, fixture, story, schema, kernel, diagnostic, compatibility, and proof fields;
- artifact snapshot preserves catalog facts read-only if exported.

## Phase 4 Implementation Gate

Before writing Rust code, confirm:

- this design is accepted;
- Phase 3 remains green on the branch base;
- existing `ControlCatalogMetadata`, `ControlCompatibilityFlags`, validation reports, story proof summaries, and artifact snapshot contracts are still current;
- planning records name Phase 4 implementation as active;
- no new stop condition has been triggered.

## Handoff

Start implementation only after this design and planning state are accepted. The first implementation pass should add the smallest catalog/discovery/inspection contract and focused tests. Do not implement Gallery UI, Designer UX, Workbench behavior, story execution, runtime mount eligibility, text, canvas, transition, renderer, or ECS behavior in Phase 4.
