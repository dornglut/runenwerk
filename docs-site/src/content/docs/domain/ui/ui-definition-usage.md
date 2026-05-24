---
title: UI Definition Usage
description: Normal public workflow for behavior-free authored UI definitions.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-05-24
related:
  - ./README.md
  - ../../design/accepted/ui-lab-api-docs-examples-runtime-closeout-design.md
  - ../../reports/closeouts/pm-ui-lab-007-api-docs-examples-and-runtime-proven-closeout/closeout.md
---

# UI Definition Usage

`ui_definition` owns behavior-free UI definition contracts. It is the right
place to author templates, validate and normalize them, apply generic visual
layout operations, validate preview fixture descriptors, validate persistence
activation descriptors, and inspect production readiness descriptors.

It is not the right place for editor command execution, provider sessions,
project files, live activation, rollback, screenshots, accessibility runners,
performance runners, or artifact writing. Those stay in the owning editor app
or runtime boundary.

## Import Path

Normal users should start with the focused prelude:

```rust
use ui_definition::prelude::*;
```

The prelude is backed by:

- `domain/ui/ui_definition/src/prelude.rs`
- `domain/ui/ui_definition/src/workflow.rs`
- `domain/ui/ui_definition/examples/ui_definition_workflow.rs`

The older crate-wide exports remain for compatibility, but new examples and
docs should teach the focused prelude first.

## Normal Workflow

The normal path is:

```text
AuthoredUiTemplate
  -> validate_ui_template
  -> normalize_ui_template
  -> apply_ui_layout_operation or preview_ui_layout_operation
  -> validate_ui_preview_library
  -> validate_ui_persistence_flow
  -> validate_ui_readiness
  -> app-owned Editor Lab project/apply/evidence flow
```

Use `validate_ui_template` when inspecting the authored document without
changing it. Use `normalize_ui_template` when the caller needs the normalized
template tree and collected diagnostics.

Use `apply_ui_layout_operation` for activation-capable generic layout edits.
Use `preview_ui_layout_operation` when the edit is only a preview. The generic
operation layer may describe layout structure, target profile compatibility,
diffs, and diagnostics; it must not execute editor behavior.

Use `validate_ui_preview_library`, `validate_ui_persistence_flow`, and
`validate_ui_readiness` for behavior-free descriptor checks. Concrete runtime
evidence for Editor Lab scenarios lives under `apps/runenwerk_editor` and the
PM-UI-LAB closeout artifacts.

## Complete Example

Run the public example from the workspace root:

```text
cargo run -p ui_definition --example ui_definition_workflow
```

The example:

- creates an `AuthoredUiTemplate`;
- validates and normalizes it through `ui_definition::prelude`;
- applies a generic `UiVisualLayoutOperation`;
- validates preview fixture, persistence activation, and readiness descriptor
  libraries;
- avoids app-owned runtime shortcuts.

## Ownership Checks

Before adding new `ui_definition` API, verify the new API remains
behavior-free:

- no app project IO;
- no provider session state;
- no command execution;
- no live activation or rollback;
- no screenshot, GPU visual-diff, accessibility runner, performance runner, or
  artifact writer.

If a workflow needs any of those capabilities, the `ui_definition` side should
expose typed descriptors or diagnostics only, and the concrete execution should
stay in the app/runtime owner.
