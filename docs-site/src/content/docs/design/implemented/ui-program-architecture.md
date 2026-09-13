---
title: UI Program Architecture
description: Implemented Runenwerk-local UiProgram architecture for typed UI programs, graph families, compilation, runtime artifacts, evaluation, host boundaries, diagnostics, source maps, fixtures, and retained-UI compatibility.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related:
  - ../../architecture/ui-framework-architecture.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ../deferred/ui-model-multiple-execution-strategies-design.md
  - ./ui-program-architecture-owner-map.md
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Program Architecture

## Status

Implemented Runenwerk-local architecture.

The concrete UiProgram architecture that this design originally specified now exists in current Runenwerk source and tests. The later `PT-UI-PROGRAM-ARCHITECTURE` correction/conformance sequence superseded earlier incomplete truth claims. The current conformance contracts mark the architecture and perfectionist-conformance claims satisfied with no known gaps, while current source, tests, and mechanically checked ownership remain the implementation authority. Historical plans, closeouts, audits, and execution evidence remain delivery/provenance records rather than current behavior authority.

The implemented semantic spine is:

```text
Authored UI source
-> Normalized UI model
-> Formed interaction contracts
-> UiProgram
-> UiCompiler
-> UiRuntimeArtifact
-> UiEvaluator
-> renderer-neutral output / route intent / inspection facts
-> host-owned effects and domain mutation
```

This lifecycle classification is deliberately narrower than a total UI-stack replacement. Current retained/foundation owners including `ui_math`, `ui_layout`, `ui_tree`, `ui_widgets`, and `ui_runtime` still coexist legitimately with the UiProgram architecture where current ownership assigns them responsibility. Their presence is compatibility/foundation truth, not an architecture-gap claim.

This document does **not** claim:

- that standalone RunenUI has replaced Runenwerk's current consumer implementation;
- that retained UI compatibility/foundation paths have been deleted or superseded;
- that every editor, app, game, world-space, or future runtime integration has completed a UiProgram cutover;
- that future reusable framework breadth belongs to Runenwerk rather than the separately owned standalone RunenUI authority;
- that `MaterialProgram`, a shared `foundation/meta`, or another domain-program family is authorized by this implementation.

Those concerns remain separately owned and require their own accepted work.

## Architecture contract

`UiProgram` is the durable UI-domain program boundary. It is not the authoring format, a retained widget tree, an ECS entity set, a renderer frame, or host/application mutation authority.

The implemented path keeps these stages distinct:

```text
AuthoredUiTemplate / normalized UI source
-> ui_program_lowering
-> UiProgram
-> ui_compiler
-> UiRuntimeArtifact
-> ui_evaluator
-> renderer-neutral output / route intent / inspection facts
-> host-owned effects and domain mutation
```

The primary implemented owners are under `domain/ui/`:

- `ui_definition` — authored UI source, normalization, validation, source locations, templates, and retained compatibility formation;
- `ui_schema` — UI-owned schema/value contracts;
- `ui_program` — durable program identity, typed graph families, source maps, diagnostics, routes, events, and versions;
- `ui_controls` — `ControlPackage` registry and package-owned control contracts, schemas, fixtures, diagnostics, migrations, route requirements, target profiles, and stories;
- `ui_program_lowering` — deterministic formation/lowering into `UiProgram`;
- `ui_compiler` — program-to-runtime-artifact validation/lowering and compiler reports;
- `ui_artifacts` — runtime artifact manifests, optimized tables, compiled source maps, cache keys, and artifact diagnostics;
- `ui_evaluator` — deterministic evaluation over artifacts;
- `ui_state` — UI runtime-state contracts;
- `ui_binding` — explicit binding, snapshot, dirty-propagation, authorization, and host-data contracts;
- `ui_hosts` — generic host profile, capability, route-map, and host-boundary contracts;
- `ui_accessibility` — accessibility facts and proof contracts;
- `ui_geometry` — UI geometry facts and geometry contracts;
- `ui_runtime_view` — runtime read model over artifacts and host facts;
- `ui_render_data` and `ui_render_primitives` — renderer-neutral output contracts;
- `ui_testing`, `ui_story`, and headless/static-mount owners — executable proof and inspection support.

