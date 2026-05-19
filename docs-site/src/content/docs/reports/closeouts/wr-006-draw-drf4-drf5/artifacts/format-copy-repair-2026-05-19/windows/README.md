---
title: Windows Format Copy Repair Evidence
description: Pending Windows evidence slot for the Draw render copy format repair.
status: active
owner: apps/runenwerk_draw
layer: apps
canonical: false
last_reviewed: 2026-05-19
---

# Windows Evidence Pending

Windows capture must be produced from the repaired windowed Draw runtime before
this platform is considered complete.

Smoke status on 2026-05-19: the repaired windowed runtime stayed alive until
manual termination after the old crash window. No render copy format panic and
no `Rgba8Unorm` read-write storage texture binding panic were observed during
that run. This is not a replacement for the required formal artifact files.

Required files:

- `render-format-report.ron`
- `gpu-ink-copy-proof.json`
- `screenshot-or-capture.png`
