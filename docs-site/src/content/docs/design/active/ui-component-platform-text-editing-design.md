---
title: UI Component Platform Text Editing / Editable Text Behavior Design
description: Owner-first package-backed design intake for reusable editable text behavior without product/editor/game mutation, authored UI editing, rich document editing, or code editor ownership.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/decision-register.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ./ui-component-platform-overlay-popup-layering-design.md
  - ./ui-component-platform-generic-text-design.md
---

# UI Component Platform Text Editing / Editable Text Behavior Design

Lifecycle state: `active-planning`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-014`.

This document opens Phase 14 design/planning intake. It does not authorize Rust implementation. Implementation requires a later `active-implementation` transition with exact owner files, accepted scope, validation envelope, evidence expectations, and stop conditions.

## Decision summary

Reusable UI controls may declare editable text behavior through package-backed descriptors. Runtime may consume those declarations and normalized input facts to produce caret, selection, composition, edit-intent, validation, suppression, focus, replay/report, proof-frame, and no-bypass evidence.

Generic UI must not commit product/editor/game state mutation. Host/product layers own actual persistence, domain mutation, command routing, document buffers, authored UI editing, app-specific editor policy, and undo stacks that affect product documents.

## Problem

Phase 12 added a read-only text-intent probe. That probe is useful for proving the normalized input and interaction seam, but it explicitly does not own caret position, selection, IME/composition, clipboard, undo/redo, validation, text-buffer mutation, or edit transactions.

Phase 14 must turn that seam into coherent reusable editable text behavior without promoting generic UI into a product editor, UI Designer, rich text editor, code editor, or command executor.

## Current reality

`ui_controls` currently exposes interaction descriptors and overlay descriptors in `ControlPackageDescriptor`. It has no canonical editable-text descriptor collection yet.

`InspectorField` currently lowers a text-intent probe through interaction lowering. That proves text intent can be observed; it is not editable behavior and must not be treated as full text editing.

`ui_input` currently owns normalized pointer, keyboard, focus, semantic, and text-intent facts. `TextIntentFact` is intentionally minimal and documented as a pre-editing probe.

`ui_runtime::input` generic interaction proof explicitly excludes full text editing. `ui_runtime::overlay` now owns completed overlay/layering proof under a separate runtime module. Phase 14 should follow that owner-first pattern with a dedicated text-editing runtime module rather than placing editing semantics inside generic input.

`ui_static_mount` validates renderer-neutral `UiFrame` proof frames. It should validate text-editing proof frames without gaining editing behavior.

## Goals

- Add reusable editable-text declaration vocabulary owned by `ui_controls`.
- Keep editable behavior package-backed and visible in catalog and inspection.
- Keep normalized keyboard/text/composition/focus/selection facts in `ui_input` as facts only.
- Add runtime-owned text-editing replay/report evidence under a dedicated text-editing module.
- Prove caret, selection, composition, edit intent, validation, suppression, focus, and no-bypass behavior without product mutation.
- Prove renderer-neutral visual evidence through `TextEditingVisualProof`, `TextEditingProofRenderFrame`, `UiFrame`, and `ui_static_mount`.
- Treat paste as a host-owned content source represented by normalized intent/evidence, not as clipboard ownership.
- Design explicit base-control adoption. Do not force text editing onto controls that are not semantically editable.

## Explicit non-goals

This phase does not implement or authorize:

- product/editor/game state mutation inside generic UI;
- command execution inside generic UI;
- authored UI editing;
- full UI Designer;
- UI Gallery product surface;
- Workbench/provider redesign;
- full rich text editor;
- code editor semantics;
- syntax highlighting, diagnostics, indentation, multi-cursor, language server, or code navigation behavior;
- app-specific undo/redo stack;
- product document buffer ownership;
- persistence, save, commit-to-domain, or route authorization;
- clipboard read/write ownership;
- dynamic plugin framework;
- `foundation/meta`;
- shared plugin primitives;
- overlay runtime changes unrelated to text editing;
- compatibility-only aliases or shims;
- phase-shaped public API names.

## Owner boundaries

### `ui_controls`

Owns reusable editable-text declarations, descriptor vocabulary, ergonomic builders, base-control text-editing lowering, package descriptor fields, package validation, catalog projection, and inspection projection.

Must not own runtime caret/session state, OS/window input collection, renderer backend behavior, product/editor/game commands, product mutation, authored UI editing, rich document semantics, code editor semantics, app-specific editor policy, or product undo stacks.

### `ui_input`

Owns normalized keyboard, text, composition, focus, and selection facts only. Facts should describe what input occurred or what host-owned source supplied content; they must not decide editable control behavior, mutate buffers, execute commands, route product actions, or own clipboard integration.

### `ui_runtime`

Owns renderer-neutral text-editing proof under `domain/ui/ui_runtime/src/text_editing/`. Runtime may consume package-backed editable-text declarations and normalized input facts to produce caret, selection, composition, edit-intent, validation, suppression, focus, replay/report, proof-frame, and no-bypass evidence.

Runtime must emit reusable text-editing intent/evidence only. It must not commit product/editor/game state, execute commands, own authored UI editing, own a product document buffer, own app undo/redo, or implement rich/code editor behavior.

### `ui_static_mount`

Owns static validation of renderer-neutral text-editing proof frames. It must not interpret text-editing policy or mutate text state.

### Host/product/editor/game layers

Own actual persistence, domain mutation, command routing, route authorization, document buffers, authored UI editing, app-specific editor policy, and undo stacks that affect product documents.

## Exact owner crates/files

Expected implementation should stay inside these owner paths unless a later accepted scope revision says otherwise:

```text
domain/ui/ui_controls/src/editable_text.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/package/editable_text_validation.rs
domain/ui/ui_controls/src/base_control/lowering/text_editing_support.rs
domain/ui/ui_controls/src/base_control/lowering/mod.rs
domain/ui/ui_controls/src/base_control/compiler.rs
domain/ui/ui_controls/src/base_control/lowering/interaction.rs
domain/ui/ui_controls/src/base_control/lowering/inspection.rs
domain/ui/ui_controls/src/base_control/preset.rs only if a new semantically editable base control is explicitly accepted
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/tests/base_controls_text_editing_package.rs
domain/ui/ui_controls/tests/base_controls_text_editing_catalog.rs
domain/ui/ui_controls/tests/base_controls_text_editing_inspection.rs

