---
title: Runenwerk Editor Coordination Semantic Model
description: Accepted normalized semantic model for editor bindings, sessions, publication, invocation, selection, history, persistence, effect knowledge, and ownership disposition.
status: accepted
owner: editor
layer: domain/app
canonical: true
last_reviewed: 2026-09-16
publication: reference
pagefind: false
related_adrs:
  - ../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ../../adr/accepted/0024-normalize-physical-input-observation-semantics.md
related_designs:
  - ../implemented/editor-tool-suite-registry-and-workbench-host-design.md
  - ./editor-native-multi-window-presentation-design.md
supersedes:
  - ../superseded/editor-workspace-document-mode-panel-architecture.md
  - ../superseded/editor-ui-workspace-tool-surface-architecture.md
---

# Runenwerk Editor Coordination Semantic Model

## Status and Scope

This is the accepted target semantic model for Runenwerk editor coordination.

It is documentation authority, not an implementation-completeness claim. Current Rust remains authoritative for current behavior. Some predecessor-shaped `editor_core`, shell, app, provider, persistence, and product contracts remain current implementation; E1–E4 have already made bounded scene-selection, scene-history, scene-persistence, and viewport-tool ownership repairs. The remaining predecessor contracts require separately activated migration work.

This design does not authorize a standalone `dornglut/runen-editor` repository, a new framework crate, or Rust changes. It defines the semantic target against which later boundary repair can be evaluated.

## Governing Theorem

Editor coordination owns editing-coordination invariants. It does not acquire ownership of edited semantic state, structural UI composition, native presentation, persistence formats, or host execution merely because it coordinates them.

The editor is therefore an orchestrator of explicit owner contracts rather than the universal source of documents, selection, history, persistence, workspace state, or domain commands.

## Historical Pre-migration Source Census

At accepted Runenwerk revision `95647afca71f5d590f1d9066358d19bfabf6dffb`, the source census recorded the predecessor shape that subsequent bounded work began to migrate.

### `domain/editor/editor_core`

The predecessor implementation included:

- `document.rs`: `DocumentId`, a central `DocumentKind` enum spanning scene, material, texture, procgen, gameplay, particle, physics, animation, UI, script, asset, runtime-debug, and editor-definition families, plus `DocumentDescriptor` dirty state;
- `session.rs`: one `EditorSession` owning the document map/tab order, one active document, one active tool, one active mode, one `SelectionSet`, and one `HistoryStack`;
- `selection.rs`: one generic `SelectionTarget` enum with document/entity/component/resource/asset/custom variants. This predecessor module was removed when scene selection moved to its owner;
- `history.rs`: one generic undo/redo `HistoryStack`;
- generic command/executor/transaction, ratification, sharing, reconciliation, workflow, migration, and capability contracts.

These are facts about the cited historical revision, not claims about current Rust or the normalized target ownership model.

### Other editor implementation owners

- `editor_definition` owns current durable editor-definition schemas and pure definition/formation helpers. Its concrete authored families remain owner-specific inputs rather than proof of one universal editor document model.
- `editor_inspector` owns inspector-specific targets, values, sessions, resolution, validation, and bridges.
- `editor_scene` owns scene-specific editor commands, proposals, SDF authoring, graph lowering, and scene-facing mutation semantics.
- `editor_persistence` owns concrete project/scene file DTOs, codecs, migrations, normalization, and formation. Those formats are not generic editor persistence merely because they live under `domain/editor`.
- `editor_preview` owns current external runtime-preview protocol/product vocabulary.
- `editor_viewport` owns viewport camera, hit, overlay, snap, expression, and presentation product contracts.
- `editor_shell` owns current shell/tool-host integration, provider-facing contracts, Workbench/tool-suite machinery, and composition adapters. Generic structural topology itself is owned by `ui_composition`.
- `apps/runenwerk_editor` owns concrete provider installation, app/runtime state, IO paths, host policy, target/window correlation, runtime integrations, and command execution.

Directory placement is implementation grouping evidence, not proof that every type in `domain/editor/*` is reusable framework semantics.

## Normalized Topology

The normalized semantic path is:

```text
external semantic authorities
        |
        | explicit owner adapters/contracts
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
        +--> OperationGroup
```

Structural placement is parallel:

```text
SurfaceSession
    |
PresentationAttachment
    |
MountedUnitId
    |
ui_composition
```

