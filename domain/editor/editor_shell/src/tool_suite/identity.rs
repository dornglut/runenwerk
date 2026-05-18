//! File: domain/editor/editor_shell/src/tool_suite/identity.rs
//! Purpose: Validated stable identifiers for editor tool-suite contracts.

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSuiteIdentityError {
    kind: &'static str,
    value: String,
}

impl ToolSuiteIdentityError {
    fn new(kind: &'static str, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

    pub const fn kind(&self) -> &'static str {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for ToolSuiteIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid {} `{}`: expected lowercase dotted identifier",
            self.kind, self.value
        )
    }
}

impl std::error::Error for ToolSuiteIdentityError {}

fn is_valid_lowercase_dotted_identifier(value: &str) -> bool {
    if value.is_empty() || value.trim().is_empty() {
        return false;
    }

    value.split('.').all(is_valid_identifier_segment)
}

fn is_valid_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !matches!(first, 'a'..='z') {
        return false;
    }

    chars.all(|ch| matches!(ch, 'a'..='z' | '0'..='9' | '_'))
}

macro_rules! stable_identity {
    ($name:ident, $kind:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ToolSuiteIdentityError> {
                let value = value.into();
                if is_valid_lowercase_dotted_identifier(&value) {
                    Ok(Self(value))
                } else {
                    Err(ToolSuiteIdentityError::new($kind, value))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl TryFrom<String> for $name {
            type Error = ToolSuiteIdentityError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl FromStr for $name {
            type Err = ToolSuiteIdentityError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

stable_identity!(ToolSuiteId, "tool suite id");
stable_identity!(ToolSurfaceStableKey, "tool surface stable key");
stable_identity!(ProviderFamilyId, "provider family id");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_suite_id_rejects_invalid_values() {
        assert_invalid::<ToolSuiteId>();
    }

    #[test]
    fn stable_surface_key_rejects_invalid_values() {
        assert_invalid::<ToolSurfaceStableKey>();
    }

    #[test]
    fn provider_family_id_rejects_invalid_values() {
        assert_invalid::<ProviderFamilyId>();
    }

    #[test]
    fn stable_ids_accept_lowercase_dotted_values() {
        let suite_id = ToolSuiteId::new("runenwerk.material_lab").unwrap();
        let surface_key = ToolSurfaceStableKey::from_str("runenwerk.material_lab.graph_canvas")
            .expect("valid stable key");
        let provider_family = ProviderFamilyId::try_from("runenwerk.material_lab".to_string())
            .expect("valid provider family");

        assert_eq!(suite_id.as_str(), "runenwerk.material_lab");
        assert_eq!(
            surface_key.to_string(),
            "runenwerk.material_lab.graph_canvas"
        );
        assert_eq!(provider_family.as_str(), "runenwerk.material_lab");
    }

    fn assert_invalid<T>()
    where
        T: FromStr<Err = ToolSuiteIdentityError>,
    {
        for value in [
            "",
            "   ",
            "Runenwerk.material_lab",
            "runenwerk.MaterialLab",
            "runenwerk material_lab",
            "runenwerk/material_lab",
            ".runenwerk.material_lab",
            "runenwerk.material_lab.",
            "runenwerk..material_lab",
        ] {
            assert!(value.parse::<T>().is_err(), "`{value}` should be rejected");
        }
    }
}
