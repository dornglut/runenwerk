---
title: Texture Domain
description: Accepted ownership boundary for Texture2D, Texture3D, generated texture products, sampler policy, and preview descriptors.
status: accepted
owner: texture
layer: domain
canonical: true
last_reviewed: 2026-05-09
related_docs:
  - ../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
  - ../../design/active/editor-asset-pipeline-and-content-workflow-design.md
  - ../material-graph/README.md
---

# Texture Domain

`domain/texture` owns texture product descriptions. Texture authoring and preview surfaces may inspect those products, but editor UI state and renderer upload handles are not texture truth.

## Pipeline

Every texture slice follows this invariant:

```text
authored texture source or generated texture document
  -> structural validation
  -> texture ratification
  -> normalized texture descriptor
  -> formed texture product
  -> asset artifact/revision
  -> runtime/preview product
  -> editor diagnostics/surfaces
```

Runtime and preview systems consume formed texture products, not editor panels, canvas state, or importer scratch data.

## Ownership

This domain owns:

- Texture2D descriptors, Texture3D/volume descriptors, dimensions, mip policy, channel layout, color-space policy, sampler policy, compression policy, and inspection metadata;
- generated texture product descriptors, deterministic cache keys, dependency/source lineage, and failed-product preservation semantics;
- preview descriptors for slice, mip, channel, color-space, and volume inspection;
- texture issue codes, ratifiers, diagnostics, and source maps.

This domain does not own:

- GPU upload, residency, bind groups, or renderer cache objects;
- platform file dialogs, importer process execution, or app-local cache folders;
- material graph semantics, except through typed product references consumed by `domain/material_graph`;
- live-reload behavior until an engine/app adapter has a formed texture product contract.

## Accepted First Slice

The first implementation slice may add:

- Texture2D descriptors with color-space and compression policy;
- Texture3D/volume descriptors with dimensions, channel layout, slice/mip inspection metadata, and cache key inputs;
- generated texture product descriptors for procedural texture outputs;
- sampler descriptors for filter, wrap, anisotropy, and comparison mode where relevant;
- source-mapped diagnostics for invalid dimensions, unsupported formats, invalid color-space/compression combinations, missing source products, and unsupported preview/upload targets.

The accepted Texture3D policy is descriptor-first: the domain owns dimensions, metadata, ratification, and preview intent; apps/engine adapters own GPU upload and runtime resource lifetimes. Editor previews start with slice/mip/channel inspection descriptors and remain fail-closed until an adapter can produce a real runtime product.

The editor owns descriptor-first Texture Viewer and Volume Texture Viewer providers. These surfaces may display texture product descriptors, generated-product lineage, cache keys, reload status, and `TexturePreviewDescriptor` slice/mip/channel intent. They must remain fail-closed when no formed texture product or runtime adapter exists.

## Gates

Do not implement runtime texture upload or reload classification until:

- `domain/texture` exists and ratifies texture candidates;
- formed texture products carry deterministic cache keys and source lineage;
- editor preview consumes typed texture preview descriptors instead of renderer-private handles;
- asset artifact/revision handling preserves the prior valid texture product on failed formation.

Do not put renderer-specific upload handles, shader binding slots, or material graph node semantics in this domain.
