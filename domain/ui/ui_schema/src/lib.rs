//! File: domain/ui/ui_schema/src/lib.rs
//! Crate: ui_schema

pub mod schema;
pub mod value;

pub use schema::*;
pub use value::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_value_preserves_typed_object_shape() {
        let value = UiSchemaValue::object([
            ("route", UiSchemaValue::string("editor.color.apply")),
            ("version", UiSchemaValue::integer(1)),
            (
                "payload",
                UiSchemaValue::object([
                    ("r", UiSchemaValue::number(0.25)),
                    ("g", UiSchemaValue::number(0.5)),
                    ("b", UiSchemaValue::number(0.75)),
                    ("a", UiSchemaValue::number(1.0)),
                ]),
            ),
        ]);

        assert_eq!(value.kind(), UiSchemaValueKind::Object);
        assert_eq!(
            value.get("route"),
            Some(&UiSchemaValue::string("editor.color.apply"))
        );
        assert_eq!(
            value.get_path(&["payload", "g"]),
            Some(&UiSchemaValue::number(0.5))
        );
    }
}
