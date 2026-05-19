---
title: Runenwerk Capability Workbench Target Architecture
description: Active future target architecture for a capability-governed Runenwerk workbench platform based on Capability Cells, Platform Planes, and Maturity Layers.
status: active
owner: editor
layer: cross-domain
canonical: false
last_reviewed: 2026-05-19
related_docs:
  - ../../guidelines/authority-centered-boundary-architecture.md
related_designs:
  - ./editor-tool-suite-registry-and-workbench-host-design.md
---

# Runenwerk Capability Workbench Target Architecture

## Status

Active future target design. This document is not an ADR, not an accepted
decision record, and not an implemented baseline.

It specializes the general
[`../../guidelines/authority-centered-boundary-architecture.md`](../../guidelines/authority-centered-boundary-architecture.md)
doctrine for Runenwerk workbench tooling. The existing `WorkbenchHost` /
`ToolSuite` foundation remains valid. Dynamic plugins remain deferred.

## 1. Purpose

Runenwerk should become a capability-governed workbench platform for domain-specific game, engine, content, simulation, rendering, procedural generation, diagnostics, runtime, and automation tools.

The target architecture is not a classic plugin marketplace and not a monolithic editor application. It is a structured platform where tools are modeled as Capability Cells, governed by shared Platform Planes, and delivered over time through Maturity Layers.

The final synthesis is:

```text
Runenwerk Capability Workbench
  Tool Unit:        Capability Cell
  Governance:      Platform Planes
  Implementation:  Maturity Layers
```

This gives Runenwerk a long-term model for:

- full editor tools,
- standalone focused editors,
- in-game constrained editors,
- headless validation hosts,
- AI-assisted workflows,
- out-of-process tool services,
- future external sandboxed components.

The goal is:

```text
stable identity,
declared capability,
host-owned policy,
domain-owned truth,
provider-owned presentation,
product-graph formation,
reconciled status,
service-isolated heavy work,
and external components only after explicit sandboxing.
```

## 2. Core Doctrine

### 2.1 Tools Request, Hosts Allow, Domains Validate

A tool does not gain authority merely because it is installed.

Every meaningful operation should pass through three distinct concepts:

```text
Capability declaration:
  what the tool says it can do

Host policy:
  whether the current host allows that ability

Domain validation:
  whether the requested operation is semantically valid
```

Example:

```text
Material Lab declares:
  can propose material_graph.connect_edge

Full editor host:
  allows material_graph.connect_edge

In-game editor host:
  rejects material_graph.connect_edge
  may allow material_instance.set_color_parameter

domain/material_graph:
  validates whether a proposed edge is legal
```

### 2.2 Stable Identity Is Mandatory At Boundaries

Durable cross-layer identity must use stable keys.

```text
Tool surfaces        -> ToolSurfaceStableKey
Tool suites          -> ToolSuiteId
Capability cells     -> CapabilityCellId
Provider families    -> ProviderFamilyId
Commands             -> CommandCapabilityKey
Products             -> ProductCapabilityKey
Resources            -> ResourceCapabilityKey
Host capabilities    -> HostCapabilityKey
Services             -> ToolServiceKey
Components           -> ComponentId
```

Enums may still exist for local exhaustive logic, structural grouping, or legacy compatibility, but they must not become durable public identity unless explicitly designed as stable protocol identity.

### 2.3 Domains Own Truth

Domain crates own source truth, commands, validation, ratification, source maps, lowering rules, semantic diagnostics, and product descriptors.

Examples:

```text
domain/material_graph
  owns material graph source truth, material graph commands, validation, ratification, lowering, source maps, and material semantics.

domain/texture
  owns texture descriptors, texture contracts, generated texture product semantics, and texture validation contracts.

domain/procgen
  owns procedural generation source truth, graph semantics, rules, lowering contracts, and generated product descriptors.
```

No provider, service, host, UI surface, manifest, or external component may bypass the owning domain's command and validation path.

### 2.4 Providers Own Presentation

Providers build UI, expose view models, map local interactions, and propose commands. Providers are not semantic authority.

Providers own presentation, not source truth.

Providers may:

- build frames and local surface UI,
- expose presentation DTOs,
- map surface interactions into command proposals,
- display diagnostics,
- observe host/domain state through approved read models.

Providers must not:

- own source truth,
- mutate domain state directly,
- bypass host policy,
- own renderer products,
- own project IO policy,
- become hidden workflow engines.

