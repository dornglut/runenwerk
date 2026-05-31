---
title: Runenwerk Domain Workbench North Star
description: Long-term accepted direction for Runenwerk as a domain workbench platform, without authorizing immediate implementation or shared platform extraction.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-30
---

# Runenwerk Domain Workbench Platform — Long-Term North-Star Design

## Status

**Accepted Direction — Long-Term North Star**  
**Long-term platform target: accepted as the intended architectural direction**  
**Immediate implementation: not authorized by this document alone**  
**Shared platform extraction: not authorized until proving-domain gates pass**

This document defines the full long-term target shape for Runenwerk as a platform-level domain workbench. It intentionally describes the complete architecture rather than a reduced near-term subset.

The design is ambitious by intent. It should not be weakened into a small helper library, generic graph crate, or incremental UI refactor. The goal is to define the clean end state first, then execute toward it with disciplined phases and proof gates.

## Core Thesis

Runenwerk should become a **Domain Workbench Platform**: a programmable engine-making environment where engine, editor, tool, game, simulation, rendering, world, and asset domains are authored as typed, versioned, inspectable domain programs.

```text
Runenwerk is a platform for building games, worlds, tools, UI,
rendering systems, simulations, assets, and diagnostics from typed,
versioned, inspectable domain programs that compile or evaluate into
efficient runtime artifacts.
```

This is larger than a game engine.  
This is larger than an editor.  
This is larger than a generic meta framework.

The target is a platform where new domains can be defined, authored, validated, evaluated, compiled, inspected, migrated, tested, and hosted with a shared architectural spine.

## Perfectionist Position

The long-term variant is not:

```text
Build a few reusable crates and call it a platform.
```

It is:

```text
Design the whole domain-program workbench as a coherent platform:
meta kernel, domain languages, domain programs, typed graphs,
evaluators, compilers, runtime fabric, authoring tools, hosts,
registries, diagnostics, migrations, and proof systems.
```

No half-measures means:

- do not hide domain semantics in ECS
- do not let the renderer own product truth
- do not create untyped node soup
- do not build one-off editors that cannot inspect or migrate assets
- do not let every domain invent its own schema, graph, diagnostics, migration, and fixture model
- do not treat UI as a special case if the real target is platform-level authoring
- do not optimize only for the next feature while damaging the eventual platform shape

However, no shortcuts also means:

- do not create vague universal abstractions before their semantics are defined
- do not collapse authoring graphs, program graphs, and runtime artifacts into one structure
- do not interpret generic graphs in hot paths by default
- do not build a grand framework without inspection, fixtures, migrations, and diagnostics
- do not call something a platform if it cannot explain itself

The perfectionist version is not the fastest version. It is the version with the least long-term architectural regret.

## North Star

```text
Runenwerk Domain Workbench Platform

A platform where engine domains are authored as typed, versioned,
inspectable programs made of domain-owned graphs, compiled or evaluated
into efficient runtime artifacts, and integrated through explicit editor,
game, headless, CLI, world-space, preview, and remote hosts.
```

The engine runtime is one host.  
The editor is one host.  
The test runner is one host.  
The workbench is the system that creates, validates, inspects, migrates, and evolves domain programs.

## Fundamental Law

```text
Domains own meaning.
The platform owns structure.
```

The platform may know about:

```text
programs
schemas
graphs
nodes
ports
capabilities
evaluators
compilers
artifacts
diagnostics
fixtures
migrations
source maps
packages
manifests
registries
hosts
proofs
```

The platform must not own the meaning of:

```text
buttons
health bars
biomes
materials
render passes
inventory slots
quests
spells
SDF creatures
world regions
animation states
enemy behavior
editor tools
asset import rules
```

Those meanings belong to domains.

## Problem

Runenwerk spans many complex domains:

- editor UI
- game UI
- world-space UI
- materials
- rendering
- render plans
- SDF modeling
- procedural animation
- world generation
- simulation
- gameplay systems
- tools
- inventory
- behavior
- asset import
- diagnostics
- proof reports
- testing
- runtime hosting

