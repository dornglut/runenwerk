---
title: Foundation Commands
description: Design direction for a foundation-level command vocabulary crate.
status: implemented
owner: foundation
layer: foundation
canonical: true
last_reviewed: 2026-05-16
related_adrs: []
---

# Foundation Commands

## Purpose

`foundation/commands` exists to define a small, reusable vocabulary for describing requestable mutation contracts and inert command proposals.

It does not define how commands execute. It does not define editor governance. It does not replace existing domain command enums. It does not replace ECS deferred commands. It does not give AI, tools, adapters, or editor panels a privileged mutation path.

The crate should let domains, editor tools, scripts, apps, adapters, and AI integrations describe command contracts and command proposals in a common way:

```text
Which command contract is being requested?
Which version of that contract is being targeted?
Which parameter schema does the command contract use?
Which portable parameter values are being proposed?
Which optional result schema describes returned data?
Which metadata helps tools display or route the proposal?
```

The long-term relationship is:

```text
schema describes data shape
commands describe requestable mutation contracts and inert proposals
domains define concrete command meaning
ratifiers decide acceptance
diagnostics explain observations
runtime/apps execute through owning boundaries
AI/editor/tools use the same contracts as everyone else
```

## Critical premise check: should this crate exist now?

Yes, design is justified now and the initial vocabulary implementation is complete.

The crate is justified because `foundation/schema` Phase 1 through Phase 4 are complete. Schema now provides stable vocabulary for ids, versions, paths, values, descriptors, metadata, diagnostics projection, one inspector interop consumer, and one domain-owned descriptor publication. That makes command descriptors practical without inventing shape/value vocabulary inside command APIs.

The crate is not justified as an executor, router, registry, command bus, patch format, transaction system, permission system, or AI runtime.

The strongest reason to add it is cross-domain discoverability:

```text
editor_scene commands
editor_shell actions
future gameplay commands
future asset/import commands
future tool/AI proposals
future UI-surface command-like operations
```

Each owner should keep its concrete command enum and execution path. The foundation crate only supplies shared descriptor/proposal vocabulary.

## Current implementation status

Phase 0, Phase 1, Phase 2, Phase 3, Phase 4, and Phase 5 are complete.

Implemented Phase 1 crate:

```text
foundation/commands
```

Implemented Phase 1 vocabulary:

```text
contract ids
contract versions
contract refs
schema refs
descriptors
proposals
target/effect/reversibility hints
metadata
issues
prelude
serde feature
no_std/alloc/std features
tests
optional diagnostics bridge
```

Implemented Phase 3 descriptor:

```text
domain/editor/editor_scene publishes editor.scene.edit_component_field
```

Implemented Phase 4 adapter:

```text
CommandProposal -> editor_scene::SceneCommandIntent::EditComponentField
```

Phase 5 evaluation:

```text
no shared ratification or diagnostics helper is justified yet
```

Not yet implemented:

```text
registries
top-level proposal target paths
proposal-carried parameter schema claims
ratification dependency
command execution
adapter rejection diagnostics
```

## Current repo evidence

### Architecture doctrine

Runenwerk’s doctrine is:

```text
AI proposes.
Domains validate.
Ratifiers check.
Diagnostics explain.
Tests protect.
Schemas describe.
Inspection views expose.
Commands mutate.
```

This means command mutation is real, but command authority belongs to the owning domain/runtime boundary, not to a vocabulary crate.

### AI doctrine

AI must use the same contracts as humans, tests, editor tools, and scripts. There is no privileged AI mutation path.

`foundation/commands` should help AI produce explicit proposals, but it must not let AI execute those proposals.

### Existing editor command execution

`domain/editor/editor_core` currently owns executable command contracts:

```text
CommandId
CommandMetadata
CommandOutcome
CommandContext
Command trait
CommandExecutor
ExecutedCommand
ExecutedTransaction
HistoryEntry integration
```

These are execution/governance concepts. They must not be moved into `foundation/commands`.

In particular, the existing `Command` trait has `apply` and `undo`, and `CommandExecutor` executes commands and transactions. That is not foundation vocabulary.

### Existing scene command ownership

`domain/editor/editor_scene` owns concrete scene command intents and executable scene commands, including:

```text
CreateEntity
DeleteEntity
ReparentEntity
AddComponent
RemoveComponent
EditComponentField
EditResourceField
RenameEntity
```