A semantic session can move, appear in several presentations, become temporarily detached, or exist headlessly without changing foreign semantic identity. Conversely, a structural mounted unit can be reconfigured without granting `ui_composition` ownership of the mounted content's semantic state.

## Identity and Lifetime Taxonomy

| Concept | Owner | Typical lifetime | Durable? | Semantic authority? |
| --- | --- | --- | --- | --- |
| `EditorBinding` | editor coordination | relationship to owner scope | no; may reference owner durable locator | relationship only |
| `SurfaceSession` | editor coordination/provider | local viewing/editing session | no, except explicitly durable local preferences | local coordination only |
| `PresentationAttachment` | editor/app coordination | attachment to structural presentation | no | no |
| `MountedUnitId` | `ui_composition` | composition structural state | according to composition persistence | structural only |
| `ActivationScope` | editor/app coordination | focus/routing scope | normally no | no foreign authority |
| `SelectionContext` | provider/owner or explicit coordinator | owner-defined | owner-defined | only its declared selection protocol |
| `HistoryContext` | owner or explicit coordinator | owner-defined | owner-defined | only its declared history protocol |
| `PersistenceContext` | persistence owner | owner-defined save/close unit | owner-defined | persistence only |
| `ProjectionAttempt` | surface/provider projection | computation attempt | no | no |
| `ProjectionPublication` | surface/provider projection | accepted visible projection | no | derived presentation only |
| `ProjectionGeneration` | `SurfaceSession` publication | accepted publication sequence | no | freshness only |
| `InteractionRoute` | provider/surface publication | one publication generation | no | routing token only |
| `InvocationContext` | editor routing | one invocation | no | immutable routing facts only |
| `PersistentLocator` | concrete semantic/persistence owner | owner-defined | yes when owner says so | owner-defined reopen reference |
| native window handle | engine/window runtime | runtime window | no | platform runtime only |
| `WidgetId` | UI runtime/presentation | runtime UI product | no | presentation only |

Runtime identity may never be promoted to durable semantic identity merely because it is convenient to serialize.

## EditorBinding

`EditorBinding` means one editor-local relationship to one semantic scope governed by an external owner.

A binding answers questions such as:

- which owner is observed;
- which owner contract/adaptor is in use;
- which owner-defined scope is being edited or inspected;
- which capabilities are available through this relationship;
- which owner-native observation/concurrency facts accompany observations.

It does not imply that the editor owns a document object, file, path, selection, history, save state, transaction, or command vocabulary.

Several bindings may refer to different scopes in the same owner. Several surface sessions may explicitly share one binding. One surface may consume several bindings.

## SurfaceSession

A `SurfaceSession` owns editor-local interaction and presentation coordination for a surface experience.

It may own:

- local view state;
- provider-local transient state;
- projection formation scheduling;
- accepted publication state;
- route publication;
- local interaction/tool sessions;
- explicit relationships to activation/selection/history/persistence contexts.

It does not own the edited semantic source merely because it displays it.

Read-only and headless surface sessions are valid. A surface session need not expose mutation, persistence, history, or native presentation.

## PresentationAttachment and Structural Composition

A `PresentationAttachment` connects a surface session to a structural `MountedUnitId` or other explicitly supported presentation target.

`ui_composition` owns structural state: targets, roots, regions, mounted units, ordering, structural transactions, structural history, and composition persistence.

The editor/provider owns semantic session state and content projection. Neither may silently mirror the other's authority.

Structural undo/redo is not document/domain undo/redo. Moving or splitting a mounted surface does not imply a foreign semantic mutation.

## Lifecycle

The conceptual editor-session lifecycle is:

```text
Configuring
  -> TopologyPrepared
    -> Active
      -> Quiescing
        -> Closed
```

### Configuring

Contributions, bindings, provider relationships, required coordination services, and static topology are assembled and validated.

### TopologyPrepared

Required static editor-coordination topology is valid and frozen. This is not App/runtime `Prepared`; the two lifecycles solve different problems.

### Active

Normal projection, interaction, invocation, owner proposals, and observation operate. Ordinary contribution topology remains stable unless a separately accepted architecture permits dynamic topology.

### Quiescing

New work is restricted while in-flight work, unresolved attempts, monitoring, attachments, bindings, and owner obligations are settled or transferred.

### Closed

