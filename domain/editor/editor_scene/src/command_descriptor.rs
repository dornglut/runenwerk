//! File: domain/editor/editor_scene/src/command_descriptor.rs
//! Purpose: Domain-owned command descriptors for editor scene authoring.

use ::commands::{
    CommandContractId, CommandContractVersion, CommandDescriptor, CommandEffectHint,
    CommandMetadataEntry, CommandReversibilityHint, CommandSchemaRef, CommandTargetHint,
};
use ::schema::{
    SchemaCompatibility, SchemaConstraint, SchemaDescriptor, SchemaField, SchemaId,
    SchemaMetadataEntry, SchemaMetadataValue, SchemaShape, SchemaVersion,
};

pub const EDIT_COMPONENT_FIELD_COMMAND_ID: &str = "editor.scene.edit_component_field";
pub const EDIT_COMPONENT_FIELD_PARAMETERS_SCHEMA_ID: &str =
    "editor.scene.edit_component_field.parameters";

pub fn edit_component_field_command_descriptor() -> CommandDescriptor {
    CommandDescriptor::new(
        CommandContractId::from_static(EDIT_COMPONENT_FIELD_COMMAND_ID)
            .expect("static command contract id is valid"),
        CommandContractVersion::new(1).expect("command contract version one is valid"),
        CommandSchemaRef::new(
            SchemaId::from_static(EDIT_COMPONENT_FIELD_PARAMETERS_SCHEMA_ID)
                .expect("static schema id is valid"),
            SchemaVersion::new(1).expect("schema version one is valid"),
        ),
    )
    .with_display_name("Edit Component Field")
    .with_description("Request a scene component field edit through the editor scene command path.")
    .with_target_hint(CommandTargetHint::ComponentLike)
    .with_effect_hint(CommandEffectHint::DomainMutation)
    .with_reversibility_hint(CommandReversibilityHint::Reversible)
    .with_metadata_entry(
        CommandMetadataEntry::new("domain", "editor_scene").expect("static metadata key is valid"),
    )
    .expect("static metadata key is unique")
    .with_metadata_entry(
        CommandMetadataEntry::new("intent_variant", "SceneCommandIntent::EditComponentField")
            .expect("static metadata key is valid"),
    )
    .expect("static metadata key is unique")
}

pub fn edit_component_field_parameters_schema_descriptor() -> SchemaDescriptor {
    SchemaDescriptor::new(
        SchemaId::from_static(EDIT_COMPONENT_FIELD_PARAMETERS_SCHEMA_ID)
            .expect("static schema id is valid"),
        SchemaVersion::new(1).expect("schema version one is valid"),
        edit_component_field_parameters_shape(),
    )
    .with_display_name("Edit Component Field Parameters")
    .with_description("Parameter shape for editor scene component field edit requests.")
    .with_compatibility(SchemaCompatibility::Compatible)
    .with_metadata_entry(
        SchemaMetadataEntry::new("domain", SchemaMetadataValue::string("editor_scene"))
            .expect("static metadata key is valid"),
    )
    .expect("static metadata key is unique")
    .with_metadata_entry(
        SchemaMetadataEntry::new(
            "command_contract",
            SchemaMetadataValue::string(EDIT_COMPONENT_FIELD_COMMAND_ID),
        )
        .expect("static metadata key is valid"),
    )
    .expect("static metadata key is unique")
}

fn edit_component_field_parameters_shape() -> SchemaShape {
    SchemaShape::object([
        editor_id_field("entity", "EntityId")
            .with_display_name("Entity")
            .with_description("Editor scene entity id whose component field is edited."),
        editor_id_field("component_type", "ComponentTypeId")
            .with_display_name("Component Type")
            .with_description("Editor component type id for the component being edited."),
        required_field(
            "path",
            SchemaShape::opaque("InspectorPath").expect("static opaque kind is valid"),
        )
        .with_display_name("Inspector Path")
        .with_description("Inspector field path within the component."),
        required_field(
            "value",
            SchemaShape::opaque("InspectorEditValue").expect("static opaque kind is valid"),
        )
        .with_display_name("Inspector Edit Value")
        .with_description("Portable inspector edit value to write at the path."),
    ])
    .expect("static field names are unique")
}

fn required_field(name: &'static str, shape: SchemaShape) -> SchemaField {
    SchemaField::new(name, shape)
        .expect("static field name is valid")
        .with_constraint(SchemaConstraint::required_presence())
}

fn editor_id_field(name: &'static str, rust_type: &'static str) -> SchemaField {
    required_field(name, SchemaShape::integer())
        .with_metadata_entry(
            SchemaMetadataEntry::new("rust_type", SchemaMetadataValue::string(rust_type))
                .expect("static metadata key is valid"),
        )
        .expect("static metadata key is unique")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_edit_component_field_descriptor_has_stable_id() {
        let descriptor = edit_component_field_command_descriptor();

        assert_eq!(descriptor.id().as_str(), EDIT_COMPONENT_FIELD_COMMAND_ID);
        assert_eq!(descriptor.version().value(), 1);
    }

    #[test]
    fn scene_edit_component_field_descriptor_references_parameter_schema() {
        let descriptor = edit_component_field_command_descriptor();

        assert_eq!(
            descriptor.parameter_schema().schema_id().as_str(),
            EDIT_COMPONENT_FIELD_PARAMETERS_SCHEMA_ID
        );
        assert_eq!(descriptor.parameter_schema().schema_version().value(), 1);
    }

    #[test]
    fn scene_edit_component_field_descriptor_does_not_execute_command() {
        let descriptor = edit_component_field_command_descriptor();

        assert_eq!(descriptor.effect_hint(), CommandEffectHint::DomainMutation);
        assert!(descriptor.result_schema().is_none());
    }

    #[test]
    fn scene_edit_component_field_parameter_schema_has_stable_id() {
        let descriptor = edit_component_field_parameters_schema_descriptor();

        assert_eq!(
            descriptor.id().as_str(),
            EDIT_COMPONENT_FIELD_PARAMETERS_SCHEMA_ID
        );
        assert_eq!(descriptor.version().value(), 1);
    }

    #[test]
    fn scene_edit_component_field_parameter_schema_preserves_field_order() {
        let descriptor = edit_component_field_parameters_schema_descriptor();
        let fields = descriptor
            .root_shape()
            .as_object_fields()
            .expect("parameter schema root should be an object");

        assert_eq!(fields[0].name(), "entity");
        assert_eq!(fields[1].name(), "component_type");
        assert_eq!(fields[2].name(), "path");
        assert_eq!(fields[3].name(), "value");
    }
}
