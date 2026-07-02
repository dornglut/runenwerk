---
title: UI Component Platform Text Editing / Editable Text Behavior Design
description: Owner-first package-backed implementation plan for reusable editable text behavior without product/editor/game mutation, authored UI editing, rich document editing, or code editor ownership.
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

Lifecycle state: `review`.

Planning ID: `PT-UI-COMPONENT-PLATFORM-014`.

This document is the Phase 14 design and implementation plan. It did not authorize Rust implementation by itself. The 2026-07-02 user handoff supplied the exact owner files, validation envelope, evidence expectations, and stop conditions required to promote the local branch through implementation and into review.

## Decision summary

Reusable UI controls may declare editable text behavior through package-backed descriptors. Runtime may consume those declarations and normalized input facts to produce deterministic caret, selection, composition, edit-intent, validation, suppression, focus, replay/report, proof-frame, and no-bypass evidence.

Generic UI must not commit product/editor/game state mutation. Host/product layers own persistence, domain mutation, command routing, document buffers, authored UI editing, app-specific editor policy, and undo stacks that affect product documents.

The phase is complete only when editable text is declared, validated, packaged, cataloged, inspectable, normalized through `ui_input`, replayed through `ui_runtime`, render-proofed, static-mount validated, and ergonomic enough that future base-control authors do not have to hand-build descriptor plumbing.

## Problem

Phase 12 added a read-only text-intent probe. That probe is useful for proving the normalized input and interaction seam, but it explicitly does not own caret position, selection, IME/composition, clipboard, undo/redo, validation, text-buffer mutation, or edit transactions.

Phase 14 must turn that seam into coherent reusable editable text behavior without promoting generic UI into a product editor, UI Designer, rich text editor, code editor, or command executor.

The failure mode to avoid is a runtime-only typing demo. Runtime typing alone is not Phase 14. Editable text must travel through the full package-backed chain so that controls, packages, validation, catalog, inspection, runtime proof, and static mount all agree.

## Current reality

`ui_controls` currently exposes interaction descriptors and overlay descriptors in `ControlPackageDescriptor`. It has no canonical editable-text descriptor collection yet.

`InspectorField` currently lowers a text-intent probe through interaction lowering. That proves text intent can be observed; it is not editable behavior and must not be treated as full text editing.

`ui_input` currently owns normalized pointer, keyboard, focus, semantic, and text-intent facts. `TextIntentFact` is intentionally minimal and documented as a pre-editing probe.

`ui_runtime::input` generic interaction proof explicitly excludes full text editing. `ui_runtime::overlay` now owns completed overlay/layering proof under a separate runtime module. Phase 14 should follow that owner-first pattern with a dedicated text-editing runtime module rather than placing editing semantics inside generic input.

`ui_static_mount` validates renderer-neutral `UiFrame` proof frames. It should validate text-editing proof frames without gaining editing behavior.

## Goals

- Add reusable editable-text declaration vocabulary owned by `ui_controls`.
- Keep editable behavior package-backed and visible in catalog and inspection.
- Provide ergonomic declaration helpers/presets so base-control authors do not manually construct every descriptor field.
- Keep normalized keyboard/text/composition/focus/selection facts in `ui_input` as facts only.
- Add runtime-owned text-editing replay/report evidence under a dedicated text-editing module.
- Prove caret, selection, composition, edit intent, validation, suppression, focus, and no-bypass behavior without product mutation.
- Prove renderer-neutral visual evidence through `TextEditingVisualProof`, `TextEditingProofRenderFrame`, `UiFrame`, and `ui_static_mount`.
- Treat paste as a host-owned content source represented by normalized intent/evidence, not as clipboard ownership.
- Design explicit base-control adoption. Do not force text editing onto controls that are not semantically editable.
- Add one canonical end-to-end proof fixture that demonstrates the whole chain in one place.
- Avoid public raw byte-offset APIs for caret and selection positions.
- Reserve future extension seams for accessibility, validation, international text, clipboard, and reusable input controls without implementing those full systems in this phase.

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
- OS clipboard provider integration;
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

Own actual persistence, domain mutation, command routing, route authorization, document buffers, authored UI editing, app-specific editor policy, clipboard provider integration, and undo stacks that affect product documents.

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