Editor-local session resources are released. Closing does not retroactively prove that foreign work had no effect.

## ActivationScope and InvocationContext

There is no mandatory global active document/editor/mode.

An `ActivationScope` is a local routing/focus context. Separate windows, views, tool regions, automation scopes, or headless clients may have independent activation.

When an action is invoked, routing produces an immutable `InvocationContext`. It captures only facts needed to route that invocation, such as surface session, accepted projection generation, activation scope, selected context references, explicit binding references, and action-specific local routing data.

A later focus/activation change cannot retarget an in-flight invocation.

`InvocationContext` is not a foreign-state snapshot. The owner validates owner-native preconditions at processing time.

Physical/backend input authority, UI interaction semantics, activation, invocation, and editor action semantics remain separate layers. `ActivationScope` must not become a duplicate input reducer.

## EditorAction Resolution

`EditorAction` is stable editor-facing intent, suitable for UI, automation, command palette, shortcut, or similar editor-facing entry points.

Resolution may produce:

### EditorLocalTransition

Changes only editor-owned local coordination state, such as view settings, local selection presentation where locally owned, or tool-session state.

### ExternalMutationProposal

Submits an owner-defined mutation request carrying owner-native precondition/concurrency evidence where required.

### HostRequest

Requests app/window/runtime behavior such as opening presentation, starting preview, showing a native window, or other host-owned action.

### OperationGroup

An explicit coordinator groups several operations. Grouping is not evidence of cross-owner atomicity. Each member retains its own attempt lifecycle, owner disposition, effect knowledge, concurrency evidence, and observation closure.

Ordinary action resolution must be unambiguous. One handler resolves or the action fails closed. Fan-out requires an explicit coordinator.

Availability projections are advisory and are revalidated when invoked.

## Projection Formation and Admission

Projection formation is derived computation and may be asynchronous.

Each attempt records enough editor-local input identity to determine whether its result remains admissible for the target surface session. Input identity may include binding set, provider-local projection request identity, owner observation identity, relevant local view state, and other explicitly declared formation inputs.

The lifecycle is:

```text
formation intent/input identity
  -> ProjectionAttempt
    -> candidate
      -> admission check
        -> ProjectionPublication
```

Admission is a current-session policy decision. It must not use task completion order as authority.

A candidate is rejected when, for example:

- its surface session is closed/quiescing beyond acceptance policy;
- its formation intent has been superseded;
- required bindings changed incompatibly;
- relevant local projection inputs changed;
- the provider reports invalid/incomplete output;
- an owner observation/precondition makes the candidate stale according to the provider contract.

Only an accepted publication receives the next session-local `ProjectionGeneration`.

Publication of all state that must agree—presentation data, route table, route scope, diagnostics, source maps, and related projection metadata—is atomic from consumers' perspective.

## ProjectionGeneration Versus Current `projection_epoch`

Current shell code carries a `projection_epoch` through shell/provider routing. That is current implementation evidence and may remain during migration.

The normalized `ProjectionGeneration` must not be introduced as a mechanical rename or broadening of that epoch.

The normalized contract is specifically:

- owned by a `SurfaceSession` publication sequence;
- advanced only by admitted publications;
- used to reject routes/actions from superseded publications;
- not a global shell/editor revision;
- not owner semantic concurrency evidence;
- not a structural `ui_composition` revision.

Migration work must prove whether any existing epoch can be narrowed/reused locally or must be replaced. This design does not assume equivalence.

## InteractionRoute

Routes are opaque, ephemeral publication products.

A route is valid only for its declared `SurfaceSession` and `ProjectionGeneration`. Generic editor coordination does not parse route payloads into domain meaning.

A stale route fails closed unless the provider/owner supplies an explicit safe-remap rule. Safe remap must be semantic and owner-defined; equality of labels, presentation IDs, paths, or current values is insufficient.

`WidgetId` and similar UI-runtime identities terminate at the adapter that resolves presentation interaction into an `InteractionRoute` or editor action.

## Foreign Concurrency

Projection freshness and foreign mutation concurrency are different concerns.

An external owner may require:

- revision numbers;
- immutable snapshot ids;
- generation ids;
- compare tokens;
- leases;
- transaction ids;
- version vectors;
- domain-specific precondition records.

The editor carries or obtains that evidence through the owner contract. It must not substitute `ProjectionGeneration`, shell epoch, UI identity, or global editor counters.

