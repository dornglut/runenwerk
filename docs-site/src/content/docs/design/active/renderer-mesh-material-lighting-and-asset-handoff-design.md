---
title: Renderer Mesh Material Lighting And Asset Handoff Platform
description: Active design for mesh/model handoff, material lowering, lighting inputs, shader specialization, pipeline cache policy, and asset cooking hooks.
status: active
owner: engine
layer: engine-runtime / renderer / asset-handoff
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ./material-lab-and-material-preview-design.md
  - ../accepted/render-product-graph-platform-design.md
  - ../accepted/render-fragment-data-driven-maturity-design.md
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
material assignment truth, animation semantics, or authored model meaning.

## Evidence

Production evidence must prove visible mesh/material pixels, material preview
continuity, shader failure diagnostics, last-good fallback behavior, pipeline
cache inspection, and compatibility with SDF and product-surface rendering.