domain/ui/ui_input/src/facts.rs
domain/ui/ui_input/src/text.rs
domain/ui/ui_input/src/composition.rs
domain/ui/ui_input/src/selection.rs
domain/ui/ui_input/src/keyboard.rs
domain/ui/ui_input/src/focus.rs
domain/ui/ui_input/src/lib.rs
domain/ui/ui_input/tests/text_editing_normalized_facts.rs

domain/ui/ui_runtime/src/text_editing/mod.rs
domain/ui/ui_runtime/src/text_editing/fixture.rs
domain/ui/ui_runtime/src/text_editing/replay.rs
domain/ui/ui_runtime/src/text_editing/report.rs
domain/ui/ui_runtime/src/text_editing/proof_frame.rs
domain/ui/ui_runtime/src/lib.rs
domain/ui/ui_runtime/tests/text_editing_report.rs
domain/ui/ui_runtime/tests/text_editing_package_backed.rs

domain/ui/ui_static_mount/tests/base_controls_text_editing_static_mount.rs
```

Do not add product mutation to `apps/runenwerk_editor`, editor shell routing, Workbench providers, UI Gallery surfaces, plugin infrastructure, or `foundation/meta` for this phase. `runenwerk_editor` may appear in the validation gate only to prove the generic crate changes remain consumable.

## Full implementation chain

```text
ui_controls editable-text declaration vocabulary
  -> base-control text-editing lowering
  -> ControlPackageDescriptor text-editing descriptors
  -> package validation
  -> catalog projection
  -> inspection projection
  -> ui_input normalized keyboard/text/composition/focus/selection facts
  -> ui_runtime text-editing replay/report/caret/selection/composition/edit-intent/suppression evidence
  -> TextEditingVisualProof
  -> TextEditingProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

## Package/catalog/inspection requirements

Phase 14 should introduce a package-backed descriptor such as `ControlEditableTextDescriptor` with domain-shaped names, not phase-shaped names.

The descriptor should capture:

- target `control_kind_id`;
- editable mode: text field, text area, or inline editable field;
- supported edit intents: insert, delete backward, delete forward, replace selection, move caret, extend selection, commit, cancel, paste intent, composition start/update/commit/cancel;
- caret support;
- selection support;
- composition/IME pre-edit support;
- focus acquisition and focus return requirements;
- disabled and read-only suppression policy;
- validation declarations such as required input mode, maximum length, empty allowance, or host-owned validation marker;
- host-owned paste/content source policy;
- proof flags showing zero host commands, zero product mutation, zero authored UI edits, and zero product undo operations.

`ControlPackageDescriptor` should expose package-level editable-text descriptors, builder helpers, and lookup helpers. Main-path package validation must reject duplicate descriptors, descriptors for missing control kinds, descriptors with no supported edit intents, invalid selection/composition combinations, and invalid editable declarations on controls that are not explicitly marked semantically editable.

Catalog projection should expose read-only facts such as editable mode, caret support, selection support, composition support, validation support, edit-intent labels, paste-intent support, read-only suppression, disabled suppression, and no-command/no-mutation flags.

Inspection should gain a first-class `TextEditing` section or an equally explicit existing section if a later implementation proves that a new section is unnecessary. Inspection must not hide editable facts in unrelated interaction or metadata sections.

## Base-control adoption requirements

Likely first candidates:

- `InspectorField`, because it already has typed prompt/input semantics through text-intent probe behavior;
- `ActionPrompt` only if actual typed prompt semantics exist or are added explicitly;
- future `TextField` or `TextArea` controls if current base controls do not contain a semantically editable control.