## SelectionContext

Selection is optional and owner/protocol-specific.

A `SelectionContext` declares:

- what semantic addresses mean;
- which owner/scope validates them;
- their validity lifetime;
- primary/multi-selection rules when relevant;
- stale-address handling;
- explicit remap behavior, if any;
- sharing policy across sessions/views/windows.

Generic editor coordination does not centrally enumerate all domain target kinds.

An entity address, graph-node address, asset address, UI-definition node address, drawing object address, runtime query address, and future target kinds remain owner-defined.

Identity reuse must never silently make an expired selection target a new semantic object.

## ToolSession and InteractionSession

A `ToolSession` represents durable-enough local tool activation/configuration for a surface or activation scope.

An `InteractionSession` represents one active gesture/interaction lifecycle such as drag, paint stroke, transform preview, scrub, box selection, or similar interaction.

Neither is automatically an owner transaction or history group.

Required distinctions include:

- local preview versus committed owner mutation;
- local abandon versus remote cancellation;
- pre-send cancellation versus cancellation request after submission;
- owner rollback versus compensation;
- undo versus failed/abandoned interaction.

## MutationAttempt Model

Each external operation is tracked as an attempt with orthogonal dimensions.

### Attempt lifecycle

Example states/facts:

```text
Formed
Submitted
TransportObserved
TerminallySettled or RetainedForMonitoring
```

Exact owner/transport implementations may refine these, but attempt progress must not be confused with effect.

### Owner/application disposition

Owner-defined examples may include:

```text
Rejected
Accepted
Queued
Running
Cancelled
Completed
```

These are processing/disposition facts, not universal proof of effect unless the owner contract explicitly says so.

### Effect knowledge

```text
ConfirmedNoEffect
ConfirmedEffect
Unknown
```

`ConfirmedEffect` requires owner-defined evidence proving at least one relevant committed effect occurred for that attempt. `ConfirmedNoEffect` requires owner-defined evidence proving no effect could have occurred. Anything else remains `Unknown`.

`ConfirmedEffect` is an attempt-history fact, not a claim that the requested state remains current forever. Later compensation, rollback, undo, or reversion may change current authoritative state, but they do not rewrite the earlier attempt from `ConfirmedEffect` to `ConfirmedNoEffect`.

A later observation equal to the desired value does not by itself prove this attempt caused it.

Unknown attempts are not automatically retried. Retry requires owner-defined idempotency or replay semantics.

## OperationGroup

`OperationGroup` is an explicit editor/coordinator construct for user-visible actions requiring several operations.

It records every member independently:

- invocation/binding target;
- owner request/proposal identity;
- concurrency/precondition evidence;
- attempt lifecycle;
- owner disposition;
- effect knowledge;
- observation/settlement state;
- compensation/rollback/undo relationships where defined.

The group may expose aggregate presentation, but aggregate status cannot erase mixed per-operation facts or imply an atomic transaction across unrelated owners.

## Observation Closure

Foreign semantic truth returns through owner observation, not editor intent.

```text
intent
 -> proposal
   -> owner processing
     -> effect knowledge
       -> owner observation
         -> projection formation
           -> admission
             -> publication
```

Optimistic UI is local derived state and must remain distinguishable from authoritative observation.

## HistoryContext

History has no mandatory global editor implementation.

A `HistoryContext` may represent:

1. owner-managed history, where editor actions request owner undo/redo;
2. editor-coordinated opaque owner history handles, where editor coordination stores only owner-valid opaque entries/tokens;
3. no history.

Global Undo/Redo presentation resolves through current invocation/activation context to exactly one history context or fails closed.

History grouping and compression are separate. `ui_composition` structural history is separate. App/runtime replay is separate. Persistence recovery is separate.

## PersistenceContext

A `PersistenceContext` defines the owner and scope of persistence operations for a semantic unit.

It may expose owner-specific operations such as save, save-as, revert, close checks, snapshot, publish, or similar operations. Generic editor coordination does not prescribe one file model.

`Save`, `Save As`, `Save All`, `Revert`, recovery backup, session restore, structural composition persistence, history, and close are separate concepts.

Save success carries an owner-defined persisted-state cut. If owner state advances afterward, later state remains unsaved even if an older save succeeds.

Dirty/clean status is therefore owner/persistence-context-derived, not a universal boolean owned by `EditorSession`.

