//! File: domain/editor/editor_shell/src/story_lab/catalog.rs
//! Purpose: Editor UX Story Lab catalog registration and validation.

use std::collections::BTreeMap;

use crate::{
    EditorLabSurfaceViewModel, EditorUxInputModality, EditorUxScenarioMatrix, EditorUxStory,
    EditorUxStoryId, EditorUxStoryInteraction, EditorUxStoryInteractionKind, EditorUxStoryKind,
    EditorUxViewportClass, MATERIAL_GRAPH_CANVAS_STORY_ID,
    MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID, MATERIAL_GRAPH_CANVAS_WIDGET_ID,
    SHELL_PATTERNS_DOCK_SPLIT_WIDGET_ID, SHELL_PATTERNS_INSPECTOR_TEXT_WIDGET_ID,
    SHELL_PATTERNS_PALETTE_ITEM_WIDGET_ID, SHELL_PATTERNS_PREVIEW_WIDGET_ID,
    SHELL_PATTERNS_TABLE_WIDGET_ID, SHELL_PATTERNS_TREE_WIDGET_ID, SHELL_PRODUCT_PATTERNS_STORY_ID,
    ToolSurfaceInstanceId, ToolSurfaceReadiness, UI_DESIGNER_WORKBENCH_STORY_ID,
    VisibleWidgetScanRequirement, VisibleWidgetState, WidgetId, build_editor_lab_surface,
    build_material_graph_surface, button, editor_surface_definitions, label,
    material_graph_canvas_evidence, material_graph_canvas_fixture_view_model,
    material_graph_canvas_required_states, panel, primary_button_design_system_evidence,
    registered_surface_evidence, shell_product_pattern_evidence,
    shell_product_pattern_fixture_root, shell_product_pattern_required_states, surface_widget_id,
    tool_surface_readiness_for_definition_id, ui_designer_workbench_evidence,
    ui_designer_workbench_fixture_view_model, ui_designer_workbench_required_states,
};
use ui_surface::SurfaceDefinition;
use ui_text::TextStyle;
use ui_theme::ThemeTokens;
use ui_widgets::{PrimitiveWidgetStoryKind, primitive_widget_stories};

#[derive(Debug, Clone, PartialEq)]
pub struct EditorUxStoryCatalog {
    stories_by_id: BTreeMap<EditorUxStoryId, EditorUxStory>,
}

impl EditorUxStoryCatalog {
    pub fn new(
        stories: impl IntoIterator<Item = EditorUxStory>,
    ) -> Result<Self, EditorUxStoryCatalogError> {
        let mut stories_by_id = BTreeMap::new();
        for story in stories {
            if story.id.as_str().trim().is_empty() {
                return Err(EditorUxStoryCatalogError::EmptyStoryId);
            }
            if story.label.trim().is_empty() {
                return Err(EditorUxStoryCatalogError::EmptyStoryLabel { story_id: story.id });
            }
            if story.scenario_matrix.is_empty() {
                return Err(EditorUxStoryCatalogError::EmptyScenarioMatrix { story_id: story.id });
            }
            if stories_by_id.insert(story.id.clone(), story).is_some() {
                return Err(EditorUxStoryCatalogError::DuplicateStoryId);
            }
        }
        Ok(Self { stories_by_id })
    }

    pub fn default_editor_ux() -> Self {
        Self::new(default_editor_ux_stories())
            .expect("default editor UX story catalog should be valid")
    }

    pub fn get(&self, story_id: &EditorUxStoryId) -> Option<&EditorUxStory> {
        self.stories_by_id.get(story_id)
    }

    pub fn stories(&self) -> impl Iterator<Item = &EditorUxStory> {
        self.stories_by_id.values()
    }

    pub fn len(&self) -> usize {
        self.stories_by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stories_by_id.is_empty()
    }

