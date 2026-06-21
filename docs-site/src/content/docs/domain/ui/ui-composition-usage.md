---
title: UI Composition Usage
description: Form, inspect, transact, undo, promote, and fixture-test app-neutral UI composition structure.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-06-19
related_designs:
  - ../../design/accepted/app-neutral-ui-composition-design.md
related_adrs:
  - ../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# UI Composition Usage

Use `ui_composition` when code needs app-neutral structural layout: presentation
targets, structural roots, split/stack/overlay/mount-point regions, and opaque
mounted-content references. The crate forms immutable saved definitions into
ratified state and permits structural mutation only through policy-authorized
transactions.

Do not use it for product content, provider/session state, adaptive reflow,
drag previews, app-extension meaning, renderer state, native windows, monitor
bounds, DPI, or editor/Draw/game behavior. The crate owns neutral canonical
composition bundles and atomic filesystem activation; apps still own extension
schemas, payload formation, storage-root policy, and writer serialization.

## Normal workflow

1. Construct namespaced semantic references with fallible constructors.
2. Build a `CompositionDefinitionV1` candidate.
3. Call `CompositionState::form`; only a successful result is ratified state.
4. Read through `CompositionSnapshot`.
5. Submit ordered `CompositionCommand` values in a
   `CompositionTransaction` with the current `StateRevision`.
6. Supply lifecycle, capability, and target policies. A successful transaction
   commits once and records structural history.

```rust
use ui_composition::*;

struct AllowAll;

impl CompositionLifecyclePolicy for AllowAll {
    fn evaluate(
        &self,
        _: CompositionSnapshot<'_>,
        _: &CompositionTransaction,
    ) -> CompositionPolicyDecision {
        CompositionPolicyDecision::Accepted
    }
}

impl CompositionCapabilityPolicy for AllowAll {
    fn evaluate(
        &self,
        _: CompositionSnapshot<'_>,
        _: &CompositionTransaction,
    ) -> CompositionPolicyDecision {
        CompositionPolicyDecision::Accepted
    }
}

impl CompositionTargetPolicy for AllowAll {
    fn evaluate(
        &self,
        _: CompositionSnapshot<'_>,
        _: &CompositionTransaction,
    ) -> CompositionPolicyDecision {
        CompositionPolicyDecision::Accepted
    }
}

fn form_and_activate() -> Result<CompositionState, Box<dyn std::error::Error>> {
    let first = MountedUnitId::new(1);
    let second = MountedUnitId::new(2);
    let content = |instance| {
        Ok::<_, NamespacedReferenceError>(MountedContentRef::new(
            ContentOwnerId::new("example.editor")?,
            ContentProfileId::new("example.document")?,
            ContentInstanceRef::new(instance)?,
        ))
    };
    let unit = |id, instance| {
        Ok::<_, NamespacedReferenceError>(MountedUnitDefinition::new(
            id,
            content(instance)?,
            [],
            UnavailableContentPolicy::ShowFallback,
        ))
    };
    let definition = CompositionDefinitionV1::new(
        CompositionDefinitionId::new(1),
        DefinitionRevision::new(1),
        vec![PresentationTargetDefinition::new(
            PresentationTargetId::new(1),
            TargetProfileId::new("example.desktop")?,
        )],
        vec![CompositionRootDefinition::new(
            CompositionRootId::new(1),
            PresentationTargetId::new(1),
            RegionId::new(1),
            true,
        )],
        vec![RegionDefinition::new(
            RegionId::new(1),
            None,
            RegionKind::Stack {
                ordered_units: vec![first, second],
                active_unit: first,
            },
        )],
        vec![unit(first, "example.document.first")?, unit(second, "example.document.second")?],
    );
    let mut state = CompositionState::form(definition)?;
    let policy = AllowAll;
    state.transact(
        CompositionTransaction::new(
            CompositionTransactionId::new(1),
            state.revision(),
            vec![CompositionCommand::activate_unit(RegionId::new(1), second)],
        ),
        CompositionPolicies {
            lifecycle: &policy,
            capability: &policy,
            target: &policy,
        },
    )?;
    Ok(state)
}
```

Applications should implement real policies. `AllowAll` is suitable only for
tests where policy behavior is outside the test subject.

## Rejection and atomicity

Formation and transaction failures return `CompositionRejection`. Inspect
`diagnostics()` for stable `ui_composition.*` codes, severity, stage, typed
subject, actionable message, and sorted context. Rejected multi-command
transactions leave definition, state revision, transaction set, journal, undo
stack, and redo stack unchanged.