If each domain evolves independently, Runenwerk risks duplicated architecture:

- one schema model per domain
- one graph model per domain
- one event model per domain
- one diagnostics model per domain
- one migration strategy per domain
- one fixture strategy per domain
- one editor integration style per domain
- one inspection/debug story per domain
- one proof/reporting style per domain

This creates entropy even if individual systems are well written.

The platform-level goal is to create a shared spine that lets domains be different where meaning differs, but consistent where structure repeats.

## What This Is Not

This is not a generic graph editor.

This is not a universal node runtime.

This is not a plugin soup.

This is not a replacement for Rust.

This is not a reason to put every system into data.

This is not a reason to interpret everything dynamically.

This is not a reason to make domain UX generic.

This is not a UI refactor.

This is a platform architecture for domain programs.

## Core Model

```text
Authored Intent
→ Domain Language
→ Authored Domain Model
→ Authoring Graph
→ Normalized Domain Model
→ Program Graphs
→ Versioned Domain Program
→ Deterministic Evaluator / Compiler
→ Optimized Runtime Artifact
→ Host Integration
→ Diagnostics / Inspection / Proofs
→ Migration / Evolution
```

This full chain matters.

Do not collapse it into:

```text
Graph → runtime
```

That is too weak.

Different stages have different jobs:

```text
Authoring Model
  optimized for humans and tools

Authoring Graph
  optimized for editing and visual manipulation

Normalized Domain Model
  optimized for validation and canonical structure

Program Graphs
  optimized for deterministic evaluation and inspection

Versioned Program
  optimized as the durable executable contract

Runtime Artifact
  optimized for hot-path execution

Host Integration
  optimized for side effects and environment-specific behavior
```

## Definitions

## Domain

A domain is a coherent area of product or engine meaning.

Examples:

```text
UI
Materials
Rendering
World Generation
Simulation
Animation
Tools
Gameplay
Inventory
Behavior
Asset Import
Diagnostics
```

A domain owns:

- vocabulary
- semantics
- authoring UX
- validation rules
- evaluator behavior
- runtime meaning
- inspection meaning
- fixtures
- domain migrations

## Domain Language

A domain language is a way to author domain intent.

It may be:

- visual
- graph-based
- textual
- data-driven
- Rust-authored
- generated
- hybrid

Examples:

```text
UI language
Material language
World rule language
Render surface language
Simulation rule language
Tool language
Animation language
Behavior language
Gameplay language
Asset import language
```

The domain language is not necessarily the runtime format.

## Domain Program

A domain program is the durable executable contract for a domain.

Examples:

```text
UiProgram
MaterialProgram
RenderProgram / RenderPlan
WorldProgram
SimulationProgram
ToolProgram
AssetImportProgram
AnimationProgram
BehaviorProgram
GameplayProgram
```

A domain program contains:

- version metadata
- source map
- typed graphs
- schemas
- required capabilities
- required packages
- validation metadata
- dependency metadata
- evaluator contract
- host contract
- diagnostic hooks
- inspection hooks
- migration metadata
- fixture references
- runtime artifact description

## Graph

A graph is typed structure inside a domain program.

Examples:

```text
ControlGraph
LayoutGraph
StyleGraph
InteractionGraph
BindingGraph
VisualGraph
ShaderGraph
ParameterGraph
PassGraph
ResourceGraph
RegionGraph
SpawnGraph
SimulationGraph
CommandGraph
ValidationGraph
DiagnosticGraph
```

Graphs describe relationships. Programs make those relationships executable, versioned, inspectable, migratable, and hostable.

## Evaluator

An evaluator deterministically executes or analyzes a domain program.

Evaluators produce facts:

- outputs
- events
- plans
- diagnostics
- traces
- inspection reports
- proof artifacts
- runtime artifacts

