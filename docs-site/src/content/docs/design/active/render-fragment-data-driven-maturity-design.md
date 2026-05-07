---
title: Render Fragment Data-Driven Maturity Design
description: Design for authored render fragments, fragment validation, hot reload, merge semantics, ID maturity, inspection, and example proofs.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
related_designs:
  - ./render-product-surface-foundation-bundle-design.md
  - ./editor-asset-pipeline-and-content-workflow-design.md
  - ./editor-procedural-content-and-simulation-workflow-plan.md
  - ./engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
related_roadmaps:
  - ../../engine/plugins/render/docs/roadmap.md
related:
  - ../../engine/reference/plugins/render/architecture.md
  - ../../engine/reference/plugins/render/render-flow-usage-guide.md
---

# Render Fragment Data-Driven Maturity Design

## Status

Active design for the full R10 render roadmap scope.

This design promotes render fragments from a roadmap placeholder into a concrete architecture for authored, validated, mergeable, inspectable, and hot-reloadable render-flow pieces.

## Goal

Make render-flow composition data-driven without weakening the runtime architecture:

```text
authored fragment asset
  -> schema/migration
  -> fragment ratification
  -> fragment registry
  -> merge into RenderFlow definition
  -> compile/validate execution plan
  -> prepared frame execution
  -> inspection and diagnostics
```

The renderer remains engine-runtime-owned. Fragments are description-level inputs that can generate or contribute to `RenderFlow` definitions and feature contributions.

## Non-Negotiable Outcomes

- Render fragments are authored descriptions, not runtime backend objects.
- Fragment validation happens before a fragment can affect active render flow execution.
- Fragment IDs, resource aliases, pass labels, and target aliases are namespace-safe.
- Fragment merge conflicts produce actionable diagnostics.
- Hot reload has revision tracking, failure recovery, and last-good behavior.
- Fragment composition does not bypass `RenderFlow` validation or compiled execution planning.
- Editor/asset tooling can inspect fragment provenance and diagnostics.
- Material, SDF, debug, and product-surface fragments can be supported without making render aware of editor-specific products.

## Current Constraints

Current render composition is mostly direct code registration:

- `engine/src/plugins/render/composition/integration.rs::RenderFlowRegistryResource` stores `RenderFlow` values and compiles them.
- `engine/src/plugins/render/composition/fragments.rs` and `engine/src/plugins/render/composition/hot_reload.rs` do not yet exist.
- material feature contributions already carry `specialization_key_fragment` strings, but that is pipeline specialization metadata, not a general render-flow fragment system.
- the render roadmap lists fragment/data-driven maturation, but it has not had an owning design.

## Core Concepts

### Render Fragment

A render fragment is a backend-neutral description of render-flow pieces:

- resource declarations;
- pass declarations;
- dependencies;
- target aliases;
- shader references;
- feature ids;
- parameter schema references;
- required capabilities;
- contribution hooks.

Fragments are not allowed to allocate backend resources directly.

### Fragment Package

A fragment package groups fragments with metadata:

- package id;
- version;
- source path;
- declared namespace;
- schema version;
- dependency fragments;
- compatibility constraints.

### Fragment Namespace

Every fragment has a namespace used to qualify labels and aliases during merge. Local labels may be ergonomic, but merged labels must be deterministic and collision-safe.

### Fragment Merge Plan

A merge plan is the validated description of how fragments combine into a `RenderFlow`:

- resolved resource labels;
- resolved pass labels;
- dependency edges;
- feature contribution slots;
- conflicts and overrides;
- provenance map from merged flow element back to source fragment.

## Engine Contract

### Fragment Model

Add:

```text
engine/src/plugins/render/composition/fragments.rs
```

Responsibilities:

- define `RenderFragmentId`, `RenderFragmentPackageId`, `RenderFragmentNamespace`;
- define fragment resource/pass/dependency descriptors;
- define fragment capability requirements;
- define fragment provenance metadata;
- stay backend-neutral and serializable where practical.

### Fragment Registry

Add:

```text
engine/src/plugins/render/composition/fragment_registry.rs
```

Responsibilities:

- store accepted fragments and package metadata;
- track source revision and last-good revision;
- expose active, stale, failed, and disabled states;
- produce deterministic fragment iteration order for merge.

### Fragment Validation

Add:

```text
engine/src/plugins/render/composition/fragment_validation.rs
```

Responsibilities:

- validate schema version;
- validate namespace and ids;
- validate resource/pass references;
- validate target aliases;
- validate capability requirements;
- reject collisions unless an explicit override policy exists;
- produce diagnostics with source paths and fragment ids.

### Fragment Merge

Add:

```text
engine/src/plugins/render/graph/merge.rs
```

Responsibilities:

- merge accepted fragments into `RenderFlow` definitions or flow builder patches;
- preserve provenance for every generated resource/pass/dependency;
- reject incompatible pass ordering, missing dependencies, and illegal resource role merges;
- output a merge report for inspection.

### Fragment Hot Reload

Add:

```text
engine/src/plugins/render/composition/hot_reload.rs
```

Responsibilities:

- watch fragment package sources through existing reload infrastructure;
- parse and validate changed fragments;
- promote valid revisions;
- preserve last-good active revision on failure;
- publish diagnostics and reload status.

### Inspection

Update:

```text
engine/src/plugins/render/inspect/graph_dump.rs
engine/src/plugins/render/inspect/pass_provenance.rs
engine/src/plugins/render/inspect/report.rs
```

Responsibilities:

- show fragment source for generated passes/resources;
- show merge conflicts;
- show active/stale/failed fragment revisions;
- show fragment contribution to pipeline specialization keys where relevant.

## Data Model Sketch

```rust
pub struct RenderFragmentDescriptor {
    pub id: RenderFragmentId,
    pub namespace: RenderFragmentNamespace,
    pub resources: Vec<RenderFragmentResource>,
    pub passes: Vec<RenderFragmentPass>,
    pub dependencies: Vec<RenderFragmentDependency>,
    pub required_capabilities: Vec<RenderCapabilityRequirement>,
}

pub struct RenderFragmentMergeReport {
    pub fragment_id: RenderFragmentId,
    pub generated_flow_id: RenderFlowId,
    pub provenance: Vec<RenderFragmentProvenanceRecord>,
    pub diagnostics: Vec<RenderFragmentDiagnostic>,
}
```

Exact Rust shapes can follow local render API conventions during implementation. The contract is that fragment descriptions are inspectable and validated before execution.

## Relationship To Assets And Editor Authoring

The engine render plugin can own fragment runtime description, validation, merge, and hot reload.

The asset/editor systems later own:

- fragment asset catalog entries;
- authoring UI;
- migration UI;
- dependency graph integration;
- project-level import diagnostics;
- save/load workflows.

This design should not wait for the full asset pipeline, but it must leave source paths, package metadata, and diagnostics structured enough for later asset integration.

## Implementation Phases

### RF1 - Fragment Description Types

Change:

- `engine/src/plugins/render/composition/fragments.rs`
- `engine/src/plugins/render/composition/mod.rs`
- `engine/src/plugins/render/api/ids.rs`

Exit criteria:

- fragment ids and namespaces are typed;
- fragment resource/pass/dependency descriptors exist;
- descriptors can be inspected in tests without backend runtime.

### RF2 - Fragment Validation

Change:

- `engine/src/plugins/render/composition/fragment_validation.rs`
- `engine/src/plugins/render/graph/validation.rs`

Exit criteria:

- invalid references, namespace collisions, missing resources, incompatible target aliases, and unsupported capabilities produce diagnostics.

### RF3 - Fragment Merge Into RenderFlow

Change:

- `engine/src/plugins/render/graph/merge.rs`
- `engine/src/plugins/render/composition/integration.rs`

Exit criteria:

- accepted fragments can merge into a `RenderFlow`;
- merge output goes through normal `compile_flow_plan`;
- provenance maps generated flow elements back to fragment source.

### RF4 - Fragment Registry And Hot Reload

Change:

- `engine/src/plugins/render/composition/fragment_registry.rs`
- `engine/src/plugins/render/composition/hot_reload.rs`
- `engine/src/plugins/shared/reload.rs` if shared reload hooks need a render fragment source kind.

Exit criteria:

- valid changed fragments promote to active;
- invalid changed fragments keep last-good active revision;
- diagnostics expose current and last-good revisions.

### RF5 - Inspection And Debug Reports

Change:

- `engine/src/plugins/render/inspect/graph_dump.rs`
- `engine/src/plugins/render/inspect/pass_provenance.rs`
- `engine/src/plugins/render/inspect/report.rs`

Exit criteria:

- graph dumps show fragment provenance;
- merge reports and reload diagnostics are visible through render inspection.

### RF6 - Example Proofs

Change:

- `engine/examples/render_fragment_compositor/`
- `engine/examples/sdf_render_flow/`
- docs under `docs-site/src/content/docs/engine/examples/`

Exit criteria:

- at least one fragment-driven compositor flow runs through builtin compiled execution;
- one SDF or product-surface example uses fragments for reusable pass/resource structure;
- no custom executor path is required.

## Validation

```text
cargo test -p engine --test render_flow_fragments
cargo test -p engine --test render_flow_v2
cargo test -p engine --test render_runtime_inspect
cargo test -p engine --example render_fragment_compositor
python3 tools/docs/validate_docs.py
```

## Final Acceptance Criteria

- Render fragments have typed ids, namespaces, descriptors, validation, merge, registry, and hot reload.
- Invalid fragments cannot affect active render execution.
- Valid fragments merge into normal `RenderFlow` definitions and use normal compiled planning/execution.
- Fragment provenance is visible in graph dumps and render inspection reports.
- Hot reload preserves last-good behavior and exposes actionable diagnostics.
- Example flows prove fragment-driven composition without custom executors.

## Relationship To Render Product Surfaces

The product surface foundation bundle should land first because fragments need stable target aliases, prepared flow invocations, and dynamic target descriptors to be useful for real product-surface workflows.

Fragment maturity then makes those product-surface flows reusable and authorable instead of hand-written in Rust for every feature.
