---
title: UI Substrate and Surface Roadmap
description: Doctrine-aligned, dependency-aware roadmap for Runenwerk UI substrate hardening and surface-semantic maturation.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# Runenwerk UI Substrate and Surface Roadmap

## 1. Title and Purpose
This document is the working architecture roadmap for the next major Runenwerk UI cycle.

Target file: `docs-site/src/content/docs/domain/ui/roadmap.md`.

Its purpose is to sequence implementation work from current repository truth toward doctrine-aligned surface semantics, without speculative feature promises or short-term architectural shortcuts.

## 2. Current Reality Summary
Implemented state:
- Foundation and substrate crates exist and are active: `domain/ui/ui_math`, `domain/ui/ui_input`, `domain/ui/ui_layout`, `domain/ui/ui_text`, `domain/ui/ui_theme`, `domain/ui/ui_render_data`, `domain/ui/ui_tree`, `domain/ui/ui_runtime`, `domain/ui/ui_widgets`.
- `domain/ui/ui_surface` exists as a first-class crate with definition, mount, observation/session/presentation/intent/ratification contracts.
- Retained runtime extraction is real: input dispatch, focus traversal, invalidation signaling, tree/layout/frame build, and render submission contracts are in place.
- Early reusable controls and runtime nodes exist and are used in editor composition paths.
- Frame build path and gallery harness exist (`domain/ui/ui_runtime/examples/substrate_gallery.rs`).
- Shell/runtime/engine seams are operational and already guarded by architecture tests in `apps/runenwerk_editor/tests/*`.

Current weaknesses:
- Surface semantics are still partially distributed across `editor_shell`, `editor_viewport`, and app/runtime glue; `ui_surface` is now integrated for core control flows but full semantic centralization is ongoing.
- Observation, session, presentation, intent, and ratification boundaries now exist for core paths, but broader surface families and advanced interactions still need migration coverage.
- Viewport slot semantic ownership is now enforced on current viewport seams, but must remain guard-tested as additional surfaces integrate.
- Some runtime bootstrapping seams remain intentionally transitional and must stay explicitly bounded.

## 3. Architectural Stance
Settled decisions:
- Doctrine alignment is mandatory.
- Retained tree/runtime (`ui_tree`, `ui_runtime`) remain technical substrate, not semantic center.
- Surfaces are the semantic center.
- `ui_surface` is created now as its own crate, not as a temporary module.
- `SurfacePresentationModel` is central: surfaces render from prepared presentation models, not directly from raw observation.
- Observation/session/presentation/intent/ratification boundaries become explicit and enforced.
- Ratification remains outside UI crates where domain authority or trust elevation is required.
- Hosts mount surfaces but do not own surface semantics.
- `editor_viewport` owns viewport slot meaning.
- `ui_render_data` stays renderer-facing and generic; it owns render/embed payload form, not viewport semantic meaning.
- Mapping between viewport semantics and renderer payload form happens at integration edges (`runenwerk_editor` and engine adapters).
- Crate splitting strategy is conservative: create `ui_surface` now, defer further micro-splitting until contracts stabilize.

## 4. Guiding Constraints
- Keep doctrine invariants explicit: observation frames, session reality, expressed reality, explicit ratification boundaries.
- Prevent god-object drift by keeping contracts narrow and compositional.
- Require capability/trust-aware semantics at mount and intent boundaries with explicit classes:
- `observe`: read observation and presentation data.
- `interact`: local UI/session interaction without domain mutation authority.
- `request_mutation`: emit `SurfaceIntent` for host/domain adjudication.
- `ratify`: privileged mutation approval path outside UI substrate where appropriate.
- Make `SessionScopeHandle` retention classes explicit and first-class:
- `Ephemeral`
- `Restorable`
- `Persistent`
- `Shareable`
- Preserve domain ownership direction; avoid semantic leakage into renderer-facing crates.
- Use dependency-correct sequencing; do not finalize downstream semantic ownership in earlier substrate phases.

## 5. Critical Weaknesses / Risks
- Risk: semantic authority remains fragmented across shell/app/runtime modules. Mitigation: establish `ui_surface` as semantic center before broad behavior expansion.
- Risk: viewport semantic meaning leaks into renderer contracts or is duplicated in parallel owners. Mitigation: keep semantic taxonomy in `editor_viewport`, keep payload form in `ui_render_data`, enforce adapter-only mapping.
- Risk: `Surface` becomes a catch-all object. Mitigation: enforce strict split between observation, session, presentation, intent, and ratification contracts.
- Risk: surfaces consume raw observation directly and accumulate transformation logic ad hoc. Mitigation: require prepared `SurfacePresentationModel` as standard surface input.
- Risk: ratification logic drifts into UI runtime for convenience. Mitigation: isolate ratification adapters outside UI substrate and enforce through tests.
- Risk: retention behavior is implicit and inconsistent. Mitigation: formalize and test `Ephemeral/Restorable/Persistent/Shareable` semantics.
- Risk: premature expansion into 3D/collab/database surface families before core contracts stabilize. Mitigation: explicit wait gates and anti-goals.

