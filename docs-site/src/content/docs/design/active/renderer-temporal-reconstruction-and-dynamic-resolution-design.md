---
title: Renderer Temporal Reconstruction And Dynamic Resolution Platform
description: Active design for TAA, TAAU, dynamic resolution, motion-vector/depth/exposure products, and FSR-style optional adapters.
status: active
owner: engine
layer: engine-runtime / renderer / postprocess
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ../accepted/render-product-graph-platform-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ./sdf-world-rendering-and-raymarch-acceleration-design.md
---

# Renderer Temporal Reconstruction And Dynamic Resolution Platform

## Decision

Temporal reconstruction is a renderer platform, not a vendor-upscaler shortcut.
The first durable contract is backend-neutral history, jitter, motion vectors,
depth, exposure, reactive-mask style metadata, dynamic internal resolution, and
diagnostics. FSR-style or vendor-specific adapters come after the portable
contract exists.

## Scope

This track covers:

- TAA and TAAU history workflows;
- jittered projection and history invalidation;
- motion vectors, depth, exposure, luminance, transparency/reactive masks, and
  disocclusion diagnostics;
- dynamic internal render resolution separate from output resolution;
- raymarch and ray-query reconstruction inputs;
- optional FSR-style adapter hooks and unsupported capability diagnostics.

It does not make FSR, DLSS, XeSS, frame generation, or any vendor technology a
required baseline renderer path.

## Evidence

Runtime evidence must report internal/output resolution, history validity,
motion-vector availability, reconstruction mode, upscaler capability state,
GPU timing, quality diagnostics, and fallback to native resolution when inputs
or backend capabilities are unavailable.
