# UI Plugin

## Purpose

`ui` owns overlay UI domain types, template/runtime helpers, and text rendering support types.

Render core should treat UI as feature data, not hardcoded render behavior.

## Usage

- Primary surface: `engine::plugins::ui::domain::*`
- Current integration path:
  - `ScenePlugin` owns overlay UI runtime/update flow.
  - `RenderPlugin` consumes published UI draw data during render submission.

## Ownership Boundaries

- Owns UI domain types, template I/O, text atlas/renderer support, and draw list data structures.
- Owns UI-specific shader selection configuration (`UiRenderShaderConfig`) as data.
- Does not own render graph orchestration/execution or scene lifecycle scheduling.

## Extension Points

- Extend template/runtime behavior under `engine/src/plugins/ui/domain/template.rs`.
- Extend text rendering support under `engine/src/plugins/ui/domain/text.rs`.
- Extend UI domain data consumed by scene/render systems.

## Additional Details

## ECS Resources

- `UiRenderShaderConfig`
  - Controls which shader asset id is used for UI rectangle pass rendering.
  - Default id: `ui_rect`.

## Shader Hot Reload Behavior

- Shader source loading + watching is handled by `ShaderRegistryResource` (render domain).
- Shader registry auto-discovers shaders from configured asset roots (default includes `assets/shaders`).
- UI plugin only declares which asset id it wants via `UiRenderShaderConfig`; it does not hardcode file-path registration in render core.

This keeps UI shader selection data-driven and ECS-owned while preserving global shader hot reload behavior.
