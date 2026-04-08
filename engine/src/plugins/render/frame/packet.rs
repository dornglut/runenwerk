use super::{
    PreparedFrameContext, PreparedFrameContributions, PreparedUiFrameContribution,
    PreparedViewFrame,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct PreparedRenderFrameResource {
    frame: Option<PreparedRenderFrame>,
    next_frame_index: u64,
    next_prepare_epoch: u64,
}

impl PreparedRenderFrameResource {
    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: publish
    pub fn publish(&mut self, frame: PreparedRenderFrame) {
        self.next_frame_index = frame.context.frame_index.saturating_add(1);
        self.next_prepare_epoch = frame.context.prepare_epoch.saturating_add(1);
        self.frame = Some(frame);
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: clear
    pub fn clear(&mut self) {
        self.frame = None;
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: frame
    pub fn frame(&self) -> Option<&PreparedRenderFrame> {
        self.frame.as_ref()
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: take
    pub fn take(&mut self) -> Option<PreparedRenderFrame> {
        self.frame.take()
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: allocate_frame_index
    pub fn allocate_frame_index(&mut self) -> u64 {
        let frame_index = self.next_frame_index;
        self.next_frame_index = self.next_frame_index.saturating_add(1);
        frame_index
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: allocate_prepare_epoch
    pub fn allocate_prepare_epoch(&mut self) -> u64 {
        let prepare_epoch = self.next_prepare_epoch;
        self.next_prepare_epoch = self.next_prepare_epoch.saturating_add(1);
        prepare_epoch
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRenderFrame {
    pub context: PreparedFrameContext,
    pub surface: PreparedSurfaceInfo,
    pub views: Vec<PreparedViewFrame>,
    pub flows: BTreeMap<String, PreparedFlowInputs>,
    pub contributions: PreparedFrameContributions,
    pub shader: PreparedShaderSnapshot,
}

impl PreparedRenderFrame {
    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: flow_inputs
    pub fn flow_inputs(&self, flow_id: &str) -> Option<&PreparedFlowInputs> {
        self.flows.get(flow_id)
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: main_view
    pub fn main_view(&self) -> Option<&PreparedViewFrame> {
        self.views.first()
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: ui
    pub fn ui(&self) -> Option<&PreparedUiFrameContribution> {
        self.contributions.ui()
    }

    /// File: engine/src/plugins/render/frame/packet.rs
    /// Method: scene_route_labels
    pub fn scene_route_labels(&self) -> Option<(&str, &str)> {
        self.contributions.scene_route_labels()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedSurfaceInfo {
    pub target_size_px: (u32, u32),
}

#[derive(Debug, Clone, Default)]
pub struct PreparedFlowInputs {
    pub projected_uniform_bytes: BTreeMap<String, Vec<u8>>,
    pub projected_dispatch_workgroups: BTreeMap<String, [u32; 3]>,
    pub required_state_types: Vec<PreparedStateTypeInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStateTypeInfo {
    pub type_name: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PreparedShaderSnapshot {
    pub registry_revision: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_frame(index: u64) -> PreparedRenderFrame {
        PreparedRenderFrame {
            context: PreparedFrameContext {
                frame_index: index,
                flow_registry_revision: 7,
                shader_registry_revision: 11,
                prepare_epoch: 3,
            },
            surface: PreparedSurfaceInfo {
                target_size_px: (1280, 720),
            },
            views: vec![PreparedViewFrame::main((1280, 720))],
            flows: BTreeMap::new(),
            contributions: PreparedFrameContributions::default(),
            shader: PreparedShaderSnapshot {
                registry_revision: 11,
            },
        }
    }

    #[test]
    fn prepared_frame_resource_allocates_monotonic_indices() {
        let mut resource = PreparedRenderFrameResource::default();
        assert_eq!(resource.allocate_frame_index(), 0);
        assert_eq!(resource.allocate_frame_index(), 1);
    }

    #[test]
    fn prepared_frame_resource_publish_and_take_roundtrip() {
        let mut resource = PreparedRenderFrameResource::default();
        resource.publish(dummy_frame(4));
        assert_eq!(
            resource
                .frame()
                .expect("frame should be present after publish")
                .context
                .frame_index,
            4
        );
        assert_eq!(resource.allocate_frame_index(), 5);

        let taken = resource.take().expect("take should return a frame");
        assert_eq!(taken.context.frame_index, 4);
        assert!(resource.frame().is_none());
    }
}
