//! File: domain/editor/editor_shell/src/workbench/manifest.rs
//! Purpose: Durable Workbench composition manifest contracts.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use editor_core::{
    DocumentKind, EDIT_MODE_ID, ModeId, PLAY_MODE_ID, PREVIEW_MODE_ID, SIMULATE_MODE_ID,
};
use editor_definition::{
    EditorDefinitionDocument, EditorDefinitionDocumentContent,
    EditorWorkbenchCompositionDefinition, EditorWorkbenchHostPolicyDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspaceProfileDefinition,
};

use crate::{
    CommandCapabilityKey, EditorToolSuite, HostCapabilityPolicy, ProductCapabilityKey, ProfileRef,
    ResourceCapabilityKey, SurfaceRef, ToolSuiteId, ToolSuiteIdentityError, WorkspaceProfileId,
    workspace::WorkspaceProfileLayoutSource as WorkspaceProfileLayoutSourceContract,
};

pub type ToolSuiteManifest = EditorToolSuite;
pub type WorkspaceProfileLayoutSource = WorkspaceProfileLayoutSourceContract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchCompositionManifest {
    pub composition_ref: ProfileRef,
    pub label: String,
    pub installed_suites: Vec<ToolSuiteId>,
    pub profile_refs: Vec<ProfileRef>,
    pub default_profile_ref: ProfileRef,
    pub host_policy: HostCapabilityPolicy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceProfileManifest {
    pub profile_ref: ProfileRef,
    pub compatibility_id: Option<WorkspaceProfileId>,
    pub label: String,
    pub layout_source: WorkspaceProfileLayoutSource,
    pub default_surfaces: Vec<SurfaceRef>,
    pub default_modes: Vec<ModeId>,
    pub document_kind_filters: Vec<DocumentKind>,
}

pub fn workbench_composition_manifest_from_definition(
    definition: &EditorWorkbenchCompositionDefinition,
) -> Result<WorkbenchCompositionManifest, AuthoredWorkbenchCompositionManifestError> {
    Ok(WorkbenchCompositionManifest {
        composition_ref: ProfileRef::new(definition.id.clone()).map_err(|source| {
            AuthoredWorkbenchCompositionManifestError::InvalidCompositionRef {
                composition_ref: definition.id.clone(),
                source,
            }
        })?,
        label: definition.label.clone(),
        installed_suites: definition
            .installed_suites
            .iter()
            .map(|suite_id| {
                ToolSuiteId::new(suite_id.clone()).map_err(|source| {
                    AuthoredWorkbenchCompositionManifestError::InvalidSuiteId {
                        suite_id: suite_id.clone(),
                        source,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        profile_refs: definition
            .profile_refs
            .iter()
            .map(|profile_ref| {
                ProfileRef::new(profile_ref.clone()).map_err(|source| {
                    AuthoredWorkbenchCompositionManifestError::InvalidProfileRef {
                        profile_ref: profile_ref.clone(),
                        source,
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        default_profile_ref: ProfileRef::new(definition.default_profile_ref.clone()).map_err(
            |source| AuthoredWorkbenchCompositionManifestError::InvalidProfileRef {
                profile_ref: definition.default_profile_ref.clone(),
                source,
            },
        )?,
        host_policy: host_policy_from_definition(&definition.host_policy)?,
    })
}

pub fn workspace_profile_manifests_from_authored_documents<'a>(
    profile_refs: impl IntoIterator<Item = &'a str>,
    documents: impl IntoIterator<Item = &'a EditorDefinitionDocument>,
) -> Result<Vec<WorkspaceProfileManifest>, AuthoredWorkspaceProfileManifestError> {
    let mut profiles = BTreeMap::<String, &'a EditorWorkspaceProfileDefinition>::new();
    let mut layouts = BTreeMap::<String, &'a EditorWorkspaceLayoutDefinition>::new();
    for document in documents {
        match &document.content {
            EditorDefinitionDocumentContent::WorkspaceProfile(profile) => {
                if profiles.insert(profile.id.clone(), profile).is_some() {
                    return Err(
                        AuthoredWorkspaceProfileManifestError::DuplicateProfileDocument {
                            profile_ref: profile.id.clone(),
                        },
                    );
                }
            }
            EditorDefinitionDocumentContent::WorkspaceLayout(layout) => {
                if layouts.insert(layout.id.clone(), layout).is_some() {
                    return Err(
                        AuthoredWorkspaceProfileManifestError::DuplicateLayoutDocument {
                            layout_ref: layout.id.clone(),
                        },
                    );
                }
            }
            _ => {}
        }
    }

    let mut manifests = Vec::new();
    let mut requested_profile_refs = BTreeSet::<String>::new();
    for profile_ref in profile_refs {
        if !requested_profile_refs.insert(profile_ref.to_string()) {
            return Err(
                AuthoredWorkspaceProfileManifestError::DuplicateRequestedProfileRef {
                    profile_ref: profile_ref.to_string(),
                },
            );
        }
        let profile = profiles.get(profile_ref).copied().ok_or_else(|| {
            AuthoredWorkspaceProfileManifestError::MissingProfileDocument {
                profile_ref: profile_ref.to_string(),
            }
        })?;
        let layout = layouts
            .get(&profile.default_layout)
            .copied()
            .ok_or_else(
                || AuthoredWorkspaceProfileManifestError::MissingLayoutDocument {
                    profile_ref: profile.id.clone(),
                    layout_ref: profile.default_layout.clone(),
                },
            )?;
        manifests.push(workspace_profile_manifest_from_definition(profile, layout)?);
    }
    Ok(manifests)
}

pub fn workspace_profile_manifest_from_definition(
    profile: &EditorWorkspaceProfileDefinition,
    layout: &EditorWorkspaceLayoutDefinition,
) -> Result<WorkspaceProfileManifest, AuthoredWorkspaceProfileManifestError> {
    if profile.default_layout != layout.id {
        return Err(AuthoredWorkspaceProfileManifestError::LayoutRefMismatch {
            profile_ref: profile.id.clone(),
            expected_layout_ref: profile.default_layout.clone(),
            actual_layout_ref: layout.id.clone(),
        });
    }

    let default_modes = profile
        .default_modes
        .iter()
        .map(|mode| {
            mode_id_from_authored_name(mode).ok_or_else(|| {
                AuthoredWorkspaceProfileManifestError::UnknownMode {
                    profile_ref: profile.id.clone(),
                    mode: mode.clone(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let document_kind_filters = profile
        .document_kind_filters
        .iter()
        .map(|kind| {
            document_kind_from_authored_name(kind).ok_or_else(|| {
                AuthoredWorkspaceProfileManifestError::UnknownDocumentKind {
                    profile_ref: profile.id.clone(),
                    document_kind: kind.clone(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(WorkspaceProfileManifest {
        profile_ref: ProfileRef::new(profile.id.clone()).map_err(|source| {
            AuthoredWorkspaceProfileManifestError::InvalidProfileRef {
                profile_ref: profile.id.clone(),
                source,
            }
        })?,
        compatibility_id: None,
        label: profile.label.clone(),
        layout_source: WorkspaceProfileLayoutSource::AuthoredLayout {
            layout_ref: layout.id.clone(),
            layout: layout.clone(),
        },
        default_surfaces: Vec::new(),
        default_modes,
        document_kind_filters,
    })
}

fn host_policy_from_definition(
    definition: &EditorWorkbenchHostPolicyDefinition,
) -> Result<HostCapabilityPolicy, AuthoredWorkbenchCompositionManifestError> {
    match definition {
        EditorWorkbenchHostPolicyDefinition::AllowAll => Ok(HostCapabilityPolicy::allow_all()),
        EditorWorkbenchHostPolicyDefinition::DenyAll => Ok(HostCapabilityPolicy::deny_all()),
        EditorWorkbenchHostPolicyDefinition::Explicit {
            allow_all,
            allowed_commands,
            denied_commands,
            allowed_products,
            denied_products,
            allowed_resources,
            denied_resources,
        } => {
            let mut policy = if *allow_all {
                HostCapabilityPolicy::allow_all()
            } else {
                HostCapabilityPolicy::deny_all()
            };
            for key in allowed_commands {
                policy = policy.allow_command(CommandCapabilityKey::new(key.clone()).map_err(
                    |source| AuthoredWorkbenchCompositionManifestError::InvalidCapabilityKey {
                        capability_key: key.clone(),
                        source,
                    },
                )?);
            }
            for key in denied_commands {
                policy = policy.deny_command(CommandCapabilityKey::new(key.clone()).map_err(
                    |source| AuthoredWorkbenchCompositionManifestError::InvalidCapabilityKey {
                        capability_key: key.clone(),
                        source,
                    },
                )?);
            }
            for key in allowed_products {
                policy = policy.allow_product(ProductCapabilityKey::new(key.clone()).map_err(
                    |source| AuthoredWorkbenchCompositionManifestError::InvalidCapabilityKey {
                        capability_key: key.clone(),
                        source,
                    },
                )?);
            }
            for key in denied_products {
                policy = policy.deny_product(ProductCapabilityKey::new(key.clone()).map_err(
                    |source| AuthoredWorkbenchCompositionManifestError::InvalidCapabilityKey {
                        capability_key: key.clone(),
                        source,
                    },
                )?);
            }
            for key in allowed_resources {
                policy = policy.allow_resource(ResourceCapabilityKey::new(key.clone()).map_err(
                    |source| AuthoredWorkbenchCompositionManifestError::InvalidCapabilityKey {
                        capability_key: key.clone(),
                        source,
                    },
                )?);
            }
            for key in denied_resources {
                policy = policy.deny_resource(ResourceCapabilityKey::new(key.clone()).map_err(
                    |source| AuthoredWorkbenchCompositionManifestError::InvalidCapabilityKey {
                        capability_key: key.clone(),
                        source,
                    },
                )?);
            }
            Ok(policy)
        }
    }
}

fn mode_id_from_authored_name(mode: &str) -> Option<ModeId> {
    match normalize_authored_symbol(mode).as_str() {
        "edit" | "editor_design" => Some(EDIT_MODE_ID),
        "preview" => Some(PREVIEW_MODE_ID),
        "play" => Some(PLAY_MODE_ID),
        "simulate" => Some(SIMULATE_MODE_ID),
        _ => None,
    }
}

fn document_kind_from_authored_name(kind: &str) -> Option<DocumentKind> {
    match normalize_authored_symbol(kind).as_str() {
        "scene" => Some(DocumentKind::Scene),
        "prefab" => Some(DocumentKind::Prefab),
        "sdf_graph" => Some(DocumentKind::SdfGraph),
        "sdf_brush_layer" => Some(DocumentKind::SdfBrushLayer),
        "field_world_definition" => Some(DocumentKind::FieldWorldDefinition),
        "field_product_preview" => Some(DocumentKind::FieldProductPreview),
        "material_graph" => Some(DocumentKind::MaterialGraph),
        "material" => Some(DocumentKind::Material),
        "procedural_texture" => Some(DocumentKind::ProceduralTexture),
        "volume_texture" => Some(DocumentKind::VolumeTexture),
        "procedural_generation_graph" => Some(DocumentKind::ProceduralGenerationGraph),
        "gameplay_graph" => Some(DocumentKind::GameplayGraph),
        "gameplay_rule_trigger" => Some(DocumentKind::GameplayRuleTrigger),
        "ability" => Some(DocumentKind::Ability),
        "quest" => Some(DocumentKind::Quest),
        "particle_graph" => Some(DocumentKind::ParticleGraph),
        "particle_emitter" => Some(DocumentKind::ParticleEmitter),
        "physics_scene" => Some(DocumentKind::PhysicsScene),
        "physics_config" => Some(DocumentKind::PhysicsConfig),
        "animation_clip" => Some(DocumentKind::AnimationClip),
        "animation_graph" => Some(DocumentKind::AnimationGraph),
        "timeline" => Some(DocumentKind::Timeline),
        "ui_layout" => Some(DocumentKind::UiLayout),
        "graph" => Some(DocumentKind::Graph),
        "script" => Some(DocumentKind::Script),
        "foreign_mesh_reference_import" => Some(DocumentKind::ForeignMeshReferenceImport),
        "asset_catalog" => Some(DocumentKind::AssetCatalog),
        "runtime_debug" => Some(DocumentKind::RuntimeDebug),
        "workspace_definition" => Some(DocumentKind::WorkspaceDefinition),
        "theme" => Some(DocumentKind::Theme),
        "shortcut" => Some(DocumentKind::Shortcut),
        "menu" => Some(DocumentKind::Menu),
        "command_binding" => Some(DocumentKind::CommandBinding),
        "panel_registry" => Some(DocumentKind::PanelRegistry),
        "tool_surface_definition" => Some(DocumentKind::ToolSurfaceDefinition),
        _ => None,
    }
}

fn normalize_authored_symbol(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut previous_was_separator = false;
    for (index, ch) in value.chars().enumerate() {
        if ch == '-' || ch == '.' || ch == ' ' {
            if !previous_was_separator && !normalized.is_empty() {
                normalized.push('_');
            }
            previous_was_separator = true;
            continue;
        }

        if ch.is_ascii_uppercase() {
            if index > 0 && !previous_was_separator {
                normalized.push('_');
            }
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push(ch.to_ascii_lowercase());
        }
        previous_was_separator = false;
    }
    normalized
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredWorkbenchCompositionManifestError {
    InvalidCompositionRef {
        composition_ref: String,
        source: ToolSuiteIdentityError,
    },
    InvalidSuiteId {
        suite_id: String,
        source: ToolSuiteIdentityError,
    },
    InvalidProfileRef {
        profile_ref: String,
        source: ToolSuiteIdentityError,
    },
    InvalidCapabilityKey {
        capability_key: String,
        source: ToolSuiteIdentityError,
    },
}

impl fmt::Display for AuthoredWorkbenchCompositionManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCompositionRef {
                composition_ref,
                source,
            } => write!(
                f,
                "authored Workbench composition `{composition_ref}` has invalid composition ref: {source}"
            ),
            Self::InvalidSuiteId { suite_id, source } => write!(
                f,
                "authored Workbench composition references invalid suite id `{suite_id}`: {source}"
            ),
            Self::InvalidProfileRef {
                profile_ref,
                source,
            } => write!(
                f,
                "authored Workbench composition references invalid profile ref `{profile_ref}`: {source}"
            ),
            Self::InvalidCapabilityKey {
                capability_key,
                source,
            } => write!(
                f,
                "authored Workbench composition references invalid capability key `{capability_key}`: {source}"
            ),
        }
    }
}

impl std::error::Error for AuthoredWorkbenchCompositionManifestError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredWorkspaceProfileManifestError {
    InvalidProfileRef {
        profile_ref: String,
        source: ToolSuiteIdentityError,
    },
    DuplicateRequestedProfileRef {
        profile_ref: String,
    },
    DuplicateProfileDocument {
        profile_ref: String,
    },
    DuplicateLayoutDocument {
        layout_ref: String,
    },
    MissingProfileDocument {
        profile_ref: String,
    },
    MissingLayoutDocument {
        profile_ref: String,
        layout_ref: String,
    },
    LayoutRefMismatch {
        profile_ref: String,
        expected_layout_ref: String,
        actual_layout_ref: String,
    },
    UnknownMode {
        profile_ref: String,
        mode: String,
    },
    UnknownDocumentKind {
        profile_ref: String,
        document_kind: String,
    },
}

impl fmt::Display for AuthoredWorkspaceProfileManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfileRef {
                profile_ref,
                source,
            } => write!(
                f,
                "authored profile `{profile_ref}` has invalid profile ref: {source}"
            ),
            Self::DuplicateRequestedProfileRef { profile_ref } => write!(
                f,
                "authored Workbench package requests profile `{profile_ref}` more than once"
            ),
            Self::DuplicateProfileDocument { profile_ref } => write!(
                f,
                "authored Workbench package contains duplicate profile document `{profile_ref}`"
            ),
            Self::DuplicateLayoutDocument { layout_ref } => write!(
                f,
                "authored Workbench package contains duplicate layout document `{layout_ref}`"
            ),
            Self::MissingProfileDocument { profile_ref } => write!(
                f,
                "authored Workbench package is missing profile document `{profile_ref}`"
            ),
            Self::MissingLayoutDocument {
                profile_ref,
                layout_ref,
            } => write!(
                f,
                "authored profile `{profile_ref}` references missing layout document `{layout_ref}`"
            ),
            Self::LayoutRefMismatch {
                profile_ref,
                expected_layout_ref,
                actual_layout_ref,
            } => write!(
                f,
                "authored profile `{profile_ref}` references layout `{expected_layout_ref}`, but layout document is `{actual_layout_ref}`"
            ),
            Self::UnknownMode { profile_ref, mode } => {
                write!(
                    f,
                    "authored profile `{profile_ref}` references unknown mode `{mode}`"
                )
            }
            Self::UnknownDocumentKind {
                profile_ref,
                document_kind,
            } => write!(
                f,
                "authored profile `{profile_ref}` references unknown document kind `{document_kind}`"
            ),
        }
    }
}

impl std::error::Error for AuthoredWorkspaceProfileManifestError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandCapabilityKey;
    use editor_definition::{
        EditorDefinitionDocument, EditorDefinitionDocumentContent, EditorDefinitionDocumentKind,
        EditorDefinitionId, EditorWorkbenchCompositionDefinition,
        EditorWorkbenchHostPolicyDefinition, EditorWorkspaceHostDefinition,
        EditorWorkspacePanelTabDefinition,
    };

    #[test]
    fn authored_profile_definition_forms_manifest_identity_and_filters() {
        let profile = EditorWorkspaceProfileDefinition {
            id: "runenwerk.editor.workspace.editor_design".to_string(),
            label: "Editor Design".to_string(),
            default_modes: vec!["editor-design".to_string(), "preview".to_string()],
            document_kind_filters: vec!["UiLayout".to_string(), "WorkspaceDefinition".to_string()],
            default_layout: "runenwerk.editor.layout.editor_design".to_string(),
        };
        let layout = EditorWorkspaceLayoutDefinition {
            id: "runenwerk.editor.layout.editor_design".to_string(),
            label: "Editor Design Layout".to_string(),
            root: EditorWorkspaceHostDefinition::TabStack {
                id: "root".to_string(),
                tabs: vec![EditorWorkspacePanelTabDefinition {
                    id: "canvas".to_string(),
                    label: "Canvas".to_string(),
                    tool_surface: "runenwerk.editor_design.ui_canvas".to_string(),
                }],
                active_tab: Some("canvas".to_string()),
            },
            floating_hosts: Vec::new(),
        };

        let manifest = workspace_profile_manifest_from_definition(&profile, &layout)
            .expect("authored profile should form a manifest");

        assert_eq!(
            manifest.profile_ref.as_str(),
            "runenwerk.editor.workspace.editor_design"
        );
        assert_eq!(manifest.default_modes, vec![EDIT_MODE_ID, PREVIEW_MODE_ID]);
        assert_eq!(
            manifest.document_kind_filters,
            vec![DocumentKind::UiLayout, DocumentKind::WorkspaceDefinition]
        );
    }

    #[test]
    fn authored_workbench_composition_definition_forms_manifest_policy() {
        let manifest =
            workbench_composition_manifest_from_definition(&EditorWorkbenchCompositionDefinition {
                id: "runenwerk.editor.workbench.custom".to_string(),
                label: "Custom Workbench".to_string(),
                installed_suites: vec![
                    "runenwerk.editor".to_string(),
                    "runenwerk.editor_design".to_string(),
                ],
                profile_refs: vec!["runenwerk.editor.workspace.custom".to_string()],
                default_profile_ref: "runenwerk.editor.workspace.custom".to_string(),
                host_policy: EditorWorkbenchHostPolicyDefinition::Explicit {
                    allow_all: false,
                    allowed_commands: vec!["runenwerk.editor.open".to_string()],
                    denied_commands: vec!["runenwerk.editor.close".to_string()],
                    allowed_products: Vec::new(),
                    denied_products: Vec::new(),
                    allowed_resources: Vec::new(),
                    denied_resources: Vec::new(),
                },
            })
            .expect("authored workbench composition should form a manifest");

        assert_eq!(
            manifest.composition_ref.as_str(),
            "runenwerk.editor.workbench.custom"
        );
        assert_eq!(manifest.installed_suites.len(), 2);
        assert_eq!(
            manifest.default_profile_ref.as_str(),
            "runenwerk.editor.workspace.custom"
        );
        assert!(
            manifest
                .host_policy
                .allows_command(&CommandCapabilityKey::new("runenwerk.editor.open").unwrap())
        );
        assert!(
            !manifest
                .host_policy
                .allows_command(&CommandCapabilityKey::new("runenwerk.editor.close").unwrap())
        );
    }

    #[test]
    fn authored_workspace_package_rejects_missing_layout_documents() {
        let profile = EditorWorkspaceProfileDefinition {
            id: "runenwerk.editor.workspace.custom".to_string(),
            label: "Custom Workspace".to_string(),
            default_modes: Vec::new(),
            document_kind_filters: Vec::new(),
            default_layout: "runenwerk.editor.layout.missing".to_string(),
        };
        let profile_document = EditorDefinitionDocument::current(
            EditorDefinitionId::from(profile.id.as_str()),
            profile.label.clone(),
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceProfile(profile),
        );

        let error = workspace_profile_manifests_from_authored_documents(
            ["runenwerk.editor.workspace.custom"],
            [&profile_document],
        )
        .expect_err("missing authored layout document should reject");

        assert!(matches!(
            error,
            AuthoredWorkspaceProfileManifestError::MissingLayoutDocument { .. }
        ));
    }

    #[test]
    fn authored_workspace_package_rejects_duplicate_profile_documents() {
        let profile = EditorWorkspaceProfileDefinition {
            id: "runenwerk.editor.workspace.custom".to_string(),
            label: "Custom Workspace".to_string(),
            default_modes: Vec::new(),
            document_kind_filters: Vec::new(),
            default_layout: "runenwerk.editor.layout.custom".to_string(),
        };
        let profile_document = EditorDefinitionDocument::current(
            EditorDefinitionId::from(profile.id.as_str()),
            profile.label.clone(),
            EditorDefinitionDocumentKind::WorkspaceDefinition,
            EditorDefinitionDocumentContent::WorkspaceProfile(profile),
        );

        let error = workspace_profile_manifests_from_authored_documents(
            ["runenwerk.editor.workspace.custom"],
            [&profile_document, &profile_document],
        )
        .expect_err("duplicate authored profile document should reject");

        assert!(matches!(
            error,
            AuthoredWorkspaceProfileManifestError::DuplicateProfileDocument { .. }
        ));
    }
}
