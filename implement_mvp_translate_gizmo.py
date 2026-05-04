#!/usr/bin/env python3
"""
Patch Runenwerk MVP translate gizmo implementation.

What this script changes:
- Adds typed viewport gizmo contracts in domain/editor/editor_viewport.
- Replaces raw string gizmo-axis hit targets with ViewportGizmoAxis.
- Keeps gizmo picking gated to the active translate tool.
- Adds visible translate gizmo overlay render state and shader pass.
- Makes viewport drag deltas axis-aware.
- Updates existing tests to the typed gizmo axis contract.
- Adds/extends focused tests for gizmo frame, picking mapping, and overlay state.

Run from repository root:
    python3 implement_mvp_translate_gizmo.py --dry-run
    python3 implement_mvp_translate_gizmo.py
    cargo test -p editor_viewport
    cargo test -p runenwerk_editor --test scene_authoring_workflow_smoke
    cargo test -p runenwerk_editor
    cargo check -p runenwerk_editor
"""

from __future__ import annotations

import argparse
import difflib
import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Change:
    path: Path
    before: str | None
    after: str
    description: str


class PatchError(RuntimeError):
    pass


def read_text(path: Path) -> str:
    if not path.exists():
        raise PatchError(f"required file is missing: {path}")
    return path.read_text()


def replace_once(text: str, old: str, new: str, path: Path, description: str) -> str:
    if new in text and old in new:
        return text
    count = text.count(old)
    if count == 0:
        if new in text:
            return text
        raise PatchError(f"{path}: expected block not found for {description}")
    if count > 1:
        raise PatchError(f"{path}: expected one block for {description}, found {count}")
    return text.replace(old, new, 1)


def replace_all(text: str, old: str, new: str) -> str:
    return text.replace(old, new)


def regex_replace_once(text: str, pattern: str, replacement: str, path: Path, description: str) -> str:
    new_text, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE | re.DOTALL)
    if count == 0:
        if re.search(re.escape(replacement), text, re.MULTILINE | re.DOTALL):
            return text
        raise PatchError(f"{path}: expected pattern not found for {description}")
    return new_text


def insert_after_once(text: str, anchor: str, insertion: str, path: Path, description: str) -> str:
    if insertion.strip() in text:
        return text
    count = text.count(anchor)
    if count == 0:
        raise PatchError(f"{path}: anchor not found for {description}")
    if count > 1:
        raise PatchError(f"{path}: expected one anchor for {description}, found {count}")
    return text.replace(anchor, anchor + insertion, 1)


def add_change(changes: list[Change], root: Path, relative: str, after: str, description: str) -> None:
    path = root / relative
    before = path.read_text() if path.exists() else None
    if before == after:
        return
    changes.append(Change(path=path, before=before, after=after, description=description))


def unified_diff(change: Change, root: Path) -> str:
    before_lines = [] if change.before is None else change.before.splitlines(keepends=True)
    after_lines = change.after.splitlines(keepends=True)
    rel = change.path.relative_to(root)
    return "".join(
        difflib.unified_diff(
            before_lines,
            after_lines,
            fromfile=f"a/{rel}",
            tofile=f"b/{rel}",
        )
    )


