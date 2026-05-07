---
title: Editor Definition Current Architecture
description: Current architecture for editor-owned authored definition documents and activation boundaries.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-06
---

# Editor Definition Current Architecture

`domain/editor/editor_definition` owns editor-specific authored definition
schemas and validation. These documents describe editor/UI layouts, workspace
profiles and layouts, themes, shortcuts, menus, command bindings, panel
registries, tool-surface registries, and editor UI binding metadata.

## Domain Ownership

The domain crate owns durable schemas, validation, and pure formation helpers.
It may depend on generic `domain/ui` definition and theme contracts, but it does
not own the runnable editor app, runtime resources, windowing, provider
registries, file IO, or live activation policy.

Current schema entry points:

- `domain/editor/editor_definition/src/document.rs::EditorDefinitionDocument`
- `domain/editor/editor_definition/src/document.rs::EditorDefinitionDocumentContent`
- `domain/editor/editor_definition/src/theme.rs::EditorThemeDefinition`
- `domain/editor/editor_definition/src/validate.rs::validate_editor_definition_document`

Theme formation is the first live activation-ready formation path:

- `domain/editor/editor_definition/src/theme.rs::form_theme_tokens`

It starts from a supplied `ui_theme::ThemeTokens` base, applies known authored
color, spacing, radius, and typography tokens, and rejects malformed or unknown
tokens with `ui_definition::UiDefinitionDiagnostic` diagnostics. The function is
pure domain logic; it does not mutate app or runtime state.

## App Activation Boundary

Live activation belongs to `apps/runenwerk_editor`, not this domain crate. The
app-level seam is:

- `apps/runenwerk_editor/src/shell/applied_editor_definition.rs::activate_editor_definition_document`

For theme documents, that seam calls `form_theme_tokens` and returns a live theme
activation. For other editor definition document kinds, it currently returns no
live activation rather than pretending that snapshot apply changes runtime
behavior.

## Current Live Behavior

After an Editor Design apply command succeeds:

1. `apps/runenwerk_editor/src/shell/self_authoring.rs::SelfAuthoringWorkspaceState::apply_selected`
   stores an applied snapshot.
2. `apps/runenwerk_editor/src/shell/dispatch_shell_command.rs::dispatch_shell_command`
   queues the applied document for app-owned activation.
3. `apps/runenwerk_editor/src/runtime/resources.rs::EditorHostResource::apply_pending_editor_definition_activations`
   drains the queue at the runtime host boundary.
4. Theme documents form `ThemeTokens` and replace the live host theme through
   `EditorHostResource::apply_theme`.

UI templates, workspace layouts, menus, shortcuts, command bindings, panel
registries, and tool-surface registries remain draft/preview/snapshot-capable
only until their own explicit activation paths are implemented.
