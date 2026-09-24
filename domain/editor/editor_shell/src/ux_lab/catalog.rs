//! File: domain/editor/editor_shell/src/ux_lab/catalog.rs
//! Purpose: Editor UX Lab catalog registration and validation.

use std::collections::BTreeMap;

use crate::{
    EditorLabSurfaceViewModel, EditorUxInputModality, EditorUxScenario, EditorUxScenarioId,
    EditorUxScenarioInteraction, EditorUxScenarioInteractionKind, EditorUxScenarioKind,
    EditorUxScenarioMatrix, EditorUxViewportClass, MATERIAL_GRAPH_CANVAS_SCENARIO_ID,
    MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID, MATERIAL_GRAPH_CANVAS_WIDGET_ID,
    SHELL_PATTERNS_DOCK_SPLIT_WIDGET_ID, SHELL_PATTERNS_INSPECTOR_TEXT_WIDGET_ID,
    SHELL_PATTERNS_PALETTE_ITEM_WIDGET_ID, SHELL_PATTERNS_PREVIEW_WIDGET_ID,
    SHELL_PATTERNS_TABLE_WIDGET_ID, SHELL_PATTERNS_TREE_WIDGET_ID,
    SHELL_PRODUCT_PATTERNS_SCENARIO_ID, ToolSurfaceInstanceId, ToolSurfaceReadiness,
    UI_DESIGNER_WORKBENCH_SCENARIO_ID, VisibleWidgetScanRequirement, VisibleWidgetState, WidgetId,
    build_editor_lab_surface, build_material_graph_surface, button, editor_surface_definitions,
    label, material_graph_canvas_evidence, material_graph_canvas_fixture_view_model,
    material_graph_canvas_required_states, panel, primary_button_design_system_evidence,
    registered_surface_evidence, shell_product_pattern_evidence,
    shell_product_pattern_fixture_root, shell_product_pattern_required_states, surface_widget_id,
    tool_surface_readiness_for_definition_id, ui_designer_workbench_evidence,
    ui_designer_workbench_fixture_view_model, ui_designer_workbench_required_states,
};
use ui_surface::SurfaceDefinition;
use ui_text::TextStyle;
use ui_theme::ThemeTokens;
use ui_widgets::{PrimitiveWidgetScenarioKind, primitive_widget_scenarios};

#[derive(Debug, Clone, PartialEq)]
pub struct EditorUxScenarioCatalog {
    scenarios_by_id: BTreeMap<EditorUxScenarioId, EditorUxScenario>,
}

impl EditorUxScenarioCatalog {
    pub fn new(
        scenarios: impl IntoIterator<Item = EditorUxScenario>,
    ) -> Result<Self, EditorUxScenarioCatalogError> {
        let mut scenarios_by_id = BTreeMap::new();
        for scenario in scenarios {
            if scenario.id.as_str().trim().is_empty() {
                return Err(EditorUxScenarioCatalogError::EmptyScenarioId);
            }
            if scenario.label.trim().is_empty() {
                return Err(EditorUxScenarioCatalogError::EmptyScenarioLabel {
                    scenario_id: scenario.id,
                });
            }
            if scenario.scenario_matrix.is_empty() {
                return Err(EditorUxScenarioCatalogError::EmptyScenarioMatrix {
                    scenario_id: scenario.id,
                });
            }
            if scenarios_by_id
                .insert(scenario.id.clone(), scenario)
                .is_some()
            {
                return Err(EditorUxScenarioCatalogError::DuplicateScenarioId);
            }
        }
        Ok(Self { scenarios_by_id })
    }

    pub fn default_editor_ux() -> Self {
        Self::new(default_editor_ux_scenarios())
            .expect("default editor UX scenario catalog should be valid")
    }

    pub fn get(&self, scenario_id: &EditorUxScenarioId) -> Option<&EditorUxScenario> {
        self.scenarios_by_id.get(scenario_id)
    }

    pub fn scenarios(&self) -> impl Iterator<Item = &EditorUxScenario> {
        self.scenarios_by_id.values()
    }

    pub fn len(&self) -> usize {
        self.scenarios_by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scenarios_by_id.is_empty()
    }