## 6. Dependency-Aware Phased Roadmap
Execution order is strict: Phase 1 -> Phase 2 -> Phase 3 -> Phase 4 -> Phase 5 -> Phase 6.

### Phase 1 - Substrate Ownership Hardening
Goal:
- Harden ownership and lifecycle boundaries for retained substrate without locking viewport semantic ownership in the wrong layer.

Why this phase comes here:
- Later semantic consolidation depends on stable substrate contracts and explicit transitional seam boundaries.

Concrete target areas:
- `domain/ui/ui_tree`
- `domain/ui/ui_runtime`
- `domain/ui/ui_widgets`
- `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs`
- `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs`
- `apps/runenwerk_editor/src/runtime/viewport/*`
- Existing guard tests in `apps/runenwerk_editor/tests/*`

Done-when criteria:
- Runtime substrate crates are free of editor-domain semantic ownership.
- Transitional bootstrap paths are explicit, bounded, and test-guarded.
- No new viewport semantic taxonomy is declared in `ui_render_data` during this phase.
- Ownership and lifecycle boundaries are documented and aligned with current implemented seams.

Explicit non-goals:
- Do not canonicalize viewport slot meaning in renderer-facing crates.
- Do not introduce `ui_surface` full contract stack yet.
- Do not expand surface families.

### Phase 2 - Introduce `ui_surface` Crate as Semantic Kernel
Goal:
- Establish `domain/ui/ui_surface` with minimal surface lifecycle primitives and mount contracts.

Why this phase comes here:
- Semantic authority must be centralized before contract formalization and viewport seam consolidation.

Concrete target areas:
- New crate `domain/ui/ui_surface`
- Initial contracts: `SurfaceDefinition`, `MountedSurfaceInstance`, mount/unmount containment and lifecycle boundaries
- Host integration points in `domain/editor/editor_shell` and `apps/runenwerk_editor` where mounting occurs

Done-when criteria:
- At least one production surface is mounted through `ui_surface` contracts.
- Mount ownership is explicit: host mounts, surface semantics remain in surface layer.
- Lifecycle boundaries are testable without embedding domain mutation logic in substrate runtime paths.

Explicit non-goals:
- Do not split `ui_surface` into multiple crates yet.
- Do not attempt full surface migration in one pass.

### Phase 3 - Formalize Observation/Session/Presentation/Intent/Ratification Contracts
Goal:
- Define and adopt doctrine-aligned semantic contracts with `SurfacePresentationModel` as the central build input.

Why this phase comes here:
- Prevents god-object collapse and clarifies authority before viewport/render seam migration.

Concrete target areas:
- `ui_surface` contract modules for `ObservationFrame`, `SessionScopeHandle`, `SurfacePresentationModel`, `SurfaceIntent`
- Capability/trust contract expression for `observe/interact/request_mutation/ratify`
- Host-side `RatificationAdapter` boundaries in editor/app domain integration layers

Done-when criteria:
- Surface build paths consume prepared `SurfacePresentationModel`, not raw observation directly.
- `SessionScopeHandle` semantics explicitly encode `Ephemeral`, `Restorable`, `Persistent`, `Shareable`.
- `SurfaceIntent` emission and ratification are explicit and test-covered across at least one end-to-end flow.
- Ratification remains outside UI substrate for privileged mutation paths.

Explicit non-goals:
- Do not redesign persistence infrastructure.
- Do not implement collaboration protocol or sharing backend.

### Phase 4 - Viewport/Embed/Render-Data Seam Consolidation (Corrected Ownership Model)
Goal:
- Consolidate viewport embedding seams while preserving semantic ownership in `editor_viewport` and renderer payload ownership in `ui_render_data`.

Why this phase comes here:
- Requires stable semantic contracts from Phases 2 and 3 to avoid semantic drift into renderer layers.

Concrete target areas:
- `domain/editor/editor_viewport/src/expression/*` as semantic owner of viewport slot meaning
- `domain/ui/ui_render_data/src/primitives/viewport_surface_embed.rs` and related render payload contracts as generic renderer-facing forms
- Integration adapters in `apps/runenwerk_editor/src/runtime/viewport/*`
- Engine consumption paths under `engine/src/plugins/render/features/ui/*`

