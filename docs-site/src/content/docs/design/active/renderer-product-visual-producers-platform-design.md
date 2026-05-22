---
title: Renderer Product Visual Producers Platform
description: Active design for product-owned particles, VFX, vegetation, water, atmosphere, weather, decals, trails, and animation render producer integration.
status: active
owner: engine
layer: engine-runtime / product-render-integration
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../accepted/sdf-first-field-world-platform-design.md
  - ./editor-procedural-content-and-simulation-workflow-plan.md
  - ../accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
---

# Renderer Product Visual Producers Platform

## Decision

Particles, VFX, vegetation, water, atmosphere, weather, trails, decals, and
animation/deformation visuals must be product-owned producers that feed the
renderer. The renderer supplies execution APIs, product-surface integration,
residency, timing, and diagnostics. Product domains own semantics, source
truth, authority, freshness, fallback legality, rebuild policy, and user-facing
meaning.

## Scope

This track covers renderer integration for:

- particles, VFX, smoke, sparks, trails, decals, sorting, and transparency;
- vegetation, grass, foliage, wind response, and far-field summaries;
- water, wetness, shoreline, flow fields, weather, snow, sand, erosion, heat,
  humidity, atmosphere, and day/night visual products;
- animation and deformation render producers;
- prepared render contributions, residency requests, product-surface outputs,
  and diagnostics for the above families.

It does not define product truth for any of those domains.

## Evidence

Runtime evidence must prove product-owned producers can emit prepared render
contributions and residency requests, use renderer scale/temporal/SDF
capabilities when available, and report missing, stale, fallback, over-budget,
or unsupported states without silent promotion.
