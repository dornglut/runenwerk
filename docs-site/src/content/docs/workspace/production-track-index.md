---
title: Production Track Index
description: Generated index of long-term production tracks and their milestone states.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-24
related:
  - ./production-track-planning-model.md
  - ./production-tracks.yaml
  - ./production-milestone-register.md
  - ./roadmap-items.yaml
  - ./roadmap-decision-register.md
  - ./schemas/production-tracks.schema.json
  - ./diagrams/production-track-roadmap.puml
  - ./diagrams/production-track-full-roadmap.puml
---

# Production Track Index

This page is generated from [production-tracks.yaml](./production-tracks.yaml).
Do not edit it directly; update the YAML source and run `task production:render`.

Production tracks guide long-term sequencing. The WR roadmap remains the
dependency-checked execution graph.

## Tracks

| ID | Track | State | Owner | Target quality | Strategic goal | Success criteria |
|---|---|---|---|---|---|---|
| PT-SDF-OW | SDF-first open-world playable vertical | active | workspace | not_applicable | Prove the SDF-first field-world architecture through a playable, visible, inspectable open world. | Player movement, world rendering, strict query products, diagnostics, and content products are integrated through production contracts.<br>Deferred world capabilities move through design gates before implementation, not through one-off prototype shortcuts.<br>The track remains extensible for caves, multiplayer, richer simulation, advanced VFX, and gameplay systems. |
| PT-ECS-FABRIC | ECS Execution Fabric Platform | active | ecs | not_applicable | Make ECS, scheduler planning, and runtime product jobs deterministic, inspectable, ergonomic, and ready for future parallel execution without moving worker-thread ownership into domain crates. | ECS APIs make live state, systems, deferred commands, queries, messaging, snapshots, and runtime plans easy to inspect and use correctly.<br>Scheduler planning exposes deterministic phases, waves, barriers, conflicts, and diagnostics through non-panicking APIs suitable for tools and production debugging.<br>Runtime jobs remain the active multithreaded path, with serial fallback, stale suppression, panic capture, backpressure diagnostics, and barrier-based product/query publication.<br>Public ECS parallel execution is introduced only after accepted design, deterministic command merge, blocked-parallel diagnostics, and serial/parallel equivalence tests. |
| PT-WB-CAP | Capability Workbench Platform | active | editor | not_applicable | Replace legacy Workbench tool-surface compatibility with a registry-owned capability platform that can host the full editor, standalone Material Lab, constrained hosts, and headless validation through one typed composition model. | Workbench identity, profile construction, provider requests, and persistence use typed suite/profile/provider declarations and stable surface keys only.<br>Material Lab mounts in full-editor and standalone hosts without legacy tool-surface metadata.<br>Host command, product, and resource policy is enforced before provider proposals mutate app or domain state.<br>External dynamic components remain blocked until sandbox and security design is accepted. |
| PT-RENDER-PG | Render Product Graph Platform | completed | engine | not_applicable | Make rendering a product-driven, inspectable, graph-compiled platform without moving product truth into the renderer. | Domains and Product Jobs own product truth, lineage, freshness, authority class, fallback legality, rebuild policy, residency intent, and diagnostics.<br>The Render Execution Graph Compiler consumes prepared render product selections and feature-owned render fragments only.<br>The backend runtime owns derived GPU execution state only: WGPU allocation, command encoding, pipelines, bind groups, uploads, captures, timing, and presentation.<br>Product surfaces, render fragments, diagnostics, multi-surface presentation, and future product families share one contract path without renderer-owned semantic shortcuts. |
| PT-RENDER-GPU | Renderer GPU Evidence And Procedural Visuals Platform | completed | engine | runtime_proven | Add runtime GPU evidence, render-flow shape guards, hybrid procedural visual APIs, and canonical boids proof without moving product truth or product policy into the renderer. | GPU pass timing distinguishes CPU encode/submit work from GPU execution cost and exposes unsupported timing diagnostics when backend capabilities are absent.<br>Render-flow validation and prepared-frame preflight diagnose dangerous pass-shape and instance-count combinations before they can become runtime stutter.<br>Procedural instance APIs cover mesh/quad sprites, local SDF impostors, shared storage-backed instance buffers, and explicit blend/depth/cull/primitive policy.<br>The canonical boids example uses storage-backed compute simulation plus bounded local per-boid mesh/SDF sprite rendering, with no fullscreen-per-boid rendering.<br>Runtime inspection, docs, benchmarks, examples, and closeout evidence support a runtime_proven production quality target.<br>Product truth, product selection, freshness, authority, fallback legality, rebuild policy, residency policy, field/VFX emitters, and gameplay particle semantics stay outside the renderer. |
| PT-RENDER-SCALE | Renderer Scale Residency And GPU Driven Visibility Platform | completed | engine | runtime_proven | Make renderer scale explicit through finite resident, visible, and submitted working sets for millions-scale content without per-entity CPU submission or renderer-owned product truth. | Renderer-facing chunk, page, cluster, and instance registries define bounded working sets derived from product selections.<br>GPU memory budgets, upload budgets, dynamic target pressure, and residency pressure are inspectable.<br>GPU-driven culling, LOD, visible-list compaction, and indirect draw or dispatch generation bound submitted work.<br>Scale evidence distinguishes addressable records, resident GPU records, visible candidates, submitted commands, and actual frame cost.<br>Product truth, streaming policy, freshness, fallback legality, authority, rebuild policy, and residency intent stay outside the renderer. |
| PT-RENDER-PROCEDURAL-POPULATION | Renderer Procedural Population Platform | completed | engine | runtime_proven | Provide reusable renderer infrastructure for large GPU procedural populations, proven by an aspect-correct, visually stable, grid-accelerated, evidence-backed boids example. | Procedural-owned authoring supports uniforms, surface-aware uniforms, and indirect draws without exposing GraphicsPassBuilder.<br>Render graph draw sources distinguish direct and indirect submission while preserving existing direct draw authoring.<br>GPU scan, reset, scatter/compaction, and indirect-args primitives are reusable outside boids.<br>Bounded uniform-grid population support replaces O(n^2) fixed-radius neighbor loops for canonical boids.<br>Boids evidence reports fixed-step limitations, aspect-correct impostors, stable visual heading, bounded work, unsupported diagnostics, and benchmark commands.<br>Spatial hash and chunked unbounded population support remain a later milestone after bounded evidence passes. |
| PT-RENDER-PROCEDURAL-POPULATION-HARDENING | Renderer Procedural Population Hardening Platform | completed | engine | runtime_proven | Close the direct procedural population runtime gaps with fail-closed indirect draw validation, reusable renderer-dispatched GPU primitives, graph-level fixed-step catch-up scheduling, and reusable procedural camera projection. | Indirect draw sources distinguish direct, indexed direct, indirect, and indexed indirect submission with typed fail-closed validation.<br>Indirect argument buffer type, element count, byte size, byte-offset alignment, and byte-offset bounds are validated before submit.<br>GPU primitive plans lower into reusable renderer-owned shader dispatches rather than descriptor-only contracts.<br>U32 prefix scan supports arbitrary total counts through hierarchical block scan, block-sum scan, and block-offset propagation.<br>Fixed-step catch-up is graph-level bounded repeated pass execution from runtime fixed-time resources, not boids-local timing logic.<br>Cursor movement, mouse motion, redraw bursts, and resize events do not increase submitted simulation steps per real second.<br>Procedural camera projection fills the viewport without letterbox or non-uniform stretch while preserving equal world x/y scale.<br>Spatial hash and chunked unbounded populations remain a separate intake/design item.<br>Richer flock split/merge behavior remains a separate behavior-authoring intake/design item.<br>The track closes at runtime_proven only and does not claim perfectionist_verified. |
| PT-RENDER-SDF | Sparse SDF World Rendering And Raymarch Acceleration Platform | completed | engine | runtime_proven | Build sparse SDF world rendering on product selection and derived GPU residency with conservative raymarch acceleration and visible diagnostics. | Sparse SDF bricks, page tables, clipmaps, analytic instances, cluster fields, and aggregate fields are derived renderer representations.<br>Distance mips, empty-space skipping, screen-tile/depth-slice candidate lists, and temporal raymarch caches are conservative and inspectable.<br>Fullscreen raymarching is bounded per view and never multiplied by per-entity instance counts.<br>SDF runtime evidence exposes step counts, missed-surface risk, overstep risk, cache pressure, residency pressure, and GPU pass cost. |
| PT-RENDER-SDF-RUNTIME | Shader-Bound Sparse SDF Terrain Runtime Platform | active | engine | runtime_proven | Turn completed SDF residency and raymarch inspection evidence into a reusable shader-bound sparse SDF terrain runtime path without making renderer code own SDF product truth. | Sparse SDF terrain rendering consumes renderer-owned page table, brick atlas, distance mip, and candidate-list GPU resources from shader bindings.<br>Fullscreen terrain raymarching remains one bounded pass per prepared view and is never multiplied by chunks, entities, or source payload counts.<br>Renderer diagnostics fail closed for missing residency, stale generations, unsupported limits, candidate explosion, unsafe overstep, over-budget residency, and empty candidate coverage.<br>Camera-relative world framing keeps endless-world coordinates independent from fragile absolute f32 positions.<br>The existing procedural sky SDF terrain example remains an analytic visual demo; production proof uses a dedicated sparse SDF runtime example. |
| PT-RENDER-MESH-MATERIAL | Mesh Material Lighting Shader And Asset Handoff Platform | completed | engine | runtime_proven | Connect mesh, material, lighting, shader, and asset handoff work to renderer execution without moving asset, material, model, or scene truth into the renderer. | Mesh/model/skinning/deformation render contributions are prepared data, not live ECS extraction.<br>Material graph lowering, shader specialization, pipeline cache policy, and last-good shader fallback are validated and inspectable.<br>Material and mesh previews route through product surfaces and share renderer contracts with SDF and procedural paths.<br>Asset cooking hooks feed renderer artifacts without making renderer state canonical. |
| PT-RENDER-TEMPORAL | Temporal Reconstruction Dynamic Resolution And Upscaling Platform | completed | engine | runtime_proven | Build portable temporal reconstruction and dynamic resolution before optional FSR-style or vendor-specific upscaling adapters. | TAA/TAAU, jitter, history validity, motion vectors, depth, exposure, and reactive-mask style inputs are explicit renderer contracts.<br>Dynamic internal resolution is separate from output resolution and visible in diagnostics.<br>Raymarch and ray-query reconstruction inputs are supported by prepared products and history invalidation.<br>FSR-style adapters are optional capability paths with unsupported diagnostics, not the baseline renderer. |
| PT-RENDER-RT | Hardware Ray Query And Hybrid Tracing Platform | completed | engine | runtime_proven | Add optional capability-gated hardware ray-query and hybrid tracing paths without making RT hardware a baseline requirement. | Ray-query and acceleration-resource support is feature-detected and reports explicit unsupported diagnostics.<br>Derived acceleration resources preserve product, mesh, material, and SDF ownership boundaries.<br>Hybrid raster, SDF raymarch, and ray-query paths share timing, reconstruction, and inspection evidence.<br>Non-RT fallback is mandatory and production-valid. |
| PT-RENDER-PRODUCT-VISUALS | Product Visual Producers Platform | completed | engine | runtime_proven | Integrate product-owned particles, VFX, vegetation, water, atmosphere, weather, trails, decals, and animation producers with renderer execution contracts. | Product domains emit prepared render contributions and residency requests without moving semantic truth into the renderer.<br>Particles, VFX, trails, decals, vegetation, grass, water, atmosphere, weather, and animation/deformation visuals use shared renderer contracts.<br>Scale, SDF, temporal, and mesh/material capabilities are consumed when available and diagnosed when missing.<br>Missing, stale, fallback, over-budget, and unsupported product visual states remain visible. |
| PT-RENDER-PERFECTION | Renderer Production Audit And Perfectionist Verification | active | workspace | perfectionist_verified | Verify the complete renderer production stack after runtime-proven tracks close, with no known quality gaps or ownership leaks. | Cross-track evidence matrix covers GPU, scale, SDF, mesh/material, temporal, RT, and product visual tracks.<br>Docs, public APIs, examples, benchmarks, and inspection DTOs agree.<br>Hardware evidence is explicit and unsupported states are documented.<br>No renderer track moves product truth, freshness, fallback legality, authority, rebuild policy, or residency policy across ownership boundaries.<br>Final completion can claim perfectionist_verified only when known quality gaps are empty. |
| PT-VIEWPORT-PROJECTION | Viewport Camera And Projection Contract Platform | active | workspace | runtime_proven | Establish durable camera, projection, viewport presentation, and surface-fit contracts across renderer, editor viewport, app adapter, examples, and UI embedding without moving source truth into the wrong owner. | Producer and editor viewport contexts own camera intent while renderer code owns only derived projection and presentation data.<br>PreparedViewFrame remains camera-free and continues to carry view identity, target size, history identity, and render preparation data only.<br>UI primitives remain camera-free and embed typed product or viewport surfaces rather than owning projection semantics.<br>Editor viewport CPU picking and GPU projection behavior have one accepted contract and drift-guard evidence.<br>Examples and docs prove aspect, surface-fit, and product-surface behavior without boids-only or editor-only shortcuts. |
| PT-UI-DESIGN | UI Designer And Interface Lab Platform | completed | editor | not_applicable | Make UI/interface authoring a generic, definition-driven, target-profile-aware Designer/Lab platform for editor/workbench UI and game-runtime UI without moving domain semantics into the Designer. | Designer documents remain source truth only for UI/interface definitions.<br>Editor/workbench and game-runtime targets project from shared Canonical UI IR through explicit target profiles.<br>Runtime projections are reproducible from authored definitions, target profile, policy, fixtures, and validated composition.<br>Visual editing round-trips through Canonical UI IR with stable ids and reviewable textual diffs.<br>Preview, diagnostics, migration, accessibility, compatibility, performance, and golden evidence are first-class. |
| PT-UI-LAB | Runtime-Proven Editor Interface Lab Productization | completed | editor | runtime_proven | Turn completed UI Designer contracts into an app-hosted Editor Interface Lab with visual authoring, project IO, diff/apply, preview evidence, and public API ergonomics without moving domain semantics into UI definitions. | PT-UI-DESIGN remains completed design-contract input and is not reopened to justify runtime claims.<br>Editor Lab V1 is app-hosted, usable through direct controls, and proves real editor runtime behavior with captured evidence.<br>Commands, surface metadata, project IO, diagnostics, and activation reports have one typed source of truth at their owning boundaries.<br>Generic UI definition logic remains behavior-free while editor-specific execution, project IO, and provider behavior stay in editor/app-owned modules.<br>Runtime-proven closeout lists remaining gaps truthfully and defers perfectionist verification to a separate no-gap audit track. |
| PT-UI-LAB-PERFECTION | Editor Lab V1 Perfectionist No-Gap Certification | completed | editor | perfectionist_verified | Certify the completed Editor Lab V1 productization with zero known quality gaps, runtime evidence, coherent public APIs, and clean ownership boundaries without expanding scope into game-runtime UI projection. | PT-UI-LAB remains completed runtime_proven input and is not reopened to justify perfectionist claims.<br>Editor Lab V1 has native or explicitly platform-impossible runtime evidence for visual truth, focus, contrast, timing, diagnostics, degraded-provider behavior, reload, apply, rollback, and failure preservation.<br>Editor command, surface, operation, persistence, diff/apply, and public API paths have one normal source of truth at their owning boundaries.<br>Generic UI definition logic remains behavior-free; editor/game/app execution stays in editor-owned or app-owned domains.<br>Public APIs, preludes, guides, examples, runtime artifacts, generated planning docs, and closeout reports agree.<br>Final completion can claim perfectionist_verified only when known quality gaps are empty and phase drift-check evidence is complete. |
| PT-EDITOR-UX | Editor Product UX Native Story Lab And Surface Perfection | active | editor | perfectionist_verified | Make the editor product UI certifiable through a native Story Lab, all-surface readiness policy, reusable design-system layers, graph/node-editor productization, standalone UI Designer workbench proof, and local-native no-gap evidence while preserving future game UI target-profile compatibility. | PT-EDITOR-UX is a new product UX track and does not reopen PT-UI-LAB or implement PT-GAME-RUNTIME-UI.<br>Generic UI truth remains in domain/ui, editor product semantics remain in domain/editor, and native evidence execution remains in apps/runenwerk_editor.<br>Every registered visible user-facing editor surface and explicit diagnostic/fallback surface is certified, fallback-only, diagnostic, or hidden until productized.<br>Every certified primitive widget, product widget pattern, registered surface, and host scenario has Story Lab coverage, visible-widget scan evidence, and state matrix evidence.<br>Final certification requires local-native screenshots where supported, accessibility and interaction reports, performance evidence, and zero hard-budget UI gaps.<br>Future game-runtime UI readiness is proven only through target-profile compatibility seams and never through editor-shell dependency. |
| PT-GAME-RUNTIME-UI | Game Runtime UI Projection And HUD Platform | active | workspace | runtime_proven | Turn completed UI Designer contracts into a runtime-proven game UI projection and HUD platform with explicit ownership, fail-closed diagnostics, engine-owned UI expression submission, and SDF screen-HUD proof without moving gameplay or render truth into UI. | PT-UI-DESIGN remains completed design-contract input and is not reopened to justify runtime claims.<br>Game-runtime UI projection consumes shared Canonical UI IR and game-runtime target policy without depending on editor shell ownership.<br>UI definitions bind only to read-only game/runtime view-model packets and emit validated intent proposals.<br>Engine UI submission remains generic expression infrastructure; game HUD semantics stay with the accepted game-runtime UI owner or proof adapter.<br>SDF render-flow proof shows FPS/status and tab controls rendered in-frame through UI composition, not WindowState title text or debug overlay reuse.<br>Runtime-proven closeout lists known gaps truthfully and creates a separate perfectionist-audit intake before any no-gap claim. |

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
| PM-RENDER-GPU-001 | GPU Evidence And Procedural Visuals Doctrine | design | completed | WR-082 | The accepted design and production track identify ownership, sequence, gates, non-goals, acceptance decisions, and runtime_proven target evidence. |
| PM-RENDER-GPU-002 | GPU Pass Timing Foundation | implementation | completed | WR-056 | Renderer inspection reports GPU pass cost when timestamp queries are supported and explicit unsupported diagnostics when they are not. |
| PM-RENDER-GPU-003 | Render-Flow Pass-Shape And Instance Guards | implementation | completed | WR-057 | Fullscreen-style rendering multiplied by instance count is diagnosed before submit unless an accepted explicit opt-in path exists. |
| PM-RENDER-GPU-004 | Hybrid Procedural Instance Rendering API | implementation | completed | WR-058 | Renderer users can build bounded local procedural visuals without renderer-private handles or unsafe pass-shape conventions. |
| PM-RENDER-GPU-005 | Canonical Boids Hybrid Procedural Rewrite | implementation | completed | WR-059 | Boids compute simulation remains storage-backed while rendering uses bounded local per-boid mesh/SDF sprite work with no fullscreen-per-boid pass. |
| PM-RENDER-GPU-006 | Procedural Visuals Production Readiness | hardening | completed | WR-060 | The track can claim runtime_proven only after completed evidence demonstrates GPU timing, guards, procedural APIs, and canonical boids behavior. |