## Implementation sequence

Implementation should proceed in this order:

```text
1. ui_controls editable-text vocabulary
2. ergonomic builders and presets
3. ControlPackageDescriptor editable-text descriptor field and lookup helpers
4. package validation and negative validation tests
5. base-control lowering through text_editing_support
6. catalog projection
7. inspection projection
8. ui_input normalized text/focus/selection/composition facts
9. ui_runtime text-editing replay/report state machine
10. TextEditingVisualProof and TextEditingProofRenderFrame / UiFrame projection
11. ui_static_mount validation
12. canonical end-to-end proof fixture
13. docs closeout and validation gate
```

Do not start with runtime. Runtime consumes package-declared editable behavior; it does not discover or invent editable behavior independently.

## Package/catalog/inspection requirements

Phase 14 should introduce a package-backed descriptor such as `ControlEditableTextDescriptor` with domain-shaped names, not phase-shaped names.

The descriptor should capture:

- target `control_kind_id`;
- editable mode: text field, text area, inline editable field, read-only selectable text, search/query field, or command/action prompt input where semantically valid;
- supported edit intents: insert, delete backward, delete forward, replace selection, move caret, extend selection, commit, cancel, paste intent, composition start/update/commit/cancel;
- caret support;
- selection support;
- composition/IME pre-edit support;
- focus acquisition and focus return requirements;
- disabled and read-only suppression policy;
- validation declarations such as required input mode, maximum length, empty allowance, or host-owned validation marker;
- placeholder and label metadata where available;
- accessibility-reserved metadata such as role, label, value summary, selection summary, composition summary, required state, invalid state, disabled state, and read-only state;
- host-owned paste/content source policy;
- proof flags showing zero host commands, zero product mutation, zero authored UI edits, and zero product undo operations.

`ControlPackageDescriptor` should expose package-level editable-text descriptors, builder helpers, and lookup helpers. Main-path package validation must reject duplicate descriptors, descriptors for missing control kinds, descriptors with no supported edit intents, invalid selection/composition combinations, and invalid editable declarations on controls that are not explicitly marked semantically editable.

Catalog projection should expose read-only facts such as editable mode, caret support, selection support, composition support, validation support, edit-intent labels, paste-intent support, read-only suppression, disabled suppression, accessibility-reserved metadata, and no-command/no-mutation flags.

Inspection should gain a first-class `TextEditing` section or an equally explicit existing section if a later implementation proves that a new section is unnecessary. Inspection must not hide editable facts in unrelated interaction or metadata sections.

Inspection must answer:

- Is this control editable?
- Which descriptor declared it?
- Which package provided it?
- Which control kind or lowered node owns the declaration?
- Which input facts can target it?
- Which edit intents are accepted?
- Which edit intents are suppressed?
- Which lifecycle state and policy caused the current behavior?
- Which proof frame and runtime report rows came from it?

## Authoring ergonomics requirements

Editable text must not require callers to manually fill every descriptor field. Phase 14 should provide ergonomic constructors, builders, or base-control helpers that encode coherent defaults.

The implementation should provide APIs equivalent in convenience to:

```text
EditableTextDescriptor::single_line(...)
EditableTextDescriptor::multi_line(...)
EditableTextDescriptor::read_only_selectable(...)
EditableTextDescriptor::search_field(...)
EditableTextDescriptor::command_input(...)
control.with_editable_text(...)
package.with_editable_text_descriptor(...)
```

Names may differ, but the ergonomics requirement does not. Base-control authors should be able to opt into a coherent editable mode without hand-assembling caret policy, selection policy, composition policy, suppression policy, and proof metadata every time.

Required preset coverage:

- single-line editable text;
- multi-line editable text if a semantically valid base-control candidate exists;
- read-only selectable text;
- inspector field input;
- search/query input if semantically valid;
- command/action prompt input only if typed prompt semantics are explicit.

The phase does not need to polish a public component library, but it must prove that these behavior modes are representable without duplicating descriptor boilerplate.

## Base-control adoption requirements

Likely first candidates:

- `InspectorField`, because it already has typed prompt/input semantics through text-intent probe behavior;
- `ActionPrompt` only if actual typed prompt semantics exist or are added explicitly;
- future `TextField` or `TextArea` controls if current base controls do not contain a semantically editable control.

