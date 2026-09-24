//! File: domain/editor/editor_scene/src/sdf_authoring/commands.rs
//! Purpose: Domain command intents for authored SDF operation documents.

use crate::{
    SdfLayerMoveDirection, SdfOperationDocument, SdfOperationDocumentError, SdfOperationEntryId,
    SdfOperationLayerId, SdfPrimitiveSpec,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SdfOperationCommandIntent {
    AddLayer {
        stable_name: String,
        display_name: String,
    },
    SetLayerEnabled {
        layer_id: SdfOperationLayerId,
        enabled: bool,
    },
    MoveLayer {
        layer_id: SdfOperationLayerId,
        direction: SdfLayerMoveDirection,
    },
    AddPrimitiveOperation {
        layer_id: SdfOperationLayerId,
        display_name: String,
        primitive: SdfPrimitiveSpec,
        material_channel: u16,
    },
    SetOperationEnabled {
        operation_id: SdfOperationEntryId,
        enabled: bool,
    },
    UpdateOperationPrimitive {
        operation_id: SdfOperationEntryId,
        primitive: SdfPrimitiveSpec,
    },
}

impl SdfOperationCommandIntent {
    pub fn apply_to(
        self,
        document: &mut SdfOperationDocument,
    ) -> Result<SdfOperationCommandOutcome, SdfOperationDocumentError> {
        match self {
            Self::AddLayer {
                stable_name,
                display_name,
            } => Ok(SdfOperationCommandOutcome::Layer(
                document.add_layer(stable_name, display_name),
            )),
            Self::SetLayerEnabled { layer_id, enabled } => {
                document.set_layer_enabled(layer_id, enabled)?;
                Ok(SdfOperationCommandOutcome::Updated)
            }
            Self::MoveLayer {
                layer_id,
                direction,
            } => {
                document.move_layer(layer_id, direction)?;
                Ok(SdfOperationCommandOutcome::Updated)
            }
            Self::AddPrimitiveOperation {
                layer_id,
                display_name,
                primitive,
                material_channel,
            } => Ok(SdfOperationCommandOutcome::Operation(
                document.add_operation(layer_id, display_name, primitive, material_channel)?,
            )),
            Self::SetOperationEnabled {
                operation_id,
                enabled,
            } => {
                document.set_operation_enabled(operation_id, enabled)?;
                Ok(SdfOperationCommandOutcome::Updated)
            }
            Self::UpdateOperationPrimitive {
                operation_id,
                primitive,
            } => {
                document.update_operation_primitive(operation_id, primitive)?;
                Ok(SdfOperationCommandOutcome::Updated)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfOperationCommandOutcome {
    Layer(SdfOperationLayerId),
    Operation(SdfOperationEntryId),
    Updated,
}