### PT-RENDER-SCALE - Renderer Scale Residency And GPU Driven Visibility Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-SCALE-001 | Scale Residency And Visibility Doctrine | design | completed | N/A | Accepted design records unbounded world/product space as finite renderer working sets with explicit evidence requirements. |
| PM-RENDER-SCALE-002 | Working Set Registry And Residency Budgets | implementation | completed | WR-061 | Renderer inspection can explain resident GPU working sets, budget pressure, eviction or downgrade needs, and product-lineage keys. |
| PM-RENDER-SCALE-003 | GPU Driven Culling LOD And Indirect Submission | implementation | completed | WR-062 | Large addressable populations produce bounded draw or dispatch work with visible diagnostics for culled, visible, and submitted counts. |
| PM-RENDER-SCALE-004 | Scale Evidence And Production Readiness | hardening | completed | WR-063 | Runtime evidence can support millions-scale renderer claims without hiding culled, degraded, unsupported, or over-budget states. |

### PT-RENDER-PROCEDURAL-POPULATION - Renderer Procedural Population Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-POP-001 | Procedural Population Doctrine | design | completed | WR-083 | Active design and production track define long-term renderer-owned population infrastructure, bounded first scope, and WR implementation slices. |
| PM-RENDER-POP-002 | Procedural Builder And Draw Sources | implementation | completed | WR-084 | Procedural authors can bind uniforms and surface-aware uniforms, and graphics execution can inspect typed draw sources. |
| PM-RENDER-POP-003 | GPU Primitive Contracts | implementation | completed | WR-085 | Population support can compose total-count-sized primitive buffers without silent fixed bucket overflow. |
| PM-RENDER-POP-004 | Bounded Uniform Grid Population | implementation | completed | WR-086 | Canonical procedural populations can build cell counts, prefix offsets, sorted indices, and adjacent-cell neighbor traversal. |
| PM-RENDER-POP-005 | Boids Production Upgrade | implementation | completed | WR-087 | Boids render correctly across resize, use fixed-step evidence, smooth visual heading separately from simulation velocity, and stop using the production O(n^2) neighbor loop. |
| PM-RENDER-POP-006 | Evidence Benchmarks And Docs | hardening | completed | WR-088 | Renderer users can understand and validate procedural population authoring, bounded grid behavior, fixed-step limits, and unsupported diagnostics. |

