//! File: domain/editor/editor_core/src/migration.rs
//! Purpose: Migration-path contracts for explicit workflow promotion and compensation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MigrationPathId(pub u64);

pub const SCENE_MIGRATION_V1_TO_V2_DEFAULT_PRIMITIVE: MigrationPathId = MigrationPathId(1);

pub const fn migration_path_label(path: MigrationPathId) -> &'static str {
    match path {
        SCENE_MIGRATION_V1_TO_V2_DEFAULT_PRIMITIVE => "scene:v1->v2:default-primitive",
        _ => "migration:unknown",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationVisibilityPolicy {
    LocalOnly,
    ShareAfterCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationCompensation {
    None,
    ReloadPreviousSnapshot,
    RebuildRuntimeProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MigrationPathContract {
    pub id: MigrationPathId,
    pub visibility: MigrationVisibilityPolicy,
    pub compensation: MigrationCompensation,
}