def patch_editor_viewport_domain(root: Path, changes: list[Change]) -> None:
    gizmo_path = "domain/editor/editor_viewport/src/gizmo.rs"
    gizmo = """//! File: domain/editor/editor_viewport/src/gizmo.rs
//! Purpose: Typed viewport gizmo contracts.

use editor_core::{EntityId, RealityVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewportGizmoAxis {
    X,
    Y,
    Z,
}

impl ViewportGizmoAxis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslateGizmoHandle {
    pub axis: ViewportGizmoAxis,
    pub start_world: [f32; 3],
    pub end_world: [f32; 3],
    pub pick_radius_px: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslateGizmoFrame {
    pub source_version: RealityVersion,
    pub entity: EntityId,
    pub origin_world: [f32; 3],
    pub handles: [TranslateGizmoHandle; 3],
}

impl TranslateGizmoFrame {
    pub fn new(
        source_version: RealityVersion,
        entity: EntityId,
        origin_world: [f32; 3],
        axis_length: f32,
        pick_radius_px: f32,
    ) -> Self {
        let [x, y, z] = origin_world;
        Self {
            source_version,
            entity,
            origin_world,
            handles: [
                TranslateGizmoHandle {
                    axis: ViewportGizmoAxis::X,
                    start_world: origin_world,
                    end_world: [x + axis_length, y, z],
                    pick_radius_px,
                },
                TranslateGizmoHandle {
                    axis: ViewportGizmoAxis::Y,
                    start_world: origin_world,
                    end_world: [x, y + axis_length, z],
                    pick_radius_px,
                },
                TranslateGizmoHandle {
                    axis: ViewportGizmoAxis::Z,
                    start_world: origin_world,
                    end_world: [x, y, z + axis_length],
                    pick_radius_px,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_gizmo_frame_builds_three_axis_handles() {
        let frame = TranslateGizmoFrame::new(
            RealityVersion(7),
            EntityId(42),
            [1.0, 2.0, 3.0],
            1.25,
            10.0,
        );

        assert_eq!(frame.entity, EntityId(42));
        assert_eq!(frame.source_version, RealityVersion(7));
        assert_eq!(frame.handles.len(), 3);
        assert_eq!(frame.handles[0].axis, ViewportGizmoAxis::X);
        assert_eq!(frame.handles[0].start_world, [1.0, 2.0, 3.0]);
        assert_eq!(frame.handles[0].end_world, [2.25, 2.0, 3.0]);
        assert_eq!(frame.handles[1].axis, ViewportGizmoAxis::Y);
        assert_eq!(frame.handles[1].end_world, [1.0, 3.25, 3.0]);
        assert_eq!(frame.handles[2].axis, ViewportGizmoAxis::Z);
        assert_eq!(frame.handles[2].end_world, [1.0, 2.0, 4.25]);
    }
}
"""
    add_change(changes, root, gizmo_path, gizmo, "create typed viewport gizmo contracts")

    lib_path = root / "domain/editor/editor_viewport/src/lib.rs"
    lib = read_text(lib_path)
    lib = insert_after_once(
        lib,
        "pub mod expression;\n",
        "pub mod gizmo;\n",
        lib_path,
        "export gizmo module",
    )
    lib = insert_after_once(
        lib,
        "pub use expression::*;\n",
        "pub use gizmo::*;\n",
        lib_path,
        "re-export gizmo contracts",
    )
    add_change(changes, root, "domain/editor/editor_viewport/src/lib.rs", lib, "export typed gizmo contracts")

    hit_path = root / "domain/editor/editor_viewport/src/hit.rs"
    hit = read_text(hit_path)
    hit = replace_once(
        hit,
        "use editor_core::{ComponentTypeId, EntityId};\n",
        "use editor_core::{ComponentTypeId, EntityId};\n\nuse crate::gizmo::ViewportGizmoAxis;\n",
        hit_path,
        "import ViewportGizmoAxis",
    )
    hit = replace_once(
        hit,
        "    GizmoAxis(&'static str),\n",
        "    GizmoAxis(ViewportGizmoAxis),\n",
        hit_path,
        "type ViewportHitTarget::GizmoAxis",
    )
    hit = replace_once(
        hit,
        "    pub fn gizmo_axis(axis: &'static str) -> Self {\n        Self::GizmoAxis(axis)\n    }\n",
        "    pub fn gizmo_axis(axis: ViewportGizmoAxis) -> Self {\n        Self::GizmoAxis(axis)\n    }\n",
        hit_path,
        "type ViewportHitTarget::gizmo_axis",
    )
    hit = replace_once(
        hit,
        "    pub fn gizmo_axis(axis: &'static str, distance: f32) -> Self {\n        Self::new(ViewportHitTarget::gizmo_axis(axis), distance)\n    }\n",
        "    pub fn gizmo_axis(axis: ViewportGizmoAxis, distance: f32) -> Self {\n        Self::new(ViewportHitTarget::gizmo_axis(axis), distance)\n    }\n",
        hit_path,
        "type ViewportHitResult::gizmo_axis",
    )
    if "fn builds_gizmo_axis_hit_result()" not in hit:
        hit = insert_after_once(
            hit,
            """    fn builds_entity_hit_result() {
        let hit = ViewportHitResult::entity(EntityId(1), 2.5);

        assert_eq!(hit.target, ViewportHitTarget::Entity(EntityId(1)));
        assert_eq!(hit.distance, 2.5);
    }

""",
            """    #[test]
    fn builds_gizmo_axis_hit_result() {
        let hit = ViewportHitResult::gizmo_axis(ViewportGizmoAxis::X, 1.25);

        assert_eq!(hit.target, ViewportHitTarget::GizmoAxis(ViewportGizmoAxis::X));
        assert_eq!(hit.distance, 1.25);
    }

""",
            hit_path,
            "add typed gizmo hit constructor test",
        )
    add_change(changes, root, "domain/editor/editor_viewport/src/hit.rs", hit, "type viewport gizmo hit target")