### 2.5 Products Are Formed Through Product Contracts

Products are formed artifacts derived from source truth and validated contracts.

```text
source truth
  -> domain validation / lowering
  -> product descriptor
  -> product graph node
  -> app/service formation
  -> formed product
  -> engine/render/runtime consumption
```

Render and runtime systems consume formed products. They do not own authoring source truth.

### 2.6 Desired State And Observed Status Are Separate

Workbench state should be reconciled, not mutated by uncontrolled side effects.

```text
desired suite/profile/capability/product/service state
  -> validation
  -> reconciliation
  -> mounted surfaces / providers / services / products
  -> observed status / diagnostics
```

This pattern applies to full editor workspaces, standalone editors, in-game editors, headless validation, service orchestration, and future external components.

## 3. Architecture Summary

The clean target architecture is a hybrid of cells, planes, and maturity layers.

```text
Capability Cell
  crosses all relevant planes

Platform Planes
  define authority and governance

Maturity Layers
  define safe implementation order
```

### Why This Hybrid Is Better Than A Pure Layer Stack

A pure six-layer stack is good for roadmap sequencing, but real tools cut across layers.

Material Lab is not only a surface and not only a service. It has surfaces, providers, command capabilities, product capabilities, resource access, diagnostics, host rules, and eventually service execution.

A Capability Cell models that real shape.

### Why This Hybrid Is Better Than A Classic Plugin Model

A classic plugin model often starts with:

```text
load code and let it extend the app
```

Runenwerk should start with:

```text
declare identity,
declare capabilities,
check host policy,
validate in domain,
form products through product contracts,
execute services under supervision,
allow external components only after sandboxing exists.
```

## 4. Capability Cells

### 4.1 Definition

A Capability Cell is the durable architectural unit for a tool family.

Examples:

```text
Material Lab Cell
Texture Lab Cell
Procgen Lab Cell
Animation Lab Cell
Physics Debugger Cell
Diagnostics Cell
In-Game Building Editor Cell
```

A cell declares what it provides and what it may request. It does not automatically receive permission.

### 4.2 Cell Anatomy

A mature Capability Cell contains:

```text
id
version
surfaces
providers
provider families
command capabilities
product capabilities
resource capabilities
service dependencies
host compatibility
status conditions
migration metadata
diagnostics metadata
documentation links
```

### 4.3 Example: Material Lab Cell

```text
CapabilityCell "runenwerk.material_lab"

Identity:
  runenwerk.material_lab

Surfaces:
  runenwerk.material_lab.graph_canvas
  runenwerk.material_lab.inspector
  runenwerk.material_lab.preview

Providers:
  MaterialGraphCanvasProvider
  MaterialInspectorProvider
  MaterialPreviewProvider

Command capabilities:
  material_graph.add_node
  material_graph.connect_edge
  material_graph.set_parameter

Product capabilities:
  material.preview.shader
  material.runtime.material_product

Resource capabilities:
  texture.catalog.read
  asset.catalog.read
  render.preview.request

Host compatibility:
  full_editor: allowed
  standalone_material_editor: allowed
  in_game_editor: restricted parameter mode only
  headless_ci: validation and product formation only

Status conditions:
  source_valid
  resource_bindings_resolved
  preview_published
  preview_failed
  last_good_available
```

### 4.4 Cell Rule

A cell declares intent and required integration points.

It does not:

- mutate source truth directly,
- grant itself permissions,
- bypass domain validation,
- bypass product registries,
- bypass host policy,
- execute external code by default.

## 5. Platform Planes

Platform Planes are cross-cutting governance systems. Every Capability Cell is interpreted through these planes.

### 5.1 Identity Plane

#### Purpose

Provide stable identity for all durable workbench concepts.

#### Owns

- stable key grammar,
- id validation,
- id namespaces,
- collision detection,
- deprecation metadata,
- migration metadata.

#### Examples

```text
runenwerk.material_lab.graph_canvas
runenwerk.texture.viewer_2d
runenwerk.procgen.graph_canvas
runenwerk.command.material_graph.connect_edge
runenwerk.product.material.preview_shader
runenwerk.host.render_preview
```

#### Invariants

- Durable identity is never label text.
- Durable identity is never provider order.
- Durable identity is never incidental enum naming.
- Unknown stable keys fail closed.

### 5.2 Capability Plane