### PT-RENDER-PROCEDURAL-POPULATION-HARDENING - Renderer Procedural Population Hardening Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-POP-HARDEN-001 | Hardening Doctrine And Track Activation | design | completed | WR-089 | Active hardening design, production track metadata, roadmap rows, and implementation contracts define the long-term path without product code changes. |
| PM-RENDER-POP-HARDEN-002 | Indirect Draw Contract Hardening | hardening | completed | WR-090 | Render-flow validation rejects wrong indirect argument types and invalid byte offsets before execution, while direct draw authoring remains simple and source-compatible. |
| PM-RENDER-POP-HARDEN-003 | Reusable GPU Primitive Shader Dispatch | hardening | completed | WR-091 | Counter reset, u32 prefix scan, scatter/compaction, and indirect args generation execute through normal render-flow compute passes with typed diagnostics. |
| PM-RENDER-POP-HARDEN-004 | Fixed Step Graph Catch Up Scheduling | hardening | completed | WR-092 | Render-flow scheduling can submit 0..N bounded substeps deterministically from runtime fixed-time resources while preserving ping-pong, primitive resource sequencing, and iteration-scoped uniform projection. |
| PM-RENDER-POP-HARDEN-005 | Procedural Camera And View Projection | hardening | completed | WR-101 | Procedural examples can fill the target without letterbox or non-uniform stretch while keeping producer-owned camera intent separate from prepared view packets. |
| PM-RENDER-POP-HARDEN-006 | Hardening Evidence Benchmarks Docs And Closeout | hardening | completed | WR-093 | Renderer users can rely on documented fail-closed indirect draws, reusable primitive dispatch, graph catch-up scheduling, and procedural camera projection with runtime-proven evidence. |

