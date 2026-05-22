//! Visual layout edit application over authored UI templates.

use super::{
    UiVisualLayoutActivationMode, UiVisualLayoutDiagnostic, UiVisualLayoutDiff,
    UiVisualLayoutDiffChange, UiVisualLayoutDiffChangeKind, UiVisualLayoutEditContext,
    UiVisualLayoutEditKind, UiVisualLayoutOperation,
};
use crate::{
    AuthoredId, AuthoredUiNodePath, AuthoredUiTemplate, UiAxisDefinition,
    UiDefinitionDiagnosticSeverity, UiNodeDefinition, UiNodeId,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq)]
pub struct UiVisualLayoutEditReport {
    pub template: AuthoredUiTemplate,
    pub diff: Option<UiVisualLayoutDiff>,
    pub diagnostics: Vec<UiVisualLayoutDiagnostic>,
}

impl UiVisualLayoutEditReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiDefinitionDiagnosticSeverity::Error)
    }
}

pub fn apply_visual_layout_operation(
    mut template: AuthoredUiTemplate,
    operation: &UiVisualLayoutOperation,
    mode: UiVisualLayoutActivationMode,
    context: &UiVisualLayoutEditContext,
) -> UiVisualLayoutEditReport {
    let mut diagnostics = Vec::new();

    if mode == UiVisualLayoutActivationMode::Activate && operation.preview_only {
        diagnostics.push(diagnostic(
            "ui.visual_layout.preview_only_activation",
            "preview-only visual layout edits cannot be activated",
            operation,
            context,
            Some(operation.target_path.clone()),
            "serialize the edit into a deterministic authored definition patch before activation",
        ));
        return report(template, None, diagnostics);
    }

    if !context.supports_target_profile(&operation.target_profile) {
        diagnostics.push(diagnostic(
            "ui.visual_layout.target_profile.unsupported",
            format!(
                "target profile '{}' does not support this visual layout edit context",
                operation.target_profile
            ),
            operation,
            context,
            Some(operation.target_path.clone()),
            "select a supported target profile or add an explicit profile compatibility declaration",
        ));
        return report(template, None, diagnostics);
    }

    match validate_target(&template.root, operation) {
        Ok(()) => {}
        Err(diagnostic_message) => {
            diagnostics.push(diagnostic_message.into_diagnostic(operation, context));
            return report(template, None, diagnostics);
        }
    }

    let changes = match apply_operation(&mut template.root, operation, context) {
        Ok(changes) => changes,
        Err(diagnostic_message) => {
            diagnostics.push(diagnostic_message.into_diagnostic(operation, context));
            return report(template, None, diagnostics);
        }
    };

    let diff = if operation.preview_only {
        None
    } else {
        Some(UiVisualLayoutDiff {
            operation_id: operation.id.clone(),
            source_document: operation.source_document.clone(),
            target_profile: operation.target_profile.clone(),
            changes,
        })
    };

    report(template, diff, diagnostics)
}

fn report(
    template: AuthoredUiTemplate,
    diff: Option<UiVisualLayoutDiff>,
    diagnostics: Vec<UiVisualLayoutDiagnostic>,
) -> UiVisualLayoutEditReport {
    UiVisualLayoutEditReport {
        template,
        diff,
        diagnostics,
    }
}

fn apply_operation(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    context: &UiVisualLayoutEditContext,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    match &operation.kind {
        UiVisualLayoutEditKind::InsertNode { index, node } => {
            apply_insert(root, operation, context, *index, node.clone())
        }
        UiVisualLayoutEditKind::RemoveNode => apply_remove(root, operation),
        UiVisualLayoutEditKind::MoveNode {
            new_parent_path,
            new_index,
        } => apply_move(root, operation, new_parent_path, *new_index),
        UiVisualLayoutEditKind::ReorderSibling {
            from_index,
            to_index,
            expected_child_id,
        } => apply_reorder(root, operation, *from_index, *to_index, expected_child_id),
        UiVisualLayoutEditKind::ChangeStackAxis { axis } => {
            apply_stack_axis(root, operation, *axis)
        }
        UiVisualLayoutEditKind::ChangeSplitRatio { ratio } => {
            apply_split_ratio(root, operation, *ratio)
        }
        UiVisualLayoutEditKind::WrapSelectionInContainer {
            first_index,
            count,
            container,
        } => apply_wrap(
            root,
            operation,
            context,
            *first_index,
            *count,
            container.clone(),
        ),
        UiVisualLayoutEditKind::UnwrapContainer => apply_unwrap(root, operation),
        UiVisualLayoutEditKind::ReplaceTemplateReference { template } => {
            apply_replace_template(root, operation, template.clone())
        }
    }
}

