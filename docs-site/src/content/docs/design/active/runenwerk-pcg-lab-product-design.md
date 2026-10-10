---
title: Runenwerk PCG Lab Product Design
description: Active product and boundary design for a generator-agnostic inspection workbench, initially qualifying deterministic cave topology, 2.5D field geometry, materials, and deferred water.
status: active
owner: workspace
layer: product / domain / app-integration
canonical: true
last_reviewed: 2026-10-10
publication: reference
pagefind: false
related_docs:
  - ../../domain/procgen/README.md
  - ../../domain/world-sdf/README.md
  - ../../domain/material-graph/README.md
  - ../../domain/texture/README.md
  - ./editor-procedural-content-and-simulation-workflow-plan.md
  - ./material-lab-and-material-preview-design.md
  - ../accepted/sdf-first-field-world-platform-design.md
  - ../implemented/field-visualizer-product-workflow-design.md
---

# Runenwerk PCG Lab Product Design

## Status and decision authority

**Active design, NOT accepted implementation authority.** [Runenwerk #1243](https://github.com/dornglut/runenwerk/issues/1243) owns this investigation, critical review, and eventual acceptance decision. Accepted implementation requires separately bounded owner issues; this document does not authorize Rust, renderer, Editor, water, or GameTrack work. In case of disagreement, accepted ADRs, framework owners, current behavior, and the owning issues take precedence.

Evidence census at Runenwerk accepted main `9d890e59b23ac211f297ae75063eefb06a2617e4`; this SHA is provenance, not a pinned future implementation base. [World authority A0 PR #1132](https://github.com/dornglut/runenwerk/pull/1132) is unmerged at census and has independent accepted-doc ownership. [Material/Texture P3 #841](https://github.com/dornglut/runenwerk/issues/841) is investigation-only. [Game product GD0 #1242](https://github.com/dornglut/runenwerk/issues/1242) has not selected a complete game/camera/art contract. Do not elevate any of their candidates into accepted PCG Lab prerequisites without later verification.

## Product promise

PCG Lab is an **independently launched Runenwerk product application** where a designer can configure procedural content, generate bounded candidates, inspect intermediate meaning, compare and diagnose candidates, retain a reproducible recipe and, eventually, explicitly publish an accepted product. It is **not** a new generation engine, material/shader editor, simulator, renderer, world source authority, or universal graph type.

The **first qualifying scenario** is an organic **top-down cave** with a flat gameplay plane, readable ground and wall regions, seeded topology, rules, geometric shape/noise, procedural field inspection, and surface material composition. Rich stone/dirt appearance, wetness, lighting, and eventually shallow transparent puddles with visual interaction ripples are **desired experience targets**, not accepted current renderer capabilities. The product/game design remains open under GD0; do not impose fixed objectives, combat archetypes, enemy sizes, camera pitch, extraction timings or art palette from this engineering example.

Future world, planet, and universe generators are relevant volatility pressure, **not** permission to standardize their coordinate systems, field storage, dataflow algorithms, topology, quality metrics, or execution plans from the cave case.

### Ordinary first user journey

1. Launch the PCG Lab app without a Runenwerk Editor scene or project; pick a *Cave* generator preset or reopen a versioned recipe. Author seed, bounded scope and intelligible topological/shape controls. A generator execution graph is an *advanced authoring facility*, not required knowledge for the first user.
2. Generate a bounded **macro topology candidate**; inspect chambers, intended connections, graph hard-constraint failures, optional paths and diagnostic source identities. Replay exactly, compare seeds and explicitly distinguish an invalid candidate from a valid but lower-ranked one.
3. Form a **2D spatial embodiment** of a selected valid topology. Inspect open/rock occupancy, derived signed wall-boundary distance, passage clearance and the correspondence between intended edges and actually traversable corridors after every geometry-changing transformation.
4. Inspect the **formed cave scene** (not a static/mock texture): flat ground, walls, and separately identified ground/wall material products. Changing material noise does not change collision; changing shape noise produces a new candidate and reruns required spatial checks.
5. Later: retain authored pins/overrides, save recipe/candidate selections, compare metrics, and explicitly admit/publish through existing product and world authority. A separate qualified water/runtime track may later show shallow puddles and local disturbances.

The first usable result is **real topology and actual generated field geometry with truthful diagnostics**, not an elaborate node catalog or a text panel renamed PCG Lab.

## Existing source-backed boundaries and false shortcuts

| Existing owner/path | Proven capability/limit at census | PCG Lab consequence |
| --- | --- | --- |
| `domain/procgen/src/document.rs`, `ratification.rs`, `planning.rs` | Document graph, typed first-slice terrain/material nodes, seed/version/scope, reservations, issue codes, lowering and candidate lifecycle vocabulary. | Add cave-family semantics under its owner, not a second generator owned by an Editor provider. A procgen *execution graph* is not a generated cave *topology graph*. |
| `domain/procgen/src/field_preview.rs` | Deterministic bounded **3D** height/noise scalar-distance and material-channel preview (initial 32³ policy). | Not proof of a `Field2D` source, 2D cave producer, playable cave, or valid cave geometry. |
| `domain/world_sdf` | Current world SDF payloads, sampled metrics, collision queries and field-preview DTOs. | A cave-specific 2D candidate must have an explicit qualified adapter to accepted world/collision truth. Do not equate `world_sdf::caves` summaries with runnable caves. |
| `domain/product` | Shared descriptor/job/query/publication/consumption vocabulary, not central product semantic authority. | Reuse common product contracts; preserve cave- and world-specific truth at their owners. A0 #1132 is separately reconciling accepted docs to current source. |
| `domain/editor/editor_viewport/src/expression/product.rs` and Field Visualizer | `ScalarField2D`/`VectorField2D` are **derived Editor viewport kinds**, not source-field truth. | Reuse suitable *neutral product visualization concepts* without binding the standalone app to Editor application internals. |
| `apps/runenwerk_editor/src/shell/providers/procgen_*.rs` | Current graph/preview providers present text/diagnostics and do not map substantive cave actions. | These are optional **future Editor adapters**, not mandatory PCG Lab or common generator runtime. Do not copy source truth into a second app. |
| `domain/material_graph`, `domain/texture`, Editor Material Lab | Material graph ratification, PBR/noise/field vocabulary, texture descriptors, source-backed formed products and renderer previews exist. | No duplicate Material Lab; a *real cave-field → material → cave pixels* integration is still an unproved consumer. #841 owns broader P3 gaps. |
| `dornglut/runen-render` current maintained ray evaluator | Primary rays, sphere/plane and sampled-field intersections, direct spectral radiance and directional occlusion visibility exist. Separate `composition_2d` is a UI/image compositor. | Do not claim ready-made full RGB PBR, multi-bounce GI, refraction, water or normal-rich cave scene from “ray tracing.” Probe the exact selected Runenwerk render/material path and its accepted evidence before scene claims. |

## Newly verified D0 blockers: first-slice Procgen admission and metric clearance

An independent follow-up source review found **two gates that cannot be postponed to routine C1/C2 implementation**:

**D0-B1 — A topology-only product is not currently an admitted Procgen first-slice output.** `domain/procgen/src/ratification.rs::ratify_graph` explicitly emits `MissingWorldOpsOutput` and `MissingFieldProductOutput` unless **both** output nodes appear. Its `ratify_output_products` and write-target logic enforce corresponding existing descriptors/targets. A macro graph candidate is neither a fake WorldOps mutation nor a fabricated field-preview product. Before C1, the owner MUST decide and ratify a compatible bounded *candidate-only* or newly typed topology output contract, including its publication/lineage and separate later geometry/bake admission. Do not add dummy output nodes, bypass current ratification, or let an Editor-only Rust structure stand in for an admitted candidate. The existing `ProcgenScope` also requires a `WorldId` and nonempty chunk/region scope; this is valid for the cave/world slice but must not be misrepresented as a generic universe-generator scope.

**D0-B2 — The ordinary collision query is not certified metric distance.** In `domain/world_sdf/src/collision.rs`, `sample_sign_from_brick_samples` returns `sample.signum()`, with `occupancy_sign_from_mask` yielding `+1/-1`. Thus the `CollisionQueryService::sample_signed_distance` path can classify solid vs open but cannot be reused without qualification as a physically meaningful distance-to-wall or actor-clearance measure. By contrast, `domain/world_sdf/src/metric.rs::WorldSdfMetricSample` explicitly carries a signed-distance **estimate in meters** plus maximum error and a safe lower bound. The first cave product MUST define which bounded distance/equivalent clearance contract it forms and which **actual game collision and rendering consumers** have compatible guarantees; adapters must handle sampling and errors rather than asserting that a sign-only collision result proves passage width. A successful abstract path or 2D colored distance preview is not an adequate acceptance test.

**Regression counterexamples:** a narrow valid-looking corridor for which sign-only occupancy remains traversable under naive checks but radius-based clearance fails; a field whose interpolated sign is correct but numerical magnitude has no metric meaning; macro generation rejected by the current mandatory output-node ratifier; and a candidate inserted through dummy output records or an unratified publication path. Both D0-B1 and D0-B2 require explicit reviewed dispositions before C1/C2 delivery authority is activated.

## Hosting review and revised product decision — standalone first (2026-10-10)

The initial **Editor-first** proposal reflected the existing Procgen providers, not the best product boundary. The owner requested an independent Lab. The accepted Runenwerk app architecture supports that: `apps/runenwerk_render_lab` is a dedicated Render Lab crate/executable with native and headless execution, while `apps/runenwerk_draw` independently composes UI, engine and domain products without depending on `apps/runenwerk_editor`. By contrast, Material Lab and UI Designer launch dedicated binaries via `RunenwerkRuntimeWorkbench` inside the Editor crate. **A separate launch icon is not necessarily independent application ownership.**

### Host comparison

| Alternative | Evidence-backed advantage | Limitation | Decision |
| --- | --- | --- | --- |
| Editor-only primary workbench | Reuse of current Procgen panels, persistence, viewport | Forces an unrelated tool to depend on heavy Editor app/scene/session ownership and encourages editor-local generation truth | Rejected as primary |
| Dedicated executable inside Editor crate | Material Lab/UI Designer proof exists; lower near-term wiring | Still uses Editor App/host, inputs and presentation; not really independent from Editor | Transitional only if separately justified |
| Dedicated `apps/runenwerk_pcg_lab` app within Runenwerk | Render Lab/Draw prove distinct product crates can use shared engine, UI and domain APIs; headless parity fits PCG generation; app owns ordinary launch/session/recipe UX | Requires a bounded new app composition, native input/viewport/product adapters; naive copying of Editor runtime is a risk | **Selected long-term and first product-delivery host** |

External tools corroborate the *product* distinction without determining the implementation: [World Machine](https://www.world-machine.com/features.php) and [Gaea](https://docs.gaea.app/ui/index.html) have standalone graph/viewport-centered terrain authoring; [Gaea's automation path](https://docs.gaea.app/developers/automation/index.html) decouples repeatable builds from GUI launch. [Houdini Engine](https://www.sidefx.com/products/houdini-engine/) demonstrates procedural source assets consumed by distinct host applications; [Unreal PCG](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-overview) is an embedded alternative when generation is tightly coupled to the game's world editor. These are UX/product precedents, **not** authority for a Runenwerk global PCG IR, new framework, or plugin API.

### Selected application/domain ownership

```text
domain/procgen            cave recipe / generator semantics / ratification / candidates
world_sdf + world_ops     spatial/query/collision/publication authority
material_graph + texture source-backed material and texture semantics
domain/product            existing descriptor, job, observation vocabulary
             |
      owner-defined products
             |
      +------+------------------------+
      |                               |
headless formation / tests     apps/runenwerk_pcg_lab (native)
                                      |
                            engine::App + app-owned session
                            ui_composition + neutral UI/render-data
                            RunenRender/RunenGPU consumer integration
                                      |
                         optional future Editor-hosted adapter
```

Standalone means **app/process launch, lifecycle, file/recipe handling, native input/window routing, workspace and presentation ownership**, not independent generator/world/material/renderer authorities. The PCG Lab app should consume `domain/procgen` directly through legitimate owner contracts and use `engine::App`, `ui_composition`, `ui_render_data` and product/render integrations wherever suitable. Do NOT make ordinary generation depend on `RunenwerkEditorApp`, `EditorHostResource`, the Editor Material Lab application state or Editor-private field visualizer/provider handlers; any genuinely reusable neutral visualization primitive must be qualified and factored without a new universal Lab host.

The current PCG execution/preview/bake adapters live substantially in `apps/runenwerk_editor/src/runtime/procgen/mod.rs`. Therefore **the standalone path is not free**: first form C1's ratified headless owner output, then implement only the app adapter to request and inspect those exact products. Later C2 must similarly use the same authorized world/query bridge and accepted recipe identities rather than recoding generation in the standalone app. A second visual-only cave generator, copied Editor runtime, parallel product registry or global input manager is disallowed.

### Staging and proof

- **C1 headless first:** genuinely ratified topology candidate and constraints with deterministic recipe/seed/version/scope lineage, no GPU/Editor dependency.
- **L0 standalone native app:** dedicated executable launches with no Editor project or scene, creates/opens a recipe, requests C1 outputs, shows selectable macro nodes/edges and rule diagnostics, compares two seeds and preserves last-good on failure. Headless and native output identities/diagnostics agree.
- **C2/L1:** world/clearance-qualified geometry plus genuine field and material-scene views, product revision/staleness and resizing/pan/zoom/input evidence through app-neutral integration.
- **Later** separate Material Lab cave pixels, water/ripples, and optional embedded Editor consumer. A future Editor adapter consumes the **same** source products, not a copied cave algorithm.

**D0 host gate:** accept the new product identity and app dependency direction; confirm the minimal standalone `App`/composition/native/render UI route by independently reusing the Render Lab and Draw patterns; reconcile topology-output admission D0-B1 before C1 and metric clearance D0-B2 before a physical cave viewer. The D0 design issue **does not authorize creating `apps/runenwerk_pcg_lab`**, changing Cargo, or implementing algorithms.

## Editor future and adjacent Visual Lab boundary — exact-current audit (2026-10-10)

**Scope:** PCG Lab is a Runenwerk product, but cannot choose its new UI host merely by cloning today's Editor `UiRuntime`. This audit distinguishes accepted architecture, implemented consumers, planned replacement and *unaccepted* Visual Lab product work.

### Current Editor and UI status

- [ADR 0025](../../adr/accepted/0025-normalize-editor-coordination-and-semantic-ownership.md) and its accepted Editor model normalize explicit owner bindings, surface sessions, interaction routes, projection publication, selection/history/persistence contexts. They do **not** assert that those terms are all implemented public Rust APIs, and do not authorize a standalone `runen-editor` repository.
- [Current Editor architecture](../../apps/runenwerk-editor/current-architecture.md) documents real `ui_composition` structural authority, target-local Editor viewport/tool sessions and remaining `DocumentKind`/EditorSession migration seams. [Editor roadmap](../../apps/runenwerk-editor/roadmap.md) retains broad M6 authoring ambitions, **but** its current M6 authority is owner-domain formation first, Editor Tool Suite/provider integration second, with Procgen Phase 6D already complete. This is **not** a requirement to locate new cave generation inside the Editor application or extend central enums.
- Repository architecture program [E0–E3](https://github.com/dornglut/runenwerk/issues/1153) completed structural retirement/internal shell, self-authoring and runtime responsibility decomposition through closed [E1 #1180](https://github.com/dornglut/runenwerk/issues/1180), [E2 #1189](https://github.com/dornglut/runenwerk/issues/1189), [E3 #1204](https://github.com/dornglut/runenwerk/issues/1204). Those were not Editor product replacement, standalone extraction, or UI-framework migration. [Editor #737](https://github.com/dornglut/runenwerk/issues/737) closed with **no-cut-yet** for generic binding/extraction until genuine multi-scope consumer proof.
- [ADR 0013](../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md) makes standalone `dornglut/runen-ui` the target reusable UI runtime, but preserves Runenwerk-owned structural `ui_composition`, authorship, product/Workbench and renderer integration semantics. `ui_tree`, `ui_widgets`, `ui_runtime`, generic `ui_input` and relevant `ui_render_data` publication paths are current **predecessor/mixed** state for nonmigrated consumers, **not default long-term implementation choices for a new standalone PCG product**.
- [RunenUI adoption #994](https://github.com/dornglut/runenwerk/issues/994) remains **open**. Counter U5 migrated a bounded consumer, not the whole Editor or arbitrary typed apps. [U8 #1238](https://github.com/dornglut/runenwerk/issues/1238) concluded the first full Editor target cut is **deferred**, because the pinned public controls/multi-target host/renderer path do not prove complete replacement. [U9 #1239](https://github.com/dornglut/runenwerk/issues/1239) is investigating general typed `UiApp` host/input/render integration and is **not delivery authorization** for game UI or other products. The latest upstream framework API cannot be assumed compatible with Runenwerk's actual pinned dependency.
- [Visual Lab #236](https://github.com/dornglut/runenwerk/issues/236) remains **open**; its former PR #237 was **closed unmerged**. Its host-neutral procedural-visual-study composition direction is *an adjacent proposal*, not an accepted `VisualLabCore`, common Lab host, generalized graph, source schema or application. There is real product-workflow overlap (run/update/see/compare/keep), not demonstrated common execution/authoring semantic authority.

### D0 architecture dispositions and new gates

**D0-B7 — Standalone UI consumer readiness:** C1 headless generator formation **does not depend on any GUI framework**, and must proceed through owner contracts independently. Before L0 native work, qualify the **actual selected UI consumer path** using accepted **pinned public RunenUI** View/Element, mounted/runtime/host/input/focus/layout/text/paint and renderer/RunenRender output evidence. Reuse U9's accepted generic typed-host contracts *if and when available*; U9 itself is **not an automatic blocking predecessor** for an independently decision-complete Lab app-host adapter. The new Lab must not establish lasting mounted/runtime/widget semantics using a Runenwerk-local predecessor UI stack selected only for fast initial coding. Nor may it force a premature Editor U8 migration, invent dual RunenUI/legacy mounted authority, pass private RunenUI state to Runenwerk, or ship a generic compatibility-UI framework. If a complete native host cannot yet be qualified, report the exact missing capability; do not mislabel a headless/domain proof as an interactive PCG Lab.

**D0-B8 — Visual Lab overlap resolution:** PCG Lab owns **specialized procedural-generation product workflows**: recipe/seed controls, topology/constraints, stage/field/geometry inspection, reproducible generated candidate comparison and qualified publish/export. Visual Lab #236 proposes **broader visual experiments and cross-domain creative composition** without becoming any owner of Procgen generation. No accepted Visual Lab design currently requires PCG Lab to become one of its modes, depend on a Visual Study schema or use shared `RunenLab` infrastructure. Treat app-host consolidation, product navigation, study interoperability and shared compare/inspection UI as **future evaluated user/product pressures**, not a default generic Lab abstraction. Before D0 acceptance explicitly record the product relationship and avoid duplicating one global recipe/study authority in both labs.

**D0-B9 — Editor integration after standalone:** the completed Editor E0–E3 program and its continued M6 aspirations do not require PCG Lab to be Editor-hosted. A later Editor consumer uses stable Tool Suite keys, provider-family routing and optional owner-backed projections. It must consume the exact accepted PCG candidates and product revisions; it cannot mirror cave truth, introduce a `DocumentKind` variant or depend on hypothetical generic `EditorBinding` Rust APIs. Native/Editor consumer cutover or extracted shared host mechanisms each require separately accepted owner issues and evidence.

### Revised developer/product review matrix

| Concern | Accepted now | Investigating/deferred | Lab consequence |
| --- | --- | --- | --- |
| Editor structural composition | `ui_composition`, ADR 0013 | residual Editor coordinator normalization | Standalone may use app-neutral composition; Editor source is not required |
| Editor live shell/runtime refactor | E0–E3 completed | no whole-Editor replacement plan activated | Do not wait for or reimplement Editor internal cleanup |
| Generic Editor extraction | ADR 0025 target only | #737 no-cut-yet | No dependency on a future standalone `runen-editor` |
| Reusable mounted UI framework | RunenUI selected as target; Counter U5 accepted | U8 Editor source cut deferred; U9 typed-App integration open | L0 needs its own exact-public-API UI/host/present qualification |
| Procedural product | Procgen baseline owner and M6 6D completed | cave topology/C1 and field geometry/C2 missing | Form once at domain; native Lab and later Editor project it |
| Broad creative Lab | #236 remains unmerged proposed direction | no accepted shared host/VisualStudy schema | Keep specialized PCG Lab and avoid duplicate general workflow authority |

**Critical review conclusion:** the standalone-first PCG Lab product identity still matches Runenwerk's accepted architectural direction. **Its original reuse-by-default wording for Editor UI/viewport modules and local `ui_render_data` was insufficiently future-proof** given the active RunenUI clean-cutover program. The implementation order remains headless owner proof → independently qualified native consumer → optional Editor adapter, not old Editor UI → duplicate standalone UI → later rewrite.
## Semantic model and one-way dataflow

```text
authored generator recipe / version / bounded scope / seed
    -> domain/procgen ratification + deterministic formation
    -> candidate macro topology (chambers, intended connections, rules evidence)
    -> candidate spatial embedding (2D coordinates, area reservation)
    -> bounded geometric formation (open/solid, contours, wall geometry)
    -> independently evaluated final-geometry constraints
    -> ratified cave spatial product + optional auxiliary field products
         |                |                   |
         v                v                   v
   query/collision    Editor field view   material inputs
         |                |                   |
         +---------------> product-composed cave viewport <--- Material Lab products
                                      |
                         explicit owner-governed world publish
```

**Do not infer authority from the diagram**: topology candidates, sampled masks, editor graph canvas positions, color ramps, generated texture pixels, renderer caches and runtime water ripples are not equivalent representations of one owning truth. The precise world-admission boundary is a D0 gate, to be reconciled with A0 current accepted owner contracts.

### Three graph notions

1. **Generator graph**: existing authored port/dataflow structure via `ProcgenDocument.graph` with Procgen-owned meaning/ratification. Its result is not necessarily a graph.
2. **Macro topology**: typed *generated content* with stable chamber/edge identities, optional semantic role labels, edge requirements and rule lineage. An edge is **intended** passage connectivity only.
3. **Spatial connectivity**: adjacency and agent-clearance on formed final **geometry**, produced by independent field/collision query evidence. Graph validity is necessary but never sufficient to claim a playable route.

The first cave does not require a universal shape grammar, mission-graph DSL, solver framework or graph-authoring GUI. Presets with bounded, explicit typed rules and a real generated macro graph inspection surface are a more reviewable first consumer.

### Rules, constraints, and quality

- **Constructive rules** propose bounded variations: backbone, optional chambers, branches, cycles, room roles, passage attachments.
- **Hard constraints** reject candidates: required-graph reachability; legal node identities; bounds and spatial overlap; intended-edge realization; formed physical minimum clearance under a *declared agent footprint*; required destinations reachable; no unapproved additional shortcuts when those matter to the recipe.
- **Soft objectives** rank only already valid candidates: branch depth, loops, chamber variation, navigable area, chokepoint distribution, route alternatives and morphology variety.
- **Game-derived constraints** (spawn placement, extraction mechanics, enemy-specific widths, objective order) are **not** universal Procgen laws. Only accepted game-owner input can qualify them.

A constraint is tagged with **the stage it observes** (topology, spatial placement, final geometry, or contextual game evidence), its source identity, exact candidate revision and affected spatial/graph subject. A violated hard condition cannot be disguised as a low score or silently relaxed. Bounded deterministic attempts/backtracking/repair are permitted only with declared search budgets, reproducible attempt counts, explicit infeasible/exhausted status and no unbounded reroll loop.

Example counterexamples that MUST be demonstrated:
- topology connects A–B but CA smoothing seals the only passage;
- wall noise opens an unrequested shortcut despite satisfying the intended-edge graph;
- the path looks open at pixel resolution but fails a declared actor-clearance query;
- two constraints are contradictory, so no candidate can be accepted;
- an out-of-order finished job overwrites the selected seed's newer preview;
- prior accepted world or preview is corrupted by an invalid draft.

## 2D fields and 2.5D cave geometry

**Cave's first geometric workload is planar in gameplay, not flat in appearance.** Use one consistent **open/rock classification** on a world-mapped XZ region and a horizontal walkable plane. A signed wall-boundary field (if sufficiently guaranteed) can drive curvature, query clearance and 2.5D wall shape. Floor and wall shading are derived presentation of accepted formed geometry; they must not have independent incompatible collision geometry.

For a horizontal flat floor at vertical coordinate `y0`, the analytic plane distance is `d_floor(x,y,z) = y - y0` under a stated sign/normal convention. **A floor plane by itself neither restricts traversability to the cave nor generates walls.** A 2D open/solid mask or bounded distance representation and an explicit wall/plane composition policy are required. Wall height, camera-obstruction policy, cap/ceiling semantics and whether a wall is an extruded 2D contour or a sampled 3D WorldSDF must be decided by evidence, not by the word "SDF".

**Do not conflate the following meanings** even if all become sampled grids or Texture2D artifacts:

| Candidate value | Meaning and consumer |
| --- | --- |
| Open/solid occupancy | Generated cave partition; eventual geometry/collision source policy |
| Signed wall-boundary distance | Spatial query/clearance and possible ray-surface intersection, only with explicit sign, metric/error/conservatism |
| Coarse geological scalar or masks | Material variation, regional wetness or placement; does not cause collision unless explicitly promoted by the owning geometric operation |
| Normal/roughness/bump detail | Appearance; can be high-frequency independent of low-frequency collision shape |
| Puddle region/depth hint | Placement/appearance input; not water simulation authority |
| Texture2D or sampled atlas | A data/storage/render artifact carrying values and mapping metadata; not necessarily field truth |

Continuous procedural functions, sampled fields and baked Texture2D products are **distinct evaluation/storage choices**. A signed field distorted with arbitrary fractal noise is generally **not automatically a metric signed distance field**. If a strict distance consumer needs a certified error or conservative stepping bound, the formation/adapter MUST prove it or reject the candidate. Noise changed in the material graph must not change cave collision. Noise used for contours MUST invalidate affected geometry/query acceptance.

### Noise and geometry method comparison

- **CA-first** can generate organic open/solid shapes simply but does not by itself guarantee required chamber roles, final connectivity, passage width or spatial layout.
- **Topology-first with bounded spatial embedding and shape passes** makes graph requirements explicit and allows organic local irregularity; risk: mechanically obvious rounded rooms/corridors. Test silhouette variety and recognizable procedural artifacts.
- **Hybrid** may use connected chambers/passages as hard spatial anchors and CA/domain-warp only within allowable envelopes. Compare against an unconstrained CA baseline on the same seed corpus.
- **Visual detail** uses independent multi-scale noise, rock cracks, roughness and normal variation, with separate cost and material ownership. **CA smoothing is not actual geological erosion**, and live erosion/fluids are out of this slice.

No specific noise implementation, CA rule, graph grammar or SDF storage layout is approved by this design. Compare deterministic candidates and accept only based on measured geometry validity, user-facing visual quality and workload behavior.

## Material Lab, lighting and shallow water

**Material Lab remains the only material authoring owner.** PCG Lab selects material product identities and exposes the sampled cave context needed by their declared semantic inputs (world position, geometry/normal/distance, independently versioned material masks, wetness). Material identities/scene assignments and texture products remain at existing owners. A cave material preview must use actual formed cave geometry, not substitute a sphere or synthetic field-material fixture; [#841](https://github.com/dornglut/runenwerk/issues/841) separately decides missing material/PBR/procedural-texture semantics.

Appearance target for first finished cave scene: layered ground/wall rock/dirt materials, visible noise variation, wetness response, readable wall depth and shadows. These are **independent evidence gates**; don't assert that material output transport proves actual consuming shader pixels or that a ray-intersection shader automatically provides full material shading.

**Shallow-water concept**: a region of walkable floor, bounded water surface depth and appearance inputs, with visible terrain *under* the water. At later runtime, player movement may emit disturbance events and create local ripple-like surface normals/displacement. Separate the owners:

- Procgen: deterministic initial puddle extent and optional supporting spatial masks.
- Material/Texture: water/wet-rock appearance inputs, if qualified by a specific material product.
- Rendering: transparent/refraction/reflection/depth compositing, correct visibility of underlying terrain, shadow/lighting as implemented.
- Game/Simulation: disturbance event and any actual water state, only if separately authorized. Visual-only ripples are non-colliding derived state.

Do not implement or invent a generalized `domain/water` API under this design. Water/ripple is a separately accepted successor, not a prerequisite to topology/field viewing. Depth readability is a product target even if the eventual renderer selects a bounded alternative to physical refraction.

## UI and failure behavior

PCG Lab MUST own a dedicated bounded native App composition over accepted app-neutral UI/render/product contracts; it MUST NOT require the full Editor App/Tool Suite runtime or create a competing global input/focus authority. Editor embedding is optional later. The initial useful workspace pairs:

- macro topology view with node/edge selection, violation overlays, graph metrics and seed comparison;
- field/geometry viewport selecting accepted or candidate products, occupancy/distance/clearance/connected regions and exact stage/source revisions;
- compact recipe/rule inspector; clear hard failures vs soft scores; 2D scene preview when genuine renderer/material integration is available.

Selecting a macro chamber or connection should correlate it to its placed/formed region. Display **unavailable** as unavailable: no placeholder pretending to be a real generated GPU scene. Separate generated candidate, selected preview, preserved last-good, accepted/baked and live-world revisions. Generation runs have request identity and terminal success/failure/cancel/stale states; a newer selection must not be replaced by late older completions. Cancellation and failure must leave prior accepted products intact. Comparisons should record source, generator version, scope, inputs and acceptance predicate rather than a screenshot and one anonymous quality score.

## Decisions and deferred questions

| Question | Current disposition and gate |
| --- | --- |
| Standalone PCG Lab app now? | **Yes as the target product**: dedicated `apps/runenwerk_pcg_lab` crate/native executable in Runenwerk after D0/C1 authorization. Editor embedding is secondary. This issue does not authorize creating the app. |
| Which UI runtime for L0? | Use an exact accepted public **RunenUI** consumer path where support is verified; U8 Editor cut is deferred and U9 general typed App host is investigation-only. Do not build a new product's long-term mounted/widget runtime on retiring local UI predecessors. D0-B7 must qualify real native interaction/paint/presentation. |
| Does Visual Lab own PCG Lab? | **No accepted authority**. #236 is an open, unmerged creative Visual Lab proposal. Preserve PCG-specific recipes and generation; evaluate overlap without inventing a shared Lab host or VisualStudy schema (D0-B8). |
| Universal `Field2D` authoritative Rust abstraction? | **No current mandate**. Establish the smallest typed cave fields with coordinate, unit, error, scope and consumer contracts; promote shared semantics only with multiple qualified consumers. |
| Generator graph or macro graph first? | **Macro topology proof first** for the cave slice; generator graph editing may follow. Do not confuse the two. |
| Specific CA vs graph grammar vs solver? | **Not selected**. Require bounded comparative fixtures; topology and geometry invariants independent of implementation. |
| Runtime source is necessarily sampled 3D WorldSDF? | **Unresolved** until accepted field-to-world product and render/collision query adapter is demonstrated. No 2D texture as hidden authority. |
| Full shaded scene from current raytracer? | **Unproved**: reconcile Material Lab's actual cave scene path and RunenRender's current spectral/direct evaluator; qualify real pixel evidence. |
| Physical height variation in gameplay? | **Out** of first cave; flat collision surface with procedural shading detail. |
| Physical water simulation? | **Out** of first cave. Puddle scene/ripples require independent future evidence/owner acceptance. |
| Planet/universe plugin API? | **Deferred**. Reuse only existing product lifecycle and explicitly common workflows. |

## Acceptance evidence, not claims

For the eventual bounded delivery issues, require:

- **Headless deterministic test corpus**: identical complete lineage gives exactly reproducible macro proposals, candidate rankings, valid or failed outcomes and final field products; invalid seeds retained. Graph identity must remain stable under well-defined ordering/canonicalization and seed partitioning.
- **Invariant tests**: disconnected intended graphs, contrary rules, spatial overlap, severed corridor, invalid metric-SDF bound, actor-footprint clearance failure, extra unwanted adjacency, out-of-budget placement, empty scope, canceled/stale jobs.
- **End-to-end field proof**: chosen seed's final open/rock mask, distance/metric guarantee, derived geometry, collision queries and rendered boundary agree within stated discretization/error; if not, the feature fails instead of silently projecting a beautiful but incompatible mask.
- **Standalone application proof**: launch without the Editor app/project, generate two seeds with controls, inspect real graph and geo candidates, pan/zoom/select, pick failing edge/region, inspect stage and revision, recover from failure and preserve selection across supported save/load. Real images, no status-only widgets.
- **Cave shading proof**: source-backed floor/wall material assignment and separate material-noise edits produce identifiable visible native scene pixels, while topology, geometry and strict collision remain unchanged. Ray/lighting features must have exact method/consumer evidence.
- **Later shallow water proof**: bounded generated puddle over accepted traversable ground; terrain clearly visible beneath the surface; a triggered local disturbance changes water appearance only, no unauthorized mutation of world physics or route geometry.
- **Performance**: report actual seeds, dimensions, budgets, CPU/GPU device, timings, memory/cost and fail-closed limits for the executed tests. No unmeasured universal performance target and no successful-render claim from a shader compile or DTO test.
- **Validation**: `cargo validate` via repository-owned exact-head hosted CI plus independent code/design review and protected merge guards for each accepted implementation and design PR; design-only markdown/links can be reviewed and checked separately but are not code conformance.

### Delivery ownership suggested for later issue activation

The investigation issue remains sole current PCG Lab owner. Only after design review and approval:

- cave topology/rule formation and headless conformance — `domain/procgen` owner;
- cave geometric realization and qualified world/query adapter — Procgen + owning WorldSDF/WorldOps contracts, explicitly coordinated with A0;
- actual Lab graph/field scene inspector — dedicated `apps/runenwerk_pcg_lab` native application over the shared App, UI composition, product/renderer and qualified owner adapters. Editor-hosted integration may follow;
- qualified cave material/lighting evidence — Material Graph/Texture + actual renderer consumer; coordinate #841;
- shallow water/ripples and GameTrack input/consumer — separate water/render/game-authorized issues; coordinate #1242/#946.

These are **ownership boundaries and acceptance gates**, not a duplicate roadmap or authorization to create all implementation issues immediately.

## Alternatives critically rejected at this stage

1. **CA + post-hoc repair as the only level structure.** Loses explicit intentional topology and may produce looping/nontermination or constant invalid seeds; retain CA as a candidate organic-shape method.
2. **Texture2D as authoritative geometry and collision.** Sampling/pixel filtering/LOD/revision may diverge from geometry queries and the world owner; allow texture outputs as declared derived representations.
3. **Renderer-side procedural displacement for playable walls.** Rendering cannot independently decide where the game can walk; no invisible collision disagreement.
4. **A new universal generator/rule/field runtime.** The existing Procgen/Graph/Product/World owners already cover shared contracts; adding speculative cross-domain authority would violate single-owner design.
5. **A standalone viewer with a duplicated cave generator.** Still rejected: the selected standalone app consumes exactly the same domain-owned Procgen candidates as headless and future Editor/game consumers.
6. **Treating water as “just a blue transparent material.”** Fails visible-bottom depth/occlusion/ripple requirements; later water compositor and event evidence is necessary.

## External technique comparisons (design inputs, not Runenwerk authority)

- [Unreal PCG Overview](https://dev.epicgames.com/documentation/unreal-engine/procedural-content-generation-overview): graph/node inspection and debug rendering are relevant UX comparisons; their node/data/executor authority is not imported.
- [Unreal PCG data types](https://dev.epicgames.com/documentation/en-us/unreal-engine/procedural-content-generation-framework-data-types-reference-in-unreal-engine): distinguish spatial, composite and attribute data; not a mandate for the same universal IR.
- [Blender Geometry Nodes Fields](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/fields.html): lazy/contextual field evaluation reinforces the distinction between a field expression and where/how it is sampled.
- [Houdini heightfield layers/masks](https://www.sidefx.com/docs/houdini/heightfields/masking.html): masks can control independent procedural operations without becoming geometry itself. Runenwerk is not a heightfield-first cave by assumption.
- [Godot FastNoiseLite](https://docs.godotengine.org/en/stable/classes/class_fastnoiselite.html): procedural noise families and domain warping are available algorithmic comparisons, not evidence of gameplay-correct geometry.
- [Linden, Lopes, Bidarra, *Designing Procedurally Generated Levels* (2013)](https://ojs.aaai.org/index.php/AIIDE/article/view/12592): designer constraints can guide a graph-to-space construction. Do not require a graph grammar before the simpler typed cave constraints have a demonstrated need.
- [van der Linden, Lopes, Bidarra, *Procedural Generation of Dungeons* (2014)](https://research.tudelft.nl/en/publications/procedural-generation-of-dungeons/): highlights controllability of generated levels as a material design challenge.

## Review/acceptance questions still requiring independent resolution


1. What **minimal spatial cave candidate** and world-admission adapter is compatible with A0's final accepted `domain/product`, WorldSDF, and query contracts, without assigning semantic authority to a derived mask?
2. What exact **distance conservatism / sample error** must be preserved for both collision and the selected ray-based renderer for a noisy derived wall?
3. What first **macro topology controls** are actually designer-usable without an unnecessary mission grammar or authoring graph requirement?
4. Which **real cave-material → scene-ray render** path can form acceptable layered cave pixels with current capabilities, and which semantics must be separately delivered under #841 or renderer issue authority?
5. What **bounded standalone App/UI/native window/field/rendering path** can present changing cave products without copying the Editor app/runtime or creating a new global input/render authority?
6. What smallest representative **seed corpus, fail-closed budgets and user review** can reject the easy but poor designs before production scope is activated?
7. Which **water rendering ownership** is appropriate only after initial cave geometry/material and actual rendered scene evidence exists?
8. Is the L0 UI consumer's pinned public RunenUI/Engine/RunenRender/RunenInput path complete enough for actual native graph controls, focus, text, painting, surface lifetime and headless parity **without reintroducing predecessor UI or relying on deferred Editor U8**?
9. What is the precise non-overlap and possible *later* interoperability between PCG Lab cave-generation/acceptance workflows and the still-unmerged Visual Lab #236 creative-study product?

#1243 remains open until these architecture questions are reviewed, dispositions recorded in that issue, and the design is accepted through normal repository governance.