    pub fn validate(&self) -> Result<(), EditorUxStoryCatalogError> {
        if self.is_empty() {
            return Err(EditorUxStoryCatalogError::EmptyCatalog);
        }
        for story in self.stories() {
            if story.interactions.is_empty() {
                return Err(EditorUxStoryCatalogError::MissingInteractions {
                    story_id: story.id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorUxStoryCatalogError {
    EmptyCatalog,
    EmptyStoryId,
    EmptyStoryLabel { story_id: EditorUxStoryId },
    EmptyScenarioMatrix { story_id: EditorUxStoryId },
    DuplicateStoryId,
    MissingInteractions { story_id: EditorUxStoryId },
}

pub fn default_editor_ux_stories() -> Vec<EditorUxStory> {
    let mut stories = primitive_widget_stories()
        .into_iter()
        .map(|primitive| {
            let root_id = primitive.root.id;
            let mut story = EditorUxStory::new(
                EditorUxStoryId::new(primitive.id),
                primitive.label,
                EditorUxStoryKind::PrimitiveWidget(primitive.kind),
                ToolSurfaceReadiness::Product,
                EditorUxScenarioMatrix::baseline(
                    primitive.scan_requirement.required_states.iter().copied(),
                ),
                primitive.root,
                primitive.scan_requirement,
            )
            .with_interactions([
                EditorUxStoryInteraction::new(
                    "focus",
                    "Focus",
                    EditorUxStoryInteractionKind::FocusTraversal,
                    Some(root_id),
                ),
                EditorUxStoryInteraction::new(
                    "activate",
                    "Activate",
                    EditorUxStoryInteractionKind::PointerActivate,
                    Some(root_id),
                ),
            ]);
            if primitive.kind == PrimitiveWidgetStoryKind::Button {
                story = story.with_design_system_evidence(primary_button_design_system_evidence());
            }
            story
        })
        .collect::<Vec<_>>();

    stories.extend(editor_surface_definitions().into_iter().map(surface_story));
    stories.push(ui_designer_workbench_story());
    stories.push(material_graph_canvas_story());
    stories.push(shell_product_patterns_story());
    stories.push(host_scenario_story());
    stories
}

fn shell_product_patterns_story() -> EditorUxStory {
    let mut scenario_matrix =
        EditorUxScenarioMatrix::baseline(shell_product_pattern_required_states());
    scenario_matrix.viewport_classes = vec![
        EditorUxViewportClass::Narrow,
        EditorUxViewportClass::Standard,
        EditorUxViewportClass::Wide,
    ];
    scenario_matrix
        .densities
        .push(crate::EditorUxDensity::Compact);
    scenario_matrix.input_modalities = vec![
        EditorUxInputModality::Pointer,
        EditorUxInputModality::Keyboard,
    ];
    scenario_matrix.expected_diagnostics = vec![
        "diagnostics warning route available",
        "degraded preview evidence available",
        "long text overflow policy available",
    ];

    EditorUxStory::new(
        EditorUxStoryId::new(SHELL_PRODUCT_PATTERNS_STORY_ID),
        "Shell Product Patterns",
        EditorUxStoryKind::ProductPattern,
        ToolSurfaceReadiness::Product,
        scenario_matrix,
        shell_product_pattern_fixture_root(),
        VisibleWidgetScanRequirement::strict_interactive(shell_product_pattern_required_states()),
    )
    .with_interactions([
        EditorUxStoryInteraction::new(
            "focus-inspector",
            "Focus Inspector",
            EditorUxStoryInteractionKind::FocusTraversal,
            Some(SHELL_PATTERNS_INSPECTOR_TEXT_WIDGET_ID),
        ),
        EditorUxStoryInteraction::new(
            "select-palette-item",
            "Select Palette Item",
            EditorUxStoryInteractionKind::PointerActivate,
            Some(SHELL_PATTERNS_PALETTE_ITEM_WIDGET_ID),
        ),
        EditorUxStoryInteraction::new(
            "select-table-row",
            "Select Table Row",
            EditorUxStoryInteractionKind::PointerActivate,
            Some(SHELL_PATTERNS_TABLE_WIDGET_ID),
        ),
        EditorUxStoryInteraction::new(
            "select-tree-row",
            "Select Tree Row",
            EditorUxStoryInteractionKind::KeyboardActivate,
            Some(SHELL_PATTERNS_TREE_WIDGET_ID),
        ),
        EditorUxStoryInteraction::new(
            "focus-preview",
            "Focus Preview",
            EditorUxStoryInteractionKind::ScenarioCapture,
            Some(SHELL_PATTERNS_PREVIEW_WIDGET_ID),
        ),
        EditorUxStoryInteraction::new(
            "capture-dock-split",
            "Capture Dock Split",
            EditorUxStoryInteractionKind::ScenarioCapture,
            Some(SHELL_PATTERNS_DOCK_SPLIT_WIDGET_ID),
        ),
    ])
    .with_product_pattern_evidence(shell_product_pattern_evidence())
}

fn material_graph_canvas_story() -> EditorUxStory {
    let surface_id = ToolSurfaceInstanceId::try_from_raw(45)
        .expect("material graph canvas fixture surface id should be non-zero");
    let view_model = material_graph_canvas_fixture_view_model();
    let (root, routes) = build_material_graph_surface(
        &ThemeTokens::default(),
        surface_id,
        &view_model,
        vec![
            "degraded provider: none".to_string(),
            "dense graph overflow policy: enabled".to_string(),
        ],
        Vec::new(),
    );
    let mut scenario_matrix =
        EditorUxScenarioMatrix::baseline(material_graph_canvas_required_states());
    scenario_matrix.viewport_classes = vec![
        EditorUxViewportClass::Narrow,
        EditorUxViewportClass::Standard,
        EditorUxViewportClass::Wide,
    ];
    scenario_matrix
        .densities
        .push(crate::EditorUxDensity::Compact);
    scenario_matrix.input_modalities = vec![
        EditorUxInputModality::Pointer,
        EditorUxInputModality::Keyboard,
    ];
    scenario_matrix.expected_diagnostics = vec![
        "preview build required",
        "unfinished graph families hidden until productized",
    ];

    EditorUxStory::new(
        EditorUxStoryId::new(MATERIAL_GRAPH_CANVAS_STORY_ID),
        "Material Graph Canvas Product",
        EditorUxStoryKind::RegisteredSurface(MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID),
        ToolSurfaceReadiness::Product,
        scenario_matrix,
        root,
        VisibleWidgetScanRequirement::strict_interactive(material_graph_canvas_required_states()),
    )
    .with_interactions([
        EditorUxStoryInteraction::new(
            "select-node",
            "Select Node",
            EditorUxStoryInteractionKind::PointerActivate,
            Some(surface_widget_id(
                surface_id,
                MATERIAL_GRAPH_CANVAS_WIDGET_ID,
            )),
        ),
        EditorUxStoryInteraction::new(
            "edit-node-value",
            "Edit Node Value",
            EditorUxStoryInteractionKind::TextEntry,
            Some(surface_widget_id(surface_id, WidgetId(43_010))),
        ),
        EditorUxStoryInteraction::new(
            "navigate-diagnostic",
            "Navigate Diagnostic",
            EditorUxStoryInteractionKind::PointerActivate,
            Some(surface_widget_id(surface_id, WidgetId(45_010))),
        ),
        EditorUxStoryInteraction::new(
            "capture-graph",
            "Capture Graph",
            EditorUxStoryInteractionKind::ScenarioCapture,
            routes.iter().next().map(|(widget_id, _)| *widget_id),
        ),
    ])
    .with_graph_canvas_evidence(material_graph_canvas_evidence())
    .with_registered_surface_evidence(registered_surface_evidence(
        surface_definition_by_id(MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID),
        tool_surface_readiness_for_definition_id(MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID),
    ))
}

fn ui_designer_workbench_story() -> EditorUxStory {
    let surface_id = ToolSurfaceInstanceId::try_from_raw(44)
        .expect("UI Designer workbench fixture surface id should be non-zero");
    let view_model = ui_designer_workbench_fixture_view_model();
    let (root, routes) = build_editor_lab_surface(
        &ThemeTokens::default(),
        surface_id,
        &EditorLabSurfaceViewModel::UiDesignerWorkbench(view_model),
    );
    let mut scenario_matrix =
        EditorUxScenarioMatrix::baseline(ui_designer_workbench_required_states());
    scenario_matrix.viewport_classes = vec![
        EditorUxViewportClass::Narrow,
        EditorUxViewportClass::Standard,
        EditorUxViewportClass::Wide,
    ];
    scenario_matrix
        .densities
        .push(crate::EditorUxDensity::Compact);
    scenario_matrix.input_modalities = vec![
        EditorUxInputModality::Pointer,
        EditorUxInputModality::Keyboard,
    ];

    EditorUxStory::new(
        EditorUxStoryId::new(UI_DESIGNER_WORKBENCH_STORY_ID),
        "Standalone UI Designer Workbench",
        EditorUxStoryKind::HostScenario,
        ToolSurfaceReadiness::Product,
        scenario_matrix,
        root,
        VisibleWidgetScanRequirement::strict_interactive(ui_designer_workbench_required_states()),
    )
    .with_interactions([
        EditorUxStoryInteraction::new(
            "select-canvas-node",
            "Select Canvas Node",
            EditorUxStoryInteractionKind::PointerActivate,
            Some(surface_widget_id(surface_id, WidgetId(70_006))),
        ),
        EditorUxStoryInteraction::new(
            "edit-inspector-property",
            "Edit Inspector Property",
            EditorUxStoryInteractionKind::TextEntry,
            Some(surface_widget_id(surface_id, WidgetId(70_011))),
        ),
        EditorUxStoryInteraction::new(
            "capture-workbench",
            "Capture Workbench",
            EditorUxStoryInteractionKind::ScenarioCapture,
            routes.iter().next().map(|(widget_id, _)| *widget_id),
        ),
    ])
    .with_workbench_evidence(ui_designer_workbench_evidence())
}

fn surface_story(definition: SurfaceDefinition) -> EditorUxStory {
    let readiness = tool_surface_readiness_for_definition_id(definition.id);
    let root_id = WidgetId(20_000 + definition.id.raw() * 10);
    let action_id = WidgetId(root_id.0 + 1);
    let style = TextStyle::default();
    let theme = ThemeTokens::default();
    let root = if readiness.visible_in_product() {
        panel(
            root_id,
            theme.clone(),
            vec![button(
                action_id,
                definition.display_name,
                style.clone(),
                theme,
            )],
        )
    } else {
        panel(
            root_id,
            theme,
            vec![label(
                WidgetId(action_id.0),
                format!("{} hidden until productized", definition.display_name),
                style,
            )],
        )
    };
    let mut requirement = VisibleWidgetScanRequirement::strict_interactive([
        VisibleWidgetState::Default,
        VisibleWidgetState::Focused,
    ]);
    if !readiness.visible_in_product() {
        requirement.require_accessible_labels = false;
        requirement.require_focus_reachability = false;
        requirement.required_states.clear();
    }

    EditorUxStory::new(
        EditorUxStoryId::new(format!("editor.surface.{}", definition.semantic_key)),
        definition.display_name,
        EditorUxStoryKind::RegisteredSurface(definition.id),
        readiness,
        EditorUxScenarioMatrix::baseline(requirement.required_states.iter().copied()),
        root,
        requirement,
    )
    .with_interactions([EditorUxStoryInteraction::new(
        "capture",
        "Capture Surface",
        EditorUxStoryInteractionKind::ScenarioCapture,
        readiness.visible_in_product().then_some(action_id),
    )])
    .with_registered_surface_evidence(registered_surface_evidence(definition, readiness))
}

fn surface_definition_by_id(definition_id: ui_surface::SurfaceDefinitionId) -> SurfaceDefinition {
    editor_surface_definitions()
        .into_iter()
        .find(|definition| definition.id == definition_id)
        .expect("registered surface definition should exist")
}

fn host_scenario_story() -> EditorUxStory {
    let style = TextStyle::default();
    let theme = ThemeTokens::default();
    let root_id = WidgetId(30_000);
    let action_id = WidgetId(30_001);
    EditorUxStory::new(
        EditorUxStoryId::new("editor.host.workspace.default"),
        "Editor Workspace Host / Default",
        EditorUxStoryKind::HostScenario,
        ToolSurfaceReadiness::Product,
        EditorUxScenarioMatrix::baseline([
            VisibleWidgetState::Default,
            VisibleWidgetState::Focused,
        ]),
        panel(
            root_id,
            theme.clone(),
            vec![button(action_id, "Open Workspace", style, theme)],
        ),
        VisibleWidgetScanRequirement::strict_interactive([
            VisibleWidgetState::Default,
            VisibleWidgetState::Focused,
        ]),
    )
    .with_interactions([
        EditorUxStoryInteraction::new(
            "focus-workspace",
            "Focus Workspace",
            EditorUxStoryInteractionKind::FocusTraversal,
            Some(action_id),
        ),
        EditorUxStoryInteraction::new(
            "capture-host",
            "Capture Host",
            EditorUxStoryInteractionKind::ScenarioCapture,
            Some(root_id),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_covers_primitives_surfaces_and_host_scenarios() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();

        catalog.validate().expect("default catalog should validate");
        assert!(
            catalog
                .stories()
                .any(|story| matches!(story.kind, EditorUxStoryKind::PrimitiveWidget(_)))
        );
        assert!(
            catalog
                .stories()
                .any(|story| matches!(story.kind, EditorUxStoryKind::RegisteredSurface(_)))
        );
        assert!(
            catalog
                .stories()
                .any(|story| story.kind == EditorUxStoryKind::HostScenario)
        );
    }

    #[test]
    fn registered_surfaces_have_readiness_classification() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();

        assert!(catalog.stories().any(|story| {
            matches!(story.kind, EditorUxStoryKind::RegisteredSurface(_))
                && story.readiness == ToolSurfaceReadiness::Product
        }));
        assert!(catalog.stories().any(|story| {
            matches!(story.kind, EditorUxStoryKind::RegisteredSurface(_))
                && story.readiness == ToolSurfaceReadiness::HiddenUntilProductized
        }));
    }

    #[test]
    fn registered_surface_stories_name_pm007_evidence_contract() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();
        let surface_stories = catalog
            .stories()
            .filter(|story| matches!(story.kind, EditorUxStoryKind::RegisteredSurface(_)))
            .collect::<Vec<_>>();

        let covered_definition_ids = surface_stories
            .iter()
            .filter_map(|story| {
                story
                    .registered_surface_evidence
                    .as_ref()
                    .map(|evidence| evidence.surface_definition_id)
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            covered_definition_ids.len(),
            editor_surface_definitions().len()
        );
        assert!(surface_stories.iter().all(|story| {
            story
                .registered_surface_evidence
                .as_ref()
                .is_some_and(|evidence| evidence.semantic_key.starts_with("editor.tool_surface."))
        }));
        assert!(surface_stories.iter().any(|story| {
            story.readiness == ToolSurfaceReadiness::HiddenUntilProductized
                && story
                    .registered_surface_evidence
                    .as_ref()
                    .is_some_and(|evidence| !evidence.visible_in_product)
        }));
        assert!(surface_stories.iter().any(|story| {
            story.readiness == ToolSurfaceReadiness::Product
                && story
                    .registered_surface_evidence
                    .as_ref()
                    .is_some_and(|evidence| {
                        evidence
                            .required_artifact_kinds
                            .contains(&"PlatformImpossibleReport")
                    })
        }));
    }

    #[test]
    fn migrated_button_story_names_design_system_evidence() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();
        let story = catalog
            .stories()
            .find(|story| {
                matches!(
                    story.kind,
                    EditorUxStoryKind::PrimitiveWidget(PrimitiveWidgetStoryKind::Button)
                )
            })
            .expect("button story should be registered");
        let evidence = story
            .design_system_evidence
            .as_ref()
            .expect("button story should carry design-system evidence");

        assert_eq!(evidence.recipe_id.as_str(), "editor.pattern.primary_button");
        assert!(
            evidence
                .token_ids
                .iter()
                .any(|id| id.as_str() == "color.accent")
        );
        assert!(
            story
                .scenario_matrix
                .design_system_state_variants
                .iter()
                .any(|id| id.as_str() == "accessibility.high-contrast")
        );
    }

    #[test]
    fn standalone_ui_designer_workbench_story_names_native_evidence_contract() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();
        let story = catalog
            .get(&EditorUxStoryId::new(UI_DESIGNER_WORKBENCH_STORY_ID))
            .expect("standalone UI Designer workbench story should be registered");
        let evidence = story
            .workbench_evidence
            .as_ref()
            .expect("workbench story should require native workbench evidence");

        assert_eq!(story.readiness, ToolSurfaceReadiness::Product);
        assert!(evidence.pane_kinds.contains(&"canvas"));
        assert!(evidence.route_kinds.contains(&"set_ui_node_text"));
        assert!(evidence.legacy_self_authoring_bypass);
        assert!(
            story
                .scenario_matrix
                .required_widget_states
                .contains(&VisibleWidgetState::Overflow)
        );
    }

    #[test]
    fn material_graph_canvas_story_names_product_evidence_contract() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();
        let story = catalog
            .get(&EditorUxStoryId::new(MATERIAL_GRAPH_CANVAS_STORY_ID))
            .expect("material graph canvas story should be registered");
        let evidence = story
            .graph_canvas_evidence
            .as_ref()
            .expect("material graph story should require graph canvas evidence");

        assert_eq!(story.readiness, ToolSurfaceReadiness::Product);
        assert!(evidence.interaction_kinds.contains(&"drag_node_commit"));
        assert!(
            evidence
                .route_kinds
                .contains(&"provider_owned_graph_canvas")
        );
        assert!(
            evidence
                .readiness_decisions
                .contains(&"sdf_graph_canvas=hidden_until_productized")
        );
        assert!(
            story
                .scenario_matrix
                .required_widget_states
                .contains(&VisibleWidgetState::Selected)
        );
    }

    #[test]
    fn shell_product_patterns_story_names_pm006_evidence_contract() {
        let catalog = EditorUxStoryCatalog::default_editor_ux();
        let story = catalog
            .get(&EditorUxStoryId::new(SHELL_PRODUCT_PATTERNS_STORY_ID))
            .expect("shell product pattern story should be registered");
        let evidence = story
            .product_pattern_evidence
            .as_ref()
            .expect("shell pattern story should require product-pattern evidence");

        assert_eq!(story.readiness, ToolSurfaceReadiness::Product);
        assert!(evidence.pattern_kinds.contains(&"inspector"));
        assert!(evidence.pattern_kinds.contains(&"dock"));
        assert!(evidence.state_kinds.contains(&"degraded"));
        assert!(
            evidence
                .native_evidence_checks
                .contains(&"product_pattern_report")
        );
        assert!(
            story
                .scenario_matrix
                .required_widget_states
                .contains(&VisibleWidgetState::Overflow)
        );
    }
}