fn apply_insert(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    context: &UiVisualLayoutEditContext,
    index: usize,
    node: UiNodeDefinition,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    ensure_new_node_ids_are_unique(root, &node, operation, context)?;
    let parent_path = operation.target_path.clone();
    let node_path = parent_path.child(node.id());
    let after = canonical_text(&node, &node_path)?;
    let children = children_mut_at_path(root, &parent_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "target node cannot contain inserted children",
            Some(parent_path.clone()),
            "target a layout container node",
        )
    })?;
    if index > children.len() {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.child_index.invalid",
            format!(
                "insert index '{}' is outside parent child count '{}'",
                index,
                children.len()
            ),
            Some(parent_path),
            "choose an insertion index within the target parent child range",
        ));
    }
    children.insert(index, node);
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Insert,
        path: node_path,
        before: None,
        after: Some(after),
    }])
}

fn apply_remove(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    let before = canonical_text(
        node_at_path(root, &operation.target_path).expect("target already validated"),
        &operation.target_path,
    )?;
    let removed = remove_node_at_path(root, &operation.target_path)?;
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Remove,
        path: operation.target_path.clone(),
        before: Some(before),
        after: Some(format!("removed '{}'", removed.id())),
    }])
}

fn apply_move(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    new_parent_path: &AuthoredUiNodePath,
    new_index: usize,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    if is_descendant_path(new_parent_path, &operation.target_path) {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.composition_conflict",
            "a node cannot be moved into one of its descendants",
            Some(operation.target_path.clone()),
            "choose a target parent outside the moved node subtree",
        ));
    }
    let old_parent_path = parent_path(&operation.target_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.root_move.unsupported",
            "root node cannot be moved",
            Some(operation.target_path.clone()),
            "move one of the root node children instead",
        )
    })?;
    let old_index = child_index_at_path(root, &operation.target_path)?;
    let new_parent_len = children_at_path(root, new_parent_path)
        .ok_or_else(|| {
            PendingDiagnostic::new(
                "ui.visual_layout.target.missing",
                "move target parent does not exist",
                Some(new_parent_path.clone()),
                "target an existing layout container",
            )
        })?
        .len();
    let adjusted_index =
        if old_parent_path == *new_parent_path && old_index < new_index && new_index > 0 {
            new_index - 1
        } else {
            new_index
        };
    let effective_len = if old_parent_path == *new_parent_path {
        new_parent_len.saturating_sub(1)
    } else {
        new_parent_len
    };
    if adjusted_index > effective_len {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.child_index.invalid",
            format!(
                "move index '{}' is outside target parent child count '{}'",
                new_index, effective_len
            ),
            Some(new_parent_path.clone()),
            "choose a move index within the target parent child range",
        ));
    }

    let moved = remove_node_at_path(root, &operation.target_path)?;
    let moved_id = moved.id().clone();
    let moved_text = canonical_text(&moved, &operation.target_path)?;
    let new_path = new_parent_path.child(&moved_id);
    let new_parent = children_mut_at_path(root, new_parent_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.target.missing",
            "move target parent does not exist after removal",
            Some(new_parent_path.clone()),
            "target an existing layout container",
        )
    })?;
    new_parent.insert(adjusted_index, moved);
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Move,
        path: operation.target_path.clone(),
        before: Some(moved_text),
        after: Some(format!("moved to '{}'", new_path.as_str())),
    }])
}