`AuthorizedTransaction` is opaque. Callers cannot alter its commands after the
three policy ports accept it. Internal history restoration cannot be submitted
through `transact`; use `CompositionState::undo` and `CompositionState::redo`,
which reauthorize and revalidate against the current revision.

Composition history is structural only. It never replaces document undo,
browser history, drawing-stroke undo, graph editing history, terminal command
history, game state, or provider/session history.

## Content liveness and fallback

Keep `ContentLivenessObservation` outside canonical composition state. A
missing, loading, suspended, denied, unsupported, or crashed content instance
does not invalidate structure.

`select_content_projection_fallback` encodes the neutral fallback order:
app-provided unavailable projection, neutral diagnostic placeholder, then
hidden only when both the mounted-unit policy and host permit hiding. A later
projection owner performs the actual rendering.

## Promotion and persistence boundary

`CompositionState::promote_definition` creates a normalized `LayoutPromotion`
candidate with a new definition ID, display metadata, scope, compatibility,
and the source state revision. Promotion never captures liveness or adaptive
projection state.

Implement `CompositionExtensionSnapshotPort` in the app owner. The port must
return every required app extension as typed-codec-produced canonical RON in
one call. `snapshot_bundle` fails without producing a partial bundle when any
extension snapshot fails.

```rust
use ui_composition::*;

struct EditorSnapshots;

fn invalid_reference(error: NamespacedReferenceError) -> CompositionPersistenceRejection {
    CompositionPersistenceRejection::single(
        CompositionPersistenceDiagnosticRecord::error(
            CompositionPersistenceDiagnosticCode::InvalidPathComponent,
            CompositionPersistenceDiagnosticStage::Promotion,
            CompositionPersistenceDiagnosticSubject::General("extension_profile".to_owned()),
            "Use a valid namespaced extension profile ID.",
        )
        .with_context("source_error", error.to_string()),
    )
}

impl CompositionExtensionSnapshotPort for EditorSnapshots {
    fn snapshot_extensions(
        &self,
        _: CompositionDefinitionId,
        _: StateRevision,
    ) -> Result<Vec<CanonicalExtensionPayload>, CompositionPersistenceRejection> {
        Ok(vec![CanonicalExtensionPayload::new(
            ExtensionProfileId::new("example.editor.session").map_err(invalid_reference)?,
            ExtensionSchemaVersion::new(1)?,
            "(selected_document:\"example.document.first\")\n",
        )?])
    }
}

fn save_layout(
    state: &CompositionState,
    root: &std::path::Path,
    expected: Option<&CompositionGenerationId>,
) -> Result<CompositionGenerationId, Box<dyn std::error::Error>> {
    let compatibility = CompositionCompatibility::new(
        AppProfileId::new("example.editor")?,
        AppSchemaVersion::new(1)?,
        AppSchemaVersion::new(2)?,
    )?;
    let promotion = state.promote_definition(
        CompositionDefinitionId::new(2),
        LayoutDisplayName::new("Primary editing")?,
        CompositionLayoutScope::Project,
        compatibility,
    );
    let candidate = promotion.snapshot_bundle(&EditorSnapshots)?;
    let activation = CompositionBundleRepository::new(root).activate(&candidate, expected)?;
    Ok(activation.active_generation)
}
```

Activation is compare-and-swap and single-writer per scope. It stages and
read-validates an immutable full-generation bundle before atomically replacing
the pointer. A failure before pointer replacement preserves the prior pointer.
If a committed pointer's final directory durability sync fails, activation
returns the committed generation with a warning diagnostic instead of falsely
reporting that the old generation remained active.

`load_active` validates compatibility and the complete exact extension set. If
the active generation was externally corrupted, it may return the explicit
previous generation with `RecoveredLastGood`; it does not rewrite the pointer.
`CompositionLayoutCatalog` resolves whole layouts as User > Project > BuiltIn
and fails closed on an invalid higher-precedence scope.

Canonical document APIs are closed under `CanonicalCompositionDocuments`.
Decoding rejects unknown fields, noncanonical bytes, and noncanonical semantic
record order. V1-V5 legacy sources are detected read-only and rejected; there
is no automatic migration or success-shaped default.

## Headless fixtures

`CompositionFixture` declares host and target profiles, a definition, mounted
content, liveness, expected validity/capabilities/diagnostics/adaptive proposal
IDs, and forbidden product imports/behavior. `ui_testing` supplies browser,
terminal, dashboard, mobile, and game profiles as structural conformance
fixtures only. Those fixtures do not implement the named products.

Run focused conformance with:

```sh
cargo test -p ui_composition
cargo test -p ui_testing composition_fixture
cargo test -p ui_composition --bench composition_persistence --no-run
task ui:dependencies
```
