//! File: domain/scene/src/schema.rs
//! Purpose: Domain-owned schema descriptors for scene value contracts.

use ::schema::{
    SchemaCompatibility, SchemaConstraint, SchemaDescriptor, SchemaField, SchemaId,
    SchemaMetadataEntry, SchemaMetadataValue, SchemaShape, SchemaVersion,
};

pub const LOCAL_TRANSFORM_SCHEMA_ID: &str = "scene.local_transform";

pub fn local_transform_schema_descriptor() -> SchemaDescriptor {
    SchemaDescriptor::new(
        SchemaId::from_static(LOCAL_TRANSFORM_SCHEMA_ID).expect("static schema id is valid"),
        SchemaVersion::new(1).expect("schema version one is valid"),
        local_transform_shape(),
    )
    .with_display_name("Local Transform")
    .with_description("Scene-local translation, rotation, and scale shape.")
    .with_compatibility(SchemaCompatibility::Compatible)
    .with_metadata_entry(
        SchemaMetadataEntry::new("domain", SchemaMetadataValue::string("scene"))
            .expect("static metadata key is valid"),
    )
    .expect("static metadata key is unique")
    .with_metadata_entry(
        SchemaMetadataEntry::new("rust_type", SchemaMetadataValue::string("LocalTransform"))
            .expect("static metadata key is valid"),
    )
    .expect("static metadata key is unique")
}

fn local_transform_shape() -> SchemaShape {
    SchemaShape::object([
        SchemaField::new("translation", vec3_shape("meters"))
            .expect("static field name is valid")
            .with_display_name("Translation")
            .with_constraint(SchemaConstraint::required_presence())
            .with_constraint(
                SchemaConstraint::display_unit_label("meters").expect("static unit label is valid"),
            ),
        SchemaField::new("rotation", quat_shape()).expect("static field name is valid"),
        SchemaField::new("scale", vec3_shape("scale")).expect("static field name is valid"),
    ])
    .expect("static field names are unique")
}

fn vec3_shape(unit_label: &'static str) -> SchemaShape {
    SchemaShape::object([
        axis_field("x", unit_label),
        axis_field("y", unit_label),
        axis_field("z", unit_label),
    ])
    .expect("static Vec3 field names are unique")
}

fn quat_shape() -> SchemaShape {
    SchemaShape::object([
        axis_field("x", "quaternion component"),
        axis_field("y", "quaternion component"),
        axis_field("z", "quaternion component"),
        axis_field("w", "quaternion component"),
    ])
    .expect("static quaternion field names are unique")
}

fn axis_field(name: &'static str, unit_label: &'static str) -> SchemaField {
    SchemaField::new(name, SchemaShape::float())
        .expect("static field name is valid")
        .with_constraint(
            SchemaConstraint::display_unit_label(unit_label).expect("static unit label is valid"),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_has_stable_schema_id() {
        let descriptor = local_transform_schema_descriptor();

        assert_eq!(descriptor.id().as_str(), LOCAL_TRANSFORM_SCHEMA_ID);
    }

    #[test]
    fn descriptor_has_version_one() {
        let descriptor = local_transform_schema_descriptor();

        assert_eq!(descriptor.version().value(), 1);
    }

    #[test]
    fn descriptor_preserves_field_order() {
        let descriptor = local_transform_schema_descriptor();
        let fields = descriptor
            .root_shape()
            .as_object_fields()
            .expect("local transform descriptor root should be an object");

        assert_eq!(fields[0].name(), "translation");
        assert_eq!(fields[1].name(), "rotation");
        assert_eq!(fields[2].name(), "scale");
    }

    #[test]
    fn descriptor_contains_expected_fields_for_local_transform() {
        let descriptor = local_transform_schema_descriptor();
        let fields = descriptor
            .root_shape()
            .as_object_fields()
            .expect("local transform descriptor root should be an object");

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].display_name(), Some("Translation"));
        assert_eq!(
            fields[0].constraints()[0],
            SchemaConstraint::required_presence()
        );
        assert_eq!(
            fields[0].constraints()[1],
            SchemaConstraint::display_unit_label("meters").unwrap()
        );
    }

    #[test]
    fn descriptor_preserves_nested_vec3_field_order() {
        let descriptor = local_transform_schema_descriptor();
        let fields = descriptor
            .root_shape()
            .as_object_fields()
            .expect("local transform descriptor root should be an object");
        let translation_fields = fields[0]
            .shape()
            .as_object_fields()
            .expect("translation should be a Vec3 object shape");

        assert_eq!(translation_fields[0].name(), "x");
        assert_eq!(translation_fields[1].name(), "y");
        assert_eq!(translation_fields[2].name(), "z");
    }

    #[test]
    fn descriptor_preserves_nested_quaternion_field_order() {
        let descriptor = local_transform_schema_descriptor();
        let fields = descriptor
            .root_shape()
            .as_object_fields()
            .expect("local transform descriptor root should be an object");
        let rotation_fields = fields[1]
            .shape()
            .as_object_fields()
            .expect("rotation should be a quaternion object shape");

        assert_eq!(rotation_fields[0].name(), "x");
        assert_eq!(rotation_fields[1].name(), "y");
        assert_eq!(rotation_fields[2].name(), "z");
        assert_eq!(rotation_fields[3].name(), "w");
    }

    #[test]
    fn descriptor_preserves_metadata_order() {
        let descriptor = local_transform_schema_descriptor();
        let metadata = descriptor.metadata().entries();

        assert_eq!(metadata[0].key(), "domain");
        assert_eq!(metadata[1].key(), "rust_type");
    }

    #[test]
    fn descriptor_has_no_global_registry_side_effect() {
        let first = local_transform_schema_descriptor();
        let second = local_transform_schema_descriptor();

        assert_eq!(first, second);
    }

    #[test]
    fn descriptor_does_not_validate_domain_values() {
        let _descriptor = local_transform_schema_descriptor();
        let transform = crate::LocalTransform::default().translated(1.0, 2.0, 3.0);

        assert_eq!(transform.translation, crate::Vec3Value::new(1.0, 2.0, 3.0));
    }
}
