---
title: SDF World Rendering And Raymarch Acceleration Platform
description: Active design for sparse SDF bricks, page tables, clipmaps, distance mips, candidate lists, and conservative raymarch acceleration.
status: active
owner: engine
layer: engine-runtime / renderer / sdf-products
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ../accepted/sdf-first-field-world-platform-design.md
  - ../accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ./renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ./sdf-procedural-animation-and-animated-models-design.md
---

# SDF World Rendering And Raymarch Acceleration Platform

## Decision

Sparse SDF world rendering is a product-driven renderer consumer, not renderer
source truth. Domain SDF and world products own authored operations, formed
chunk/page/brick payloads, lineage, freshness, query policy, and strict
consumer semantics. The renderer owns derived GPU residency, raymarch
execution, acceleration data, timing, and diagnostics.

## Scope

This track covers:

- SDF brick atlases, page tables, sparse pages, and clipmap windows;
- analytic SDF instances, sampled/proxy fields, cluster fields, and aggregate
  fields as explicit renderer representations;
- distance mips, conservative empty-space skipping, step caps, and missed
  surface diagnostics;
- screen-tile plus depth-slice candidate lists;
- temporal raymarch data and cache invalidation keyed by product generation;
- SDF world runtime examples and evidence.

It does not own SDF authoring, SDF physics truth, collision conservatism,
gameplay interaction fields, or product fallback policy.

## Raymarch Policy

Fullscreen raymarching is allowed only as one bounded view pass over prepared
resident products. It must not be multiplied by per-entity instance counts.

Raymarch acceleration must be conservative: empty-space hierarchies, distance
mips, and macro steps must never overestimate safe travel distance. Diagnostics
must expose unsafe overstep risk, candidate-list explosion, cache pressure,
missed-surface risk, step count, and fallback state.

## Evidence

Production evidence must include GPU timing, ray step statistics, page and
brick residency state, clipmap coverage, candidate-list sizes, cache rebuild
pressure, memory pressure, and visual runtime proof for near, mid, far, and
summary scale bands.
