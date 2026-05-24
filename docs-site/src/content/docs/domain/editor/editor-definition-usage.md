---
title: Editor Definition Usage
description: Normal public workflow for runtime-neutral editor definition documents and Editor Lab operations.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-24
related:
  - ./README.md
  - ./editor-definition/current-architecture.md
  - ../../design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
  - ../../reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md
---

# Editor Definition Usage

`editor_definition` owns runtime-neutral editor definition documents and
Editor Lab operation contracts. It is the right place to construct editor
definition documents, validate them, apply typed `EditorLabOperation` values,
inspect operation reports, and form editor theme tokens.

It does not own editor app project files, provider sessions, live activation,
failed activation preservation, rollback, runtime scenario execution, or
artifact writing. Those are owned by `apps/runenwerk_editor`.

## Import Path

Normal users should start with the focused prelude:

```rust
use editor_definition::prelude::*;
```

The prelude is backed by:

- `domain/editor/editor_definition/src/prelude.rs`
- `domain/editor/editor_definition/src/workflow.rs`
- `domain/editor/editor_definition/examples/editor_definition_workflow.rs`

The crate-wide exports remain available for compatibility, but the prelude is
the documented normal path.

## Normal Workflow

The normal runtime-neutral path is:

```text
EditorDefinitionDocumentContent
  -> new_editor_definition_document
  -> validate_editor_document
  -> apply_editor_lab_edit
  -> inspect EditorLabOperationReport diagnostics and diff
  -> hand resulting documents to app-owned Editor Lab project/apply flow
```

Use `new_editor_definition_document` to create a versioned current document.
Use `validate_editor_document` and
`editor_document_has_blocking_diagnostics` before applying or handing off a
document.

Use `apply_editor_lab_edit` for supported runtime-neutral Editor Lab edits,
including UI text edits, generic UI visual layout operations, document rename,
theme color edits, and workspace layout operations. The report carries the
resulting document, operation status, deterministic diff, and diagnostics.

Use `form_editor_theme_tokens` when the caller needs to validate and form
editor theme definitions into `ui_theme` tokens without entering the app
runtime.

## Complete Example

Run the public example from the workspace root:

```text
cargo run -p editor_definition --example editor_definition_workflow
```

The example:

- creates a current editor definition document containing a UI template;
- validates it through `editor_definition::prelude`;
- applies a typed `EditorLabOperation`;
- inspects the accepted report and deterministic diff;
- avoids app-owned runtime shortcuts.

## App Handoff

After domain validation and operation application, app-owned Editor Lab flows
take over for project IO, apply review, live activation, failed activation
preservation, reload, rollback, and runtime evidence. Those product paths are
proved by the PM-UI-LAB-005 and PM-UI-LAB-006 closeouts.

`editor_definition` should only receive typed documents, operations, reports,
diffs, and diagnostics. It should not gain app-specific file locations,
provider state, command dispatch, or runtime artifact storage.