### PT-RENDER-SDF - Sparse SDF World Rendering And Raymarch Acceleration Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-SDF-001 | Sparse SDF Rendering Doctrine | design | completed | N/A | Accepted design identifies renderer-owned SDF residency and acceleration separately from SDF product truth. |
| PM-RENDER-SDF-002 | SDF Brick Page And Clipmap Residency | implementation | completed | WR-064 | Renderer can inspect SDF resident pages, brick atlases, generation keys, cache pressure, and invalidation state. |
| PM-RENDER-SDF-003 | Raymarch Acceleration And Candidate Lists | implementation | completed | WR-065 | SDF raymarch flows avoid scanning all SDF sources per ray step and report unsafe-overstep or candidate explosion risks. |
| PM-RENDER-SDF-004 | SDF World Runtime Evidence | hardening | completed | WR-066 | SDF world rendering can claim runtime_proven with visible near/mid/far/summary behavior and diagnostic evidence. |

### PT-RENDER-SDF-RUNTIME - Shader-Bound Sparse SDF Terrain Runtime Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-SDF-RUNTIME-001 | Shader-Bound Sparse SDF Terrain Runtime Governance And Track Activation | design | active | WR-103 | Renderer SDF runtime work has an active design, applied WR-103 governance row, production milestones, and explicit non-goals before engine or shader code changes begin. |
| PM-RENDER-SDF-RUNTIME-002 | Sparse SDF GPU ABI And Runtime Bind Plan | implementation | designing | N/A | Render flows can bind derived page table, brick atlas, distance mip, candidate-list, generation, and camera-relative framing resources without moving SDF source truth into renderer state. |
| PM-RENDER-SDF-RUNTIME-003 | Shader-Bound Sparse Terrain Runtime Example | implementation | designing | N/A | A production-oriented example renders deterministic synthetic SDF terrain payloads through shader-bound page table, brick atlas, distance mip, and candidate-list resources. |
| PM-RENDER-SDF-RUNTIME-004 | Sparse SDF Runtime Evidence Benchmarks Docs And Closeout | hardening | designing | N/A | The follow-on runtime track can claim runtime_proven only with shader-bound evidence, performance diagnostics, docs, and explicit remaining quality gaps. |

