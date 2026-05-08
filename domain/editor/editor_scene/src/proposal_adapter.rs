//! File: domain/editor/editor_scene/src/proposal_adapter.rs
//! Purpose: Narrow adapters from command proposals into editor scene intents.

use ::commands::CommandProposal;
use ::schema::{SchemaValue, SchemaValueObjectField};
use editor_core::{ComponentTypeId, EntityId};
use editor_inspector::{InspectorEditValue, InspectorPath};

use crate::{
    EDIT_COMPONENT_FIELD_COMMAND_ID, SceneCommandIntent, edit_component_field_command_descriptor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditComponentFieldProposalError {
    ContractMismatch,
    UnsupportedContractVersion,
    ParametersNotObject,
    MissingField(&'static str),
    InvalidEntity,
    InvalidComponentType,
    InvalidPath,
    InvalidPathSegment,
    InvalidValue,
}

pub fn edit_component_field_proposal_to_intent(
    proposal: &CommandProposal,
) -> Result<SceneCommandIntent, EditComponentFieldProposalError> {
    let descriptor = edit_component_field_command_descriptor();
    if proposal.contract().id().as_str() != EDIT_COMPONENT_FIELD_COMMAND_ID {
        return Err(EditComponentFieldProposalError::ContractMismatch);
    }

    if proposal.contract().version() != descriptor.version() {
        return Err(EditComponentFieldProposalError::UnsupportedContractVersion);
    }

    let fields = proposal
        .parameters()
        .as_object()
        .ok_or(EditComponentFieldProposalError::ParametersNotObject)?;

    let entity = EntityId(read_u64_field(
        fields,
        "entity",
        EditComponentFieldProposalError::InvalidEntity,
    )?);
    let component_type = ComponentTypeId(read_u64_field(
        fields,
        "component_type",
        EditComponentFieldProposalError::InvalidComponentType,
    )?);
    let path = read_path(required_field(fields, "path")?)?;
    let value = read_edit_value(required_field(fields, "value")?)?;

    Ok(SceneCommandIntent::EditComponentField {
        entity,
        component_type,
        path,
        value,
    })
}

fn required_field<'a>(
    fields: &'a [SchemaValueObjectField],
    name: &'static str,
) -> Result<&'a SchemaValue, EditComponentFieldProposalError> {
    fields
        .iter()
        .find(|field| field.key() == name)
        .map(SchemaValueObjectField::value)
        .ok_or(EditComponentFieldProposalError::MissingField(name))
}

fn read_u64_field(
    fields: &[SchemaValueObjectField],
    name: &'static str,
    invalid: EditComponentFieldProposalError,
) -> Result<u64, EditComponentFieldProposalError> {
    let value = required_field(fields, name)?;
    read_u64(value).ok_or(invalid)
}

fn read_u64(value: &SchemaValue) -> Option<u64> {
    if let Some(value) = value.as_unsigned_integer() {
        return Some(value);
    }

    value
        .as_integer()
        .and_then(|value| u64::try_from(value).ok())
}

fn read_path(value: &SchemaValue) -> Result<InspectorPath, EditComponentFieldProposalError> {
    let segments = value
        .as_list()
        .ok_or(EditComponentFieldProposalError::InvalidPath)?;
    let mut path = InspectorPath::root();

    for segment in segments {
        let fields = segment
            .as_object()
            .ok_or(EditComponentFieldProposalError::InvalidPathSegment)?;
        let field = find_field(fields, "field");
        let index = find_field(fields, "index");

        match (field, index) {
            (Some(field), None) => {
                let name = field
                    .as_string()
                    .ok_or(EditComponentFieldProposalError::InvalidPathSegment)?;
                path = path.child_field(name.to_string());
            }
            (None, Some(index)) => {
                let index =
                    read_u64(index).ok_or(EditComponentFieldProposalError::InvalidPathSegment)?;
                let index = usize::try_from(index)
                    .map_err(|_| EditComponentFieldProposalError::InvalidPathSegment)?;
                path = path.child_index(index);
            }
            _ => return Err(EditComponentFieldProposalError::InvalidPathSegment),
        }
    }

    Ok(path)
}

fn read_edit_value(
    value: &SchemaValue,
) -> Result<InspectorEditValue, EditComponentFieldProposalError> {
    let fields = value
        .as_object()
        .ok_or(EditComponentFieldProposalError::InvalidValue)?;

    let mut parsed = None;

    for field in fields {
        let value = match field.key() {
            "bool" => field.value().as_bool().map(InspectorEditValue::Bool),
            "integer" => field.value().as_integer().map(InspectorEditValue::Integer),
            "float" => field.value().as_float().map(InspectorEditValue::Float),
            "text" => field
                .value()
                .as_string()
                .map(|value| InspectorEditValue::Text(value.to_string())),
            "enum" => field
                .value()
                .as_enum_symbol()
                .map(|value| InspectorEditValue::EnumSymbol(value.to_string())),
            _ => None,
        };

        if let Some(value) = value {
            if parsed.is_some() {
                return Err(EditComponentFieldProposalError::InvalidValue);
            }
            parsed = Some(value);
        }
    }

    parsed.ok_or(EditComponentFieldProposalError::InvalidValue)
}

