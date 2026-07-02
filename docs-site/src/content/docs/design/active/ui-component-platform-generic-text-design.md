---
title: UI Component Platform Generic Text Design
description: Phase 15 implementation-ready design for reusable renderer-neutral text display and layout proof.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-07-02
related_designs:
  - ./runenwerk-ui-platform-capability-roadmap.md
  - ./runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./ui-component-platform-text-editing-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
---

# UI Component Platform Generic Text Design

Lifecycle state: `active-planning`, implementation-ready after this design is reviewed and the docs PR is validated.

Planning ID: `PT-UI-COMPONENT-PLATFORM-015`.

Generic Text is the reusable renderer-neutral text display and layout proof for the UI Component Platform. It is display/layout infrastructure for labels, headings, body copy, helper text, badges, tabs, menu items, inspector rows, and future reusable controls that render text. It is not text editing, not a rich text editor, not a code editor, not a product document surface, and not an authored UI editor.

## Decision summary

Phase 15 must perform a clean text-display cutover. The current `ui_text` layout contract is too narrow because it accepts one string, one style, one optional width, and returns one glyph run. That shape cannot faithfully represent inline spans, multiple lines, line boxes, text direction, truncation evidence, overflow decisions, block alignment, role-based package declarations, catalog facts, inspection facts, or static proof. Do not stretch that contract with ad hoc optional fields. Replace it with a renderer-neutral text block model and layout evidence model.

The accepted design is a vertical proof chain:

```text
ui_text text block/run/span/style/layout/evidence contracts
  -> ui_render_data renderer-neutral text primitive compatibility
  -> ui_controls package-backed generic-text declarations
  -> package validation
  -> catalog projection
  -> inspection projection
  -> deterministic generic-text layout proof
  -> generic-text runtime report
  -> generic-text visual proof frame / UiFrame
  -> ui_static_mount validation
```

The proof must be complete enough for future reusable controls to consume text without reinventing layout vocabulary. It must not become a single-label demo.

## Problem

Text is a cross-cutting UI primitive. Buttons, tabs, menus, overlays, inspectors, popups, property rows, dashboards, command surfaces, and future app-building controls all need text. If each control emits glyphs directly, the system will fragment around local style, wrapping, truncation, metrics, and evidence conventions.

The previous text substrate proves only limited renderer-neutral glyph emission. It does not provide a package-visible Generic Text capability. It does not expose sufficient evidence for catalog, inspection, visual proof, or static mount validation.

The failure modes to avoid are:

- treating plain `char` emission as the text platform;
- treating one `GlyphRun` as enough evidence;
- baking product copy, editor buffers, or document mutation into generic UI;
- tying text layout to a renderer backend;
- adding compatibility aliases around the existing narrow API;
- creating a label-only phase that future controls cannot build on.

## Goals

- Define reusable renderer-neutral text display/layout vocabulary.
- Support text blocks made from one or more ordered text runs.
- Support inline spans with style overrides and semantic roles.
- Support explicit newlines, word wrapping, character wrapping, no-wrap layout, and max-line clamping.
- Support horizontal alignment and block alignment without renderer ownership.
- Support truncation and ellipsis with explicit evidence.
- Support line metrics, baselines, line boxes, content bounds, measured size, and overflow state.
- Emit glyph/run evidence that is stable enough for visual proof, static mount validation, and future renderer consumption.
- Avoid `char` as the public glyph identity; use cluster/glyph evidence that can later support shaping, ligatures, fallback fonts, emoji, and bidirectional text.
- Keep `ui_render_data` primitives compatible with the new text layout evidence without introducing renderer backend ownership.
- Make generic text capability visible through package validation, catalog projection, and inspection projection.
- Provide a renderer-neutral visual proof frame that shows source, layout, lines, glyph evidence, truncation, and boundary assertions.
- Add static mount proof for the generic text frame.
- Preserve owner boundaries between `ui_text`, `ui_render_data`, `ui_controls`, `ui_runtime`, `ui_static_mount`, renderer backends, and product/editor/game layers.
- Avoid public phase-shaped API names and compatibility-only aliases/shims.

## Explicit non-goals