def patch_runtime_expression(root: Path, changes: list[Change]) -> None:
    path = root / "apps/runenwerk_editor/src/runtime/expression/picking.rs"
    text = read_text(path)
    text = replace_once(
        text,
        "use editor_viewport::{ViewportHitResult, ViewportId};\n",
        "use editor_viewport::{ViewportGizmoAxis, ViewportHitResult, ViewportId};\n",
        path,
        "import ViewportGizmoAxis",
    )
    text = replace_once(
        text,
        """        PickingExpressionTarget::GizmoAxis(axis) => {
            ViewportHitResult::gizmo_axis(axis_label(axis), frame.distance)
        }
""",
        """        PickingExpressionTarget::GizmoAxis(axis) => {
            ViewportHitResult::gizmo_axis(map_viewport_axis(axis), frame.distance)
        }
""",
        path,
        "map picking expression axis to typed viewport axis",
    )
    text = regex_replace_once(
        text,
        r"""fn axis_label\(axis: PickingExpressionAxis\) -> &'static str \{
    match axis \{
        PickingExpressionAxis::X => "X",
        PickingExpressionAxis::Y => "Y",
        PickingExpressionAxis::Z => "Z",
    \}
\}""",
        """fn map_viewport_axis(axis: PickingExpressionAxis) -> ViewportGizmoAxis {
    match axis {
        PickingExpressionAxis::X => ViewportGizmoAxis::X,
        PickingExpressionAxis::Y => ViewportGizmoAxis::Y,
        PickingExpressionAxis::Z => ViewportGizmoAxis::Z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picking_expression_gizmo_axis_maps_to_typed_viewport_hit_axis() {
        let frame = PickingExpressionFrame::new(
            RealityVersion(1),
            PickingExpressionTarget::GizmoAxis(PickingExpressionAxis::X),
            3.0,
        );

        let hit = viewport_hit_from_picking_expression(&frame);

        assert_eq!(
            hit.target,
            editor_viewport::ViewportHitTarget::GizmoAxis(ViewportGizmoAxis::X)
        );
        assert_eq!(hit.distance, 3.0);
    }
}
""",
        path,
        "replace raw axis labels with typed viewport axis and test",
    )
    add_change(changes, root, "apps/runenwerk_editor/src/runtime/expression/picking.rs", text, "type picking expression gizmo axis")


def patch_viewport_interaction(root: Path, changes: list[Change]) -> None:
    path = root / "apps/runenwerk_editor/src/editor_features/viewport/interaction.rs"
    text = read_text(path)
    text = replace_once(
        text,
        "use editor_viewport::{ViewportHitResult, ViewportHitTarget};\n",
        "use editor_viewport::{ViewportGizmoAxis, ViewportHitResult, ViewportHitTarget};\n",
        path,
        "import ViewportGizmoAxis",
    )
    text = replace_once(
        text,
        "                    let translate_axis = map_gizmo_axis(axis)?;\n",
        "                    let translate_axis = map_gizmo_axis(axis);\n",
        path,
        "remove unsupported raw-axis error path",
    )
    text = regex_replace_once(
        text,
        r"""fn map_gizmo_axis\(axis: &'static str\) -> Result<TranslateAxis, EditorMutationError> \{
    match axis \{
        "x" \| "X" => Ok\(TranslateAxis::X\),
        "y" \| "Y" => Ok\(TranslateAxis::Y\),
        "z" \| "Z" => Ok\(TranslateAxis::Z\),
        _ => Err\(EditorMutationError::runtime_rejected\(
            "unsupported gizmo axis",
        \)\),
    \}
\}""",
        """fn map_gizmo_axis(axis: ViewportGizmoAxis) -> TranslateAxis {
    match axis {
        ViewportGizmoAxis::X => TranslateAxis::X,
        ViewportGizmoAxis::Y => TranslateAxis::Y,
        ViewportGizmoAxis::Z => TranslateAxis::Z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_gizmo_axis_maps_to_translate_axis() {
        assert_eq!(map_gizmo_axis(ViewportGizmoAxis::X), TranslateAxis::X);
        assert_eq!(map_gizmo_axis(ViewportGizmoAxis::Y), TranslateAxis::Y);
        assert_eq!(map_gizmo_axis(ViewportGizmoAxis::Z), TranslateAxis::Z);
    }
}
""",
        path,
        "replace raw string axis mapper",
    )
    add_change(changes, root, "apps/runenwerk_editor/src/editor_features/viewport/interaction.rs", text, "type viewport interaction gizmo axis")


