# UI Plugin

## Purpose

`ui` owns overlay UI state, UI batch extraction, and text rendering domain types.

Render core should treat UI as a feature plugin consumer, not as hardcoded render behavior.

## Usage

- Plugins:
  - `UiInputPlugin`
  - `UiRenderPlugin`
- Core UI pipeline nodes:
  - `overlay_ui_hot_reload`
  - `overlay_ui_input`
  - `overlay_ui_editor`
  - `overlay_ui_layout`
  - `overlay_ui_build_batches`
  - `overlay_ui_render_extract`

## Ownership Boundaries

- Owns UI interaction/input-to-UI mapping, layout, batch extraction, and text domain logic.
- Owns UI-specific render shader selection configuration (`UiRenderShaderConfig`).
- Does not own render graph orchestration/execution or global shader registry behavior.

## Extension Points

- Extend UI batch command generation and extraction systems.
- Extend UI editor/template integration behavior.
- Override `UiRenderShaderConfig` to bind alternative UI shader assets.

## Additional Details

## ECS Resources

- `UiRenderShaderConfig`
  - Resource stored in `overlay_runtime.world`.
  - Controls which shader asset id is used for UI rectangle pass rendering.
  - Default id: `ui_rect`.

`UiRenderPlugin::setup` ensures this resource exists so feature/plugin code can override it before frame submit.

## Shader Hot Reload Behavior

- Shader source loading + watching is handled by `ShaderRegistryResource` (render domain).
- Shader registry auto-discovers shaders from configured asset roots (default includes `assets/shaders`).
- UI plugin only declares which asset id it wants via `UiRenderShaderConfig`; it does not hardcode file-path registration in render core.

This keeps UI shader selection data-driven and ECS-owned while preserving global shader hot reload behavior.
