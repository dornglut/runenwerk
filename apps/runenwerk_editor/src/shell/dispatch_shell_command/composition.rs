use editor_core::EditorMutationError;
use editor_shell::{
    DockSplitSide, EditorCompositionRejection, EditorDockingDestination, EditorDockingIntent,
    EditorStructuralEditPlan, PanelInstanceId, TabDropDestination, TabStackId,
    ToolSurfaceStableKey, WorkspaceSplitAxis, plan_editor_activate_unit,
    plan_editor_close_other_units, plan_editor_close_stack, plan_editor_close_unit,
    plan_editor_create_unit, plan_editor_duplicate_stack, plan_editor_reset_stack,
    plan_editor_set_stack_lock, plan_editor_split_with_new_unit,
};
use ui_adaptive_composition::DockZone;
use ui_composition::{CompositionPolicies, RegionId, SplitFraction, StateRevision};

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{EditorCompositionPolicy, RunenwerkEditorShellState};

pub(super) fn undo_composition_layout(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
) -> Result<(), EditorMutationError> {
    let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
        EditorMutationError::runtime_rejected("missing shell state for composition undo")
    })?;
    let policy = EditorCompositionPolicy;
    let policies = CompositionPolicies {
        lifecycle: &policy,
        capability: &policy,
        target: &policy,
    };
    shell_state
        .undo_structural_composition(policies)
        .map_err(|rejection| record_composition_rejection(app, rejection))?;
    Ok(())
}

pub(super) fn redo_composition_layout(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
) -> Result<(), EditorMutationError> {
    let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
        EditorMutationError::runtime_rejected("missing shell state for composition redo")
    })?;
    let policy = EditorCompositionPolicy;
    let policies = CompositionPolicies {
        lifecycle: &policy,
        capability: &policy,
        target: &policy,
    };
    shell_state
        .redo_structural_composition(policies)
        .map_err(|rejection| record_composition_rejection(app, rejection))?;
    Ok(())
}

pub(super) fn set_tab_stack_active_panel(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    panel_instance_id: PanelInstanceId,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "activate tab",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let unit = composition_unit_for_panel(shell_state, panel_instance_id)?;
    let plan = plan_editor_activate_unit(
        shell_state.composition_runtime(),
        stack,
        unit,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn commit_tab_drop(
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    panel_instance_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    destination: TabDropDestination,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
        EditorMutationError::runtime_rejected("missing shell state for composition docking")
    })?;
    if !shell_state.is_projection_epoch_current(projection_epoch) {
        return Err(EditorMutationError::runtime_rejected(
            "stale editor composition docking projection",
        ));
    }
    let unit = shell_state
        .mounted_unit_id_for_panel(panel_instance_id)
        .ok_or_else(|| {
            EditorMutationError::runtime_rejected("docked panel has no mounted composition unit")
        })?;
    let source_region = shell_state
        .region_id_for_tab_stack(source_tab_stack_id)
        .ok_or_else(|| {
            EditorMutationError::runtime_rejected(
                "docking source tab stack has no composition region",
            )
        })?;
    let actual_source = shell_state
        .composition_runtime()
        .composition()
        .definition()
        .regions()
        .iter()
        .find(|region| region.kind.mounted_units().contains(&unit))
        .map(|region| region.id);
    if actual_source != Some(source_region) {
        return Err(EditorMutationError::runtime_rejected(
            "docking source no longer owns the mounted unit",
        ));
    }
    let destination = docking_destination(shell_state, destination)?;
    shell_state.queue_docking_intent(EditorDockingIntent {
        source_revision: shell_state.composition_runtime().composition().revision(),
        unit,
        destination,
    });
    Ok(())
}

pub(super) fn commit_composition_dock(
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    intent: EditorDockingIntent,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = shell_state.as_deref_mut().ok_or_else(|| {
        EditorMutationError::runtime_rejected("missing shell state for composition docking")
    })?;
    if !shell_state.is_projection_epoch_current(projection_epoch) {
        return Err(EditorMutationError::runtime_rejected(
            "stale editor composition docking projection",
        ));
    }
    if intent.source_revision != shell_state.composition_runtime().composition().revision() {
        return Err(EditorMutationError::runtime_rejected(
            "stale editor composition docking revision",
        ));
    }
    shell_state.queue_docking_intent(intent);
    Ok(())
}

