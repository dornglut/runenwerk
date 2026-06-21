---
title: Adaptive Composition Usage
description: Derive transient adaptive layouts and proposal-only interactions without taking structural authority.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-20
related_designs:
  - ../../design/accepted/adaptive-ui-composition-design.md
  - ../../design/accepted/app-neutral-ui-composition-design.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# Adaptive Composition Usage

Use `ui_adaptive_composition` to derive a transient layout from an immutable
`CompositionSnapshot`, apply viewport and accessibility constraints, hit-test
regions, preview drag/resize changes, and emit typed proposals. The crate does
not mutate `CompositionState`, execute commands, persist layouts, or own app
content and native-window policy.

## Derive a projection

Hosts provide app-neutral target constraints and optional per-region compact
policy. Missing target constraints fail with stable
`ui_adaptive_composition.*` diagnostics. A missing region policy preserves the
region.

```rust
use ui_adaptive_composition::*;
use ui_math::UiRect;

let fixture = normal_benchmark_fixture();
let policy = AdaptiveProjectionPolicy::new(
    fixture.state.definition().targets().iter().map(|target| {
        AdaptiveTargetConstraints {
            target: target.id,
            bounds: UiRect::new(0.0, 0.0, 1440.0, 1024.0),
            text_scale: 1.0,
            minimum_touch_target: 44.0,
            high_contrast: false,
            reduced_motion: false,
        }
    }),
    fixture.state.definition().regions().iter().map(|region| {
        AdaptiveRegionPolicy {
            region: region.id,
            minimum_width: 120.0,
            minimum_height: 80.0,
            priority: 1,
            compact_behavior: CompactBehavior::Drawer,
        }
    }),
)?;
let projection = AdaptiveProjectionState::derive(
    fixture.state.snapshot(),
    &policy,
)?;
assert_eq!(projection.source_revision(), fixture.state.revision());
# Ok::<(), AdaptiveCompositionRejection>(())
```

## Preview and propose; do not commit

A session shares immutable projection storage. Pointer updates replace only
bounded preview/proposal state and do not clone the region graph. Cancel,
Escape, and rollback clear the session delta without changing canonical state.

```rust
use std::sync::Arc;
use ui_adaptive_composition::*;
use ui_composition::MountedUnitId;
use ui_input::{SemanticActionEvent, SemanticInputSource, UiSemanticAction};
use ui_math::UiPoint;

let fixture = normal_benchmark_fixture();
let projection = Arc::new(AdaptiveProjectionState::derive(
    fixture.state.snapshot(),
    &fixture.policy,
)?);
let mut drag = DragSession::begin(projection, MountedUnitId::new(1));
drag.update_pointer(UiPoint::new(10.0, 10.0));
assert_eq!(drag.metrics().full_graph_clones, 0);

let outcome = drag.handle_semantic_action(SemanticActionEvent::new(
    SemanticInputSource::Keyboard,
    UiSemanticAction::Commit,
));
assert_eq!(outcome, SessionSemanticOutcome::CommitRequested);

let proposal = drag.commit().expect("a hit produced a proposal");
assert_eq!(
    proposal.classification,
    AdaptiveEditClassification::StructuralTransaction,
);
assert!(proposal.requires_host_transaction());
// The host now evaluates policy and materializes this typed intent into a
// ui_composition transaction against proposal.source_revision.
# Ok::<(), AdaptiveCompositionRejection>(())
```

Pointer, keyboard, touch, and controller input use the same semantic actions.
Raw controller buttons/axes and app commands do not enter this crate.

## Edit classification and saving

Every adaptive edit has one classification:

- `TransientAdaptive`: projected reflow, drawer/overflow state, hover, focus,
  preview, or active session state;
- `StructuralTransaction`: ordered commands proposed for host authorization;
- `PromotionCandidate`: a named, scoped `AdaptivePromotionDelta` that a later
  app-owned save workflow may consume.

Saving canonical composition excludes adaptive projection state. Promotion is
explicit and deterministic; it never silently overwrites the source layout.

## Accessibility and inspection

Each projected node exposes a non-empty inspection label, stable focus order,
visible-focus metadata, contrast mode, text scale, minimum touch target, and
reduced-motion transition duration. Product renderers consume these facts but
remain responsible for actual visuals, screen-reader integration, and OS input
bridges.

## Fixtures and performance

`ui_testing::run_adaptive_composition_conformance_fixtures` checks browser,
terminal, dashboard, mobile, and game profiles as product-free headless
fixtures. The benchmark crate separately measures hit testing, proposal
generation, preview projection, complete drag-frame updates, transaction
validation, committed mutation, serialization, and validation/deserialization.

Run the focused proof with:

```sh
cargo test -p ui_adaptive_composition
cargo test -p ui_input semantic
cargo test -p ui_testing adaptive_composition
cargo bench -p ui_adaptive_composition --bench adaptive_composition
task ui:dependencies
```