These must remain domain-owned.

`foundation/commands` may later describe these commands, but it must not own their concrete enum variants or execution semantics.

### Existing ECS command meaning

ECS has command-like deferred structural mutation semantics. Those are not the same as editor/domain command descriptors. `foundation/commands` must not replace `ecs::Commands`, ECS command queues, or stage-boundary flushing.

### Existing schema relationship

`foundation/schema` already states that future command descriptors should primarily reference `SchemaId + SchemaVersion`, while command proposals may carry `SchemaValue` parameters.

`foundation/schema` must not define command descriptors, command proposal types, command outcomes, or command execution APIs. Those belong to `foundation/commands` or owning domains.

## Alternatives considered

### Alternative 1: no new crate

Keep all command description vocabulary inside existing domains.

Pros:

```text
no new foundation dependency
no risk of over-abstracting
domain ownership remains obvious
```

Cons:

```text
AI/tools cannot discover commands uniformly
descriptors duplicate across editor_scene, future gameplay, assets, and UI tools
schema references are re-invented per domain
proposal envelopes drift across domains
```

Verdict: acceptable temporarily, but not the best long-term path after schema Phase 1-4.

### Alternative 2: put command descriptors in `editor_core`

Pros:

```text
editor_core already owns command governance
near existing command metadata and history concepts
```

Cons:

```text
would make editor_core too central
would mix editor governance with foundation-level proposal vocabulary
would make non-editor domains depend on editor concepts
would blur editor commands, ECS commands, and future gameplay/tool commands
```

Verdict: rejected.

### Alternative 3: put command descriptors in `foundation/schema`

Pros:

```text
command parameters use schema
schema already owns ids, paths, values, descriptors
```

Cons:

```text
schema would stop being shape vocabulary
schema would gain mutation semantics
schema docs explicitly forbid command descriptor/proposal/outcome APIs
```

Verdict: rejected.

### Alternative 4: create `foundation/commands`

Pros:

```text
keeps command description separate from schema description
keeps execution out of foundation
allows cross-domain proposal vocabulary
supports future AI/tooling without privilege
keeps concrete commands domain-owned
```

Cons:

```text
must be tightly scoped
easy to overreach into registries/execution/provenance/capabilities
must avoid name collisions with existing editor_core::Command
```

Verdict: accepted as design direction.

## Scope

`foundation/commands` owns shared vocabulary for:

```text
command contract identity
command contract version
command contract references
command descriptor metadata
command parameter schema references
optional result schema references
command proposal envelopes
proposal parameter values
proposal metadata
proposal self-invariant issues
optional diagnostics projection for command-definition/proposal-shape issues
optional serialization
```

The crate may define:

```text
CommandContractId
CommandContractVersion
CommandContractRef
CommandDescriptor
CommandSchemaRef
CommandResultSchemaRef
CommandProposal
CommandProposalId
CommandMetadata
CommandMetadataEntry
CommandTargetHint
CommandEffectHint
CommandReversibilityHint
CommandIssue
CommandIssueCode
CommandIssueSubject
```

The crate may validate only vocabulary self-invariants:

```text
contract id is non-empty
contract id has stable syntax
contract version is non-zero
schema references have non-zero versions
metadata keys are non-empty
metadata keys are unique
metadata order is deterministic
proposal references a non-empty command contract
proposal carries portable parameters
```

It must not validate domain meaning.

## Non-scope

`foundation/commands` must not own:

```text
command execution
command routing
command buses
global command registries
domain command catalogs
editor governance
editor history
undo/redo
transaction retention
reconciliation
permission checks
capability enforcement
runtime scheduling
ECS command queues
reflective mutation
schema-driven mutation
patch formats
transaction formats
domain validation
domain ratification rules
AI agents
LLM clients
prompts
tool orchestration
backend adapters
persistence migrations
network protocols
```

It must not introduce:

```text
UniversalCommand
GlobalCommandExecutor
GlobalRegistry
GlobalCommandRegistry
EditorCommandBus
AiCommandRunner
ReflectionMutator
SchemaDrivenExecutor
AnyDomainState
EngineObject
```

It must not provide an API equivalent to:

```text
proposal.apply()
descriptor.execute()
global_registry.dispatch(proposal)
schema_path.set(value)
ai_runner.execute(command)
```