fn apply_reorder(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    from_index: usize,
    to_index: usize,
    expected_child_id: &UiNodeId,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    let children = children_mut_at_path(root, &operation.target_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "target node cannot reorder children",
            Some(operation.target_path.clone()),
            "target a layout container node",
        )
    })?;
    if from_index >= children.len() || to_index >= children.len() {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.child_index.invalid",
            format!(
                "reorder range '{}' -> '{}' is outside child count '{}'",
                from_index,
                to_index,
                children.len()
            ),
            Some(operation.target_path.clone()),
            "choose reorder indices within the target parent child range",
        ));
    }
    if children[from_index].id() != expected_child_id {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.authored_id.unstable",
            format!(
                "expected child id '{}' at index '{}' but found '{}'",
                expected_child_id,
                from_index,
                children[from_index].id()
            ),
            Some(operation.target_path.clone()),
            "refresh the edit operation against the latest authored definition tree",
        ));
    }

    let moved = children.remove(from_index);
    let moved_id = moved.id().clone();
    children.insert(to_index, moved);
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Reorder,
        path: operation.target_path.child(&moved_id),
        before: Some(format!("index {from_index}")),
        after: Some(format!("index {to_index}")),
    }])
}

fn apply_stack_axis(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    axis: UiAxisDefinition,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    let node = node_mut_at_path(root, &operation.target_path).expect("target already validated");
    let UiNodeDefinition::Stack {
        axis: current_axis, ..
    } = node
    else {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "only Stack nodes can change stack axis",
            Some(operation.target_path.clone()),
            "target a Stack node or convert the node through an explicit structural edit first",
        ));
    };
    let before = canonical_text(current_axis, &operation.target_path)?;
    *current_axis = axis;
    let after = canonical_text(current_axis, &operation.target_path)?;
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Update,
        path: operation.target_path.clone(),
        before: Some(before),
        after: Some(after),
    }])
}

fn apply_split_ratio(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    ratio: f32,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    if !(0.0..1.0).contains(&ratio) {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.split.ratio_out_of_range",
            format!("split ratio '{ratio}' must be greater than 0 and less than 1"),
            Some(operation.target_path.clone()),
            "choose a split ratio in the open range 0.0..1.0",
        ));
    }
    let node = node_mut_at_path(root, &operation.target_path).expect("target already validated");
    let UiNodeDefinition::Split {
        ratio: current_ratio,
        ..
    } = node
    else {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "only Split nodes can change split ratio",
            Some(operation.target_path.clone()),
            "target a Split node or convert the node through an explicit structural edit first",
        ));
    };
    let before = canonical_text(current_ratio, &operation.target_path)?;
    *current_ratio = ratio;
    let after = canonical_text(current_ratio, &operation.target_path)?;
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Update,
        path: operation.target_path.clone(),
        before: Some(before),
        after: Some(after),
    }])
}

fn apply_wrap(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    context: &UiVisualLayoutEditContext,
    first_index: usize,
    count: usize,
    mut container: UiNodeDefinition,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    ensure_new_node_ids_are_unique(root, &container, operation, context)?;
    if count == 0 {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.child_count.invalid",
            "wrap selection requires at least one child",
            Some(operation.target_path.clone()),
            "select one or more sibling nodes before wrapping",
        ));
    }
    if !container.children().is_empty() {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.container.children_non_empty",
            "wrap container must be supplied without authored children",
            Some(operation.target_path.clone()),
            "provide an empty layout container for wrapping selected children",
        ));
    }
    if matches!(container, UiNodeDefinition::Split { .. }) && count != 2 {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.split.child_count",
            "wrapping in a Split container requires exactly two selected children",
            Some(operation.target_path.clone()),
            "select exactly two siblings or use a non-Split layout container",
        ));
    }
    let container_id = container.id().clone();
    let children = children_mut_at_path(root, &operation.target_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "target node cannot wrap children",
            Some(operation.target_path.clone()),
            "target a layout container node",
        )
    })?;
    if first_index >= children.len() || first_index + count > children.len() {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.child_index.invalid",
            format!(
                "wrap range '{}..{}' is outside child count '{}'",
                first_index,
                first_index + count,
                children.len()
            ),
            Some(operation.target_path.clone()),
            "choose a wrap range inside the target parent child range",
        ));
    }
    let selected: Vec<_> = children.drain(first_index..first_index + count).collect();
    *container.children_mut().ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "wrap target is not a layout container",
            Some(operation.target_path.clone()),
            "provide a Panel, Row, Column, Stack, Scroll, or Split container",
        )
    })? = selected;
    let after = canonical_text(&container, &operation.target_path.child(&container_id))?;
    children.insert(first_index, container);
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Wrap,
        path: operation.target_path.child(&container_id),
        before: Some(format!("wrapped {count} children")),
        after: Some(after),
    }])
}

