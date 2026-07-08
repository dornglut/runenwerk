---
title: UI Game And World-Space Host Requirements Design
description: Long-term host requirements for game HUDs, game menus, controller navigation, multi-player focus, world-space UI, diegetic surfaces, safe areas, input glyphs, and runtime host compatibility.
status: active
owner: ui
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ./ui-framework-runtime-requirements-design.md
  - ./ui-reactive-runtime-and-invalidation-design.md
  - ./ui-source-projection-and-program-lowering-design.md
  - ./ui-live-editing-and-preview-design.md
  - ./ui-program-architecture.md
  - ./ui-program-architecture-owner-map.md
---

# UI Game And World-Space Host Requirements Design

## Status

Active long-term UI design direction. This document defines game UI and
world-space UI host requirements for the mature Runenwerk UI framework. It does
not authorize implementation or game/runtime integration by itself.

## Decision

Game UI and world-space UI are first-class UI host profiles. They are not special
cases bolted onto editor UI.

The UI framework must support:

```text
GameHudHost
GameMenuHost
GameOverlayHost
WorldSpaceHost
SplitScreenHost
LocalMultiplayerHost
HeadlessGameUiHost
RemoteGamePreviewHost
```

Game and world-space hosts consume `UiProgram`, `UiRuntimeArtifact`, `UiOutput`,
`UiFrame`, and `UiEventPacket` facts. They must not own UI source truth or app
model truth.

## Game UI Classes

The framework must support common game UI patterns:

```text
HUD overlays
health/resource bars
status effects
quest/objective trackers
inventory screens
radial menus
pause menus
settings menus
dialogue UI
shop/trade UI
map/minimap UI
context prompts
input action prompts
damage/healing numbers
loot notifications
tutorial overlays
crosshair/reticle UI
party/raid frames
chat/log panels
```

These are product/domain meanings layered on top of UI controls and host
capabilities. UI provides the framework; game domains own game semantics.

## Game Input Requirements

Required game input classes:

```text
gamepad buttons
gamepad sticks
gamepad d-pad
keyboard/mouse
pointer/touch
virtual cursor
radial selection
long press / hold
input chord
accessibility action
local-player scoped input
```

Input routing must support:

```text
input modality detection
input glyph selection
focus scopes
modal input layers
selective interactivity
input capture
pointer capture
gameplay-vs-ui input priority
pause/menu input mode
```

UI may propose route/action facts. Gameplay input policy and game state mutation
belong to game/runtime hosts.

## Gamepad And Controller Navigation

Gamepad navigation is required for mature game UI.

Navigation requirements:

```text
cardinal navigation
explicit navigation edges
automatic spatial fallback
focus restoration
focus trapping in modal layers
wrap policy
disabled item skipping
multi-column/grid navigation
radial navigation
focus history
per-local-player focus ownership
navigation diagnostics
```

Every focusable control must expose navigation facts or explicitly opt out with a
diagnostic reason.

## Input Glyphs

Input glyph support must be source/program/host-driven.

Required concepts:

```text
InputActionId
InputBindingId
InputDeviceKind
InputGlyphSet
InputGlyphRef
InputGlyphFallback
InputPromptSource
HostInputProfile
```

Game UI source should bind to semantic actions, not hardcoded controller button
images:

```text
ui::input_prompt("interact", GameAction::Interact)
```

The host resolves glyphs from the current input device/profile.

## Safe Area And Scaling

Game UI must support:

```text
safe areas
notches/cutouts
overscan margins
DPI scaling
resolution scaling
aspect-ratio variants
split-screen viewport constraints
VR/AR comfort constraints where applicable
font scaling
accessibility scaling
```

Safe area and scaling facts belong to host profile and surface constraints. Product
UI source may declare intent; final resolution happens in layout/surface host
integration.

## Layering And Priority

Game UI requires explicit layer policy:

```text
HUD layer
notification layer
dialogue layer
menu layer
modal layer
tutorial layer
debug layer
accessibility layer
world-space overlay layer
```

Layer policy must define:

```text
render order
hit-test order
focus priority
input capture
modal blocking
dismissal rules
pause/gameplay interaction
```

Renderer receives derived order facts. It does not own layer semantics.

## World-Space UI Requirements

World-space UI surfaces must support:

```text
world anchors
entity anchors
bone/socket anchors
screen-space projection
billboarding policy
occlusion policy
distance scaling
LOD policy
visibility cones
interaction rays
hit regions in projected space
world-space safe size
surface lifetime
surface ownership
```

World-space UI can host:

```text
nameplates
interaction prompts
dialogue bubbles
health bars
quest markers
in-world panels
diegetic controls
AR/VR-style surfaces
```

World-space projection belongs to surface/host contracts, not generic controls.

## Diegetic And Non-Diegetic UI

The framework must distinguish:

```text
non-diegetic HUD
screen-space menu
world-space overlay
true diegetic surface
hybrid projected prompt
```

This matters for input routing, occlusion, scaling, depth testing, and
accessibility fallbacks.

## Multi-Player And Split-Screen

Local multiplayer requires:

```text
LocalPlayerId
PlayerUiContext
per-player focus scope
per-player input routing
per-player safe area
per-player HUD surface
shared menu ownership policy
split-screen viewport binding
conflict diagnostics
```

No global singleton focus state is acceptable for local multiplayer.

## Performance Requirements

Game UI hot paths require:

```text
incremental invalidation
stable runtime artifacts
output deltas
batched render packets
text/layout cache keys
glyph atlas preparation keys
minimal allocation in frame hot paths
visibility culling
world-space surface LOD
dirty-region rendering where supported
```

Runtime hot paths consume artifacts and runtime state. They must not interpret
authoring source graphs by default.

## Host Compatibility Matrix

Game/world-space host compatibility must report:

```text
supported input devices
supported focus/navigation modes
supported surface kinds
supported layering modes
supported text/font features
supported accessibility features
supported animation features
supported world projection features
unsupported controls/packages
required fallbacks
```

## Proof Requirements

Required proof classes:

```text
GameHudProof
GameMenuProof
GamepadNavigationProof
InputGlyphProof
SafeAreaProof
SplitScreenFocusProof
WorldSpaceProjectionProof
WorldSpaceHitTestProof
LayeringModalProof
AccessibilityFallbackProof
```

A first game UI proof should demonstrate:

```text
HUD counter label updates reactively
gamepad navigation activates reset button
input glyph changes when host device changes
modal menu traps focus
safe-area layout changes with viewport profile
headless replay records all route/focus/navigation facts
```

A first world-space proof should demonstrate:

```text
surface anchored to world/entity
projection to screen-space hit regions
visibility/occlusion decision reported
interaction route emitted through UiEventPacket
host compatibility matrix records supported projection mode
```

## Rejected Shapes

Reject:

```text
global singleton focus for all players
hardcoded controller glyph images in UI source
renderer-owned layer semantics
world-space projection inside base controls
product app manual hit-test construction
UI controls mutating gameplay state directly
game UI as editor-only surface workaround
unreported input routing or focus changes
```