Evaluators should not hide side effects.

## Compiler

A compiler transforms domain programs or graphs into optimized runtime artifacts.

Examples:

- shader code
- render plans
- GPU buffers
- ECS-ready artifacts
- baked lookup tables
- flattened runtime data
- generated Rust/data artifacts
- cached asset products

## Host

A host connects evaluated outputs to a concrete environment.

Examples:

```text
EditorHost
GameRuntimeHost
WorldSpaceHost
HeadlessTestHost
CliHost
RemoteDevtoolsHost
PreviewHost
```

Hosts perform effects:

- mutate ECS state
- submit render artifacts
- apply editor commands
- perform IO
- trigger asset writes
- dispatch runtime events
- run environment-specific services

## Package

A package contributes domain capabilities.

A package may define:

- schemas
- typed nodes
- graph operators
- evaluator kernels
- compiler kernels
- visual operators
- authoring metadata
- fixtures
- diagnostics
- migrations
- source-map behavior
- inspection behavior
- runtime artifact builders

Packages are how domains grow without central enum bottlenecks.

## Architecture Layers

```text
Runenwerk Domain Workbench Platform

1. Meta Kernel
2. Domain Language Layer
3. Domain Program Layer
4. Graph Layer
5. Evaluator / Compiler Layer
6. Runtime Artifact Layer
7. Runtime Fabric
8. Host Layer
9. Workbench Editor Layer
10. Registry / Governance Layer
11. Proof / Feedback Layer
```

## 1. Meta Kernel

The Meta Kernel is the shared platform spine.

It owns structure, not domain meaning.

Long-term Meta Kernel primitives:

```text
StableId
DomainId
PackageId
SchemaId
ProgramId
GraphId
NodeId
PortId
CapabilityId
EvaluatorId
CompilerId
HostId
ArtifactId
DiagnosticId
FixtureId
MigrationId
ProofId
```

Long-term Meta Kernel systems:

```text
schema registry
program manifest model
package manifest model
typed graph substrate
capability registry
evaluator contract model
compiler contract model
host contract model
diagnostic model
inspection model
source-map model
fixture model
migration model
artifact model
proof model
validation pipeline
serialization/versioning contracts
```

The Meta Kernel must be strict and boring.

It should not contain UI-specific, render-specific, material-specific, world-specific, or gameplay-specific meaning.

## 2. Domain Language Layer

Each domain defines authoring languages appropriate to its meaning.

Examples:

```text
UI language
  controls, layouts, bindings, styles, visual intent

Material language
  shader nodes, parameters, previews, material constraints

World language
  regions, rules, structures, biomes, spawns, constraints

Render language
  render surfaces, pass intents, resource dependencies, frame products

Tool language
  commands, selections, previews, constraints, input mappings

Simulation language
  rules, state transitions, time, constraints, deterministic replay
```

Domain languages are user-facing or author-facing. They should not be forced to look generic.

## 3. Domain Program Layer

Domain programs are stable contracts between authoring, evaluation, compilation, testing, inspection, and hosting.

Candidate long-term programs:

```text
UiProgram
MaterialProgram
RenderProgram
RenderPlan
WorldProgram
SimulationProgram
ToolProgram
AssetImportProgram
AnimationProgram
BehaviorProgram
GameplayProgram
InventoryProgram
QuestProgram
VfxProgram
AudioProgram
DiagnosticsProgram
BuildProgram
TestProgram
```

Not every program must exist immediately, but the platform should be designed so these can exist without architectural rewrites.

## 4. Graph Layer

The Graph Layer provides typed graph infrastructure.

The platform may provide common graph structure:

- graph identity
- node identity
- port identity
- edges
- dependency metadata
- source mapping
- validation hooks
- traversal utilities
- serialization hooks
- diagnostic attachment points

Domains define graph meaning.

No untyped universal node soup.

Correct shape:

```text
TypedGraph<DomainGraphKind>
```

Not:

```text
UniversalNodeGraph
```

## 5. Evaluator / Compiler Layer

Evaluators and compilers are domain-owned but platform-shaped.

They should share contracts for:

- input manifest
- required capabilities
- output packets
- diagnostics
- trace output
- inspection report
- proof report
- artifact declaration
- reproducibility metadata

Evaluators should be deterministic unless explicitly marked otherwise.

Compilers should produce optimized artifacts for runtime use.

## 6. Runtime Artifact Layer

Runtime artifacts are optimized products created from programs.

Examples:

```text
UiRuntimeTree
UiRenderBatches
ShaderModuleArtifact
MaterialPipelineArtifact
RenderPlanArtifact
WorldChunkRecipe
SpawnTableArtifact
SimulationRuntimePlan
ToolCommandTable
AssetImportRecipe
AnimationRuntimeGraph
BehaviorRuntimePlan
```

Runtime artifacts may be cached, hashed, diffed, tested, inspected, and migrated.

They are not the same as authoring graphs.

## 7. Runtime Fabric

The Runtime Fabric is the low-level execution substrate.

It includes:

- ECS
- renderer
- asset system
- scheduler
- IO
- GPU backend
- platform services
- persistence
- networking where applicable
- task runtime
- cache system

The Runtime Fabric executes optimized artifacts and hosts domain outputs.

It must not become the semantic owner of domains.

## 8. Host Layer

Hosts integrate domain outputs into environments.

Target hosts:

```text
EditorHost
GameRuntimeHost
WorldSpaceHost
HeadlessTestHost
CliHost
PreviewHost
RemoteDevtoolsHost
BuildHost
CiHost
```

Hosts own side effects. Evaluators own facts.

## 9. Workbench Editor Layer

The Workbench Editor is the authoring and inspection product.

It should eventually include:

- domain browsers
- program editors
- graph editors
- schema inspectors
- package editors
- capability inspectors
- evaluator trace viewers
- compiler artifact viewers
- fixture previews
- diagnostics dashboards
- migration panels
- proof report viewers
- visual diff tools
- source-map navigation
- runtime artifact inspectors
- cross-domain dependency viewers

The Workbench Editor is not just a UI shell. It is the place where the platform becomes usable.

## 10. Registry / Governance Layer

A platform needs registries.

Long-term registries:

```text
Domain Registry
Program Registry
Package Registry
Schema Registry
Capability Registry
Evaluator Registry
Compiler Registry
Host Registry
Artifact Registry
Fixture Registry
Migration Registry
Diagnostic Registry
Proof Manifest Registry
```

These registries should support:

- validation
- generated docs
- architecture maps
- compatibility checks
- migration planning
- proof reports
- test selection
- dependency inspection

Without registries, the platform will drift.

## 11. Proof / Feedback Layer

The Proof / Feedback Layer makes the platform explainable.

Every serious domain should support some combination of:

- validation reports
- evaluation traces
- compiler reports
- dependency reports
- migration reports
- fixture reports
- diagnostic reports
- performance reports
- visual diffs
- source-map reports
- reproducibility metadata
- proof manifests

The workbench should answer:

```text
Why did this output happen?
Which authored source caused it?
Which graph node contributed?
Which package owns it?
Which schema validated it?
Which evaluator produced it?
Which host applied it?
Which migration touched it?
Which fixture proves it?
Which runtime artifact contains it?
```

## Universal Domain Pipeline

Every serious domain should follow the full platform pipeline:

```text
Authored Domain Package
→ Domain Language
→ Authored Domain Model
→ Authoring Graph
→ Schema Validation
→ Normalized Domain Model
→ Program Graphs
→ Versioned Domain Program
→ Deterministic Evaluator / Compiler
→ Runtime Artifact
→ Host Integration
→ Diagnostics / Inspection / Proofs
→ Migration / Evolution
```

This is the long-term perfectionist pipeline.

