---
title: Commands
description: Current public API and boundaries for the foundation commands crate.
status: active
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-05
related:
  - ../../design/active/foundation-commands-design.md
---

# Commands

`foundation/commands` owns portable command descriptor and inert proposal
vocabulary.

It describes requestable mutation contracts. It does not execute commands,
route proposals, validate domain meaning, register descriptors globally, grant
permissions, own undo/redo, replace ECS deferred commands, or map proposals to
concrete domain command enums.

## Public API

Core entry points from `commands::`:

- `CommandContractId`, `CommandContractVersion`, and `CommandContractRef`
- `CommandSchemaRef` and `CommandResultSchemaRef`
- `CommandDescriptor`
- `CommandProposal` and `CommandProposalId`
- `CommandTargetHint`, `CommandEffectHint`, and `CommandReversibilityHint`
- `CommandMetadata` and `CommandMetadataEntry`
- `CommandIssue`, `CommandIssueCode`, and `CommandIssueSubject`

With the `diagnostics` feature, command vocabulary issues can be projected into
`foundation/diagnostics` reports.

## Invariants

- Command contract IDs are non-empty, whitespace-free, and use stable
  identifier characters.
- Command contract versions start at `1`.
- Command schema references point to schema IDs plus non-zero schema versions.
- Metadata keys are non-empty and duplicate keys are rejected.
- Proposals preserve contract references and schema value parameters without
  validating domain meaning.

## Boundary

Owning domains define concrete command meaning, parameter interpretation,
validation, ratification, execution, history, undo/redo, and routing.
`foundation/commands` remains inert vocabulary.
