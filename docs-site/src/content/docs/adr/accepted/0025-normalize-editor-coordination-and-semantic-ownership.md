---
title: Normalize Editor Coordination and Semantic Ownership
description: Accepted decision for owner-correct editor coordination, publication freshness, invocation, history, persistence, and structural UI boundaries.
status: accepted
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-09-14
related_adrs:
  - ./0001-use-domain-owned-commands.md
  - ./0003-ratification-is-domain-specific.md
  - ./0004-separate-description-from-execution.md
  - ./0005-projections-are-derived-state.md
  - ./0009-ui-interaction-formation-v2.md
  - ./0013-app-neutral-ui-composition-clean-cutover.md
  - ./0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ./0024-normalize-physical-input-observation-semantics.md
related_designs:
  - ../../design/accepted/runenwerk-editor-coordination-semantic-model.md
  - ../../design/implemented/editor-tool-suite-registry-and-workbench-host-design.md
  - ../../design/accepted/editor-native-multi-window-presentation-design.md
supersedes:
  - ../../design/superseded/editor-workspace-document-mode-panel-architecture.md
  - ../../design/superseded/editor-ui-workspace-tool-surface-architecture.md
---

# ADR 0025: Normalize Editor Coordination and Semantic Ownership

## Context

Runenwerk currently contains useful editor implementation that grew around a broad shared model: one generic document taxonomy, one `EditorSession`, one active document/mode/tool, one generic selection set, one generic history stack, workspace-oriented shell concepts, and provider/tool-surface routing.

Those contracts are current implementation facts, but they are too broad to serve as the durable semantic owner for a modular editor that coordinates independently owned scenes, assets, graphs, UI definitions, runtime views, drawing content, future domain tools, headless consumers, multiple views, and multiple windows.

The editor must coordinate semantic authorities without acquiring their source truth merely because it hosts, observes, presents, routes, saves, or edits them. Structural UI composition is already independently owned by `ui_composition`; native windows and render surfaces are independently owned by app/engine/render layers; domain commands and ratification remain owner-defined.

## Decision

Runenwerk adopts an owner-correct editor coordination model.

The governing rule is:

> Editor coordination owns editing-coordination invariants. It does not acquire ownership of edited semantic state, structural UI composition, native presentation, persistence formats, or host execution merely because it coordinates them.

The normalized coordination flow is:

```text
external semantic authorities
        |
        | explicit adapters/contracts
        v
EditorBinding
        |
        v
SurfaceSession
        |
        v
projection formation
        |
        v
publication admission
        |
        v
ProjectionPublication
        |
        v
InteractionRoute
        |
        v
InvocationContext
        |
        v
EditorAction / provider intent
        |
        +--> EditorLocalTransition
        +--> ExternalMutationProposal
        +--> HostRequest
        +--> explicit OperationGroup
```

Structural placement is parallel rather than subordinate:

```text
SurfaceSession
    |
PresentationAttachment
    |
MountedUnitId
    |
ui_composition
```

Neither hierarchy owns the other.

## Orthogonal Coordination Contexts

The following concepts are independently modeled and explicitly related:

- `EditorBinding`: one editor-local relationship to one owner-governed semantic scope;
- `SurfaceSession`: one local editing/viewing session that may consume one or more bindings;
- `PresentationAttachment`: the attachment of a session to structural presentation;
- `ActivationScope`: the local activation/focus/routing scope used for actions;
- `SelectionContext`: an explicitly owned selection protocol when selection exists;
- `HistoryContext`: an explicitly owned undo/redo/history target when history exists;
- `PersistenceContext`: an explicitly owned save/revert/close persistence unit when persistence exists.

Equal labels, paths, IDs, URIs, values, or current observations do not imply shared authority. Sharing must be explicit.

An `EditorBinding` is not a universal document, editor object, file, URI, persistence unit, selection unit, history unit, or global object identity. A concrete owner may map those concepts together only within its own contract.

## Editor Session Lifecycle

The conceptual editor-local lifecycle is:

```text
Configuring
  -> TopologyPrepared
    -> Active
      -> Quiescing
        -> Closed
```

`TopologyPrepared` means required static editor-coordination relationships and contribution topology are validated and frozen for the session. It is editor-local terminology and is not the App/runtime `Prepared` lifecycle state.

Ordinary contribution topology is stable while `Active` unless a separately accepted dynamic-topology architecture explicitly allows change.

This lifecycle does not imply one global active document, surface, mode, selection, history, or persistence target.

## Activation and Invocation

There is no mandatory session-global active editor target.