Do not force text editing onto Label, Button, ColorPicker, ListView, TreeView, or TableView unless a later design explicitly defines an editable subrole for them. If a new base control is needed, design it as a real base control instead of hiding editing behavior inside unrelated controls.

## Runtime proof requirements

`ui_runtime` should produce a dedicated text-editing report with at least these evidence groups:

- package descriptor facts proving the runtime consumed package-backed editable-text declarations;
- ordered normalized input steps;
- focus acquisition and focus return evidence;
- caret placement and movement evidence;
- selection range creation, extension, replacement, and collapse evidence;
- composition/IME pre-edit start/update/commit/cancel evidence;
- edit-intent evidence for insert, delete, replace, paste, commit, and cancel;
- validation and suppression evidence;
- disabled/read-only suppression evidence;
- no-target and invalid-target evidence where relevant;
- no-bypass boundary counters.

Runtime proof may form reusable edit intents such as `InsertTextRequested`, `DeleteBackwardRequested`, `ReplaceSelectionRequested`, `CommitTextRequested`, or `PasteIntentRequested`. Those are evidence and host-consumable intent only; they must not mutate product buffers or execute product commands.

## Positive proof scenarios

The implementation must prove:

- package-backed editable-text declaration for every semantically editable base-control candidate included in scope;
- base-control lowering attaches editable descriptors through the compiled package path;
- package validation accepts valid editable descriptors through the main validation path;
- catalog and inspection expose editable text support without mutable internals;
- focus acquisition for an editable target;
- caret placement from normalized focus/pointer/keyboard facts;
- caret movement by arrow, Home, End, and equivalent normalized navigation facts;
- text insertion intent;
- backward and forward deletion intent;
- replacement intent when a selection is active;
- selection extension, selection collapse, and optional select-all intent if included in the normalized vocabulary;
- composition start/update/commit/cancel as pre-edit facts;
- paste intent as a host-owned content source;
- validation evidence for accepted and rejected text intent;
- disabled and read-only suppression;
- runtime report to `TextEditingVisualProof` to `TextEditingProofRenderFrame` to `UiFrame`;
- `ui_static_mount` acceptance of the renderer-neutral proof frame.

## Negative and no-bypass proof scenarios

The implementation must prove:

- unresolved editable-text descriptors are rejected;
- duplicate editable-text descriptors are rejected;
- empty edit-intent support is rejected;
- invalid composition or selection combinations are rejected;
- non-editable controls do not receive editable behavior accidentally;
- disabled editable controls suppress edit intent;
- read-only editable controls may expose focus/caret/read evidence but suppress mutation intents;
- paste evidence does not read from or write to the clipboard;
- generic UI executes zero host commands;
- generic UI performs zero product/editor/game mutations;
- generic UI performs zero authored UI edits;
- generic UI performs zero product undo/redo operations;
- generic UI performs zero plugin-framework operations;
- generic UI does not open unrelated overlay runtime paths;
- rich document editing and code editor behavior remain absent.

## Static mount proof requirements

Text-editing proof frames should include renderer-neutral evidence for:

- a main proof area with editable target, caret marker, selection marker, and optional composition marker;
- an inspector/evidence area with descriptor support facts;
- a report area with input, edit-intent, validation, suppression, and no-bypass rows;
- stable draw order;
- at least one surface;
- rect/background, border/outline, and glyph-run evidence.

`ui_static_mount` validates the frame shape. It does not own text editing behavior, text shaping policy, product state, or editor commands.

## Validation gate

The eventual implementation gate is:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo check -p ui_input
cargo check -p ui_runtime
cargo check -p ui_static_mount
cargo check -p ui_story
cargo check -p runenwerk_editor
cargo test -p ui_controls text
cargo test -p ui_controls --test base_controls_text_editing_package
cargo test -p ui_controls --test base_controls_text_editing_catalog
cargo test -p ui_controls --test base_controls_text_editing_inspection
cargo test -p ui_input input
cargo test -p ui_runtime text_editing
cargo test -p ui_runtime --test text_editing_report
cargo test -p ui_runtime --test text_editing_package_backed
cargo test -p ui_static_mount text_editing
python tools/docs/validate_docs.py
git diff --check
```

This planning pass does not claim those commands were run.

## Stop conditions

Stop and redesign if Phase 14 requires:

- product/editor/game state mutation inside generic UI;
- command execution inside generic UI;
- authored UI editing;
- full UI Designer;
- UI Gallery product surface;
- Workbench/provider redesign;
- full rich text editor;
- code editor semantics;
- app-specific undo/redo stack;
- dynamic plugin framework;
- `foundation/meta`;
- shared plugin primitives;
- overlay runtime changes unrelated to text editing;
- compatibility-only aliases or shims;
- phase-shaped public API names;
- text-editing semantics hidden under `ui_runtime::input` instead of a dedicated runtime text-editing owner path;
- editable behavior attached to controls that are not semantically editable without an explicit design update.

## Relationship to current work

Phase 13 is completed through PR #44. Phase 14 is the next planning/design intake only. Implementation remains blocked until this design is accepted and the active-work record is promoted to `active-implementation` with exact scope.
