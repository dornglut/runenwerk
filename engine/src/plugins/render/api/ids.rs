use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceIdError {
    kind: &'static str,
    id: String,
    reason: &'static str,
}

impl NamespaceIdError {
    pub fn new(kind: &'static str, id: impl Into<String>, reason: &'static str) -> Self {
        Self {
            kind,
            id: id.into(),
            reason,
        }
    }
}

impl Display for NamespaceIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid {} id '{}': {}", self.kind, self.id, self.reason)
    }
}

impl std::error::Error for NamespaceIdError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderFlowId(String);

impl RenderFlowId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderFlowId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderFlowId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPassId(String);

impl RenderPassId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate_namespaced(&self) -> Result<(), NamespaceIdError> {
        validate_namespaced_id("pass", self.as_str())
    }
}

impl From<&str> for RenderPassId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderPassId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderResourceId(String);

impl RenderResourceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate_namespaced(&self) -> Result<(), NamespaceIdError> {
        validate_namespaced_id("resource", self.as_str())
    }
}

impl From<&str> for RenderResourceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderResourceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

pub fn validate_namespaced_id(kind: &'static str, id: &str) -> Result<(), NamespaceIdError> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(NamespaceIdError::new(kind, id, "value must not be empty"));
    }

    if !is_namespaced_id(trimmed) {
        return Err(NamespaceIdError::new(
            kind,
            trimmed,
            "expected dot-separated namespace segments (for example 'post.tonemap')",
        ));
    }

    Ok(())
}

pub fn is_namespaced_id(value: &str) -> bool {
    let mut saw_dot = false;
    for segment in value.split('.') {
        if segment.is_empty() {
            return false;
        }
        if !segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return false;
        }
        saw_dot = true;
    }

    saw_dot && value.contains('.')
}

pub fn namespace_of(value: &str) -> Option<&str> {
    let mut split = value.split('.');
    let namespace = split.next()?;
    if namespace.is_empty() || split.next().is_none() {
        return None;
    }
    Some(namespace)
}
