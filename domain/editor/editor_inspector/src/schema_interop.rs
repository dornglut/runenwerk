//! File: domain/editor/editor_inspector/src/schema_interop.rs
//! Purpose: Explicit interoperability between inspector paths/values and schema vocabulary.

use crate::{InspectorEditValue, InspectorPath, InspectorPathSegment, InspectorValue};
use schema::{SchemaPath, SchemaPathError, SchemaPathSegment, SchemaValue, SchemaValueError};

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorSchemaInteropError {
    InvalidSchemaPath(SchemaPathError),
    InvalidSchemaValue(SchemaValueError),
    UnsupportedSchemaPathSegment,
    UnsupportedInspectorValue,
}

pub fn inspector_path_to_schema_path(
    path: &InspectorPath,
) -> Result<SchemaPath, InspectorSchemaInteropError> {
    let segments = path
        .segments()
        .iter()
        .map(|segment| match segment {
            InspectorPathSegment::Field(name) => SchemaPathSegment::field(name.clone())
                .map_err(InspectorSchemaInteropError::InvalidSchemaPath),
            InspectorPathSegment::Index(index) => Ok(SchemaPathSegment::index(*index)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    SchemaPath::from_segments(segments).map_err(InspectorSchemaInteropError::InvalidSchemaPath)
}

pub fn schema_path_to_inspector_path(
    path: &SchemaPath,
) -> Result<InspectorPath, InspectorSchemaInteropError> {
    let mut inspector_path = InspectorPath::root();

    for segment in path.segments() {
        inspector_path = match segment {
            SchemaPathSegment::Field(name) => inspector_path.child_field(name.clone()),
            SchemaPathSegment::Index(index) => inspector_path.child_index(*index),
            SchemaPathSegment::Key(_) | SchemaPathSegment::Variant(_) => {
                return Err(InspectorSchemaInteropError::UnsupportedSchemaPathSegment);
            }
        };
    }

    Ok(inspector_path)
}

pub fn inspector_value_to_schema_value(
    value: &InspectorValue,
) -> Result<SchemaValue, InspectorSchemaInteropError> {
    match value {
        InspectorValue::Bool(value) => Ok(SchemaValue::bool(*value)),
        InspectorValue::Integer(value) => Ok(SchemaValue::integer(*value)),
        InspectorValue::Float(value) => {
            SchemaValue::float(*value).map_err(InspectorSchemaInteropError::InvalidSchemaValue)
        }
        InspectorValue::Text(value) => Ok(SchemaValue::string(value.clone())),
        InspectorValue::Enum { current, .. } => SchemaValue::enum_symbol(current.clone())
            .map_err(InspectorSchemaInteropError::InvalidSchemaValue),
        InspectorValue::ReadOnlyText(_)
        | InspectorValue::Group
        | InspectorValue::Unsupported { .. } => {
            Err(InspectorSchemaInteropError::UnsupportedInspectorValue)
        }
    }
}

pub fn inspector_edit_value_to_schema_value(
    value: &InspectorEditValue,
) -> Result<SchemaValue, InspectorSchemaInteropError> {
    match value {
        InspectorEditValue::Bool(value) => Ok(SchemaValue::bool(*value)),
        InspectorEditValue::Integer(value) => Ok(SchemaValue::integer(*value)),
        InspectorEditValue::Float(value) => {
            SchemaValue::float(*value).map_err(InspectorSchemaInteropError::InvalidSchemaValue)
        }
        InspectorEditValue::Text(value) => Ok(SchemaValue::string(value.clone())),
        InspectorEditValue::EnumSymbol(value) => SchemaValue::enum_symbol(value.clone())
            .map_err(InspectorSchemaInteropError::InvalidSchemaValue),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::SchemaShape;

    #[test]
    fn inspector_path_converts_to_schema_path() {
        let inspector_path = InspectorPath::root()
            .child_field("transform")
            .child_field("translation")
            .child_index(2);

        let schema_path = inspector_path_to_schema_path(&inspector_path)
            .expect("inspector path should convert to schema path");

        assert_eq!(schema_path.segments()[0].as_field(), Some("transform"));
        assert_eq!(schema_path.segments()[1].as_field(), Some("translation"));
        assert_eq!(schema_path.segments()[2].as_index(), Some(2));
    }

    #[test]
    fn schema_path_converts_to_inspector_path_when_supported() {
        let schema_path = SchemaPath::from_segments([
            SchemaPathSegment::field("transform").unwrap(),
            SchemaPathSegment::field("translation").unwrap(),
            SchemaPathSegment::index(1),
        ])
        .unwrap();

        let inspector_path = schema_path_to_inspector_path(&schema_path)
            .expect("supported schema path should convert to inspector path");

        assert_eq!(
            inspector_path.segments(),
            &[
                InspectorPathSegment::Field("transform".to_string()),
                InspectorPathSegment::Field("translation".to_string()),
                InspectorPathSegment::Index(1),
            ]
        );
    }

    #[test]
    fn schema_path_with_unsupported_segment_is_rejected_by_inspector_interop() {
        let schema_path = SchemaPath::from_segments([SchemaPathSegment::key("material").unwrap()])
            .expect("schema key path should be structurally valid");

        let error = schema_path_to_inspector_path(&schema_path)
            .expect_err("inspector interop should reject unsupported schema path segments");

        assert_eq!(
            error,
            InspectorSchemaInteropError::UnsupportedSchemaPathSegment
        );
    }

    #[test]
    fn inspector_value_converts_to_schema_value() {
        let value = inspector_value_to_schema_value(&InspectorValue::Integer(-42))
            .expect("supported inspector value should convert");

        assert_eq!(value.as_integer(), Some(-42));
    }

    #[test]
    fn inspector_edit_value_converts_to_schema_value() {
        let value =
            inspector_edit_value_to_schema_value(&InspectorEditValue::Text("Player".into()))
                .expect("supported inspector edit value should convert");

        assert_eq!(value, SchemaValue::string("Player"));
    }

    #[test]
    fn enum_inspector_values_convert_to_schema_enum_symbols() {
        let value = inspector_value_to_schema_value(&InspectorValue::Enum {
            current: "Linear".to_string(),
            options: vec!["Nearest".to_string(), "Linear".to_string()],
        })
        .expect("enum inspector value should convert to schema enum symbol");

        assert_eq!(value.as_enum_symbol(), Some("Linear"));

        let edit_value =
            inspector_edit_value_to_schema_value(&InspectorEditValue::EnumSymbol("Nearest".into()))
                .expect("enum inspector edit value should convert to schema enum symbol");

        assert_eq!(edit_value.as_enum_symbol(), Some("Nearest"));
    }

    #[test]
    fn schema_interop_does_not_validate_value_against_shape() {
        let _shape = SchemaShape::string();
        let value = inspector_edit_value_to_schema_value(&InspectorEditValue::Integer(7))
            .expect("interop should only project values");

        assert_eq!(value.as_integer(), Some(7));
    }
}
