---
title: Production Track Index
description: Generated index of long-term production tracks and their milestone states.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-21
related:
  - ./production-track-planning-model.md
  - ./production-tracks.yaml
  - ./production-milestone-register.md
  - ./roadmap-items.yaml
  - ./roadmap-decision-register.md
  - ./schemas/production-tracks.schema.json
  - ./diagrams/production-track-roadmap.puml
---

# Production Track Index

This page is generated from [production-tracks.yaml](./production-tracks.yaml).
Do not edit it directly; update the YAML source and run `task production:render`.

Production tracks guide long-term sequencing. The WR roadmap remains the
dependency-checked execution graph.

## Tracks

| ID | Track | State | Owner | Strategic goal | Success criteria |
|---|---|---|---|---|---|
| PT-SDF-OW | SDF-first open-world playable vertical | active | workspace | Prove the SDF-first field-world architecture through a playable, visible, inspectable open world. | Player movement, world rendering, strict query products, diagnostics, and content products are integrated through production contracts.<br>Deferred world capabilities move through design gates before implementation, not through one-off prototype shortcuts.<br>The track remains extensible for caves, multiplayer, richer simulation, advanced VFX, and gameplay systems. |
| PT-ECS-FABRIC | ECS Execution Fabric Platform | active | ecs | Make ECS, scheduler planning, and runtime product jobs deterministic, inspectable, ergonomic, and ready for future parallel execution without moving worker-thread ownership into domain crates. | ECS APIs make live state, systems, deferred commands, queries, messaging, snapshots, and runtime plans easy to inspect and use correctly.<br>Scheduler planning exposes deterministic phases, waves, barriers, conflicts, and diagnostics through non-panicking APIs suitable for tools and production debugging.<br>Runtime jobs remain the active multithreaded path, with serial fallback, stale suppression, panic capture, backpressure diagnostics, and barrier-based product/query publication.<br>Public ECS parallel execution is introduced only after accepted design, deterministic command merge, blocked-parallel diagnostics, and serial/parallel equivalence tests. |
| PT-WB-CAP | Capability Workbench Platform | active | editor | Replace legacy Workbench tool-surface compatibility with a registry-owned capability platform that can host the full editor, standalone Material Lab, constrained hosts, and headless validation through one typed composition model. | Workbench identity, profile construction, provider requests, and persistence use typed suite/profile/provider declarations and stable surface keys only.<br>Material Lab mounts in full-editor and standalone hosts without legacy tool-surface metadata.<br>Host command, product, and resource policy is enforced before provider proposals mutate app or domain state.<br>External dynamic components remain blocked until sandbox and security design is accepted. |
| PT-RENDER-PG | Render Product Graph Platform | completed | engine | Make rendering a product-driven, inspectable, graph-compiled platform without moving product truth into the renderer. | Domains and Product Jobs own product truth, lineage, freshness, authority class, fallback legality, rebuild policy, residency intent, and diagnostics.<br>The Render Execution Graph Compiler consumes prepared render product selections and feature-owned render fragments only.<br>The backend runtime owns derived GPU execution state only: WGPU allocation, command encoding, pipelines, bind groups, uploads, captures, timing, and presentation.<br>Product surfaces, render fragments, diagnostics, multi-surface presentation, and future product families share one contract path without renderer-owned semantic shortcuts. |
| PT-RENDER-GPU | Renderer GPU Evidence And Procedural Visuals Platform | active | engine | Add runtime GPU evidence, render-flow shape guards, hybrid procedural visual APIs, and canonical boids proof without moving product truth or product policy into the renderer. | GPU pass timing distinguishes CPU encode/submit work from GPU execution cost and exposes unsupported timing diagnostics when backend capabilities are absent.<br>Render-flow validation and prepared-frame preflight diagnose dangerous pass-shape and instance-count combinations before they can become runtime stutter.<br>Procedural instance APIs cover mesh/quad sprites, local SDF impostors, shared storage-backed instance buffers, and explicit blend/depth/cull/primitive policy.<br>The canonical boids example uses storage-backed compute simulation plus bounded local per-boid mesh/SDF sprite rendering, with no fullscreen-per-boid rendering.<br>Runtime inspection, docs, benchmarks, examples, and closeout evidence support a runtime_proven production quality target.<br>Product truth, product selection, freshness, authority, fallback legality, rebuild policy, residency policy, field/VFX emitters, and gameplay particle semantics stay outside the renderer. |
| PT-UI-DESIGN | UI Designer And Interface Lab Platform | completed | editor | Make UI/interface authoring a generic, definition-driven, target-profile-aware Designer/Lab platform for editor/workbench UI and game-runtime UI without moving domain semantics into the Designer. | Designer documents remain source truth only for UI/interface definitions.<br>Editor/workbench and game-runtime targets project from shared Canonical UI IR through explicit target profiles.<br>Runtime projections are reproducible from authored definitions, target profile, policy, fixtures, and validated composition.<br>Visual editing round-trips through Canonical UI IR with stable ids and reviewable textual diffs.<br>Preview, diagnostics, migration, accessibility, compatibility, performance, and golden evidence are first-class. |

