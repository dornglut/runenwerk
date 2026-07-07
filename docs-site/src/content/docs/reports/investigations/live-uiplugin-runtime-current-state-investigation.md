---
title: Live UiPlugin Runtime Current State Investigation
description: Current-state investigation and design-gate evidence for PT-UI-RUNTIME-PLATFORM-001.
status: active
owner: ui
layer: reports
canonical: true
last_reviewed: 2026-07-06
related_docs:
  - ../../workspace/start-here.md
  - ../../workspace/workflow-lifecycle.md
  - ../../workspace/authority-model.md
  - ../../workspace/documentation-structure.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/complete-merge-readiness-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/ui-framework-architecture.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
---

# Live UiPlugin Runtime Current State Investigation

ID: `PT-UI-RUNTIME-PLATFORM-001`

Title: `Live UiPlugin Runtime and Generic Surface-Frame Rendering`

Lifecycle state: `active-planning` design-gate hardening.

Implementation authorization: not authorized by this investigation.

## Result

Complete investigation gate status: complete for current-state, authority, owner, dependency, vocabulary, capability, alternatives, confidence, and blocker evidence needed to harden the design gate.

Complete design gate recommendation: promote the companion design to design-gate complete for opening a separate implementation-planning PR only. Runtime implementation remains blocked until that later PR records exact owner files, allowed files/crates, forbidden files/crates, validation envelope, evidence expectation, stop conditions, and acceptance criteria.

Required final position:

```text
PT-UI-RUNTIME-PLATFORM-001 investigation/design gate is complete.
Runtime implementation is still not started; open a separate implementation-planning PR with exact contract.
```

## Validation and evidence limits

Evidence classes used: `E2` connector metadata/file inspection, `E3` source/test inspection by path, and `E8` accepted architecture/workflow/planning authority.

Highest evidence class reached: `E8` for policy/direction, aligned with `E3` source inspection. No `E5` local command validation was available in this connector-only session.

Command validation: local command validation unavailable in this connector-only session. No cargo/docs validation was run.

CI validation: not inspected.

User-reported validation: not used for this investigation.

Manual inspection: GitHub connector file inspection of the files listed in the authority/source matrix.

Search limitation: broad GitHub code-search queries for `App::new`, `mount_ui`, and combined app/plugin terms timed out. The investigation therefore relies on direct path inspection of the named authority/source files and records repo-wide grep as unavailable.

## Authority/source matrix

