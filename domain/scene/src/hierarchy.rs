//! Runtime scene-hierarchy relation projected from authored scene parentage.

use runen_ecs::{CyclePolicy, Directed, Relation, RelationConstraints, SourceCardinality};

/// Derived runtime parent relation for scene entities.
///
/// Authored hierarchy, persistence, and undo/redo remain owned by the editor scene
/// document. This relation is the ECS projection used by runtime consumers.
pub struct SceneChildOf;

impl Relation for SceneChildOf {
    type Kind = Directed;

    const CONSTRAINTS: RelationConstraints = RelationConstraints::new()
        .source_cardinality(SourceCardinality::One)
        .cycles(CyclePolicy::Forbid);
}