A command proposal is inert portable request data. It is not authority, not a patch, not a transaction, not permission, and not execution.

## Architectural position

`foundation/commands` belongs in the foundation layer:

```text
foundation/id
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
    ↓
domain crates
    ↓
engine/runtime
    ↓
apps/adapters/tools
```

It may depend on lower foundation vocabulary crates only.

Initial dependency recommendation:

```text
foundation/commands -> foundation/schema
```

Reason: command descriptors and proposals need schema references and schema values.

Initial non-dependencies:

```text
foundation/commands -/-> foundation/ratification
foundation/commands -/-> foundation/diagnostics by default
foundation/commands -/-> editor_core
foundation/commands -/-> editor_scene
foundation/commands -/-> ecs
foundation/commands -/-> engine
foundation/commands -/-> apps
```

A later optional diagnostics feature is allowed. A direct ratification dependency should be deferred until real duplication appears in multiple command-proposal ratification adapters.

## Dependency rules

Recommended feature shape:

```toml
[features]
default = ["std"]
std = ["alloc", "schema/std"]
alloc = ["schema/alloc"]
serde = ["dep:serde", "alloc", "schema/serde"]
diagnostics = ["dep:diagnostics", "alloc"]

[dependencies]
schema = { path = "../schema", default-features = false }
serde = { version = "1", features = ["alloc", "derive"], optional = true, default-features = false }
diagnostics = { path = "../diagnostics", default-features = false, features = ["alloc"], optional = true }
```

The exact Cargo syntax should be checked during implementation against current workspace conventions.

Phase 1 should not depend on diagnostics unless explicitly revised before implementation.

## Core concepts

### Command contract

A command contract is a stable requestable mutation contract.

It is not a Rust trait.

It is not executable.

It is not a registry entry.

It is not permission.

It is not ratification.

It is not a transaction.

It is not provenance.

### Command descriptor

`CommandDescriptor` describes one command contract.

It may contain:

```text
id: CommandContractId
version: CommandContractVersion
display_name: Option<String>
description: Option<String>
parameter_schema: CommandSchemaRef
result_schema: Option<CommandResultSchemaRef>
target_hint: Option<CommandTargetHint>
effect_hint: CommandEffectHint
reversibility_hint: CommandReversibilityHint
metadata: CommandMetadata
issues: Vec<CommandIssue>
```

A descriptor answers:

```text
what can be requested?
which parameter schema describes it?
which result schema may describe returned data?
how should tools display it?
what broad non-authoritative hints help tools?
```

It does not answer:

```text
who may execute this?
how is it executed?
which Rust enum is constructed?
is the current runtime state valid for it?
is the proposal accepted?
```

Descriptor availability is not permission. A UI/tool seeing a descriptor does not mean the current actor or runtime state may execute the command.

### Command contract identity

`CommandContractId` is a stable string identifier.

Examples:

```text
editor.scene.create_entity
editor.scene.rename_entity
editor.scene.edit_component_field
ui.surface.mount_surface
world_ops.mark_chunk_dirty
```

Rules:

```text
non-empty
no whitespace
stable formatting
not runtime allocated
not globally registered by foundation
```

Do not use `CommandId` as the foundation name. `editor_core::CommandId` already exists and is runtime/execution-facing.

### Command contract version

`CommandContractVersion` is a non-zero version number for a command contract.

It is distinct from:

```text
SchemaVersion
editor_core::CommandId
TransactionId
RatificationId
CausalityId
```

Reason: command contracts can change independently from parameter schemas.

### Command contract reference

`CommandContractRef` references one command contract by id and version:

```text
id: CommandContractId
version: CommandContractVersion
```

This is the only authoritative contract reference inside a Phase 1 `CommandProposal`.

### Command schema reference

`CommandSchemaRef` references a schema by:

```text
SchemaId
SchemaVersion
```

Descriptors should primarily use schema references, not inline schema descriptors.

Inline schema descriptors are deferred. They may be allowed later only for tests, generated docs, explicitly local/private descriptors, or a concrete need documented by the owning domain.

### Command proposal

`CommandProposal` is a portable request to invoke a command contract.

Phase 1 shape:

```text
proposal_id: Option<CommandProposalId>
contract: CommandContractRef
parameters: SchemaValue
metadata: CommandMetadata
```