| File/doc | Evidence class | What it proves | What it does not prove | Confidence | Decision impact |
|---|---:|---|---|---|---|
| `AGENTS.md` | E8 | Repository workflow requires reading docs first, preserving user work, and not claiming command validation unless run. | Does not prove current code behavior. | High | Requires connector-only validation wording. |
| `workspace/start-here.md` | E8 | Non-trivial work starts with workflow/lifecycle/gates and must report status explicitly. | Does not select UI architecture. | High | Confirms gate-first workflow. |
| `workspace/workflow-lifecycle.md` | E8 | Accepted direction is not implementation authorization; active implementation needs exact contract and gates. | Does not prove PR #74 content. | High | Blocks runtime implementation. |
| `workspace/authority-model.md` | E8 | Code/tests own current behavior; accepted docs own durable direction; planning owns active state. | Does not validate branch. | High | Determines conflict resolution. |
| `workspace/documentation-structure.md` | E8 | Investigation belongs in reports, design belongs in design, active-work stays short. | Does not prove code. | High | Determines artifact placement. |
| `workspace/complete-investigation-gate.md` | E8 | Required investigation checklist and matrices for platform/public API/domain-boundary work. | Does not decide target design. | High | Defines this report shape. |
| `workspace/complete-design-gate.md` | E8 | Required design evidence before implementation authorization. | Does not make PR #74 complete by itself. | High | Defines design hardening. |
| `workspace/complete-merge-readiness-gate.md` | E8 | Merge needs scope, evidence, validation, lifecycle, principles, decomposition. | Does not make this draft merge-ready. | High | Future merge review must use it. |
| `workspace/evidence-quality-taxonomy.md` | E8 | E2/E3/E8 classes and validation wording. | Does not provide command output. | High | Requires no local-validation claim. |
| `guidelines/programming-principles.md` | E8 | KISS, DRY, YAGNI, SOLID, SoC, no premature optimization, Demeter gate lens. | Does not prove compliance automatically. | High | Drives principle matrix. |
| `architecture/ui-framework-architecture.md` | E8 | UI source/program/proof/host/render owner model; render consumes derived output; hosts own mutation. | Does not authorize runtime implementation. | High | Canonical architecture target. |
| `design/active/ui-framework-app-integration-direction-review.md` | E8 | Accepted App/Plugin/ECS-hosted direction, `app.mount_ui(Screen)` target, proof must lower through UI source/program/story. | Does not implement API. | High | Establishes target ergonomics. |
| `reports/closeouts/pt-ui-framework-app-integration-002-closeout.md` | E8/E3 | PR #72 proof completed and intentionally did not add public `AppUiExt`, engine `UiPlugin`, render adapter, SDF, SpatialCanvas, `foundation/meta`, `domain/app_program`, or generic plugin framework. | Does not close runtime-platform gate. | High | Removes proof blocker, preserves non-goals. |
| `workspace/planning/active-work.md` | E8 | Current focus is PR #74 intake review, not implementation. | Does not contain full investigation. | High | Must stay short and point here. |
| `workspace/planning/roadmap.md` | E8 | PT-UI-RUNTIME-PLATFORM-001 is draft intake and implementation is blocked. | Does not define full gate evidence. | High | Needs alignment after this report. |
| `workspace/planning/production-tracks.md` | E8 | Runtime platform track candidate exists with staged future milestones. | Does not authorize milestone implementation. | High | Needs gate status alignment. |
| `workspace/planning/completed-work.md` | E8 | PR #72/Phase dependencies are completed; PR #74 remains follow-up. | Does not make PR #74 complete. | High | Confirms no completed-work entry for this slice. |
| `workspace/planning/decision-register.md` | E8 | Existing intake decision prefers engine-owned UiPlugin runtime over `ui_app_integration` growth and broad `ui_runtime_platform`. | Does not record this hardening yet. | High | Needs follow-up decision entry. |
| `engine/src/app/domain/app.rs` | E3 | `App` owns world/scheduler/runner and exposes plugin/resource/system/world accessors. | Does not contain `mount_ui` or public UI extension API in inspected content. | High | Valid host for future engine-side UI plugin surface. |
| `engine/src/app/domain/plugins.rs` | E3 | `IntoPlugins` supports single plugins, boxed plugins, vectors, and tuples. | Does not provide UI plugin behavior. | High | Confirms existing plugin composition shape. |
| `engine/src/plugin.rs` | E3 | `Plugin::build(&self, app: &mut App)` is the app plugin contract. | Does not define typed UI screen/action APIs. | High | UiPlugin should use existing plugin model. |
| `engine/src/plugins/mod.rs` | E3 | Engine plugin registry has render/scene/etc. but no `ui` module in inspected branch. | Does not prove no hidden UI elsewhere due search timeout. | Medium | Proposed `engine::plugins::ui` is future, not current. |
| `engine/src/prelude.rs` | E3 | Common prelude exports app/plugin/runtime/scene/time but not public UI plugin ergonomics. | Does not decide future exports. | Medium | Future re-export must be explicit. |
| `engine/Cargo.toml` | E3 | Engine already depends on selected UI crates: `ui_math`, `ui_render_data`, `ui_runtime`, `ui_text`, `ui_theme`. | Does not include `ui_surface`, `ui_hosts`, `ui_evaluator`, `ui_runtime_view` yet. | High | Adding engine-side UiPlugin dependencies is directional risk to plan. |
| `engine/src/plugins/render/plugin.rs` | E3 | `RenderPlugin` initializes UI frame resources and prepares UI during render prepare. | Does not own app UI semantics. | High | Render currently has UI-specific naming and collection. |
| `engine/src/plugins/render/features/ui/submission.rs` | E3 | Current render submission registry is `UiFrame`-specific and keyed by producer/surface with deterministic ordering. | Does not provide generic `SurfaceFrame` naming. | High | Genericization can be staged. |
| `engine/src/plugins/render/features/ui/resource.rs` | E3 | Prepared UI frame resource converts submissions to render contributions by surface. | Does not lower source/program/runtime. | High | Render can consume prepared frame data only. |
| `engine/src/plugins/render/runtime/ui_submission.rs` | E3 | Render currently collects scene overlay and debug metrics producers directly. | Does not support plugin-published UI runtime frames. | High | Scene/debug producers must move out of RenderPlugin long-term. |
| `engine/src/plugins/render/runtime/frame_prepare.rs` | E3 | Prepared frame applies UI contribution to each render surface. | Does not prove generic surface-frame submission. | High | Render boundary is already contribution-based. |
| `engine/src/plugins/render/runtime/frame_submit.rs` | E3 | Submit consumes prepared frame UI payload, shader/font atlas, and diagnostics in render backend path. | Does not define app/UI semantics. | Medium | Backend must remain renderer/projection owner only. |
| `engine/src/plugins/render/frame/contributions.rs` | E3 | Prepared frame has typed UI contribution insertion/access. | Does not genericize contribution payloads. | High | Supports staged render cleanup. |
| `domain/ui/ui_surface/Cargo.toml` | E3 | `ui_surface` has foundation dependencies only, no engine dependency. | Does not provide runtime registry resource. | High | Safe domain contract to consume from engine. |
| `domain/ui/ui_surface/src/lib.rs` | E3 | Surface contract modules exist for definition, mount, session, validation, etc. | Does not mount live app screens. | High | Reuse, do not replace. |
| `domain/ui/ui_surface/src/definition.rs` | E3 | Surface definitions and registry exist. | Does not bind to engine windows/surfaces. | High | Future UiPlugin can wrap/own engine resources. |
| `domain/ui/ui_surface/src/mount.rs` | E3 | Mounted surface instance and registry contracts exist. | Does not provide ECS runtime resource or App API. | High | Use for runtime mounted surface/session registry. |
| `domain/ui/ui_surface/src/session.rs` | E3 | Session scope and retention classes exist. | Does not persist or restore sessions. | High | Future runtime must choose retention policy. |
| `domain/ui/ui_hosts/Cargo.toml` | E3 | `ui_hosts` depends on evaluator/program/schema, not engine. | Does not own host mutation. | High | Engine can consume; domain must not invert. |
| `domain/ui/ui_hosts/src/lib.rs` | E3 | Host kinds, route maps, route resolution, host output receipts, `UiHost` contract exist. | Does not provide typed `UiActionHandler`. | High | Typed handler facade should lower to these contracts. |
| `domain/ui/ui_evaluator/Cargo.toml` | E3 | Evaluator depends on UI artifact/binding/program/schema/state crates. | Does not depend on engine. | High | Engine-side runtime may call it; domain must not call engine. |
| `domain/ui/ui_evaluator/src/lib.rs` | E3 | `UiEvaluator` and `UiOutput` are public crate outputs. | Does not prove live runtime wiring. | Medium | Required runtime output owner. |
| `domain/ui/ui_evaluator/src/output.rs` | E3 | `UiOutput` contains input/control/layout/style/state/binding/interaction/visual/accessibility/inspection/diagnostics passes. | Does not publish frames to render. | High | Future runtime report/proof envelope should reference this. |
| `domain/ui/ui_runtime_view/Cargo.toml` | E3 | Runtime view depends on UI artifacts/program/schema, not engine. | Does not evaluate live events. | High | Read model remains domain-side. |
| `domain/ui/ui_runtime_view/src/lib.rs` | E3 | Artifact-backed `UiRuntimeView` and diagnostics exist. | Does not mount app screens. | High | Runtime design can reuse it for inspection/proof. |
| `domain/ui/ui_render_data/src/frame/mod.rs` | E3 | Frame-level UI render contracts are centralized and re-exported. | Does not define generic `SurfaceFrame`. | High | Staged naming compatibility needed. |
| `domain/ui/ui_render_data/src/frame/ui_frame.rs` | E3 | `UiFrame` is surfaces plus empty checks. | Does not prove renderer-generic frame naming. | High | Current payload can back first slice. |
| `domain/ui/ui_render_data/src/frame/output_evidence.rs` | E3 | Renderer-neutral UI frame output evidence and summaries exist. | Does not prove live engine publication. | High | Future Counter live proof should emit render evidence. |
| `domain/ui/ui_app_integration/Cargo.toml` | E3 | Proof crate depends on domain UI + ECS but not engine. | Does not authorize making it final framework. | High | Preserve as proof-local dependency boundary. |
| `domain/ui/ui_app_integration/src/lib.rs` | E3 | Crate explicitly says proof bridge and not generic app framework. | Does not implement runtime platform. | High | Do not grow into final framework. |
| `domain/ui/ui_app_integration/src/source.rs` | E3 | Counter/Win source helpers build `UiNodeDefinition` source records. | Does not define final `IntoUi`. | High | Source facade can draw from proof shape. |
| `domain/ui/ui_app_integration/src/bridge.rs` | E3 | Route-event to action resolution and fail-closed diagnostics exist in proof-local form. | Does not define public typed handler API. | High | Future action handler should preserve no-bypass behavior. |
| `domain/ui/ui_app_integration/src/host.rs` | E3 | ECS-backed Counter host owns mutation in proof. | Does not generalize host-owned mutation. | High | Confirms host-owned mutation rule. |
| `domain/ui/ui_app_integration/src/proof.rs` | E3 | Proof forms source through `ui_program_lowering`, builds packets, resolves actions, mutates host, and reports output. | Does not run inside engine App/Plugin. | High | Counter live proof should reproduce this through engine runtime. |
| `domain/ui/ui_app_integration/tests/counter_ui_story_proof.rs` | E3 | Positive proof asserts route/output/source/action/mutation facts. | Does not prove tests passed in this session. | Medium | Future implementation needs equivalent live proof. |
| `domain/ui/ui_app_integration/tests/counter_ui_story_fail_closed.rs` | E3 | Fail-closed cases assert no mutation on rejected route/schema/capability/payload paths. | Does not prove tests passed in this session. | Medium | Future runtime must preserve fail-closed action dispatch. |
| PR #74 metadata | E2 | PR is open draft docs-only intake at reviewed head `9551405d77937824b9ef6459abbdd870e39558c1`. | Does not prove branch validation. | High | Update same PR, keep draft unless separately reviewed. |