#### Purpose

Describe what a cell may ask to do.

#### Owns

- capability declarations,
- command capability keys,
- product capability keys,
- resource capability keys,
- host capability keys,
- declared service dependencies.

#### Invariants

- Capability declaration is not permission.
- Capabilities are stable-keyed.
- Capabilities are inspectable.
- Providers cannot self-grant capabilities.

### 5.3 Host Policy Plane

#### Purpose

Decide what is allowed in a specific host.

#### Host Types

```text
FullWorkbenchHost
StandaloneFocusedHost
InGameEditorHost
HeadlessValidationHost
AutomationHost
```

#### Owns

- allow/deny decisions,
- trust-zone policy,
- resource access policy,
- command policy,
- product policy,
- service invocation policy,
- runtime permission policy.

#### Policy Decision And Enforcement

```text
Policy Decision Point:
  HostCapabilityPolicy decides allow/deny

Policy Enforcement Points:
  command dispatch
  product formation
  service invocation
  resource access
  workspace mutation
  external component call
```

#### Invariants

- Host policy cannot override domain validation.
- Domain validation cannot grant host permission.
- Denied capability requests fail closed.
- Policy decisions should be observable in diagnostics.

Host policy and domain validation stay separate.

### 5.4 Domain Truth Plane

#### Purpose

Preserve semantic correctness and source authority.

#### Owns

- source documents,
- command schemas,
- command validation,
- ratification,
- semantic diagnostics,
- lowering rules,
- source maps,
- product descriptors.

#### Invariants

- Domains own meaning.
- Providers propose; domains validate.
- Services compute; domains validate outputs where relevant.
- Hosts allow; domains decide semantic validity.

### 5.5 Product Graph Plane

#### Purpose

Represent product formation as explicit dependency/product graph work, not ad hoc side effects.

#### Owns

- product descriptor graph,
- dependencies,
- dirty tracking,
- cache keys,
- product provenance,
- invalidation,
- product diagnostics,
- product adoption rules.

#### Execution Modes

```text
in-process job
background worker
out-of-process service
remote service
future sandboxed component
```

#### Invariants

- Products are formed from source/product descriptors.
- Product graph nodes are observable.
- Failed products produce diagnostics.
- Last-good preservation is explicit.
- Render/runtime consumes formed products only.

### 5.6 Reconciliation / Status Plane

#### Purpose

Convert desired workbench state into observed mounted/formed/running state and expose status.

#### Desired State Examples

```text
suite installed
surface mounted
capability requested
provider family assigned
product requested
service desired
external component desired
```

#### Observed Status Examples

```text
surface mounted
provider resolved
provider missing
capability denied
product formed
product failed
service running
service unavailable
component rejected
diagnostics emitted
```

#### Invariants

- Desired and observed state are separate.
- Status is explicit.
- Failure is diagnostic, not silent.
- Inspectors can show status and reasons.

### 5.7 Execution Plane

#### Purpose

Run heavy, isolated, or external work under policy and supervision.

#### Execution Forms

```text
trusted in-process job
background worker
out-of-process service
remote service
external sandboxed component
```

#### Candidate Workloads

```text
material compilation
texture generation
asset import
procgen evaluation
physics analysis
animation solving
AI validation
world-product generation
```

#### Invariants

- Execution cannot mutate source truth directly.
- Outputs are diagnostics, command proposals, product candidates, or reports.
- Host supervises lifecycle.
- Services/components are restartable or fail safely.
- External components require sandboxing and explicit capability policy.

## 6. Maturity Layers

Maturity Layers define implementation order.

```text
Layer 1 - Internal ToolSuites
Layer 2 - Declarative Suite Manifests
Layer 3 - Capability Contracts
Layer 4 - Product / Command / Permission Registries
Layer 5 - Tool Execution Protocol
Layer 6 - External Sandboxed Components
```

### 6.1 Layer 1 - Internal ToolSuites

First-party compiled-in suite declarations.

Purpose:

- stable surfaces,
- provider families,
- workbench composition,
- provider matching,
- V5 workspace persistence.

### 6.2 Layer 2 - Declarative Suite Manifests

Data-authored metadata for suites and surfaces.

Purpose:

- make suite metadata inspectable,
- support self-authoring,
- enable manifest validation,
- prepare for external components later.

Manifests are metadata only. They do not execute behavior.