## Current Milestone States

### PT-SDF-OW - SDF-first open-world playable vertical

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-SDF-OW-001 | Production product spine | implementation | completed | WR-019, WR-026, WR-021 | Field visualization, source-backed asset adapters, material preview products, and renderer handoff were planned and executed through WR roadmap rows. |
| PM-SDF-OW-002 | Open world substrate design | design | active | WR-001, WR-014, WR-015 | Owning designs identify product formats, mutation paths, residency, strict query behavior, and diagnostics before implementation. |
| PM-SDF-OW-003 | Playable SDF character | design | designing | WR-014, WR-015, WR-022 | Character body, pose, motion, collision, render, interaction emitters, and diagnostics have accepted ownership and product boundaries. |
| PM-SDF-OW-004 | Atmosphere and material response | design | designing | WR-014, WR-015 | Day/night becomes a world/render product family, not a renderer color shortcut. |
| PM-SDF-OW-005 | Vegetation field interaction | design | designing | WR-014, WR-015 | Vegetation is planned as deterministic field products rather than per-blade authored state. |
| PM-SDF-OW-006 | SDF prefab production set | design | designing | WR-022 | Prefabs become reusable field compositions with product outputs rather than mesh-centric object bundles. |
| PM-SDF-OW-007 | Water and wetness fields | design | designing | WR-014, WR-015 | Water is a field product family with explicit interaction and render handoff. |
| PM-SDF-OW-008 | Enemy and influence AI proof | design | designing | WR-011, WR-014, WR-015, WR-022 | Enemy behavior uses explicit gameplay/influence contracts without requiring the full gameplay graph first. |
| PM-SDF-OW-009 | Production hardening and playable evidence | hardening | designing | WR-018, WR-019, WR-021, WR-022 | The first production vertical is complete only when the combined acceptance criteria are observed and documented. |

### PT-ECS-FABRIC - ECS Execution Fabric Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-ECS-FABRIC-001 | Audit and Product Baseline | design | completed | WR-002, WR-023 | The ECS execution-fabric audit identifies ownership, current implementation state, friction, gaps, future features, redesign candidates, and production-track sequencing. |
| PM-ECS-FABRIC-002 | Runtime Convergence and Diagnostics | implementation | active | WR-002 | Runtime lifecycle finalization, plan inspection, conflict diagnostics, and docs evidence are current, non-panicking, and usable from tooling. |
| PM-ECS-FABRIC-003 | Runtime Product Job Substrate Hardening | implementation | designing | WR-001, WR-002 | Runtime product jobs are the obvious worker-backed path for product computation, with clear fallbacks and diagnostics. |
| PM-ECS-FABRIC-004 | Scheduler Plan Ergonomics | implementation | designing | WR-002, WR-023 | Scheduler plans are easier to inspect, safer to compose, and ready for richer product/job scheduling diagnostics. |
| PM-ECS-FABRIC-005 | ECS Parallel Execution Readiness | design | designing | WR-023 | Parallel ECS has accepted ownership, API, safety, command-merge, diagnostics, fallback, and validation contracts before code changes make it public. |
| PM-ECS-FABRIC-006 | ECS Parallel Execution Implementation | implementation | blocked | WR-023 | ECS waves can run in parallel where safe, while serial execution remains permanent and behavior-equivalent. |

### PT-WB-CAP - Capability Workbench Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-WB-CAP-001 | Clean Registry-Owned Workbench Foundation | implementation | completed | WR-031, WR-032, WR-033, WR-034, WR-035, WR-036 | Workbench state, profiles, providers, persistence, and Material Lab routes are stable-key-only with no compatibility enum or V5 legacy fallback metadata. |
| PM-WB-CAP-002 | Host Capability Policy | implementation | completed | WR-037 | Provider proposals pass through host policy before app or domain mutation. |
| PM-WB-CAP-003 | Product And Service Capability Plane | design | completed | WR-038 | Suites declare product and service needs while domains keep semantic validation authority. |
| PM-WB-CAP-004 | Multi-Host Workbench Modes | implementation | completed | WR-039 | Hosts differ by suite/profile/provider bundle and policy, not by forked app-specific compatibility paths. |
| PM-WB-CAP-005 | External Component Readiness | design | blocked | WR-040 | Future external component work has a design-only row and cannot bypass host policy. |