fn apply_unwrap(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    let children = node_at_path(root, &operation.target_path)
        .expect("target already validated")
        .children();
    if children.is_empty() {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.child_count.invalid",
            "unwrap requires a layout container with children",
            Some(operation.target_path.clone()),
            "target a non-empty layout container",
        ));
    }
    let parent_path = parent_path(&operation.target_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.root_unwrap.unsupported",
            "root node cannot be unwrapped",
            Some(operation.target_path.clone()),
            "unwrap a child layout container instead",
        )
    })?;
    let index = child_index_at_path(root, &operation.target_path)?;
    let mut container = remove_node_at_path(root, &operation.target_path)?;
    let unwrapped = std::mem::take(container.children_mut().ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "target node cannot be unwrapped",
            Some(operation.target_path.clone()),
            "target a layout container node",
        )
    })?);
    let count = unwrapped.len();
    let parent = children_mut_at_path(root, &parent_path).expect("parent already validated");
    for (offset, child) in unwrapped.into_iter().enumerate() {
        parent.insert(index + offset, child);
    }
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::Unwrap,
        path: operation.target_path.clone(),
        before: Some(format!("container '{}'", container.id())),
        after: Some(format!("unwrapped {count} children")),
    }])
}

fn apply_replace_template(
    root: &mut UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    template: AuthoredId,
) -> Result<Vec<UiVisualLayoutDiffChange>, PendingDiagnostic> {
    let node = node_mut_at_path(root, &operation.target_path).expect("target already validated");
    let UiNodeDefinition::TemplateRef {
        template: current_template,
        ..
    } = node
    else {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.layout_feature.unsupported",
            "only TemplateRef nodes can replace template references",
            Some(operation.target_path.clone()),
            "target a TemplateRef node",
        ));
    };
    let before = canonical_text(current_template, &operation.target_path)?;
    *current_template = template;
    let after = canonical_text(current_template, &operation.target_path)?;
    Ok(vec![UiVisualLayoutDiffChange {
        kind: UiVisualLayoutDiffChangeKind::ReplaceTemplate,
        path: operation.target_path.clone(),
        before: Some(before),
        after: Some(after),
    }])
}

fn validate_target(
    root: &UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
) -> Result<(), PendingDiagnostic> {
    let target = node_at_path(root, &operation.target_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.target.missing",
            format!(
                "target path '{}' does not exist",
                operation.target_path.as_str()
            ),
            Some(operation.target_path.clone()),
            "refresh the edit operation against the latest authored definition tree",
        )
    })?;
    if target.id() != &operation.expected_node_id {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.authored_id.unstable",
            format!(
                "target path '{}' expected node id '{}' but found '{}'",
                operation.target_path.as_str(),
                operation.expected_node_id,
                target.id()
            ),
            Some(operation.target_path.clone()),
            "refresh the edit operation against the latest authored definition tree",
        ));
    }
    Ok(())
}

