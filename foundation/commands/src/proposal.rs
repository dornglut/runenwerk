use crate::{
    CommandContractRef, CommandMetadata, CommandMetadataEntry, CommandMetadataError,
    CommandProposalId,
};
use schema::SchemaValue;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandProposal {
    proposal_id: Option<CommandProposalId>,
    contract: CommandContractRef,
    parameters: SchemaValue,
    metadata: CommandMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandProposalError {
    Metadata(CommandMetadataError),
}

impl CommandProposal {
    pub fn new(contract: CommandContractRef, parameters: SchemaValue) -> Self {
        Self {
            proposal_id: None,
            contract,
            parameters,
            metadata: CommandMetadata::new(),
        }
    }

    pub fn with_proposal_id(mut self, proposal_id: CommandProposalId) -> Self {
        self.proposal_id = Some(proposal_id);
        self
    }

    pub fn with_metadata_entry(
        mut self,
        entry: CommandMetadataEntry,
    ) -> Result<Self, CommandProposalError> {
        self.metadata
            .push(entry)
            .map_err(CommandProposalError::Metadata)?;
        Ok(self)
    }

    pub fn proposal_id(&self) -> Option<&CommandProposalId> {
        self.proposal_id.as_ref()
    }

    pub fn contract(&self) -> &CommandContractRef {
        &self.contract
    }

    pub fn parameters(&self) -> &SchemaValue {
        &self.parameters
    }

    pub fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
}