## Current-state inventory

| Area | Current state | Design consequence |
|---|---|---|
| Engine App/Plugin composition | `App` owns ECS `World`, scheduler, runner, mode, title, control flow, and exposes `add_plugin`, `add_plugins`, `add_systems`, `init_resource`, `insert_resource`, `world`, and `world_mut`. `Plugin` is `build(&self, app: &mut App)`. | `UiPlugin` should be an engine plugin that uses existing app composition rather than inventing a second plugin framework. |
| Engine prelude/export behavior | The prelude exports `App`, `Plugin`, runtime, state, ECS basics, scene/time/replay/input/net basics, but no public UI plugin ergonomics. | Future public exports must be intentional and part of implementation planning. |
| Current RenderPlugin UI-frame resources/systems | `RenderPlugin` initializes `PreparedUiFrameResource`, `UiFrameSubmissionRegistryResource`, `UiFontAtlasResource`, and runs UI submission collection plus preparation before frame prepare. | Render already has an intake point for frame submissions, but the naming and hardcoded producer collection are UI-specific. |
| Current hardcoded scene/debug UI producer collection | `collect_runtime_ui_frame_submissions_system` directly reads scene overlay UI and debug overlay state, using hardcoded producer IDs. | Long-term producer collection should move to producer plugins or compatibility producer modules outside core RenderPlugin semantics. |
| `ui_surface` contracts | Definitions, mounted instances, host instance IDs, world-space prompt mount, mounted registry, session scope, and retention classes exist without engine dependency. | Use these contracts for runtime mount/session registry instead of creating a duplicate runtime-platform surface model. |
| `ui_hosts` route/output contracts | Host kind, route map versioning, route mapping, route resolution, host route diagnostics, `UiHost`, and output receipts exist. | Typed action handlers should lower to host route/action semantics rather than manual route maps as the normal app path. |
| `ui_evaluator` `UiEvaluator` / `UiOutput` | The evaluator exposes multi-pass output with diagnostics. | Live runtime must produce/evaluate output and preserve diagnostic/report shape. |
| `ui_runtime_view` artifact-backed read model | `UiRuntimeView` builds control/read-model diagnostics from `UiRuntimeArtifact`. | Runtime inspection/proof should reuse this read model instead of inventing engine-owned UI semantics. |
| `ui_render_data` `UiFrame` / evidence payload | `UiFrame` remains the renderer-facing UI frame payload and output evidence summarizes frames/primitives. | First implementation can publish current `UiFrame`; generic `SurfaceFrame` naming should be staged unless design accepts immediate migration. |
| `ui_app_integration` proof-local boundary | Proof crate builds source, forms program route facts, creates packets, resolves action, mutates proof-local ECS host, and reports fail-closed behavior. | Preserve as evidence/proof vocabulary; do not grow it into the public framework. |
| Dependency direction | Domain UI crates inspected do not depend on engine. Engine already depends on some UI crates and can add selected UI contract dependencies if explicitly planned. | Target direction is engine -> domain/ui contracts; never domain/ui -> engine. |