fn ensure_new_node_ids_are_unique(
    root: &UiNodeDefinition,
    node: &UiNodeDefinition,
    operation: &UiVisualLayoutOperation,
    context: &UiVisualLayoutEditContext,
) -> Result<(), PendingDiagnostic> {
    let mut existing = BTreeSet::new();
    collect_node_ids(root, &mut existing);
    let mut incoming = BTreeSet::new();
    if let Some(id) = collect_node_ids_unique(node, &mut incoming) {
        return Err(PendingDiagnostic::new(
            "ui.visual_layout.authored_id.duplicate",
            format!("new layout node id '{}' appears more than once", id),
            Some(operation.target_path.clone()),
            "allocate unique stable authored ids for every node in the inserted subtree",
        )
        .with_context(context));
    }
    for id in incoming {
        if existing.contains(&id) {
            return Err(PendingDiagnostic::new(
                "ui.visual_layout.authored_id.duplicate",
                format!("new layout node id '{}' already exists", id),
                Some(operation.target_path.clone()),
                "allocate a stable authored id that is not already present in the definition tree",
            )
            .with_context(context));
        }
    }
    Ok(())
}

fn collect_node_ids_unique(
    node: &UiNodeDefinition,
    ids: &mut BTreeSet<UiNodeId>,
) -> Option<UiNodeId> {
    if !ids.insert(node.id().clone()) {
        return Some(node.id().clone());
    }
    for child in node.children() {
        if let Some(duplicate) = collect_node_ids_unique(child, ids) {
            return Some(duplicate);
        }
    }
    None
}

fn collect_node_ids(node: &UiNodeDefinition, ids: &mut BTreeSet<UiNodeId>) {
    ids.insert(node.id().clone());
    for child in node.children() {
        collect_node_ids(child, ids);
    }
}

fn node_at_path<'a>(
    root: &'a UiNodeDefinition,
    path: &AuthoredUiNodePath,
) -> Option<&'a UiNodeDefinition> {
    let segments = path_segments(path);
    node_at_segments(root, &segments)
}

fn node_at_segments<'a>(
    node: &'a UiNodeDefinition,
    segments: &[&str],
) -> Option<&'a UiNodeDefinition> {
    let (head, tail) = segments.split_first()?;
    if *head != node.id().as_str() {
        return None;
    }
    if tail.is_empty() {
        return Some(node);
    }
    node.children()
        .iter()
        .find(|child| child.id().as_str() == tail[0])
        .and_then(|child| node_at_segments(child, tail))
}

fn node_mut_at_path<'a>(
    root: &'a mut UiNodeDefinition,
    path: &AuthoredUiNodePath,
) -> Option<&'a mut UiNodeDefinition> {
    let segments = path_segments(path);
    node_mut_at_segments(root, &segments)
}

fn node_mut_at_segments<'a>(
    node: &'a mut UiNodeDefinition,
    segments: &[&str],
) -> Option<&'a mut UiNodeDefinition> {
    let (head, tail) = segments.split_first()?;
    if *head != node.id().as_str() {
        return None;
    }
    if tail.is_empty() {
        return Some(node);
    }
    let children = node.children_mut()?;
    children
        .iter_mut()
        .find(|child| child.id().as_str() == tail[0])
        .and_then(|child| node_mut_at_segments(child, tail))
}

fn children_at_path<'a>(
    root: &'a UiNodeDefinition,
    path: &AuthoredUiNodePath,
) -> Option<&'a [UiNodeDefinition]> {
    Some(node_at_path(root, path)?.children())
}

fn children_mut_at_path<'a>(
    root: &'a mut UiNodeDefinition,
    path: &AuthoredUiNodePath,
) -> Option<&'a mut Vec<UiNodeDefinition>> {
    node_mut_at_path(root, path)?.children_mut()
}

