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
002 Authoring Kit — completed by user validation report
003 Story proof envelope consumption — completed by user validation report
004 Catalog / discovery / inspection — next design/planning
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
Phase 4 implementation requires a Catalog / Discovery / Inspection design to be accepted first.
Each later milestone requires its own owning design or planning update before code.
```

Evidence gates:

```text
Phase 1: local validation and branch/PR evidence.
Phase 2: user reported the authoring-kit validation gate green; authoring tests prove ordinary Phase 1 descriptors are produced and invalid output still fails closed through existing validation.
Phase 3: user reported the story-proof validation gate green; story-proof tests prove requirements, expected-failure requirements, first-blocker summaries, and conservative mount eligibility without executing stories inside ui_controls.
Later phases: catalog/discovery, diagnostics, docs evidence, and runtime-proof gates as appropriate.
```

Current blocker:

```text
Phase 4 is not yet designed. No catalog implementation, Gallery preview behavior, Designer UX, or Workbench behavior is authorized.
```

Next action:

```text
Open PT-UI-COMPONENT-PLATFORM-004 Catalog / Discovery / Inspection design/planning.
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