## Gap matrix

| Capability | Current status | Missing contract | Owner | Dependency risk | Proof/validation needed |
|---|---|---|---|---|---|
| `UiPlugin` | Missing. No inspected `engine::plugins::ui` module. | Engine plugin skeleton, schedule/resources, lifecycle, diagnostics. | Future `engine::plugins::ui`. | Adding too much UI semantic ownership to engine. | Plugin installs resources/systems without bypassing UI source/program/evaluator. |
| `AppUiExt` / `app.mount_ui` | Missing. Target spelling only. | Public extension trait or inherent method decision, re-export plan, failure diagnostics. | Future `engine::plugins::ui::app_ext` / `mount`. | Premature public API freeze. | Compile proof for `app.mount_ui(CounterScreen)` and advanced `app.ui().mount`. |
| `UiScreen` | Missing. Proof-local screen descriptors exist only in `ui_app_integration`. | Typed screen identity, state snapshot input, source lowering rule, diagnostics. | Future `engine::plugins::ui::screen` facade over `ui_definition`. | Making engine own source semantics. | Screen lowers to `UiNodeDefinition`/`UiProgram` with source maps. |
| `IntoUi` | Missing. Proof-local builder directly creates nodes. | Typed source builder trait and conversion into `ui_definition` source records. | Future `engine::plugins::ui::source`. | Bypassing `ui_definition`. | Source map and program formation evidence. |
| `UiActionHandler` / `TryUiActionHandler` | Missing. Proof-local route bridge exists. | Typed action mapping, capability/schema checks, fail-closed mutation API. | Future `engine::plugins::ui::action` and `host`. | Letting generic controls mutate state directly. | Unknown route/schema/capability/payload failures produce no mutation. |
| Engine-side host adapter | Missing. `ui_hosts` contracts exist; proof host is crate-local. | Adapter from typed app handlers to host route/output receipts. | Future `engine::plugins::ui::host`. | Moving app/product semantics into domain/ui. | Host-owned mutation report and diagnostics. |
| Runtime session registry | Missing as engine resource. `ui_surface` has domain contracts. | ECS resource around mounted surfaces/sessions with deterministic IDs and generation. | Future `engine::plugins::ui::resources` using `ui_surface`. | Duplicate surface model or engine-specific leak into domain crate. | Mount/unmount/session generation tests. |
| Source lowering facade | Missing as public runtime path. | `UiScreen`/`IntoUi` to `UiNodeDefinition`/`UiProgram` bridge. | Future `engine::plugins::ui::source`. | Bypassing validation/normalization. | Formation report with source-map evidence. |
| Event/action dispatch queue | Missing. Proof calls bridge directly. | Queue/resource for `UiEventPacket`, resolved actions, rejected actions, diagnostics. | Future `engine::plugins::ui::events` / `action`. | Direct callback bypass. | Dispatch ordering and fail-closed tests. |
| Render publication system | Missing as UiPlugin producer. Render registry exists. | System converting runtime/evaluator output to frame submissions. | Future `engine::plugins::ui::render_publish`. | Render owning UI semantics. | Prepared frame contains plugin-published frame without RenderPlugin querying UI runtime. |
| Generic `SurfaceFrame` naming / compatibility strategy | Missing. Current names are `UiFrame*`. | Staged alias/rename policy and ownership. | Render + `ui_render_data` design/implementation planning. | Premature broad migration. | Diff-scope and compatibility tests if renamed. |
| Legacy scene/debug overlay producer split | Current collection is inside render runtime. | Compatibility producer modules outside core RenderPlugin long-term. | Future `engine::plugins::ui::compat_scene_overlay` / `compat_debug_overlay`, or render compatibility owner during transition. | Keeping RenderPlugin as UI producer owner. | Scene/debug overlay frames still publish through registry after split. |
| Runtime report/proof envelope | Missing for live engine path. Proof-local reports exist. | Runtime report with source/program/evaluator/frame/host/action facts. | Future `engine::plugins::ui::report`. | Claiming visible UI success without upstream evidence. | Counter live app proof report plus render output evidence. |

