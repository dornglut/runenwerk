# Render Plugin

## Purpose

Own render graph orchestration, shader registry polling, and frame submission for the active runtime path.

## Usage

- Plugin: `RenderPlugin`
- Schedules: `RenderPrepare`, `RenderSubmit`
- Consumes:
  - render graph registrations
  - executor registrations
  - shader registry updates
  - feature/plugin-provided frame data
  - UI draw data from the active scene/overlay runtime

## Ownership Boundaries

- Owns graph compilation, pass ordering, execution dispatch, and submission lifecycle.
- Owns render diagnostics/timing orchestration.
- Does not own feature/world payload schemas or feature-specific pass logic.

## Extension Points

- Register feature-owned frame graphs through render graph registry APIs.
- Register feature-owned pass executors through executor registry APIs.
- Register typed frame resources through `RenderFrameResourceBindings::register_resource::<T>()` so submit includes them in `RenderFrameDataRegistry`.
- Schedule feature extract/publish systems to run before `RenderPrepare` so frame resources are finalized before submit.
- Register shader roots/assets and consume shader handles in feature plugins.
- Add feature-owned renderer plugins/examples without editing core render orchestration.
- Add feature-owned render authoring schemas that compile into runtime graph/executor/pipeline registrations.

## Current Architecture

- Shader assets are tracked in `ShaderRegistryResource`.
- Shader files are auto-discovered by scanning configured roots.
- Shader ids are derived from relative file paths.
- Hot reload emits shader registry events and updates revision counters.
- Render submit consumes registered frame resources through `RenderFrameResourceBindings`.
- The active UI path owns text/draw-list production and passes it to render for final submission.
- The SDF example is already using the current plugin-owned render registration model.

Longer-term render architecture notes remain in [ecs-first-proposal.md](./ecs-first-proposal.md), but this README describes the active path rather than the migration history.