This phase does not implement or authorize text editing, rich text editor behavior, code editor behavior, product document buffers, undo/redo, clipboard integration, LSP/syntax highlighting, app-specific localization policy, app-specific typography policy, renderer backend implementation, GPU atlas upload, OS font discovery, full Unicode shaping backend integration, dynamic plugin framework, `foundation/meta`, shared plugin primitives, UI Designer, UI Gallery product surface, Workbench/provider redesign, product/editor/game mutation, command execution, authored UI editing, compatibility-only aliases/shims, or phase-shaped public API names.

Important nuance: the contracts must be future-proof for shaping, fallback, emoji, bidirectional text, and localization, but Phase 15 does not need a production shaping engine. The deterministic proof may use a simple internal measurer/shaper, provided the public evidence model is not limited to one character equals one glyph.

## Owner boundaries

`ui_text` owns renderer-neutral text display/layout contracts and evidence vocabulary. It owns text block declarations, run/span contracts, style contracts, layout constraints, line metrics, glyph/cluster evidence, overflow evidence, deterministic proof layout helpers, and validation-local text layout tests.

`ui_render_data` owns renderer-neutral primitive transport for already-laid-out text evidence inside `UiFrame`. If the old glyph-run primitive depends on the retired single-run layout type, Phase 15 must update that primitive to carry the new visual-run/layout-evidence shape. It must not add backend-specific rendering, GPU atlas upload, font discovery, or product text policy.

`ui_controls` owns package-backed generic-text declarations for reusable controls. It owns descriptor attachment, package validation, catalog projection, and inspection projection. It does not own glyph placement or renderer backend policy.

`ui_runtime` owns generic-text proof reporting and proof-frame projection. It may consume `ui_text` layout evidence and turn it into renderer-neutral `UiFrame` primitives. It does not own product text state or renderer backend implementation.

`ui_static_mount` owns static validation of the renderer-neutral generic-text proof frame. It validates that the frame carries the expected text evidence, draw order, measured bounds, and no-bypass boundary assertions.

Renderer backends may later consume text layout/glyph evidence. Backend implementation is not Phase 15 scope.

Host/product/editor/game layers own app-specific copy, localization selection, content persistence, document buffers, authored UI editing, code editing, undo/redo, commands, domain mutation, and product-specific text semantics.

## Existing editable-text primitives in `ui_text`

`ui_text` currently also contains editable-text support modules such as buffer, cursor, and selection primitives. Phase 15 must not redesign or remove those modules as part of the Generic Text cutover. They may receive only mechanical compile adaptations caused by renamed layout/style exports.

The display/layout cutover must be isolated from editing semantics. Any future split between display text and editable text ownership is a separate design decision, not an implicit side effect of Phase 15.

## Required clean cutover

The current `TextLayoutRequest { text, style, max_width } -> Option<GlyphRun>` model must not remain the primary Generic Text contract. Phase 15 should introduce a new block-oriented model and either remove, demote, or adapt the old model behind the new tests.

Allowed migration shape:

```text
TextBlockLayoutRequest
TextBlockLayoutResult
TextRun
TextSpan
TextCluster
TextGlyph
TextLineMetrics
TextOverflowEvidence
```

The exact names may change during implementation, but the model must have these concepts. Do not add a parallel `Phase15Text*` API. Do not add aliases whose only purpose is compatibility.

## Core vocabulary

### Text block

A text block is the renderer-neutral display unit. It is not a document buffer and not an editable field.

Required fields:

```text
text_block_id: stable proof-local id
runs: ordered non-empty list of TextRun
base_style: TextStyle
layout: TextLayoutPolicy
semantic_role: optional TextSemanticRole
accessibility_label_role: optional role string
```

A block may represent a label, body paragraph, heading, helper line, badge, tab label, menu item, inspector label, tooltip text, or proof fixture. The block must not own product persistence.

### Text run

A text run is an ordered source fragment with a style and semantic role.

Required fields:

```text
run_id: stable proof-local id
text: String
style: TextSpanStyle or resolved TextStyle
semantic_role: optional TextSemanticRole
source_range: optional logical source range
```

Run order is source order. Empty runs are invalid unless explicitly used as diagnostic fixtures.

### Inline span

A span is a display-only style/semantic subdivision that is lowered into ordered source runs or style segments before layout. Keep authored span identity in evidence, but do not let spans become product document annotations or editable ranges.

Supported span features:

- foreground color token or resolved color;
- font family/id reference;
- font size;
- font weight category;
- font style category;
- underline/strikethrough decoration flags;
- semantic role;
- proof-local source range.

Out of scope for Phase 15 spans:

- embedded widgets;
- inline images;
- hyperlinks that execute commands;
- selectable/editable ranges;
- product document annotations.

### Text style

`TextStyle` should become a block/run style contract, not merely glyph emission style.

Required style fields or equivalents:

```text
font_id or font_family_ref
font_size
line_height policy
color or color token
font_weight
font_style
text_decoration
letter_spacing policy, default normal
```

Existing style fields such as `TextAlign`, `TextWrap`, `TextOverflow`, and vertical alignment should move into layout policy if that produces cleaner separation. Style describes appearance; layout policy describes placement and overflow.

### Layout policy

Required policy fields:

```text
width_constraint: unconstrained | exact | max
height_constraint: unconstrained | exact | max
wrap: no_wrap | word | character
whitespace: preserve | collapse_runs | trim_edges
horizontal_align: start | center | end
vertical_align: start | center | end | baseline
overflow: clip | ellipsis
ellipsis: end | start | middle
max_lines: optional non-zero integer
text_direction: auto | ltr | rtl
line_height: font_default | multiplier | absolute
```

Minimum implementation proof must cover no-wrap, word wrap, character wrap, start/center/end horizontal alignment, clipping, end ellipsis, max-line clamping, and explicit newline handling. The model must include start/middle ellipsis and direction policy so the API does not become single-locale/single-line only, even if some advanced policies initially emit deterministic diagnostics.

### Text position and range units

Public text positions must not expose Rust byte offsets. Phase 14 already established this direction for editable text. Generic Text should use the same discipline.

Allowed position units:

```text
opaque ordinal
grapheme or cluster ordinal
line/column
```

Layout evidence may include internal UTF-8 byte ranges only if they remain private implementation details. Public reports, catalog facts, and inspection facts must use logical text units.

## Deterministic proof layout algorithm

The implementation must include a deterministic proof layouter. It is allowed to be simple, but its behavior must be specified and tested so Phase 15 does not become subjective renderer behavior.

Algorithm rules:

1. Lower text blocks into ordered source runs and style segments before layout.
2. Split source text into proof clusters. The initial proof cluster may be one Unicode scalar value, but the public evidence type must be named and shaped as a cluster so later shaping can map many scalars to one glyph or one scalar to multiple glyphs.
3. Resolve font metrics from the deterministic proof atlas. Missing glyphs use a stable replacement glyph and emit fallback evidence.
4. Resolve line height before line placement. The default line height must be at least the scaled font default. Absolute or multiplier line-height policies must be recorded in line metrics.
5. Apply explicit newline breaks before width wrapping.
6. Apply whitespace policy before word wrapping. `preserve` keeps source whitespace clusters. `collapse_runs` collapses consecutive horizontal whitespace for measurement and evidence. `trim_edges` trims leading/trailing line-edge whitespace after wrapping.
7. `no_wrap` produces one visual line per explicit source line and records horizontal overflow when content exceeds the width constraint.
8. `word` wrapping prefers the last legal whitespace boundary before the width limit. If no boundary exists, it falls back to cluster wrapping and records that fallback in evidence.
9. `character` wrapping breaks at cluster boundaries.
10. Horizontal alignment shifts each visual line origin inside the width constraint after wrapping/truncation. Alignment evidence must record start, center, or end.
11. Max-line clamping is applied after wrapping and before ellipsis placement.
12. Clip overflow records visible range and omitted cluster counts without inserting ellipsis glyph evidence.
13. End ellipsis replaces the last visible clusters that fit after max-line clamping or no-wrap overflow. The proof must record ellipsis as an overflow decision even when the visual fallback is three dots rather than a single ellipsis glyph.
14. Start and middle ellipsis policies must exist in the model. If not implemented in the deterministic proof layouter, they must produce explicit unsupported-policy diagnostics rather than silently behaving like end ellipsis.
15. Text direction may default to left-to-right for the deterministic proof. `auto` and `rtl` must exist in the model. Unsupported direction behavior must be explicit in diagnostics or evidence.

The deterministic proof layouter must return a complete layout result, not `Option<GlyphRun>`. Failure cases must return diagnostics with enough context for tests and inspection.

