---
title: Renderer Mesh Material Lighting And Asset Handoff Platform
description: Accepted design for mesh/model handoff, material lowering, lighting inputs, shader specialization, pipeline cache policy, and asset cooking hooks.
status: accepted
owner: engine
layer: engine-runtime / renderer / asset-handoff
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ../active/material-lab-and-material-preview-design.md
  - ./render-product-graph-platform-design.md
  - ./render-fragment-data-driven-maturity-design.md
  - ./renderer-gpu-evidence-and-procedural-visuals-design.md
---

# Renderer Mesh Material Lighting And Asset Handoff Platform

## Decision

Mesh, material, lighting, shader, and asset cooking work must enter the
renderer as prepared products, render contributions, fragments, or derived GPU
resources. The renderer may compile pipelines, specialize shaders, allocate
buffers, and report diagnostics, but it must not become the source of asset,
material, model, rig, or scene truth.

## Scope

This track covers:

- mesh/model/skinning/deformation render handoff;
- material shader graph lowering into renderer-consumable artifacts;
- material preview and mesh preview product targets;
- lighting inputs, debug views, and pipeline specialization;
- pipeline cache policy and last-good shader fallback;
- render asset cooking hooks and generated artifact diagnostics.

It does not own asset catalog truth, material graph source truth, scene
material assignment truth, animation semantics, authored model meaning, product
freshness authority, or fallback legality.

## Boundary Contract

Renderer-owned state is derived execution state:

- prepared mesh records, vertex/index buffer views, skinning/deformation
  buffer views, and draw/visibility inputs derived from upstream products;
- material shader artifacts, bind group layouts, pipeline specialization keys,
  pipeline cache entries, last-good shader references, and renderer diagnostics;
- lighting buffers, clustered/tiled light inputs, debug views, and timing or
  cache inspection DTOs;
- asset-cook outputs that are explicitly renderer artifacts, not source assets.

Producer-owned state remains outside the renderer:

- material source documents and graph semantics in `domain/material_graph`;
- scene material assignments in `domain/editor/editor_scene`;
- asset catalog identity, import policy, and package trust in asset/workflow
  owners;
- model, mesh, rig, animation, deformation semantics, and authored meaning in
  their source domains;
- product lineage, freshness, authority class, rebuild policy, residency
  intent, and fallback legality in Product Graph/Product Jobs contracts.

Renderer diagnostics may name source lineage and prepared artifact identities,
but they must not become the authority for editing, validation, or fallback
policy outside renderer execution.

## Translation Points

Downstream implementation must use explicit translation boundaries:

- material graph ratification and lowering produce renderer-consumable shader
  artifacts without moving graph semantics into `engine/src/plugins/render`;
- scene/model/mesh products publish prepared render contributions instead of
  requiring live ECS extraction during render submission;
- mesh and material previews route through product surfaces and prepared
  renderer data so preview behavior shares the product-selection path used by
  SDF and procedural rendering;
- asset cooking may create renderer artifacts and diagnostics, but source asset
  identity, package trust, and catalog policy stay outside the renderer;
- shader specialization and pipeline cache decisions are renderer execution
  choices, not material truth.

## Invariants

- No renderer-owned source truth for materials, assets, models, rigs, animation,
  scene assignments, product freshness, or fallback legality.
- No raw artifact-id assignment from Material Lab into scene material state.
- No implementation path that makes renderer prepared data the only editable
  representation of a material, model, mesh, or lighting source.
- No silent shader or pipeline fallback. Last-good fallback must report source
  lineage, prior artifact identity, failure diagnostics, and the upstream policy
  that allows reuse.
- No mesh/material preview shortcut that bypasses product surfaces or diverges
  from the renderer contracts used by other product families.

## Sequence

The accepted production sequence is:

1. `WR-067` implements prepared mesh/material/shader handoff paths and visible
   preview products.
2. `WR-068` implements lighting inputs, pipeline specialization/cache
   diagnostics, and last-good shader fallback evidence.
3. `WR-069` closes mesh/material/lighting production evidence with docs,
   examples, tests, benchmarks, runtime output, and inspection reports.

Each implementation row must be applied, promoted, contracted, validated, and
closed independently. This doctrine does not authorize implementation before
the corresponding WR row is legally ready.

## Evidence

Production evidence must prove:

- visible mesh/material pixels from prepared renderer inputs;
- material and mesh previews routed through product surfaces;
- renderer diagnostics that preserve material, mesh, shader, and asset lineage;
- shader failure diagnostics and last-good preservation when policy allows it;
- pipeline cache inspection, cache miss/failure reporting, and specialization
  key visibility;
- compatibility with SDF, procedural visuals, render-flow pass-shape guards,
  GPU timing diagnostics, and product-surface selection.

Required fitness functions include focused renderer tests, example commands,
runtime inspection DTO guards, shader-failure and fallback tests, benchmark or
timing evidence where appropriate, docs validation, roadmap validation,
production validation, and planning validation.

## Governance

DDD bounded context owner: `engine/src/plugins/render` for renderer execution
artifacts, pipeline specialization, GPU resources, cache diagnostics, and
renderer inspection DTOs.

Stream-aligned producers own material, asset, model, scene, lighting-source,
and product truth. The renderer platform remains a complicated subsystem that
consumes prepared products and exposes execution diagnostics.

No ADR is required for this doctrine acceptance because it preserves existing
dependency direction and accepted product/render boundaries. An ADR is required
before a later change persists a new cross-domain ABI, moves material/asset/model
truth into the renderer, changes fallback authority, or changes ownership of
Product Graph/Product Jobs contracts.
