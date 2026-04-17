use editor_shell::{
    ObservationConsumerKind, ObservationFrameMetadata, ObservationSourceReality,
    OutlinerObservationFrame, OutlinerObservedRow, OutlinerRowViewModel, OutlinerViewModel,
};

use crate::editor_panels::OutlinerPanelState;

pub fn build_outliner_observation_frame(
    state: &OutlinerPanelState,
    source_version: editor_core::RealityVersion,
) -> OutlinerObservationFrame {
    OutlinerObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Outliner,
            source_version,
        ),
        rows: state
            .rows
            .iter()
            .map(|row| OutlinerObservedRow {
                entity: row.entity,
                display_name: row.display_name.clone(),
                depth: row.depth,
                is_selected: state.selected_entity == Some(row.entity),
            })
            .collect(),
    }
}

pub fn build_outliner_view_model(frame: &OutlinerObservationFrame) -> OutlinerViewModel {
    OutlinerViewModel {
        rows: frame
            .rows
            .iter()
            .map(|row| OutlinerRowViewModel {
                entity: row.entity,
                display_name: row.display_name.clone(),
                depth: row.depth,
                is_selected: row.is_selected,
            })
            .collect(),
    }
}