A proposal does not execute.

A proposal does not imply acceptance.

A proposal does not imply the descriptor exists in a global registry.

A proposal does not imply the parameter value matches the parameter schema.

A proposal does not imply permission.

A proposal does not imply actor provenance.

A proposal is never interpreted by foundation.

A proposal is not a patch format.

A proposal is not a transaction format.

A proposal is not a permission-bearing object.

Target identity belongs inside the schema-described parameter value unless a later consumer proves that a separate routing hint is required.

A proposal may not repeat the descriptor's parameter schema in Phase 1. The descriptor owns the parameter schema reference. If offline or external workflows later need a proposal-carried schema claim, that should be introduced as an explicit non-authoritative `claimed_parameter_schema: Option<CommandSchemaRef>` after a real consumer proves the need.

### Command proposal identity

`CommandProposalId` is an optional externally supplied stable id.

Foundation must not allocate proposal IDs.

Phase 1 should use an opaque non-empty string wrapper if a proposal id is implemented.

Do not create a numeric allocator in foundation.

### Command target hint

`CommandTargetHint` is descriptor display/routing metadata, not an object model.

It must not carry concrete entity/component/resource ids in Phase 1.

Initial safe variants:

```text
Unspecified
DocumentLike
EntityLike
ComponentLike
ResourceLike
PathAddressed
External
Custom(String)
```

These are intentionally weak. Concrete target identity belongs in the parameter schema and owning domain proposal adapter.

### Command effect hint

`CommandEffectHint` describes broad expected effect for tools.

Initial variants:

```text
Unknown
NoMutationExpected
SessionMutation
DomainMutation
ExternalSideEffect
```

This is descriptive only. It is not enforcement.

### Command reversibility hint

`CommandReversibilityHint` describes what tools may expect about undo/redo support.

Initial variants:

```text
Unknown
Reversible
Irreversible
DependsOnParameters
```

This does not implement undo/redo.

### Command result schema

`CommandResultSchemaRef` describes optional returned data shape.

Phase 1 should not define a full `CommandOutcome`.

Reason: outcome semantics are too close to execution, ratification, diagnostics, history, and runtime state.

Acceptable Phase 1 result vocabulary:

```text
result_schema: Option<CommandResultSchemaRef>
```

Deferred:

```text
CommandExecutionResult
CommandOutcome
CommandStatus
CommandError
CommandRejection
```

Those should wait until a real consumer proves the shape.

### Command metadata

Use one deterministic ordered metadata type for both descriptors and proposals:

```text
CommandMetadata
CommandMetadataEntry
```

Descriptor metadata is display/tooling metadata.

Proposal metadata is caller/tooling metadata.

Neither carries authority, provenance, permission, trust, or AI privilege.

Metadata may carry display/tooling facts:

```text
category
group
documentation
shortcut_hint
danger_hint
```

Metadata must not carry:

```text
authority
trust
capability enforcement
permission
AI privilege
provenance
```

### Command issue

`CommandIssue` describes malformed command vocabulary.

It should carry:

```text
code: CommandIssueCode
subject: CommandIssueSubject
message: String
```

`CommandIssueSubject` should distinguish at least:

```text
ContractId
ContractVersion
ContractRef
SchemaRef
Descriptor
Proposal
ProposalId
Metadata
```

Initial issue codes should cover only vocabulary self-invariants:

```text
command.contract_id.empty
command.contract_id.invalid
command.contract_version.zero
command.schema_ref.invalid
command.metadata.key_empty
command.metadata.duplicate_key
command.descriptor.invalid
command.proposal.invalid
```

These are not domain command rejections.

## Public vocabulary shape

Target module structure:

```text
foundation/commands/src/lib.rs
foundation/commands/src/id.rs
foundation/commands/src/version.rs
foundation/commands/src/schema_ref.rs
foundation/commands/src/descriptor.rs
foundation/commands/src/proposal.rs
foundation/commands/src/hint.rs
foundation/commands/src/metadata.rs
foundation/commands/src/issue.rs
foundation/commands/src/prelude.rs
```

Optional later:

```text
foundation/commands/src/diagnostic.rs
```

Deferred unless a real consumer proves the need:

```text
foundation/commands/src/namespace.rs
foundation/commands/src/lifecycle.rs
```

