//! Portable schema vocabulary for Runenwerk.
//!
//! This crate describes typed shapes, paths, values, fields, constraints, and
//! descriptors. It does not validate domain data, execute commands, mutate
//! state, reflect Rust values, register schemas globally, or own diagnostics.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod compatibility;
#[cfg(feature = "alloc")]
pub mod constraint;
#[cfg(feature = "alloc")]
pub mod descriptor;
#[cfg(all(feature = "alloc", feature = "diagnostics"))]
pub mod diagnostic;
#[cfg(feature = "alloc")]
pub mod field;
pub mod id;
pub mod issue;
#[cfg(feature = "alloc")]
pub mod metadata;
#[cfg(feature = "alloc")]
pub mod path;
pub mod prelude;
#[cfg(feature = "alloc")]
pub mod shape;
#[cfg(feature = "alloc")]
pub mod value;
pub mod version;

pub use compatibility::SchemaCompatibility;
#[cfg(feature = "alloc")]
pub use constraint::{SchemaConstraint, SchemaConstraintError};
#[cfg(feature = "alloc")]
pub use descriptor::{SchemaDescriptor, SchemaDescriptorError};
#[cfg(all(feature = "alloc", feature = "diagnostics"))]
pub use diagnostic::{
    schema_constraint_error_to_diagnostic, schema_descriptor_error_to_diagnostic,
    schema_field_error_to_diagnostic, schema_id_error_to_diagnostic, schema_issue_to_diagnostic,
    schema_issues_to_diagnostic_report, schema_metadata_error_to_diagnostic,
    schema_path_error_to_diagnostic, schema_shape_error_to_diagnostic,
    schema_value_error_to_diagnostic, schema_version_error_to_diagnostic,
};
#[cfg(feature = "alloc")]
pub use field::{SchemaField, SchemaFieldError};
pub use id::{SchemaId, SchemaIdError};
pub use issue::{SchemaIssue, SchemaIssueCode, SchemaIssueSubject};
#[cfg(feature = "alloc")]
pub use metadata::{SchemaMetadata, SchemaMetadataEntry, SchemaMetadataError, SchemaMetadataValue};
#[cfg(feature = "alloc")]
pub use path::{SchemaPath, SchemaPathError, SchemaPathSegment};
#[cfg(feature = "alloc")]
pub use shape::{SchemaShape, SchemaShapeError};
#[cfg(feature = "alloc")]
pub use value::{SchemaValue, SchemaValueError, SchemaValueMapEntry, SchemaValueObjectField};
pub use version::{SchemaVersion, SchemaVersionError};