### PT-RENDER-MESH-MATERIAL - Mesh Material Lighting Shader And Asset Handoff Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-MESH-MATERIAL-001 | Mesh Material Lighting Handoff Doctrine | design | completed | N/A | Accepted design records renderer responsibilities and asset/material/domain-owned truth boundaries. |
| PM-RENDER-MESH-MATERIAL-002 | Mesh Material Shader Asset Handoff | implementation | completed | WR-067 | Mesh/material previews and scene material paths produce visible pixels through prepared renderer contracts. |
| PM-RENDER-MESH-MATERIAL-003 | Lighting Pipeline Cache And Last Good Fallback | implementation | completed | WR-068 | Shader and pipeline failures are diagnosable and prior valid visual products can remain available when owning policy allows fallback. |
| PM-RENDER-MESH-MATERIAL-004 | Mesh Material Production Evidence | hardening | completed | WR-069 | Mesh/material renderer handoff can claim runtime_proven while known quality gaps stay visible. |

### PT-RENDER-TEMPORAL - Temporal Reconstruction Dynamic Resolution And Upscaling Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-TEMPORAL-001 | Temporal Reconstruction Doctrine | design | completed | N/A | Accepted design records temporal inputs, history validity, upscaling adapter policy, fallback requirements, and optional adapter boundaries. |
| PM-RENDER-TEMPORAL-002 | Temporal Inputs History And Dynamic Resolution | implementation | completed | WR-070 | Renderer inspection explains temporal input availability, history validity, reconstruction mode, and internal/output resolution. |
| PM-RENDER-TEMPORAL-003 | Upscaling Adapters And Ray Reconstruction Inputs | implementation | completed | WR-071 | Upscaling adapters can run when capabilities and inputs exist, and fail closed with diagnostics when they do not. |
| PM-RENDER-TEMPORAL-004 | Temporal Production Evidence | hardening | completed | WR-072 | Temporal reconstruction can claim runtime_proven with quality, timing, fallback, and history evidence. |

