---
title: Editor Definition Current Architecture
description: Current architecture for editor-owned authored definition documents and activation boundaries.
status: active
owner: editor
layer: domain
canonical: true
last_reviewed: 2026-05-08
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

The facade reexports the activation entry point from
`apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs`. That
module maps validated editor definition documents to live activation intents but
does not install runtime state itself.

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
5. Workspace layout documents form a shell workspace through
   `domain/editor/editor_shell/src/workspace/definition_form.rs::form_workspace_state_from_definition`.
6. UI templates, editor bindings, menus, shortcuts, command bindings, panel
   registries, and tool-surface registries install into app-owned active
   catalogs before the next shell frame is built.

Catalog storage and compatibility checks remain app-owned in
`apps/runenwerk_editor/src/shell/applied_editor_definition/catalogs.rs` and
`apps/runenwerk_editor/src/shell/applied_editor_definition/compatibility.rs`.
Command-binding definitions map authored route targets to existing app/domain
command ids; definition documents never execute commands directly.
