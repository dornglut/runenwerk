//! Drawing app runtime plugin.

use engine::BarrierKind;
use engine::SystemSetKey;
use engine::plugins::render::UiFrameSubmissionRegistryResource;
use engine::prelude::*;
use engine::runtime::{CoreSet, IntoSystemSetKey, SystemConfigExt};

use crate::runtime::ink::{
    publish_drawing_ink_products_at_barrier, publish_drawing_ink_query_snapshots_at_barrier,
};
use crate::runtime::resources::{DrawingHostResource, DrawingInkUploadTrackerResource};
use crate::runtime::systems::{route_draw_input_system, submit_draw_frame_system};

pub struct DrawingAppPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingRuntimeSet {
    InputRoute,
    FrameSubmit,
}

impl IntoSystemSetKey for DrawingRuntimeSet {
    fn system_set_key(&self) -> SystemSetKey {
        match self {
            Self::InputRoute => {
                SystemSetKey::of::<DrawingRuntimeSet>("DrawingRuntimeSet::InputRoute")
            }
            Self::FrameSubmit => {
                SystemSetKey::of::<DrawingRuntimeSet>("DrawingRuntimeSet::FrameSubmit")
            }
        }
    }
}

impl Plugin for DrawingAppPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrawingHostResource>();
        app.init_resource::<DrawingInkUploadTrackerResource>();
        app.init_resource::<UiFrameSubmissionRegistryResource>();
        app.add_barrier_handler(
            BarrierKind::ProductPublication,
            publish_drawing_ink_products_at_barrier,
        );
        app.add_barrier_handler(
            BarrierKind::QuerySnapshotPublication,
            publish_drawing_ink_query_snapshots_at_barrier,
        );

        app.add_systems(
            Update,
            route_draw_input_system
                .in_set(DrawingRuntimeSet::InputRoute)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
        app.add_systems(
            Update,
            submit_draw_frame_system
                .in_set(DrawingRuntimeSet::FrameSubmit)
                .after(DrawingRuntimeSet::InputRoute)
                .after(CoreSet::Input)
                .after(CoreSet::Time),
        );
    }
}
