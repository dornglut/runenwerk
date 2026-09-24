use alloc::string::String;
use alloc::vec::Vec;

use crate::{
    CommandContractId, CommandContractVersion, CommandEffectHint, CommandIssue, CommandMetadata,
    CommandMetadataEntry, CommandMetadataError, CommandResultSchemaRef, CommandReversibilityHint,
    CommandSchemaRef, CommandTargetHint,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandDescriptor {
    id: CommandContractId,
    version: CommandContractVersion,
    display_name: Option<String>,
    description: Option<String>,
    parameter_schema: CommandSchemaRef,
    result_schema: Option<CommandResultSchemaRef>,
    target_hint: Option<CommandTargetHint>,
    effect_hint: CommandEffectHint,
    reversibility_hint: CommandReversibilityHint,
    metadata: CommandMetadata,
    issues: Vec<CommandIssue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandDescriptorError {
    Metadata(CommandMetadataError),
}

impl CommandDescriptor {
    pub fn new(
        id: CommandContractId,
        version: CommandContractVersion,
        parameter_schema: CommandSchemaRef,
    ) -> Self {
        Self {
            id,
            version,
            display_name: None,
            description: None,
            parameter_schema,
            result_schema: None,
            target_hint: None,
            effect_hint: CommandEffectHint::Unknown,
            reversibility_hint: CommandReversibilityHint::Unknown,
            metadata: CommandMetadata::new(),
            issues: Vec::new(),
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_result_schema(mut self, result_schema: CommandResultSchemaRef) -> Self {
        self.result_schema = Some(result_schema);
        self
    }

    pub fn with_target_hint(mut self, target_hint: CommandTargetHint) -> Self {
        self.target_hint = Some(target_hint);
        self
    }

    pub fn with_effect_hint(mut self, effect_hint: CommandEffectHint) -> Self {
        self.effect_hint = effect_hint;
        self
    }

    pub fn with_reversibility_hint(mut self, reversibility_hint: CommandReversibilityHint) -> Self {
        self.reversibility_hint = reversibility_hint;
        self
    }

    pub fn with_metadata_entry(
        mut self,
        entry: CommandMetadataEntry,
    ) -> Result<Self, CommandDescriptorError> {
        self.metadata
            .push(entry)
            .map_err(CommandDescriptorError::Metadata)?;
        Ok(self)
    }

    pub fn with_issue(mut self, issue: CommandIssue) -> Self {
        self.issues.push(issue);
        self
    }

    pub fn id(&self) -> &CommandContractId {
        &self.id
    }

    pub fn version(&self) -> CommandContractVersion {
        self.version
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn parameter_schema(&self) -> &CommandSchemaRef {
        &self.parameter_schema
    }

    pub fn result_schema(&self) -> Option<&CommandResultSchemaRef> {
        self.result_schema.as_ref()
    }

    pub fn target_hint(&self) -> Option<&CommandTargetHint> {
        self.target_hint.as_ref()
    }

    pub fn effect_hint(&self) -> CommandEffectHint {
        self.effect_hint
    }

    pub fn reversibility_hint(&self) -> CommandReversibilityHint {
        self.reversibility_hint
    }

    pub fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    pub fn issues(&self) -> &[CommandIssue] {
        &self.issues
    }
}
