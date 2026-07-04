//! App action contracts.

use std::collections::BTreeMap;
use std::fmt;

use crate::ids::{AppActionId, AppActionVersion, AppIdError, validate_stable_id};
use crate::report::{AppDiagnostic, NAMESPACE_ACTION_SCHEMA};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppActionCapability(String);

impl AppActionCapability {
    pub fn new(value: impl Into<String>) -> Self {
        Self::try_new(value).expect("app action capabilities must be stable namespaced IDs")
    }

    pub fn try_new(value: impl Into<String>) -> Result<Self, AppIdError> {
        let value = value.into();
        validate_stable_id(&value, "app action capability")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppActionCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppActionSource {
    LocalHeadless {
        route_id: Option<String>,
        source_control_id: Option<String>,
        source_map: Vec<String>,
    },
}

impl AppActionSource {
    pub fn local_headless(
        route_id: Option<String>,
        source_control_id: Option<String>,
        source_map: Vec<String>,
    ) -> Self {
        Self::LocalHeadless {
            route_id,
            source_control_id,
            source_map,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppAction {
    pub action_id: AppActionId,
    pub action_version: AppActionVersion,
    pub payload: AppActionPayload,
    pub required_capabilities: Vec<AppActionCapability>,
    pub source: AppActionSource,
}

impl AppAction {
    pub fn new(
        action_id: AppActionId,
        action_version: AppActionVersion,
        payload: AppActionPayload,
        source: AppActionSource,
    ) -> Self {
        Self {
            action_id,
            action_version,
            payload,
            required_capabilities: Vec::new(),
            source,
        }
    }

    pub fn with_required_capability(mut self, capability: AppActionCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppActionPayload {
    Unit,
    Object(BTreeMap<String, AppActionPayloadValue>),
}

impl AppActionPayload {
    pub fn object(
        entries: impl IntoIterator<Item = (impl Into<String>, AppActionPayloadValue)>,
    ) -> Self {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            values.insert(key.into(), value);
        }
        Self::Object(values)
    }

    pub fn validate_against(&self, shape: &AppActionPayloadShape) -> Result<(), AppDiagnostic> {
        match (self, shape) {
            (Self::Unit, AppActionPayloadShape::Unit) => Ok(()),
            (Self::Unit, AppActionPayloadShape::Object { .. }) => Err(AppDiagnostic::new(
                NAMESPACE_ACTION_SCHEMA,
                "app.action.schema.payload_missing_object",
                "action payload must be an object",
            )),
            (Self::Object(values), AppActionPayloadShape::Unit) => {
                if values.is_empty() {
                    Ok(())
                } else {
                    Err(AppDiagnostic::new(
                        NAMESPACE_ACTION_SCHEMA,
                        "app.action.schema.payload_not_unit",
                        "action payload must be empty for this route",
                    ))
                }
            }
            (
                Self::Object(values),
                AppActionPayloadShape::Object {
                    fields,
                    allow_extra_fields,
                },
            ) => {
                for (field, kind) in fields {
                    let Some(value) = values.get(field) else {
                        return Err(AppDiagnostic::new(
                            NAMESPACE_ACTION_SCHEMA,
                            "app.action.schema.payload_missing_field",
                            format!("action payload is missing required field {field}"),
                        ));
                    };
                    if value.kind() != *kind {
                        return Err(AppDiagnostic::new(
                            NAMESPACE_ACTION_SCHEMA,
                            "app.action.schema.payload_field_kind",
                            format!("action payload field {field} has the wrong kind"),
                        ));
                    }
                }
                if !allow_extra_fields {
                    for field in values.keys() {
                        if !fields.contains_key(field) {
                            return Err(AppDiagnostic::new(
                                NAMESPACE_ACTION_SCHEMA,
                                "app.action.schema.payload_unknown_field",
                                format!("action payload field {field} is not accepted"),
                            ));
                        }
                    }
                }
                Ok(())
            }
        }
    }

    pub fn summary(&self, budget: usize) -> AppActionPayloadSummary {
        let raw = match self {
            Self::Unit => "unit".to_owned(),
            Self::Object(values) => {
                let entries = values
                    .iter()
                    .map(|(key, value)| format!("{key}:{}", value.safe_summary()))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("object{{{entries}}}")
            }
        };
        AppActionPayloadSummary::new(raw, budget)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppActionPayloadSummary {
    pub text: String,
    pub truncated: bool,
    pub budget: usize,
}

impl AppActionPayloadSummary {
    fn new(raw: String, budget: usize) -> Self {
        if raw.len() <= budget {
            Self {
                text: raw,
                truncated: false,
                budget,
            }
        } else {
            let mut text = raw
                .chars()
                .take(budget.saturating_sub(3))
                .collect::<String>();
            text.push_str("...");
            Self {
                text,
                truncated: true,
                budget,
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppActionPayloadValue {
    Integer(i64),
    Bool(bool),
    String(String),
}

impl AppActionPayloadValue {
    pub fn kind(&self) -> AppActionPayloadKind {
        match self {
            Self::Integer(_) => AppActionPayloadKind::Integer,
            Self::Bool(_) => AppActionPayloadKind::Bool,
            Self::String(_) => AppActionPayloadKind::String,
        }
    }

    fn safe_summary(&self) -> String {
        match self {
            Self::Integer(value) => format!("integer({value})"),
            Self::Bool(value) => format!("bool({value})"),
            Self::String(value) => format!("string(len={})", value.chars().count()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppActionPayloadShape {
    Unit,
    Object {
        fields: BTreeMap<String, AppActionPayloadKind>,
        allow_extra_fields: bool,
    },
}

impl AppActionPayloadShape {
    pub fn unit() -> Self {
        Self::Unit
    }

    pub fn object(
        fields: impl IntoIterator<Item = (impl Into<String>, AppActionPayloadKind)>,
    ) -> Self {
        let mut mapped = BTreeMap::new();
        for (field, kind) in fields {
            mapped.insert(field.into(), kind);
        }
        Self::Object {
            fields: mapped,
            allow_extra_fields: false,
        }
    }

    pub fn allow_extra_fields(mut self) -> Self {
        if let Self::Object {
            allow_extra_fields, ..
        } = &mut self
        {
            *allow_extra_fields = true;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppActionPayloadKind {
    Integer,
    Bool,
    String,
}
