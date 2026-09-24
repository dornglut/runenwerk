use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::shape::SchemaShape;
use crate::value::SchemaValue;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaConstraint {
    kind: SchemaConstraintKind,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum SchemaConstraintKind {
    RequiredPresence,
    ReadOnlyHint,
    Deprecated(Option<String>),
    NumericMin(f64),
    NumericMax(f64),
    NumericRange { min: f64, max: f64 },
    StringMinLength(usize),
    StringMaxLength(usize),
    NamedStringPatternHint(String),
    EnumOptions(Vec<String>),
    ListMinLength(usize),
    ListMaxLength(usize),
    MapKeyShape(Box<SchemaShape>),
    SuggestedDefaultValue(Box<SchemaValue>),
    RecommendedValue(Box<SchemaValue>),
    DisplayUnitLabel(String),
    Documentation(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaConstraintError {
    NonFiniteNumber,
    MinGreaterThanMax,
    EmptyPatternName,
    EmptyEnumOptions,
    EmptyEnumOption,
    DuplicateEnumOption(String),
    EmptyUnitLabel,
}

impl SchemaConstraint {
    pub fn required_presence() -> Self {
        Self {
            kind: SchemaConstraintKind::RequiredPresence,
        }
    }

    pub fn read_only_hint() -> Self {
        Self {
            kind: SchemaConstraintKind::ReadOnlyHint,
        }
    }

    pub fn deprecated(message: Option<impl Into<String>>) -> Self {
        Self {
            kind: SchemaConstraintKind::Deprecated(message.map(Into::into)),
        }
    }

    pub fn numeric_min(value: f64) -> Result<Self, SchemaConstraintError> {
        validate_finite(value)?;

        Ok(Self {
            kind: SchemaConstraintKind::NumericMin(value),
        })
    }

    pub fn numeric_max(value: f64) -> Result<Self, SchemaConstraintError> {
        validate_finite(value)?;

        Ok(Self {
            kind: SchemaConstraintKind::NumericMax(value),
        })
    }

    pub fn numeric_range(min: f64, max: f64) -> Result<Self, SchemaConstraintError> {
        validate_finite(min)?;
        validate_finite(max)?;

        if min > max {
            return Err(SchemaConstraintError::MinGreaterThanMax);
        }

        Ok(Self {
            kind: SchemaConstraintKind::NumericRange { min, max },
        })
    }

    pub fn string_min_length(value: usize) -> Self {
        Self {
            kind: SchemaConstraintKind::StringMinLength(value),
        }
    }

    pub fn string_max_length(value: usize) -> Self {
        Self {
            kind: SchemaConstraintKind::StringMaxLength(value),
        }
    }

    pub fn named_string_pattern_hint(
        value: impl Into<String>,
    ) -> Result<Self, SchemaConstraintError> {
        let value = value.into();
        if value.is_empty() {
            return Err(SchemaConstraintError::EmptyPatternName);
        }

        Ok(Self {
            kind: SchemaConstraintKind::NamedStringPatternHint(value),
        })
    }

    pub fn enum_options(
        options: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, SchemaConstraintError> {
        let mut unique = Vec::new();

        for option in options {
            let option = option.into();
            if option.is_empty() {
                return Err(SchemaConstraintError::EmptyEnumOption);
            }

            if unique.iter().any(|existing| existing == &option) {
                return Err(SchemaConstraintError::DuplicateEnumOption(option));
            }

            unique.push(option);
        }

        if unique.is_empty() {
            return Err(SchemaConstraintError::EmptyEnumOptions);
        }

        Ok(Self {
            kind: SchemaConstraintKind::EnumOptions(unique),
        })
    }

    pub fn list_min_length(value: usize) -> Self {
        Self {
            kind: SchemaConstraintKind::ListMinLength(value),
        }
    }

    pub fn list_max_length(value: usize) -> Self {
        Self {
            kind: SchemaConstraintKind::ListMaxLength(value),
        }
    }

    pub fn map_key_shape(shape: SchemaShape) -> Self {
        Self {
            kind: SchemaConstraintKind::MapKeyShape(Box::new(shape)),
        }
    }

    pub fn suggested_default_value(value: SchemaValue) -> Self {
        Self {
            kind: SchemaConstraintKind::SuggestedDefaultValue(Box::new(value)),
        }
    }

    pub fn recommended_value(value: SchemaValue) -> Self {
        Self {
            kind: SchemaConstraintKind::RecommendedValue(Box::new(value)),
        }
    }

    pub fn display_unit_label(value: impl Into<String>) -> Result<Self, SchemaConstraintError> {
        let value = value.into();
        if value.is_empty() {
            return Err(SchemaConstraintError::EmptyUnitLabel);
        }

        Ok(Self {
            kind: SchemaConstraintKind::DisplayUnitLabel(value),
        })
    }

    pub fn documentation(value: impl Into<String>) -> Self {
        Self {
            kind: SchemaConstraintKind::Documentation(value.into()),
        }
    }

    pub fn is_required_presence(&self) -> bool {
        matches!(self.kind, SchemaConstraintKind::RequiredPresence)
    }

    pub fn is_read_only_hint(&self) -> bool {
        matches!(self.kind, SchemaConstraintKind::ReadOnlyHint)
    }
}

fn validate_finite(value: f64) -> Result<(), SchemaConstraintError> {
    if !value.is_finite() {
        return Err(SchemaConstraintError::NonFiniteNumber);
    }

    Ok(())
}