## Layout result and evidence model

The layout result is the core of this phase. It must be richer than one glyph run.

Required result fields:

```text
block_id
input_run_count
resolved_run_count
line_count
glyph_run_count
glyph_count
measured_size
content_bounds
ink_bounds
line_metrics
visual_runs
overflow_evidence
fallback_evidence
```

Boundary/no-bypass assertions belong in the runtime proof report and static mount proof, not inside the pure `ui_text` layout result.

### Line metrics

Each line must carry evidence:

```text
line_index
source_range
visual_order
origin
baseline_y
ascent
descent
line_gap
line_height
line_box
content_width
ink_bounds
is_wrapped
is_explicit_break
is_truncated
```

Line metrics must be stable and testable. Baseline and line-box evidence are required even for one-line text.

### Glyph and cluster evidence

The public evidence model must support shaping and fallback even if the deterministic proof shaper is simple.

Each visual glyph record should include:

```text
draw_order
line_index
run_id
span_id or style segment id
font_id
glyph_id or stable glyph key
cluster_range
origin
advance
bounds
source_text_preview for proof only
```

A visual run groups glyphs that share font, resolved style, line, and direction. A visual run is not the same as an authored text run.

The proof must not require one Unicode scalar value to become exactly one glyph. That would block ligatures, emoji, fallback fonts, and shaping later.

### Overflow and truncation evidence

Overflow evidence must state:

```text
horizontal_overflow: bool
vertical_overflow: bool
clipped: bool
ellipsized: bool
ellipsis_placement
max_lines_applied
omitted_cluster_count
omitted_run_count
visible_source_range
```

A test should prove that ellipsis is an evidence decision, not just the presence of a glyph. A layout can be ellipsized even if the visual glyph fallback is `...` instead of `…`.

### Fallback and missing glyph evidence

Generic Text must record fallback decisions:

```text
requested_font
resolved_font
missing_glyph_count
fallback_glyph_count
replacement_glyph_count
```

The proof may use a deterministic test atlas and a replacement glyph. The evidence model must make missing/fallback behavior inspectable.

## Package descriptor design

Add a reusable control declaration for text display support.

Candidate shape:

```text
ControlGenericTextDescriptor
  control_kind_id
  text_roles: Vec<ControlGenericTextRoleDescriptor>
  supported_layout: ControlGenericTextLayoutSupport
  supported_overflow: Vec<TextOverflowPolicy>
  supported_wrap: Vec<TextWrapPolicy>
  supports_inline_spans
  supports_line_metrics
  supports_glyph_evidence
  supports_fallback_evidence
  renderer_backend_required: false
  proof_required: true
```

Role descriptor shape:

```text
role_id: stable string such as label.primary, helper, body, badge, tab.label
semantic_role: label | heading | body | helper | badge | tooltip | menu_item | inspector_label | inspector_value
required: bool
inline_spans_allowed: bool
max_lines: optional non-zero integer
preferred_wrap: no_wrap | word | character
preferred_overflow: clip | ellipsis
```

The descriptor must describe what a reusable control can display, not the product copy itself. Product layers remain responsible for actual copy/localization.

## Package validation

Validation must reject or report:

- generic-text descriptor referencing a missing control kind;
- duplicate generic-text descriptors for the same control kind;
- duplicate text role IDs within a descriptor;
- empty text role IDs;
- invalid role naming that cannot be inspected deterministically;
- `max_lines = 0`;
- inline spans declared unsupported while a fixture requires inline spans;
- ellipsis declared without measurable width/line constraint in proof fixture;
- renderer backend requirement inside a generic UI descriptor;
- command execution, product mutation, authored UI editing, document-buffer ownership, undo/redo, or clipboard ownership claims;
- compatibility-only descriptor aliases.

Validation must add dedicated generic-text validation reasons instead of reusing editable-text reasons. Expected variants or equivalent names:

```text
DuplicateGenericTextDescriptor
UnresolvedGenericTextDescriptor
InvalidGenericTextDescriptor
InvalidGenericTextRole
UnsupportedGenericTextLayoutPolicy
```

Validation should preserve the existing descriptor/package style: narrow builders, stable sorted summaries, deterministic diagnostics, and no global plugin mechanism.

## Catalog projection

Catalog entries should expose read-only text-display facts.

