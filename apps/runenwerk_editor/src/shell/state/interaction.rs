use std::collections::BTreeMap;

use editor_shell::{
    ActiveTabDragVisualState, BODY_CONSOLE_SPLIT_WIDGET_ID, CENTER_RIGHT_SPLIT_WIDGET_ID,
    DockDropCandidate, DockDropCandidateState, DockSplitSide, DockingInteractionVisualState,
    DockingPreviewDropTarget, EditorCompositionRuntime, EditorDockingIntent,
    LEFT_RIGHT_SPLIT_WIDGET_ID, PanelHostId, PanelInstanceId, RegionCompassViewModel, TabStackId,
    WidgetId, WorkspaceSplitAxis,
};
use ui_composition::{PresentationTargetId, RegionId, SplitFraction, StateRevision};
use ui_math::UiPoint;

use super::RunenwerkEditorShellState;

const TAB_DRAG_THRESHOLD_PX: f32 = 6.0;

pub(super) fn initial_interaction_state_by_target(
    runtime: &EditorCompositionRuntime,
) -> BTreeMap<PresentationTargetId, TargetInteractionState> {
    runtime
        .composition()
        .definition()
        .targets()
        .iter()
        .map(|target| (target.id, TargetInteractionState::default()))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TabDragSession {
    panel_instance_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    pointer_down: UiPoint,
    projection_epoch: u64,
    active: bool,
    drop_candidate_cycle_index: usize,
    drop_candidate_cycle_side: Option<DockSplitSide>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct TargetInteractionState {
    tab_drag_session: Option<TabDragSession>,
    split_resize_session: Option<SplitResizeSession>,
    corner_split_resize_session: Option<CornerSplitResizeSession>,
    corner_area_split_session: Option<CornerAreaSplitSession>,
    docking_visual_state: DockingInteractionVisualState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSplitKind {
    BodyConsole,
    LeftRight,
    CenterRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SplitResizeSession {
    split_region_id: RegionId,
    split_host_id: PanelHostId,
    split_widget_id: WidgetId,
    axis: WorkspaceSplitAxis,
    source_revision: StateRevision,
    preview_fraction: SplitFraction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerSplitResizeSession {
    pub horizontal_split_host_id: PanelHostId,
    pub horizontal_split_widget_id: WidgetId,
    pub vertical_split_host_id: PanelHostId,
    pub vertical_split_widget_id: WidgetId,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerAreaSplitSession {
    pub tab_stack_id: TabStackId,
    pub pointer_down: UiPoint,
    pub projection_epoch: u64,
}

impl RunenwerkEditorShellState {
    pub(super) fn reconcile_interaction_targets(&mut self, target_ids: &[PresentationTargetId]) {
        self.interaction_by_target
            .retain(|target_id, _| target_ids.contains(target_id));
        for &target_id in target_ids {
            self.interaction_by_target.entry(target_id).or_default();
        }
    }

    fn interaction_state(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<&TargetInteractionState> {
        self.interaction_by_target.get(&target_id)
    }

    fn interaction_state_mut(
        &mut self,
        target_id: PresentationTargetId,
    ) -> &mut TargetInteractionState {
        self.interaction_by_target.entry(target_id).or_default()
    }

    pub fn docking_visual_state(&self) -> DockingInteractionVisualState {
        self.docking_visual_state_for_target(self.primary_composition_target_id())
    }

    pub fn docking_visual_state_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> DockingInteractionVisualState {
        self.interaction_by_target
            .get(&target_id)
            .map(|state| state.docking_visual_state.clone())
            .unwrap_or_default()
    }

    pub fn begin_tab_drag_candidate(
        &mut self,
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        pointer_down: UiPoint,
        projection_epoch: u64,
    ) {
        self.begin_tab_drag_candidate_for_target(
            self.primary_composition_target_id(),
            panel_instance_id,
            source_tab_stack_id,
            pointer_down,
            projection_epoch,
        );
    }

    pub fn begin_tab_drag_candidate_for_target(
        &mut self,
        target_id: PresentationTargetId,
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        pointer_down: UiPoint,
        projection_epoch: u64,
    ) {
        let state = self.interaction_state_mut(target_id);
        state.tab_drag_session = Some(TabDragSession {
            panel_instance_id,
            source_tab_stack_id,
            pointer_down,
            projection_epoch,
            active: false,
            drop_candidate_cycle_index: 0,
            drop_candidate_cycle_side: None,
        });
        state.docking_visual_state.active_tab_drag = None;
    }

    pub fn update_tab_drag_pointer(
        &mut self,
        pointer: UiPoint,
        current_projection_epoch: u64,
    ) -> bool {
        self.update_tab_drag_pointer_for_target(
            self.primary_composition_target_id(),
            pointer,
            current_projection_epoch,
        )
    }

    pub fn update_tab_drag_pointer_for_target(
        &mut self,
        target_id: PresentationTargetId,
        pointer: UiPoint,
        current_projection_epoch: u64,
    ) -> bool {
        let state = self.interaction_state_mut(target_id);
        let Some(mut session) = state.tab_drag_session else {
            return false;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if session.active {
            state.tab_drag_session = Some(session);
            return true;
        }

        let delta_x = pointer.x - session.pointer_down.x;
        let delta_y = pointer.y - session.pointer_down.y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        if distance_squared < TAB_DRAG_THRESHOLD_PX * TAB_DRAG_THRESHOLD_PX {
            return false;
        }

        session.active = true;
        state.tab_drag_session = Some(session);
        state.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id: session.panel_instance_id,
            source_tab_stack_id: session.source_tab_stack_id,
            preview_target: None,
            preview_candidates: Vec::new(),
            region_compass_anchor: None,
            region_compass: None,
        });
        true
    }

    pub fn set_tab_drag_preview(
        &mut self,
        target: Option<DockingPreviewDropTarget>,
        candidates: Vec<DockDropCandidate>,
        active_side: Option<DockSplitSide>,
        current_projection_epoch: u64,
    ) {
        self.set_tab_drag_preview_for_target(
            self.primary_composition_target_id(),
            target,
            candidates,
            active_side,
            current_projection_epoch,
        );
    }

    pub fn set_tab_drag_preview_for_target(
        &mut self,
        target_id: PresentationTargetId,
        target: Option<DockingPreviewDropTarget>,
        candidates: Vec<DockDropCandidate>,
        active_side: Option<DockSplitSide>,
        current_projection_epoch: u64,
    ) {
        let state = self.interaction_state_mut(target_id);
        let Some(mut session) = state.tab_drag_session else {
            return;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if session.drop_candidate_cycle_side != active_side {
            session.drop_candidate_cycle_side = active_side;
        }
        if !session.active {
            state.tab_drag_session = Some(session);
            return;
        }
        state.tab_drag_session = Some(session);
        let previous = state.docking_visual_state.active_tab_drag.as_ref();
        let region_compass_anchor = previous.and_then(|drag| drag.region_compass_anchor);
        let region_compass = previous.and_then(|drag| drag.region_compass.clone());
        state.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id: session.panel_instance_id,
            source_tab_stack_id: session.source_tab_stack_id,
            preview_target: target,
            preview_candidates: candidates,
            region_compass_anchor,
            region_compass,
        });
    }

    pub fn set_region_compass_for_target(
        &mut self,
        target_id: PresentationTargetId,
        anchor: WidgetId,
        compass: RegionCompassViewModel,
        current_projection_epoch: u64,
    ) {
        let state = self.interaction_state_mut(target_id);
        let Some(mut session) = state.tab_drag_session else {
            return;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if !session.active {
            state.tab_drag_session = Some(session);
            return;
        }
        state.tab_drag_session = Some(session);
        if let Some(drag) = state.docking_visual_state.active_tab_drag.as_mut() {
            drag.region_compass_anchor = Some(anchor);
            drag.region_compass = Some(compass);
        }
    }

    pub fn clear_region_compass_for_target(&mut self, target_id: PresentationTargetId) {
        if let Some(drag) = self
            .interaction_by_target
            .get_mut(&target_id)
            .and_then(|state| state.docking_visual_state.active_tab_drag.as_mut())
        {
            drag.region_compass_anchor = None;
            drag.region_compass = None;
            drag.preview_target = None;
            drag.preview_candidates.clear();
        }
    }

    pub fn focus_region_compass_detach_for_target(
        &mut self,
        target_id: PresentationTargetId,
    ) -> bool {
        let Some(compass) = self
            .interaction_by_target
            .get_mut(&target_id)
            .and_then(|state| state.docking_visual_state.active_tab_drag.as_mut())
            .and_then(|drag| drag.region_compass.as_mut())
        else {
            return false;
        };
        compass.focus_detach();
        true
    }

    pub fn region_compass_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<&RegionCompassViewModel> {
        self.interaction_by_target
            .get(&target_id)
            .and_then(|state| state.docking_visual_state.active_tab_drag.as_ref())
            .and_then(|drag| drag.region_compass.as_ref())
    }

    pub fn finish_region_compass_for_target(
        &mut self,
        target_id: PresentationTargetId,
        current_projection_epoch: u64,
    ) -> Option<EditorDockingIntent> {
        let state = self.interaction_by_target.get(&target_id)?;
        let session = state.tab_drag_session?;
        let compass = state
            .docking_visual_state
            .active_tab_drag
            .as_ref()?
            .region_compass
            .clone()?;
        self.clear_tab_drag_for_target(target_id);
        if !session.active || session.projection_epoch != current_projection_epoch {
            return None;
        }
        let source_revision = self.composition_runtime.composition().revision();
        if compass.session == editor_shell::RegionCompassSessionState::DetachFocused {
            return Some(EditorDockingIntent::detach_to_new_target(
                source_revision,
                compass.unit,
            ));
        }
        let zone = compass.focused_zone()?;
        Some(EditorDockingIntent {
            source_revision,
            unit: compass.unit,
            destination: editor_shell::EditorDockingDestination::Region {
                target_region: compass.region,
                ordinal: compass.ordinal,
                zone,
            },
        })
    }

    pub fn tab_drag_drop_candidate_cycle(&self) -> (usize, Option<DockSplitSide>) {
        self.tab_drag_drop_candidate_cycle_for_target(self.primary_composition_target_id())
    }

    pub fn tab_drag_drop_candidate_cycle_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> (usize, Option<DockSplitSide>) {
        self.interaction_by_target
            .get(&target_id)
            .and_then(|state| state.tab_drag_session)
            .map(|session| {
                (
                    session.drop_candidate_cycle_index,
                    session.drop_candidate_cycle_side,
                )
            })
            .unwrap_or((0, None))
    }

    pub fn cycle_active_tab_drag_preview_candidate(&mut self) -> bool {
        self.cycle_active_tab_drag_preview_candidate_for_target(
            self.primary_composition_target_id(),
        )
    }

    pub fn cycle_active_tab_drag_preview_candidate_for_target(
        &mut self,
        target_id: PresentationTargetId,
    ) -> bool {
        let state = self.interaction_state_mut(target_id);
        let Some(session) = state.tab_drag_session.as_mut() else {
            return false;
        };
        if !session.active {
            return false;
        }
        let Some(drag) = state.docking_visual_state.active_tab_drag.as_mut() else {
            return false;
        };
        if drag.preview_candidates.is_empty() {
            return false;
        }
        let active_side = drag
            .preview_candidates
            .iter()
            .find(|candidate| candidate.state.is_active())
            .map(|candidate| candidate.side)
            .or(session.drop_candidate_cycle_side)
            .or_else(|| {
                drag.preview_candidates
                    .iter()
                    .find(|candidate| candidate.state.is_selectable())
                    .map(|candidate| candidate.side)
            });
        let Some(active_side) = active_side else {
            return false;
        };
        let same_side_indices = drag
            .preview_candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                (candidate.state.is_selectable() && candidate.side == active_side).then_some(index)
            })
            .collect::<Vec<_>>();
        if same_side_indices.is_empty() {
            return false;
        }
        session.drop_candidate_cycle_index = session.drop_candidate_cycle_index.saturating_add(1);
        session.drop_candidate_cycle_side = Some(active_side);
        let active_index =
            same_side_indices[session.drop_candidate_cycle_index % same_side_indices.len()];
        for (index, candidate) in drag.preview_candidates.iter_mut().enumerate() {
            if candidate.state.is_selectable() {
                candidate.state = DockDropCandidateState::selectable(index == active_index);
            }
        }
        drag.preview_target = Some(drag.preview_candidates[active_index].target);
        true
    }

    pub fn finish_tab_drag(
        &mut self,
        current_projection_epoch: u64,
    ) -> Option<(
        PanelInstanceId,
        TabStackId,
        Option<DockingPreviewDropTarget>,
        u64,
    )> {
        self.finish_tab_drag_for_target(
            self.primary_composition_target_id(),
            current_projection_epoch,
        )
    }

    pub fn finish_tab_drag_for_target(
        &mut self,
        target_id: PresentationTargetId,
        current_projection_epoch: u64,
    ) -> Option<(
        PanelInstanceId,
        TabStackId,
        Option<DockingPreviewDropTarget>,
        u64,
    )> {
        let state = self.interaction_by_target.get(&target_id)?;
        let session = state.tab_drag_session?;
        let preview_target = state
            .docking_visual_state
            .active_tab_drag
            .as_ref()
            .and_then(|drag| drag.preview_target);
        self.clear_tab_drag_for_target(target_id);
        if !session.active {
            return None;
        }
        Some((
            session.panel_instance_id,
            session.source_tab_stack_id,
            preview_target,
            current_projection_epoch,
        ))
    }

    pub fn tab_drag_candidate(&self) -> Option<(PanelInstanceId, TabStackId, u64, bool)> {
        self.tab_drag_candidate_for_target(self.primary_composition_target_id())
    }

    pub fn tab_drag_candidate_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<(PanelInstanceId, TabStackId, u64, bool)> {
        self.interaction_by_target
            .get(&target_id)
            .and_then(|state| state.tab_drag_session)
            .map(|session| {
                (
                    session.panel_instance_id,
                    session.source_tab_stack_id,
                    session.projection_epoch,
                    session.active,
                )
            })
    }

    pub fn clear_tab_drag(&mut self) {
        self.clear_tab_drag_for_target(self.primary_composition_target_id());
    }

    pub fn clear_tab_drag_for_target(&mut self, target_id: PresentationTargetId) {
        let state = self.interaction_state_mut(target_id);
        state.tab_drag_session = None;
        state.docking_visual_state.active_tab_drag = None;
    }

    pub fn clear_all_tab_drags(&mut self) {
        for state in self.interaction_by_target.values_mut() {
            state.tab_drag_session = None;
            state.docking_visual_state.active_tab_drag = None;
        }
    }

    pub fn begin_split_resize(&mut self, split_kind: WorkspaceSplitKind) {
        if let Some(split_host_id) = self.resolve_split_host_id(split_kind) {
            self.begin_workspace_split_resize(
                split_host_id,
                split_kind_widget_id(split_kind),
                split_kind_axis(split_kind),
            );
        }
    }

    pub fn begin_workspace_split_resize(
        &mut self,
        split_host_id: PanelHostId,
        split_widget_id: WidgetId,
        axis: WorkspaceSplitAxis,
    ) {
        self.begin_workspace_split_resize_for_target(
            self.primary_composition_target_id(),
            split_host_id,
            split_widget_id,
            axis,
        );
    }

    pub fn begin_workspace_split_resize_for_target(
        &mut self,
        target_id: PresentationTargetId,
        split_host_id: PanelHostId,
        split_widget_id: WidgetId,
        axis: WorkspaceSplitAxis,
    ) {
        let Some(split_region_id) = self.region_id_for_host(split_host_id) else {
            return;
        };
        let Some(initial_fraction) = self
            .composition_runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.id == split_region_id)
            .and_then(|region| match &region.kind {
                ui_composition::RegionKind::Split { fraction, .. } => Some(*fraction),
                _ => None,
            })
        else {
            return;
        };
        let source_revision = self.composition_runtime.composition().revision();
        let state = self.interaction_state_mut(target_id);
        state.corner_split_resize_session = None;
        state.split_resize_session = Some(SplitResizeSession {
            split_region_id,
            split_host_id,
            split_widget_id,
            axis,
            source_revision,
            preview_fraction: initial_fraction,
        });
        state.docking_visual_state.active_split_border_widget = Some(split_widget_id);
        state.docking_visual_state.active_split_preview_fraction = Some((
            split_widget_id,
            initial_fraction.basis_points() as f32 / 10_000.0,
        ));
    }

    pub fn update_split_resize_preview_for_target(
        &mut self,
        target_id: PresentationTargetId,
        fraction: SplitFraction,
    ) -> bool {
        let state = self.interaction_state_mut(target_id);
        let Some(session) = state.split_resize_session.as_mut() else {
            return false;
        };
        session.preview_fraction = fraction;
        state.docking_visual_state.active_split_preview_fraction = Some((
            session.split_widget_id,
            fraction.basis_points() as f32 / 10_000.0,
        ));
        true
    }

    pub fn finish_split_resize_for_target(
        &mut self,
        target_id: PresentationTargetId,
    ) -> Option<(RegionId, SplitFraction, StateRevision)> {
        let state = self.interaction_by_target.get_mut(&target_id)?;
        let session = state.split_resize_session.take()?;
        state.docking_visual_state.active_split_border_widget = None;
        state.docking_visual_state.active_split_preview_fraction = None;
        Some((
            session.split_region_id,
            session.preview_fraction,
            session.source_revision,
        ))
    }

    pub fn begin_workspace_corner_split_resize(&mut self, session: CornerSplitResizeSession) {
        let state = self.interaction_state_mut(self.primary_composition_target_id());
        state.split_resize_session = None;
        state.corner_area_split_session = None;
        state.corner_split_resize_session = Some(session);
        state.docking_visual_state.active_split_border_widget =
            Some(session.horizontal_split_widget_id);
    }

    pub fn begin_corner_area_split(&mut self, session: CornerAreaSplitSession) {
        let state = self.interaction_state_mut(self.primary_composition_target_id());
        state.split_resize_session = None;
        state.corner_split_resize_session = None;
        state.corner_area_split_session = Some(session);
    }

    pub fn active_corner_area_split_session(&self) -> Option<CornerAreaSplitSession> {
        self.interaction_state(self.primary_composition_target_id())
            .and_then(|state| state.corner_area_split_session)
    }

    pub fn clear_corner_area_split(&mut self) {
        self.interaction_state_mut(self.primary_composition_target_id())
            .corner_area_split_session = None;
    }

    pub fn active_split_resize_kind(&self) -> Option<WorkspaceSplitKind> {
        let session = self
            .interaction_state(self.primary_composition_target_id())?
            .split_resize_session?;
        [
            WorkspaceSplitKind::BodyConsole,
            WorkspaceSplitKind::LeftRight,
            WorkspaceSplitKind::CenterRight,
        ]
        .into_iter()
        .find(|kind| self.resolve_split_host_id(*kind) == Some(session.split_host_id))
    }

    pub fn active_split_resize_session(
        &self,
    ) -> Option<(PanelHostId, WidgetId, WorkspaceSplitAxis)> {
        self.active_split_resize_session_for_target(self.primary_composition_target_id())
    }

    pub fn active_split_resize_session_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<(PanelHostId, WidgetId, WorkspaceSplitAxis)> {
        self.interaction_state(target_id)
            .and_then(|state| state.split_resize_session)
            .map(|session| (session.split_host_id, session.split_widget_id, session.axis))
    }

    pub fn active_corner_split_resize_session(&self) -> Option<CornerSplitResizeSession> {
        self.interaction_state(self.primary_composition_target_id())
            .and_then(|state| state.corner_split_resize_session)
    }

    pub fn clear_split_resize(&mut self) {
        for state in self.interaction_by_target.values_mut() {
            state.split_resize_session = None;
            state.corner_split_resize_session = None;
            state.corner_area_split_session = None;
            state.docking_visual_state.active_split_border_widget = None;
            state.docking_visual_state.active_split_preview_fraction = None;
        }
    }

    pub fn clear_split_resize_for_target(&mut self, target_id: PresentationTargetId) {
        if let Some(state) = self.interaction_by_target.get_mut(&target_id) {
            state.split_resize_session = None;
            state.corner_split_resize_session = None;
            state.corner_area_split_session = None;
            state.docking_visual_state.active_split_border_widget = None;
            state.docking_visual_state.active_split_preview_fraction = None;
        }
    }

    fn resolve_split_host_id(&self, split_kind: WorkspaceSplitKind) -> Option<PanelHostId> {
        let root = self
            .composition_projection
            .roots
            .iter()
            .find(|root| root.primary)?;
        let root_region = self
            .composition_runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.id == root.region)?;
        let ui_composition::RegionKind::Split { first, .. } = &root_region.kind else {
            return None;
        };
        let host_for_region = |region_id| {
            self.composition_runtime
                .extension()
                .region(region_id)
                .and_then(|record| PanelHostId::try_from_raw(record.compatibility_host_raw).ok())
        };
        match split_kind {
            WorkspaceSplitKind::BodyConsole => host_for_region(root.region),
            WorkspaceSplitKind::LeftRight => host_for_region(*first),
            WorkspaceSplitKind::CenterRight => {
                let left_right = self
                    .composition_runtime
                    .composition()
                    .definition()
                    .regions()
                    .iter()
                    .find(|region| region.id == *first)?;
                let ui_composition::RegionKind::Split { second, .. } = &left_right.kind else {
                    return None;
                };
                host_for_region(*second)
            }
        }
    }
}

fn split_kind_widget_id(kind: WorkspaceSplitKind) -> WidgetId {
    match kind {
        WorkspaceSplitKind::BodyConsole => BODY_CONSOLE_SPLIT_WIDGET_ID,
        WorkspaceSplitKind::LeftRight => LEFT_RIGHT_SPLIT_WIDGET_ID,
        WorkspaceSplitKind::CenterRight => CENTER_RIGHT_SPLIT_WIDGET_ID,
    }
}

fn split_kind_axis(kind: WorkspaceSplitKind) -> WorkspaceSplitAxis {
    match kind {
        WorkspaceSplitKind::BodyConsole => WorkspaceSplitAxis::Vertical,
        WorkspaceSplitKind::LeftRight => WorkspaceSplitAxis::Horizontal,
        WorkspaceSplitKind::CenterRight => WorkspaceSplitAxis::Vertical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_compass_finish_emits_typed_revision_bound_docking_intent() {
        let mut shell_state = RunenwerkEditorShellState::new();
        let extension = shell_state
            .composition_runtime()
            .extension()
            .mounted_units()
            .first()
            .cloned()
            .expect("default composition should mount editor content");
        let unit = extension.mounted_unit_id;
        let panel = PanelInstanceId::try_from_raw(extension.panel_instance_raw).unwrap();
        let target_region = shell_state
            .composition_runtime()
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.kind.mounted_units().contains(&unit))
            .map(|region| region.id)
            .unwrap();
        let stack = shell_state.tab_stack_id_for_region(target_region).unwrap();
        let target = shell_state.primary_composition_target_id();
        let epoch = shell_state.current_projection_epoch();
        shell_state.begin_tab_drag_candidate_for_target(
            target,
            panel,
            stack,
            UiPoint::new(10.0, 10.0),
            epoch,
        );
        assert!(shell_state.update_tab_drag_pointer_for_target(
            target,
            UiPoint::new(18.0, 10.0),
            epoch,
        ));
        shell_state.set_region_compass_for_target(
            target,
            WidgetId(91),
            RegionCompassViewModel::active(
                target,
                target_region,
                unit,
                ui_adaptive_composition::DockZone::Center,
                "content",
                "region",
                editor_shell::RegionCompassAccessibility::default(),
            )
            .with_ordinal(1),
            epoch,
        );

        let intent = shell_state
            .finish_region_compass_for_target(target, epoch)
            .expect("typed Region Compass intent");
        assert_eq!(intent.unit, unit);
        assert_eq!(
            intent.source_revision,
            shell_state.composition_runtime().composition().revision()
        );
        assert_eq!(
            intent.destination,
            editor_shell::EditorDockingDestination::Region {
                target_region,
                ordinal: 1,
                zone: ui_adaptive_composition::DockZone::Center,
            }
        );
    }

    #[test]
    fn tab_drag_candidate_cycling_skips_invalid_drop_candidates() {
        let mut shell_state = RunenwerkEditorShellState::new();
        let panel_instance_id = PanelInstanceId::try_from_raw(1).unwrap();
        let source_tab_stack_id = TabStackId::try_from_raw(2).unwrap();
        let active_target = DockingPreviewDropTarget::SplitIntoRoot {
            side: DockSplitSide::Left,
        };
        let next_target = DockingPreviewDropTarget::SplitIntoRoot {
            side: DockSplitSide::Right,
        };

        let target_id = shell_state.primary_composition_target_id();
        let interaction = shell_state.interaction_state_mut(target_id);
        interaction.tab_drag_session = Some(TabDragSession {
            panel_instance_id,
            source_tab_stack_id,
            pointer_down: UiPoint::new(10.0, 10.0),
            projection_epoch: 1,
            active: true,
            drop_candidate_cycle_index: 0,
            drop_candidate_cycle_side: Some(DockSplitSide::Left),
        });
        interaction.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id,
            source_tab_stack_id,
            preview_target: Some(active_target),
            preview_candidates: vec![
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoArea {
                        target_tab_stack_id: source_tab_stack_id,
                        side: DockSplitSide::Left,
                    },
                    scope: editor_shell::DockDropScope::Area,
                    side: DockSplitSide::Left,
                    anchor_widget_id: WidgetId(10),
                    state: DockDropCandidateState::Invalid {
                        reason: editor_shell::DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea,
                    },
                },
                DockDropCandidate {
                    target: active_target,
                    scope: editor_shell::DockDropScope::Workspace,
                    side: DockSplitSide::Left,
                    anchor_widget_id: WidgetId(11),
                    state: DockDropCandidateState::Active,
                },
                DockDropCandidate {
                    target: next_target,
                    scope: editor_shell::DockDropScope::Workspace,
                    side: DockSplitSide::Left,
                    anchor_widget_id: WidgetId(12),
                    state: DockDropCandidateState::Candidate,
                },
            ],
            region_compass_anchor: None,
            region_compass: None,
        });

        assert!(shell_state.cycle_active_tab_drag_preview_candidate());

        let visual = shell_state.docking_visual_state_for_target(target_id);
        let drag = visual
            .active_tab_drag
            .as_ref()
            .expect("tab drag visual state should remain active");
        assert_eq!(drag.preview_target, Some(next_target));
        assert!(matches!(
            drag.preview_candidates[0].state,
            DockDropCandidateState::Invalid {
                reason: editor_shell::DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea
            }
        ));
        assert_eq!(
            drag.preview_candidates[1].state,
            DockDropCandidateState::Candidate
        );
        assert_eq!(
            drag.preview_candidates[2].state,
            DockDropCandidateState::Active
        );
    }

    #[test]
    fn tab_drag_sessions_are_isolated_by_presentation_target() {
        let mut shell_state = RunenwerkEditorShellState::new();
        let primary = shell_state.primary_composition_target_id();
        let secondary = PresentationTargetId::new(primary.raw() + 100);
        let panel = PanelInstanceId::try_from_raw(1).unwrap();
        let stack = TabStackId::try_from_raw(2).unwrap();

        shell_state.begin_tab_drag_candidate_for_target(
            primary,
            panel,
            stack,
            UiPoint::new(10.0, 10.0),
            1,
        );
        shell_state.begin_tab_drag_candidate_for_target(
            secondary,
            panel,
            stack,
            UiPoint::new(50.0, 50.0),
            1,
        );
        assert!(shell_state.update_tab_drag_pointer_for_target(
            primary,
            UiPoint::new(17.0, 10.0),
            1,
        ));

        assert!(
            shell_state
                .docking_visual_state_for_target(primary)
                .active_tab_drag
                .is_some()
        );
        assert!(
            shell_state
                .docking_visual_state_for_target(secondary)
                .active_tab_drag
                .is_none()
        );
        assert_eq!(
            shell_state.tab_drag_candidate_for_target(secondary),
            Some((panel, stack, 1, false))
        );
    }
}