### PT-RENDER-RT - Hardware Ray Query And Hybrid Tracing Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-RT-001 | Hardware Ray Query Doctrine | design | completed | N/A | Accepted design records capability gating, fallback, and ownership boundaries for hardware ray-query work. |
| PM-RENDER-RT-002 | Ray Query Capability And Acceleration Resources | implementation | completed | WR-073 | Renderer can expose optional ray-query capability state and derived acceleration resources without leaking backend handles. |
| PM-RENDER-RT-003 | Hybrid Ray SDF Raster Runtime Proof | implementation | completed | WR-074 | Hybrid tracing can run when supported and produce equivalent or diagnosed fallback behavior when unsupported. |
| PM-RENDER-RT-004 | Ray Query Production Evidence | hardening | completed | WR-075 | RT support can claim runtime_proven only as an optional path with mandatory fallback. |

### PT-RENDER-PRODUCT-VISUALS - Product Visual Producers Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-PRODUCT-VISUALS-001 | Product Visual Producer Doctrine | design | completed | N/A | Active design separates renderer execution APIs from particles, VFX, vegetation, water, atmosphere, weather, and animation product truth. |
| PM-RENDER-PRODUCT-VISUALS-002 | Particles VFX Trails And Decals | implementation | completed | WR-076 | Particle-style product visuals can submit prepared contributions and residency requests through shared renderer APIs. |
| PM-RENDER-PRODUCT-VISUALS-003 | Vegetation Water Atmosphere Weather And Field Visuals | implementation | completed | WR-077 | World visual product families consume renderer scale, SDF, temporal, and product-surface capabilities without moving semantics into renderer code. |
| PM-RENDER-PRODUCT-VISUALS-004 | Animation Deformation And Product Visual Evidence | hardening | completed | WR-078 | Product visual producers can claim runtime_proven with examples, docs, benchmarks, and diagnostics across representative families. |

