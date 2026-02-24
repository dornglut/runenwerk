use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PipelineKey(Cow<'static, str>);

impl PipelineKey {
    pub fn new(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    pub fn label(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<&str> for PipelineKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for PipelineKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for PipelineKey {
    fn as_ref(&self) -> &str {
        self.label()
    }
}