It does not mean every tiny internal helper becomes a domain program. It means every durable, authorable, inspectable, cross-host domain should eventually have this shape.

## Domain-Program Admission Rule

A system should become a domain program if it needs several of the following:

- authoring
- persistence
- versioning
- inspection
- migration
- fixtures
- cross-host execution
- deterministic replay
- compilation
- external tooling
- schema validation
- diagnostics
- proof artifacts
- generated documentation

Ordinary Rust modules remain valid for low-level or internal systems that do not need these properties.

Perfectionism does not mean forcing every helper into the platform. It means choosing the correct abstraction level and fully honoring it once chosen.

## Cross-Domain Dependency Rules

Domains may depend on another domain's public contracts:

- program contracts
- runtime artifact contracts
- inspection contracts
- diagnostic contracts
- package manifests
- schema IDs
- capability IDs

Domains must not depend on another domain's private internals:

- authoring internals
- evaluator internals
- compiler internals
- private graph representations
- private runtime caches

Example:

```text
UI may display a MaterialProgram preview artifact.
UI must not reach directly into private MaterialGraph internals.
```

Example:

```text
Render may consume MaterialPipelineArtifact.
Render must not own material product truth.
```

Example:

```text
ToolProgram may edit WorldProgram through world-domain commands.
ToolProgram must not mutate private world generation internals directly.
```

## Runtime Rule

Do not run the platform as generic graph interpretation by default.

Correct shape:

```text
Authoring Graph
→ Normalized Graph
→ Program Graph
→ Runtime Artifact
→ Specialized Runtime Execution
```

Hot paths should use:

- baked plans
- flattened tables
- GPU buffers
- compiled shaders
- specialized evaluators
- native Rust systems
- ECS-ready artifacts
- cached runtime products

Generic interpretation is allowed only when explicitly justified by the domain and performance envelope.

## Hard Design Laws

1. **Domains own meaning.**  
   Platform structure must not erase domain semantics.

2. **The platform owns structure.**  
   Shared concepts include IDs, schemas, manifests, graph structure, capabilities, diagnostics, fixtures, migrations, inspection, and proofs.

3. **Programs are durable contracts.**  
   Widgets, ECS components, Rust traits, and renderer structs are not the primary cross-layer contract.

4. **Graphs are typed internal structure.**  
   Graphs are not the architecture by themselves.

5. **No untyped node soup.**  
   Every node belongs to a typed domain graph with validation and evaluation semantics.

6. **Evaluators produce facts. Hosts perform effects.**  
   Side effects must cross explicit host boundaries.

7. **Compilers produce runtime artifacts.**  
   Runtime efficiency is a first-class platform requirement.

8. **ECS does not own domain semantics.**  
   ECS is runtime fabric and host integration.

9. **Renderer does not own product truth.**  
   Renderer consumes artifacts, plans, primitives, and resources.

10. **Every authored asset is versioned and migratable.**  
    Migration is not cleanup. It is platform infrastructure.

11. **Every serious program is inspectable.**  
    If a program cannot explain its outputs, it is not platform-grade.

12. **Every package ships fixtures and diagnostics.**  
    Packages must be previewable, testable, and explainable.

13. **Authoring format is not runtime truth.**  
    Authoring models compile into normalized programs and runtime artifacts.

14. **Shared abstractions must improve domain quality.**  
    If a shared primitive weakens domain UX or semantics, it is wrong.

15. **The platform must be designed completely, even if implemented in phases.**  
    Phasing is sequencing, not reduction of ambition.

## Non-Goals

This design does not authorize:

- generic node soup
- universal evaluator
- universal graph editor as the foundation
- universal plugin magic
- renderer-owned product truth
- ECS-owned domain semantics
- hot-path generic interpretation by default
- one-off UI-only architecture pretending to be platform architecture
- creating all crates immediately without domain proof
- replacing low-level engine infrastructure with meta abstractions
- weakening domain UX to satisfy generic platform constraints

## Domain Targets

