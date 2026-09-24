---
title: UI Component Platform Generic Text Design
description: Implemented Runenwerk-local renderer-neutral text block, layout, evidence, package projection, runtime proof, and static-mount contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-text-editing-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Generic Text Design

## Status

Implemented. Generic Text is the current renderer-neutral display/layout path for reusable Runenwerk UI text. It is distinct from editable-text behavior and from product document/editor ownership.

## Implemented flow

```text
ui_text block/run/span/style/layout/evidence contracts
  -> ui_render_data renderer-neutral text primitive transport
  -> ui_controls package-backed generic-text declarations
  -> package validation / catalog / inspection
  -> deterministic text-block layout proof
  -> ui_runtime generic-text report and proof frame
  -> ui_static_mount validation
```

## Current text model

The live `ui_text` path is block-oriented and evidence-producing. It supports ordered text runs/spans, resolved styles, explicit newlines, no-wrap/word/character wrapping, line metrics and baselines, alignment, clipping/truncation evidence, max-line handling, cluster/glyph evidence, fallback/missing-glyph evidence, measured/content/ink bounds, and deterministic proof layout through `layout_text_block`.

The public evidence model does not assume one Unicode scalar equals one glyph. Cluster/glyph evidence keeps later shaping/fallback/emoji/bidirectional evolution representable without turning this phase into a production font engine.

## Ownership

- `ui_text` owns renderer-neutral text display/layout contracts and deterministic layout evidence.
- `ui_render_data` transports already-laid-out renderer-neutral text evidence.
- `ui_controls` owns package-backed Generic Text support declarations, validation, catalog, and inspection projection.
- `ui_runtime` owns Generic Text proof reporting/frame projection.
- `ui_static_mount` owns static proof validation.
- product/editor/game layers own actual copy, localization choice, persistence, document buffers, rich/code editing, commands, and product semantics.

## Intentional landed differences

The proposal required retirement/demotion of the old `ui_text` single-string request returning an optional glyph run. Current `ui_text` no longer uses that as its Generic Text authority; the block-oriented layout path is live.

A type named `TextLayoutRequest` still exists in the separate UiProgram runtime-artifact/evaluator tables. That row is a compiled-program artifact contract, not the retired `ui_text` `{ text, style, max_width } -> Option<GlyphRun>` API. The name overlap is therefore not a Generic Text compatibility path or failed cutover.

Exact proposal-era type names were refined during implementation; the durable semantic requirements are the block/run/span/layout/evidence responsibilities above.

## Boundary rules

- Generic Text is display/layout infrastructure, not text editing.
- It does not own product document state, clipboard, undo/redo, code-editor/LSP behavior, localization policy, renderer backend resources, or GPU atlas upload.
- Public text positions/evidence must not make Rust UTF-8 byte offsets the semantic API.
- Unsupported advanced policies must fail or report evidence explicitly rather than silently changing meaning.
- Package support does not by itself grant mount eligibility.

## Evidence

Current `ui_text` source (including `proof_layout.rs` and evidence contracts), `ui_render_data` text transport, `ui_controls` Generic Text descriptors/projection, `ui_runtime` proof path, and focused/static-mount tests establish the implemented contract.
