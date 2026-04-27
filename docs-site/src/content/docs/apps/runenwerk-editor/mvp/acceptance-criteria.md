---
title: First 3D Editor MVP Acceptance Criteria
description: Acceptance criteria for the first usable Runenwerk 3D scene authoring MVP.
status: active
owner: editor
layer: app
canonical: true
last_reviewed: 2026-04-27
related:
  - ./first-3d-editor-mvp.md
  - ./implementation-sequence.md
---

# First 3D Editor MVP Acceptance Criteria

## Purpose

This document defines when the first editor MVP is usable enough to count as the first 3D scene authoring slice.

## Core Acceptance Criteria

The MVP is accepted when a user can:

- launch the editor window;
- read panel titles and field labels;
- see at least one real scene entity in the viewport;
- select entities from the viewport and outliner;
- inspect the selected entity;
- edit basic properties;
- move an entity with a translate gizmo;
- undo and redo those edits.

## Scene Authoring Acceptance

The MVP scene authoring loop must support:

- a simple 3D scene;
- basic primitive entities;
- visible transforms;
- named entities;
- selection synchronization across viewport, outliner, and inspector;
- movement working end-to-end.

## Out of Scope

The MVP does not need to prove:

- production-grade editor UX;
- full asset workflows;
- arbitrary component authoring;
- docking/layout persistence;
- rotate/scale gizmos;
- UI authoring mode;
- production scene persistence beyond the near-immediate follow-up slice.

## Related Documents

- [`first-3d-editor-mvp.md`](./first-3d-editor-mvp.md)
- [`implementation-sequence.md`](./implementation-sequence.md)
- [`../roadmap.md`](../roadmap.md)