Required catalog fields or equivalent summary facts:

```text
generic_text_supported
text_roles
text_semantic_roles
text_wrap_policies
text_overflow_policies
text_alignment_policies
inline_spans_supported
line_metrics_supported
glyph_evidence_supported
fallback_evidence_supported
renderer_backend_required
control_owned_runtime_behavior
executes_host_commands
mutates_product_state
```

For Generic Text, the last three boundary fields must remain false unless a future separate design explicitly changes ownership.

## Inspection projection

Add a `TextDisplay` inspection section or equivalent. Do not overload `TextEditing`.

Required inspection facts:

```text
text_display.supported
text_display.roles
text_display.semantic_roles
text_display.inline_spans_supported
text_display.wrap
text_display.overflow
text_display.alignment
text_display.line_metrics_supported
text_display.glyph_evidence_supported
text_display.fallback_evidence_supported
text_display.renderer_backend_required
text_display.executes_host_commands
text_display.mutates_product_state
text_display.authored_ui_edits
text_display.product_undo_redo
```

The facts must be deterministic, sorted where order is not semantically meaningful, and directly traceable to package descriptors and layout evidence.

## Runtime proof report

`ui_runtime` should introduce a generic-text proof report. The exact type names may vary, but the report must include:

```text
proof_id
descriptor_evidence
source_block_evidence
layout_request_evidence
layout_result_evidence
line_metric_evidence
glyph_run_evidence
overflow_evidence
catalog_projection_evidence
inspection_projection_evidence
static_mount_expectations
boundary_assertions
```

Boundary assertions must prove:

```text
host_commands_executed = 0
product_mutations = 0
authored_ui_edits = 0
product_undo_redo_operations = 0
plugin_framework_operations = 0
renderer_backend_operations = 0
```

The report must distinguish authored/source runs from visual runs. This is required for future shaping and fallback.

## Visual proof frame

The Generic Text proof frame should make the proof visible without relying on a renderer backend.

Required frame panels:

1. **Source panel**: control kind, text roles, source runs, span roles, layout policy.
2. **Layout panel**: line boxes, baselines, measured size, content bounds, overflow status.
3. **Evidence panel**: glyph run count, glyph count, draw order, truncation, fallback, catalog/inspection facts, boundary assertions.

The `UiFrame` must contain renderer-neutral primitives only. It may use rectangles for line boxes and an updated text/glyph primitive that carries visual-run layout evidence. The summary struct must expose counts and booleans that tests can assert without screenshot comparison.

Minimum render summary fields:

```text
source_blocks
source_runs
inline_spans
line_count
glyph_run_count
glyph_count
wrapped_lines
aligned_lines
truncated_lines
fallback_rows
catalog_rows
inspection_rows
has_source_layout_and_evidence_panels
no_bypass_proven
```

## Static mount proof

`ui_static_mount` must validate that the Generic Text proof frame:

- contains a source/layout/evidence panel structure;
- contains at least one renderer-neutral text/glyph primitive;
- contains line-box/baseline evidence;
- proves inline spans;
- proves no-wrap layout;
- proves wrapping layout;
- proves horizontal alignment;
- proves truncation/ellipsis evidence;
- proves fallback or replacement-glyph evidence;
- exposes package descriptor evidence;
- exposes catalog and inspection projection evidence;
- carries no product/editor/game mutation;
- carries no command execution;
- carries no authored UI editing;
- carries no dynamic plugin/framework dependency.

Static mount should validate report/frame summaries and stable primitive facts, not pixel snapshots.

## Required fixtures and tests

Phase 15 is not complete until these scenarios are covered by focused tests:

1. **Simple label**: one block, one run, no wrap, no overflow.
2. **Inline spans**: one block with at least three resolved style/semantic spans.
3. **Explicit newline**: two lines caused by source newline, with line metrics.
4. **Word wrap**: text wraps at a word boundary under max width.
5. **Character wrap**: long unbreakable text wraps by character/cluster.
6. **Start alignment**: line origin uses logical start.
7. **Center alignment**: line origin changes and evidence records center alignment.
8. **End alignment**: line origin changes and evidence records end alignment.
9. **Clip overflow**: overflow detected without ellipsis.
10. **End ellipsis**: visible source range is shortened and ellipsis evidence is emitted.
11. **Max-line clamp**: multi-line text clamps to a configured max line count.
12. **Fallback glyph**: missing glyph/replacement evidence is emitted.
13. **Render-data primitive compatibility**: `UiFrame` can carry the new visual-run/text evidence without depending on the retired single-run layout type.
14. **Generic-text validation reasons**: duplicate, unresolved, invalid role, unsupported policy, and backend-required descriptors use dedicated generic-text diagnostics.
15. **Catalog projection**: descriptor facts appear in catalog entry.
16. **Inspection projection**: text display facts appear in inspection descriptor under text display, not text editing.
17. **Runtime proof frame**: `UiFrame` summary proves source/layout/evidence panels.
18. **Static mount acceptance**: static mount validates the renderer-neutral proof frame.
19. **Negative validation**: duplicate role, missing kind, invalid max lines, unsupported span, and backend-required descriptor produce diagnostics.
20. **Boundary/no-bypass**: no commands, product mutation, authored UI edits, undo/redo, renderer backend ownership, or plugin operations.

## Implementation roadmap

### Step 1: Replace the core text layout contract

Implement the new block/run/span/layout/evidence model in `ui_text`. Keep the old single-run tests only if they are rewritten against the new model. Do not preserve the old API as the primary contract.

Expected owner files:

```text
domain/ui/ui_text/src/lib.rs
domain/ui/ui_text/src/style.rs
domain/ui/ui_text/src/layout.rs
domain/ui/ui_text/src/atlas.rs
```

New files are acceptable if they improve separation, for example:

```text
domain/ui/ui_text/src/block.rs
domain/ui/ui_text/src/evidence.rs
domain/ui/ui_text/src/policy.rs
domain/ui/ui_text/src/proof_layout.rs
```

Do not redesign `domain/ui/ui_text/src/buffer.rs`, `cursor.rs`, or `selection.rs` as part of this step. Those modules remain editing-related substrate from earlier work unless a separate design explicitly changes ownership.

### Step 2: Keep render-data primitives compatible with the cutover

Update the renderer-neutral text primitive surface if it currently depends on the retired `GlyphRun` shape. This is not a renderer backend task; it is the `UiFrame` transport layer for already-laid-out text evidence.

Expected owner files:

```text
domain/ui/ui_render_data/src/primitives/glyph_run.rs
domain/ui/ui_render_data/src/primitives/mod.rs
domain/ui/ui_render_data/src/primitives/ui_primitive.rs
domain/ui/ui_render_data/src/lib.rs
```

### Step 3: Add package-backed Generic Text declarations

Add a `ui_controls` descriptor module for generic text display. Wire it into `ControlPackageDescriptor` in the same style as interaction, overlay, and editable text descriptors.

Expected owner files:

```text
domain/ui/ui_controls/src/generic_text.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package/descriptor.rs
domain/ui/ui_controls/src/package/validation.rs
domain/ui/ui_controls/src/package/generic_text_validation.rs
```

### Step 4: Project catalog and inspection facts

Add Generic Text summaries to catalog and inspection. Use a distinct `TextDisplay` inspection section or equivalent, not the existing `TextEditing` section.

Expected owner files:

```text
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/inspection.rs
```

### Step 5: Add deterministic layout proof fixtures

Create fixture helpers that make it easy for future controls to request text proof without duplicating layout boilerplate. Builders are allowed when they reduce ceremony and remain narrow.

### Step 6: Add runtime report and visual proof frame

Add a generic-text runtime proof module. The proof frame must expose source, layout, and evidence panels through renderer-neutral `UiFrame` primitives.

Expected owner area:

```text
domain/ui/ui_runtime/src/generic_text/
domain/ui/ui_runtime/tests/
```

### Step 7: Add static mount validation

Add static mount tests that validate the Generic Text proof frame and no-bypass assertions.

Expected owner area:

```text
domain/ui/ui_static_mount/tests/
```

### Step 8: Run validation and close out

Implementation validation must include focused crate tests plus full workspace/doc validation before merge.

```text
cargo test -p ui_text
cargo test -p ui_render_data
cargo test -p ui_controls
cargo test -p ui_runtime
cargo test -p ui_static_mount
cargo test --workspace
python tools/docs/validate_docs.py
git diff --check
```

If the repo has narrower existing validation commands, record the exact commands run in the PR body and closeout docs.

