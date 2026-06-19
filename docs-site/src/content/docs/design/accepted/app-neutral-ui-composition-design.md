---
title: App-Neutral UI Composition Clean Cutover Design
description: Accepted architecture for deterministic app-neutral layout definitions, ratified structural state, mounted content references, transactions, persistence bundles, fixtures, and clean replacement of workspace and surface structural authority.
status: accepted
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-19
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_designs:
  - ./adaptive-ui-composition-design.md
  - ./editor-native-multi-window-presentation-design.md
  - ../active/ui-program-architecture-owner-map.md
related_roadmaps:
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# App-Neutral UI Composition Clean Cutover Design

## Status And Authority

This is the accepted core-composition contract for
`PT-UI-COMPOSITION-CUTOVER`. Implementation requires the ordered roadmap and
production checkpoints. It does not by itself claim that current code conforms.

The cutover is deliberately clean:

- `domain/ui/ui_composition` becomes the only app-neutral structural authority;
- editor workspace and `ui_surface` state are temporary branch-local migration
  inputs, not parallel final APIs;
- no aliases, dual persistence authority, or automatic legacy migration may
  survive the cleanup checkpoint;
- final merge is blocked until cleanup and perfectionist closeout pass.

## Ownership

`domain/ui/ui_composition` owns reusable structural mechanism:

- authored layout definitions;
- ratified mutable structure;
- presentation-target, root, region, and mounted-unit identity;
- typed structural transactions and invariant validation;
- structural undo/redo records;
- deterministic serialization contracts;
- state-to-definition promotion primitives;
- neutral content-resolution observations;
- headless conformance fixture contracts.

It does not own app lifecycle, native windows, provider sessions, editor or game
semantics, document mutation, renderer state, or adaptive interaction sessions.

Editor and Draw own their content profiles, providers, extension documents,
capability policy, lifecycle policy, projection, and content actions.

## Four-Part State Model

### `CompositionDefinitionV1`

The saved and authored layout. It contains only deterministic structural data,
profile references, mounted content references, capabilities, and retention
metadata. It contains no live provider state, viewport state, or native handles.

### `CompositionState`

The ratified mutable structural authority formed from a valid definition. It
owns the current revision, transaction journal, and structural undo/redo data.
It is never persisted directly.

### `AdaptiveProjectionState`

A transient derivation owned by `ui_adaptive_composition`. It may collapse,
reflow, preview, or temporarily reposition regions without changing canonical
state.

### `LayoutPromotion`

An explicit operation that derives a candidate definition from ratified state
and coordinates a simultaneous app-extension snapshot. Promotion validates and
persists the complete bundle atomically.

`Save Layout` promotes canonical state and excludes transient adaptive changes.
`Promote Current Arrangement to Layout` is a separate explicit action for a
validated adaptive promotion delta. Promotion requires a name and scope and
never silently overwrites the source layout.

## Structural Vocabulary

Required identities:

- `CompositionDefinitionId`
- `PresentationTargetId`
- `CompositionRootId`
- `RegionId`
- `MountedUnitId`
- `CompositionTransactionId`

Extensible semantic references:

- `HostProfileId`
- `TargetProfileId`
- `RegionProfileId`
- `ContentOwnerId`
- `ContentProfileId`
- `CapabilityId`

The closed region algebra is:

- `Split { axis, fraction, first, second }`
- `Stack { ordered_units, active_unit }`
- `Overlay { base, ordered_overlays }`
- `MountPoint { mounted_unit }`

App-specific meaning is expressed through profile IDs, not central app-kind or
content enums.

## Mounted Content

`MountedContentRef` is an opaque typed tuple of content owner, content profile,
and stable instance reference. It is never an arbitrary payload and never
requires `UiProgram`.

Providers and sessions remain app/domain-owned and are keyed by
`MountedUnitId`. Composition records structural placement, capability
references, and retention metadata only.

App-owned resolvers report one neutral liveness state:

- `Resolved`
- `Missing`
- `Loading`
- `Suspended`
- `Denied`
- `UnsupportedProfile`
- `Crashed`

Liveness is transient. Unresolved content never invalidates the composition
graph. Projection uses this strict fallback order:

1. app-provided unavailable-content projection;
2. neutral diagnostic placeholder;
3. hidden only when the mounted-unit/content profile explicitly permits
   `hide_on_unavailable` and host policy accepts it.

## Invariants

Formation and every committed transaction must prove:

- IDs are unique and references resolve;
- region graphs are acyclic and every region has one structural parent;
- each mounted unit has exactly one structural location;
- every target has exactly one primary root;
- stack active units and explicit ordinals are valid;
- overlay z-order is unambiguous;
- split fractions and structural bounds are valid fixed-point values;
- capability and lifecycle policy accepted the complete transaction;
- rejection leaves state and revision unchanged.

## Transactions And Structural History

`CompositionTransaction` carries a stable ID, expected revision, and ordered
typed commands. Commands cover mount, unmount, activate, move, reorder, split,
resize, merge, root creation/movement/closure, and target attachment/detachment.

The host evaluates lifecycle, capability, and proposal-acceptance ports before
the composition domain validates and commits atomically. There is no `AppHost`
god trait.

Undo/redo is structural only. It never owns document, drawing, browser,
terminal, graph, or game-state history. Stored inverse commands are submitted
as new transactions against the current revision and revalidated. Conflicts
reject explicitly.

Closing a non-primary target requires one atomic rehome transaction. If any
unit lacks a legal primary fallback, target closure is vetoed without mutation.

## Deterministic Serialization

Persistence uses canonical UTF-8 RON through a dedicated canonical writer:

- schema-defined field order;
- LF line endings and one trailing newline;
- no comments or default-dependent omission;
- no unordered-map iteration;
- unknown fields rejected;
- persisted fractions use integer fixed-point types;
- hashes use lowercase `blake3:<hex>`.

Stable ordering is mandatory:

- targets, roots, regions, and mounted units sort by their IDs;
- profile and capability references sort by canonical ID;
- extension links sort by extension profile ID and schema version;
- fixture records sort by fixture ID;
- diagnostics sort by stage rank, severity rank, code, subject type, and
  subject ID;
- journal entries sort by applied revision and transaction ID.

Semantic sequences retain explicit order: stack ordinal, overlay z-order then
region-ID tie break, transaction command ordinal, and named split children.
Duplicate or ambiguous ordinals reject. Display labels never establish
identity, equality, or ordering.

Golden tests must prove byte stability, insertion-order independence,
round-trip byte identity, and stable BLAKE3 hashes.

## Core And Extension Bundles

The core definition and every typed app extension share:

- layout ID;
- definition revision;
- core schema version;
- extension profile and schema version;
- app/profile compatibility declaration;
- core payload hash;
- extension payload hash.

Hashes cover canonical shared metadata without hash fields plus canonical
payload bytes. The core envelope contains a sorted extension-link list and each
extension repeats matching link metadata. Any missing file or metadata/hash
mismatch rejects the whole bundle.

The storage adapter writes a new generation directory, reads it back, validates
it, flushes it, then atomically replaces one active-generation pointer. The
previous valid generation remains last-good recovery.

Whole-definition precedence is built-in, project, then user. Session recovery
is separate. There is no field-level scope merge.

Legacy editor schemas V1-V5 remain untouched and are rejected with an
unsupported-schema diagnostic. Recovery is an explicit reset/create-default or
valid-layout selection; no automatic migration or silent fallback is allowed.

## Diagnostic Ownership

Every rejection emits at least one structured diagnostic with stable code,
severity, stage, typed subject ID, actionable message, and sorted context.

Reserved prefixes:

- `ui_composition.*`
- `ui_adaptive_composition.*`
- `ui_program_hosts.*`
- `editor_composition.*`
- `draw_composition.*`
- `composition_persistence.*`
- `composition_fixture.*`

App and windowing policy diagnostics remain app-owned and may not use reserved
prefixes.

## Fixture Contract

Each executable headless fixture declares fixture ID, host/target profiles,
definition, mounted content references, expected validity, capabilities,
diagnostics, adaptive proposals, and forbidden imports/product behavior.

Browser, terminal, dashboard, mobile, and game fixtures prove structural
conformance only. They implement no product behavior or product providers.

## Multi-Window Boundary

Composition owns `PresentationTargetId`, root relationships, and structural
cross-target movement. App/engine/windowing owns native windows, OS lifecycle
and vetoes, monitors, DPI, restore behavior, native bounds, render surfaces,
and platform failures.

## Clean Cutover Rules

- Before editor static projection, old state may be temporary migration input.
- After editor static projection, legacy workspace state is read-only.
- After editor docking runtime, all structural mutation enters through
  composition transactions.
- Cleanup deletes workspace structural authority, V1-V5 schemas, aliases,
  adapters, and `ui_surface` before final merge.
- Historical reports and superseded designs remain historical evidence and may
  retain old terminology.

## Extraction Gate

The owner remains `domain/ui`. Extraction is allowed only after two real
non-UI bounded contexts prove the same contract and a separate ADR accepts the
new dependency direction. Headless fixtures do not satisfy this gate.