def patch_render_state(root: Path, changes: list[Change]) -> None:
    path = root / "apps/runenwerk_editor/src/runtime/resources.rs"
    text = read_text(path)
    text = insert_after_once(
        text,
        """pub struct EditorViewportSceneProductUniform {
    pub surface: [f32; 4],
    pub viewport: [f32; 4],
    pub camera_position: [f32; 4],
    pub camera_forward: [f32; 4],
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
    pub object_transform: [f32; 4],
    pub primitive_params_a: [f32; 4],
    pub primitive_params_b: [f32; 4],
    pub primitive_flags: [u32; 4],
}

""",
        """#[derive(Debug, Clone, Copy, GpuUniform)]
pub struct EditorViewportOverlayProductUniform {
    pub surface: [f32; 4],
    pub viewport: [f32; 4],
    pub camera_position: [f32; 4],
    pub camera_forward: [f32; 4],
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
    pub gizmo_origin_axis_length: [f32; 4],
    pub gizmo_state: [u32; 4],
}

""",
        path,
        "add overlay product uniform",
    )
    text = replace_once(
        text,
        """    pub capsule_half_height: f32,
    pub visibility_contradiction_active: bool,
""",
        """    pub capsule_half_height: f32,
    pub has_translate_gizmo: bool,
    pub translate_gizmo_origin: Vec3Value,
    pub translate_gizmo_axis_length: f32,
    pub active_translate_gizmo_axis: u32,
    pub visibility_contradiction_active: bool,
""",
        path,
        "add translate gizmo render state fields",
    )
    text = replace_once(
        text,
        """            capsule_radius: 0.35,
            capsule_half_height: 0.75,
            visibility_contradiction_active: false,
""",
        """            capsule_radius: 0.35,
            capsule_half_height: 0.75,
            has_translate_gizmo: false,
            translate_gizmo_origin: Vec3Value::zero(),
            translate_gizmo_axis_length: 1.25,
            active_translate_gizmo_axis: 0,
            visibility_contradiction_active: false,
""",
        path,
        "initialize translate gizmo render state",
    )
    text = insert_after_once(
        text,
        """    pub fn clear_primitive(&mut self) {
        self.has_primitive = false;
    }

""",
        """    pub fn set_translate_gizmo(
        &mut self,
        origin: Vec3Value,
        axis_length: f32,
        active_axis: u32,
    ) {
        self.has_translate_gizmo = true;
        self.translate_gizmo_origin = origin;
        self.translate_gizmo_axis_length = axis_length.max(0.01);
        self.active_translate_gizmo_axis = active_axis;
    }

    pub fn clear_translate_gizmo(&mut self) {
        self.has_translate_gizmo = false;
        self.active_translate_gizmo_axis = 0;
    }

""",
        path,
        "add translate gizmo render-state mutators",
    )
    text = replace_once(
        text,
        """    pub fn compose_scene_product_uniform(
        &self,
        surface: (u32, u32),
    ) -> EditorViewportSceneProductUniform {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let camera = editor_viewport_camera();

        EditorViewportSceneProductUniform {
            surface: [width, height, 1.0 / width, 1.0 / height],
            viewport: [
                self.viewport_bounds_px.0,
                self.viewport_bounds_px.1,
                self.viewport_bounds_px.2.max(0.0),
                self.viewport_bounds_px.3.max(0.0),
            ],
            camera_position: [
                camera.position.x,
                camera.position.y,
                camera.position.z,
                editor_viewport_camera_fov_y_radians(),
            ],
            camera_forward: [camera.forward.x, camera.forward.y, camera.forward.z, 0.0],
            camera_right: [camera.right.x, camera.right.y, camera.right.z, 0.0],
            camera_up: [camera.up.x, camera.up.y, camera.up.z, 0.0],
            object_transform: [
                self.primitive_translation.x,
                self.primitive_translation.y,
                self.primitive_translation.z,
                0.0,
            ],
            primitive_params_a: [
                self.box_half_extents.x.max(0.05),
                self.box_half_extents.y.max(0.05),
                self.box_half_extents.z.max(0.05),
                self.sphere_radius.max(0.05),
            ],
            primitive_params_b: [
                self.capsule_radius.max(0.05),
                self.capsule_half_height.max(0.05),
                0.0,
                0.0,
            ],
            primitive_flags: [
                self.primitive_kind.as_u32(),
                if self.has_primitive { 1 } else { 0 },
                self.debug_stage.as_u32(),
                if self.root_background_opaque { 1 } else { 0 },
            ],
        }
    }
""",
        """    pub fn compose_scene_product_uniform(
        &self,
        surface: (u32, u32),
    ) -> EditorViewportSceneProductUniform {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let camera = editor_viewport_camera();

        EditorViewportSceneProductUniform {
            surface: [width, height, 1.0 / width, 1.0 / height],
            viewport: [
                self.viewport_bounds_px.0,
                self.viewport_bounds_px.1,
                self.viewport_bounds_px.2.max(0.0),
                self.viewport_bounds_px.3.max(0.0),
            ],
            camera_position: [
                camera.position.x,
                camera.position.y,
                camera.position.z,
                editor_viewport_camera_fov_y_radians(),
            ],
            camera_forward: [camera.forward.x, camera.forward.y, camera.forward.z, 0.0],
            camera_right: [camera.right.x, camera.right.y, camera.right.z, 0.0],
            camera_up: [camera.up.x, camera.up.y, camera.up.z, 0.0],
            object_transform: [
                self.primitive_translation.x,
                self.primitive_translation.y,
                self.primitive_translation.z,
                0.0,
            ],
            primitive_params_a: [
                self.box_half_extents.x.max(0.05),
                self.box_half_extents.y.max(0.05),
                self.box_half_extents.z.max(0.05),
                self.sphere_radius.max(0.05),
            ],
            primitive_params_b: [
                self.capsule_radius.max(0.05),
                self.capsule_half_height.max(0.05),
                0.0,
                0.0,
            ],
            primitive_flags: [
                self.primitive_kind.as_u32(),
                if self.has_primitive { 1 } else { 0 },
                self.debug_stage.as_u32(),
                if self.root_background_opaque { 1 } else { 0 },
            ],
        }
    }

    pub fn compose_overlay_product_uniform(
        &self,
        surface: (u32, u32),
    ) -> EditorViewportOverlayProductUniform {
        let width = surface.0.max(1) as f32;
        let height = surface.1.max(1) as f32;
        let camera = editor_viewport_camera();

        EditorViewportOverlayProductUniform {
            surface: [width, height, 1.0 / width, 1.0 / height],
            viewport: [
                self.viewport_bounds_px.0,
                self.viewport_bounds_px.1,
                self.viewport_bounds_px.2.max(0.0),
                self.viewport_bounds_px.3.max(0.0),
            ],
            camera_position: [
                camera.position.x,
                camera.position.y,
                camera.position.z,
                editor_viewport_camera_fov_y_radians(),
            ],
            camera_forward: [camera.forward.x, camera.forward.y, camera.forward.z, 0.0],
            camera_right: [camera.right.x, camera.right.y, camera.right.z, 0.0],
            camera_up: [camera.up.x, camera.up.y, camera.up.z, 0.0],
            gizmo_origin_axis_length: [
                self.translate_gizmo_origin.x,
                self.translate_gizmo_origin.y,
                self.translate_gizmo_origin.z,
                self.translate_gizmo_axis_length.max(0.01),
            ],
            gizmo_state: [
                if self.has_translate_gizmo { 1 } else { 0 },
                self.active_translate_gizmo_axis,
                0,
                0,
            ],
        }
    }
""",
        path,
        "add overlay uniform projection",
    )

    add_change(changes, root, "apps/runenwerk_editor/src/runtime/resources.rs", text, "add translate gizmo overlay render state")


