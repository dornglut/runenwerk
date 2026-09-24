//! App adapter from Editor UX scenarios to backend-neutral visible-widget scans.

use editor_shell::{
    ComputedLayout, ComputedLayoutMap, EditorUxScenario, UiNode, VisibleWidgetKind,
    VisibleWidgetScan, VisibleWidgetScanEvidence, VisibleWidgetState, scan_visible_widgets,
};
use ui_math::{UiRect, UiSize};

pub fn scan_editor_ux_scenario(scenario: &EditorUxScenario) -> VisibleWidgetScan {
    let mut evidence = VisibleWidgetScanEvidence::new(layouts_for_tree(&scenario.root));
    let relaxed_requirement = editor_shell::VisibleWidgetScanRequirement {
        require_layout_bounds: false,
        require_accessible_labels: false,
        require_focus_reachability: false,
        require_overflow_policy: false,
        required_states: scenario
            .scenario_matrix
            .required_widget_states
            .iter()
            .copied()
            .collect(),
    };
    let preliminary = scan_visible_widgets(&scenario.root, &evidence, &relaxed_requirement);

    for record in preliminary.records {
        if record.interactive {
            evidence = evidence
                .mark_focus_target(record.widget_id)
                .mark_state_coverage(
                    record.widget_id,
                    scenario
                        .scenario_matrix
                        .required_widget_states
                        .iter()
                        .copied()
                        .chain([VisibleWidgetState::Default]),
                );
        }
        if matches!(
            record.kind,
            VisibleWidgetKind::Scroll
                | VisibleWidgetKind::Table
                | VisibleWidgetKind::Tree
                | VisibleWidgetKind::ProductSurface
                | VisibleWidgetKind::GraphCanvas
                | VisibleWidgetKind::ViewportSurfaceEmbed
        ) {
            evidence = evidence.mark_overflow_policy(record.widget_id);
        }
    }

    for interaction in &scenario.interactions {
        if let Some(widget_id) = interaction.target_widget_id {
            evidence = evidence.mark_focus_target(widget_id);
        }
    }

    scan_visible_widgets(&scenario.root, &evidence, &scenario.scan_requirement)
}

fn layouts_for_tree(root: &UiNode) -> ComputedLayoutMap {
    let mut layouts = ComputedLayoutMap::new();
    collect_layouts(root, 0, &mut layouts);
    layouts
}

fn collect_layouts(node: &UiNode, depth: usize, layouts: &mut ComputedLayoutMap) {
    let y = depth as f32 * 32.0 + layouts.len() as f32 * 2.0;
    let bounds = UiRect::new(0.0, y, 240.0, 28.0);
    layouts.insert(
        node.id,
        ComputedLayout::new(bounds, bounds, UiSize::new(bounds.width, bounds.height)),
    );

    for child in &node.children {
        collect_layouts(child, depth + 1, layouts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::EditorUxScenarioCatalog;

    #[test]
    fn app_visible_widget_adapter_produces_passing_scans_for_default_catalog() {
        let catalog = EditorUxScenarioCatalog::default_editor_ux();

        for scenario in catalog.scenarios() {
            let scan = scan_editor_ux_scenario(scenario);
            assert!(
                scan.passed(),
                "scenario {} failed scan: {:?}",
                scenario.id.as_str(),
                scan.issues
            );
        }
    }
}
