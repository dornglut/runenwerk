---
title: Runenwerk UI Story Driven Golden Workflow Design
description: Productization design for making Runenwerk UI authoring, preview, validation, inspection, proof, and mount eligibility flow through one story-driven workflow without renderer-owned UI truth.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-16
related_designs:
  - ./ui-runtime-rendering-pipeline-roadmap.md
  - ./ui-program-architecture.md
  - ./ui-program-contract-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../domain/ui/story-acceptance-and-review-checklist.md
  - ../../workspace/production-track-planning-model.md
  - ../../workspace/planning-and-implementation-workflow.md
---

# Runenwerk UI Story Driven Golden Workflow Design

## Status

This is an active UI productization design. It defines the canonical UI-only
direction for authoring, preview, validation, inspection, proof, and mount
eligibility through `UiStory`.

This document does not authorize implementation, crate creation, generated
planning-document edits, or runtime behavior changes by itself. Implementation
still requires the normal accepted design, WR roadmap, production-track,
validation, and closeout gates.

## Decision

Runenwerk UI authoring, validation, preview, inspection, testing, and mount
eligibility must be centered on one first-class product unit: `UiStory`.

A `UiStory` is the canonical developer-facing proof envelope for one UI unit. A
story may describe a primitive control, a compound component, a full surface,
an interaction state, a responsive viewport state, an accessibility state, or
an intentionally failing diagnostic fixture.

The final workflow is:

```text
UiStory manifest
  -> authored source load
  -> authored source parse
  -> definition validation
  -> definition normalization
  -> control package/schema validation
  -> UiProgramFormationReport
  -> UiProgram
  -> UiCompilerReport
  -> UiRuntimeArtifact
  -> UiRuntimeViewReport
  -> binding/host data report
  -> host route report
  -> layout/style/text/accessibility reports
  -> interaction trace report
  -> backend-neutral render primitive report
  -> render data report
  -> static mount report
  -> gallery preview
  -> inspection report
  -> mount eligibility verdict
```

A UI surface may mount into an editor, game, headless, or world-space host only
after its story has passed every required stage for that host profile.

## Existing Repository Truth

This design productizes the current UI direction. It does not replace it.

Current evidence anchors:

- `docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md`
  already defines the durable target from authored UI plus control package
  snapshot, host data, theme tokens, and viewport constraints into formation,
  compiler, runtime artifact, runtime view, reports, render primitives, backend
  adapter, and visible UI.
- The same roadmap already forbids rendering from authored `.ron` directly,
  inventing package truth, inferring control semantics from strings, and
  bypassing formation/compiler/evaluator diagnostics.
- `domain/ui/ui_definition` owns authored UI definitions, validation,
  normalization, retained formation, template references, repeaters, embeds,
  menus, availability products, and stable authored IDs.
- `domain/ui/ui_program_lowering` exposes
  `form_ui_program_report_from_node_with_registry_snapshot(...)`.
- `domain/ui/ui_compiler` exposes `UiCompiler::compile_report(...)`.
- `domain/ui/ui_runtime_view` is the canonical runtime read model over compiled
  artifact tables.
- `domain/ui/ui_controls` models package-backed controls including button,
  label, inspector field, color picker, action prompt, list view, tree view,
  and table view.
- `domain/ui/ui_binding` models host data, binding snapshots, dirty
  propagation, authorization, and diagnostics.
- `domain/ui/ui_hosts` models editor, game, world-space, and headless host
  kinds plus route-to-command mapping.