## PersistentLocator and Restore

`PersistentLocator` is a conceptual owner/provider role, not a required universal struct or registry.

When an owner supports durable reopen, it may expose an opaque, versioned locator. The editor treats it as owner data and does not infer path/URI/UUID/asset semantics.

Editor restore may persist:

- stable contribution/provider identity where contractually durable;
- owner-defined persistent locators;
- explicitly persistent local view/session preferences;
- app-owned presentation preferences when appropriate.

It must not persist runtime binding/session/activation/route/publication/widget/native-handle identities as semantic reopen identity.

Restore may fail with explicit unavailable, denied, missing, incompatible-version, ambiguous, or owner-defined outcomes.

## Close, Quiescence, and Uncertain Effects

Closing presentation, surface session, binding, persistence context, native window, project, and application are independent.

Before a surface/binding is discarded, every possibly-effectful in-flight attempt must be:

- settled with adequate owner evidence;
- safely cancelled before any effect was possible;
- transferred to retained monitoring owned by a longer-lived coordinator;
- or retained as explicitly indeterminate with user/policy-visible diagnostics.

Closing UI does not erase uncertainty.

## Cross-Authority Surface Semantics

A surface may combine several owners—for example a domain document plus runtime observation plus asset status plus diagnostics.

The provider owns admission policy for that composition.

There is no generic guarantee that separately latest observations form a consistent joint snapshot. Where consistency matters, the provider requires explicit owner correlation such as shared snapshot lineage, correspondence ids, transaction context, or another owner-defined relation.

No global editor revision is introduced to paper over independent authorities.

## Multi-View and Multi-Window

Several views/windows may explicitly share:

- `EditorBinding`s;
- `SelectionContext`s;
- `HistoryContext`s;
- `PersistenceContext`s.

Sharing is opt-in according to owner/editor policy.

Each window/view retains independent:

- `ActivationScope`;
- native/presentation focus;
- pointer capture and UI interaction state;
- local presentation/session state where not explicitly shared.

Logical editor-window identity, native-window identity, render-surface identity, and app-owned correlation remain governed by the native multi-window design and existing app/engine/render owners. This model only corrects the assumption that all windows inherently share one global document/command/history authority.

## Read-Only and Headless Consumers

A conforming editor-coordination consumer may be:

- read-only inspector;
- headless validation/automation client;
- one-window interactive editor;
- multi-view editor;
- multi-window editor;
- product-specific focused workbench;
- app-integrated tool host.

Such consumers do not need fake documents, files, save state, selection, history, mutation capability, GUI focus, or native windows when those concepts do not apply.

## Structural UI Boundary

`ui_composition` remains owner of presentation targets, roots, regions, mounted units, structural transactions, structural history, promotion, and composition persistence.

Editor contribution/provider semantics attach through `MountedUnitId` and editor-owned extension/association data. Editor coordination must not recreate a workspace topology or structural history in parallel.

## Tool Suite / Workbench Relationship

The implemented Tool Suite Registry and Workbench Host design remains authority for:

- stable surface keys;
- tool-suite contribution vocabulary;
- provider-family narrowing;
- deterministic/fail-closed provider resolution;
- provider-owned interaction proposal mapping;
- host capability policy;
- Workbench composition/integration machinery.

Those contracts do not own universal documents, selection, history, persistence, workspace structure, foreign domain commands, or foreign semantic state. Where older Workbench terminology implies those concepts, this semantic model governs coordination ownership.

## Native Multi-Window Relationship

The accepted native multi-window design remains authority for logical/native/render presentation identities and lifecycle mechanics.

Its semantic sharing rule is narrowed by this model: windows may explicitly share binding/selection/history/persistence contexts; they do not automatically inherit one global editor session/document/command/undo authority.

No native-window or render-surface ownership is reopened.

## Predecessor Disposition Matrix