## Alternatives matrix

| Alternative | Benefit | Cost | Dependency impact | Ergonomics impact | Proof impact | Recommendation |
|---|---|---|---|---|---|---|
| A. Grow `ui_app_integration` into final framework | Reuses proof code directly. | Violates proof-local crate purpose and freezes ECS-specific proof APIs. | Risks `domain/ui` becoming app/framework host owner. | Likely exposes proof vocabulary to app authors. | Mixes proof and runtime ownership. | Reject. Keep proof-local. |
| B. Add broad `domain/ui/ui_runtime_platform` crate | Centralizes runtime platform vocabulary. | Duplicates `ui_surface`, `ui_hosts`, `ui_evaluator`, `ui_runtime_view`, and render data. | Risk of domain crate depending outward or becoming god crate. | Adds ceremony before app author path. | Harder to prove owner boundaries. | Reject for first runtime platform. |
| C. Add engine-owned `UiPlugin` runtime layer and reuse domain UI crates | Matches App/Plugin host surface and keeps domain contracts reusable. | Requires careful dependency planning and public API control. | Engine may depend on selected domain UI crates; domain stays engine-free. | Best common path: `app.mount_ui(Screen)`. | Can reproduce proof through real engine runtime. | Accept as target. |
| D. Keep `RenderPlugin` UI-specific | Minimal near-term changes. | Preserves renderer knowledge of UI producers and names. | Render remains coupled to UI semantics. | No app ergonomics improvement. | Cannot prove generic producer model. | Reject as long-term owner; tolerate staged compatibility. |
| E. Genericize render toward surface-frame submissions | Moves render to producer-agnostic consumption. | Requires staged naming and adapter design. | Render consumes generic data; producers own semantics. | Invisible to common app authors. | Enables UiPlugin and other producers. | Accept staged. |
| F. Force users to write host adapters | Avoids public API design. | Bad common-path ergonomics; repeats manual route maps. | Pushes internal contracts to app authors. | High friction and error risk. | Proof becomes manual wiring proof only. | Reject for common path; allow advanced/internal escape hatch. |
| G. Typed `UiScreen` + `UiActionHandler` default path | Clear authoring and action model. | Requires careful trait shape and diagnostics. | Engine facade over domain contracts. | Best primary UX. | Can prove no-bypass route/action flow. | Accept. |
| H. Keep `AppUiExt`-only slice separate | Smaller public API surface. | Freezes sugar before runtime/session/render contracts. | Risk of API churn. | Looks ergonomic but may be hollow. | Does not prove live runtime. | Reject as immediate standalone slice. |
| I. Absorb `AppUiExt` into broader runtime platform track | Aligns ergonomics with runtime/session/render proof. | Larger design gate required. | Cleaner dependency sequencing. | Keeps primary API but ties it to real behavior. | Supports Counter live proof. | Accept. |

