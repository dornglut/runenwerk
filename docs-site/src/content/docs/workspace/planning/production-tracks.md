---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
---

# Production Tracks

Use this file for strategic production-track planning in Markdown.

## PT-UI-COMPONENT-PLATFORM

Track ID: `PT-UI-COMPONENT-PLATFORM`

Title: UI Component Platform

State: active

Goal:

```text
Make reusable Runenwerk UI controls and component surfaces story-proven, descriptor-backed, inspectable, and consumable by Gallery, Workbench, UI Designer, and future product tracks without moving reusable control semantics into those consumers.
```

Milestones:

```text
001 ControlPackage / ControlKernel contract — completed by user report
002 Authoring Kit — active design/planning
003 Story proof envelope consumption — future
004 Catalog / discovery / inspection — future
005 Input / gesture / device — future
006 State binding / host intent — future
007 Theme / state / style — future
008 Accessibility / focus / inspection — future
009 Layout / container / virtualization — future
010 Render surface / output — future
011 Base control packages — future
012 Generic interaction — future
013 Overlay / popup / layering — future
014 Minimum text editing — future
015 Generic text — future
016 Surface2D — future
017 SpatialCanvas — future
018 NodeCanvas — future
019 PortGraphCanvas — future
020 ProgressionTreeView — future
021 TrackSurface / Timeline — future
022 Transitions / effects — future
023 Adoption gates — future
024 Runtime-proven closeout — future
```

Design gates:

```text
Phase 2 implementation requires the authoring-kit design to be accepted and Phase 1 to remain green.
Each later milestone requires its own owning design or planning update before code.
```

Evidence gates:

```text
Phase 1: local validation and branch/PR evidence.
Phase 2: authoring API tests prove ordinary Phase 1 descriptors are produced and invalid output still fails closed through existing validation.
Later phases: story proof, diagnostics, docs evidence, and runtime-proof gates as appropriate.
```

Current blocker:

```text
Phase 2 is in design/planning. No Rust implementation is authorized by planning alone.
```

Next action:

```text
Review and accept ui-component-platform-authoring-kit-design.md, then start a bounded implementation pass for the authoring kit.
```

## Track shape

```text
Track ID:
Title:
State:
Goal:
Milestones:
Design gates:
Evidence gates:
Current blocker:
Next action:
```

## Rules

- Production tracks guide sequencing.
- Production tracks do not authorize code without an owned implementation scope.
- Strategic order must be readable without generated views.
- Legacy structured track files may remain as optional mirrors.
