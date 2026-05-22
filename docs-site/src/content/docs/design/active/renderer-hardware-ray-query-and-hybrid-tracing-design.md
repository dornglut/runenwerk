---
title: Renderer Hardware Ray Query And Hybrid Tracing Platform
description: Active design for optional hardware ray-query capability, derived acceleration resources, hybrid tracing paths, and non-RT fallback.
status: active
owner: engine
layer: engine-runtime / renderer / backend-capabilities
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ../accepted/render-product-graph-platform-design.md
  - ../accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ./renderer-temporal-reconstruction-and-dynamic-resolution-design.md
---

# Renderer Hardware Ray Query And Hybrid Tracing Platform

## Decision

Hardware ray-query and raytracing support is optional, capability-gated renderer
execution work. It must not become the baseline renderer path and must not move
scene, material, asset, or SDF truth into backend acceleration structures.

## Scope

This track covers:

- ray-query capability detection and unsupported diagnostics;
- derived BLAS/TLAS-style resource ownership where the backend supports it;
- hybrid raster, SDF raymarch, and ray-query render paths;
- timing, debug, denoising, and reconstruction hooks where portable;
- mandatory non-RT fallback evidence.

It does not require hardware raytracing for production rendering, and it does
not bypass prepared product, residency, material, or render-flow validation.

## Evidence

Production evidence must prove unsupported-state behavior, optional hardware
path timing, derived acceleration-resource inspection, visual parity or
documented differences against fallback paths, and no dependency on RT hardware
for core renderer correctness.
