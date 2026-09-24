use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandTargetHint {
    Unspecified,
    DocumentLike,
    EntityLike,
    ComponentLike,
    ResourceLike,
    PathAddressed,
    External,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandEffectHint {
    #[default]
    Unknown,
    NoMutationExpected,
    SessionMutation,
    DomainMutation,
    ExternalSideEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandReversibilityHint {
    #[default]
    Unknown,
    Reversible,
    Irreversible,
    DependsOnParameters,
}