pub const FOUNDATION_SCHEMA_DOMAIN: &str = "foundation.schema";

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use alloc::vec::Vec;

    use crate::{
        SchemaCompatibility, SchemaConstraint, SchemaDescriptor, SchemaField, SchemaId,
        SchemaMetadata, SchemaMetadataEntry, SchemaMetadataValue, SchemaPath, SchemaPathSegment,
        SchemaShape, SchemaValue, SchemaValueMapEntry, SchemaValueObjectField, SchemaVersion,
    };

    fn string_shape() -> SchemaShape {
        SchemaShape::string()
    }

    #[test]
    fn schema_id_rejects_empty_identifier() {
        assert!(SchemaId::new("").is_err());
    }

    #[test]
    fn schema_id_rejects_whitespace() {
        assert!(SchemaId::new("scene.local transform").is_err());
    }

    #[test]
    fn schema_version_rejects_zero() {
        assert!(SchemaVersion::new(0).is_err());
    }

    #[test]
    fn schema_path_root_is_valid() {
        let path = SchemaPath::root();

        assert!(path.is_root());
        assert!(path.segments().is_empty());
    }

    #[test]
    fn schema_path_preserves_segment_order() {
        let path = SchemaPath::from_segments([
            SchemaPathSegment::field("transform").unwrap(),
            SchemaPathSegment::field("translation").unwrap(),
            SchemaPathSegment::index(1),
        ])
        .unwrap();

        assert_eq!(path.segments()[0].as_field(), Some("transform"));
        assert_eq!(path.segments()[1].as_field(), Some("translation"));
        assert_eq!(path.segments()[2].as_index(), Some(1));
    }

    #[test]
    fn schema_path_rejects_empty_field_name() {
        assert!(SchemaPathSegment::field("").is_err());
    }

    #[test]
    fn schema_value_preserves_integer_signedness() {
        let signed = SchemaValue::integer(-1);
        let unsigned = SchemaValue::unsigned_integer(1);

        assert_eq!(signed.as_integer(), Some(-1));
        assert_eq!(unsigned.as_unsigned_integer(), Some(1));
    }

    #[test]
    fn schema_value_rejects_non_finite_float() {
        assert!(SchemaValue::float(f64::NAN).is_err());
        assert!(SchemaValue::float(f64::INFINITY).is_err());
    }

    #[test]
    fn schema_value_object_rejects_duplicate_keys() {
        let result = SchemaValue::object([
            SchemaValueObjectField::new("name", SchemaValue::string("A")).unwrap(),
            SchemaValueObjectField::new("name", SchemaValue::string("B")).unwrap(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn schema_value_map_rejects_duplicate_keys() {
        let result = SchemaValue::map([
            SchemaValueMapEntry::new("name", SchemaValue::string("A")).unwrap(),
            SchemaValueMapEntry::new("name", SchemaValue::string("B")).unwrap(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn schema_value_object_preserves_key_order() {
        let value = SchemaValue::object([
            SchemaValueObjectField::new("b", SchemaValue::bool(true)).unwrap(),
            SchemaValueObjectField::new("a", SchemaValue::bool(false)).unwrap(),
        ])
        .unwrap();

        let fields = value.as_object().unwrap();
        assert_eq!(fields[0].key(), "b");
        assert_eq!(fields[1].key(), "a");
    }

    #[test]
    fn object_shape_rejects_duplicate_field_names() {
        let result = SchemaShape::object([
            SchemaField::new("name", string_shape()).unwrap(),
            SchemaField::new("name", SchemaShape::integer()).unwrap(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn constraint_range_rejects_min_greater_than_max() {
        assert!(SchemaConstraint::numeric_range(10.0, 1.0).is_err());
    }

    #[test]
    fn descriptor_preserves_field_order() {
        let shape = SchemaShape::object([
            SchemaField::new("second", string_shape()).unwrap(),
            SchemaField::new("first", SchemaShape::bool()).unwrap(),
        ])
        .unwrap();

        let fields = shape.as_object_fields().unwrap();
        assert_eq!(fields[0].name(), "second");
        assert_eq!(fields[1].name(), "first");
    }

    #[test]
    fn descriptor_preserves_constraint_order() {
        let field = SchemaField::new("name", string_shape())
            .unwrap()
            .with_constraint(SchemaConstraint::read_only_hint())
            .with_constraint(SchemaConstraint::required_presence());

        assert!(field.constraints()[0].is_read_only_hint());
        assert!(field.constraints()[1].is_required_presence());
    }

    #[test]
    fn metadata_preserves_key_order() {
        let metadata = SchemaMetadata::from_entries([
            SchemaMetadataEntry::new("b", SchemaMetadataValue::string("second")).unwrap(),
            SchemaMetadataEntry::new("a", SchemaMetadataValue::string("first")).unwrap(),
        ])
        .unwrap();

        assert_eq!(metadata.entries()[0].key(), "b");
        assert_eq!(metadata.entries()[1].key(), "a");
    }

    #[test]
    fn descriptor_reports_highest_schema_issue_deterministically() {
        let descriptor = SchemaDescriptor::new(
            SchemaId::from_static("scene.local_transform").unwrap(),
            SchemaVersion::new(1).unwrap(),
            SchemaShape::object(Vec::new()).unwrap(),
        )
        .with_compatibility(SchemaCompatibility::Compatible);

        assert!(descriptor.highest_issue().is_none());
    }

    #[test]
    fn phase1_has_no_generic_schema_value_shape_validator() {
        let _shape = SchemaShape::string();
        let _value = SchemaValue::string("value");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn schema_descriptor_round_trips_with_version() {
        let descriptor = SchemaDescriptor::new(
            SchemaId::from_static("scene.local_transform").unwrap(),
            SchemaVersion::new(1).unwrap(),
            SchemaShape::object(Vec::new()).unwrap(),
        );

        let json = serde_json::to_string(&descriptor).unwrap();
        let round_trip: SchemaDescriptor = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip.version().value(), 1);
        assert_eq!(round_trip.id().as_str(), "scene.local_transform");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn schema_path_round_trips_structurally() {
        let path = SchemaPath::from_segments([
            SchemaPathSegment::field("transform").unwrap(),
            SchemaPathSegment::index(2),
            SchemaPathSegment::variant("Some").unwrap(),
        ])
        .unwrap();

        let json = serde_json::to_string(&path).unwrap();
        let round_trip: SchemaPath = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, path);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn schema_value_round_trips_without_losing_numeric_kind() {
        let value = SchemaValue::list([SchemaValue::integer(-7), SchemaValue::unsigned_integer(7)]);

        let json = serde_json::to_string(&value).unwrap();
        let round_trip: SchemaValue = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, value);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn constraint_round_trips_without_display_string_parsing() {
        let constraint = SchemaConstraint::numeric_range(1.0, 3.0).unwrap();

        let json = serde_json::to_string(&constraint).unwrap();
        let round_trip: SchemaConstraint = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip, constraint);
    }
}