- `domain/ui/ui_render_primitives`, `domain/ui/ui_headless_render_data`, and
  `domain/ui/ui_static_mount` form the renderer-facing proof path.
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs` currently hosts a
  hardcoded, button-specific gallery path. That path is useful first-slice proof
  but must not remain the final ergonomic architecture.

## Problem

Runenwerk has enough UI substrate for a serious pipeline, but the current
authoring/product workflow is still too implicit.

Current weaknesses to remove:

1. Gallery fixtures are hardcoded in app-local Rust code.
2. Gallery execution is button-specific instead of story-generic.
3. Bare node fixtures and full authored templates do not share one
   user-facing story envelope.
4. Validation, compilation, binding, route, layout, style, text,
   accessibility, interaction, render, and static-mount diagnostics are not
   presented as one inspectable run report.
5. Mounting policy is not yet expressed as a story-derived eligibility
   contract.
6. Advanced platform components such as graph canvas, timeline, rich text,
   drag/drop, world-space UI, effects, and visual builder do not yet have a
   reusable story matrix gate.

The result is architectural correctness without enough product ergonomics.

## Goals

The final UI workflow must make normal UI creation pleasant and advanced UI
platform work disciplined.

Goals:

- create one canonical `UiStory` workflow for components, surfaces, states,
  failure cases, interactions, and host profiles;
- replace hardcoded gallery fixture lists with manifest-driven discovery;
- make `UiStoryRunReport` the single inspection object from authored source to
  mounted frame;
- make the gallery an inspector over story reports, not a hand-authored preview
  shell;
- enforce story-first mount eligibility;
- require story matrices for every reusable platform component;
- preserve renderer ignorance of UI/product semantics;
- preserve app/editor/game ownership of host commands and domain mutation;
- preserve control package/schema truth as explicit inputs or snapshot
  artifacts;
- provide a clean cutover path with no permanent compatibility layer.

## Non-Goals

This design does not authorize:

- creating new crates without accepted design and WR/production authority;
- editing generated production documents directly;
- adding a parallel visual-builder file format;
- adding renderer-owned button, graph, timeline, text editor, or world-space UI
  semantics;
- using debug overlay behavior as production UI proof;
- making editor/game/domain state mutable from generic UI code;
- treating a visible result as proof when upstream reports failed;
- preserving a permanent hardcoded button gallery path beside the story path.

## Core Concept: UiStory

`UiStory` is the stable contract for one UI proof case.

A story describes:

- what authored source to load;
- whether the source is a bare `UiNodeDefinition` or a full
  `AuthoredUiTemplate`;
- which control package snapshot to use;
- which host kind and route map to validate against;
- which theme/profile to resolve;
- which viewport matrix to test;
- which host data to evaluate;
- which input traces to replay;
- which diagnostics are expected;
- whether the story is expected to pass or intentionally fail;
- whether the story is mount-eligible.

Story categories:

| Category | Purpose |
|---|---|
| `control` | primitive or package-backed control proof |
| `component` | compound reusable component proof |
| `surface` | complete authored panel/screen/surface proof |
| `state` | hover, pressed, focused, disabled, selected, invalid, loading, empty |
| `interaction` | deterministic pointer/keyboard/gamepad trace |
| `accessibility` | accessibility-tree and role/label proof |
| `layout` | responsive, clipping, overflow, split, grid, scroll proof |
| `failure` | intentionally invalid fixture with exact diagnostic expectation |
| `host` | editor/game/world-space/headless route and binding proof |

## Proposed Crate: `domain/ui/ui_story`

Preferred long-term owner:

```text
domain/ui/ui_story
```

This crate is an orchestrator. It does not replace existing UI crates.

Public module target:

```text
domain/ui/ui_story/src/lib.rs
domain/ui/ui_story/src/manifest.rs
domain/ui/ui_story/src/source.rs
domain/ui/ui_story/src/registry.rs
domain/ui/ui_story/src/runner.rs
domain/ui/ui_story/src/report.rs
domain/ui/ui_story/src/diagnostics.rs
domain/ui/ui_story/src/states.rs
domain/ui/ui_story/src/interaction.rs
domain/ui/ui_story/src/mount.rs
```

Target public types:

| Type | Purpose |
|---|---|
| `UiStoryManifest` | parsed story manifest |
| `UiStoryId` | stable story identifier |
| `UiStorySourceKind` | `node` or `template` |
| `UiStoryHostProfile` | host kind, capability policy, route policy |
| `UiStoryThemeProfile` | theme id/profile and token policy |
| `UiStoryViewportProfile` | width, height, scale, target profile |
| `UiStoryStateProfile` | default/hover/focus/disabled/etc. |
| `UiStoryInputTrace` | deterministic input replay |
| `UiStoryRunRequest` | one runner invocation |
| `UiStoryRunReport` | full pipeline report |
| `UiStoryStageReport` | one stage report |
| `UiStoryDiagnostic` | unified diagnostic envelope |
| `UiStoryVerdict` | pass/fail/skipped with first failing stage |
| `UiStoryMountEligibility` | explicit mount decision |

Crate creation remains subject to normal repository authority. If an accepted
future slice designates an existing crate as the story runner owner, that crate
must expose the same public story contract and preserve the ownership rules in
this document.

## Story Manifest File Layout

Stories live in assets, not Rust constants.

```text
assets/ui_gallery/stories/
  controls/
    button/
      basic.story.ron
      selected.story.ron
      disabled.story.ron
      missing_label.failure.story.ron
  surfaces/
    editor_toolbar.story.ron
    inspector.story.ron
  platform/
    graph_canvas/
    timeline/
    rich_text/
    drag_drop/
    world_space/
    effects/