## UI Domain

Target program:

```text
UiProgram
├── ControlGraph
├── LayoutGraph
├── StyleGraph
├── InteractionGraph
├── BindingGraph
├── VisualGraph
├── AccessibilityGraph
└── InspectionGraph
```

UI supports:

- editor panels
- game HUD
- world-space UI
- inventories
- health bars
- radial menus
- node graphs
- material inspectors
- debug overlays
- interaction prompts
- self-authoring controls

UI packages contribute:

- control schemas
- property schemas
- state schemas
- event payload schemas
- style slots
- layout kernels
- input kernels
- visual kernels
- accessibility metadata
- editor metadata
- fixtures
- diagnostics
- migrations

## Material Domain

Target program:

```text
MaterialProgram
├── ShaderGraph
├── ParameterGraph
├── PreviewGraph
├── CapabilityGraph
├── ArtifactGraph
└── DiagnosticGraph
```

Material supports:

- visual material graphs
- code-authored materials
- shader generation
- parameter editing
- previews
- material fixtures
- compatibility diagnostics
- renderer artifact generation

## Render Domain

Target programs:

```text
RenderProgram
RenderPlan
RenderSurfaceProgram
```

Possible graphs:

```text
SurfaceGraph
PassGraph
ResourceGraph
DependencyGraph
FrameGraph
ArtifactGraph
DiagnosticGraph
```

Render consumes artifacts and product requests. It does not own product truth.

## World Domain

Target program:

```text
WorldProgram
├── RegionGraph
├── RuleGraph
├── SpawnGraph
├── BiomeGraph
├── StructureGraph
├── EcologyGraph
├── ConstraintGraph
└── SimulationGraph
```

World supports:

- finite worlds
- infinite worlds
- dungeons
- arenas
- regions
- biomes
- structures
- caves
- nests
- procedural rules
- authored overrides
- deterministic generation
- replay and inspection

## Simulation Domain

Target program:

```text
SimulationProgram
├── StateGraph
├── RuleGraph
├── TimeGraph
├── ConstraintGraph
├── EventGraph
└── DiagnosticGraph
```

Simulation supports:

- deterministic replay
- state transitions
- constraints
- time stepping
- headless testing
- proof reports

## Tool Domain

Target program:

```text
ToolProgram
├── CommandGraph
├── InputGraph
├── SelectionGraph
├── PreviewGraph
├── ConstraintGraph
└── DiagnosticGraph
```

Tools should be authorable, inspectable, fixture-tested, and hostable instead of only hard-coded editor behavior.

## Asset Import Domain

Target program:

```text
AssetImportProgram
├── SourceGraph
├── TransformGraph
├── ValidationGraph
├── DependencyGraph
├── ArtifactGraph
└── DiagnosticGraph
```

Asset import should be reproducible, inspectable, migratable, and testable.

## Gameplay Domain

Target programs may include:

```text
GameplayProgram
InventoryProgram
QuestProgram
AbilityProgram
BehaviorProgram
ProgressionProgram
```

Gameplay domain programs should support authored rules, deterministic validation, fixture worlds, runtime artifacts, and explicit host integration.

## Animation Domain

Target program:

```text
AnimationProgram
├── PoseGraph
├── StateGraph
├── BlendGraph
├── ProceduralRuleGraph
├── ConstraintGraph
└── DiagnosticGraph
```

Animation supports authored and procedural motion, including SDF/procedural character systems.

## Platform Repository Target

Long-term possible shape:

```text
foundation/
  ids/
  schema/
  graph/
  program/
  package/
  capability/
  diagnostics/
  inspection/
  migration/
  fixtures/
  artifacts/
  proof/

domain/
  ui/
    ui_language/
    ui_program/
    ui_controls/
    ui_evaluator/
    ui_compiler/
    ui_artifacts/
    ui_hosts/

  material/
    material_language/
    material_program/
    material_graph/
    material_evaluator/
    material_compiler/
    material_artifacts/
    material_preview/

  render/
    render_program/
    render_plan/
    render_surface/
    render_compiler/
    render_artifacts/
    render_diagnostics/

  world/
    world_language/
    world_program/
    world_rules/
    world_generation/
    world_evaluator/
    world_artifacts/

  simulation/
    simulation_program/
    simulation_rules/
    simulation_evaluator/
    simulation_artifacts/

  tools/
    tool_language/
    tool_program/
    tool_evaluator/
    tool_packages/
    tool_artifacts/

  gameplay/
    gameplay_program/
    inventory_program/
    ability_program/
    quest_program/
    behavior_program/

apps/
  runenwerk_editor/
    workbench/
    domain_browsers/
    graph_editors/
    program_editors/
    inspectors/
    package_editors/
    fixture_previews/
    migration_panels/
    diagnostics_panels/
    proof_viewers/

  runenwerk_runtime/
    hosts/
    runtime_integration/

  runenwerk_cli/
    validation/
    migration/
    artifact_builds/
    proof_reports/

docs-site/
  design/
  architecture/
  domain-registry/
  program-registry/
  package-registry/
  schema-registry/
  capability-registry/
  migration-registry/
  proof-manifests/
```

This is the long-term shape. It is not a request to create empty crates prematurely.

## Governance and Maturity Levels

Platform work should use explicit maturity levels:

```text
Exploratory North Star
Accepted Direction
Design Authorized
Implementation Authorized
Proof Accepted
Extraction Authorized
Runtime Accepted
Tooling Accepted
Deprecated / Superseded
```

Meaning:

```text
Exploratory North Star
  Defines target shape. Does not authorize code.

Accepted Direction
  Direction is agreed, but implementation scope is still bounded.

Design Authorized
  A specific domain design may be written.

Implementation Authorized
  A specific bounded implementation slice may be built.

Proof Accepted
  A domain proves the concept with tests, fixtures, diagnostics, and docs.

Extraction Authorized
  A repeated primitive may move into shared platform/foundation ownership.

Runtime Accepted
  Runtime artifact path is proven efficient enough.

Tooling Accepted
  Workbench/editor support is sufficient for real use.
```

## Acceptance Gates

## Design Gate

A domain-program design may proceed when it defines:

- domain ownership
- program boundary
- graph ownership
- schema strategy
- evaluator contract
- compiler/artifact strategy
- host boundary
- diagnostics strategy
- fixture strategy
- migration strategy
- inspection strategy

## Proof Gate

A domain proves the architecture when it has:

- a real program
- typed graphs
- real evaluator/compiler path
- real diagnostics
- real fixtures
- headless tests where applicable
- host integration
- source maps
- migration story
- docs

## Second-Domain Gate

A platform primitive is considered real only after a second domain proves it independently.

The second domain should not be a trivial sibling. UI plus material/render is stronger than UI plus another UI package.

## Extraction Gate

A primitive may move into shared platform/foundation ownership only when:

1. At least two domains need it.
2. Its contract is domain-agnostic.
3. It does not weaken domain meaning.
4. It improves inspection, validation, migration, or testing.
5. It has versioning implications documented.
6. It does not force runtime overhead.
7. It has docs and tests.

## Runtime Gate

A domain program is runtime-acceptable only when:

- hot paths use optimized artifacts
- performance costs are explicit
- caching strategy is defined
- artifact invalidation is defined
- runtime diagnostics exist
- host integration is explicit

## Tooling Gate

A domain is workbench-grade only when it has:

- fixture previews
- program inspection
- diagnostics display
- source-map navigation
- migration reports
- artifact inspection where applicable

## Target Milestones

## Milestone 1 — UI Program as First Platform Proof

UI should prove:

- `UiProgram` as durable contract
- typed UI graphs
- schema-based events
- control packages
- stable kernel/capability IDs
- deterministic evaluator
- visual/render boundary
- style/theme/token system
- fixture previews
- diagnostics
- source maps
- editor host
- game host
- world-space host
- headless host