One editor session may contain multiple `ActivationScope`s. User or automation-facing intent executes against an immutable `InvocationContext` that captures only the editor-routing facts required for that invocation. Later focus or activation changes do not retarget in-flight work.

`InvocationContext` does not snapshot or freeze external owner state. Owners revalidate their own current preconditions when processing proposals.

## Editor Actions and Domain Commands

`EditorAction` represents stable editor-facing intent. It is not a universal domain command.

One action may resolve contextually to:

- `EditorLocalTransition`;
- `ExternalMutationProposal`;
- `HostRequest`;
- an explicit `OperationGroup` whose coordinator owns the fan-out.

Exactly one ordinary action handler must resolve unless an explicit coordinator owns multi-target behavior. Availability is advisory/derived and must be revalidated at invocation.

Domain commands, mutation semantics, ratification, effect rules, and semantic validation remain owner-defined.

## Projection Publication and Freshness

Projection computation may execute asynchronously. Completion order never establishes publication authority.

The required lifecycle is:

```text
formation inputs
  -> ProjectionAttempt
    -> projection candidate
      -> publication admission
        -> accepted ProjectionPublication
```

Publication admission compares the attempt to the `SurfaceSession`'s currently admissible formation intent/input identity and policy. Only an admitted candidate becomes visible and only that accepted publication receives the next `ProjectionGeneration` for the session.

Older or otherwise superseded work completing later must not overwrite a newer admitted publication. Coherent publication state becomes visible atomically.

`ProjectionGeneration` proves only editor-projection freshness inside the relevant `SurfaceSession`. It is not:

- a global editor revision;
- an owner semantic revision;
- a persistence version;
- a `ui_composition` revision;
- a native-window generation;
- a replacement name for the current implementation's shell-wide `projection_epoch`.

External owners independently validate owner-native concurrency evidence such as revisions, snapshots, generations, compare tokens, leases, transactions, or version vectors.

## Interaction Routes

`InteractionRoute` is:

- scoped to a `SurfaceSession`;
- scoped to one accepted `ProjectionGeneration`;
- ephemeral;
- opaque to generic editor coordination;
- non-durable.

A route from a superseded publication must not submit foreign mutation unless the owner/provider supplies an explicit safe-remap contract.

Presentation-local identities such as `WidgetId` terminate at presentation adapters and are not promoted to editor or domain semantic identity.

## Selection

Selection is an optional protocol, not a mandatory editor-global state model.

Generic editor coordination does not define a universal enum of semantic targets such as Entity, Asset, Component, Resource, GraphNode, or similar domain concepts.

Providers/owners expose semantic addresses with explicit validity scope. Expired addresses are removed or explicitly remapped. Identity reuse must never silently retarget a selection.

## Tool and Interaction State

The model distinguishes:

```text
ToolSession
InteractionSession
owner transaction
history grouping
```

It also distinguishes:

- local abandon;
- pre-send cancellation;
- cancellation request for running work;
- owner rollback;
- compensation;
- undo.

These concepts must not be collapsed into a generic editor transaction or undo mechanism.

## External Mutation and Effect Knowledge

Request lifecycle, owner/application disposition, and effect knowledge are independent facts.

At minimum distinguish:

```text
request formed
request submitted
transport received
owner accepted/queued request
effect committed
new authoritative state observed
```

Transport acknowledgement, queue admission, or request acceptance does not establish that an effect committed.

Effect knowledge is at least:

- `ConfirmedNoEffect`: owner-defined evidence proves the attempt could not have produced an effect;
- `ConfirmedEffect`: owner-defined evidence proves an effect occurred;
- `Unknown`: the available evidence cannot prove either outcome.

Observing later state equal to requested state does not alone prove causality. Unknown-effect attempts must not be blindly retried unless the owner provides real idempotency/replay semantics.

`OperationGroup` preserves per-operation attempt lifecycle, owner disposition, and effect knowledge. Grouping does not manufacture atomic success or shared transaction semantics across unrelated authorities.

## Observation Closes the Loop

The normal foreign-mutation loop is:

```text
editor intent
  -> owner proposal
    -> owner processing
      -> effect knowledge
        -> new owner observation
          -> projection formation
            -> publication admission
              -> new publication
```

The editor does not manufacture foreign source truth from user intent, provider intent, optimistic presentation, or transport acknowledgement.

## History

There is no universal `EditorSession` history stack in the target model.

History is represented through explicit `HistoryContext`s. Valid shapes include:

- owner-managed history;
- editor-coordinated opaque owner history;
- no history.