| Predecessor responsibility | Disposition |
| --- | --- |
| workspace/split/tab/mounted structural topology | ADR 0013 / `ui_composition` |
| old `WorkspaceState` structural authority | superseded; migration/current-code evidence only |
| panel/tool-surface stable host vocabulary | implemented Tool Suite/Workbench design where still current |
| provider-family narrowing and provider-owned routing | implemented Tool Suite/Workbench design |
| generic Document/DocumentKind ownership | predecessor target superseded; current Rust to migrate |
| one global EditorSession document/tool/mode aggregation | viewport tool migrated in E4; document/mode predecessor remains to migrate |
| one generic selection enum/set | scene selection moved to its owner in E1; no generic replacement |
| one generic history stack | scene history moved to app-owned scene context in E2; generic stack removed |
| generic dirty/save state | generic session dirty/save/close removed in E3; scene persistence uses an explicit context |
| native window / render surface | retained existing app/engine/render owners |
| UI input/focus/interaction substrate | current UI authority and ADR 0009/0024 boundaries |
| normalized editor coordination | ADR 0025 + this design |

## Source-Disposition Matrix

This matrix describes bounded current implementation state after E1–E4; it does not assert full ADR-0025 conformance. Removed predecessor files are explicitly marked as historical.

| Current or former source area | Current role or removal | Normalized target disposition |
| --- | --- | --- |
| `editor_core/document.rs` | central `DocumentId`/`DocumentKind` taxonomy and `DocumentDescriptor` without generic dirty metadata | document taxonomy remains predecessor-shaped; owner scopes exposed through bindings/persistence contracts |
| `editor_core/session.rs` | generic document map/tab ordering, active document and active mode; no active tool, scene selection, history, or dirty/save authority | remaining document/mode predecessor to decompose into explicit coordination contexts |
| former `editor_core/selection.rs` | removed in E1; predecessor cross-domain target enum | scene-owned selection context; no generic replacement |
| former `editor_core/history.rs` | removed in E2; predecessor generic history stack | app-owned scene history context; owner-specific histories remain independent |
| `apps/runenwerk_editor/src/shell/surface_session.rs` | mounted-unit-scoped viewport tool activation and local viewport session state | retain local tool coordination; no universal active-tool authority |
| `editor_core` command/executor/transaction | generic editor mutation machinery | review case-by-case; retain only true editor-local coordination, owner commands stay owner-defined |
| `editor_definition` | editor-authored definition schemas/formation | retain owner-specific definition responsibility; do not generalize into universal edited-state authority |
| `editor_inspector` | inspector-specific contracts | retain bounded owner/adapter semantics; integrate via bindings/sessions |
| `editor_scene` | scene editor commands/proposals/SDF authoring | retain scene-owner/editor-adapter semantics; foreign domain truth does not move to core editor |
| `editor_persistence` | current editor project/scene formats | retain concrete format ownership where correct; expose persistence contexts rather than universal editor save state |
| `editor_preview` | preview protocol/product vocabulary | retain bounded preview ownership; host execution remains app/runtime-owned |
| `editor_viewport` | viewport semantic/presentation contracts | retain bounded viewport ownership; projection/session coordination uses normalized model |
| `editor_shell` tool-suite/workbench | editor host/provider machinery | retain implemented bounded responsibilities |
| `editor_shell` old workspace structures | compatibility/current-code migration input | no new normative structural authority; `ui_composition` wins |
| `apps/runenwerk_editor` | concrete providers, IO, runtime/window/app integration | retain integration ownership; do not become generic semantic owner |
| `domain/ui/ui_composition` | structural composition | retain sole structural authority |

## Migration Constraints

Later implementation must use clean owner cuts rather than aliases or mirrored authority.

Required constraints:

- no `Document` compatibility facade that preserves the same universal semantics under a new name;
- no generic `SelectionTarget::Custom` escape hatch used to pretend a universal selection model survived;
- no second history stack mirroring owner history;
- no editor-global revision substituted for owner concurrency;
- no projection generation persisted as durable semantic identity;
- no workspace structural mirror next to `ui_composition`;
- no native-window handle in editor semantic/domain state;
- no new central enum that must enumerate every future edited domain;
- no compatibility forwarding modules without explicit short-lived migration requirement and removal condition;
- no change to foreign owner semantics merely to fit editor coordination.

Migration issues must start from exact current source/tests and state the specific predecessor authority being removed, the new owner boundary, focused fitness tests, and deletion/closure condition.

## Failure, Time, and Consistency Semantics

### Failure

Failures remain typed by layer:

- projection formation failure does not mutate foreign source truth;
- publication rejection preserves the last accepted publication where policy allows;
- route rejection is not foreign mutation failure because the proposal was never validly routed;
- owner proposal rejection follows owner semantics;
- host request failure follows host/app semantics;
- persistence failure follows persistence owner semantics;
- structural transaction failure follows `ui_composition` semantics.