Recommended prelude exports:

```text
CommandContractId
CommandContractVersion
CommandContractRef
CommandDescriptor
CommandSchemaRef
CommandResultSchemaRef
CommandProposal
CommandProposalId
CommandTargetHint
CommandEffectHint
CommandReversibilityHint
CommandMetadata
CommandMetadataEntry
```

Do not export diagnostics bridge helpers from the prelude unless they prove common.

## Relationship to schema

`foundation/commands` should depend directly on `foundation/schema`.

Command descriptors use schema for parameter and result description:

```text
CommandDescriptor
  parameter_schema: CommandSchemaRef
  result_schema: Option<CommandResultSchemaRef>
```

Command proposals use schema for values:

```text
CommandProposal
  parameters: SchemaValue
```

`foundation/commands` must not perform generic `SchemaValue`-against-`SchemaShape` validation in Phase 1.

Correct flow:

```text
SchemaDescriptor describes parameter shape.
CommandDescriptor references parameter schema.
CommandProposal carries SchemaValue parameters.
Owning domain maps proposal to concrete intent/command.
Owning domain ratifies or rejects.
Owning execution path mutates or does nothing.
```

Forbidden flow:

```text
CommandProposal + SchemaDescriptor -> automatic mutation
```

## Relationship to diagnostics

Diagnostics explain command-description or proposal-shape issues.

Phase 1 should not require diagnostics.

Phase 2 adds an optional diagnostics feature that projects command issues into diagnostic reports by explicit caller action.

Correct relationship:

```text
CommandIssue -> optional Diagnostic
DomainCommandRejection -> owning domain Diagnostic
RatificationIssue -> optional Diagnostic via owning domain
```

Forbidden relationship:

```text
Diagnostic severity decides whether a command proposal executes.
```

## Relationship to ratification

Ratification answers whether a candidate is accepted.

Command proposals are candidates.

However, `foundation/commands` should not depend directly on `foundation/ratification` in Phase 1.

Correct relationship:

```text
CommandProposal is submitted to owning domain.
Owning domain maps proposal to a domain candidate.
Owning domain ratifier returns RatificationReport.
Owning runtime/app decides whether to execute through existing boundary.
```

Forbidden relationship:

```text
foundation/commands accepts/rejects domain command semantics.
```

A future optional bridge can be reconsidered only after multiple domains duplicate command-proposal ratification wrappers.

## Relationship to editor_core

`editor_core` owns editor governance and executable command contracts.

Do not replace or move:

```text
editor_core::Command
editor_core::CommandExecutor
editor_core::CommandContext
editor_core::CommandMetadata
editor_core::CommandOutcome
GoverningChangeError
RatifiedChange
HistoryEntry
TransactionMetadata
```

`foundation/commands` may describe editor command contracts. It must not execute them.

Correct relationship:

```text
foundation/commands::CommandDescriptor describes a requestable editor command.
editor_scene owns SceneCommandIntent.
editor_core owns executable Command trait and command governance.
apps/runenwerk_editor orchestrates execution through current runtime boundary.
```

## Relationship to editor_scene

`editor_scene` owns concrete scene command intents and executable scene commands.

It may later publish descriptors for:

```text
CreateEntity
RenameEntity
EditComponentField
AddComponent
RemoveComponent
```

Implemented descriptor:

```text
editor_scene publishes descriptor for EditComponentField
```

Reason:

```text
it already uses path/value payloads
schema interop already exists in editor_inspector
scene.local_transform descriptor already exists
it proves descriptor usefulness without changing execution
```

The descriptor is published without changing execution and without converting proposals to `SceneCommandIntent`.

## Relationship to ECS

ECS has its own deferred command queue semantics.

Do not merge ECS deferred commands into `foundation/commands`.

Correct distinction:

```text
foundation/commands = portable proposal/descriptor vocabulary
ecs::Commands = deferred structural world mutation mechanism
```

ECS may later publish descriptors for tooling, but ECS execution must remain ECS-owned.

## Relationship to runtime/apps/adapters

Runtime/apps/adapters may:

```text
list descriptors from owning domains
display command descriptors
collect proposal parameters
serialize command proposals
route proposals to owning domains
show diagnostics/ratification failures
```

They must not:

```text
execute proposals without owning-domain mapping
use schema paths as mutation backdoors
define core domain invariants
invent global command registries
give AI direct runtime mutation access
```

## Relationship to AI/editor tooling

AI integrations belong in apps/tools/adapters, not foundation or pure domain crates.

AI may produce `CommandProposal` values.

AI must not execute them.

Correct AI flow:

```text
tool/app exposes allowed domain-owned descriptors
AI proposes CommandProposal
owning adapter maps proposal to domain intent if supported
owning domain ratifies
existing execution path applies if accepted
diagnostics explain rejection
history/ratified change log records accepted mutation where applicable
```

Forbidden AI flow:

```text
AI emits path/value pair
foundation/commands applies it directly
runtime mutates arbitrary state
```

## ID policy

Use strings for stable public command contract identity:

```text
CommandContractId("editor.scene.edit_component_field")
```

Reasons:

```text
stable across runs
human-readable
AI/tool friendly
serializable
does not require a global allocator
does not require a global registry
works across crates/apps/adapters
```

Use numeric versions for contract versions:

```text
CommandContractVersion(1)
```

Reasons:

```text
simple ordering
simple compatibility checks
non-zero invariant
clear migration semantics
```

Use optional opaque string proposal IDs for external correlation:

```text
CommandProposalId("01HX...")
```

Reasons:

```text
foundation should not allocate proposal ids
external tools may already have UUID/ULID/string correlation ids
proposal id is correlation, not authority
```

Runtime-local execution/history IDs may remain numeric in their owning crates.

## Versioning and compatibility

`CommandContractVersion` changes when the command contract changes.

Compatible examples:

```text
display label changes
description changes
non-authoritative metadata changes
additional optional metadata is added
documentation changes
```

Potentially breaking examples:

```text
required parameter shape changes
parameter meaning changes
result schema meaning changes
effect hint changes in a stricter direction
reversibility hint changes in a stricter direction
a descriptor starts requiring a different owning-domain interpretation
```

`foundation/commands` records versions. It does not decide global migration compatibility.

Owning domains decide how to handle older command contract versions.

## Serialization policy

The crate should support optional `serde`.

Serialization must preserve:

```text
contract id
contract version
schema references
proposal id
proposal parameter value numeric kind
metadata order
issue order
```

Serialized output must not depend on hash-map iteration order.

## no_std / alloc policy

The crate should be `no_std` capable with `alloc`.

Recommended feature shape:

```text
default = ["std"]
std = ["alloc", "schema/std"]
alloc = ["schema/alloc"]
serde = ["dep:serde", "alloc", "schema/serde"]
diagnostics = ["dep:diagnostics", "alloc"]
```

Reason: foundation vocabulary should not force runtime assumptions downward.

## Validation policy

`foundation/commands` may validate only its own vocabulary invariants:

```text
non-empty ids
valid id syntax
non-zero versions
duplicate metadata keys
deterministic ordering
schema ref well-formedness
proposal self-consistency
```

It must not validate:

```text
parameters against schema shape
domain existence
entity existence
component existence
resource existence
permission/capability
ratification acceptance
command executability
undo availability
runtime state preconditions
```

## Error/diagnostic policy

Constructor and well-formedness failures should be typed errors.

Diagnostics are optional reporting projections.

Do not make diagnostics the ordinary constructor return type.

Recommended typed errors:

```text
CommandContractIdError
CommandContractVersionError
CommandSchemaRefError
CommandDescriptorError
CommandProposalIdError
CommandProposalError
CommandMetadataError
CommandIssueError
```

Optional diagnostics bridge:

```text
CommandIssue -> Diagnostic
CommandIssue iterator -> DiagnosticReport
```

## Invariants

- Command contract ids are stable and non-empty.
- Command contract versions start at 1.
- Command contract refs contain id and version.
- Schema references are explicit `SchemaId + SchemaVersion`.
- Metadata order is deterministic.
- Duplicate metadata keys are rejected.
- Proposal parameter payloads preserve `SchemaValue` numeric kind.
- A proposal never executes itself.
- A proposal is never interpreted by foundation.
- A proposal is not a patch format.
- A proposal is not a transaction format.
- A proposal is not a permission-bearing object.
- A descriptor never registers itself globally.
- Descriptor availability is not permission.
- Foundation never maps a proposal to a concrete domain enum.
- Foundation never calls `apply`, `undo`, or an executor.
- Foundation never decides domain acceptance.
- Foundation never grants AI or tools special mutation authority.