### PT-RENDER-PG - Render Product Graph Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-PG-001 | Render Product Graph Doctrine And Boundary Ratification | design | completed | WR-003 | The accepted design, roadmap mapping, and render docs define a product-first render platform without changing WR execution legality. |
| PM-RENDER-PG-002 | Render Contract Ergonomics | implementation | completed | WR-041 | Product-surface and render-flow authoring have focused examples, diagnostics, and contract tests. |
| PM-RENDER-PG-003 | Feature-Owned Render Contributions | implementation | completed | WR-003, WR-042 | Feature-owned collectors are typed, inspectable, capability-declared, diagnostic-producing, and validated before submit. |
| PM-RENDER-PG-004 | Render Execution Graph Compiler Maturity | implementation | completed | WR-003, WR-043 | The compiler validates render resources, pass order, target aliases, history scope, resource lifetimes, and backend capability constraints. |
| PM-RENDER-PG-005 | Product Surface Platform Hardening | implementation | completed | WR-003, WR-044 | Product-surface producers share the same dynamic target, prepared view, invocation, history, UI sampling, and diagnostic contracts. |
| PM-RENDER-PG-006 | Multi-Surface Presentation | implementation | completed | WR-009 | Render frames are surface-scoped and submit/present cannot cross native surfaces. |
| PM-RENDER-PG-007 | Render Fragments And Hot Reload | implementation | completed | WR-010 | Fragment-driven flows run through normal RenderFlow validation and compiled execution. |
| PM-RENDER-PG-008 | Production Readiness And Inspection | hardening | completed | WR-045, WR-003, WR-009, WR-010 | Complex renderer behavior is inspectable and validated enough for production product teams. |

### PT-RENDER-GPU - Renderer GPU Evidence And Procedural Visuals Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-GPU-001 | GPU Evidence And Procedural Visuals Doctrine | design | active | N/A | The active design and production track identify ownership, sequence, gates, non-goals, open decisions, and runtime_proven target evidence. |
| PM-RENDER-GPU-002 | GPU Pass Timing Foundation | implementation | designing | WR-056 | Renderer inspection reports GPU pass cost when timestamp queries are supported and explicit unsupported diagnostics when they are not. |
| PM-RENDER-GPU-003 | Render-Flow Pass-Shape And Instance Guards | implementation | designing | WR-057 | Fullscreen-style rendering multiplied by instance count is diagnosed before submit unless an accepted explicit opt-in path exists. |
| PM-RENDER-GPU-004 | Hybrid Procedural Instance Rendering API | implementation | designing | WR-058 | Renderer users can build bounded local procedural visuals without renderer-private handles or unsafe pass-shape conventions. |
| PM-RENDER-GPU-005 | Canonical Boids Hybrid Procedural Rewrite | implementation | designing | WR-059 | Boids compute simulation remains storage-backed while rendering uses bounded local per-boid mesh/SDF sprite work with no fullscreen-per-boid pass. |
| PM-RENDER-GPU-006 | Procedural Visuals Production Readiness | hardening | designing | WR-060 | The track can claim runtime_proven only after completed evidence demonstrates GPU timing, guards, procedural APIs, and canonical boids behavior. |

### PT-UI-DESIGN - UI Designer And Interface Lab Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-UI-DESIGN-001 | UI Designer Doctrine And Target Boundary Ratification | design | completed | WR-046 | The active design defines source truth, domain boundaries, target profiles, and planning-only constraints without changing runtime behavior. |
| PM-UI-DESIGN-002 | Canonical UI IR And Composition Pipeline | design | completed | N/A | Visual and textual authoring converge through one typed, inspectable, versioned UI definition pipeline. |
| PM-UI-DESIGN-003 | Target Projection Profiles | design | completed | N/A | Editor Workbench Projection and Game Runtime UI Projection consume shared Canonical UI IR through explicit target-profile rules. |
| PM-UI-DESIGN-004 | Visual Layout And Interface Composition | implementation | completed | WR-047 | Designers can author layouts visually while preserving typed definitions, stable ids, and reviewable textual diffs. |
| PM-UI-DESIGN-005 | Theme Tokens Modes Skins And State Variants | implementation | completed | WR-049 | Style resolution is reproducible, inspectable, target-profile-aware, and guarded by typed token/theme diagnostics. |
| PM-UI-DESIGN-006 | Component Surface And Widget Recipe Library | implementation | completed | WR-050 | Editor/workbench and game-runtime UI reuse common recipes without sharing domain semantics or direct mutation authority. |
| PM-UI-DESIGN-007 | View-Model Capability And Intent Binding | implementation | completed | WR-051 | UI definitions can display domain-owned state and emit validated intents while domains retain semantic authority. |
| PM-UI-DESIGN-008 | Live Preview Fixtures Scenarios And Target Matrix | implementation | completed | WR-052 | Designer previews prove empty, loading, error, denied, offline, heavy, accessibility, performance, and interaction behavior across targets. |
| PM-UI-DESIGN-009 | Persistence Migration Diff And Activation | implementation | completed | WR-053 | UI definition changes are reviewable, migratable, dry-runnable, and fail closed before runtime activation. |
| PM-UI-DESIGN-010 | Production Readiness And Evidence | hardening | completed | WR-054 | UI/interface authoring is inspectable and validated enough for production editor/workbench and game-runtime UI work. |
