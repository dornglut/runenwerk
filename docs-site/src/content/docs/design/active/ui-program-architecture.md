---
title: UI Program Architecture
description: First proving-domain design for the Runenwerk Domain Workbench Platform north star, defining UiProgram as a durable UI contract without authorizing code or shared platform extraction.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-31
related:
  - ./runenwerk-domain-workbench-north-star.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../deferred/ui-model-multiple-execution-strategies-design.md
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Program Architecture

## Status

Active proving-domain design for the accepted
[Runenwerk Domain Workbench North Star](./runenwerk-domain-workbench-north-star.md).

This document treats UI as the first concrete proof of the Domain Workbench
Platform direction:

```text
Authored UI source
-> Normalized UI model
-> Formed interaction contracts
-> UiProgram
-> UiCompiler
-> UiRuntimeArtifact
-> UiEvaluator
-> UiOutput
   -> editor, game, world-space, and headless hosts
```

This document does not implement code, create crates, rename existing domains
or crates, or authorize shared foundation/meta extraction.

Implementation requires later bounded roadmap work, explicit validation gates,
and migration evidence. Shared platform extraction remains forbidden until
proving-domain gates and second-domain gates are satisfied.

## 1. Problem Statement

Runenwerk UI already has meaningful pieces:

- authored UI definitions in `domain/ui/ui_definition`;
- formed interaction contracts in `FormedInteractionModel`;
- retained UI trees and runtime execution in `ui_tree`, `ui_runtime`, and
  `ui_widgets`;
- renderer-facing output through `ui_render_data`;
- editor/app command execution outside generic UI.

The current shape is good at preserving ownership boundaries, but it does not
yet prove the Domain Workbench Platform north star. Rich controls, authored
surfaces, game HUDs, world-space UI, headless previews, fixtures, diagnostics,
source maps, migrations, and runtime artifacts still do not share one durable
program contract.

The UI proving-domain must solve these issues without collapsing UI into:

- a giant runtime enum;
- a generic graph runtime;
- ECS-owned UI semantics;
- renderer-owned UI/product truth;
- app-owned one-off behavior;
- untyped node packages.

The goal is to make UI a platform-grade domain:

```text
UiProgram is the durable executable contract.
ControlPackage contributes typed UI meaning.
UiCompiler produces optimized runtime artifacts.
UiEvaluator produces deterministic facts.
Hosts perform environment-specific effects.
Renderer consumes derived visual products only.
```

## 2. Non-Goals

This design does not authorize:

- code implementation;
- new workspace crates;
- renaming existing UI crates or domains;
- shared foundation/meta extraction;
- moving current retained UI production paths immediately;
- making ECS the owner of UI semantics;
- making renderer primitives the owner of UI or product truth;
- replacing UI definitions with runtime widgets;
- a universal graph editor;
- a universal node runtime;
- a giant `UiSemanticEvent` enum;
- generic graph interpretation in hot paths by default;
- external UI framework adoption as the core UI model;
- weakening editor, game, or domain-specific UI authoring to fit generic tools.

This design is not a UI-only architecture pretending to be a platform. It is
the first proving-domain design under the Domain Workbench Platform.

## 3. Architecture Target

The target UI architecture is:

```text
AuthoredUiPackage
-> AuthoredUiTemplate
-> schema validation
-> NormalizedUiTemplate
-> FormedInteractionModel
-> UiProgram
-> UiCompiler
-> UiRuntimeArtifact
-> UiEvaluator
-> UiOutput
   -> UiFrame
   -> UiEventPacket
   -> UiInspectionReport
   -> UiDiagnosticReport
```

The primary durable contract is `UiProgram`.

`UiProgram` is not the authoring format, not a retained tree, not an ECS entity
set, and not renderer output. It is the versioned UI domain program that
connects authoring, compilation, evaluation, inspection, migration, fixtures,
diagnostics, and hosts.

The design has five major ownership groups:

```text
Source/formation owner:
  ui_definition

Schema owner:
  UI-owned schemas for controls, bindings, events, accessibility, fixtures, and
  migrations

UI program owner:
  UiProgram model, graph contracts, state model, source maps, versions

Control package owner:
  ControlPackage schema, properties, kernels, fixtures, diagnostics

Execution owner:
  UiCompiler, UiEvaluator, UiRuntimeArtifact

Host owner:
  pure editor, game, world-space, and headless host contracts
```

The improved long-term domain-owned target shape is:

```text
domain/ui/
|-- ui_definition/
|-- ui_schema/
|-- ui_program/
|-- ui_controls/
|-- ui_compiler/
|-- ui_artifacts/
|-- ui_evaluator/
|-- ui_state/
|-- ui_binding/
|-- ui_hosts/
|-- ui_render_data/
|-- ui_theme/
|-- ui_text/
|-- ui_input/
|-- ui_accessibility/
|-- ui_geometry/
`-- ui_testing/
```

This is target architecture only and a long-term ownership map, not immediate
crate creation. Existing crates such as `ui_math`, `ui_tree`, `ui_widgets`, and
`ui_runtime` are not renamed or removed by this design. `ui_geometry` names the
long-term geometry ownership responsibility currently represented by `ui_math`.
Retained UI remains the production/compatibility path until bounded UiProgram
proof slices replace specific surfaces under explicit gates.

Ownership constraints for the long-term shape:

- `ui_schema` is UI-owned and must not become `foundation/meta` by default.
- `ui_hosts` contains pure host contracts only.
- concrete editor integration stays in `domain/editor/*` and
  `apps/runenwerk_editor`;
- concrete game and world integration stays in the owning game/runtime/world
  host layers;
- `ui_controls` owns package definitions and kernels, but package registries
  must be explicit inputs/snapshots, never hidden global mutable state;
- the second platform proof is `MaterialProgram`;
- `RenderPlan` follows after `MaterialProgram`;
- shared extraction remains forbidden until UI and `MaterialProgram` both prove
  the same domain-agnostic primitive and a separate extraction design accepts
  it.

The visual/render boundary is explicit:

```text
VisualGraph
-> visual evaluation
-> generic render primitives in UiFrame
-> engine renderer
```

UI owns visual intent. Renderer owns rendering execution. Renderer does not own
UI meaning or product truth.

## 4. Authoring-To-Runtime Pipeline

The full pipeline is:

```text
Authored UI source
  -> authored templates, packages, bindings, themes, routes, fixtures

Schema validation
  -> package schemas, control properties, event payload schemas

NormalizedUiTemplate
  -> canonical source model with source maps and diagnostics

FormedInteractionModel
  -> popup, focus, scroll, docking, viewport arbitration, route contracts

UiProgram
  -> durable graph program with required packages and capabilities

UiCompiler
  -> optimized runtime artifacts for target host and profile

UiRuntimeArtifact
  -> cached executable UI data for hot paths

UiEvaluator
  -> deterministic output, events, diagnostics, traces, inspection

Host
  -> side effects: command dispatch, render submission, ECS anchoring, IO
```

The pipeline intentionally separates authoring, program, runtime artifact, and
host effect layers. A control package may contribute code-backed kernels, but
the durable cross-layer contract is still `UiProgram` plus runtime artifacts.

## 5. Package Contract

A `ControlPackage` contributes typed UI capabilities to the UI domain.

It may define:

- control schemas;
- property schemas;
- state schemas;
- event payload schemas;
- binding requirements;
- style slots;
- layout kernels;
- interaction kernels;
- visual kernels;
- accessibility metadata;
- inspection metadata;
- editor/authoring metadata;
- fixtures and preview scenarios;
- diagnostics;
- migrations;
- source-map behavior;
- runtime artifact builders.

The package contract is declarative at the program boundary:

```text
ControlPackageDescriptor
  package_id
  version
  control_kinds
  property_schemas
  state_schemas
  event_payload_schemas
  required_capabilities
  kernel_ids
  fixture_ids
  migration_ids
  diagnostic_ids
```

Package registry use must be explicit. `UiCompiler` and `UiEvaluator` consume a
declared package registry snapshot or artifact manifest. They must not discover
packages through hidden global mutable state.

Control packages are how UI grows without central enum bottlenecks. A
wheel-plus-triangle color picker package, for example, owns rich selection math
as UI-domain behavior, while exposing events through schema-based payloads and
routes. RGB cube projection is deferred.

Control packages must not:

- mutate editor or game state directly;
- perform app IO;
- own renderer backend resources;
- require ECS as their semantic model;
- bypass `FormedInteractionModel`;
- emit untyped event payloads.

### UiSchemaValue

`UiSchemaValue` is the UI-owned structured value model for schema-bound data.
It is not extracted to `foundation/meta` by this design.

Supported primitive values:

- null;
- boolean;
- signed integer;
- unsigned integer;
- finite floating-point number;
- string;
- stable ID reference;
- binary blob reference by stable asset/blob ID, not inline arbitrary bytes.

Composite values:

- object values use string field names and `UiSchemaValue` field values;
- list values use ordered `UiSchemaValue` elements;
- optional values are represented by schema-declared nullable fields or
  explicit absence, not by ad hoc sentinel values.

Schema behavior:

- every schema-bound value carries or is evaluated against a stable schema ID
  and schema version;
- breaking schema changes require a new schema version;
- schema IDs are UI-owned during this proving-domain phase;
- schemas define required fields, optional fields, accepted primitive kinds,
  list element kinds, object field policies, default behavior, and validation
  diagnostics.

Validation rules:

- values must match the declared schema before they enter `UiProgram`,
  `UiRuntimeArtifactManifest`, fixtures, event packets, binding snapshots, or
  migration output;
- unknown fields are rejected by default;
- schemas may explicitly allow preserved unknown fields for forward-compatible
  fixture/debug data, but those fields must remain inspectable and must not
  affect runtime behavior unless a later schema version accepts them;
- invalid values produce diagnostics with stable diagnostic IDs, schema IDs,
  schema versions, and source-map attachments where available.

Phase 1 serialization:

- phase 1 uses deterministic fixture/debug serialization for `UiSchemaValue`;
- serialization must have stable object field ordering, stable number
  formatting, explicit schema ID/version context, and reproducible output;
- this serialization is for fixtures, diagnostics, debug reports, and
  migration evidence, not a commitment to final graph serialization.

Migration behavior:

- schema migrations transform values from one schema version to another;
- migrations must report source schema ID/version, target schema ID/version,
  changed fields, removed fields, defaulted fields, unknown-field handling, and
  diagnostics;
- migrations must not reinterpret package-owned values without package-owned
  migration rules.

`UiSchemaValue` is used by:

- event payloads;
- binding snapshots and write payloads;
- fixtures;
- diagnostics;
- migration reports;
- package property and state schemas;
- route payload validation.

### Stable ID Policy

Stable IDs are namespaced UI-domain identifiers. They are never silently
repurposed, and collisions fail validation.

The policy applies to:

- control kind IDs;
- package IDs;
- schema IDs;
- event payload schema IDs;
- kernel IDs;
- capability IDs;
- route IDs;
- artifact IDs;
- fixture IDs;
- diagnostic IDs.

Rules:

- every ID belongs to an explicit namespace owned by a package, UI subsystem,
  host contract, or artifact family;
- ID names and versions are separate concerns: the ID identifies the contract,
  while the version identifies compatible contract shape;
- breaking changes require a new version;
- non-breaking additions must define compatibility and unknown-field behavior;
- IDs may be deprecated but must continue to validate with migration
  diagnostics until the owning migration policy removes support;
- deprecated IDs must name their replacement when one exists;
- collisions across package registry snapshots, schema registries, route maps,
  fixtures, diagnostics, artifacts, kernels, or capabilities fail validation;
- artifacts record the exact stable IDs and versions they were compiled from.

### Event / Route / Command Boundary

UI emits semantic event packets. Hosts decide what effects those events cause.

Definitions:

- `UiEventPacket` is semantic UI output. It contains a route, source control,
  phase, payload schema ID, payload value, source map attachment, and optional
  diagnostic context.
- `HostCommand` is an environment-specific effect request. Editor, game,
  world-space, and headless hosts may define different host commands.
- `DomainCommand` is a domain-owned mutation request such as an editor command,
  gameplay command, material edit command, or world command.

Route contract IDs:

- `RouteId` is a stable namespaced ID for a semantic UI route;
- `RouteSchemaVersion` versions the route payload and route contract shape;
- `RouteCapability` declares the capability required to emit or consume the
  route;
- `HostRouteMapVersion` versions the host-owned route map that translates
  `UiEventPacket` output into `HostCommand` and optional `DomainCommand`
  requests.

Routes are stable IDs, not free-form magic strings. Host route maps are
versioned and inspectable. A route can be bound to a host command only through
host-owned route-map policy, binding policy, and capability checks.

Route migration rules:

- route renames require a migration from old `RouteId` to new `RouteId`;
- route payload changes require a new `RouteSchemaVersion`;
- removed routes must produce migration diagnostics and compatibility warnings;
- host route maps must declare which route IDs, schema versions, and
  capabilities they accept;
- route migrations must preserve source maps and report changed route IDs,
  changed payload schemas, removed routes, and replacement routes.

Route validation diagnostics must cover:

- unknown route ID;
- unsupported route schema version;
- missing route capability;
- invalid route payload;
- missing host route-map entry;
- deprecated route use;
- route migration failure.

Correct flow:

```text
UiEventPacket
-> host route map
-> HostCommand
-> optional DomainCommand
-> domain/app-owned ratification and execution
```

Incorrect flow:

```text
ControlPackage
-> app mutation

UiEventPacket route string
-> hidden app behavior

Renderer primitive
-> product truth mutation
```

This boundary is mandatory for editor UI, game UI, world-space UI, and headless
fixtures.

## 6. Program Graph Model

`UiProgram` contains typed graphs. These graphs are related, but they are not
one universal graph.

Required graph families:

```text
ControlGraph
LayoutGraph
StateGraph
StyleGraph
InteractionGraph
BindingGraph
VisualGraph
AccessibilityGraph
InspectionGraph
```

### ControlGraph

`ControlGraph` owns UI control structure:

- control identity;
- parent/child structure;
- package/control kind;
- source mapping;
- authored path mapping;
- local retained state requirements;
- package capability requirements.

It is typed UI structure, not generic node soup.

### LayoutGraph

`LayoutGraph` owns layout constraints and measurement dependencies:

- constraints;
- intrinsic sizing;
- alignment;
- flow and split behavior;
- popup placement inputs;
- host viewport/surface bounds;
- layout invalidation dependencies.

Layout may call package kernels for specialized controls, but the graph must
record those calls through stable kernel/capability IDs.

### StateGraph

`StateGraph` owns UI state relationships:

- transient state;
- preview state;
- committed state;
- focus state;
- hover state;
- pressed/captured state;
- drag state;
- animation state;
- host-fed state;
- package-owned state;
- state invalidation dependencies;
- state persistence eligibility.

State ownership is split intentionally:

- the target `ui_program/graphs/state.rs` module responsibility owns
  structural state requirements, state declarations, dependencies, source maps,
  package state schemas, and which state participates in evaluation. This is an
  eventual module path, not an immediate file-creation instruction.
- `ui_state` owns UI-domain state contracts, lifecycle classes, retention
  policy, preview/commit rules, and host-fed state vocabulary.
- `ui_artifacts` contains optimized state tables, default values, dependency
  indexes, and artifact-local state layout.
- `ui_evaluator` owns per-instance state mutation during evaluation, including
  focus, hover, capture, drag, animation advancement, preview state, committed
  UI state updates, and dirty propagation.

Host state may feed UI through explicit binding contracts. It must not become
hidden UI state.

### StyleGraph

`StyleGraph` owns UI style relationships:

- theme token references;
- state variants;
- style inheritance;
- typography;
- radius/spacing;
- visual state inputs;
- package style slots.

Style remains UI-domain data. Renderer receives resolved primitive data only.

### InteractionGraph

`InteractionGraph` owns UI interaction contracts:

- focus scopes;
- pointer capture;
- keyboard ownership;
- scroll ownership;
- popup and menu stack behavior;
- route slots;
- event packet emission;
- viewport input arbitration;
- gesture kernels.

`FormedInteractionModel` feeds this graph. Alternate execution targets may not
bypass it.

### BindingGraph

`BindingGraph` owns host data contracts and read/write dataflow:

- value bindings;
- collection bindings;
- selection bindings;
- route binding declarations;
- host-provided view-model inputs;
- read models;
- write models;
- value snapshots;
- collection diffs;
- dirty dependencies;
- dirty propagation;
- authorization and capability policy;
- binding failure diagnostics;
- source-map links for data-driven diagnostics;
- host data contract versions.

Bindings are explicit. UI does not reach into private editor/app/provider state.
Hosts provide snapshots and consume route/event outputs through declared
contracts. Binding failure must produce diagnostics rather than silent fallback.

### VisualGraph

`VisualGraph` owns UI visual intent:

- resolved drawing order;
- shape and text visual operators;
- gradients;
- image slots;
- clips;
- overlays;
- control visual kernels;
- visual invalidation dependencies.

It is not `UiFrame`. It is UI-owned visual program data that evaluates into
renderer-facing primitives.

### Text / Render Boundary

Text is part of the visual boundary and must not be collapsed into renderer
truth.

UI owns:

- font and style intent;
- text value bindings;
- text layout requests;
- text overflow/wrap policy;
- text semantic role and accessibility labels;
- text source maps;
- scale/DPI intent requirements;
- fallback font policy requirements.

The text backend owns:

- shaping;
- fallback font resolution and selected fallback fonts according to UI policy;
- glyph identity keys;
- atlas preparation keys;
- glyph metrics;
- text layout metrics;
- glyph cache and atlas preparation;
- invalidation policy for font, DPI, scale, and text-policy changes;
- diagnostics for missing glyphs, unsupported scripts, and fallback failure.

The renderer owns:

- drawing resolved glyph/image/mesh primitives;
- GPU glyph atlas residency when the renderer backend requires it;
- glyph upload handles;
- atlas eviction implementation;
- backend resource residency;
- backend resource lifetime;
- batching and render execution.

Renderer handles are not UI truth. The renderer does not own text meaning,
control state, binding state, product truth, or authored source identity.

### AccessibilityGraph

`AccessibilityGraph` owns accessibility and input-discoverability contracts:

- accessible names;
- roles;
- states;
- focus reachability;
- value descriptions;
- keyboard alternatives;
- screen-reader/reporting metadata where applicable.

### InspectionGraph

`InspectionGraph` owns debug and workbench inspection relationships:

- source-map navigation;
- package provenance;
- graph node provenance;
- kernel provenance;
- diagnostic attachment;
- fixture coverage;
- trace labels;
- runtime artifact links.

## 7. Evaluator / Compiler Model

### Compiler Vs Evaluator Timing

`UiCompiler` runs when any input that can change graph topology, artifact
layout, kernel selection, validation, or target profile changes:

- `UiProgram`;
- package definitions or package registry snapshot;
- control, event, binding, state, accessibility, fixture, or migration schemas;
- themes and style sources;
- target host profile;
- binding shape and host data contracts;
- kernel/capability registry snapshot;
- source inputs and source-map inputs;
- migration inputs;
- text, scale, or DPI policy inputs.

`UiEvaluator` runs per frame, tick, input pass, fixture pass, or headless proof
pass against a `UiRuntimeArtifact`. Its inputs are runtime state, host input
packets, binding data, surface facts, and optional fixture controls.

The boundary is intentionally asymmetric:

```text
Compiler:
  full UiProgram graphs, schemas, packages, source maps, host profile

Evaluator:
  UiRuntimeArtifact, UiState instance, host packets, binding snapshots
```

Hot paths use runtime artifacts. They must not interpret generic graphs by
default.

### UiCompiler

`UiCompiler` transforms `UiProgram` into one or more `UiRuntimeArtifact`
outputs for a target profile.

Inputs:

- `UiProgram`;
- package registry snapshot;
- kernel/capability registry snapshot;
- target host profile;
- theme/style source snapshots;
- binding shape and host data contracts;
- source input versions;
- feature/capability flags;
- validation policy.

Outputs:

- layout artifact;
- interaction artifact;
- binding artifact;
- state artifact;
- visual artifact;
- accessibility artifact;
- inspection artifact;
- diagnostics;
- source map;
- `UiRuntimeArtifactManifest`;
- `UiRuntimeArtifactTables`;
- reproducibility metadata.

`UiCompiler` must fail clearly when required packages, kernels, schemas, or
capabilities are missing.

### UiEvaluator

`UiEvaluator` deterministically evaluates a `UiRuntimeArtifact`.

Inputs:

- runtime artifact;
- host input packet;
- current runtime `UiState` instance;
- host-provided binding values;
- value snapshots and collection diffs;
- pointer/keyboard/text events;
- target bounds, scale, and DPI facts;
- optional fixture mode.

Outputs:

- `UiFrame`;
- `UiEventPacket` list;
- `UiInspectionReport`;
- `UiDiagnosticReport`;
- state mutation report;
- dirty/invalidation report;
- trace/proof packet when requested.

Hot paths must evaluate optimized runtime artifacts. They must not interpret
generic authoring graphs by default.

Generic graph interpretation is allowed only for:

- debugging;
- fixture proof;
- migration tools;
- compiler validation;
- intentionally low-frequency authoring work.

## 8. Host Model

Hosts perform side effects. Evaluators produce facts.

Required host classes:

```text
EditorHost
GameHost
WorldSpaceHost
HeadlessHost
```

### Editor Host

The editor host owns:

- command dispatch;
- provider state integration;
- workbench surfaces;
- active editor definition/application policy;
- project IO;
- shell/app routing;
- render submission.

The editor host consumes `UiEventPacket` and maps routes to editor-owned
commands. It does not make generic UI controls execute editor commands
directly.

### Game Host

The game host owns:

- HUD view-model inputs;
- gameplay route mapping;
- screen-space UI submission;
- game settings and player-facing menus;
- runtime-safe data feeds.

Game UI uses the same `UiProgram` model as editor UI where possible, with
different packages, bindings, routes, fixtures, and capability policies.

### World-Space Host

The world-space host owns:

- anchors;
- projection;
- visibility;
- lifetime;
- interest/culling;
- optional ECS component attachment.

ECS may host world-space UI instances by identity, anchor, lifetime, and data
feeds. ECS does not own UI semantics. The UI program evaluator remains the UI
semantic engine.

### Headless Host

The headless host owns:

- fixture evaluation;
- deterministic tests;
- migration validation;
- accessibility checks;
- diagnostics snapshots;
- source-map proof;
- artifact reproducibility checks.

The headless host is mandatory for platform-grade UI because it proves UI
programs without a renderer or editor process.

### Host Proof Scenarios

The first UI proof must demonstrate host boundaries through concrete scenarios:

- editor inspector field: reads a domain value through a declared binding
  snapshot, previews edits through UI state, emits a route-based
  `UiEventPacket`, and lets the editor host map that packet to a
  domain-owned command;
- game HUD value: reads runtime-safe host-fed values, evaluates visual output,
  and emits no editor-specific command or product mutation;
- world-space entity prompt: attaches UI instance identity, anchor, projection,
  visibility, lifetime, and data feeds through the world-space host while ECS
  remains outside UI semantics;
- headless fixture evaluation: evaluates the same program/artifact without a
  renderer or app process and produces deterministic frame, event, diagnostic,
  state, and inspection reports.

## 9. Runtime Artifact Model

`UiRuntimeArtifact` is the compiled runtime product of `UiProgram`.

It has two conceptual parts:

```text
UiRuntimeArtifact
|-- UiRuntimeArtifactManifest
`-- UiRuntimeArtifactTables
```

`UiRuntimeArtifactManifest` is the durable inspectable contract. It contains:

- artifact ID and artifact version;
- target host profile and host profile version;
- source `UiProgram` version;
- package IDs and package versions;
- control kind IDs;
- schema IDs and schema versions;
- event payload schema IDs and versions;
- kernel IDs and versions;
- capability IDs and versions;
- route IDs and route schema versions;
- fixture IDs;
- diagnostic IDs;
- cache keys;
- source-map indexes;
- diagnostics;
- migration metadata;
- reproducibility metadata.

`UiRuntimeArtifactTables` are optimized in-memory runtime tables. They may
contain:

- flattened control tables;
- layout plans;
- style resolution tables;
- state layout tables;
- state lifecycle policy tables;
- default state value tables;
- interaction dispatch tables;
- binding dependency tables;
- binding snapshot layout tables;
- collection diff plans;
- visual operator batches;
- text layout request tables;
- accessibility records;
- inspection records.

Runtime artifacts are target-profile aware. One `UiProgram` may compile to
different artifacts for editor, game, world-space, and headless hosts.

State in artifacts is structural, not instance truth. Artifacts may contain
state layout, defaults, retention classes, dependency indexes, lifecycle
policies, and package state schema references. Per-instance state values live in
the runtime `UiState` instance owned by the evaluator/host boundary.

Runtime artifacts must be:

- hashable;
- cacheable;
- inspectable;
- reproducible where inputs are deterministic;
- invalidatable by package/schema/source changes;
- versioned.

Runtime artifacts must not own source truth. They are derived products.

The manifest is the stable artifact contract used by inspection, diagnostics,
fixtures, migration evidence, cache validation, and host compatibility checks.
The table layout is an implementation/runtime optimization and is not the
durable cross-version contract.

Runtime artifacts must also be invalidated by host profile, theme/style source,
binding shape, schema, kernel/capability, text policy, DPI policy, or migration
input changes when those changes affect artifact layout or behavior.

## 10. Inspection / Diagnostics / Fixtures

Platform-grade UI must explain itself.

### Inspection

Inspection reports should answer:

```text
Which authored source created this control?
Which package owns this control?
Which graph nodes produced this layout?
Which binding changed this frame?
Which kernel emitted this visual output?
Which route emitted this event packet?
Which host consumed the event?
Which runtime artifact table was used?
Which fixture proves this behavior?
```

### Diagnostics

Diagnostics must cover:

- schema validation;
- missing package;
- missing kernel;
- missing capability;
- invalid route;
- invalid binding;
- invalid interaction contract;
- inaccessible control;
- unsupported target host;
- unsupported world-space behavior;
- runtime artifact mismatch;
- stale migration;
- fixture failure.

Diagnostics must use stable IDs and source-map attachments.

### Fixtures

Every serious `ControlPackage` must ship fixtures.

Fixture categories:

- default visual fixture;
- interactive state fixture;
- accessibility fixture;
- event payload fixture;
- migration fixture;
- host compatibility fixture;
- error/diagnostic fixture;
- performance profile fixture where relevant.

`ui_testing` owns fixture execution contracts, expected-output comparison,
headless proof helpers, visual proof capture metadata, and regression fixture
organization. It does not own package semantics; package-owned fixtures remain
part of the package contract.

The first UI Program proof must include fixtures for editor host, game host,
world-space host, and headless host behavior.

## 11. Migration / Versioning

Versioning applies at several layers:

```text
UiProgramVersion
ControlPackageVersion
ControlSchemaVersion
EventPayloadSchemaVersion
KernelVersion
UiRuntimeArtifactVersion
HostProfileVersion
MigrationVersion
```

Migrations must be explicit and inspectable.

Migration reports should include:

- source version;
- target version;
- package migrations applied;
- schema migrations applied;
- route changes;
- event payload changes;
- graph changes;
- removed capabilities;
- compatibility warnings;
- source-map preservation result.

Control packages own package-specific migrations. UI program migration
orchestrates package migrations but must not reinterpret package meaning
without package-owned rules.

## 12. Acceptance Gates

This proving-domain design separates design acceptance from proof execution.
Implementation work may not begin from this document alone. A later bounded
implementation contract must name the exact surface, write scopes, validation,
and gate evidence.

### Design Gate

The UI Design Gate requires the design to define:

- program boundary;
- graph ownership;
- package contract;
- explicit package registry boundary;
- schema/event model;
- `UiSchemaValue` contract;
- route/event/command boundary;
- route contract IDs and route migration rules;
- kernel/capability model;
- stable kernel/capability IDs;
- stable ID policy;
- compiler/evaluator timing;
- host boundaries;
- concrete integration boundaries outside `domain/ui`;
- visual/render boundary;
- text/render boundary;
- state model;
- binding read/write model;
- authorization/capability policy;
- fixtures;
- diagnostics;
- source maps;
- migration story;
- runtime artifact strategy;
- proof surface plan.

The design gate does not pass if it authorizes shared `foundation/meta`
extraction, creates crates, renames existing crates, or hides app behavior
behind route strings.

### Proof Gate

The first UI proof must include:

- `Label` or another text display control for the text/render boundary;
- `Button` for the route/event migration bridge;
- one binding-heavy UI surface, preferably `InspectorField`, `ListView`,
  `TreeView`, or `TableView`;
- the rich custom control proof is `ColorPicker`;
- one world-space anchored prompt;
- one headless fixture evaluation;
- one structural `UiFrame` visual output proof.

The proof must demonstrate:

- `ControlGraph`;
- `LayoutGraph`;
- `StateGraph`;
- `StyleGraph`;
- `InteractionGraph`;
- `BindingGraph`;
- `VisualGraph`;
- `AccessibilityGraph`;
- `InspectionGraph`;
- source maps;
- schema-based event payloads;
- route-based event packets;
- event/route/command separation;
- explicit binding snapshots and collection diffs;
- binding failure diagnostics;
- stable package, kernel, and capability IDs;
- package-owned fixtures and migrations.

The `ColorPicker` `ControlPackage` must demonstrate:

- property schema;
- state schema;
- event payload schema;
- stable kernel/capability IDs;
- layout kernel;
- interaction kernel;
- visual kernel;
- accessibility metadata;
- fixtures;
- diagnostics;
- migration story.

The first `ColorPicker` proof uses wheel-plus-triangle. It requires rich input,
rich visuals, text/visual/render boundary decisions, semantic event payloads,
accessibility, and editor/game use. RGB cube projection is deferred.

The proof must fail the gate if it relies on:

- a giant `UiSemanticEvent` enum;
- ECS-owned UI semantics;
- renderer-owned UI/product truth;
- generic node soup;
- hidden global package registries;
- hidden app behavior behind route strings;
- broad shared `foundation/meta` extraction.

### Runtime Gate

The runtime path must prove:

- `UiCompiler` runs when program, package, schema, theme, host profile, binding
  shape, source, migration, kernel/capability, text, or DPI policy inputs
  change;
- `UiEvaluator` runs per frame, tick, input pass, fixture pass, or headless
  proof pass against `UiRuntimeArtifact`;
- hot paths use `UiRuntimeArtifact`;
- generic graph interpretation is not the default runtime path;
- artifact invalidation is explicit;
- state layout lives in artifacts while instance state lives in `UiState`;
- binding snapshots and collection diffs are hot-path inputs;
- deterministic headless evaluation works;
- inspection, diagnostics, and state mutation reports are available.

### Tooling Gate

The tooling path must prove:

- fixture execution for editor, game, world-space, and headless hosts;
- source-map navigation from runtime reports back to authored source;
- diagnostics for schema, package, kernel, capability, route, binding,
  migration, accessibility, and host incompatibility failures;
- migration reports with package and schema migration evidence;
- inspection reports for graph provenance, kernel provenance, binding changes,
  route emission, host consumption, and artifact table use;
- visual/render output proof without letting the renderer own UI truth.

### Second-Domain Extraction Gate

Shared `foundation/meta` or platform primitives may be proposed only after the
UI proof and `MaterialProgram` proof both show the same domain-agnostic
primitive. The second platform proof is `MaterialProgram`; `RenderPlan`
follows after `MaterialProgram`. Shared extraction remains forbidden until UI
and `MaterialProgram` both prove the same primitive and a separate extraction
design accepts it. Extraction is still not automatic after that evidence. The
separate extraction design must name the repeated contract, prove it is not
UI-specific or material-specific, and define migration, ownership, validation,
and compatibility rules.

The prior gate names map into the primary UI proof gates:

```text
Program Gate  -> Proof Gate
Package Gate  -> Proof Gate
Host Gate     -> Proof Gate and Tooling Gate
Boundary Gate -> Design Gate, Proof Gate, and Runtime Gate
```

## PM-UI-PROGRAM-002 Stage 1 Contract

This section is the bounded Stage 1 contract produced for `PM-UI-PROGRAM-002`.
It tightens the design-level contracts only. It does not authorize code,
crates, placeholder folders, Stage 6 proof work, MaterialProgram work, or
shared `foundation/meta` extraction.

### Graph Ownership Contract

`UiProgram` owns the durable UI program contract. The graph list is closed for
Stage 1 and contains `ControlGraph`, `LayoutGraph`, `StateGraph`,
`StyleGraph`, `InteractionGraph`, `BindingGraph`, `VisualGraph`,
`AccessibilityGraph`, and `InspectionGraph`.

Each graph owns one class of UI truth and may reference other graphs only by
stable IDs, source-map spans, and explicit dependency edges. None of these
graphs are generic node soup:

- `ControlGraph` owns control identity, package/control kind, hierarchy, and
  capability requirements.
- `LayoutGraph` owns measurement, constraints, placement, and layout kernel
  dependencies.
- `StateGraph` owns structural state declarations and dependencies.
- `StyleGraph` owns UI-domain style intent and state variants.
- `InteractionGraph` owns focus, capture, gestures, route slots, and event
  packet emission.
- `BindingGraph` owns read/write host data contracts and binding dependency
  flow.
- `VisualGraph` owns UI visual intent before renderer-facing output exists.
- `AccessibilityGraph` owns semantic roles, names, focus order, and assistive
  metadata.
- `InspectionGraph` owns provenance, source-map links, diagnostics, and
  runtime inspection references.

### UiSchemaValue Contract

`UiSchemaValue` is UI-owned until the Second-Domain Extraction Gate. It supports
null, booleans, signed integers, unsigned integers, finite floats, strings,
stable ID references, route references, opaque host references, object values,
list values, and schema-declared optional values.

Opaque host references are handles to host-provided data that UI may compare,
route, inspect, and diagnose, but never dereference into hidden app behavior.
Route references identify stable `RouteId` contracts and never replace
`UiEventPacket`.

Every schema-bound value is validated against a schema ID and schema version.
Unknown fields are rejected unless the schema explicitly marks them as
preserved debug/fixture data. Breaking schema changes require a new version and
a migration report. Validation failures attach schema ID, schema version,
source-map span, diagnostic ID, and the graph/artifact location that observed
the failure.

### Stable ID Policy

Stable IDs are namespaced and versioned. Stage 1 applies the policy to control
kind IDs, package IDs, schema IDs, event payload schema IDs, kernel IDs,
capability IDs, route IDs, artifact IDs, fixture IDs, and diagnostic IDs.

IDs are never silently repurposed. Collisions fail validation. Deprecation
requires a replacement or explicit no-replacement reason plus migration
diagnostics. Breaking shape changes require a new version. Artifact manifests
record the exact IDs and versions used for compile, evaluation, diagnostics,
source maps, and fixtures.

### StateGraph And UiStateModel Contract

`StateGraph` owns structural state requirements: declarations, state class,
dependencies, package state schemas, persistence eligibility, source-map spans,
and artifact layout inputs.

`UiStateModel` owns runtime state contracts: transient state, preview state,
committed state, focus state, hover state, pressed/captured state, drag state,
animation state, host-fed state, and package-owned state. Compiled state tables
belong to `UiRuntimeArtifactTables`. Evaluation-time transitions belong to
`UiEvaluator`. Host-fed state enters only through explicit binding or host
contracts and cannot become hidden UI semantics.

### BindingGraph Contract

`BindingGraph` owns read model declarations, write model declarations, value
snapshots, collection diffs, dirty propagation, host data contracts,
authorization policy, capability checks, binding diagnostics, and source-map
attachments.

Reads consume host-provided snapshots. Writes produce route/event payloads or
domain-owned mutation requests; UI never mutates host/domain state directly.
Collection diffs identify insert, remove, move, replace, selection, and
expansion changes with stable item IDs when available. Binding failure is
diagnostic output, not silent fallback.

### VisualGraph And UiFrame Boundary

`VisualGraph` is UI-owned visual intent. It describes draw order, shapes, text
operators, image slots, clips, overlays, control visual kernels, invalidation
dependencies, and source maps.

`UiFrame` is derived renderer-facing structural output. It may contain resolved
visual commands, text layout requests/results, clipping, z-order, accessibility
links, source-map references, diagnostics, and render-handoff metadata. It is
not product truth. The renderer owns backend execution and resource residency,
not UI meaning.

### Text / Render Boundary

UI owns font/style intent, text value bindings, text layout requests, wrapping,
overflow policy, semantic role, accessibility labels, source maps, DPI/scale
intent, and fallback policy requirements.

The text backend owns shaping, fallback font resolution, glyph identity keys,
atlas preparation keys, glyph metrics, text layout metrics, cache preparation,
and invalidation policy for font, DPI, scale, locale, and text-policy changes.

The renderer owns GPU atlas residency, upload handles, eviction implementation,
backend resource lifetime, batching, and draw execution. Renderer handles are
not UI truth and must not be stored as semantic UI data.

### Event Packet And Payload Contract

`UiEventPacket` is semantic UI output. It carries `RouteId`,
`RouteSchemaVersion`, source control ID, interaction phase, payload schema ID,
`UiSchemaValue` payload, source-map attachment, capability context, and
diagnostic context.

There is no giant `UiSemanticEvent` enum. Event payloads are schema-based and
route-based. Unknown routes, unsupported route schema versions, missing
capabilities, invalid payloads, and missing host route-map entries produce
diagnostics and do not execute hidden behavior.

### Route / HostCommand / DomainCommand Boundary

`RouteId` is a stable namespaced ID. `RouteSchemaVersion` versions the route
payload and route contract shape. `RouteCapability` names the capability needed
to emit or consume the route. `HostRouteMapVersion` versions the host-owned map
from `UiEventPacket` to `HostCommand` and optional `DomainCommand`.

`HostCommand` is environment-specific. `DomainCommand` is domain-owned mutation
authority. Hosts map UI events into commands only through inspectable route-map
policy and capability checks. Routes must not become hidden app behavior or
free-form strings.

### Source-Map Attachment Points

Source-map attachments are required on control nodes, graph edges, schema
values, package properties, state declarations, bindings, routes, event
payloads, visual operators, text layout requests, artifact manifest entries,
artifact tables, diagnostics, fixtures, migrations, and host route-map entries.

Every runtime diagnostic and fixture assertion must be able to point back to
the authored source when source context exists. Generated or synthesized nodes
must record generated provenance and the source span or rule that produced
them.

### Diagnostics Attachment Points

Diagnostics attach to schema validation, ID collision, deprecated ID use,
package registration, missing kernels, capability denial, invalid bindings,
collection diff mismatch, route validation, host route-map mismatch, text
fallback failure, source-map loss, artifact invalidation, migration failure,
fixture mismatch, and renderer-boundary misuse.

Diagnostic IDs are stable IDs. Diagnostics must include severity, source-map
span when available, owning graph/artifact, schema or route context when
available, suggested migration or fix when known, and whether evaluation may
continue.

### Open Questions And Blocked Decisions

- Final graph serialization format remains deferred until Stage 6 evidence
  proves the minimum artifact and fixture shape.
- Exact future module paths remain target responsibilities, not immediate file
  creation instructions.
- Shared `foundation/meta` extraction remains blocked until UI and
  `MaterialProgram` prove the same primitive and a separate extraction design
  accepts it.
- Stage 6 proof work remains blocked until Stages 1 through 5 close with
  evidence.

## 13. Staged Implementation Plan

This plan describes sequencing only. It does not authorize code.

### Stage 0 - ADR / Design Promotion

- Promote this active design through the repository's governance path.
- Record that UI Program Architecture is the first proving-domain design under
  the Domain Workbench Platform north star.
- Define exact write scopes and validation gates before implementation.

### Stage 1 - UI Program Contract Design

- Specify `UiProgram` graph contracts.
- Specify `UiSchemaValue` and the stable ID policy.
- Specify `StateGraph` / `UiStateModel` contracts.
- Specify `BindingGraph` read/write, snapshot, dirty propagation, collection
  diff, and authorization contracts.
- Specify visual/render and text/render boundaries.
- Specify event packet and payload schema contracts.
- Specify route/event/command boundary contracts.
- Specify source-map and diagnostic attachment contracts.
- Keep all contracts inside UI-owned design scope.

### Stage 2 - Control Package Proof Design

- Define the first serious `ControlPackage`: wheel-plus-triangle
  `ColorPicker`.
- Include property schema, state schema, event payload schema, visual operators,
  accessibility, fixtures, diagnostics, and migration behavior.
- Define explicit package registry input/snapshot behavior.
- Prove route-based event packets instead of central event enum variants.
- Choose the binding-heavy proof surface: `InspectorField`, `ListView`,
  `TreeView`, or `TableView`.

### Stage 3 - Compiler / Runtime Artifact Design

- Define `UiCompiler` inputs and outputs.
- Define `UiRuntimeArtifact` tables and manifests.
- Define cache keys and invalidation.
- Define host target profiles.
- Define compiler/evaluator timing and the artifact boundary for hot paths.
- Define state layout tables, binding snapshot tables, collection diff plans,
  and text layout request tables.

### Stage 4 - Evaluator / Host Design

- Define `UiEvaluator`.
- Define editor, game, world-space, and headless host contracts.
- Define how hosts consume event packets and submit frames.
- Define `UiEventPacket`, `HostCommand`, and `DomainCommand` mapping rules.
- Define editor inspector, game HUD, world-space entity prompt, and headless
  fixture proof scenarios.

### Stage 5 - Migration From Retained UI

- Map current retained UI concepts to the new program architecture.
- Preserve current production behavior until replacement paths are proven.
- Use Strangler migration: authored definitions and retained runtime remain
  valid until UiProgram artifacts are ready for a bounded surface.

### Stage 6 - Ordered Runtime-Proven Slices

The full Proof Gate remains the final target, but implementation evidence
should be gathered through ordered sub-proofs:

- 6A: `Label` plus structural `UiFrame` text proof.
- 6B: `Button` plus route/event/host-command proof.
- 6C: `InspectorField` plus binding/state proof.
- 6D: `ColorPicker` plus rich `ControlPackage` proof.
- 6E: world-space prompt plus ECS/host boundary proof.
- 6F: headless fixture plus diagnostics/source-map proof.

Each sub-proof must produce source-map, fixture, diagnostic, route, artifact,
and host-boundary evidence appropriate to its scope. The full Proof Gate is
not satisfied until all sub-proofs are complete. Do not extract shared platform
primitives.

### Stage 7 - Second-Domain Preparation

- Record what UI proved and what remains UI-specific.
- Prepare `MaterialProgram` as the second platform proof before any
  foundation/meta extraction.
- Prepare `RenderPlan` only after `MaterialProgram`.

## 14. Resolved Decisions

The previous open questions are answered as current design decisions:

- `UiProgram` is a future crate boundary, but it may evolve inside existing UI
  modules during transition.
- `ui_geometry` is a long-term ownership target. During transition, existing
  `ui_math` remains the geometry/math home and is not renamed by this design.
- Phase 1 uses typed Rust data structures plus deterministic fixture/debug
  serialization. Final graph serialization is deferred until proof evidence
  shows the required shape.
- event payloads use UI-owned `UiSchemaValue` before shared platform
  extraction.
- package, kernel, and capability registries are explicit UI-owned registry
  values or registry snapshots, not hidden globals.
- the first color picker proof uses wheel-plus-triangle. RGB cube projection
  is deferred.
- `InspectorField` is the preferred first binding-heavy proof surface because
  it exercises read/write binding, preview state, committed state, route
  mapping, and domain command handoff. `ListView`, `TreeView`, and `TableView`
  remain follow-up stress surfaces.
- structural state requirements belong to the target
  `ui_program/graphs/state.rs` module responsibility, which is an eventual
  module path rather than an immediate file-creation instruction; runtime state
  contracts belong to `ui_state`; compiled state tables belong to
  `ui_artifacts`; evaluation-time state transitions belong to `ui_evaluator`.
- current `ui_runtime` becomes compatibility/strangler infrastructure, not the
  future semantic core.
- authored UI definitions may dual-lower to retained UI and `UiProgram` during
  migration.
- the minimum game-host proof is one HUD value plus one action route.
- the minimum world-space proof is one anchored interaction prompt.
- early visual diff evidence uses structural `UiFrame` assertions before pixel
  diffs.
- the second platform proof is `MaterialProgram`; `RenderPlan` follows after.
- extraction into shared foundation/platform ownership requires the
  Second-Domain Extraction Gate.