### PT-RENDER-PERFECTION - Renderer Production Audit And Perfectionist Verification

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-RENDER-PERFECTION-001 | Renderer Audit Doctrine | design | active | N/A | Active audit design defines what blocks perfectionist_verified across renderer production tracks. |
| PM-RENDER-PERFECTION-002 | Cross Track Evidence Matrix | hardening | designing | WR-079 | Reviewers can see which renderer features are proven, unsupported, deferred, or blocked. |
| PM-RENDER-PERFECTION-003 | Gap Closure And Consistency Audit | hardening | designing | WR-080 | Public docs, APIs, examples, diagnostics, and closeout evidence agree with no hidden renderer ownership leaks. |
| PM-RENDER-PERFECTION-004 | Perfectionist Verification Closeout | release | designing | WR-081 | Renderer production stack has a completed audit with empty known quality gaps and coherent runtime evidence. |

### PT-VIEWPORT-PROJECTION - Viewport Camera And Projection Contract Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-VIEWPORT-PROJECTION-001 | Contract Governance And Design Split | design | active | WR-106 | The viewport projection platform has an active design, deferred WR row, production track, boundary decisions, and explicit follow-on implementation slices without changing engine, editor, UI, shader, or example runtime behavior. |
| PM-VIEWPORT-PROJECTION-002 | Renderer Surface Fit And Procedural Projection Contracts | implementation | designing | N/A | Renderer examples and APIs use typed projection and presentation helpers for aspect and surface-fit behavior while preserving producer-owned camera intent. |
| PM-VIEWPORT-PROJECTION-003 | Editor Viewport Camera And Projection Hardening | implementation | designing | N/A | Editor viewport camera semantics, CPU picking, and GPU projection use an accepted editor-owned contract with drift-guard tests. |
| PM-VIEWPORT-PROJECTION-004 | Examples And Product Surface Evidence | hardening | designing | N/A | Boids, Game of Life, SDF-style examples, and product-surface embedding evidence demonstrate the preferred contracts without example-local projection shortcuts. |
| PM-VIEWPORT-PROJECTION-005 | Runtime Proven Closeout And Perfection Handoff | release | designing | N/A | The platform has closeout evidence, known quality gaps are explicit, and renderer perfection inputs are handed off without claiming game UI or boid behavior completion. |

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

### PT-UI-LAB - Runtime-Proven Editor Interface Lab Productization

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-UI-LAB-001 | Productization Governance And Code-Truth Reconciliation | design | completed | N/A | PT-UI-LAB has an active design, architecture-governance findings, disjoint WR candidate scopes, runtime evidence requirements, and explicit stop conditions for implementation. |
| PM-UI-LAB-002 | Registry And Command Source Of Truth | implementation | completed | WR-094 | Menu, toolbar, palette, keybinding, routing, command availability, disabled reasons, surface identity, capabilities, retention, and provider family projection derive from one owned source per boundary. |
| PM-UI-LAB-003 | App-Hosted Editor Lab Surface Shell | implementation | completed | WR-095 | Authors can inspect and edit UI/editor definitions through hierarchy, palette, canvas or preview, inspector, command diff, diagnostics, and preview console panels. |
| PM-UI-LAB-004 | Operation-Driven Visual Authoring | implementation | completed | WR-096 | Canvas, hierarchy, and inspector edits round-trip through stable operations and produce deterministic retained previews. |
| PM-UI-LAB-005 | Persistence Project IO Diff Apply And Rollback | implementation | completed | WR-097 | Authored lab definition changes can be saved, loaded, imported, exported, migrated, reviewed, applied, rejected, and recovered after failed activation. |
| PM-UI-LAB-006 | Preview Lab And Runtime Evidence | hardening | completed | WR-098 | Editor Lab closeout can prove behavior with replayable scenarios, screenshots or equivalent visual artifacts, diagnostics snapshots, accessibility checks, and performance evidence. |
| PM-UI-LAB-007 | API Docs Examples And Runtime-Proven Closeout | release | completed | WR-099 | UI definition and editor definition users have discoverable normal workflows, the Editor Lab has runtime-proven closeout evidence, and perfectionist verification remains separate. |

### PT-UI-LAB-PERFECTION - Editor Lab V1 Perfectionist No-Gap Certification

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-UI-LAB-PERF-001 | Governance Audit Doctrine And Code Truth Matrix | design | completed | WR-100 | The perfectionist track has an accepted audit doctrine, applied WR governance row, source-truth reconciliation, and disjoint future implementation scopes before app or domain code changes begin. |
| PM-UI-LAB-PERF-002 | Runtime Evidence Platform Closure | hardening | completed | WR-105 | Editor Lab V1 can capture or explicitly reject native screenshots, visual diffs, focus traversal, contrast sampling, timing, diagnostics snapshots, and retained artifacts with typed evidence results. |
| PM-UI-LAB-PERF-003 | Command And Surface Source Of Truth Closure | implementation | completed | WR-107 | Normal Editor Lab command and surface behavior derives from the owning catalog or registry, with legacy enum compatibility isolated to migration and persistence edges. |
| PM-UI-LAB-PERF-004 | Direct Manipulation Editor Lab UX Closure | implementation | completed | WR-108 | Authors can use hierarchy, palette, canvas, inspector, operation diff, diagnostics, preview console, undo, and redo without action-list or text-panel dependency for normal workflows. |
| PM-UI-LAB-PERF-005 | Persistence Diff Apply API And Examples Ergonomics | hardening | completed | WR-109 | Normal users can author, validate, preview, persist, diff, apply, recover, and learn Editor Lab workflows through focused APIs and realistic examples. |
| PM-UI-LAB-PERF-006 | Final No Gap Certification Closeout | release | completed | WR-110 | Editor Lab V1 has completed perfectionist closeout evidence, empty known quality gaps, and truthful generated planning docs. |