### 6.3 Layer 3 - Capability Contracts

Declarative ability model.

Purpose:

- describe what a suite may request,
- define command/product/resource capability keys,
- prepare for host policy enforcement.

Capabilities are not permissions.

### 6.4 Layer 4 - Product / Command / Permission Registries

Enforcement infrastructure.

Purpose:

- check command capability,
- check product capability,
- check resource access,
- check host policy,
- record allow/deny diagnostics.

### 6.5 Layer 5 - Tool Execution Protocol

Protocol for heavy work execution.

Purpose:

- unify in-process jobs, background workers, and out-of-process services,
- keep tool execution supervised,
- keep outputs explicit.

### 6.6 Layer 6 - External Sandboxed Components

Deferred external extension model.

Purpose:

- eventually support third-party or modder components,
- consume the same manifests/capabilities/policies,
- avoid privileged side APIs.

### 6.7 Dependency Order

The order is strict:

```text
Layer 1 before Layer 2
Layer 2 before Layer 3
Layer 3 before Layer 4
Layer 4 before Layer 5
Layer 5 before Layer 6
```

Do not implement external sandboxed components before manifests, capabilities, and host policy exist.

Do not implement tool services before command/product/permission boundaries are clear.

Do not build capability contracts before stable suite/surface identity is reliable.

## 7. Host Types

### 7.1 Full Workbench Editor

Installs broad authoring capabilities.

```text
Runenwerk Editor
  -> scene tools
  -> material tools
  -> texture tools
  -> procgen tools
  -> UI tools
  -> diagnostics tools
  -> runtime/render preview adapters
```

Authority:

- project IO,
- asset editing,
- source document editing,
- product publication,
- preview/runtime adapters,
- broad command/product capabilities.

### 7.2 Standalone Focused Editor

Installs only relevant cells and capabilities.

Examples:

```text
Runenwerk Material Editor
Runenwerk Texture Lab
Runenwerk Procgen Lab
Runenwerk UI Editor
```

Authority:

- narrower host policy,
- focused suite installation,
- same domain commands,
- same product contracts,
- less UI clutter.

### 7.3 In-Game Editor Host

A constrained host, not a separate architecture.

Authority:

- runtime-safe subset,
- gameplay/session authority participates,
- user/admin/modder permissions,
- curated surfaces,
- restricted commands,
- no full project source authority unless explicitly allowed.

Example:

```text
Full editor:
  material_graph.connect_edge allowed

In-game editor:
  material_graph.connect_edge rejected
  material_instance.set_color_parameter allowed
```

### 7.4 Headless / CI Host

Runs tools without UI surfaces.

Authority:

- validation,
- import checks,
- product formation checks,
- diagnostics/report generation,
- regression proof.

### 7.5 Automation / AI Host

Runs automation clients through explicit capability and command-proposal rules.

Authority:

- may inspect state,
- may propose commands,
- may request validation,
- may request product previews,
- must not bypass host policy or domain validation.

## 8. Trust Zones

Different origins imply different permissions.

```text
first-party compiled-in cell
first-party declarative manifest
project-authored manifest
workspace-local automation
in-game user-authored content
external sandboxed component
AI/automation client
```

Trust-zone rules:

- first-party compiled-in cells may use internal Rust APIs through normal code review,
- declarative manifests execute no behavior,
- project-authored manifests require validation,
- in-game user content is runtime-policy constrained,
- external components are sandboxed and capability-gated,
- AI clients propose commands and cannot perform hidden writes.

## 9. Promotion Path

Not everything starts as a Capability Cell.

Recommended path:

```text
prototype
  -> app-local first-party tool
  -> internal stable-key ToolSuite
  -> Capability Cell
  -> declarative manifest
  -> capability-governed cell
  -> service-backed cell
  -> external sandboxed component if needed
```

This prevents bureaucracy while preserving a clean promotion path for durable tools.

## 10. Representative Cells

### 10.1 Material Lab Cell

Purpose:

- material graph authoring,
- material inspection,
- material preview,
- texture/resource binding diagnostics,
- shader/product preview.

Planes:

```text
Identity:
  runenwerk.material_lab.*

Capabilities:
  material graph editing
  material product formation
  texture catalog read
  render preview request

Domain truth:
  domain/material_graph
  domain/texture

Product graph:
  material graph source -> material product -> shader/preview product

Execution:
  in-process preview now
  future material compile service
```