def patch_frame_submit(root: Path, changes: list[Change]) -> None:
    path = root / "apps/runenwerk_editor/src/runtime/systems/frame_submit.rs"
    text = read_text(path)
    text = replace_once(
        text,
        "use crate::editor_runtime::EditorPrimitive;\n",
        "use crate::editor_runtime::EditorPrimitive;\nuse crate::editor_tools_state::TranslateAxis;\n",
        path,
        "import TranslateAxis",
    )
    text = replace_once(
        text,
        """use crate::runtime::viewport::{
    MountedSurfaceRegistryResource, ToolSurfaceRuntimeBindingRegistryResource,
""",
        """use crate::runtime::viewport::{
    MountedSurfaceRegistryResource, ToolSurfaceRuntimeBindingRegistryResource,
""",
        path,
        "stable viewport import anchor",
    )
    text = replace_once(
        text,
        """const VIEWPORT_BRANCH_TRACE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_BRANCH_TRACE";

""",
        """const VIEWPORT_BRANCH_TRACE_ENV: &str = "RUNENWERK_EDITOR_VIEWPORT_BRANCH_TRACE";
const TRANSLATE_GIZMO_AXIS_LENGTH: f32 = 1.25;

""",
        path,
        "add translate gizmo axis length constant",
    )
    text = replace_once(
        text,
        """    let runtime = app.runtime();
    if let Some((transform, primitive)) = selected_or_first_editor_primitive(runtime) {
        render_state.set_primitive(transform.translation, primitive);
    } else {
        render_state.clear_primitive();
    }

    bounds_changed
}
""",
        """    let runtime = app.runtime();
    if let Some((transform, primitive)) = selected_or_first_editor_primitive(runtime) {
        render_state.set_primitive(transform.translation, primitive);
    } else {
        render_state.clear_primitive();
    }

    if let Some((transform, _primitive)) = selected_entity_primitive(runtime)
        && runtime.session().active_tool() == Some(crate::shell::TRANSLATE_TOOL_ID)
    {
        render_state.set_translate_gizmo(
            transform.translation,
            TRANSLATE_GIZMO_AXIS_LENGTH,
            translate_axis_to_overlay_id(app.viewport_tool_state().active_translate_axis),
        );
    } else {
        render_state.clear_translate_gizmo();
    }

    bounds_changed
}
""",
        path,
        "populate translate gizmo render state only for active translate tool",
    )
    text = insert_after_once(
        text,
        """fn selected_or_first_editor_primitive(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
) -> Option<(LocalTransform, EditorPrimitive)> {
    if let Some(selected) = runtime.selected_entity()
        && let Some(result) = entity_primitive(runtime, selected)
    {
        return Some(result);
    }

    runtime
        .document()
        .entity_ids()
        .find_map(|entity| entity_primitive(runtime, entity))
}

""",
        """fn selected_entity_primitive(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
) -> Option<(LocalTransform, EditorPrimitive)> {
    let selected = runtime.selected_entity()?;
    entity_primitive(runtime, selected)
}

fn translate_axis_to_overlay_id(axis: Option<TranslateAxis>) -> u32 {
    match axis {
        Some(TranslateAxis::X) => 1,
        Some(TranslateAxis::Y) => 2,
        Some(TranslateAxis::Z) => 3,
        None => 0,
    }
}

""",
        path,
        "add selected-entity gizmo helpers",
    )
    add_change(changes, root, "apps/runenwerk_editor/src/runtime/systems/frame_submit.rs", text, "derive visible translate gizmo state from active translate tool")


