use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::field::SchemaField;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaShape {
    kind: SchemaShapeKind,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum SchemaShapeKind {
    Bool,
    Integer,
    Float,
    String,
    Enum(Vec<String>),
    Object(Vec<SchemaField>),
    List(Box<SchemaShape>),
    Map {
        key: Box<SchemaShape>,
        value: Box<SchemaShape>,
    },
    Optional(Box<SchemaShape>),
    Opaque(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaShapeError {
    EmptyEnumOptions,
    EmptyEnumOption,
    DuplicateEnumOption(String),
    DuplicateFieldName(String),
    EmptyOpaqueKind,
}

impl SchemaShape {
    pub fn bool() -> Self {
        Self {
            kind: SchemaShapeKind::Bool,
        }
    }

    pub fn integer() -> Self {
        Self {
            kind: SchemaShapeKind::Integer,
        }
    }

    pub fn float() -> Self {
        Self {
            kind: SchemaShapeKind::Float,
        }
    }

    pub fn string() -> Self {
        Self {
            kind: SchemaShapeKind::String,
        }
    }

    pub fn enumeration(
        options: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SchemaShapeError> {
        let mut unique = Vec::new();

        for option in options {
            let option = option.into();
            if option.is_empty() {
                return Err(SchemaShapeError::EmptyEnumOption);
            }

            if unique.iter().any(|existing| existing == &option) {
                return Err(SchemaShapeError::DuplicateEnumOption(option));
            }

            unique.push(option);
        }

        if unique.is_empty() {
            return Err(SchemaShapeError::EmptyEnumOptions);
        }

        Ok(Self {
            kind: SchemaShapeKind::Enum(unique),
        })
    }

    pub fn object(fields: impl IntoIterator<Item = SchemaField>) -> Result<Self, SchemaShapeError> {
        let mut unique = Vec::new();

        for field in fields {
            if unique
                .iter()
                .any(|existing: &SchemaField| existing.name() == field.name())
            {
                return Err(SchemaShapeError::DuplicateFieldName(field.name().into()));
            }

            unique.push(field);
        }

        Ok(Self {
            kind: SchemaShapeKind::Object(unique),
        })
    }

    pub fn list(item: SchemaShape) -> Self {
        Self {
            kind: SchemaShapeKind::List(Box::new(item)),
        }
    }

    pub fn map(key: SchemaShape, value: SchemaShape) -> Self {
        Self {
            kind: SchemaShapeKind::Map {
                key: Box::new(key),
                value: Box::new(value),
            },
        }
    }

    pub fn optional(inner: SchemaShape) -> Self {
        Self {
            kind: SchemaShapeKind::Optional(Box::new(inner)),
        }
    }

    pub fn opaque(kind: impl Into<String>) -> Result<Self, SchemaShapeError> {
        let kind = kind.into();
        if kind.is_empty() {
            return Err(SchemaShapeError::EmptyOpaqueKind);
        }

        Ok(Self {
            kind: SchemaShapeKind::Opaque(kind),
        })
    }

    pub fn as_object_fields(&self) -> Option<&[SchemaField]> {
        match &self.kind {
            SchemaShapeKind::Object(fields) => Some(fields.as_slice()),
            _ => None,
        }
    }
}