    pub fn validate(&self) -> Result<(), EditorUxScenarioCatalogError> {
        if self.is_empty() {
            return Err(EditorUxScenarioCatalogError::EmptyCatalog);
        }
        for scenario in self.scenarios() {
            if scenario.interactions.is_empty() {
                return Err(EditorUxScenarioCatalogError::MissingInteractions {
                    scenario_id: scenario.id.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorUxScenarioCatalogError {
    EmptyCatalog,
    EmptyScenarioId,
    EmptyScenarioLabel { scenario_id: EditorUxScenarioId },
    EmptyScenarioMatrix { scenario_id: EditorUxScenarioId },
    DuplicateScenarioId,
    MissingInteractions { scenario_id: EditorUxScenarioId },
}

pub fn default_editor_ux_scenarios() -> Vec<EditorUxScenario> {
    let mut scenarios = primitive_widget_scenarios()
        .into_iter()
        .map(|primitive| {
            let root_id = primitive.root.id;
            let mut scenario = EditorUxScenario::new(
                EditorUxScenarioId::new(primitive.id),
                primitive.label,
                EditorUxScenarioKind::PrimitiveWidget(primitive.kind),
                ToolSurfaceReadiness::Product,
                EditorUxScenarioMatrix::baseline(
                    primitive.scan_requirement.required_states.iter().copied(),
                ),
                primitive.root,
                primitive.scan_requirement,
            )
            .with_interactions([
                EditorUxScenarioInteraction::new(
                    "focus",
                    "Focus",
                    EditorUxScenarioInteractionKind::FocusTraversal,
                    Some(root_id),
                ),
                EditorUxScenarioInteraction::new(
                    "activate",
                    "Activate",
                    EditorUxScenarioInteractionKind::PointerActivate,
                    Some(root_id),
                ),
            ]);
            if primitive.kind == PrimitiveWidgetScenarioKind::Button {
                scenario =
                    scenario.with_design_system_evidence(primary_button_design_system_evidence());
            }
            scenario
        })
        .collect::<Vec<_>>();

    scenarios.extend(
        editor_surface_definitions()
            .into_iter()
            .map(surface_scenario),
    );
    scenarios.push(ui_designer_workbench_scenario());
    scenarios.push(material_graph_canvas_scenario());
    scenarios.push(shell_product_patterns_scenario());
    scenarios.push(host_scenario_scenario());
    scenarios
}

fn shell_product_patterns_scenario() -> EditorUxScenario {
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

    EditorUxScenario::new(
        EditorUxScenarioId::new(SHELL_PRODUCT_PATTERNS_SCENARIO_ID),
        "Shell Product Patterns",
        EditorUxScenarioKind::ProductPattern,
        ToolSurfaceReadiness::Product,
        scenario_matrix,
        shell_product_pattern_fixture_root(),
        VisibleWidgetScanRequirement::strict_interactive(shell_product_pattern_required_states()),
    )
    .with_interactions([
        EditorUxScenarioInteraction::new(
            "focus-inspector",
            "Focus Inspector",
            EditorUxScenarioInteractionKind::FocusTraversal,
            Some(SHELL_PATTERNS_INSPECTOR_TEXT_WIDGET_ID),
        ),
        EditorUxScenarioInteraction::new(
            "select-palette-item",
            "Select Palette Item",
            EditorUxScenarioInteractionKind::PointerActivate,
            Some(SHELL_PATTERNS_PALETTE_ITEM_WIDGET_ID),
        ),
        EditorUxScenarioInteraction::new(
            "select-table-row",
            "Select Table Row",
            EditorUxScenarioInteractionKind::PointerActivate,
            Some(SHELL_PATTERNS_TABLE_WIDGET_ID),
        ),
        EditorUxScenarioInteraction::new(
            "select-tree-row",
            "Select Tree Row",
            EditorUxScenarioInteractionKind::KeyboardActivate,
            Some(SHELL_PATTERNS_TREE_WIDGET_ID),
        ),
        EditorUxScenarioInteraction::new(
            "focus-preview",
            "Focus Preview",
            EditorUxScenarioInteractionKind::ScenarioCapture,
            Some(SHELL_PATTERNS_PREVIEW_WIDGET_ID),
        ),
        EditorUxScenarioInteraction::new(
            "capture-dock-split",
            "Capture Dock Split",
            EditorUxScenarioInteractionKind::ScenarioCapture,
            Some(SHELL_PATTERNS_DOCK_SPLIT_WIDGET_ID),
        ),
    ])
    .with_product_pattern_evidence(shell_product_pattern_evidence())
}

fn material_graph_canvas_scenario() -> EditorUxScenario {
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

    EditorUxScenario::new(
        EditorUxScenarioId::new(MATERIAL_GRAPH_CANVAS_SCENARIO_ID),
        "Material Graph Canvas Product",
        EditorUxScenarioKind::RegisteredSurface(MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID),
        ToolSurfaceReadiness::Product,
        scenario_matrix,
        root,
        VisibleWidgetScanRequirement::strict_interactive(material_graph_canvas_required_states()),
    )
    .with_interactions([
        EditorUxScenarioInteraction::new(
            "select-node",
            "Select Node",
            EditorUxScenarioInteractionKind::PointerActivate,
            Some(surface_widget_id(
                surface_id,
                MATERIAL_GRAPH_CANVAS_WIDGET_ID,
            )),
        ),
        EditorUxScenarioInteraction::new(
            "edit-node-value",
            "Edit Node Value",
            EditorUxScenarioInteractionKind::TextEntry,
            Some(surface_widget_id(surface_id, WidgetId(43_010))),
        ),
        EditorUxScenarioInteraction::new(
            "navigate-diagnostic",
            "Navigate Diagnostic",
            EditorUxScenarioInteractionKind::PointerActivate,
            Some(surface_widget_id(surface_id, WidgetId(45_010))),
        ),
        EditorUxScenarioInteraction::new(
            "capture-graph",
            "Capture Graph",
            EditorUxScenarioInteractionKind::ScenarioCapture,
            routes.iter().next().map(|(widget_id, _)| *widget_id),
        ),
    ])
    .with_graph_canvas_evidence(material_graph_canvas_evidence())
    .with_registered_surface_evidence(registered_surface_evidence(
        surface_definition_by_id(MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID),
        tool_surface_readiness_for_definition_id(MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID),
    ))
}

fn ui_designer_workbench_scenario() -> EditorUxScenario {
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

    EditorUxScenario::new(
        EditorUxScenarioId::new(UI_DESIGNER_WORKBENCH_SCENARIO_ID),
        "Standalone UI Designer Workbench",
        EditorUxScenarioKind::HostScenario,
        ToolSurfaceReadiness::Product,
        scenario_matrix,
        root,
        VisibleWidgetScanRequirement::strict_interactive(ui_designer_workbench_required_states()),
    )
    .with_interactions([
        EditorUxScenarioInteraction::new(
            "select-canvas-node",
            "Select Canvas Node",
            EditorUxScenarioInteractionKind::PointerActivate,
            Some(surface_widget_id(surface_id, WidgetId(70_006))),
        ),
        EditorUxScenarioInteraction::new(
            "edit-inspector-property",
            "Edit Inspector Property",
            EditorUxScenarioInteractionKind::TextEntry,
            Some(surface_widget_id(surface_id, WidgetId(70_011))),
        ),
        EditorUxScenarioInteraction::new(
            "capture-workbench",
            "Capture Workbench",
            EditorUxScenarioInteractionKind::ScenarioCapture,
            routes.iter().next().map(|(widget_id, _)| *widget_id),
        ),
    ])
    .with_workbench_evidence(ui_designer_workbench_evidence())
}

fn surface_scenario(definition: SurfaceDefinition) -> EditorUxScenario {
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

    EditorUxScenario::new(
        EditorUxScenarioId::new(format!("editor.surface.{}", definition.semantic_key)),
        definition.display_name,
        EditorUxScenarioKind::RegisteredSurface(definition.id),
        readiness,
        EditorUxScenarioMatrix::baseline(requirement.required_states.iter().copied()),
        root,
        requirement,
    )
    .with_interactions([EditorUxScenarioInteraction::new(
        "capture",
        "Capture Surface",
        EditorUxScenarioInteractionKind::ScenarioCapture,
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

fn host_scenario_scenario() -> EditorUxScenario {
    let style = TextStyle::default();
    let theme = ThemeTokens::default();
    let root_id = WidgetId(30_000);
    let action_id = WidgetId(30_001);
    EditorUxScenario::new(
        EditorUxScenarioId::new("editor.host.workspace.default"),
        "Editor Workspace Host / Default",
        EditorUxScenarioKind::HostScenario,
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
        EditorUxScenarioInteraction::new(
            "focus-workspace",
            "Focus Workspace",
            EditorUxScenarioInteractionKind::FocusTraversal,
            Some(action_id),
        ),
        EditorUxScenarioInteraction::new(
            "capture-host",
            "Capture Host",
            EditorUxScenarioInteractionKind::ScenarioCapture,
            Some(root_id),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_covers_primitives_surfaces_and_host_scenarios() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();

        catalog.validate().expect("default catalog should validate");
        assert!(
            catalog
                .scenarios()
                .any(|scenario| matches!(scenario.kind, EditorUxScenarioKind::PrimitiveWidget(_)))
        );
        assert!(
            catalog.scenarios().any(|scenario| matches!(
                scenario.kind,
                EditorUxScenarioKind::RegisteredSurface(_)
            ))
        );
        assert!(
            catalog
                .scenarios()
                .any(|scenario| scenario.kind == EditorUxScenarioKind::HostScenario)
        );
    }

    #[test]
    fn registered_surfaces_have_readiness_classification() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();

        assert!(catalog.scenarios().any(|scenario| {
            matches!(scenario.kind, EditorUxScenarioKind::RegisteredSurface(_))
                && scenario.readiness == ToolSurfaceReadiness::Product
        }));
        assert!(catalog.scenarios().any(|scenario| {
            matches!(scenario.kind, EditorUxScenarioKind::RegisteredSurface(_))
                && scenario.readiness == ToolSurfaceReadiness::HiddenUntilProductized
        }));
    }

    #[test]
    fn registered_surface_scenarios_name_pm007_evidence_contract() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();
        let surface_scenarios = catalog
            .scenarios()
            .filter(|scenario| matches!(scenario.kind, EditorUxScenarioKind::RegisteredSurface(_)))
            .collect::<Vec<_>>();

        let covered_definition_ids = surface_scenarios
            .iter()
            .filter_map(|scenario| {
                scenario
                    .registered_surface_evidence
                    .as_ref()
                    .map(|evidence| evidence.surface_definition_id)
            })
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(
            covered_definition_ids.len(),
            editor_surface_definitions().len()
        );
        assert!(surface_scenarios.iter().all(|scenario| {
            scenario
                .registered_surface_evidence
                .as_ref()
                .is_some_and(|evidence| evidence.semantic_key.starts_with("editor.tool_surface."))
        }));
        assert!(surface_scenarios.iter().any(|scenario| {
            scenario.readiness == ToolSurfaceReadiness::HiddenUntilProductized
                && scenario
                    .registered_surface_evidence
                    .as_ref()
                    .is_some_and(|evidence| !evidence.visible_in_product)
        }));
        assert!(surface_scenarios.iter().any(|scenario| {
            scenario.readiness == ToolSurfaceReadiness::Product
                && scenario
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
    fn migrated_button_scenario_names_design_system_evidence() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();
        let scenario = catalog
            .scenarios()
            .find(|scenario| {
                matches!(
                    scenario.kind,
                    EditorUxScenarioKind::PrimitiveWidget(PrimitiveWidgetScenarioKind::Button)
                )
            })
            .expect("button scenario should be registered");
        let evidence = scenario
            .design_system_evidence
            .as_ref()
            .expect("button scenario should carry design-system evidence");

        assert_eq!(evidence.recipe_id.as_str(), "editor.pattern.primary_button");
        assert!(
            evidence
                .token_ids
                .iter()
                .any(|id| id.as_str() == "color.accent")
        );
        assert!(
            scenario
                .scenario_matrix
                .design_system_state_variants
                .iter()
                .any(|id| id.as_str() == "accessibility.high-contrast")
        );
    }

    #[test]
    fn standalone_ui_designer_workbench_scenario_names_native_evidence_contract() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();
        let scenario = catalog
            .get(&EditorUxScenarioId::new(UI_DESIGNER_WORKBENCH_SCENARIO_ID))
            .expect("standalone UI Designer workbench scenario should be registered");
        let evidence = scenario
            .workbench_evidence
            .as_ref()
            .expect("workbench scenario should require native workbench evidence");

        assert_eq!(scenario.readiness, ToolSurfaceReadiness::Product);
        assert!(evidence.pane_kinds.contains(&"canvas"));
        assert!(evidence.route_kinds.contains(&"set_ui_node_text"));
        assert!(evidence.legacy_self_authoring_bypass);
        assert!(
            scenario
                .scenario_matrix
                .required_widget_states
                .contains(&VisibleWidgetState::Overflow)
        );
    }

    #[test]
    fn material_graph_canvas_scenario_names_product_evidence_contract() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();
        let scenario = catalog
            .get(&EditorUxScenarioId::new(MATERIAL_GRAPH_CANVAS_SCENARIO_ID))
            .expect("material graph canvas scenario should be registered");
        let evidence = scenario
            .graph_canvas_evidence
            .as_ref()
            .expect("material graph scenario should require graph canvas evidence");

        assert_eq!(scenario.readiness, ToolSurfaceReadiness::Product);
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
            scenario
                .scenario_matrix
                .required_widget_states
                .contains(&VisibleWidgetState::Selected)
        );
    }

    #[test]
    fn shell_product_patterns_scenario_names_pm006_evidence_contract() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();
        let scenario = catalog
            .get(&EditorUxScenarioId::new(SHELL_PRODUCT_PATTERNS_SCENARIO_ID))
            .expect("shell product pattern scenario should be registered");
        let evidence = scenario
            .product_pattern_evidence
            .as_ref()
            .expect("shell pattern scenario should require product-pattern evidence");

        assert_eq!(scenario.readiness, ToolSurfaceReadiness::Product);
        assert!(evidence.pattern_kinds.contains(&"inspector"));
        assert!(evidence.pattern_kinds.contains(&"dock"));
        assert!(evidence.state_kinds.contains(&"degraded"));
        assert!(
            evidence
                .native_evidence_checks
                .contains(&"product_pattern_report")
        );
        assert!(
            scenario
                .scenario_matrix
                .required_widget_states
                .contains(&VisibleWidgetState::Overflow)
        );
    }
}