def patch_render_flow_and_shader(root: Path, changes: list[Change]) -> None:
    path = root / "apps/runenwerk_editor/src/runtime/app.rs"
    text = read_text(path)
    text = replace_once(
        text,
        'pub const EDITOR_VIEWPORT_SCENE_PRODUCT_SHADER_ID: &str = "editor_viewport_scene_product";\n',
        'pub const EDITOR_VIEWPORT_SCENE_PRODUCT_SHADER_ID: &str = "editor_viewport_scene_product";\npub const EDITOR_VIEWPORT_OVERLAY_PRODUCT_SHADER_ID: &str = "editor_viewport_overlay_product";\n',
        path,
        "add overlay shader id",
    )
    text = replace_once(
        text,
        """        .fullscreen_pass(EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID)
        .depends_on(EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID)
        .write_color_target(VIEWPORT_RESOURCE_OVERLAY)
        .finish()
""",
        """        .fullscreen_pass(EDITOR_VIEWPORT_OVERLAY_PRODUCT_PASS_ID)
        .depends_on(EDITOR_VIEWPORT_PICKING_PRODUCT_PASS_ID)
        .shader_asset(EDITOR_VIEWPORT_OVERLAY_PRODUCT_SHADER_ID)
        .uniform_from_state_with_surface(EditorViewportRenderState::compose_overlay_product_uniform)
        .write_color_target(VIEWPORT_RESOURCE_OVERLAY)
        .finish()
""",
        path,
        "attach overlay shader and uniform to overlay pass",
    )
    add_change(changes, root, "apps/runenwerk_editor/src/runtime/app.rs", text, "enable shader-backed translate gizmo overlay pass")

    shader_path = "assets/shaders/editor_viewport_overlay_product.wgsl"
    shader = """struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
};

struct OverlayUniform {
    surface: vec4<f32>,
    viewport: vec4<f32>,
    camera_position: vec4<f32>,
    camera_forward: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
    gizmo_origin_axis_length: vec4<f32>,
    gizmo_state: vec4<u32>,
};

@group(0) @binding(0)
var<uniform> overlay: OverlayUniform;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );

    var out: VsOut;
    out.clip_position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
    return out;
}

fn inside_viewport(pixel: vec2<f32>) -> bool {
    let v = overlay.viewport;
    return pixel.x >= v.x
        && pixel.y >= v.y
        && pixel.x <= v.x + v.z
        && pixel.y <= v.y + v.w
        && v.z > 0.0
        && v.w > 0.0;
}

fn project_world_to_screen(world: vec3<f32>) -> vec2<f32> {
    let relative = world - overlay.camera_position.xyz;
    let x_cam = dot(relative, overlay.camera_right.xyz);
    let y_cam = dot(relative, overlay.camera_up.xyz);
    let z_cam = max(dot(relative, overlay.camera_forward.xyz), 0.001);
    let tan_half_fov = max(tan(overlay.camera_position.w * 0.5), 0.0001);
    let aspect = max(overlay.viewport.z / max(overlay.viewport.w, 1.0), 0.01);

    let ndc_x = x_cam / (z_cam * tan_half_fov * aspect);
    let ndc_y = y_cam / (z_cam * tan_half_fov);

    return vec2<f32>(
        overlay.viewport.x + (ndc_x * 0.5 + 0.5) * overlay.viewport.z,
        overlay.viewport.y + (0.5 - ndc_y * 0.5) * overlay.viewport.w,
    );
}

fn point_segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let segment = end - start;
    let length_sq = max(dot(segment, segment), 0.0001);
    let t = clamp(dot(point - start, segment) / length_sq, 0.0, 1.0);
    let closest = start + segment * t;
    return distance(point, closest);
}

fn axis_alpha(pixel: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let line_distance = point_segment_distance(pixel, start, end);
    let handle_distance = distance(pixel, end);
    let line_alpha = 1.0 - smoothstep(2.0, 5.5, line_distance);
    let handle_alpha = 1.0 - smoothstep(5.0, 11.0, handle_distance);
    return max(line_alpha, handle_alpha);
}

fn blend_axis(
    current_color: vec3<f32>,
    current_alpha: f32,
    axis_color: vec3<f32>,
    axis_alpha_value: f32,
    axis_id: u32,
) -> vec4<f32> {
    let active_id = overlay.gizmo_state.y;
    let boost = select(1.0, 1.35, active_id == axis_id);
    let next_alpha = max(current_alpha, clamp(axis_alpha_value * boost, 0.0, 1.0));
    let next_color = mix(current_color, axis_color * boost, axis_alpha_value);
    return vec4<f32>(next_color, next_alpha);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let pixel = position.xy;

    if !inside_viewport(pixel) || overlay.gizmo_state.x == 0u {
        return vec4<f32>(0.0);
    }

    let origin = overlay.gizmo_origin_axis_length.xyz;
    let axis_length = overlay.gizmo_origin_axis_length.w;
    let start = project_world_to_screen(origin);

    let x_end = project_world_to_screen(origin + vec3<f32>(axis_length, 0.0, 0.0));
    let y_end = project_world_to_screen(origin + vec3<f32>(0.0, axis_length, 0.0));
    let z_end = project_world_to_screen(origin + vec3<f32>(0.0, 0.0, axis_length));

    var color = vec3<f32>(0.0);
    var alpha = 0.0;

    let x = blend_axis(color, alpha, vec3<f32>(1.0, 0.20, 0.18), axis_alpha(pixel, start, x_end), 1u);
    color = x.xyz;
    alpha = x.w;

    let y = blend_axis(color, alpha, vec3<f32>(0.25, 1.0, 0.25), axis_alpha(pixel, start, y_end), 2u);
    color = y.xyz;
    alpha = y.w;

    let z = blend_axis(color, alpha, vec3<f32>(0.30, 0.55, 1.0), axis_alpha(pixel, start, z_end), 3u);
    color = z.xyz;
    alpha = z.w;

    return vec4<f32>(color * alpha, alpha);
}
"""
    add_change(changes, root, shader_path, shader, "create translate gizmo overlay shader")


