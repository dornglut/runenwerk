# Fully ECS-First Render Proposal

Date: `2026-02-24`  
Status: `proposed`

## Objective

Render core should not define, own, or require any concrete world/frame payload type.

Render core should consume only ECS state prepared by feature/plugin systems.

## Core Principles

1. ECS is the source of truth.
2. Feature plugins publish render inputs as ECS resources/components.
3. Render plugin reads typed ECS data through generic contexts.
4. No render-local adapter structs for world-specific payloads.
5. No hardcoded builtins/defaults in render core.

## Proposed Architecture

### A) Data Production (Feature Plugins)

- Feature plugins register normal ECS systems in pre-render schedule stages.
- Those systems write/update render-relevant resources/components in `EngineData.render_resources` (or dedicated ECS worlds owned by the feature).
- Examples:
  - camera state resource
  - world bounds resource
  - SDF proxy component sets
  - UI stats/resource payloads

### B) Data Consumption (Render Core)

- Render submit does not build/adapt a concrete frame struct.
- Render pass contexts expose typed lookup from ECS-backed frame data access:
  - `ctx.frame_data::<T>()`
  - optional keyed lookups for multi-source data of the same type
- Executors decide how to handle missing optional data.
- Missing required data is reported with actionable diagnostics.

### C) Scheduling

- Add explicit render-prep schedule stage before `frame_render_submit`.
- Stage ordering:
  1. feature/world/ui extract systems write ECS render resources
  2. render submit compiles graph and executes passes
- This replaces a separate provider-registry concept; providers are plain ECS systems.

## API Direction

Render plugin should keep:

- frame graph registry
- pass executor registry
- typed context lookup APIs
- diagnostics

Render plugin should remove:

- concrete frame payload params in renderer/gfx APIs
- any `WorldRenderFrame`/feature-specific type ownership
- adapter paths from runtime data -> render packet structs

## Migration Plan

1. Introduce a render-prep stage and move all current frame population into ECS systems.
2. Replace `RenderFrameData` usage in core renderer with direct typed ECS lookups.
3. Remove `WorldRenderFrame` and `RenderFrameData` ownership from render domain.
4. Move any remaining payload types to owning feature plugins (scene/ui/sdf/etc.).
5. Keep compatibility shims temporarily only where unavoidable, then delete them.

## Acceptance Criteria

- Render core accepts no concrete frame struct arguments.
- At least two independent feature plugins publish different data types in ECS and render consumes both in one frame.
- A new renderer feature can be added without modifying render core files.
- Missing required ECS data produces diagnostics (no release-path panic).

## Non-Goals

- Moving ECS core abstractions unless a concrete ECS limitation is identified.
- Reintroducing render-local provider registries that duplicate ECS scheduling/state.