The exact dependency/ownership policy is mechanically represented by `domain/ui/ui-crate-ownership.toml` and checked by repository validation.

## Source, program, artifact, and host separation

The durable separation is:

```text
authoring source != semantic program != runtime artifact != host effects
```

Authoring source may carry ergonomic templates, source locations, package references, bindings, and preview metadata. `UiProgram` carries durable UI semantics. Runtime artifacts are derived execution products. Hosts perform environment-specific side effects.

UI code may emit route intent and renderer-neutral output. It does not gain ownership of editor, game, application, ECS, renderer, OS, or persistence semantics merely because those owners consume its output.

## UiSchemaValue contract

`ui_schema` implements the UI-owned structured value model used by UiProgram contracts. Current `UiSchemaValue` variants are:

```text
Null
Bool(bool)
Integer(i64)
UnsignedInteger(u64)
Number(f64)
String(String)
StableIdRef(UiStableIdRef)
RouteRef(UiRouteRef)
OpaqueHostRef(UiOpaqueHostRef)
List(Vec<UiSchemaValue>)
Object(BTreeMap<String, UiSchemaValue>)
```

`Number` rejects non-finite values. Stable, route, and opaque-host references are non-empty namespaced references and accept only ASCII alphanumeric characters plus `.`, `_`, and `-`. Object construction rejects duplicate fields; `BTreeMap` provides deterministic field ordering.

Schema-bound payloads are validated through explicit `UiSchema` / `UiSchemaRef` contracts. `UiSchemaValue` is used by event payloads and other schema-bound program/runtime facts. It remains UI-owned; this architecture does not extract it into shared `foundation/meta`.

`OpaqueHostRef` is an opaque namespaced host reference, not permission for UI to dereference private host state. `RouteRef` is a value-level route reference, not a substitute for `UiEventPacket`.

## Stable ID policy

Durable UI contracts use stable namespaced identifiers where their concrete types require namespacing. Current explicit examples include `RouteId`, `RouteCapability`, `UiStableIdRef`, `UiRouteRef`, and `UiOpaqueHostRef`.

Rules:

- semantic identity is separate from visible labels and runtime object identity;
- route schema versions are non-zero and explicit;
- duplicate package IDs and duplicate control-kind IDs fail registry registration;
- artifacts record the concrete package, control-kind, schema, route, kernel, capability, program, target-profile, source-map, and cache-key facts used to build them;
- runtime widget IDs, ECS entities, renderer resources, or arbitrary labels do not silently become semantic identity.

Package-specific versioning and migration remain package-owned where the package contract supplies them. This implemented classification does not claim a generic cross-domain ID or migration substrate.

## ControlPackage registry

`ui_controls` owns package-backed control semantics and an explicit `ControlPackageRegistry`. Registration validates package contracts and rejects duplicate package or control-kind identity.

`ControlPackageRegistry::snapshot()` produces an explicit `ControlPackageRegistrySnapshot` containing:

```text
packages
control_kinds
schemas
kernels
fixtures
diagnostics
migrations
stories
route_requirements
target_profiles
```

Package resolution is therefore an explicit input/snapshot boundary rather than hidden mutable global state.

Control packages do not own host/application mutation, renderer backend resources, editor commands, game state, or ECS semantics.

## ColorPicker proof obligations

The implemented ColorPicker proof uses the wheel-plus-triangle contract and records package/control identity, property/state/event schema obligations, route/event behavior, diagnostics, fixtures, and migration evidence through the package architecture. RGB-cube projection remains a separately deferred direction and is not implied by this implemented classification.

## UiProgram graph families

`UiProgram` contains typed graph families. These graphs are related through explicit typed identity/provenance; they are not one universal graph or generic node soup.

Current `UiProgramGraphs` contains:

```text
ControlGraph
ControlPropertyGraph
LayoutGraph
StateGraph
StyleGraph
InteractionGraph
BindingGraph
VisualGraph
AccessibilityGraph
InspectionGraph
```

The graph families preserve UI-domain ownership while allowing compiler/artifact/evaluator stages to lower and consume optimized tables.