def patch_input_bridge(root: Path, changes: list[Change]) -> None:
    path = root / "apps/runenwerk_editor/src/runtime/systems/input_bridge.rs"
    text = read_text(path)
    text = replace_once(
        text,
        "use crate::editor_features::viewport::ViewportInteractionCommand;\n",
        "use crate::editor_features::viewport::ViewportInteractionCommand;\nuse crate::editor_tools_state::TranslateAxis;\n",
        path,
        "import TranslateAxis for axis-aware drag",
    )
    text = replace_once(
        text,
        """    {
        let amount = position.x - previous.x;
        if amount != 0.0
            && let Err(error) = host.app.dispatch_viewport_interaction_command(
                ViewportInteractionCommand::PointerDragAxis { amount },
            )
        {
            eprintln!("viewport axis drag failed: {error}");
        }
    }
""",
        """    {
        let pointer_delta = position - previous;
        let amount = viewport_axis_drag_amount(
            host.app.viewport_interaction_state().active_axis(),
            pointer_delta,
        );
        if amount != 0.0
            && let Err(error) = host.app.dispatch_viewport_interaction_command(
                ViewportInteractionCommand::PointerDragAxis { amount },
            )
        {
            eprintln!("viewport axis drag failed: {error}");
        }
    }
""",
        path,
        "make pointer drag delta axis-aware",
    )
    text = insert_after_once(
        text,
        """fn dispatch_shortcuts(
    input: &engine::plugins::InputState,
""",
        """fn viewport_axis_drag_amount(axis: Option<TranslateAxis>, pointer_delta: UiVector) -> f32 {
    match axis {
        Some(TranslateAxis::X) => pointer_delta.x,
        Some(TranslateAxis::Y) => -pointer_delta.y,
        Some(TranslateAxis::Z) => pointer_delta.x,
        None => 0.0,
    }
}

""",
        path,
        "add isolated axis drag scalar helper",
    )
    add_change(changes, root, "apps/runenwerk_editor/src/runtime/systems/input_bridge.rs", text, "make viewport translate drag axis-aware")


