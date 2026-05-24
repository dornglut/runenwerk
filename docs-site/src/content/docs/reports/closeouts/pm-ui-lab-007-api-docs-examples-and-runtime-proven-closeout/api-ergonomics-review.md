---
title: PM-UI-LAB-007 Public API Ergonomics Review
description: Public API discoverability review for UI definition and editor definition workflow entry points.
status: completed
owner: editor
layer: domain/app/docs
canonical: true
last_reviewed: 2026-05-24
related:
  - ../../../domain/ui/ui-definition-usage.md
  - ../../../domain/editor/editor-definition-usage.md
  - ../../../design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
  - ./closeout.md
---

# PM-UI-LAB-007 Public API Ergonomics Review

## Scope

This review covers the public API experience required for PM-UI-LAB-007:

- `domain/ui/ui_definition/src/lib.rs`
- `domain/ui/ui_definition/src/prelude.rs`
- `domain/ui/ui_definition/src/workflow.rs`
- `domain/ui/ui_definition/examples/ui_definition_workflow.rs`
- `domain/editor/editor_definition/src/lib.rs`
- `domain/editor/editor_definition/src/prelude.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/editor/editor_definition/examples/editor_definition_workflow.rs`
- `docs-site/src/content/docs/domain/ui/ui-definition-usage.md`
- `docs-site/src/content/docs/domain/editor/editor-definition-usage.md`

## Findings

No blocking public API ergonomics issues remain for the bounded Editor Lab V1
closeout.

The prior friction was real: both crates exposed every module through broad
crate-level exports, so normal users had to infer the authoring path from
module names, tests, and closeout artifacts. PM007 adds focused `prelude` and
`workflow` modules so normal users can start with explicit workflow functions
instead of discovering the API through internal module topology.

## UI Definition Review

Normal imports are now discoverable through:

```rust
use ui_definition::prelude::*;
```

The prelude exposes the normal behavior-free workflow:

- `validate_ui_template`
- `normalize_ui_template`
- `apply_ui_layout_operation`
- `preview_ui_layout_operation`
- `validate_ui_preview_library`
- `validate_ui_persistence_flow`
- `validate_ui_readiness`

The public example compiles and runs through the prelude. It validates and
normalizes an `AuthoredUiTemplate`, applies a generic visual layout operation,
and validates preview, persistence, and readiness descriptor libraries without
app/runtime shortcuts.

Ownership check: `ui_definition` still exposes descriptors, operations,
reports, diffs, and diagnostics only. It does not own provider sessions,
project IO, live activation, rollback, screenshots, accessibility runners,
performance runners, or artifact writing.

## Editor Definition Review

Normal imports are now discoverable through:

```rust
use editor_definition::prelude::*;
```

The prelude exposes the normal runtime-neutral workflow:

- `new_editor_definition_document`
- `validate_editor_document`
- `editor_document_has_blocking_diagnostics`
- `apply_editor_lab_edit`
- `form_editor_theme_tokens`

The public example compiles and runs through the prelude. It creates a current
editor definition document, validates it, applies an `EditorLabOperation`, and
inspects the accepted report and deterministic diff.

Ownership check: `editor_definition` remains runtime-neutral. It does not own
app project files, provider sessions, live activation, failed activation
preservation, rollback, runtime scenario execution, or runtime evidence
artifact storage.

## Docs And Examples Review

The domain index pages now link usage docs before architecture-only material:

- `docs-site/src/content/docs/domain/ui/README.md`
- `docs-site/src/content/docs/domain/editor/README.md`

The usage docs teach the focused workflow path first and then state the app
handoff boundaries. They do not present test-only helpers or app-owned runtime
shortcuts as normal domain API.

The examples were run with:

```text
cargo run -p ui_definition --example ui_definition_workflow
cargo run -p editor_definition --example editor_definition_workflow
```

## Remaining Gaps

The bounded API ergonomics work is complete for Editor Lab V1 closeout, but it
does not claim `perfectionist_verified`:

- the broad crate-level glob exports still exist for compatibility;
- a future no-gap audit may decide whether to narrow or reorganize advanced
  exports;
- native screenshot, GPU visual diff, native focus traversal, contrast
  sampling, and deeper accessibility/performance platform checks remain later
  audit scope;
- game-runtime UI projection execution remains out of this Editor Lab V1
  closeout.
