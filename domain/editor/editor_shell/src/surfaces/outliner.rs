//! File: domain/editor/editor_shell/src/surfaces/outliner.rs
//! Purpose: Outliner surface workflow contracts.

use editor_core::EntityId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutlinerSurfaceAction {
    SelectEntity { entity: EntityId },
    SelectRow { entities: Vec<EntityId> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutlinerDomainMutation {
    SelectEntity { entity: EntityId },
}
