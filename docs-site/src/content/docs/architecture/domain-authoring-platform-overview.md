---
title: Domain Authoring Platform Overview
description: Architecture overview for Runenwerk domain authoring, linking the shared source/program pattern, typed counter app-program proof, UI framework runtime docs, UI source lowering, and deferred non-UI proof candidates.
status: active
owner: workspace
layer: architecture
canonical: false
last_reviewed: 2026-07-08
related:
  - ../design/active/domain-authoring-source-and-program-pattern.md
  - ../design/active/typed-app-program-counter-proof-design.md
  - ../design/active/ui-framework-runtime-requirements-design.md
  - ../design/active/ui-reactive-runtime-and-invalidation-design.md
  - ../design/active/ui-live-editing-and-preview-design.md
  - ../design/active/ui-game-and-worldspace-host-requirements-design.md
  - ../design/active/ui-source-projection-and-program-lowering-design.md
  - ../design/deferred/material-program-authoring-pattern.md
  - ../design/deferred/procgen-program-authoring-pattern.md
  - ../guidelines/domain-program-architecture-pattern.md
---

# Domain Authoring Platform Overview

## Purpose

This page is a short architecture index for the domain authoring direction. It is
not a full design manual and should not duplicate the detailed active/deferred
design docs.

## Direction

Runenwerk domains should be authored through a domain-program lifecycle:

```text
Authoring Source
-> Source Model
-> Normalized Domain Model
-> Domain Program
-> Compiler / Evaluator
-> Runtime Artifact / Output Facts
-> Host Integration
-> Proof / Diagnostics / Migration Reports
```

The lifecycle is shared. Domain meaning is not shared.

## Core Rule

```text
Domains own meaning.
The platform owns structure.
```

Examples of domain meaning:

```text
UI controls
material nodes
procgen rules
render passes
animation states
behavior nodes
gameplay effects
asset import rules
editor tool behavior
```

Examples of platform structure:

```text
stable ids
versions
package manifests
source-map envelopes
diagnostic envelopes
capability references
artifact manifests
host profiles
compatibility reports
proof report envelopes
invalidation report envelopes after proof
```

## Document Map

| Document | Role |
|---|---|
| `domain-authoring-source-and-program-pattern.md` | Shared source/program lifecycle and extraction rules. |
| `typed-app-program-counter-proof-design.md` | Counter as typed app-program proof with UI projection and win screen at count ten. |
| `ui-framework-runtime-requirements-design.md` | Full UI framework runtime requirements. |
| `ui-reactive-runtime-and-invalidation-design.md` | Reactive update, dependency tracking, retained state, incremental evaluation. |
| `ui-live-editing-and-preview-design.md` | Live editing, preview, hot-swap, diagnostics, last-known-good policy. |
| `ui-game-and-worldspace-host-requirements-design.md` | Game HUD/menu, gamepad navigation, world-space UI, split-screen, input glyphs. |
| `ui-source-projection-and-program-lowering-design.md` | UI-specific `UiSource` vocabulary and lowering into `UiProgram`. |
| `material-program-authoring-pattern.md` | Deferred non-UI material-domain instantiation. |
| `procgen-program-authoring-pattern.md` | Deferred non-UI procgen-domain instantiation. |
| `domain-program-architecture-pattern.md` | Existing guideline for domain-program tracks. |

## Relationship To UI

UI is the first proving domain. UI should use:

```text
UiSource
-> AuthoredUiTemplate
-> NormalizedUiTemplate
-> FormedInteractionModel
-> UiProgram
-> UiRuntimeArtifact
-> UiOutput
```

A mature standalone UI framework additionally requires:

```text
reactive invalidation
retained runtime state
input/focus/navigation
accessibility
layout/style/text systems
animation and transitions
overlay/popup/layering
surface mounting
game and world-space hosts
live editing and preview
inspection and proof reports
```

`UiSource` is a UI-domain source stage. It should not become the platform-wide
source type for material, procgen, render, animation, behavior, tools, or asset
import.

## Relationship To Counter

Counter is a small app-program proof, not a UI runtime ownership pattern.

Counter owns:

```text
CounterModel
CounterAction
Counter reducer
Counter action availability
Counter proof scenarios
```

UI owns:

```text
projection into UiSource
UI package resolution
interaction formation
UiProgram lowering
UI runtime artifacts
UI output and event packets
reactive update reports
```

The active screen is derived from count. When `count >= 10`, projection emits the
win screen. Reset returns count to zero and projection emits the counting screen.

## Relationship To Materials And Procgen

Material and procgen are deferred non-UI proof candidates. They should reuse the
source/program/artifact/host/report lifecycle, but not UI vocabulary.

Material direction:

```text
MaterialSourceGraph
-> NormalizedMaterialGraph
-> MaterialProgram
-> ShaderModuleArtifact / MaterialPipelineArtifact
```

Procgen direction:

```text
ProcgenSourceGraph
-> NormalizedProcgenGraph
-> ProcgenProgram
-> WorldChunkRecipe / SpawnTableArtifact / FieldCacheArtifact
```

Both deferred tracks should help prove which structural ideas can later be safely
abstracted, such as source-map envelopes, diagnostic envelopes, package manifests,
typed graph substrate, artifact manifests, host compatibility matrices, proof
reports, and invalidation reports.

## Relationship To ECS And Graphs

Graphs are common structure. ECS is runtime fabric.

Graphs may represent source relationships, program relationships, dependencies,
ports, resources, control flow, dataflow, bindings, rules, or validation facts.
Their structural substrate may eventually share IDs, edges, ports, traversal,
serialization, source maps, and diagnostic attachment points.

Graph meaning stays domain-owned.

ECS may execute optimized artifacts, hold live runtime state, run schedules, and
bridge host behavior. ECS must not own domain source truth, package catalogs,
program semantics, or app model truth.

## Rejected Direction

Do not use this architecture to justify:

```text
UniversalAst
UniversalNodeGraph
generic graph interpretation in hot paths by default
ECS-owned source truth
renderer-owned product truth
hidden global mutable registries
foundation/meta
shared extraction before UI plus one non-UI proof
```