def patch_tests(root: Path, changes: list[Change]) -> None:
    test_paths = [
        "apps/runenwerk_editor/tests/scene_authoring_workflow_smoke.rs",
        "apps/runenwerk_editor/src/editor_runtime/tests/viewport.rs",
    ]
    for relative in test_paths:
        path = root / relative
        text = read_text(path)
        text = text.replace(
            "use editor_viewport::ViewportHitResult;",
            "use editor_viewport::{ViewportGizmoAxis, ViewportHitResult};",
        )
        text = text.replace(
            "use editor_viewport::{ViewportHitResult, ViewportHitTarget};",
            "use editor_viewport::{ViewportGizmoAxis, ViewportHitResult, ViewportHitTarget};",
        )
        text = text.replace('ViewportHitResult::gizmo_axis("X",', "ViewportHitResult::gizmo_axis(ViewportGizmoAxis::X,")
        text = text.replace('ViewportHitResult::gizmo_axis("Y",', "ViewportHitResult::gizmo_axis(ViewportGizmoAxis::Y,")
        text = text.replace('ViewportHitResult::gizmo_axis("Z",', "ViewportHitResult::gizmo_axis(ViewportGizmoAxis::Z,")
        add_change(changes, root, relative, text, "update tests to typed viewport gizmo axis")

    # Add focused render-state tests to existing startup smoke test.
    path = root / "apps/runenwerk_editor/tests/startup_render_smoke.rs"
    text = read_text(path)
    if "startup_render_smoke_keeps_translate_gizmo_hidden_until_translate_tool_is_active" not in text:
        addition = """

#[test]
fn startup_render_smoke_keeps_translate_gizmo_hidden_until_translate_tool_is_active() {
    let app = runenwerk_editor::runtime::build_headless_app()
        .run_for_frames(2)
        .expect("headless editor app should run");

    let viewport_state = app
        .world()
        .resource::<EditorViewportRenderState>()
        .expect("viewport render state should exist");

    assert!(
        !viewport_state.has_translate_gizmo,
        "translate gizmo should be hidden until the translate tool is active"
    );
}
"""
        text = text + addition
    add_change(changes, root, "apps/runenwerk_editor/tests/startup_render_smoke.rs", text, "add translate gizmo hidden-by-default smoke assertion")


def validate_no_raw_gizmo_axis_strings(root: Path, staged: dict[Path, str]) -> None:
    candidates = [
        root / "apps/runenwerk_editor/tests/scene_authoring_workflow_smoke.rs",
        root / "apps/runenwerk_editor/src/editor_runtime/tests/viewport.rs",
        root / "apps/runenwerk_editor/src/runtime/expression/picking.rs",
        root / "domain/editor/editor_viewport/src/hit.rs",
    ]
    for path in candidates:
        text = staged.get(path, path.read_text() if path.exists() else "")
        if re.search(r'gizmo_axis\("[XYZxyz]"', text):
            raise PatchError(f"{path}: raw string gizmo_axis call remains")


def build_changes(root: Path) -> list[Change]:
    if not (root / "Cargo.toml").exists():
        raise PatchError("run this script from the repository root containing Cargo.toml")

    changes: list[Change] = []
    patch_editor_viewport_domain(root, changes)
    patch_runtime_expression(root, changes)
    patch_viewport_interaction(root, changes)
    patch_render_state(root, changes)
    patch_frame_submit(root, changes)
    patch_render_flow_and_shader(root, changes)
    patch_input_bridge(root, changes)
    patch_tests(root, changes)

    staged = {change.path: change.after for change in changes}
    validate_no_raw_gizmo_axis_strings(root, staged)
    return changes


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", help="repository root, default: current directory")
    parser.add_argument("--dry-run", action="store_true", help="print diffs without writing")
    parser.add_argument("--check", action="store_true", help="exit non-zero if changes would be made")
    args = parser.parse_args()

    root = Path(args.root).resolve()

    try:
        changes = build_changes(root)
    except PatchError as error:
        print(f"[error] {error}", file=sys.stderr)
        return 2

    if not changes:
        print("[ok] MVP translate gizmo implementation patch is already applied.")
        return 0

    for change in changes:
        print(unified_diff(change, root), end="")
        print(f"[changed] {change.path.relative_to(root)}: {change.description}")

    if args.check:
        print(f"[check] {len(changes)} file(s) need changes.")
        return 1

    if args.dry_run:
        print(f"[dry-run] {len(changes)} file(s) would be changed.")
        return 0

    for change in changes:
        change.path.parent.mkdir(parents=True, exist_ok=True)
        change.path.write_text(change.after)

    print(f"[ok] applied {len(changes)} file change(s).")
    print()
    print("Run validation:")
    print("  cargo test -p editor_viewport")
    print("  cargo test -p runenwerk_editor --test scene_authoring_workflow_smoke")
    print("  cargo test -p runenwerk_editor")
    print("  cargo check -p runenwerk_editor")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