### PT-EDITOR-UX - Editor Product UX Native Story Lab And Surface Perfection

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-EDITOR-UX-001 | Governance Truth Audit And Track Activation | design | completed | WR-111 | The editor product UX track exists with nine ordered milestones, an active design, completed WR-111 governance closeout evidence, and blocked follow-on rows for the native Story Lab through final no-gap certification. |
| PM-EDITOR-UX-002 | Native Editor UX Story Lab And Evidence Harness | implementation | designing | WR-112 | Editor UX stories, args, controls, interactions, fixture matrices, local-native evidence manifests, and hard widget/surface gates are available for primitives, patterns, surfaces, and host scenarios. |
| PM-EDITOR-UX-003 | Layered Editor Design System Migration | implementation | designing | WR-113 | Editor UI styling and product patterns resolve through shared UI Designer token/recipe/state contracts while editor-specific semantics remain in domain/editor adapters. |
| PM-EDITOR-UX-004 | Standalone UI Designer Workbench | implementation | designing | WR-114 | Authors can use canvas, hierarchy, inspector, property panels, token/recipe/binding previews, scenario matrices, and readiness evidence in a standalone UI Designer workbench. |
| PM-EDITOR-UX-005 | Graph Canvas And Node Editor Productization | implementation | designing | WR-115 | Material graph UX is complete enough for product evidence, and SDF/procgen/gameplay/particle/animation graph surfaces are either productized, fallback-only/diagnostic, or hidden until productized. |
| PM-EDITOR-UX-006 | Shell And Product Pattern Polish | implementation | designing | WR-116 | Editor product patterns have reusable adapters, story coverage, state coverage, focus/keyboard coverage, overflow policy, and native evidence. |
| PM-EDITOR-UX-007 | All Registered Visible Surface Wave | implementation | designing | WR-117 | The editor has no visible misleading placeholders and every visible registered surface has readiness classification, story coverage, scenario evidence, and native proof where required. |
| PM-EDITOR-UX-008 | Game UI Readiness Seam | hardening | designing | WR-118 | Future game-runtime UI has compatible generic UI contracts and evidence descriptors while PT-GAME-RUNTIME-UI remains the owner of runtime HUD implementation. |
| PM-EDITOR-UX-009 | Final Local Native No Gap Certification | release | designing | WR-119 | PT-EDITOR-UX can claim perfectionist_verified only after local-native evidence and all hard zero-budget gates pass with empty known_quality_gaps. |

### PT-GAME-RUNTIME-UI - Game Runtime UI Projection And HUD Platform

| ID | Milestone | Kind | State | Roadmap links | Outcome |
|---|---|---|---|---|---|
| PM-GAME-RUNTIME-UI-001 | Governance Owner Boundary And Code Truth Matrix | design | designing | WR-104 | The game-runtime UI track has an active design, roadmap intake row, architecture-governance findings, production metadata, and follow-on implementation slices without changing app, domain, engine, renderer, or SDF runtime code. |
| PM-GAME-RUNTIME-UI-002 | Game Runtime Target Extension Contract | design | designing | N/A | Game-runtime UI target extensions have an accepted ownership and projection contract before code introduces a new owner crate or runtime behavior. |
| PM-GAME-RUNTIME-UI-003 | View-Model And Intent Contract Activation | implementation | designing | N/A | Game-runtime UI can display domain-owned state and emit validated intent proposals while the owning domain, app, or example remains responsible for mutation. |
| PM-GAME-RUNTIME-UI-004 | Generic Runtime UI Expression Submission | implementation | designing | N/A | Engine runtime owns generic UI expression submission while game HUD semantics stay with game-runtime UI owners and proof adapters. |
| PM-GAME-RUNTIME-UI-005 | SDF Screen HUD Runtime Proof | implementation | designing | N/A | The SDF render-flow example proves screen HUD rendering and validated tab intent behavior while SDF/example state remains the mutation owner. |
| PM-GAME-RUNTIME-UI-006 | Evidence Docs API Ergonomics And Hardening | hardening | designing | N/A | Game-runtime UI runtime behavior is inspectable, reproducible, documented, and honest about unsupported or deferred evidence. |
| PM-GAME-RUNTIME-UI-007 | Runtime-Proven Closeout And Perfectionist Audit Intake | release | designing | N/A | PT-GAME-RUNTIME-UI has runtime-proven closeout evidence and a separate perfectionist audit path for zero-gap certification. |
| PM-GAME-RUNTIME-UI-008 | World Space And Screen Projected Attachment UI | design | deferred | N/A | World-space and screen-projected attachment UI remains outside the SDF screen-HUD proof and gets its own accepted design before implementation. |