Generic editor coordination preserves provenance rather than flattening all errors into one status.

### Time

No correctness rule depends on wall-clock completion order. Attempt timestamps may aid diagnostics, but authority derives from explicit identity, lifecycle, admission, owner evidence, and policy.

### Consistency

No hidden global snapshot is assumed. Cross-owner consistency must be explicit. Derived projections declare the observations/lineage used so stale or mixed-source presentation can be diagnosed and, where needed, rejected.

## Fitness Functions

Future implementation must provide focused executable guards for the boundary it changes. The architecture-level fitness set is:

1. **No global semantic enumeration**: generic editor coordination does not need a central enum update to support a new owner/domain target.
2. **No global active-target requirement**: independent activation scopes operate without a mandatory global active document/editor.
3. **Owner-preserving mutation**: foreign changes are proposals/commands validated by the owner; editor intent alone never mutates foreign truth.
4. **Publication race safety**: two out-of-order projection computations cannot publish in completion order; superseded work is rejected.
5. **Atomic publication**: presentation and route state for one accepted generation become visible coherently.
6. **Stale-route rejection**: an interaction route from an older generation fails closed unless an explicit safe remap exists.
7. **Concurrency separation**: projection generation cannot satisfy an owner revision/precondition check.
8. **Selection expiry**: stale semantic addresses do not silently retarget after identity reuse.
9. **History isolation**: structural composition undo cannot become document/domain undo, and unrelated owners do not share a hidden editor history stack.
10. **Save-cut correctness**: a completed save of state N cannot mark N+1 clean.
11. **Uncertain-effect retention**: closing a surface cannot erase an `Unknown` possibly-effectful attempt.
12. **OperationGroup honesty**: mixed per-operation outcomes remain visible; no fake cross-owner atomic success.
13. **Headless validity**: a read-only/headless consumer operates without fake native window, focus, mutation, selection, history, or persistence.
14. **Multi-window independence**: focus/activation/presentation remain window-local even when semantic contexts are explicitly shared.
15. **Structural ownership**: editor coordination has no writable mirror of `ui_composition` structure.
16. **Durability boundary**: runtime IDs/routes/generations are rejected as durable reopen identity.

Repository-level delivery still requires focused tests plus canonical `cargo validate` and exact-head hosted acceptance according to Engineering governance.

## Proving Consumers

Before claiming the normalized model is broadly reusable, implementation evidence should cover at least these distinct shapes inside Runenwerk or real consumers:

- the main editor with mutable owner-backed content;
- one read-only or headless surface/session;
- one multi-view or multi-window shape with explicit sharing and independent activation;
- one cross-authority surface combining observations without assuming a global revision.

Fixtures alone prove invariants but not repository extraction readiness.

## Extraction Gate

ADR 0025 does not authorize a standalone RunenEditor repository.

Any future extraction must follow ADR 0014 and a separate accepted boundary decision. At minimum it must demonstrate:

- a stable, independently useful coordination contract rather than a directory copy of current Runenwerk code;
- no dependency from the candidate framework back into Runenwerk product/app semantics;
- real consumer pressure showing the reusable boundary, not only headless fixtures;
- explicit adapter ownership for Runenwerk-specific integrations;
- independent validation and public downstream conformance;
- a clean cutover plan with no source mirror, forwarding namespace, compatibility authority, branch dependency, or writable duplicate semantic owner;
- exact revision/version consumption and provenance/licensing closure.

Existing `domain/editor/*` placement is not evidence that extraction is ready. Current predecessor-shaped `editor_core` is specifically evidence that semantic ownership must be repaired before any extraction could be credible.

A reusable bridge or editor-coordination framework is extracted only after independent consumption proves stable ownership and a separate ADR accepts the dependency direction.

## Documentation Cutover

This design replaces the two predecessor documents as current editor-coordination authority:

- `editor-workspace-document-mode-panel-architecture.md`;
- `editor-ui-workspace-tool-surface-architecture.md`.

Those documents are retired from the live corpus; their implementation-era and
migration evidence remains in Git and GitHub history. Current docs must link to
this model for generic editor coordination, to ADR 0013 for structural
composition, and to the implemented Tool Suite design for registry/provider
host contracts.

Historical plans/reports retain their original prose and receive path-only reference repair when required by the documentation validator.