### 10.2 Texture Lab Cell

Purpose:

- texture inspection,
- generated texture diagnostics,
- volume texture viewing,
- texture preview products,
- material texture handoff.

Planes:

```text
Identity:
  runenwerk.texture.*

Capabilities:
  texture product inspection
  generated texture preview
  texture upload/product formation

Domain truth:
  domain/texture

Product graph:
  texture source/descriptor -> texture product -> upload/preview artifact
```

### 10.3 Procgen Lab Cell

Purpose:

- procedural graph editing,
- seed/config controls,
- generated world previews,
- explanation/diagnostic views,
- service-ready evaluation.

Planes:

```text
Identity:
  runenwerk.procgen.*

Capabilities:
  procgen graph editing
  generated product formation
  world preview request

Execution:
  local job graph first
  future procgen evaluation service
```

### 10.4 Diagnostics Cell

Purpose:

- inspect suites,
- inspect providers,
- inspect capabilities,
- inspect product graph status,
- inspect policy decisions,
- inspect service/component health.

The diagnostics cell should evolve from registry inspection toward full Capability Workbench inspection.

## 11. Versioning And Compatibility

Every cross-layer contract needs a versioning story.

Versioned contracts:

```text
manifest schema version
capability schema version
command payload version
product descriptor version
service protocol version
component interface version
host policy version
```

Rules:

- version changes must be explicit,
- migrations must be testable,
- deprecated capabilities must have replacement metadata,
- old manifests must fail clearly or migrate safely,
- service protocol mismatches must fail closed,
- external components must declare compatible interface versions.

## 12. Anti-Patterns

Do not:

- make manifests executable,
- grant capabilities by installation alone,
- put host policy into domains,
- put domain validation into providers,
- let services mutate source truth,
- let external components use privileged side APIs,
- let AI/automation bypass command proposals,
- model every tiny helper as a capability,
- check capabilities in per-frame hot loops,
- implement external components before policy exists,
- implement dynamic plugins before explicit capability policy and sandboxing exist,
- create separate architectures for full editor, standalone editor, in-game editor, and headless host.

## 13. When Not To Use This Architecture

Do not use the full Capability Workbench architecture for:

- tiny internal debug helpers,
- local UI-only callbacks,
- pure view-model transformations,
- temporary prototypes,
- engine hot loops,
- pure renderer internals,
- styling/layout-only tweaks,
- throwaway experiments.

Use it when a tool crosses boundaries:

- persistent identity,
- domain mutation,
- product formation,
- resource access,
- runtime/preview access,
- host-specific permission,
- service execution,
- external component execution,
- in-game/editor/headless host differences.

## 14. Non-Goals

This target architecture does not:

- implement maturity layers beyond the current foundation,
- accept dynamic external plugins immediately,
- define a plugin marketplace,
- choose a sandbox runtime prematurely,
- remove domain-owned commands,
- move app workflow into editor_shell,
- move renderer ownership into domains,
- allow providers to mutate source truth directly,
- make AI/automation a privileged bypass,
- replace the WorkbenchHost / ToolSuite foundation.

## 15. Acceptance Criteria

This target architecture is acceptable when it provides:

- a clear Capability Cell model,
- clear Platform Planes,
- clear Maturity Layers,
- stable identity throughout,
- strict domain/app/provider/host/render ownership separation,
- capability vs policy vs domain validation distinction,
- host types and trust zones,
- versioning strategy,
- promotion path,
- anti-patterns,
- dynamic plugins deferred,
- current implementation work not blocked by this design.

## 16. Summary

The most elegant long-term architecture for Runenwerk is:

```text
Capability Cells governed by Platform Planes,
delivered through Maturity Layers.
```

In short:

```text
Tool unit:
  Capability Cell

Governance:
  Identity Plane
  Capability Plane
  Host Policy Plane
  Domain Truth Plane
  Product Graph Plane
  Reconciliation / Status Plane
  Execution Plane

Maturity:
  Internal ToolSuites
  Declarative Suite Manifests
  Capability Contracts
  Product / Command / Permission Registries
  Tool Execution Protocol
  External Sandboxed Components
```

This gives Runenwerk a durable architecture for full editor tools, standalone focused editors, in-game editors, headless validation, service-backed heavy tools, AI-assisted workflows, and eventual sandboxed external components without sacrificing authority, testability, or long-term maintainability.