## Dependency-direction review

Target dependency direction:

```text
engine/src/plugins/ui may depend on domain/ui contracts.
domain/ui crates must not depend on engine.
render may consume frame/submission data but must not own UI semantics.
ui_app_integration remains proof-local.
```

Current evidence supports this direction. Inspected domain UI manifests do not add engine dependencies. `engine/Cargo.toml` already depends on a limited set of UI crates; the future implementation-planning PR must explicitly add only the additional UI crates required for `ui_surface`, `ui_hosts`, `ui_evaluator`, and `ui_runtime_view` consumption, or record why each is deferred.

Risk: adding engine dependencies to existing UI crates would invert ownership and must stop implementation. If a typed screen/action API needs engine `App` or ECS details, that API belongs in `engine::plugins::ui`, not in `domain/ui` crates.

## Render-boundary review

Should `RenderPlugin` know UI directly?

Answer: long-term no. `RenderPlugin` may know renderable frame/submission payloads, routes/layers/orders needed for drawing, font atlas/render resources, shader handles, render surfaces, and diagnostics. It must not own UI source identity, `UiProgram` semantics, host route policy, action dispatch, app mutation, story truth, or session semantics.

Acceptable render knowledge:

```text
producer id
render surface id
route/layer/order as draw ordering facts
frame payload / primitive data
font atlas and shader resources
diagnostics about render preparation/submission
```