pub(super) fn resize_composition_split(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    split: RegionId,
    fraction: SplitFraction,
    expected_revision: StateRevision,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "resize composition split",
    )?;
    if shell_state.composition_runtime().composition().revision() != expected_revision {
        return Err(EditorMutationError::runtime_rejected(
            "stale editor composition resize revision",
        ));
    }
    let plan = editor_shell::plan_editor_resize_split(
        shell_state.composition_runtime(),
        split,
        fraction,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn create_panel_tab_stable_key(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    stable_surface_key: ToolSurfaceStableKey,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "create tab",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let plan = plan_editor_create_unit(
        shell_state.composition_runtime(),
        stack,
        app.workbench_host().tool_surface_registry(),
        stable_surface_key,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn close_panel_tab(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    panel_instance_id: PanelInstanceId,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state =
        require_composition_shell_state(shell_state.as_deref_mut(), projection_epoch, "close tab")?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let unit = composition_unit_for_panel(shell_state, panel_instance_id)?;
    require_unit_source(shell_state, unit, stack)?;
    let plan = plan_editor_close_unit(
        shell_state.composition_runtime(),
        unit,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn close_other_panel_tabs(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    keep_panel_instance_id: PanelInstanceId,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "close other tabs",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let keep = composition_unit_for_panel(shell_state, keep_panel_instance_id)?;
    let plan = plan_editor_close_other_units(
        shell_state.composition_runtime(),
        stack,
        keep,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn split_tab_stack_area_stable_key(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    axis: WorkspaceSplitAxis,
    stable_surface_key: ToolSurfaceStableKey,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "split area",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let plan = plan_editor_split_with_new_unit(
        shell_state.composition_runtime(),
        stack,
        axis,
        app.workbench_host().tool_surface_registry(),
        stable_surface_key,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn duplicate_tab_stack_area(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "duplicate area",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let plan = plan_editor_duplicate_stack(
        shell_state.composition_runtime(),
        stack,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn close_tab_stack_area(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "close area",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let plan = plan_editor_close_stack(
        shell_state.composition_runtime(),
        stack,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn reset_tab_stack_area_stable_key(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    stable_surface_key: ToolSurfaceStableKey,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "reset area",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let plan = plan_editor_reset_stack(
        shell_state.composition_runtime(),
        stack,
        app.workbench_host().tool_surface_registry(),
        stable_surface_key,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

pub(super) fn lock_tab_stack_area_stable_key(
    app: &mut RunenwerkEditorApp,
    mut shell_state: Option<&mut RunenwerkEditorShellState>,
    tab_stack_id: TabStackId,
    locked_stable_surface_key: Option<ToolSurfaceStableKey>,
    projection_epoch: u64,
) -> Result<(), EditorMutationError> {
    let shell_state = require_composition_shell_state(
        shell_state.as_deref_mut(),
        projection_epoch,
        "change area lock",
    )?;
    let stack = composition_region_for_stack(shell_state, tab_stack_id)?;
    let plan = plan_editor_set_stack_lock(
        shell_state.composition_runtime(),
        stack,
        locked_stable_surface_key,
        shell_state.composition_identity_allocator(),
    )
    .map_err(|rejection| record_composition_rejection(app, rejection))?;
    apply_editor_structural_plan(app, shell_state, plan)?;
    Ok(())
}

fn require_composition_shell_state<'a>(
    shell_state: Option<&'a mut RunenwerkEditorShellState>,
    projection_epoch: u64,
    operation: &'static str,
) -> Result<&'a mut RunenwerkEditorShellState, EditorMutationError> {
    let shell_state = shell_state.ok_or_else(|| {
        EditorMutationError::runtime_rejected("missing shell state for composition edit")
    })?;
    if !shell_state.is_projection_epoch_current(projection_epoch) {
        return Err(EditorMutationError::runtime_rejected(match operation {
            "activate tab" => "stale composition projection for tab activation",
            "create tab" => "stale composition projection for tab creation",
            "close tab" => "stale composition projection for tab close",
            "close other tabs" => "stale composition projection for close-other-tabs",
            "split area" => "stale composition projection for area split",
            "duplicate area" => "stale composition projection for area duplicate",
            "close area" => "stale composition projection for area close",
            "reset area" => "stale composition projection for area reset",
            "change area lock" => "stale composition projection for area lock",
            _ => "stale composition projection for structural edit",
        }));
    }
    Ok(shell_state)
}

fn composition_region_for_stack(
    shell_state: &RunenwerkEditorShellState,
    tab_stack_id: editor_shell::TabStackId,
) -> Result<ui_composition::RegionId, EditorMutationError> {
    shell_state
        .region_id_for_tab_stack(tab_stack_id)
        .ok_or_else(|| {
            EditorMutationError::runtime_rejected(
                "tab stack has no region in the current composition projection",
            )
        })
}

fn composition_unit_for_panel(
    shell_state: &RunenwerkEditorShellState,
    panel_instance_id: editor_shell::PanelInstanceId,
) -> Result<ui_composition::MountedUnitId, EditorMutationError> {
    shell_state
        .mounted_unit_id_for_panel(panel_instance_id)
        .ok_or_else(|| {
            EditorMutationError::runtime_rejected(
                "panel has no mounted unit in the current composition projection",
            )
        })
}

fn require_unit_source(
    shell_state: &RunenwerkEditorShellState,
    unit: ui_composition::MountedUnitId,
    expected: ui_composition::RegionId,
) -> Result<(), EditorMutationError> {
    let actual = shell_state
        .composition_runtime()
        .composition()
        .definition()
        .regions()
        .iter()
        .find(|region| region.kind.mounted_units().contains(&unit))
        .map(|region| region.id);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(EditorMutationError::runtime_rejected(
            "panel no longer belongs to the selected composition stack",
        ))
    }
}

fn apply_editor_structural_plan(
    app: &mut RunenwerkEditorApp,
    shell_state: &mut RunenwerkEditorShellState,
    plan: EditorStructuralEditPlan,
) -> Result<(), EditorMutationError> {
    let policy = EditorCompositionPolicy;
    let policies = CompositionPolicies {
        lifecycle: &policy,
        capability: &policy,
        target: &policy,
    };
    shell_state
        .apply_structural_edit_plan(plan, policies)
        .map_err(|rejection| record_composition_rejection(app, rejection))?;
    shell_state.close_tab_stack_popup_menu();
    Ok(())
}

fn record_composition_rejection(
    app: &mut RunenwerkEditorApp,
    rejection: EditorCompositionRejection,
) -> EditorMutationError {
    for diagnostic in rejection.diagnostics() {
        app.append_console_line(format!(
            "[{}] {}",
            diagnostic.code().as_str(),
            diagnostic.message()
        ));
    }
    EditorMutationError::runtime_rejected("editor composition structural edit rejected")
}

fn docking_destination(
    shell_state: &RunenwerkEditorShellState,
    destination: TabDropDestination,
) -> Result<EditorDockingDestination, EditorMutationError> {
    let region_destination = |target_region, ordinal, zone| {
        Ok(EditorDockingDestination::Region {
            target_region,
            ordinal,
            zone,
        })
    };
    match destination {
        TabDropDestination::TabStack {
            tab_stack_id,
            insert_index,
        } => region_destination(
            shell_state
                .region_id_for_tab_stack(tab_stack_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "docking destination tab stack has no composition region",
                    )
                })?,
            insert_index,
            DockZone::Center,
        ),
        TabDropDestination::SplitIntoArea {
            target_tab_stack_id,
            side,
        } => region_destination(
            shell_state
                .region_id_for_tab_stack(target_tab_stack_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "split destination tab stack has no composition region",
                    )
                })?,
            0,
            dock_zone(side),
        ),
        TabDropDestination::SplitIntoHost {
            target_host_id,
            side,
        } => region_destination(
            shell_state
                .stack_region_for_host(target_host_id)
                .ok_or_else(|| {
                    EditorMutationError::runtime_rejected(
                        "split destination host has no stack composition region",
                    )
                })?,
            0,
            dock_zone(side),
        ),
        TabDropDestination::SplitIntoRoot { side } => region_destination(
            shell_state.primary_stack_region().ok_or_else(|| {
                EditorMutationError::runtime_rejected(
                    "primary composition root has no stack destination",
                )
            })?,
            0,
            dock_zone(side),
        ),
        TabDropDestination::NewFloatingHost => Ok(EditorDockingDestination::NewTarget),
    }
}

fn dock_zone(side: DockSplitSide) -> DockZone {
    match side {
        DockSplitSide::Left => DockZone::Left,
        DockSplitSide::Right => DockZone::Right,
        DockSplitSide::Top => DockZone::Top,
        DockSplitSide::Bottom => DockZone::Bottom,
    }
}