## Anti-goals

`foundation/commands` is not:

```text
an editor command executor
an ECS command queue
a global command registry
a command bus
a permission system
a capability system
a provenance system
a transaction system
an undo/redo system
a reflection mutator
a schema validator
an AI tool runtime
a persistence migration system
a patch format
```

## Examples of correct usage

### Domain-owned descriptor publication

```text
editor_scene owns SceneCommandIntent::EditComponentField
editor_scene publishes CommandDescriptor for editor.scene.edit_component_field
descriptor references schema id/version for parameters
runtime/editor displays descriptor
proposal is mapped by editor_scene/app adapter later
current editor runtime executes through existing governed command path
```

### AI proposal

```text
AI reads allowed command descriptors
AI creates CommandProposal
proposal uses command contract id/version
proposal includes SchemaValue parameters
tooling submits proposal to owning adapter
owning domain ratifies
existing command path executes if accepted
```

### Diagnostic projection

```text
CommandProposal is malformed
foundation/commands emits CommandIssue
caller explicitly projects CommandIssue to Diagnostic
diagnostic is shown in tool UI
nothing executes
```

## Examples of forbidden usage

```text
foundation/commands::execute(proposal)
GlobalCommandRegistry::dispatch(proposal)
CommandDescriptor::apply(parameters)
SchemaPath::set(value)
AI command runner inside foundation
reflection-generated mutator inside commands
editor_core::Command moved into foundation
editor_scene::SceneCommandIntent moved into foundation
ecs::Commands replaced by foundation/commands
CommandProposal used as a patch format
CommandProposal used as a transaction format
CommandProposal carrying permission authority
```

## Migration/adoption plan

### Phase 0: Design accepted

Status: complete.

Create and review:

```text
docs-site/src/content/docs/design/implemented/foundation-commands-design.md
```

No code.

### Phase 1: Core command descriptor/proposal vocabulary

Status: complete.

Implement `foundation/commands` with:

```text
contract ids
contract versions
contract refs
schema refs
descriptors
proposals
target/effect/reversibility hints
metadata
issues
prelude
serde feature
no_std/alloc/std features
tests
```

Do not add consumers.

Do not add registry.

Do not map proposals to domain intents.

Do not add top-level proposal target paths.

Do not add proposal-carried parameter schema claims.

### Phase 2: Optional diagnostics bridge

Status: complete.

Add optional diagnostics projection for command descriptor/proposal issues.

Do not add ratification dependency.

Do not decide acceptance.

### Phase 3: One domain-owned command descriptor

Status: complete.

Recommended candidate after re-inspection:

```text
editor_scene::SceneCommandIntent::EditComponentField
```

The descriptor should reference parameter schema and optional result schema.

Do not change command execution.

Do not map proposals yet.

### Phase 4: One explicit proposal-to-domain-intent adapter

Status: complete.

Add a narrow adapter in the owning domain or app boundary for one command descriptor.

Recommended candidate after re-inspection:

```text
CommandProposal -> editor_scene::SceneCommandIntent::EditComponentField
```

This must be explicit, narrow, tested, and non-reflective.

Do not add global dispatch.

Do not add AI runtime.

### Phase 5: Ratification/diagnostics integration evaluation

Status: complete.

Evaluate whether repeated command-proposal rejection flows need optional helper vocabulary.

Current verdict: no new helper vocabulary is justified yet.

Reason:

```text
only one explicit proposal-to-domain-intent adapter exists
only one adapter rejection enum exists
the rejection shape is still domain-owned
no repeated cross-domain conversion/reporting pattern exists
```

Keep `EditComponentFieldProposalError` in `editor_scene`.

Do not add a `foundation/commands -> foundation/ratification` dependency.

Do not add a generic command proposal rejection report.

Do not add adapter rejection diagnostics until at least two owning domains duplicate the same reporting shape.

Do not move:

```text
RatifiedChange
GoverningChangeError
EditorMutationError
CommandExecutor
CommandContext
HistoryEntry
```

into foundation.

### After every phase

Run the phase completion drift-check routine before starting the next phase.

## Testing strategy

### Phase 1 tests

