//! File: apps/runenwerk_editor/src/shell/applied_editor_definition/activation.rs
//! Purpose: Map applied editor definition documents to live activation intents.

use editor_definition::{
    EditorCommandBindingSetDefinition, EditorDefinitionBindings, EditorDefinitionDocument,
    EditorDefinitionDocumentContent, EditorMenuDefinition, EditorPanelRegistryDefinition,
    EditorShortcutSetDefinition, EditorToolSurfaceRegistryDefinition,
    EditorWorkspaceLayoutDefinition, editor_definition_has_blocking_diagnostics, form_theme_tokens,
    validate_editor_definition_document,
};
use ui_definition::{NormalizedUiTemplate, UiDefinitionDiagnostic, normalize_authored_template};
use ui_theme::ThemeTokens;

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDefinitionActivation {
    ThemeChanged(ThemeTokens),
    UiTemplateCatalogChanged {
        template_id: String,
        template: NormalizedUiTemplate,
    },
    EditorBindingsCatalogChanged(EditorDefinitionBindings),
    MenuCatalogChanged {
        menu_id: String,
        menu: EditorMenuDefinition,
    },
    ShortcutCatalogChanged {
        shortcut_set_id: String,
        shortcuts: EditorShortcutSetDefinition,
    },
    CommandBindingCatalogChanged {
        command_binding_set_id: String,
        command_bindings: EditorCommandBindingSetDefinition,
    },
    PanelRegistryCatalogChanged {
        registry_id: String,
        registry: EditorPanelRegistryDefinition,
    },
    ToolSurfaceRegistryCatalogChanged {
        registry_id: String,
        registry: EditorToolSurfaceRegistryDefinition,
    },
    WorkspaceLayoutChanged {
        workspace_id: String,
        layout: EditorWorkspaceLayoutDefinition,
    },
    NoLiveActivation,
}

pub fn activate_editor_definition_document(
    document: &EditorDefinitionDocument,
    base_theme: &ThemeTokens,
) -> Result<EditorDefinitionActivation, Vec<UiDefinitionDiagnostic>> {
    match &document.content {
        EditorDefinitionDocumentContent::Theme(theme) => form_theme_tokens(theme, base_theme)
            .map(EditorDefinitionActivation::ThemeChanged)
            .map_err(|error| error.diagnostics),
        EditorDefinitionDocumentContent::WorkspaceLayout(layout) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::WorkspaceLayoutChanged {
                    workspace_id: layout.id.clone(),
                    layout: layout.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::UiTemplate(template) => {
            let normalized = normalize_authored_template(template.clone());
            if editor_definition_has_blocking_diagnostics(&normalized.diagnostics) {
                Err(normalized.diagnostics)
            } else {
                Ok(EditorDefinitionActivation::UiTemplateCatalogChanged {
                    template_id: normalized.id.as_str().to_string(),
                    template: normalized,
                })
            }
        }
        EditorDefinitionDocumentContent::EditorBindings(bindings) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::EditorBindingsCatalogChanged(
                    bindings.clone(),
                ))
            }
        }
        EditorDefinitionDocumentContent::Menu(menu) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::MenuCatalogChanged {
                    menu_id: menu.id.clone(),
                    menu: menu.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::Shortcuts(shortcuts) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::ShortcutCatalogChanged {
                    shortcut_set_id: shortcuts.id.clone(),
                    shortcuts: shortcuts.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::CommandBindings(command_bindings) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::CommandBindingCatalogChanged {
                    command_binding_set_id: command_bindings.id.clone(),
                    command_bindings: command_bindings.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::PanelRegistry(registry) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(EditorDefinitionActivation::PanelRegistryCatalogChanged {
                    registry_id: registry.id.clone(),
                    registry: registry.clone(),
                })
            }
        }
        EditorDefinitionDocumentContent::ToolSurfaceRegistry(registry) => {
            let diagnostics = validate_editor_definition_document(document);
            if editor_definition_has_blocking_diagnostics(&diagnostics) {
                Err(diagnostics)
            } else {
                Ok(
                    EditorDefinitionActivation::ToolSurfaceRegistryCatalogChanged {
                        registry_id: registry.id.clone(),
                        registry: registry.clone(),
                    },
                )
            }
        }
        _ => Ok(EditorDefinitionActivation::NoLiveActivation),
    }
}
