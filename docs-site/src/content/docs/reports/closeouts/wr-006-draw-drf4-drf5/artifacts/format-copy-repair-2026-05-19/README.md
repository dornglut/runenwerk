---
title: Format Copy Repair Artifacts 2026-05-19
description: Cross-platform artifact requirements and pending evidence for the Draw render copy format repair.
status: active
owner: apps/runenwerk_draw
layer: apps
canonical: false
last_reviewed: 2026-05-19
---

# Format Copy Repair Artifacts - 2026-05-19

This folder stores cross-platform evidence for the long-term render copy format
repair.

Expected platform layout:

```text
macos/
  render-format-report.ron
  gpu-ink-copy-proof.json
  screenshot-or-capture.png
windows/
  render-format-report.ron
  gpu-ink-copy-proof.json
  screenshot-or-capture.png
```

Each platform report must record:

- OS
- GPU adapter name
- `wgpu` backend
- selected surface format
- supported surface format list, if available
- source texture resource id
- source texture format
- destination texture resource id
- destination texture format
- copy pass id
- raw-compatible decision
- whether Draw CPU reference target used exact `Rgba8Unorm`
- whether GPU ink validation reached report processing without renderer crash

## Current Evidence State

- Windows: formal artifact export pending. A repaired windowed smoke run on
  2026-05-19 stayed alive until manually terminated after the old crash window,
  with no render copy format panic or storage-texture binding validation panic.
- macOS: pending. The current implementation workspace is Windows-only, so no
  macOS evidence is recorded here.

Do not add platform evidence unless it was captured on that platform.
