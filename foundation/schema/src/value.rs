use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaValue {
    kind: SchemaValueKind,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum SchemaValueKind {
    Null,
    Bool(bool),
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    String(String),
    EnumSymbol(String),
    List(Vec<SchemaValue>),
    Map(Vec<SchemaValueMapEntry>),
    Object(Vec<SchemaValueObjectField>),
    Opaque(String),
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaValueMapEntry {
    key: String,
    value: SchemaValue,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SchemaValueObjectField {
    key: String,
    value: SchemaValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaValueError {
    NonFiniteFloat,
    EmptyKey,
    EmptyEnumSymbol,
    EmptyOpaqueKind,
    DuplicateKey(String),
}

impl SchemaValue {
    pub fn null() -> Self {
        Self {
            kind: SchemaValueKind::Null,
        }
    }

    pub fn bool(value: bool) -> Self {
        Self {
            kind: SchemaValueKind::Bool(value),
        }
    }

    pub fn integer(value: i64) -> Self {
        Self {
            kind: SchemaValueKind::Integer(value),
        }
    }

    pub fn unsigned_integer(value: u64) -> Self {
        Self {
            kind: SchemaValueKind::UnsignedInteger(value),
        }
    }

    pub fn float(value: f64) -> Result<Self, SchemaValueError> {
        if !value.is_finite() {
            return Err(SchemaValueError::NonFiniteFloat);
        }

        Ok(Self {
            kind: SchemaValueKind::Float(value),
        })
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self {
            kind: SchemaValueKind::String(value.into()),
        }
    }

    pub fn enum_symbol(value: impl Into<String>) -> Result<Self, SchemaValueError> {
        let value = value.into();
        if value.is_empty() {
            return Err(SchemaValueError::EmptyEnumSymbol);
        }

        Ok(Self {
            kind: SchemaValueKind::EnumSymbol(value),
        })
    }

    pub fn list(values: impl IntoIterator<Item = SchemaValue>) -> Self {
        Self {
            kind: SchemaValueKind::List(values.into_iter().collect()),
        }
    }

    pub fn map(
        entries: impl IntoIterator<Item = SchemaValueMapEntry>,
    ) -> Result<Self, SchemaValueError> {
        let entries =
            collect_unique_entries(entries.into_iter().map(|entry| (entry.key, entry.value)))?
                .into_iter()
                .map(|(key, value)| SchemaValueMapEntry { key, value })
                .collect();

        Ok(Self {
            kind: SchemaValueKind::Map(entries),
        })
    }

    pub fn object(
        fields: impl IntoIterator<Item = SchemaValueObjectField>,
    ) -> Result<Self, SchemaValueError> {
        let fields =
            collect_unique_entries(fields.into_iter().map(|field| (field.key, field.value)))?
                .into_iter()
                .map(|(key, value)| SchemaValueObjectField { key, value })
                .collect();

        Ok(Self {
            kind: SchemaValueKind::Object(fields),
        })
    }

    pub fn opaque(kind: impl Into<String>) -> Result<Self, SchemaValueError> {
        let kind = kind.into();
        if kind.is_empty() {
            return Err(SchemaValueError::EmptyOpaqueKind);
        }

        Ok(Self {
            kind: SchemaValueKind::Opaque(kind),
        })
    }

    pub fn as_object(&self) -> Option<&[SchemaValueObjectField]> {
        match &self.kind {
            SchemaValueKind::Object(fields) => Some(fields.as_slice()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.kind {
            SchemaValueKind::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match &self.kind {
            SchemaValueKind::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_unsigned_integer(&self) -> Option<u64> {
        match &self.kind {
            SchemaValueKind::UnsignedInteger(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match &self.kind {
            SchemaValueKind::Float(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match &self.kind {
            SchemaValueKind::String(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn as_enum_symbol(&self) -> Option<&str> {
        match &self.kind {
            SchemaValueKind::EnumSymbol(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[SchemaValue]> {
        match &self.kind {
            SchemaValueKind::List(values) => Some(values.as_slice()),
            _ => None,
        }
    }
}

impl SchemaValueMapEntry {
    pub fn new(key: impl Into<String>, value: SchemaValue) -> Result<Self, SchemaValueError> {
        let key = key.into();
        validate_key(&key)?;

        Ok(Self { key, value })
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn value(&self) -> &SchemaValue {
        &self.value
    }
}

impl SchemaValueObjectField {
    pub fn new(key: impl Into<String>, value: SchemaValue) -> Result<Self, SchemaValueError> {
        let key = key.into();
        validate_key(&key)?;

        Ok(Self { key, value })
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn value(&self) -> &SchemaValue {
        &self.value
    }
}

fn collect_unique_entries(
    entries: impl IntoIterator<Item = (String, SchemaValue)>,
) -> Result<Vec<(String, SchemaValue)>, SchemaValueError> {
    let mut unique = Vec::new();

    for (key, value) in entries {
        if unique
            .iter()
            .any(|(existing_key, _): &(String, SchemaValue)| existing_key == &key)
        {
            return Err(SchemaValueError::DuplicateKey(key));
        }

        unique.push((key, value));
    }

    Ok(unique)
}

fn validate_key(value: &str) -> Result<(), SchemaValueError> {
    if value.is_empty() {
        return Err(SchemaValueError::EmptyKey);
    }

    Ok(())
}