## Ergonomics requirements

Generic Text must be easy for future control authors to consume.

Required ergonomics:

- builder for simple one-run labels;
- builder for inline-span blocks;
- builder for common layout policies: no-wrap label, wrapping body, badge, tab label, inspector row value;
- helper for fixture text blocks;
- helper for sorted summary facts;
- helper for proof report creation from descriptor plus fixture block;
- no repeated manual assembly of line metrics in control code;
- no repeated manual catalog/inspection wiring per fixture;
- no product/app-specific text policy hidden in helpers.

The API should allow a future control author to declare text support in a few lines while still producing full evidence.

## Future-facing considerations kept out of Phase 15 implementation

The data model must leave room for:

- production Unicode shaping;
- bidirectional layout;
- emoji and multi-codepoint grapheme clusters;
- font fallback chains;
- localization-provided copy;
- semantic text roles for accessibility;
- renderer backend consumption of glyph evidence;
- richer typographic controls;
- selectable text as a separate future capability;
- rich text editing as a separate future capability.

Do not implement these as product features in Phase 15 unless they are needed for the renderer-neutral proof. The important requirement is that the Phase 15 contract does not block them.

## Acceptance criteria

Phase 15 can be considered complete only when:

- `ui_text` has a block/run/span/layout/evidence model, not only a single string-to-glyph-run API;
- existing `ui_text` buffer/cursor/selection editing primitives are not redesigned as part of Generic Text;
- the deterministic proof layouter follows the documented wrapping, whitespace, alignment, overflow, ellipsis, fallback, and diagnostics rules;
- `ui_render_data` can carry the new visual-run/text evidence in `UiFrame` without depending on the retired single-run layout shape;
- inline spans are declared, laid out, and visible in proof evidence;
- no-wrap, word-wrap, character-wrap, alignment, explicit newline, clip overflow, ellipsis, and max-line clamping are proven;
- line metrics include baseline, line box, content width, measured size, and truncation state;
- glyph evidence includes visual order, run/span linkage, font identity, cluster/range evidence, origin, advance, and bounds;
- package descriptors can declare Generic Text support;
- package validation rejects invalid or boundary-breaking declarations with dedicated generic-text validation reasons;
- catalog projection exposes text display support;
- inspection projection exposes text display facts separately from text editing;
- runtime proof report exposes descriptor/source/layout/glyph/overflow/projection/boundary evidence;
- visual proof frame exposes source/layout/evidence panels through `UiFrame`;
- static mount validates the proof frame;
- no product/editor/game mutation is introduced;
- no command execution is introduced;
- no authored UI editing is introduced;
- no renderer backend implementation is introduced;
- no dynamic plugin, `foundation/meta`, or shared plugin primitive dependency is introduced;
- no compatibility-only aliases/shims or phase-shaped public APIs are introduced;
- docs closeout records actual validation commands and results.

## Stop conditions

Stop and redesign if implementation attempts any of the following:

- preserves the current single-run layout API as the primary Generic Text contract;
- leaves `ui_render_data` tied to a retired single-run text layout type after the cutover;
- redesigns existing `ui_text` buffer/cursor/selection editing primitives as part of Generic Text;
- represents public glyph evidence only as `char` plus position;
- omits line metrics from the proof;
- omits inline spans from the proof;
- treats ellipsis as only a rendered character without overflow evidence;
- silently maps unsupported start/middle ellipsis or unsupported text direction to another policy without diagnostics;
- stores product copy or localization policy inside package descriptors;
- places text editing, selection, clipboard, undo/redo, or document-buffer ownership in Generic Text;
- requires renderer backend implementation to complete the proof;
- mutates product/editor/game state;
- executes host commands;
- edits authored UI;
- introduces dynamic plugin framework, `foundation/meta`, or shared plugin primitives;
- introduces compatibility-only aliases/shims;
- uses phase-shaped public API names.

## Relationship to completed work

Phase 14 is completed through PR #46 and remains the editable-text behavior proof. Phase 15 builds adjacent display/layout infrastructure. It must not reopen Phase 14 text editing scope.

Generic Text may later be consumed by editable text rendering, inspectors, popups, menus, tabs, galleries, and app-building surfaces. That reuse must come from clean display/layout contracts, not from mixing editing behavior into this phase.