fn find_field<'a>(fields: &'a [SchemaValueObjectField], name: &str) -> Option<&'a SchemaValue> {
    fields
        .iter()
        .find(|field| field.key() == name)
        .map(SchemaValueObjectField::value)
}

#[cfg(test)]
mod tests {
    use ::commands::{CommandContractId, CommandContractRef, CommandContractVersion};
    use ::schema::{SchemaValue, SchemaValueObjectField};
    use editor_core::{ComponentTypeId, EntityId};
    use editor_inspector::{InspectorEditValue, InspectorPathSegment};

    use super::*;

    fn proposal() -> CommandProposal {
        CommandProposal::new(
            CommandContractRef::new(
                CommandContractId::from_static(EDIT_COMPONENT_FIELD_COMMAND_ID).unwrap(),
                CommandContractVersion::new(1).unwrap(),
            ),
            parameters(),
        )
    }

    fn parameters() -> SchemaValue {
        SchemaValue::object([
            SchemaValueObjectField::new("entity", SchemaValue::unsigned_integer(7)).unwrap(),
            SchemaValueObjectField::new("component_type", SchemaValue::unsigned_integer(11))
                .unwrap(),
            SchemaValueObjectField::new(
                "path",
                SchemaValue::list([
                    SchemaValue::object([SchemaValueObjectField::new(
                        "field",
                        SchemaValue::string("transform"),
                    )
                    .unwrap()])
                    .unwrap(),
                    SchemaValue::object([SchemaValueObjectField::new(
                        "field",
                        SchemaValue::string("translation"),
                    )
                    .unwrap()])
                    .unwrap(),
                    SchemaValue::object([SchemaValueObjectField::new(
                        "index",
                        SchemaValue::unsigned_integer(1),
                    )
                    .unwrap()])
                    .unwrap(),
                ]),
            )
            .unwrap(),
            SchemaValueObjectField::new(
                "value",
                SchemaValue::object([SchemaValueObjectField::new(
                    "float",
                    SchemaValue::float(9.25).unwrap(),
                )
                .unwrap()])
                .unwrap(),
            )
            .unwrap(),
        ])
        .unwrap()
    }

    #[test]
    fn scene_edit_component_field_proposal_maps_to_intent_when_supported() {
        let intent = edit_component_field_proposal_to_intent(&proposal())
            .expect("supported proposal should map to intent");

        assert_eq!(
            intent,
            SceneCommandIntent::EditComponentField {
                entity: EntityId(7),
                component_type: ComponentTypeId(11),
                path: editor_inspector::InspectorPath::root()
                    .child_field("transform")
                    .child_field("translation")
                    .child_index(1),
                value: InspectorEditValue::Float(9.25),
            }
        );
    }

    #[test]
    fn scene_edit_component_field_proposal_rejects_descriptor_mismatch() {
        let proposal = CommandProposal::new(
            CommandContractRef::new(
                CommandContractId::from_static("editor.scene.rename_entity").unwrap(),
                CommandContractVersion::new(1).unwrap(),
            ),
            parameters(),
        );

        let error = edit_component_field_proposal_to_intent(&proposal)
            .expect_err("mismatched contract should be rejected");

        assert_eq!(error, EditComponentFieldProposalError::ContractMismatch);
    }

    #[test]
    fn scene_edit_component_field_proposal_rejects_unsupported_parameter_shape() {
        let proposal = CommandProposal::new(
            CommandContractRef::new(
                CommandContractId::from_static(EDIT_COMPONENT_FIELD_COMMAND_ID).unwrap(),
                CommandContractVersion::new(1).unwrap(),
            ),
            SchemaValue::string("not an object"),
        );

        let error = edit_component_field_proposal_to_intent(&proposal)
            .expect_err("unsupported parameters should be rejected");

        assert_eq!(error, EditComponentFieldProposalError::ParametersNotObject);
    }

    #[test]
    fn proposal_mapping_does_not_bypass_ratification_or_execution() {
        let intent = edit_component_field_proposal_to_intent(&proposal())
            .expect("mapping should only construct a domain intent");

        assert!(matches!(
            intent,
            SceneCommandIntent::EditComponentField {
                path,
                value: InspectorEditValue::Float(_),
                ..
            } if path.segments()[0] == InspectorPathSegment::Field("transform".to_string())
        ));
    }

    #[test]
    fn scene_edit_component_field_proposal_supports_enum_symbol_values() {
        let value = read_edit_value(
            &SchemaValue::object([SchemaValueObjectField::new(
                "enum",
                SchemaValue::enum_symbol("Linear").unwrap(),
            )
            .unwrap()])
            .unwrap(),
        )
        .expect("enum proposal value should map to inspector enum edit value");

        assert_eq!(value, InspectorEditValue::EnumSymbol("Linear".to_string()));
    }
}
