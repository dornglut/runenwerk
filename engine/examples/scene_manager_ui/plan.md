# Scene Manager UI Demo Plan

## Evaluation (2026-02-23)
- Confirmed bug: `scene_manager_ui` still renders shared overlay console/log diagnostics from the default UI pipeline.
- Root cause: template flow reused `ConsoleUiState` defaults while `ui_build_batches_system` always drew log and diagnostics batches.

## Demo Cleanup Goals
1. Center scene content in the viewport.
2. Remove world logs, diagnostics HUD, and console-input noise for this demo.
3. Keep template scene transitions and pause flow behavior.

## Implementation Plan
- [x] Add a UI presentation profile for centered scene-template demos.
- [x] Enable that profile from `template_flow` setup and tune layout metrics.
- [x] Center panel/title/button layout for demo mode.
- [x] Suppress logs and diagnostics batches in demo mode.
- [x] Clear inherited log buffers when applying scene templates.
- [x] Restyle example template assets for a cleaner demo.
- [x] Validate compile path with `cargo check -p engine --example scene_manager_ui`.

## Acceptance Criteria
- `cargo run -p engine --example scene_manager_ui` shows centered scene content.
- No world logs window is rendered.
- No world diagnostics panel is rendered.
- No default console input prompt is rendered.
