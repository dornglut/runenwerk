# Scene Plugin

## Purpose

Coordinates world/overlay scene lifecycle, transitions, and message flow.

## Usage

- Plugin: `ScenePlugin`
- Core nodes:
  - `scene_transition`
  - `world_scene_update`
  - `scene_overlay_format_messages`
  - `scene_overlay_apply_messages`

The plugin manages scene stack commands and applies transition side effects.

## Ownership Boundaries

- Owns scene transition orchestration and scene lifecycle event flow.
- Owns world scene runtime updates and overlay/world interaction state.
- Owns applying compiled scene/template authoring outputs to runtime state.
- Does not own render graph execution or input device event collection.

## Extension Points

- Register new scene labels/aliases and transition commands.
- Add new world-to-overlay message types and formatting paths.
- Extend template scene flow integration hooks.
- Add scene authoring schemas/compilers under scene-owned authoring modules.
- Integrate dependency-aware scene/template hot reload without exposing parsing logic to runtime orchestration.