Do not force text editing onto Label, Button, ColorPicker, ListView, TreeView, or TableView unless a later design explicitly defines an editable subrole for them. If a new base control is needed, design it as a real base control instead of hiding editing behavior inside unrelated controls.

## Text position and range policy

Public editable-text APIs must not expose raw Rust string byte offsets as caret or selection positions.

Phase 14 should define domain-shaped position/range types such as `TextPosition`, `TextRange`, `CaretPosition`, or equivalent. Exact names may differ, but the public model must preserve room for grapheme-aware cursor movement, IME composition, emoji clusters, accented characters, and future bidirectional text.

The implementation may use a simple internal representation while the proof remains narrow, but public descriptor, input fact, runtime report, and proof-frame vocabulary should avoid byte-index lock-in.

Minimum policy:

- positions are opaque or domain-shaped, not raw byte indices;
- ranges have anchor/extent or start/end semantics explicitly documented;
- collapsed selection and caret position are distinguishable but interoperable;
- composition range is represented separately from committed text range;
- invalid position movement produces suppression/evidence, not panics or silent mutation.

## Runtime state model

Runtime text editing should use an explicit lifecycle state model rather than ad hoc booleans.

Required states or equivalent evidence states:

```text
Unfocused
Focused
Editing
Composing
Selecting
Submitting
Cancelled
Suppressed
```

Required transitions or equivalent report rows:

```text
focus gained
focus lost
text inserted
delete requested
selection changed
selection collapsed
caret moved
composition started
composition updated
composition committed
composition cancelled
submit requested
cancel requested
edit suppressed
```

Runtime evidence should be generated from this shape:

```text
normalized input fact
  -> edit intent
  -> transient proof state
  -> report evidence
```

It must not become:

```text
keyboard input
  -> product string mutation
```

Edit intents such as `InsertTextRequested`, `DeleteBackwardRequested`, `ReplaceSelectionRequested`, `CommitTextRequested`, or `PasteIntentRequested` are evidence and host-consumable intent only; they must not mutate product buffers or execute product commands.

## Composition and international text policy

IME/composition must be separate from committed text.

Proof state should be able to represent examples equivalent to:

```text
committed_text: "caf"
composition_text: "é"
visual proof: caf[é composing]
```

After commit, proof state should become equivalent to:

```text
committed_text: "café"
composition_text: none
```

Composition must not be silently modeled as normal text insertion. Phase 14 does not need complete international text behavior, but it must preserve the conceptual split between committed text, pre-edit composition text, composition range, and composition cancellation.

## Clipboard and host-owned content source policy

Clipboard read/write is out of scope. Clipboard-related behavior may be represented only as normalized intent or suppression evidence.

Allowed Phase 14 proof examples:

```text
PasteIntentRequested suppressed: clipboard provider unavailable
CopyIntentRequested emitted: selection exists
CutIntentRequested suppressed: product mutation is host-owned
```

Forbidden Phase 14 behavior:

```text
generic UI reads OS clipboard
generic UI writes OS clipboard
generic UI mutates a product document because cut/paste occurred
```

## Runtime proof requirements

`ui_runtime` should produce a dedicated text-editing report with at least these evidence groups:

- package descriptor facts proving the runtime consumed package-backed editable-text declarations;
- ordered normalized input steps;
- lifecycle state transitions;
- focus acquisition and focus return evidence;
- caret placement and movement evidence;
- selection range creation, extension, replacement, and collapse evidence;
- composition/IME pre-edit start/update/commit/cancel evidence;
- edit-intent evidence for insert, delete, replace, paste, commit, and cancel;
- validation and suppression evidence;
- disabled/read-only suppression evidence;
- no-target and invalid-target evidence where relevant;
- no-bypass boundary counters.

The report should distinguish accepted intent, rejected intent, suppressed intent, and host-owned intent. It should also state why suppression occurred: no target, disabled target, read-only target, unsupported edit intent, invalid selection, invalid composition state, host-owned clipboard source unavailable, host-owned mutation required, or descriptor/policy mismatch.

## Visual proof requirements

