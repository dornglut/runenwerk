use editor_shell::{OutlinerRowViewModel, OutlinerViewModel};

use crate::editor_panels::OutlinerPanelState;

pub fn build_outliner_view_model(state: &OutlinerPanelState) -> OutlinerViewModel {
    OutlinerViewModel {
        rows: state
            .rows
            .iter()
            .map(|row| OutlinerRowViewModel {
                entity: row.entity,
                display_name: row.display_name.clone(),
                depth: row.depth,
                is_selected: state.selected_entity == Some(row.entity),
            })
            .collect(),
    }
}
