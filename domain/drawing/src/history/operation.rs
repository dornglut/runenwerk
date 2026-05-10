//! File: domain/drawing/src/history/operation.rs
//! Purpose: Domain-owned drawing commands and operation DTOs.

use graph::{NodeDefinition, NodeId};
use ratification::RatificationIssue;

use crate::{
    AdjustmentDescriptor, BlendMode, BrushId, ColorRgba, DrawingCompositeNode, DrawingDocument,
    DrawingDocumentRevision, DrawingIssueCode, DrawingIssueSubject, DrawingOperationId,
    DrawingRatificationReport, GroupIsolationPolicy, GroupNode, LayerStackEntry,
    LayerStackEntryContent, LayerStackEntryId, PaintTarget, StrokeId, StrokeRecord, StrokeSample,
    ratify_drawing_document,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PendingStrokeRecord {
    pub stroke_id: StrokeId,
    pub target: PaintTarget,
    pub brush_id: BrushId,
    pub color: ColorRgba,
    pub samples: Vec<StrokeSample>,
    pub source_revision: DrawingDocumentRevision,
}

impl PendingStrokeRecord {
    pub fn new(
        stroke_id: StrokeId,
        target: PaintTarget,
        brush_id: BrushId,
        color: ColorRgba,
        source_revision: DrawingDocumentRevision,
    ) -> Self {
        Self {
            stroke_id,
            target,
            brush_id,
            color,
            samples: Vec::new(),
            source_revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DrawingCommand {
    BeginStroke {
        stroke_id: StrokeId,
        target: PaintTarget,
        brush_id: BrushId,
        color: ColorRgba,
    },
    AppendStrokeSample {
        stroke_id: StrokeId,
        sample: StrokeSample,
    },
    CommitStroke {
        stroke_id: StrokeId,
    },
    CreateStackEntry {
        entry: LayerStackEntry,
    },
    RenameStackEntry {
        entry_id: LayerStackEntryId,
        name: String,
    },
    ReorderStackEntry {
        entry_id: LayerStackEntryId,
        new_index: usize,
    },
    ShowStackEntry {
        entry_id: LayerStackEntryId,
        visible: bool,
    },
    RemoveStackEntry {
        entry_id: LayerStackEntryId,
    },
    CreateIsolatedGroup {
        group_node_id: NodeId,
        graph_node: NodeDefinition,
        group: GroupNode,
        entry: LayerStackEntry,
    },
    AttachMask {
        entry_id: LayerStackEntryId,
        mask_node: NodeId,
    },
    RemoveMask {
        entry_id: LayerStackEntryId,
    },
    AddAlphaClip {
        entry_id: LayerStackEntryId,
    },
    RemoveAlphaClip {
        entry_id: LayerStackEntryId,
    },
    SetLayerOpacity {
        entry_id: LayerStackEntryId,
        opacity: f32,
    },
    SetLayerBlendMode {
        entry_id: LayerStackEntryId,
        blend_mode: BlendMode,
    },
    SetAdjustmentDescriptor {
        node_id: NodeId,
        descriptor: AdjustmentDescriptor,
    },
    SelectCompositeOutput {
        output_id: crate::CompositeOutputId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingCommandOutcome {
    Updated,
    StrokeBegun(StrokeId),
    StrokeCommitted(StrokeId),
    StackEntryCreated(LayerStackEntryId),
    StackEntryRemoved(LayerStackEntryId),
    GroupCreated(NodeId),
    OutputSelected(crate::CompositeOutputId),
}

impl DrawingCommand {
    pub fn apply_to(
        &self,
        document: &mut DrawingDocument,
    ) -> Result<DrawingCommandOutcome, DrawingRatificationReport> {
        let mut candidate = document.clone();
        let outcome = self.apply_unratified(&mut candidate)?;
        let report = ratify_drawing_document(&candidate);
        if report.has_blocking_issues() {
            return Err(report);
        }
        *document = candidate;
        Ok(outcome)
    }

    fn apply_unratified(
        &self,
        document: &mut DrawingDocument,
    ) -> Result<DrawingCommandOutcome, DrawingRatificationReport> {
        let outcome = match self {
            Self::BeginStroke {
                stroke_id,
                target,
                brush_id,
                color,
            } => {
                if document
                    .strokes
                    .iter()
                    .any(|stroke| stroke.stroke_id == *stroke_id)
                    || document
                        .pending_strokes
                        .iter()
                        .any(|stroke| stroke.stroke_id == *stroke_id)
                {
                    return Err(command_error(
                        DrawingIssueCode::DuplicateStrokeId,
                        DrawingIssueSubject::Stroke(*stroke_id),
                        "stroke id is already present",
                    ));
                }
                document.pending_strokes.push(PendingStrokeRecord::new(
                    *stroke_id,
                    *target,
                    *brush_id,
                    *color,
                    document.revision,
                ));
                DrawingCommandOutcome::StrokeBegun(*stroke_id)
            }
            Self::AppendStrokeSample { stroke_id, sample } => {
                let pending = document
                    .pending_strokes
                    .iter_mut()
                    .find(|stroke| stroke.stroke_id == *stroke_id)
                    .ok_or_else(|| {
                        command_error(
                            DrawingIssueCode::CommandTargetMissing,
                            DrawingIssueSubject::Stroke(*stroke_id),
                            "pending stroke was not found",
                        )
                    })?;
                pending.samples.push(*sample);
                DrawingCommandOutcome::Updated
            }
            Self::CommitStroke { stroke_id } => {
                let Some(index) = document
                    .pending_strokes
                    .iter()
                    .position(|stroke| stroke.stroke_id == *stroke_id)
                else {
                    return Err(command_error(
                        DrawingIssueCode::CommandTargetMissing,
                        DrawingIssueSubject::Stroke(*stroke_id),
                        "pending stroke was not found",
                    ));
                };
                let pending = document.pending_strokes.remove(index);
                let Some(stroke) = StrokeRecord::new(
                    pending.stroke_id,
                    pending.target,
                    pending.brush_id,
                    pending.color,
                    pending.samples,
                    document.revision,
                ) else {
                    return Err(command_error(
                        DrawingIssueCode::EmptyCommittedStroke,
                        DrawingIssueSubject::Stroke(*stroke_id),
                        "committed stroke must contain at least one valid sample",
                    ));
                };
                document.strokes.push(stroke);
                DrawingCommandOutcome::StrokeCommitted(*stroke_id)
            }
            Self::CreateStackEntry { entry } => {
                let root = root_stack_mut(document)?;
                root.entries.push(entry.clone());
                DrawingCommandOutcome::StackEntryCreated(entry.entry_id)
            }
            Self::RenameStackEntry { entry_id, name } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.name = name.clone();
                DrawingCommandOutcome::Updated
            }
            Self::ReorderStackEntry {
                entry_id,
                new_index,
            } => {
                let root = root_stack_mut(document)?;
                let Some(index) = root
                    .entries
                    .iter()
                    .position(|entry| entry.entry_id == *entry_id)
                else {
                    return Err(command_error(
                        DrawingIssueCode::CommandTargetMissing,
                        DrawingIssueSubject::LayerStackEntry(*entry_id),
                        "stack entry was not found",
                    ));
                };
                if *new_index >= root.entries.len() {
                    return Err(command_error(
                        DrawingIssueCode::InvalidLayerOrdering,
                        DrawingIssueSubject::LayerStackEntry(*entry_id),
                        "new stack index is outside the stack",
                    ));
                }
                let entry = root.entries.remove(index);
                root.entries.insert(*new_index, entry);
                DrawingCommandOutcome::Updated
            }
            Self::ShowStackEntry { entry_id, visible } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.visible = *visible;
                DrawingCommandOutcome::Updated
            }
            Self::RemoveStackEntry { entry_id } => {
                let root = root_stack_mut(document)?;
                let before = root.entries.len();
                root.entries.retain(|entry| entry.entry_id != *entry_id);
                if root.entries.len() == before {
                    return Err(command_error(
                        DrawingIssueCode::CommandTargetMissing,
                        DrawingIssueSubject::LayerStackEntry(*entry_id),
                        "stack entry was not found",
                    ));
                }
                DrawingCommandOutcome::StackEntryRemoved(*entry_id)
            }
            Self::CreateIsolatedGroup {
                group_node_id,
                graph_node,
                group,
                entry,
            } => {
                if group.isolation_policy != GroupIsolationPolicy::Isolated {
                    return Err(command_error(
                        DrawingIssueCode::PassThroughGroupUnsupported,
                        DrawingIssueSubject::Node(*group_node_id),
                        "only isolated groups are supported in this phase",
                    ));
                }
                document
                    .composition
                    .nodes
                    .insert(*group_node_id, DrawingCompositeNode::Group(group.clone()));
                document.composition.graph.nodes.push(graph_node.clone());
                let root = root_stack_mut(document)?;
                root.entries.push(entry.clone());
                DrawingCommandOutcome::GroupCreated(*group_node_id)
            }
            Self::AttachMask {
                entry_id,
                mask_node,
            } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.mask_node = Some(*mask_node);
                DrawingCommandOutcome::Updated
            }
            Self::RemoveMask { entry_id } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.mask_node = None;
                DrawingCommandOutcome::Updated
            }
            Self::AddAlphaClip { entry_id } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.clip_to_below = true;
                DrawingCommandOutcome::Updated
            }
            Self::RemoveAlphaClip { entry_id } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.clip_to_below = false;
                DrawingCommandOutcome::Updated
            }
            Self::SetLayerOpacity { entry_id, opacity } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.opacity = *opacity;
                DrawingCommandOutcome::Updated
            }
            Self::SetLayerBlendMode {
                entry_id,
                blend_mode,
            } => {
                let entry = stack_entry_mut(document, *entry_id)?;
                entry.blend_mode = *blend_mode;
                DrawingCommandOutcome::Updated
            }
            Self::SetAdjustmentDescriptor {
                node_id,
                descriptor,
            } => {
                let Some(DrawingCompositeNode::Adjustment(adjustment)) =
                    document.composition.nodes.get_mut(node_id)
                else {
                    return Err(command_error(
                        DrawingIssueCode::CommandTargetMissing,
                        DrawingIssueSubject::Node(*node_id),
                        "adjustment node was not found",
                    ));
                };
                adjustment.descriptor = descriptor.clone();
                DrawingCommandOutcome::Updated
            }
            Self::SelectCompositeOutput { output_id } => {
                document.composition.active_output = Some(*output_id);
                DrawingCommandOutcome::OutputSelected(*output_id)
            }
        };

        document.bump_revision();
        Ok(outcome)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingTransaction {
    pub label: String,
    pub commands: Vec<DrawingCommand>,
}

impl DrawingTransaction {
    pub fn new(
        label: impl Into<String>,
        commands: impl IntoIterator<Item = DrawingCommand>,
    ) -> Self {
        Self {
            label: label.into(),
            commands: commands.into_iter().collect(),
        }
    }

    pub fn apply_to(
        &self,
        document: &mut DrawingDocument,
    ) -> Result<Vec<DrawingCommandOutcome>, DrawingRatificationReport> {
        let mut candidate = document.clone();
        let mut outcomes = Vec::with_capacity(self.commands.len());
        for command in &self.commands {
            outcomes.push(command.apply_unratified(&mut candidate)?);
        }
        let report = ratify_drawing_document(&candidate);
        if report.has_blocking_issues() {
            return Err(report);
        }
        *document = candidate;
        Ok(outcomes)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingOperation {
    pub operation_id: DrawingOperationId,
    pub document_revision: DrawingDocumentRevision,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawingRecoveryState {
    pub last_accepted_revision: DrawingDocumentRevision,
    pub pending_operation_count: u32,
    pub has_last_good_tiles: bool,
}

fn root_stack_mut(
    document: &mut DrawingDocument,
) -> Result<&mut crate::LayerStackNode, DrawingRatificationReport> {
    document.composition.root_stack_mut().ok_or_else(|| {
        command_error(
            DrawingIssueCode::MissingRootLayerStack,
            DrawingIssueSubject::CompositionGraph,
            "root layer stack node was not found",
        )
    })
}

fn stack_entry_mut(
    document: &mut DrawingDocument,
    entry_id: LayerStackEntryId,
) -> Result<&mut LayerStackEntry, DrawingRatificationReport> {
    let root = root_stack_mut(document)?;
    root.entries
        .iter_mut()
        .find(|entry| entry.entry_id == entry_id)
        .ok_or_else(|| {
            command_error(
                DrawingIssueCode::CommandTargetMissing,
                DrawingIssueSubject::LayerStackEntry(entry_id),
                "stack entry was not found",
            )
        })
}

fn command_error(
    code: DrawingIssueCode,
    subject: DrawingIssueSubject,
    message: impl Into<String>,
) -> DrawingRatificationReport {
    DrawingRatificationReport::from_issue(RatificationIssue::error(code, subject, message))
}

#[allow(dead_code)]
fn _entry_content_node(entry: &LayerStackEntry) -> NodeId {
    match entry.content {
        LayerStackEntryContent::PaintSource(node)
        | LayerStackEntryContent::Group(node)
        | LayerStackEntryContent::ReferenceImage(node)
        | LayerStackEntryContent::Adjustment(node) => node,
    }
}
