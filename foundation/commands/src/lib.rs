//! Portable command descriptor and proposal vocabulary for Runenwerk.
//!
//! This crate describes requestable mutation contracts and inert command
//! proposals. It does not execute commands, route proposals, validate domain
//! meaning, register descriptors globally, grant permissions, or map proposals
//! to concrete domain command enums.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod descriptor;
#[cfg(feature = "alloc")]
pub mod hint;
pub mod id;
pub mod issue;
#[cfg(feature = "alloc")]
pub mod metadata;
pub mod prelude;
#[cfg(feature = "alloc")]
pub mod proposal;
pub mod schema_ref;
pub mod version;

#[cfg(feature = "alloc")]
pub use descriptor::{CommandDescriptor, CommandDescriptorError};
#[cfg(feature = "alloc")]
pub use hint::{CommandEffectHint, CommandReversibilityHint, CommandTargetHint};
pub use id::{
    CommandContractId, CommandContractIdError, CommandContractRef, CommandProposalId,
    CommandProposalIdError,
};
pub use issue::{CommandIssue, CommandIssueCode, CommandIssueSubject};
#[cfg(feature = "alloc")]
pub use metadata::{CommandMetadata, CommandMetadataEntry, CommandMetadataError};
#[cfg(feature = "alloc")]
pub use proposal::{CommandProposal, CommandProposalError};
pub use schema_ref::{CommandResultSchemaRef, CommandSchemaRef, CommandSchemaRefError};
pub use version::{CommandContractVersion, CommandContractVersionError};

pub const FOUNDATION_COMMANDS_DOMAIN: &str = "foundation.commands";

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use alloc::vec::Vec;

    use schema::{SchemaId, SchemaValue, SchemaVersion};

    use crate::{
        CommandContractId, CommandContractRef, CommandContractVersion, CommandDescriptor,
        CommandEffectHint, CommandMetadataEntry, CommandProposal, CommandReversibilityHint,
        CommandSchemaRef, CommandTargetHint,
    };

    fn contract_id() -> CommandContractId {
        CommandContractId::from_static("editor.scene.edit_component_field").unwrap()
    }

    fn contract_version() -> CommandContractVersion {
        CommandContractVersion::new(1).unwrap()
    }

    fn schema_ref() -> CommandSchemaRef {
        CommandSchemaRef::new(
            SchemaId::from_static("scene.local_transform").unwrap(),
            SchemaVersion::new(1).unwrap(),
        )
    }

    fn descriptor() -> CommandDescriptor {
        CommandDescriptor::new(contract_id(), contract_version(), schema_ref())
    }

    #[test]
    fn command_contract_id_rejects_empty() {
        assert!(CommandContractId::new("").is_err());
    }

    #[test]
    fn command_contract_id_rejects_whitespace() {
        assert!(CommandContractId::new("editor.scene edit").is_err());
    }

    #[test]
    fn command_contract_version_rejects_zero() {
        assert!(CommandContractVersion::new(0).is_err());
    }

    #[test]
    fn command_contract_ref_preserves_id_and_version() {
        let reference = CommandContractRef::new(contract_id(), contract_version());

        assert_eq!(reference.id().as_str(), "editor.scene.edit_component_field");
        assert_eq!(reference.version().value(), 1);
    }

    #[test]
    fn command_schema_ref_preserves_schema_id_and_version() {
        let reference = schema_ref();

        assert_eq!(reference.schema_id().as_str(), "scene.local_transform");
        assert_eq!(reference.schema_version().value(), 1);
    }

    #[test]
    fn command_descriptor_preserves_metadata_order() {
        let descriptor = descriptor()
            .with_metadata_entry(CommandMetadataEntry::new("b", "second").unwrap())
            .unwrap()
            .with_metadata_entry(CommandMetadataEntry::new("a", "first").unwrap())
            .unwrap();

        assert_eq!(descriptor.metadata().entries()[0].key(), "b");
        assert_eq!(descriptor.metadata().entries()[1].key(), "a");
    }

    #[test]
    fn command_descriptor_rejects_duplicate_metadata_keys() {
        let result = descriptor()
            .with_metadata_entry(CommandMetadataEntry::new("same", "first").unwrap())
            .unwrap()
            .with_metadata_entry(CommandMetadataEntry::new("same", "second").unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn command_proposal_preserves_contract_ref() {
        let proposal = CommandProposal::new(
            CommandContractRef::new(contract_id(), contract_version()),
            SchemaValue::string("value"),
        );

        assert_eq!(
            proposal.contract().id().as_str(),
            "editor.scene.edit_component_field"
        );
        assert_eq!(proposal.contract().version().value(), 1);
    }

    #[test]
    fn command_proposal_preserves_schema_value_parameters() {
        let proposal = CommandProposal::new(
            CommandContractRef::new(contract_id(), contract_version()),
            SchemaValue::unsigned_integer(7),
        );

        assert_eq!(proposal.parameters().as_unsigned_integer(), Some(7));
    }

    #[test]
    fn command_proposal_has_no_universal_target_path() {
        let proposal = CommandProposal::new(
            CommandContractRef::new(contract_id(), contract_version()),
            SchemaValue::object(Vec::new()).unwrap(),
        );

        assert!(proposal.metadata().is_empty());
    }

    #[test]
    fn command_descriptor_does_not_execute() {
        let descriptor = descriptor()
            .with_target_hint(CommandTargetHint::EntityLike)
            .with_effect_hint(CommandEffectHint::DomainMutation)
            .with_reversibility_hint(CommandReversibilityHint::DependsOnParameters);

        assert_eq!(descriptor.effect_hint(), CommandEffectHint::DomainMutation);
    }

    #[test]
    fn command_proposal_does_not_validate_against_schema_shape() {
        let _descriptor = descriptor();
        let _proposal = CommandProposal::new(
            CommandContractRef::new(contract_id(), contract_version()),
            SchemaValue::string("not structurally checked here"),
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn command_descriptor_round_trips_with_schema_refs() {
        let descriptor = descriptor()
            .with_result_schema(crate::CommandResultSchemaRef::new(
                SchemaId::from_static("scene.edit_result").unwrap(),
                SchemaVersion::new(1).unwrap(),
            ))
            .with_display_name("Edit Component Field");

        let json = serde_json::to_string(&descriptor).unwrap();
        let round_trip: CommandDescriptor = serde_json::from_str(&json).unwrap();

        assert_eq!(
            round_trip.id().as_str(),
            "editor.scene.edit_component_field"
        );
        assert_eq!(
            round_trip.parameter_schema().schema_id().as_str(),
            "scene.local_transform"
        );
        assert_eq!(
            round_trip.result_schema().unwrap().schema_id().as_str(),
            "scene.edit_result"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn command_proposal_round_trips_without_losing_numeric_kind() {
        let proposal = CommandProposal::new(
            CommandContractRef::new(contract_id(), contract_version()),
            SchemaValue::unsigned_integer(7),
        );

        let json = serde_json::to_string(&proposal).unwrap();
        let round_trip: CommandProposal = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip.parameters().as_unsigned_integer(), Some(7));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn command_metadata_round_trips_preserving_order() {
        let descriptor = descriptor()
            .with_metadata_entry(CommandMetadataEntry::new("b", "second").unwrap())
            .unwrap()
            .with_metadata_entry(CommandMetadataEntry::new("a", "first").unwrap())
            .unwrap();

        let json = serde_json::to_string(descriptor.metadata()).unwrap();
        let round_trip: crate::CommandMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(round_trip.entries()[0].key(), "b");
        assert_eq!(round_trip.entries()[1].key(), "a");
    }
}