Success criterion:

```text
A UiProgram can be authored, previewed, inspected, fixture-tested,
evaluated headlessly, and hosted in editor/game/world-space contexts.
```

## Milestone 2 — MaterialProgram or RenderPlan as Second Platform Proof

A non-UI domain should prove the same platform spine.

Preferred candidates:

1. `MaterialProgram`
2. `RenderPlan`
3. `RenderSurfaceProgram`

Success criterion:

```text
A MaterialProgram or RenderPlan reuses the same platform concepts for
program manifests, schema IDs, diagnostics, fixtures, source maps,
artifacts, and inspection without sharing UI meaning.
```

## Milestone 3 — Platform Kernel Extraction

Only after UI and material/render prove the shape should shared kernel primitives move into foundation/platform crates.

This extraction should be deliberate, not opportunistic.

## Milestone 4 — Workbench Tooling

Build authoring and inspection tools around proven programs:

- graph editors
- program inspectors
- package editors
- fixture previews
- migration panels
- diagnostics dashboards
- proof viewers
- artifact inspectors

## Milestone 5 — World, Simulation, Gameplay, Animation, Asset Import

Expand the platform once the spine is proven by UI and material/render.

Candidate expansion:

- `WorldProgram`
- `SimulationProgram`
- `ToolProgram`
- `AssetImportProgram`
- `AnimationProgram`
- `GameplayProgram`

## Risk Register

## Risk: Second-System Syndrome

The platform could become an engine-making engine that delays actual engine progress.

Mitigation:

- keep the north star complete
- implement through real vertical domains
- require proof gates
- forbid empty abstractions

## Risk: Generic Graph Soup

Everything could become a node with unclear meaning.

Mitigation:

- typed domain graphs only
- no universal node runtime
- every graph has validation and evaluator semantics

## Risk: Weak Domain UX

Generic tooling can make every domain feel unnatural.

Mitigation:

- domains own authoring UX
- platform owns structure only
- package fixtures and diagnostics are required

## Risk: Runtime Overhead

Graph interpretation can harm performance.

Mitigation:

- compile to runtime artifacts
- keep hot paths specialized
- require runtime gates

## Risk: Debugging Opacity

Meta systems become unusable without explanation tools.

Mitigation:

- source maps mandatory
- diagnostics mandatory
- traces/proofs first-class
- workbench inspection required

## Risk: Migration Burden

Versioned assets require long-term migration discipline.

Mitigation:

- schema IDs
- program versions
- package versions
- migration registry
- migration reports

## Risk: Documentation Drift

Platform concepts can drift from implementation.

Mitigation:

- registries
- generated docs
- validators
- proof manifests
- closeout discipline

## Risk: Premature Extraction

Shared crates can freeze bad abstractions.

Mitigation:

- complete target design first
- extract after two-domain proof
- require extraction gate

## Immediate Next Decision

The next decision is not whether to create a generic meta framework immediately.

The next decision is:

```text
Should Runenwerk accept the Domain Workbench Platform as the long-term
north star, and should UI Program Architecture become the first proving
domain for that platform?
```

If yes, the next design document should be:

```text
docs-site/src/content/docs/design/active/ui-program-architecture.md
```

That document should be written as the first concrete platform proof, not as an isolated UI cleanup.

## Final Target Shape

```text
Runenwerk Domain Workbench Platform

A programmable platform where engine domains are authored as typed,
versioned, inspectable programs made of domain-owned graphs, compiled or
evaluated into efficient runtime artifacts, and integrated through explicit
editor, game, headless, CLI, world-space, preview, and remote hosts.
```

The long-term product is not only an engine and not only an editor.

The long-term product is a domain-program workbench for building games, worlds, tools, UI, rendering systems, simulations, assets, diagnostics, and runtime experiences with durable, inspectable, evolvable architecture.