### ControlGraph

`ControlGraph` owns stable control identity, package/control-kind references, hierarchy, control-local semantic requirements, capability facts, and source provenance.

### ControlPropertyGraph

`ControlPropertyGraph` owns typed property snapshots associated with controls and their schemas. Property values remain source/program facts rather than being reconstructed from renderer output.

### LayoutGraph

`LayoutGraph` owns program-level layout constraints, measurement dependencies, layout-kernel references, and source provenance. Artifact lowering produces runtime layout rows from those facts.

Reusable foundation layout contracts remain legitimately owned by `ui_layout`; this graph does not require `ui_program` to absorb all layout implementation.

### StateGraph

`StateGraph` owns structural state requirements, lifecycle/persistence facts, invalidation dependencies, schemas, and source provenance. Runtime state values and transitions remain separated through `ui_state`, artifact tables, and evaluator execution.

### StyleGraph

`StyleGraph` owns UI style intent, style slots, property schemas, state variants, and provenance. Renderer backends consume resolved output; they do not become style-semantic owners.

### InteractionGraph

`InteractionGraph` owns triggers, focus/input interaction contracts, route identifiers, payload schemas, capabilities, and source provenance. `FormedInteractionModel` remains an execution-neutral upstream contract where applicable.

### BindingGraph

`BindingGraph` owns explicit UI-state, control-property, and host-data endpoints plus snapshot, dependency, authorization, dirty-propagation, and diagnostic facts. Binding failures are reportable; UI does not reach through bindings into private owner state.

### VisualGraph

`VisualGraph` owns draw-neutral visual intent, operators, ordering, visual-kernel references, invalidation relationships, and source provenance. It lowers to renderer-neutral products rather than backend resources.

### AccessibilityGraph

`AccessibilityGraph` owns roles, names/labels, values/states, focus/navigation semantics, source associations, and accessibility runtime rows.

### InspectionGraph

`InspectionGraph` owns inspectable semantic entries, display names, value schemas, bindings, provenance, fixture coverage, and links to runtime/artifact evidence.

## schema route event contracts

UI event output is route- and schema-based rather than a growing central semantic event enum.

Current `UiEventPacket` carries:

```text
route: RouteId
schema_version: RouteSchemaVersion
source_control: Option<UiEventSourceControlId>
phase: UiEventPhase
payload: UiEventPayload
capabilities: Vec<RouteCapability>
source_map: Vec<UiProgramSourceMapEntry>
diagnostics: Vec<UiProgramDiagnostic>
```

`UiEventPayload` carries an explicit `UiSchemaRef` plus `UiSchemaValue`; packet validation can validate that value against a supplied `UiSchema`.

`RouteId` and `RouteCapability` enforce stable namespaced IDs. `RouteSchemaVersion` must be non-zero. Current conformance proves route/event schema/payload/capability behavior and fail-closed invalid cases without requiring a central semantic-event enum.

Correct authority flow:

```text
UiEventPacket
-> host route/capability policy
-> host/domain action or command proposal
-> owner ratification/execution
```

Rejected authority flow:

```text
ControlPackage -> app mutation
renderer primitive -> product mutation
free-form route string -> hidden behavior
```

Generic route migration is not claimed here as a separately implemented runtime subsystem. Migration hooks currently proven by this architecture are package-owned through `ui_controls`; any broader route-migration mechanism requires separate current evidence.

## compiler and runtime artifact contracts

`UiCompiler` consumes `UiProgram` plus explicit package/schema/host/capability/source inputs and produces compiler reports plus `UiRuntimeArtifact` products.

The artifact contract separates:

```text
UiRuntimeArtifactManifest
  inspectable artifact metadata

UiRuntimeArtifactTables
  optimized runtime tables
```

Current `UiRuntimeArtifactManifest` records:

```text
artifact_id
artifact_version
target_profile
target_profile_version
program_id
program_version
cache_key
packages
package_ids
control_kind_ids
schema_ids
route_ids
kernel_ids
capability_ids
capabilities
compiled source_map
diagnostics
```

Current runtime target profiles are:

```text
Editor
Game
WorldSpace
Headless
```

Current `UiRuntimeArtifactTables` contains:

```text
controls
properties
layout
style
state
interaction
binding_snapshots
collection_diffs
visual
text_layout_requests
accessibility
inspection
```

Artifact construction is deterministic from the program and explicit target/profile facts. The manifest derives sorted package/control/schema/route/kernel/capability records, a compiled source map, diagnostics, and a stable cache key. Runtime tables are derived from `UiProgram`; they are execution products, not source truth.

Hot paths consume artifacts rather than interpreting generic authoring graphs by default.

## evaluator and state execution

`UiEvaluator` consumes runtime artifacts and explicit evaluation inputs. It deterministically evaluates compiled program facts while state/binding execution remains separated through `ui_state` and `ui_binding`.

`ui_state` owns runtime-state contracts; `ui_binding` owns host-data/binding contracts; artifact tables own compiled layouts/indexes; evaluator code owns evaluation-time execution and reports. Host/application state remains outside UI semantic authority.

Generic graph interpretation is not the default runtime hot path. Debugging, fixtures, compiler validation, and intentionally low-frequency authoring/inspection work may use richer program structures where appropriate.

## Host boundary

The artifact model explicitly supports editor, game, world-space, and headless target profiles. `ui_hosts` owns generic host profile/capability/route contracts; concrete app/editor/game/world owners retain environment-specific semantics.

Generic UI may provide:

- renderer-neutral output;
- route/event proposals;
- diagnostics/inspection reports;
- host compatibility facts;
- binding snapshots or requests.

Concrete hosts retain:

- command/action execution;
- application/editor/game/domain mutation;
- ECS resource/system ownership;
- world anchors/projection/lifetime policy;
- project/file/network/OS side effects;
- render/backend submission.

The existence of the four target-profile variants proves the program/artifact host boundary. It does **not** claim that every concrete editor/game/world-space integration has completed a consumer cutover.

## retained UI migration boundary

Runenwerk still contains current retained/foundation UI paths. They are supported owners until a separately accepted consumer migration proves and deletes them.

Current retained/foundation responsibilities include at least:

- `ui_math` — primitive UI math/value contracts;
- `ui_schema`, `ui_text`, `ui_theme`, `ui_input`, and `ui_layout` — reusable UI foundation contracts;
- `ui_tree` — retained tree compatibility substrate;
- `ui_widgets` — retained widget compatibility substrate;
- `ui_runtime` — retained runtime compatibility substrate;
- `ui_graph_editor` — current graph-editor compatibility layer;
- `ui_render_data` — renderer-neutral render-data contracts.

`ui_definition` also carries explicit transitional dependencies into retained/render preview/formation paths. `ui_tree` and `ui_runtime` carry explicit transitional graph-editor/render-data edges documented by `ui-crate-ownership.toml`.

The coexistence rule is explicit:

```text
implemented UiProgram architecture
+ retained/foundation compatibility paths
!= completed total consumer cutover
```

Removal or realignment of a retained owner requires a separate accepted boundary with consumer census, replacement proof, diagnostics/behavior parity, migration/rollback evidence where applicable, mechanical owner-map repair, and clean deletion.

## Render and text boundary

UI owns semantic visual/text intent and renderer-neutral facts. `ui_text` owns text model contracts and renderer-neutral text facts; `ui_layout` owns reusable layout contracts/facts; `ui_render_data` and `ui_render_primitives` own renderer-neutral render products.

Renderer/backend owners own GPU resources, backend-specific realization, batching, submission, and resource lifetime. Renderer handles, glyph-atlas GPU resources, command encoders, or backend pipeline objects are not UI semantic truth.

## Accessibility, geometry, and inspection boundary

The implementation has explicit owners for accessibility (`ui_accessibility`), geometry (`ui_geometry`), runtime read models (`ui_runtime_view`), and inspection/evaluator facts (`ui_program` / `ui_evaluator`). These remain usable in headless/testing contexts and do not require renderer ownership.

## fixtures diagnostics source maps

Source maps, diagnostics, fixtures, and reproducibility are first-class architecture contracts.