`TextEditingVisualProof` and `TextEditingProofRenderFrame` should display behavior state, not just a final string.

Required proof elements:

- editable target identity;
- committed text/value evidence;
- caret marker;
- selection marker;
- composition marker;
- placeholder/label evidence where available;
- disabled/read-only/error/suppressed marker where relevant;
- accepted edit-intent rows;
- suppressed edit-intent rows;
- no-bypass counters;
- package/catalog/inspection provenance.

The visual proof should be renderer-neutral. It should not require a real editor surface, renderer backend, OS input system, or product document buffer.

## Positive proof scenarios

The implementation must prove:

- package-backed editable-text declaration for every semantically editable base-control candidate included in scope;
- ergonomic helpers/presets can declare editable behavior without manual descriptor plumbing;
- base-control lowering attaches editable descriptors through the compiled package path;
- package validation accepts valid editable descriptors through the main validation path;
- catalog and inspection expose editable text support without mutable internals;
- focus acquisition for an editable target;
- lifecycle transition from unfocused to focused to editing;
- caret placement from normalized focus/pointer/keyboard facts;
- caret movement by arrow, Home, End, and equivalent normalized navigation facts;
- text insertion intent;
- backward and forward deletion intent;
- replacement intent when a selection is active;
- selection extension, selection collapse, and optional select-all intent if included in the normalized vocabulary;
- composition start/update/commit/cancel as pre-edit facts separate from committed text;
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
- raw public byte-offset APIs are not introduced for caret/selection descriptors, input facts, reports, or proof frames;
- invalid caret/selection movement produces suppression/evidence instead of panic or silent mutation;
- generic UI executes zero host commands;
- generic UI performs zero product/editor/game mutations;
- generic UI performs zero authored UI edits;
- generic UI performs zero product undo/redo operations;
- generic UI performs zero plugin-framework operations;
- generic UI does not open unrelated overlay runtime paths;
- rich document editing and code editor behavior remain absent.

## Canonical end-to-end proof fixture

Phase 14 must add one canonical end-to-end fixture that proves the complete chain in one place. Unit tests may cover smaller pieces, but the phase should not be accepted if the only evidence is scattered across independent tests.

The fixture should demonstrate:

