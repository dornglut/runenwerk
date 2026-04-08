//! File: domain/editor/editor_inspector/src/resolution.rs
//! Purpose: Resolve inspector targets from editor selection state.

use editor_core::SelectionSet;

use crate::InspectTarget;

pub fn resolve_primary_inspect_target(selection: &SelectionSet) -> Option<InspectTarget> {
    selection.primary().map(InspectTarget::from)
}

pub fn resolve_all_inspect_targets(selection: &SelectionSet) -> Vec<InspectTarget> {
    selection.iter().map(InspectTarget::from).collect()
}
