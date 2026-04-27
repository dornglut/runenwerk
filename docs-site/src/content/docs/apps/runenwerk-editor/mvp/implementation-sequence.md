---
title: First 3D Editor MVP Implementation Sequence
description: Ordered implementation sequence for the first Runenwerk 3D scene authoring MVP.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-04-27
related:
  - ./first-3d-editor-mvp.md
  - ./acceptance-criteria.md
---

# First 3D Editor MVP Implementation Sequence

## Purpose

This document keeps the MVP implementation order separate from the product scope and acceptance criteria.

## Sequence

Implement in this order:

1. readable editor shell and panel labels;
2. engine-owned editor window/runtime integration;
3. document-driven scene state;
4. projection into runtime/world state;
5. viewport rendering of real scene entities;
6. viewport picking and hit detection;
7. outliner, inspector, and viewport selection synchronization;
8. inspector transform editing;
9. translate gizmo interaction;
10. undo and redo;
11. scene persistence follow-up.

## Rule

Do not expand into post-MVP editor features until the core loop is usable end-to-end.

## Related Documents

- [`first-3d-editor-mvp.md`](./first-3d-editor-mvp.md)
- [`acceptance-criteria.md`](./acceptance-criteria.md)
- [`../roadmap.md`](../roadmap.md)