//! Stable app-program identity and version contracts.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppIdError {
    EmptyId { kind: &'static str },
    TrimmedId { kind: &'static str, value: String },
    UnnamespacedId { kind: &'static str, value: String },
    InvalidIdCharacter { kind: &'static str, value: String },
    PathLikeId { kind: &'static str, value: String },
    ZeroVersion { kind: &'static str },
}

impl fmt::Display for AppIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId { kind } => write!(formatter, "empty {kind} id"),
            Self::TrimmedId { kind, value } => {
                write!(
                    formatter,
                    "{kind} id {value:?} contains leading or trailing whitespace"
                )
            }
            Self::UnnamespacedId { kind, value } => {
                write!(formatter, "{kind} id {value} is not namespaced")
            }
            Self::InvalidIdCharacter { kind, value } => {
                write!(formatter, "{kind} id {value} contains an invalid character")
            }
            Self::PathLikeId { kind, value } => {
                write!(formatter, "{kind} id {value} looks like a file path")
            }
            Self::ZeroVersion { kind } => write!(formatter, "{kind} version must be non-zero"),
        }
    }
}

impl std::error::Error for AppIdError {}

pub(crate) fn validate_stable_id(value: &str, kind: &'static str) -> Result<(), AppIdError> {
    if value.is_empty() {
        return Err(AppIdError::EmptyId { kind });
    }
    if value.trim() != value {
        return Err(AppIdError::TrimmedId {
            kind,
            value: value.to_owned(),
        });
    }
    if !value.contains('.') {
        return Err(AppIdError::UnnamespacedId {
            kind,
            value: value.to_owned(),
        });
    }
    if value.contains('/') || value.contains('\\') {
        return Err(AppIdError::PathLikeId {
            kind,
            value: value.to_owned(),
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(AppIdError::InvalidIdCharacter {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}

macro_rules! stable_id {
    ($type_name:ident, $kind:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $type_name(String);

        impl $type_name {
            pub fn new(value: impl Into<String>) -> Self {
                Self::try_new(value).expect(concat!($kind, " IDs must be stable namespaced IDs"))
            }

            pub fn try_new(value: impl Into<String>) -> Result<Self, AppIdError> {
                let value = value.into();
                validate_stable_id(&value, $kind)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

macro_rules! stable_version {
    ($type_name:ident, $kind:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $type_name(u32);

        impl $type_name {
            pub const fn new(value: u32) -> Self {
                assert!(value > 0);
                Self(value)
            }

            pub fn try_new(value: u32) -> Result<Self, AppIdError> {
                if value == 0 {
                    Err(AppIdError::ZeroVersion { kind: $kind })
                } else {
                    Ok(Self(value))
                }
            }

            pub const fn value(self) -> u32 {
                self.0
            }
        }
    };
}

stable_id!(AppProgramId, "app program");
stable_version!(AppProgramVersion, "app program");
stable_id!(AppModelId, "app model");
stable_version!(AppModelVersion, "app model");
stable_id!(AppActionId, "app action");
stable_version!(AppActionVersion, "app action");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppModelRevision(u64);

impl AppModelRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn initial() -> Self {
        Self(0)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}