Done-when criteria:
- Viewport slot semantic taxonomy is defined in `editor_viewport` only.
- `ui_render_data` carries renderer-facing payload shape without owning viewport semantic meaning.
- Adapter mapping from viewport semantics to render payload form is explicit and test-covered.
- `runenwerk_editor` and engine consume/adapt mappings without introducing parallel semantic taxonomies.

Explicit non-goals:
- Do not redesign the renderer architecture.
- Do not implement 3D/world-space UI features.

### Phase 5 - Control Semantics Hardening (Surface-Centered, Not Widget-Centered)
Goal:
- Strengthen control semantics through surface-level intent and presentation contracts without regressing to widget-centric ownership.

Why this phase comes here:
- Control semantics should be upgraded only after semantic contracts and viewport seams are stable.

Concrete target areas:
- Surface composition paths in `domain/editor/editor_shell/src/composition/*`
- Reusable control usage in `domain/ui/ui_widgets` aligned with surface contracts
- Intent routing and reducer integration in app/shell integration seams

Done-when criteria:
- Core editor surfaces route behavior through typed `SurfaceIntent` and prepared presentation models.
- Control interactions reflect capability/trust distinctions instead of direct mutation shortcuts.
- Behavior regressions are covered by interaction and architecture guard tests.

Explicit non-goals:
- Do not build a broad speculative widget catalog.
- Do not perform a visual redesign unrelated to architecture boundaries.

### Phase 6 - Verification, Reflector/Debugging, Gallery, and Docs Hardening
Goal:
- Lock architecture correctness with durable verification and operational introspection.

Why this phase comes here:
- Enforcement is most valuable after core ownership and semantic boundaries are implemented.

Concrete target areas:
- Architecture and seam guard suites in `apps/runenwerk_editor/tests/*` and UI crate tests
- Gallery scenarios in `domain/ui/ui_runtime/examples/substrate_gallery.rs` expanded for surface lifecycle cases
- Reflector/debugging traces for observation -> presentation -> intent -> ratification flow
- Documentation alignment in:
- `docs-site/src/content/docs/domain/ui/architecture.md`
- `docs-site/src/content/docs/domain/ui/roadmap.md`
- `domain/ui/README.md`

Done-when criteria:
- Boundary regressions fail fast in CI.
- Debug artifacts can trace capability gates and ratification boundaries.
- Gallery scenarios cover multi-surface and viewport embed lifecycle cases.
- Docs and tests describe the same architecture, with no stale ownership claims.

Explicit non-goals:
- Do not create unrelated documentation refactors.
- Do not treat instrumentation as a substitute for boundary design.

## 8. Expansion Paths That Should Wait
- World-surface and 3D UI surface families wait until Phase 4 and Phase 6 stability gates pass.
- SDF-backed UI rendering paths wait until semantic-to-render adaptation seams are stable.
- Database/editor-authoring surfaces wait until session retention classes and ratification boundaries are proven in existing surfaces.
- Collaborative/shared surfaces wait until capability and ratification flows are validated in single-user architecture.
- Broad external/plugin-authored surface API waits until `ui_surface` contracts survive at least one full migration cycle.

## 9. Explicit Anti-Goals
- Do not recenter semantics in host shells.
- Do not treat retained runtime as semantic authority.
- Do not place viewport slot semantic ownership in `ui_render_data`.
- Do not allow direct raw-observation-to-surface rendering as default architecture.
- Do not collapse observation/session/presentation/intent/ratification into one monolithic `Surface` type.
- Do not pull privileged ratification responsibilities into generic UI substrate code.
- Do not over-split crates before contracts prove stable.
- Do not front-load speculative future feature systems.

## 10. Final Sequencing Rationale
The sequence is intentionally conservative and dependency-correct. Phase 1 stabilizes substrate ownership without making downstream semantic ownership decisions. Phase 2 establishes `ui_surface` as semantic center. Phase 3 defines explicit doctrine contracts, with `SurfacePresentationModel` and capability/ratification boundaries preventing god-object drift. Phase 4 then consolidates viewport/embed/render seams with corrected ownership (`editor_viewport` semantics, `ui_render_data` payload form, adapter mapping at integration edges). Phase 5 hardens control semantics against that foundation. Phase 6 turns the architecture into an enforceable system through tests, reflector/debugging, gallery scenarios, and aligned documentation. This order minimizes architectural backtracking while keeping room for future surface families without premature overbuilding.
