use std::fmt;

use super::source::UiTypedSource;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiTypedScreenId(String);

impl UiTypedScreenId {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("typed UI screen IDs must be stable namespaced IDs")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, UiTypedIdentityError> {
        Ok(Self(validate_typed_contract_id("screen", value.into())?))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiTypedIdentityError {
    EmptyId { kind: &'static str },
    UnnamespacedId { kind: &'static str, value: String },
    InvalidIdCharacter { kind: &'static str, value: String },
}

impl fmt::Display for UiTypedIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId { kind } => write!(formatter, "typed UI {kind} id must not be empty"),
            Self::UnnamespacedId { kind, value } => {
                write!(formatter, "typed UI {kind} id {value} is not namespaced")
            }
            Self::InvalidIdCharacter { kind, value } => write!(
                formatter,
                "typed UI {kind} id {value} contains an invalid character"
            ),
        }
    }
}

impl std::error::Error for UiTypedIdentityError {}

pub(crate) fn validate_typed_contract_id(
    kind: &'static str,
    value: String,
) -> Result<String, UiTypedIdentityError> {
    if value.is_empty() {
        return Err(UiTypedIdentityError::EmptyId { kind });
    }
    if !value.contains('.') {
        return Err(UiTypedIdentityError::UnnamespacedId { kind, value });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'))
    {
        return Err(UiTypedIdentityError::InvalidIdCharacter { kind, value });
    }
    Ok(value)
}

pub trait UiScreen {
    fn screen_id(&self) -> UiTypedScreenId;
    fn build_source(&self) -> UiTypedSource;
}

pub trait IntoUi {
    fn into_ui_source(self) -> UiTypedSource;
}

impl<T> IntoUi for T
where
    T: UiScreen,
{
    fn into_ui_source(self) -> UiTypedSource {
        self.build_source()
    }
}