fn remove_node_at_path(
    root: &mut UiNodeDefinition,
    path: &AuthoredUiNodePath,
) -> Result<UiNodeDefinition, PendingDiagnostic> {
    let parent_path = parent_path(path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.root_remove.unsupported",
            "root node cannot be removed",
            Some(path.clone()),
            "remove one of the root node children instead",
        )
    })?;
    let child_id = last_path_segment(path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.target.missing",
            format!("target path '{}' is empty", path.as_str()),
            Some(path.clone()),
            "target an existing authored node path",
        )
    })?;
    let parent = children_mut_at_path(root, &parent_path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.target.missing",
            format!(
                "target parent path '{}' does not exist",
                parent_path.as_str()
            ),
            Some(parent_path.clone()),
            "refresh the edit operation against the latest authored definition tree",
        )
    })?;
    let index = parent
        .iter()
        .position(|node| node.id().as_str() == child_id)
        .ok_or_else(|| {
            PendingDiagnostic::new(
                "ui.visual_layout.target.missing",
                format!("target child '{}' does not exist", child_id),
                Some(path.clone()),
                "refresh the edit operation against the latest authored definition tree",
            )
        })?;
    Ok(parent.remove(index))
}

fn child_index_at_path(
    root: &UiNodeDefinition,
    path: &AuthoredUiNodePath,
) -> Result<usize, PendingDiagnostic> {
    let parent_path = parent_path(path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.root_index.unsupported",
            "root node has no sibling index",
            Some(path.clone()),
            "target one of the root node children instead",
        )
    })?;
    let child_id = last_path_segment(path).ok_or_else(|| {
        PendingDiagnostic::new(
            "ui.visual_layout.target.missing",
            format!("target path '{}' is empty", path.as_str()),
            Some(path.clone()),
            "target an existing authored node path",
        )
    })?;
    children_at_path(root, &parent_path)
        .and_then(|children| {
            children
                .iter()
                .position(|node| node.id().as_str() == child_id)
        })
        .ok_or_else(|| {
            PendingDiagnostic::new(
                "ui.visual_layout.target.missing",
                format!("target child '{}' does not exist", child_id),
                Some(path.clone()),
                "refresh the edit operation against the latest authored definition tree",
            )
        })
}

fn parent_path(path: &AuthoredUiNodePath) -> Option<AuthoredUiNodePath> {
    path.as_str()
        .rsplit_once('/')
        .map(|(parent, _)| AuthoredUiNodePath(parent.to_string()))
}

fn last_path_segment(path: &AuthoredUiNodePath) -> Option<&str> {
    path.as_str()
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
}

fn is_descendant_path(candidate: &AuthoredUiNodePath, ancestor: &AuthoredUiNodePath) -> bool {
    candidate
        .as_str()
        .strip_prefix(ancestor.as_str())
        .is_some_and(|rest| rest.starts_with('/'))
}