```text
command_contract_id_rejects_empty
command_contract_id_rejects_whitespace
command_contract_version_rejects_zero
command_contract_ref_preserves_id_and_version
command_schema_ref_preserves_schema_id_and_version
command_descriptor_preserves_metadata_order
command_descriptor_rejects_duplicate_metadata_keys
command_proposal_preserves_contract_ref
command_proposal_preserves_schema_value_parameters
command_proposal_has_no_universal_target_path
command_descriptor_does_not_execute
command_proposal_does_not_validate_against_schema_shape
```

### Serde tests

```text
command_descriptor_round_trips_with_schema_refs
command_proposal_round_trips_without_losing_numeric_kind
command_metadata_round_trips_preserving_order
```

### Diagnostics tests

With `diagnostics` later:

```text
command_issue_maps_to_diagnostic_code
command_issue_maps_to_diagnostic_subject
command_issue_report_preserves_issue_order
command_diagnostic_bridge_does_not_define_acceptance
```

### Consumer tests

For `editor_scene` descriptor publication:

```text
scene_edit_component_field_descriptor_has_stable_id
scene_edit_component_field_descriptor_references_parameter_schema
scene_edit_component_field_descriptor_does_not_execute_command
```

For proposal mapping later:

```text
scene_edit_component_field_proposal_maps_to_intent_when_supported
scene_edit_component_field_proposal_rejects_descriptor_mismatch
scene_edit_component_field_proposal_rejects_unsupported_parameter_shape
proposal_mapping_does_not_bypass_ratification
```

### Validation gates

Focused:

```text
cargo fmt --all
cargo test -p commands
cargo test -p commands --no-default-features
cargo test -p commands --features alloc
cargo test -p commands --features serde
cargo clippy -p commands --all-targets --all-features -- -D warnings
```

Workspace:

```text
python3 tools/docs/validate_docs.py
./quiet_full_gate.sh
```

## Open questions

1. Should `CommandProposalId` exist in Phase 1?
   - Recommendation: yes, as optional externally supplied opaque string correlation only. Foundation must not allocate it.

2. Should Phase 1 include a true `CommandOutcome`?
   - No. Use only optional `result_schema`. Outcome semantics should wait for a consumer.

3. Should `foundation/commands` depend on `foundation/ratification`?
   - No for Phase 1. Re-evaluate only after multiple owning domains duplicate proposal-ratification wrappers.

4. Should command descriptors include capability requirements?
   - Defer. Capability requirements belong to a future capability design only if real enforcement exists.

5. Should command proposals include provenance?
   - Defer. Provenance spans origin, actor, trust, causality, replay, editor history, AI proposals, networking, and multiplayer. It should not be folded into commands prematurely.

6. Should commands expose a global registry?
   - No. Owning domains/apps may publish descriptor collections explicitly. Foundation must not own global lookup.

7. Should proposal-carried schema claims be supported?
   - Defer. The descriptor owns the parameter schema reference in Phase 1. If offline/external workflows need a proposal-carried non-authoritative schema claim later, add it explicitly as a claim, not as a second source of truth.

8. Should top-level proposal target paths be supported?
   - Defer. Target identity belongs inside domain-owned parameter schemas unless a later consumer proves a separate routing hint is required.

9. Should command lifecycle/deprecation be typed?
   - Defer. Use metadata/documentation first. Add typed lifecycle only when multiple descriptor publishers need consistent lifecycle behavior.

## Final recommendation

Keep `docs-site/src/content/docs/design/implemented/foundation-commands-design.md` as the active phase roadmap for `foundation/commands`.

Phase 0, Phase 1, Phase 2, Phase 3, Phase 4, and Phase 5 are complete.

No further command phase should start until a concrete duplicated need appears.

The clean Phase 1 target is:

```text
foundation/commands = portable command descriptor and inert proposal vocabulary
depends on foundation/schema
string contract ids
numeric contract versions
optional opaque string proposal ids
no execution
no registry
no domain mapping
no ratification dependency
diagnostics dependency optional only
no top-level proposal target path
no proposal-carried parameter schema claim
```

The long-term shape is:

```text
foundation/schema describes data shape
foundation/commands describes requestable mutation contracts and inert proposals
owning domains define concrete command meaning
owning domains ratify
existing execution paths mutate
diagnostics explain
AI/editor/tools propose through the same path as everyone else
```