Global Undo/Redo UI actions are `EditorAction`s that resolve to exactly one `HistoryContext` or fail closed.

History grouping is separate from history compression. Structural composition history remains owned independently by `ui_composition`.

## Persistence and Reopen

`PersistenceContext` represents one explicitly owned persistence/save/close unit and is orthogonal to `SurfaceSession` and `EditorBinding` identity.

Keep distinct:

- Save;
- Save As;
- Save All;
- Revert;
- recovery backup;
- editor-session restore;
- `ui_composition` persistence;
- history;
- close.

A successful save applies to an owner-defined persisted state cut. Saving state N does not mark later state N+1 clean.

Runtime editor identities are not durable reopen identity.

When reopening is required, an owner/provider may expose an opaque, versioned conceptual `PersistentLocator`. Generic editor coordination does not require it to be a path, URI, UUID, asset id, registry key, or one universal Rust type.

Editor-session/view restore may persist only explicitly durable editor data and owner-defined durable references. It must not reinterpret these as durable semantic identity:

- `EditorBindingId`;
- `SurfaceSessionId`;
- `ActivationScopeId`;
- `InteractionRoute`;
- projection-generation-local identity;
- `WidgetId`;
- native runtime window handles.

Restore may explicitly fail, be unavailable, be denied, or be ambiguous.

## Close and Release

The following are distinct operations:

```text
detach PresentationAttachment
close SurfaceSession
release EditorBinding
close PersistenceContext
close native window
close project
quit application
```

Releasing a binding or closing a session must not erase unresolved possibly-effectful work. Every in-flight mutation attempt must be settled, safely cancelled before effect, transferred to retained monitoring, or remain explicitly indeterminate with observable policy/diagnostics.

## Cross-Authority Surfaces

One `SurfaceSession` may consume several `EditorBinding`s. The consuming provider owns admission of those observations into one presentation.

The model does not assume that `latest(A) + latest(B)` is mutually consistent. Cross-authority correlation requires explicit correspondence, lineage, snapshot, or owner-provided consistency evidence. There is no global editor revision.

## Structural UI Boundary

`ui_composition` remains the structural authority for:

- presentation targets;
- roots and regions;
- split/stack/overlay/mount topology;
- `MountedUnitId`;
- structural transactions;
- structural undo/redo;
- composition persistence and promotion.

Editor coordination must not recreate these invariants.

## Native Presentation and Host Boundary

Editor coordination may relate sessions and activation scopes to logical editor windows, but it does not own native OS handles, window-runtime lifecycle, render surfaces, swapchains, renderer execution, app lifecycle, or host execution authority.

Multiple windows may explicitly share bindings, selection contexts, history contexts, and persistence contexts. Each window retains independent activation/focus/presentation state unless an explicit policy says otherwise.

Read-only, headless, multi-view, and multi-window configurations are first-class. No fake mutation capability, file identity, global GUI focus, or native window is required.

## Predecessor Disposition

The former Workspace/Document/Mode/Panel and Editor UI Workspace/Tool-Surface architecture documents are superseded as normative editor-coordination authority.

Their implementation-era evidence remains useful for current code archaeology and migration. Surviving responsibilities are owned by:

- structural composition: ADR 0013 and `ui_composition`;
- tool-suite/provider host contracts: the implemented Tool Suite Registry and Workbench Host design;
- normalized editor coordination: this ADR and its companion semantic model;
- UI substrate/runtime semantics: current UI owners;
- native-window/render-surface mechanics: their existing app/engine/render owners.

## Consequences

Current Rust is not claimed to conform merely because this ADR is accepted. Existing `editor_core` document/session/selection/history contracts and other predecessor-shaped seams are implementation to migrate under separately activated work.

Future editor-boundary repair must remove duplicate semantic authority rather than alias or mirror the predecessor model.

This ADR does not authorize:

- a standalone `dornglut/runen-editor` repository;
- a new universal editor framework crate;
- Rust implementation changes;
- compatibility aliases or forwarding APIs;
- relocation of domain semantics into editor coordination;
- reopening `ui_composition`, native-window, render-surface, input-observation, or app-runtime ownership decisions.

Implementation requires its own bounded issue, current-source review, exact accepted base, focused tests, and exact-head repository validation.

## Companion Model

The detailed normalized model, current-source census, identity/lifetime taxonomy, failure/time/consistency semantics, predecessor and source disposition, fitness functions, proving consumers, migration constraints, and extraction gate are defined in:

[`runenwerk-editor-coordination-semantic-model.md`](../../design/accepted/runenwerk-editor-coordination-semantic-model.md).