fn path_segments(path: &AuthoredUiNodePath) -> Vec<&str> {
    path.as_str()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn canonical_text<T: Serialize + ?Sized>(
    value: &T,
    path: &AuthoredUiNodePath,
) -> Result<String, PendingDiagnostic> {
    to_string_pretty(value, PrettyConfig::default()).map_err(|error| {
        PendingDiagnostic::new(
            "ui.visual_layout.diff.non_deterministic",
            format!("failed to serialize deterministic diff text: {error}"),
            Some(path.clone()),
            "ensure the edited value uses deterministic serde-compatible definition data",
        )
    })
}

fn diagnostic(
    code: impl Into<String>,
    message: impl Into<String>,
    operation: &UiVisualLayoutOperation,
    context: &UiVisualLayoutEditContext,
    path: Option<AuthoredUiNodePath>,
    suggested_fix: impl Into<String>,
) -> UiVisualLayoutDiagnostic {
    UiVisualLayoutDiagnostic::blocking(code, message, operation, context, path, suggested_fix)
}

#[derive(Debug, Clone)]
struct PendingDiagnostic {
    code: String,
    message: String,
    path: Option<AuthoredUiNodePath>,
    suggested_fix: String,
}

impl PendingDiagnostic {
    fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        path: Option<AuthoredUiNodePath>,
        suggested_fix: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path,
            suggested_fix: suggested_fix.into(),
        }
    }

    fn with_context(self, _context: &UiVisualLayoutEditContext) -> Self {
        self
    }

    fn into_diagnostic(
        self,
        operation: &UiVisualLayoutOperation,
        context: &UiVisualLayoutEditContext,
    ) -> UiVisualLayoutDiagnostic {
        diagnostic(
            self.code,
            self.message,
            operation,
            context,
            self.path,
            self.suggested_fix,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        UiScrollInputDefinition, UiScrollOwnershipDefinition, UiTemplateId, UiValueBinding,
    };

    #[test]
    fn visual_layout_move_preserves_stable_ids() {
        let template = layout_template();
        let operation = operation(
            "move.b",
            "root/b",
            "b",
            UiVisualLayoutEditKind::MoveNode {
                new_parent_path: path("root"),
                new_index: 0,
            },
        );

        let report = apply_visual_layout_operation(
            template,
            &operation,
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );

        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        let UiNodeDefinition::Column { children, .. } = report.template.root else {
            panic!("root should stay a column");
        };
        let ids: Vec<_> = children
            .iter()
            .map(|node| node.id().as_str().to_string())
            .collect();
        assert_eq!(ids, ["b", "a", "c"]);
        assert_eq!(
            report.diff.unwrap().changes[0].kind,
            UiVisualLayoutDiffChangeKind::Move
        );
    }

    #[test]
    fn visual_layout_reorder_preserves_stable_ids() {
        let template = layout_template();
        let operation = operation(
            "reorder.c",
            "root",
            "root",
            UiVisualLayoutEditKind::ReorderSibling {
                from_index: 2,
                to_index: 0,
                expected_child_id: "c".into(),
            },
        );

        let report = apply_visual_layout_operation(
            template,
            &operation,
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );

        assert!(!report.has_errors(), "{:?}", report.diagnostics);
        let UiNodeDefinition::Column { children, .. } = report.template.root else {
            panic!("root should stay a column");
        };
        let ids: Vec<_> = children
            .iter()
            .map(|node| node.id().as_str().to_string())
            .collect();
        assert_eq!(ids, ["c", "a", "b"]);
    }

    #[test]
    fn visual_layout_diff_text_is_deterministic() {
        let first = apply_visual_layout_operation(
            stack_template(),
            &operation(
                "axis.stack",
                "root/stack",
                "stack",
                UiVisualLayoutEditKind::ChangeStackAxis {
                    axis: UiAxisDefinition::Horizontal,
                },
            ),
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );
        let second = apply_visual_layout_operation(
            stack_template(),
            &operation(
                "axis.stack",
                "root/stack",
                "stack",
                UiVisualLayoutEditKind::ChangeStackAxis {
                    axis: UiAxisDefinition::Horizontal,
                },
            ),
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );

        assert!(!first.has_errors(), "{:?}", first.diagnostics);
        assert_eq!(first.diff, second.diff);
        let change = &first.diff.unwrap().changes[0];
        assert_eq!(change.before.as_deref(), Some("Vertical"));
        assert_eq!(change.after.as_deref(), Some("Horizontal"));
    }

    #[test]
    fn visual_layout_preview_only_edit_rejects_activation() {
        let mut operation = operation(
            "preview.axis",
            "root/stack",
            "stack",
            UiVisualLayoutEditKind::ChangeStackAxis {
                axis: UiAxisDefinition::Horizontal,
            },
        );
        operation.preview_only = true;

        let report = apply_visual_layout_operation(
            stack_template(),
            &operation,
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );

        assert!(report.has_errors());
        assert_eq!(
            report.diagnostics[0].code,
            "ui.visual_layout.preview_only_activation"
        );
        assert!(report.diff.is_none());
    }

    #[test]
    fn visual_layout_invalid_edit_reports_source_mapped_diagnostic() {
        let operation = operation(
            "bad.axis",
            "root/a",
            "a",
            UiVisualLayoutEditKind::ChangeStackAxis {
                axis: UiAxisDefinition::Horizontal,
            },
        );

        let report = apply_visual_layout_operation(
            layout_template(),
            &operation,
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );

        let diagnostic = report
            .diagnostics
            .first()
            .expect("invalid layout edit should produce diagnostic");
        assert_eq!(
            diagnostic.code,
            "ui.visual_layout.layout_feature.unsupported"
        );
        assert_eq!(
            diagnostic.path.as_ref().map(AuthoredUiNodePath::as_str),
            Some("root/a")
        );
        assert_eq!(diagnostic.target_profile.as_str(), "editor.workbench");
        assert_eq!(
            diagnostic.activation_impact,
            crate::UiVisualLayoutActivationImpact::BlocksActivation
        );
        assert!(!diagnostic.suggested_fix.is_empty());
    }

    #[test]
    fn visual_layout_target_profile_compatibility_is_fail_closed() {
        let operation = operation(
            "profile.axis",
            "root/stack",
            "stack",
            UiVisualLayoutEditKind::ChangeStackAxis {
                axis: UiAxisDefinition::Horizontal,
            },
        );
        let edit_context =
            UiVisualLayoutEditContext::with_supported_target_profiles(["game.runtime".into()]);

        let report = apply_visual_layout_operation(
            stack_template(),
            &operation,
            UiVisualLayoutActivationMode::Activate,
            &edit_context,
        );

        assert!(report.has_errors());
        assert_eq!(
            report.diagnostics[0].code,
            "ui.visual_layout.target_profile.unsupported"
        );
        assert_eq!(
            report.diagnostics[0].target_profile.as_str(),
            "editor.workbench"
        );
    }

    #[test]
    fn visual_layout_insert_rejects_duplicate_ids_inside_new_subtree() {
        let operation = operation(
            "insert.duplicate.subtree",
            "root",
            "root",
            UiVisualLayoutEditKind::InsertNode {
                index: 0,
                node: UiNodeDefinition::Column {
                    id: "new-root".into(),
                    children: vec![label("dup"), label("dup")],
                },
            },
        );

        let report = apply_visual_layout_operation(
            layout_template(),
            &operation,
            UiVisualLayoutActivationMode::Activate,
            &context(),
        );

        assert!(report.has_errors());
        assert_eq!(
            report.diagnostics[0].code,
            "ui.visual_layout.authored_id.duplicate"
        );
    }

    fn context() -> UiVisualLayoutEditContext {
        UiVisualLayoutEditContext::with_supported_target_profiles(["editor.workbench".into()])
    }

    fn operation(
        id: &str,
        target_path: &str,
        expected_node_id: &str,
        kind: UiVisualLayoutEditKind,
    ) -> UiVisualLayoutOperation {
        UiVisualLayoutOperation {
            id: id.into(),
            source_document: UiTemplateId::from("test.template"),
            target_path: path(target_path),
            expected_node_id: expected_node_id.into(),
            target_profile: "editor.workbench".into(),
            kind,
            source_location: None,
            preview_only: false,
        }
    }

    fn layout_template() -> AuthoredUiTemplate {
        AuthoredUiTemplate {
            id: "test.template".into(),
            root: UiNodeDefinition::Column {
                id: "root".into(),
                children: vec![label("a"), label("b"), label("c")],
            },
            templates: Vec::new(),
            menus: Vec::new(),
        }
    }

    fn stack_template() -> AuthoredUiTemplate {
        AuthoredUiTemplate {
            id: "test.template".into(),
            root: UiNodeDefinition::Column {
                id: "root".into(),
                children: vec![UiNodeDefinition::Stack {
                    id: "stack".into(),
                    axis: UiAxisDefinition::Vertical,
                    children: vec![label("a")],
                }],
            },
            templates: Vec::new(),
            menus: Vec::new(),
        }
    }

    fn label(id: &str) -> UiNodeDefinition {
        UiNodeDefinition::Label {
            id: id.into(),
            label: UiValueBinding::static_text(id),
            availability: None,
        }
    }

    #[allow(dead_code)]
    fn scroll(id: &str) -> UiNodeDefinition {
        UiNodeDefinition::Scroll {
            id: id.into(),
            axis: crate::UiScrollAxisDefinition::Vertical,
            input: UiScrollInputDefinition::default(),
            ownership: UiScrollOwnershipDefinition::default(),
            children: Vec::new(),
        }
    }

    fn path(value: &str) -> AuthoredUiNodePath {
        AuthoredUiNodePath(value.to_string())
    }
}