```text
package-declared editable field
  -> base-control lowering
  -> package validation
  -> catalog projection
  -> inspection projection
  -> normalized focus/text/selection/composition input facts
  -> runtime replay/report
  -> caret evidence
  -> selection evidence
  -> composition evidence
  -> accepted edit intent
  -> suppressed edit intent
  -> TextEditingVisualProof
  -> TextEditingProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

Candidate names:

```text
base_controls_text_editing_end_to_end
text_editing_package_backed
TextEditingVisualProof
```

Exact names may differ, but there must be a single canonical scenario that a reviewer can inspect to answer whether Phase 14 works as a vertical slice.

## Test ergonomics requirements

Tests should not require large hand-written descriptor graphs for every scenario. Add narrow fixture helpers where useful, such as equivalents to:

```text
given_editable_text_package()
given_focused_text_target()
insert_text("abc")
move_caret_left()
select_previous()
commit_composition("é")
assert_text_editing_report(...)
assert_text_editing_static_mount(...)
```

The helper names may differ. The requirement is that future text-editing and reusable-control tests can express intent without copying brittle setup code.

## Static mount proof requirements

Text-editing proof frames should include renderer-neutral evidence for:

- a main proof area with editable target, caret marker, selection marker, and optional composition marker;
- an inspector/evidence area with descriptor support facts;
- a report area with input, edit-intent, validation, suppression, lifecycle, and no-bypass rows;
- stable draw order;
- at least one surface;
- rect/background, border/outline, and glyph-run evidence.

`ui_static_mount` validates the frame shape. It does not own text editing behavior, text shaping policy, product state, or editor commands.

## Future considerations intentionally reserved

These are important for long-term design, but they are not Phase 14 implementation scope unless explicitly promoted later.

### Generic text rendering and layout

Future text phases should cover measurement, line wrapping, shaping, font fallback, paragraph layout, inline runs, selection geometry, caret geometry, scroll-to-caret, clipping, and text overflow.

Phase 14 may emit simple glyph-run proof evidence, but it must not become a full text layout engine.

### Reusable input controls

Future reusable controls may include:

- `TextField`;
- `TextArea`;
- `SearchField`;
- `CommandInput`;
- `NumberInput`;
- `PasswordInput`;
- `EditableInspectorField`;
- `EditableTableCell`;
- `EditableTreeLabel`.

Phase 14 should make these possible by proving behavior descriptors and runtime evidence. It does not need to implement the full polished set.

### Validation and forms

Future work may add required fields, min/max length, allowed character classes, numeric parsing, input masks, validation messages, dirty/touched state, and form-level submission.

Phase 14 may reserve validation metadata, but it must not turn into a full form framework.

### Undo/redo

Generic UI may later expose local edit transaction evidence. Product/editor layers still own command stacks and persisted document undo/redo.

Future split:

```text
generic UI: edit transaction evidence
product/editor: command stack and persistence
```

### Clipboard

Future work may introduce copy, cut, paste, clipboard provider abstraction, sanitization, and plain-text vs rich-text paste policy.

Phase 14 models or suppresses clipboard intent only.

### Accessibility

Future work may add screen reader role/value/selection reporting, IME accessibility hooks, focus traversal, label association, and error announcements.

Phase 14 reserves descriptor/report/proof vocabulary for these concepts but does not need to implement the full accessibility stack.

### International text

Future work may add grapheme-aware movement, bidirectional text, CJK IME integration, dead keys, accent composition, emoji clusters, and RTL selection behavior.

Phase 14 must avoid public byte-offset lock-in and keep composition separate from committed text.

### Rich text and code editing

Future work may cover styled spans, inline widgets, syntax highlighting, markdown editing, code editor behavior, diagnostics, language-server integration, and multi-cursor editing.

These remain out of scope for Phase 14.

### App builder / UI Designer

Future app-builder work may cover visually editing UI templates, property-inspector editing, interactive design surfaces, component authoring, and layout editing.

Phase 14 only gives the generic UI substrate reusable editable text behavior. It does not create authored UI editing.

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

Review readiness requires this gate to pass locally before completion is claimed.

## Completion criteria

Phase 14 is complete only when all of this is true:

```text
1. Editable text is declared through stable ui_controls vocabulary.
2. Base controls can opt in through ergonomic helpers/presets.
3. ControlPackageDescriptor carries editable-text descriptors.
4. Package validation rejects duplicate, unresolved, incompatible, and bypassed editable-text declarations.
5. Catalog projection exposes editable-text capability.
6. Inspection projection explains editable-text behavior and suppression policy.
7. ui_input emits normalized focus, keyboard, text, selection, composition, and edit-intent facts.
8. ui_runtime replays those facts only for package-declared editable targets.
9. Runtime report proves accepted and suppressed edit intents.
10. Runtime state distinguishes focus, editing, selection, composition, submit, cancel, and suppression.
11. Visual proof displays value, caret, selection, composition, lifecycle, provenance, and suppression evidence.
12. Static mount validates the proof frame.
13. At least one canonical end-to-end fixture proves the whole chain.
14. Public APIs avoid byte-index lock-in, phase names, compatibility aliases, and product-specific concepts.
15. No editor/game/product mutation is introduced.
16. Future extension seams for accessibility, validation, international text, clipboard intent, and reusable input controls remain clean.
17. Validation gate passes.
```

## Required stop conditions

Stop and redesign if Phase 14 requires:

- product/editor/game mutation inside generic UI;
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
- compatibility-only aliases/shims;
- phase-shaped public API names.

Additional design tripwires:

- text-editing semantics hidden under `ui_runtime::input` instead of a dedicated runtime text-editing owner path;
- editable behavior attached to controls that are not semantically editable without an explicit design update;
- public caret/selection APIs exposed as raw byte offsets;
- composition modeled as ordinary committed text insertion;
- clipboard read/write introduced inside generic UI;
- tests requiring repeated hand-built descriptor plumbing instead of reusable fixture helpers.

## Relationship to current work

Phase 13 is completed through PR #44. Phase 14 is implemented on a local review branch using the owner files, validation envelope, evidence expectations, and stop conditions supplied by the 2026-07-02 handoff. Completion remains blocked until branch acceptance or merge and completion truth recording.