```

The current hardcoded gallery fixture array must be removed during cutover.

Required manifest fields:

```text
story_id
category
title
source_kind
source_path
source_id
program_id
control_package
host_profile
viewport_matrix
theme_profile
expected
mount_policy
```

Optional manifest fields:

```text
host_data
route_map
state_profiles
input_traces
snapshot_policy
diagnostic_expectations
accessibility_policy
performance_budget
localization_profile
```

Every story must explicitly declare whether it is expected to pass or fail.
Failure stories are first-class because they prove diagnostic quality.

## Example Control Story

```ron
(
    story_id: "ui.controls.button.basic",
    category: "controls/button",
    title: "Button / Basic",
    source_kind: "node",
    source_path: "assets/ui_gallery/button/basic.ron",
    source_id: "assets.ui_gallery.button.basic",
    program_id: "ui.gallery.button.basic",
    control_package: "runenwerk.ui.controls@1",
    host_profile: (
        kind: "headless",
        route_policy: "visual_unmapped_allowed",
    ),
    viewport_matrix: [
        (id: "default", width: 240, height: 96, scale: 1.0),
    ],
    theme_profile: "editor.dark",
    expected: (
        verdict: "pass",
        require_definition_validation: true,
        require_formation: true,
        require_compiler: true,
        require_runtime_view: true,
        require_accessibility: true,
        require_render_primitives: true,
        require_static_mount: true,
    ),
    mount_policy: "gallery_only",
)
```

## Example Surface Story

```ron
(
    story_id: "ui.surfaces.editor.settings_panel.default",
    category: "surfaces/editor",
    title: "Editor Settings Panel / Default",
    source_kind: "template",
    source_path: "assets/editor/ui/surfaces/settings_panel.ron",
    source_id: "assets.editor.ui.surfaces.settings_panel",
    program_id: "editor.settings_panel",
    control_package: "runenwerk.ui.controls@1",
    host_profile: (
        kind: "editor",
        route_policy: "all_routes_mapped",
    ),
    viewport_matrix: [
        (id: "compact", width: 360, height: 560, scale: 1.0),
        (id: "default", width: 420, height: 640, scale: 1.0),
        (id: "wide", width: 760, height: 640, scale: 1.0),
    ],
    theme_profile: "editor.dark",
    host_data: {
        "settings.graphics.vsync": Bool(true),
        "settings.audio.master_volume": Number(0.75),
    },
    route_map: {
        "settings.graphics.set_vsync": "editor.settings.graphics.set_vsync",
        "settings.audio.set_master_volume": "editor.settings.audio.set_master_volume",
        "settings.apply": "editor.settings.apply",
    },
    expected: (
        verdict: "pass",
        require_accessibility: true,
        require_interaction_traces: true,
        require_static_mount: true,
    ),
    mount_policy: "eligible_when_passed",
)
```

## UiStoryRunReport

`UiStoryRunReport` is the one object the CLI, gallery, tests, docs, and mount
eligibility gate inspect.

Required report sections:

| Section | Required owner |
|---|---|
| `manifest` | `ui_story` |
| `source_load` | `ui_story::source` |
| `source_parse` | `ui_story::source` |
| `definition_validation` | `ui_definition` |
| `definition_normalization` | `ui_definition` |
| `schema_validation` | `ui_schema` and `ui_controls` |
| `control_package` | `ui_controls` |
| `program_formation` | `ui_program_lowering` |
| `compiler` | `ui_compiler` |
| `runtime_artifact` | `ui_artifacts` |
| `runtime_view` | `ui_runtime_view` |
| `binding` | `ui_binding` |
| `host_routes` | `ui_hosts` |
| `layout` | `ui_layout` / `ui_geometry` |
| `style` | `ui_theme` |
| `text` | `ui_text` |
| `accessibility` | `ui_accessibility` |
| `interaction` | `ui_runtime` / `ui_input` |
| `render_primitives` | `ui_render_primitives` |
| `render_data` | `ui_headless_render_data` |
| `static_mount` | `ui_static_mount` |
| `preview_frame` | `ui_render_data` |
| `mount_eligibility` | `ui_story::mount` |
| `verdict` | `ui_story` |

The report must carry stage timings, source maps where available, stable
diagnostic codes, and first-failing-stage information.

## Mount Eligibility

A surface is mount-eligible only when:

1. the story is not an expected-failure story;
2. the source loads and parses;
3. definition validation and normalization pass;
4. control package/schema validation passes;
5. program formation passes;
6. compilation passes;
7. runtime artifact/source maps are valid;
8. runtime view passes;
9. host data/bindings pass;
10. routes are mapped for the target host;
11. layout, style, text, accessibility, and interaction requirements pass;
12. render primitive report passes;
13. render-data report passes;
14. static mount passes;
15. every required viewport/theme/state/input trace passes.

Normal product rule:

```text
story first
mount second
```

Forbidden rule:

```text
mount first
debug visually later
```

## Gallery Product Design

The gallery is the UI development product.

Required panels:

| Panel | Purpose |
|---|---|
| Story Browser | discover and select all stories |
| Preview | rendered selected story |
| Manifest | parsed story manifest |
| Source | authored source with source-map links |
| Pipeline | stage timeline and pass/fail states |
| Diagnostics | ordered diagnostics with exact owner/stage |
| Program Graph | `UiProgram` rows and source maps |
| Artifact Tables | compiled runtime artifact tables |
| Runtime View | canonical derived control/surface view |
| Bindings | host data, binding snapshots, dirty/authorization status |
| Routes | route proposals and host route map |
| Layout | resolved boxes, constraints, clipping, scroll ownership |
| Style | semantic tokens and resolved raw values |
| Text | layout requests/results, glyph runs, overflow, localization |
| Accessibility | role/label/focus tree and failures |
| Interaction Trace | pointer/keyboard/gamepad replay |
| Render Primitives | backend-neutral primitive list |
| Render Data | frame/surface/layer/primitive output |
| Static Mount | final visual proof gate |
| Snapshot | visual snapshot and diff status |

## CLI Product Design

The CLI and gallery must share `UiStoryRunner`.

Required commands:

```text
tools/ui discover
tools/ui run <story_id>
tools/ui run-all
tools/ui validate <path>
tools/ui inspect <story_id>
tools/ui explain <diagnostic_code>
tools/ui snapshot <story_id>
tools/ui mount-check <story_id>
tools/ui new-story
tools/ui new-component-story
tools/ui new-surface-story
```

No duplicate CLI-specific runner is allowed.

## Advanced UI Platform Capabilities

The following wanted features must be platform tracks, not ad-hoc editor
features:

| Feature | Correct platform form |
|---|---|
| node graph editor | `GraphCanvas` package-backed platform component |
| animation timeline | `Timeline` package-backed platform component |
| rich text/code editor | `RichText` / `CodeEditor` text platform |
| advanced drag/drop | `ui_drag_drop` interaction platform |
| visual UI builder | `apps/runenwerk_ui_builder` editing authored source and stories |
| shader-driven effects | `ui_effects` plus renderer contract |
| world-space 3D UI | `ui_world_space_host` host/platform path |

These capabilities must use the same story workflow and story matrix gates.

## Cutover Rule

Clean cutover only.

The current hardcoded button gallery path may be used as the source evidence for
the first slice, but it must not remain as a parallel production path.

The first implementation slice must remove or fully convert:

```text
apps/runenwerk_editor/src/runtime/ui_gallery.rs::UI_GALLERY_FIXTURES
apps/runenwerk_editor/src/runtime/ui_gallery.rs::load_fixture_node
apps/runenwerk_editor/src/runtime/ui_gallery.rs::compile_fixture_button_report
```

The final gallery resource must consume story reports, not button-specific
reports.

## First Production Milestone To Record

`PM-UI-STORY-001 - Story Workflow Authority And Track Activation`

Outcome:

- activate `PT-UI-STORY-PLATFORM` as the single story-first UI production
  track;
- defer the standalone static gallery rendering path as temporary evidence;
- record `UiStoryManifest`, `UiStoryRegistry`, `UiStoryRunner`,
  `UiStoryRunReport`, and `UiStoryMountEligibility` as future public
  contracts;
- keep runtime code, crate creation, gallery migration, and product mounting
  forbidden until the owning WR and production plan exist;
- sequence runtime rendering proof after story runner/report creation.

Target files and functions for the later implementation contract:

- `apps/runenwerk_editor/src/runtime/ui_gallery.rs::UI_GALLERY_FIXTURES`
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs::load_fixture_node`
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs::compile_fixture_button_report`
- `apps/runenwerk_editor/src/runtime/ui_gallery.rs::submit_ui_gallery_frame_system`
- `domain/ui/ui_story/src/runner.rs::UiStoryRunner::run_story`
- `domain/ui/ui_story/src/report.rs::UiStoryRunReport`
- `domain/ui/ui_story/src/registry.rs::UiStoryRegistry::discover`
- `domain/ui/ui_story/src/mount.rs::UiStoryMountEligibility::from_report`

The milestone is intentionally recorded here as design intake only. It must not
be implemented until the repository roadmap and production-track gates authorize
it.

## Validation Expectations

Design/docs validation:

```text
task docs:validate
task production:validate
task production:check
task planning:validate
```

Implementation validation must add focused tests before broad CI.

Minimum first-slice envelope:

```text
cargo fmt --all
cargo test -p ui_definition
cargo test -p ui_schema
cargo test -p ui_program_lowering
cargo test -p ui_compiler
cargo test -p ui_artifacts
cargo test -p ui_binding
cargo test -p ui_accessibility
cargo test -p ui_runtime_view
cargo test -p ui_render_primitives
cargo test -p ui_headless_render_data
cargo test -p ui_static_mount
cargo test -p ui_story
```

If a crate does not yet exist, the implementation slice must either create it
through accepted authority or keep the validation command out of the first slice
and record why.

## Acceptance Criteria For This Design

This design is intake-ready when:

- it is added as an active design source;
- the active design index links it;
- it does not claim implementation authority;
- it cites current pipeline ownership and current gaps;
- it defines `UiStory` as the golden workflow unit;
- it defines the new `ui_story` orchestrator crate target;
- it defines gallery and CLI product requirements;
- it defines clean cutover requirements;
- it defines mount eligibility;
- it defines advanced UI platform features as story-gated platform tracks;
- it names validation expectations.

## Stop Conditions

Stop and redesign if any implementation:

- renders directly from authored `.ron`;
- keeps hardcoded gallery fixtures as a production path;
- adds a button/control-specific gallery pipeline instead of story reports;
- infers package truth from control-kind strings;
- lets unknown control kinds pass formation;
- allows renderer-owned component semantics;
- allows app/editor/game state mutation from generic UI;
- creates a second visual-builder UI format;
- creates component-specific drag/drop/selection/focus systems instead of
  platform interaction primitives;
- mounts a surface before story mount eligibility passes.
