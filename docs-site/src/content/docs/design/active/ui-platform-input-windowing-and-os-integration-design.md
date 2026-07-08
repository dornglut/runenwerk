---
title: UI Platform Input Windowing And OS Integration Design
description: Long-term platform integration requirements for UI windows, surfaces, pointer/keyboard/text/IME input, clipboard, drag-and-drop, cursor, OS accessibility bridge, files, and host-side services.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-accessibility-internationalization-and-text-conformance-design.md
  - ./ui-game-and-worldspace-host-requirements-design.md
  - ./ui-package-security-versioning-and-migration-design.md
  - ./ui-program-architecture-owner-map.md
---

# UI Platform Input Windowing And OS Integration Design

## Status

Active long-term UI design direction. This document defines platform, windowing,
input, clipboard, drag-and-drop, cursor, OS accessibility bridge, and host service
requirements. It does not authorize implementation by itself.

## Decision

Platform integration is a host boundary. UI source/programs declare intent and
consume normalized facts. Platform hosts perform OS/window/input side effects.

Correct direction:

```text
OS/window/game platform input
-> HostInputFacts
-> NormalizedUiInput
-> UI routing/evaluation
-> UiEventPacket / output facts
-> host side effects only after route/action/capability decisions
```

UI must not call OS services directly from controls.

## Window And Surface Integration

Platform hosts must model:

```text
WindowId
SurfaceId
ViewportId
MonitorId
DpiScale
SafeArea
WindowFocusState
WindowVisibilityState
WindowResizeEvent
WindowCloseRequest
FullscreenMode
CursorConfinement
```

Window/surface facts feed layout, input routing, rendering, and accessibility.

## Input Normalization

Input sources:

```text
mouse
pointer
touch
stylus/pen
tablet
keyboard
gamepad/controller
wheel/scroll
text input
IME
accessibility action
virtual/remote input
```

Normalized input must carry:

```text
input id
source device kind
device id where available
local player id where applicable
surface id
position in surface coordinates
pressure/tilt/contact geometry where available
button/key state
modifiers
timestamp/revision
trusted/untrusted flag where applicable
```

Pointer-like inputs should follow hardware-agnostic pointer principles: mouse,
pen, and touch can share routing logic where appropriate, while still preserving
input-specific facts.

## Keyboard And Text Input

Keyboard events and text input are different channels.

Required facts:

```text
physical key
logical key
text input
modifiers
repeat state
composition state
shortcut/chord candidate
keyboard layout info where host exposes it
```

Text input must support IME/preedit/candidate behavior where the host exposes it.
Product code must not approximate IME by raw key events.

## Clipboard

Clipboard access requires explicit capabilities:

```text
ui.clipboard.read
ui.clipboard.write
```

Clipboard facts/proposals:

```text
ClipboardReadProposal
ClipboardWriteProposal
supported mime/types
plain text
rich text where supported
image refs where supported
security/trust decision
host acceptance/rejection
```

Controls must not silently read or write clipboard.

## Drag And Drop

Drag/drop requires explicit route and capability facts:

```text
DragSourceId
DropTargetId
DragPayloadSchema
DropEffect
DropPreview
DragEnter/Over/Leave/Drop facts
ui.dragdrop.read
ui.dragdrop.write
```

External file drops are host/platform facts and require capability checks before
being accepted into app/editor/domain state.

## Cursor And Pointer Capture

Cursor and capture are host side effects proposed by UI:

```text
CursorIntent
CursorIconRef
PointerCaptureRequest
PointerCaptureRelease
CursorConfinementRequest
```

Host may accept, reject, or substitute. Decisions must be reportable.

## OS Accessibility Bridge

Where supported, desktop/editor hosts should bridge derived semantic UI facts to
native accessibility APIs.

Required bridge facts:

```text
semantic node id
role/name/description/value/state
bounds
focus
relations
live region updates
action invocations
host accessibility object id where applicable
```

Unsupported bridge features must produce host compatibility diagnostics.

## File And URL Interaction

UI may propose, but not directly perform:

```text
open file dialog
save file dialog
open URL
reveal in file explorer
import dropped file
export asset
```

These are host/app/editor effects with capability checks and reports.

## Platform Service Capabilities

Candidate capabilities:

```text
ui.clipboard.read
ui.clipboard.write
ui.dragdrop.external.read
ui.dragdrop.external.write
ui.cursor.write
ui.window.request_close
ui.window.fullscreen.write
ui.file_dialog.open
ui.file_dialog.save
ui.url.open
ui.accessibility.bridge
ui.remote.input
```

## Security And Trust

Untrusted packages/source may emit declarative UI but must not gain platform
services without explicit host trust and capability acceptance.

All platform service proposals require:

```text
origin source id
origin package id
capability id
payload schema
host profile
trust decision
accept/reject result
```

## Reports

Required reports:

```text
UiPlatformInputReport
UiInputNormalizationReport
UiWindowSurfaceReport
UiClipboardCapabilityReport
UiDragDropReport
UiCursorReport
UiImeReport
UiPlatformAccessibilityBridgeReport
UiPlatformServiceProposalReport
```

## Rejected Shapes

Reject:

```text
controls directly calling OS APIs
text input implemented only through keydown events
clipboard access without capability report
external file drops accepted without schema/capability checks
native accessibility bridge as renderer-only afterthought
window/surface data hidden in globals
pointer capture without host decision report
```

## Acceptance Criteria

A first platform integration proof should demonstrate:

```text
pointer input normalizes to UI input facts
keyboard activation and text input are separate
IME composition is represented in retained UI state
clipboard write emits host proposal, not direct write
external drop is rejected without capability
window resize invalidates layout through surface facts
semantic tree can be exported or compatibility diagnostic is reported
```
