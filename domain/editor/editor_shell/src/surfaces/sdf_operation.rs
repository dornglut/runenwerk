//! File: domain/editor/editor_shell/src/surfaces/sdf_operation.rs
//! Purpose: SDF operation surface workflow contracts.

use editor_scene::{
    SdfGraphCommandIntent, SdfOperationCommandIntent, SdfOperationEntryId, SdfOperationLayerId,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SdfOperationSurfaceAction {
    SelectLayer { layer_id: SdfOperationLayerId },
    SelectOperation { operation_id: SdfOperationEntryId },
    ApplyCommand { intent: SdfOperationCommandIntent },
    ApplyGraphCommand { intent: SdfGraphCommandIntent },
    LowerGraphToOperationDocument,
    CommitOperationWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdfOperationSessionMutation {
    SelectLayer { layer_id: SdfOperationLayerId },
    SelectOperation { operation_id: SdfOperationEntryId },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SdfOperationDomainMutation {
    ApplyCommand { intent: SdfOperationCommandIntent },
    ApplyGraphCommand { intent: SdfGraphCommandIntent },
    LowerGraphToOperationDocument,
    CommitOperationWindow,
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_scene::{SdfBooleanIntent, SdfPrimitiveKind, SdfPrimitiveSpec};

    #[test]
    fn sdf_operation_surface_contracts_carry_typed_command_intents() {
        let layer_id = SdfOperationLayerId::new(3);
        let intent = SdfOperationCommandIntent::AddPrimitiveOperation {
            layer_id,
            display_name: "Sphere Add".to_string(),
            primitive: SdfPrimitiveSpec::new(SdfPrimitiveKind::Sphere, SdfBooleanIntent::Add),
            material_channel: 1,
        };

        let action = SdfOperationSurfaceAction::ApplyCommand {
            intent: intent.clone(),
        };
        let mutation = SdfOperationDomainMutation::ApplyCommand { intent };

        assert!(matches!(
            action,
            SdfOperationSurfaceAction::ApplyCommand { .. }
        ));
        assert!(matches!(
            mutation,
            SdfOperationDomainMutation::ApplyCommand { .. }
        ));

        let graph_action = SdfOperationSurfaceAction::ApplyGraphCommand {
            intent: SdfGraphCommandIntent::AddOutputNode {
                display_name: "Output".to_string(),
            },
        };
        let graph_mutation = SdfOperationDomainMutation::LowerGraphToOperationDocument;

        assert!(matches!(
            graph_action,
            SdfOperationSurfaceAction::ApplyGraphCommand { .. }
        ));
        assert!(matches!(
            graph_mutation,
            SdfOperationDomainMutation::LowerGraphToOperationDocument
        ));
    }
}
