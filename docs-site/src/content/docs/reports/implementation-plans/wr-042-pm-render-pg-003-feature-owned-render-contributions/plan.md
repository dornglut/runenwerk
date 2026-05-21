---
title: WR-042 PM-RENDER-PG-003 Feature-Owned Render Contributions Plan
description: Promotion and implementation-readiness contract for the bounded PM-RENDER-PG-003 feature-owned render contribution collector slice.
status: active
owner: engine
layer: engine-runtime / render prepared-frame contracts
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/feature-owned-render-contributions-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-contract-ergonomics-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-042 PM-RENDER-PG-003 Feature-Owned Render Contributions Plan

## Goal

Promote and implement `PM-RENDER-PG-003` as one bounded collector-registry
slice for feature-owned prepared render contributions.

The slice replaces central contribution collection growth with a typed,
inspectable, diagnostic-producing collector path while preserving the existing
prepared-frame submit boundary. It must not implement render fragments, hot
reload, render graph compiler maturity, material lowering, all-feature
migration, or renderer-owned product policy.

## Source Of Truth

- Production milestone: `PM-RENDER-PG-003`.
- Bounded implementation row: `WR-042`.
- Accepted PM-003 design:
  `docs-site/src/content/docs/design/accepted/feature-owned-render-contributions-design.md`.
- Boundary design:
  `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`.
- PM-002 prerequisite closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-pg-002-render-contract-ergonomics/closeout.md`.
- Support context only: `WR-003`.
- Out of scope implementation row: `WR-010`; it remains for render fragments
  and hot reload under `PM-RENDER-PG-007`.

## Promotion Readiness

`task production:plan -- --milestone PM-RENDER-PG-003 --roadmap WR-042`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-RENDER-PG-002` remains completed with closeout evidence.
- `WR-041` remains completed in the roadmap archive.
- `PM-RENDER-PG-003` links `WR-042` as the bounded implementation row.
- `WR-010` is not used as the PM-003 implementation row.
- No current-candidate WR row conflicts with `WR-042`.
- The accepted PM-003 design and accepted Render Product Graph Platform design
  are still present and valid.

If promotion succeeds, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, planning validate,
and `task ai:goal -- --track PT-RENDER-PG` before product code changes.

## Allowed Write Scope

Implementation may touch only:

- `engine/src/plugins/render`;
- `engine/tests`;
- `docs-site/src/content/docs/engine/reference/plugins/render`;
- `docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md`;
- `docs-site/src/content/docs/design/accepted/feature-owned-render-contributions-design.md`;
- `docs-site/src/content/docs/reports/implementation-plans/wr-042-pm-render-pg-003-feature-owned-render-contributions/plan.md`;
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-21-pm-render-pg-003-bounded-implementation-`;
- roadmap and production-track source/generated files;
- the PM-003 closeout evidence after implementation validation passes.

No app producer migration, material graph lowering, asset pipeline work, native
multi-window work, fragment hot reload, or backend-specific allocation work is
in scope.

## Owning Modules

The implementation should keep ownership inside the render plugin:

- `engine/src/plugins/render/frame/contribution_registry.rs` for collector
  descriptors, typed payload kind metadata, declared resource requirements, and
  registry validation.
- `engine/src/plugins/render/frame/contribution_diagnostics.rs` for structured
  collector diagnostics.
- `engine/src/plugins/render/frame/contributions.rs` for the registered payload
  compatibility packet and `PreparedFrameContributions` bridge.
- `engine/src/plugins/render/runtime/frame_prepare.rs` for deterministic
  collector execution during render prepare.
- `engine/src/plugins/render/inspect/prepared_frame.rs` for prepared-frame
  inspection of registered contribution packets and diagnostics.
- `engine/src/plugins/render/renderer/prepare.rs` only for runtime signature
  support for the registered compatibility packet.
- `engine/src/plugins/render/features` only where feature descriptor metadata
  needs to connect to collector descriptors.

## Implementation Contract

Add a registry-backed contribution path beside the current central path:

1. Define collector ids, payload kinds, resource requirements, descriptors, and
   typed diagnostics.
2. Add a `RenderFeatureContributionCollectorRegistryResource` or equivalent
   render-owned registry.
3. Add a registered payload compatibility packet so new features do not need
   feature-specific `PreparedFeaturePayload` variants.
4. Require registered payloads to provide validation, inspection, and runtime
   signatures.
5. Run collectors only during render prepare and only against declared prepared
   resources.
6. Preserve current `PreparedFeaturePayload` variants and insertion helpers for
   existing features during the strangler migration.
7. Prove a test-only registered feature can contribute without a new
   feature-specific central payload variant.
8. Migrate exactly one low-risk existing contribution path through the
   registered collector path. Prefer scene route or another small contribution
   with no product truth, no material lowering, and no backend allocation
   policy.
9. Keep missing contribution insertion and fallback policy behavior equivalent
   for execution feature ids.
10. Expose typed diagnostics and inspection for registered payloads.

## Non-Goals

Do not implement:

- render fragments, fragment assets, fragment merge, hot reload, or last-good
  fragment promotion;
- whole-frame render execution graph compiler maturity;
- product-surface hardening beyond contribution collection;
- material graph truth, material lowering, shader fallback policy, or asset
  cooking;
- native multi-window or multi-surface presentation;
- all-feature migration or central payload variant removal;
- plugin ABI, external mod support, or persisted contribution packets;
- renderer-owned product truth, freshness, authority, fallback legality,
  rebuild policy, product dependencies, or residency policy;
- submit-time ECS extraction.

## Validation

Focused implementation validation must include:

```text
cargo test -p engine render_feature_contributions
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
```

Workflow validation must include:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Closeout Contract

Create PM-003 closeout evidence only after implementation and validation pass.
Only then update `PM-RENDER-PG-003` completion evidence.

Expected completion quality is `bounded_contract`. Do not claim
`runtime_proven` or `perfectionist_verified` unless a later explicit audit
requires and proves that higher bar.

The closeout must name:

- the accepted PM-003 design;
- the promoted WR row;
- the implementation plan;
- focused test output;
- workflow validation output;
- remaining known gaps for PM-004 through PM-008.

## Stop Conditions

Stop before implementation if:

- `WR-042` cannot be promoted legally;
- the PM-003 accepted design is missing or no longer accepted;
- production or roadmap validation fails;
- ownership would move product truth or product policy into the renderer;
- implementation requires render fragments, hot reload, compiler maturity, or
  material lowering;
- submit needs live ECS extraction;
- the write scope expands outside this contract;
- focused tests cannot be defined for registry validation, diagnostics,
  registered payload inspection, compatibility adapter behavior, and the
  migrated low-risk feature.