Current proof includes source-owned unit/integration tests plus `ui_testing` conformance utilities and headless/story/static-mount proof owners. `ui_artifacts` compiles source-map entries into artifact indexes and maps program diagnostics onto compiled source-map indexes. `ui_controls` registry snapshots carry package fixtures, diagnostics, migrations, stories, route requirements, and target profiles as explicit data.

Fixtures and semantic tests cover positive and fail-closed behavior for graph families, package contracts, route/schema/payload validation, artifact lowering, evaluator/state/binding execution, retained compatibility, and renderer-boundary behavior as required by the live conformance registry.

Historical execution evidence remains historical evidence. Current source, tests, and current conformance mapping decide implementation truth.

## Migration and versioning

Current implemented migration truth is bounded:

- control packages carry explicit `ControlMigrationHook` records in registry snapshots;
- package validation can require migration evidence;
- program/artifact/route/schema contracts carry explicit identity/version facts where their concrete current types define them;
- retained-path deletion remains gated by separate accepted migration/cutover work.

This document does not promote unimplemented generic migration machinery merely because the original design described it.

## Conformance and proof

The implemented architecture is backed by current code/tests and the live requirement-to-code/test mapping in `workspace/design-conformance/pt-ui-program-architecture.requirements.yaml`.

That registry marks the architecture requirements verified and maps them to current source/test subjects. The later truth-closure contracts mark both `ui-program-architecture-implementation` and `ui-program-perfectionist-conformance` satisfied only after semantic verification, complete requirement coverage, retained-compatibility/render-boundary probes, evidence/digest closure, and durable checkpoint checks reached zero findings and no known gaps.

Current verified architecture requirements include:

- final owner-map/current-directory closure;
- typed graph families and graph-specific semantic contracts;
- route/event schema/payload/capability validation;
- `ControlPackage` registry and ColorPicker proof obligations;
- compiler-to-runtime-artifact lowering;
- evaluator/state/binding execution;
- retained-UI compatibility and renderer boundary;
- headless fixtures, diagnostics, source maps, and reproducibility;
- explicit blocking of `MaterialProgram` and shared `foundation/meta` extraction;
- extension-readiness through explicit package, route, fixture, diagnostic, migration, and registry boundaries.

This implemented classification records that the architecture-level implementation gap is closed. It does not promote every possible product/framework feature to implemented.

## Historical delivery sequence

The design was delivered through bounded architecture and proof work: program/graph contracts, schema/route/event contracts, compiler/runtime artifacts, evaluator/state execution, host boundaries, retained-UI migration/compatibility, runtime proof fixtures, and later conformance/truth-closure corrections.

The former active design contained detailed stage/PM planning language. That delivery-era sequencing is preserved in repository history, implementation plans, closeouts, audits, and execution evidence. It is not repeated here as current architecture authority because those historical stages no longer authorize or describe future work.

## MaterialProgram gate

This implemented UiProgram classification does not authorize or imply a `MaterialProgram` implementation or a generic domain-program platform. Any future MaterialProgram work must be re-derived from current authority and activated separately.

## foundation/meta gate

No shared `foundation/meta`, universal graph runtime, generic compiler/evaluator framework, or cross-domain semantic substrate is authorized by this implementation. Shared extraction requires a separately accepted architecture decision and concrete cross-domain evidence under current Runenwerk/Engineering authority.

## create an owner only when a proven slice needs that owner

The implemented architecture follows the current owner-map rule: do not create placeholder UI crates merely because a diagram can name a concern. A new owner requires an accepted slice with a semantic responsibility that cannot truthfully remain with an existing owner, a real producer/consumer path, executable proof, and an explicit migration boundary when replacing existing ownership.

Future product features, standalone RunenUI adoption, broad game/editor/world-space integration, and new program families remain separate work until those conditions are met.

## Future integration boundary

Future Runenwerk consumer integration may use this architecture, but this document does not claim completion of every app/editor/game/world-space path. Standalone RunenUI ownership and a Runenwerk consumer cutover are separately issue-owned.

The durable rule is to preserve current semantic owners until a separately accepted cutover proves the replacement and removes the obsolete path cleanly.
