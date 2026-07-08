---
title: UI Live Editing And Preview Design
description: Long-term live-editing, visual-design, hot-swap, preview-host, diagnostics, and rollback model for Runenwerk UI source and runtime artifacts.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
---

# UI Live Editing And Preview Design

## Status

Active long-term UI design direction. This document defines the live-editing and
preview model needed for a mature Runenwerk UI framework. It does not authorize
implementation or editor integration by itself.

## Decision

Runenwerk UI live editing uses the same source/program/artifact/evaluator/host
pipeline as runtime UI. Live editing must not be a separate bypass path.

Correct live-edit loop:

```text
source edit
-> UiSource revision
-> validation
-> normalization
-> interaction formation
-> UiProgram diff
-> artifact invalidation / rebuild
-> retained state policy
-> preview evaluation
-> preview host output
-> diagnostics and source-map overlays
```

Invalid source must not corrupt the current running preview. The preview host
keeps the last known good program/artifact and overlays diagnostics until a valid
replacement is available.

## Live Editing Goals

The live-editing system must support:

```text
Rust-authored projected UI preview
RON/template source preview
visual designer source preview
generated projection source preview
style/theme live editing
layout live editing
control package fixture preview
host-profile preview
world-space preview
game HUD preview
headless proof preview
source-map diagnostic navigation
rollback to last known good state
```

## Source Edit Events

A live edit produces `UiSourceEditEvent`:

```text
edit id
source id
source version before
source version after
edit kind
edited source span or visual element id
authoring tool id
host profile id
timestamp/revision
```

Edit kinds:

```text
NodeInserted
NodeRemoved
NodeMoved
PropertyChanged
BindingChanged
ActionBindingChanged
StyleTokenChanged
ThemeChanged
TextKeyChanged
AccessibilityChanged
TemplateInstanceChanged
PackageRequirementChanged
SourceBodyReplaced
```

## Live Edit Pipeline

### 1. Validate Source

Validate source syntax/body shape, stable ids, package requirements, schemas,
action bindings, localization keys, accessibility metadata, style references, and
host requirements.

Output:

```text
UiSourceValidationReport
```

### 2. Normalize Source

Canonicalize the edited source into normalized UI definition inputs.

Output:

```text
UiNormalizationReport
NormalizedUiTemplate revision
source-map tables
```

### 3. Form Interaction Model

Recompute affected interaction facts:

```text
focus
routes
actions
scroll ownership
popup/layering
dismissal
navigation
host capabilities
```

Output:

```text
FormedInteractionReport
```

### 4. Diff UiProgram

Compare previous and next `UiProgram` revisions.

Output:

```text
UiProgramDiffReport
added program nodes
removed program nodes
changed program nodes
changed routes
changed bindings
changed package requirements
changed capabilities
migration requirements
```

### 5. Invalidate Artifacts

Map program/source changes to artifact cache invalidation.

Output:

```text
UiArtifactInvalidationReport
cache hits
cache misses
rebuilt artifact tables
reused artifact tables
```

### 6. Retain Or Reset Runtime State

Apply `UiStateRetentionPolicy` to focus, hover, scroll, text edit, animation,
popup stack, and package-owned state.

Output:

```text
UiStateRetentionReport
```

### 7. Evaluate Preview

Evaluate the preview using the chosen host profile and preview data.

Output:

```text
UiPreviewEvaluationReport
UiOutput / UiOutputDelta
UiInspectionReport
UiDiagnosticReport
```

### 8. Apply To Preview Host

Preview host applies output or output delta and overlays diagnostics.

Output:

```text
UiPreviewHostReport
```

## Preview Host Profiles

Required preview profiles:

```text
DesktopPreviewHost
EditorPanelPreviewHost
GameHudPreviewHost
GameMenuPreviewHost
WorldSpacePreviewHost
HeadlessPreviewHost
RemotePreviewHost
AccessibilityPreviewHost
```

A preview host must declare:

```text
input capabilities
render capabilities
text/font capabilities
accessibility support
surface constraints
world-space projection constraints where applicable
available package set
hot-swap support
state retention support
diagnostic overlay support
```

## Last Known Good Policy

Live preview must maintain:

```text
last known good source
last known good normalized model
last known good UiProgram
last known good artifact
last known good runtime state snapshot
last known good preview output
```

If a new edit fails:

```text
keep last known good artifact
show diagnostics overlay
preserve source-map navigation
record failed preview revision
block hot-swap into runtime hosts
allow continued editing
```

## Hot-Swap Plan

Hot-swap is allowed only after:

```text
source validation accepted
normalization accepted
interaction formation accepted
program compatibility accepted
artifact build accepted
host compatibility accepted
state retention policy accepted
```

Hot-swap plan fields:

```text
old program id / revision
new program id / revision
old artifact id
new artifact id
compatibility decision
state retention decisions
host application steps
rollback artifact id
rollback state snapshot id
diagnostics
```

## Visual Designer Integration

A visual designer is one `UiSource` authoring frontend.

It must produce source, not runtime widgets:

```text
VisualDesignerBody
source node ids
source-map locations
style/theme references
package requirements
localization keys
accessibility metadata
```

The designer may display live runtime preview, but it must not become the semantic
owner of UI behavior. Designer edits go through the same validation/lowering path
as Rust and file-based source.

## Rust Projection Preview

Projected UI from an app program should be previewable with fixture model data.

Required fixture inputs:

```text
AppModelSnapshot
HostDataSnapshot
UiRuntimeStateSnapshot optional
ThemeSnapshot optional
PreviewHostProfile
```

For the counter proof, preview fixtures should include:

```text
count = 0 -> counting screen
count = 9 -> counting screen
count = 10 -> win screen
count = 10 + reset route -> counting screen after action
```

## Diagnostic Overlay

Preview diagnostics should show:

```text
source validation errors
schema errors
missing package errors
missing capability errors
route/action binding errors
accessibility warnings
layout overflow warnings
theme token missing errors
text localization missing warnings
artifact build errors
host compatibility errors
```

Every diagnostic must include stable id, severity, source-map link when possible,
and suggested repair where available.

## Live Preview Reports

Required reports:

```text
UiSourceEditReport
UiSourceValidationReport
UiNormalizationReport
FormedInteractionReport
UiProgramDiffReport
UiArtifactInvalidationReport
UiStateRetentionReport
UiPreviewEvaluationReport
UiPreviewHostReport
UiHotSwapPlan
UiLivePreviewSessionReport
```

## Rejected Live-Edit Shapes

Reject:

```text
visual designer mutates retained runtime widgets directly
runtime preview bypasses UiProgram
invalid source replaces last known good artifact
manual renderer hot-swap from product code
unreported artifact invalidation
source edits without source-map diagnostics
host-specific preview behavior hidden in UI source
```

## Acceptance Criteria

A first implementation slice should prove:

```text
edit source text/property
validate and normalize source
produce UiProgram diff
reuse or rebuild artifacts with report
preserve or reset runtime state with report
show preview output
show diagnostics for invalid source
keep last known good preview on invalid edit
run same path in headless preview fixture
```