Forbidden render knowledge:

```text
UiScreen identity as app source truth
IntoUi lowering
UiActionHandler mapping
host mutation policy
route authorization semantics
UiStory mount eligibility
SDF/world-space semantic ownership
app/editor/game/product state mutation
```

Should current `UiFrameSubmission` names be renamed immediately or staged?

Answer: staged. Immediate rename to `SurfaceFrame` is optional and not required in the first implementation slice. The first slice may publish current `UiFrame` through the existing registry if the design records the compatibility boundary and stop condition. Genericization should be its own planned slice when render ownership, aliases, tests, and downstream usage are known.

Where should scene/debug overlay producer collection move?

Answer: long-term out of core RenderPlugin collection. Scene/debug overlays should become compatibility producers that publish submissions through the same producer registry as `UiPlugin`. During the first implementation slice, current collection may remain only as a compatibility path if the implementation records its removal condition and does not add new UI semantics to RenderPlugin.

## Ergonomics review

| Authoring path | Common-path status | Review |
|---|---|---|
| `app.mount_ui(CounterScreen)` | Primary API | Best default: smallest app-author path and aligns with target direction. |
| `app.ui().mount(CounterScreen)` | Advanced API | Useful when configuration, diagnostics, or grouping is needed. |
| `app.ui().surface(...).source(...).host(...).mount()` | Internal/advanced builder | Acceptable for tests and advanced platform wiring, not normal app authoring. |
| `UiSurfaceFactory` | Forbidden common path | Too factory/host-map oriented; exposes internal surface wiring. |
| Manual host adapter | Forbidden common path | Useful only for advanced/custom host implementations. |
| Typed `UiScreen` + `UiActionHandler` | Default model | Required to keep action handling typed while preserving host-owned mutation. |

Conclusion:

```text
Primary API: app.mount_ui(CounterScreen)
Advanced API: app.ui().mount(CounterScreen)
Internal/advanced builder only: explicit surface/source/host builder
Forbidden common path: Factory/manual host adapter/manual route maps
```

## Confidence matrix

| Finding | Confidence | Reason | Missing evidence to improve confidence |
|---|---|---|---|
| PR #74 is docs-only intake and not implementation authorization. | High | PR metadata and planning/design files agree. | None for planning status. |
| Engine-owned `UiPlugin` layer is the best target owner. | High | Architecture, direction review, closeout, and source dependency direction align. | Exact future implementation contract. |
| `ui_app_integration` must remain proof-local. | High | Crate docs and closeout explicitly say so. | None for current boundary. |
| Render should consume producer submissions and not own UI semantics. | High | Architecture spine and render source support this separation. | Future render genericization design. |
| Immediate `SurfaceFrame` rename is not required for first slice. | Medium | Current code is `UiFrame`-based and can serve compatibility; exact migration cost not measured. | Repo-wide search and future diff plan. |
| Repo-wide symbol absence such as no `mount_ui` anywhere. | Medium | Direct inspected files do not show it, but broad code search timed out. | Local grep or successful code search. |

## Blockers before implementation

Runtime implementation remains blocked until a separate implementation-planning PR records all of these:

```text
exact module/file contract
exact allowed files/crates
exact forbidden files/crates
validation envelope
proof/evidence expectation
feature support matrix
future-use-case pressure matrix
hierarchy/composition matrix
ergonomics/usability matrix
principle compliance matrix
stop conditions
acceptance criteria
```

Stop immediately if implementation requires any of these in the PR #74 docs-only branch:

```text
runtime Rust implementation
engine UiPlugin code
public AppUiExt code
app.mount_ui implementation
UiScreen / IntoUi implementation
UiActionHandler implementation
render adapter code
SurfaceFrame type migration code
SDF/world-space/SpatialCanvas implementation
foundation/meta
domain/app_program resurrection
generic plugin framework
changing PR #72/PR #75 closeout truth
making RenderPlugin own UI semantics
making ECS own durable UI semantics
making ui_app_integration the final framework
```

## Recommended next gate

Next gate: complete design gate hardening in `docs-site/src/content/docs/design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md`, followed by a separate implementation-planning PR.

This investigation does not authorize code.