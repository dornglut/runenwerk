---
title: SDF Character Animation System Design
description: Deferred detail draft for SDF-first character modelling, rigging, animation, products, and runtime handoff.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ./sdf-prefab-composition-system-design.md
---

# SDF Character Animation System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after prefab composition, rig/pose ownership, and collision/render product
contracts are accepted.

This document defines character modelling and animation for an SDF-first engine.

Characters are not mesh-skinned assets in the primary architecture. They are animated SDF/field prefabs that produce render, collision, interaction, and gameplay products.

---

# Purpose

Runenwerk needs a production character system for:

- player characters
- enemies
- creatures
- animated props
- fairytale/dark fantasy beings
- future procedural bodies

The system must support:

- SDF body graphs
- rig/pose controls
- animation clips
- animation graphs
- field deformation
- material masks
- collision/query products
- interaction emitters
- renderer handoff
- AI/gameplay hooks
- diagnostics

---

# Core Concept

```text
SDF Character Definition
  -> Rig / Pose Model
  -> Animation Evaluation
  -> Animated SDF Pose Product
  -> Render / Collision / Interaction Products
```

A character is a specialized SDF prefab with animated field components.

---

# Design Goals

1. Make character modelling SDF-first.
2. Keep animation description separate from runtime evaluation.
3. Support pose-driven SDF field deformation.
4. Support player and enemy characters through the same product model.
5. Separate render products from collision/query products.
6. Support field interaction emitters such as footsteps, grass bending, wetness, threat, scent, and sound.
7. Support LOD/fallback without visual popping or gameplay authority errors.
8. Support diagnostics for pose, products, and animation state.
9. Leave room for procedural animation and future editor tools.
10. Avoid mesh skinning as the primary path.

---

# Non-Goals

This design is not:

- a mesh skinning system
- a full animation editor UI
- a behavior tree design
- a combat system
- a physics solver
- a renderer implementation
- a final asset format

It defines character animation architecture and product contracts.

---

# Character Definition Model

A character definition should include:

| Field | Meaning |
|---|---|
| Character identity | Stable ID. |
| Display name | Human-readable name. |
| Body graph | SDF body composition. |
| Rig definition | Controls, joints, anchors, constraints. |
| Animation set | Clips, blend spaces, state machines, graph refs. |
| Material masks | Skin, cloth, armor, hair, glow, etc. |
| Collision model | Query/collision output policy. |
| Interaction emitters | Footsteps, scent, threat, bend, sound, etc. |
| Bounds policy | Static and dynamic conservative bounds. |
| LOD policy | Render/collision/animation bands. |
| Diagnostics | Definition validation. |

---

# SDF Body Graph

The body graph may include:

- torso fields
- head fields
- limb fields
- hand/foot fields
- cloth/armor fields
- hair/fur fields
- accessory fields
- eye/glow fields
- deformation masks
- smooth unions
- attachments
- material channels

Example:

```text
HumanoidCharacter
  torso
  pelvis
  head
  upper/lower arms
  hands
  upper/lower legs
  feet
  cloak/hair optional fields
```

The graph should support stylized silhouettes and non-human creatures.

---

# Rig and Pose Model

The rig controls field components.

A rig may include:

- joints
- control handles
- field anchors
- constraints
- IK targets
- attachment sockets
- blend masks
- deformation regions
- pose-space parameters

A pose product should describe:

- control transforms
- field-node transforms
- blend weights
- deformation parameters
- material/visibility toggles
- conservative bounds
- generation

---

# Animation Model

Animation descriptions should be serializable and inspectable.

Supported concepts:

- clips
- blend spaces
- state machines
- animation graphs
- layered blends
- masks
- events
- root motion policy
- procedural modifiers
- footstep markers
- interaction events

Runtime evaluation should produce pose products.

```text
animation state
  -> pose evaluation
  -> animated SDF pose product
  -> render/collision/interaction products
```

---

# Runtime Products

A character may produce:

| Product | Purpose |
|---|---|
| Animated SDF pose product | Renderer field deformation input. |
| Character render product | Visible SDF character. |
| Collision/query product | Gameplay/physics queries. |
| Interaction emitter product | Footsteps, grass bending, sound, threat. |
| Influence product | Enemy/player presence, danger, scent. |
| Diagnostic product | Pose, bounds, stale/fallback state. |

---

# Renderer Integration

Renderer receives:

- character bounds
- SDF body graph reference
- pose transforms
- deformation parameters
- material masks
- LOD/fade state
- product generation
- diagnostics flags

Renderer does not own animation truth.

---

# Collision and Physics Integration

Collision can use simpler products than visual rendering.

Examples:

- capsule-like SDF body
- conservative body field
- per-limb query fields
- foot contact fields
- attack/hit query volumes

Rules:

1. Collision products are strict when gameplay-relevant.
2. Visual fallback cannot satisfy strict collision.
3. Collision LOD must not switch during active contact.
4. Pose/collision products must stay generation-compatible.

---

# Interaction Emitters

Characters can emit field interactions.

Examples:

| Emitter | Product Impact |
|---|---|
| Footstep | grass bend/trample, sound, wetness ripple |
| Presence | scent/threat/influence |
| Attack | impact field, damage query |
| Glow | radiance/material response |
| Movement | wind/grass disturbance |
| Death/decay | substance/influence products |

Emitters should be product-scoped and diagnosable.

---

# Character LOD

Character LOD has several independent axes.

| Axis | Near | Far |
|---|---|---|
| Render | full animated SDF | simplified SDF / silhouette |
| Animation | full graph | reduced update rate / summary pose |
| Collision | strict body query | no collision or server summary |
| Interaction | full emitters | reduced/disabled |
| AI | active perception | strategic summary/dormant |

LOD transitions must avoid visible popping and gameplay authority errors.

---

# Player Character

Player character requirements:

- high-priority residency
- strict collision
- responsive animation
- footstep emitters
- field interaction
- camera-relative render priority
- diagnostics for movement/collision/render products

---

# Enemy Character

Enemy requirements:

- same SDF character system
- AI/perception hooks
- threat/influence emitters
- active/dormant simulation states
- animation LOD
- multiplayer authority later

---

# Authoring Model

Authoring should support:

- SDF body graph editing
- rig control editing
- animation clip assignment
- material mask assignment
- interaction emitter configuration
- preview poses
- product preview
- diagnostics

The first implementation may not include full UI, but data contracts should allow it.

---

# Multiplayer

Replicate:

- authoritative character state
- movement/pose-relevant gameplay state
- animation state where required
- interaction events
- generation/state changes

Do not replicate:

- local render pose cache
- GPU buffers
- client-only visual smoothing
- editor-only diagnostics

Authority rules should be explicit per product.

---

# Diagnostics

Character diagnostics should expose:

- invalid body graph
- missing rig controls
- invalid pose
- stale pose product
- render/collision generation mismatch
- missing collision product
- dynamic bounds invalid
- LOD fallback active
- emitter product missing
- animation state error

---

# Open Questions

1. What is the minimum SDF rig representation for first production characters?
2. Should rig controls live in a new animation domain or reuse scene transforms initially?
3. What animation graph substrate should be used?
4. How should SDF body graphs encode deformation masks?
5. What collision shape is required for first player movement?
6. How much root motion is supported initially?
7. How are footstep events generated from SDF pose data?
8. How should enemy animation update rates scale with distance?
9. How are character material masks represented?
10. What parts of animation state are multiplayer-authoritative?

---

# Design Decisions

1. Characters are animated SDF prefabs.
2. Mesh skinning is not the primary architecture.
3. Animation description and runtime evaluation are separate.
4. Pose products are first-class.
5. Render and collision products are separate.
6. Footstep/interaction emitters are required.
7. Player and enemies share the same character product model.
8. LOD must not break gameplay authority.
9. Diagnostics are required.
10. Future procedural animation must fit the same product model.

---

# Implementation Phases

## Phase 1: Character Descriptor Contract

Deliver:

- character definition descriptor
- body graph reference
- rig descriptor
- animation set descriptor
- product outputs
- diagnostics

## Phase 2: Simple SDF Humanoid

Deliver:

- torso/head/limb SDF body
- simple rig controls
- conservative bounds
- render product

## Phase 3: Pose and Animation

Deliver:

- idle/walk/run clips
- pose product
- animation state machine
- renderer handoff

## Phase 4: Collision and Movement

Deliver:

- strict collision/query product
- foot contact points
- generation compatibility checks

## Phase 5: Interaction Emitters

Deliver:

- footstep trample/bend emitter
- sound/influence emitter
- wetness/ripple emitter near water

## Phase 6: Enemy Character

Deliver:

- enemy SDF prefab
- animation products
- threat/perception emitter
- diagnostics

## Phase 7: LOD and Streaming

Deliver:

- animation update LOD
- render LOD
- dormant enemy summaries
- fallback rules

---

# Acceptance Criteria

This design is accepted when:

1. A player character can be represented as an animated SDF prefab.
2. Pose evaluation produces renderable SDF pose products.
3. Collision/query products are separate and strict.
4. Footstep interaction emitters are product-scoped.
5. Enemies can reuse the same character model.
6. Renderer does not require mesh skinning.
7. Diagnostics explain pose/render/collision product state.
