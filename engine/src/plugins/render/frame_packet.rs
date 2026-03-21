use crate::plugins::ui::domain::UiDrawList;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct PreparedRenderFrameResource {
    frame: Option<PreparedRenderFrame>,
    next_frame_index: u64,
}

impl PreparedRenderFrameResource {
    pub fn publish(&mut self, frame: PreparedRenderFrame) {
        let next_frame_index = frame.frame_index.saturating_add(1);
        self.frame = Some(frame);
        self.next_frame_index = next_frame_index;
    }

    pub fn clear(&mut self) {
        self.frame = None;
    }

    pub fn frame(&self) -> Option<&PreparedRenderFrame> {
        self.frame.as_ref()
    }

    pub fn take(&mut self) -> Option<PreparedRenderFrame> {
        self.frame.take()
    }

    pub fn allocate_frame_index(&mut self) -> u64 {
        let frame_index = self.next_frame_index;
        self.next_frame_index = self.next_frame_index.saturating_add(1);
        frame_index
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRenderFrame {
    pub frame_index: u64,
    pub flow_registry_revision: u64,
    pub surface: PreparedSurfaceInfo,
    pub scene: PreparedSceneInfo,
    pub ui: PreparedUiInput,
    pub flows: BTreeMap<String, PreparedFlowInputs>,
    pub shader: PreparedShaderSnapshot,
    pub ui_rect_shader_id: Option<String>,
}

impl PreparedRenderFrame {
    pub fn flow_inputs(&self, flow_id: &str) -> Option<&PreparedFlowInputs> {
        self.flows.get(flow_id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedSurfaceInfo {
    pub target_size_px: (u32, u32),
}

#[derive(Debug, Clone)]
pub struct PreparedSceneInfo {
    pub world_scene_label: String,
    pub overlay_scene_label: String,
}

#[derive(Debug, Clone)]
pub enum PreparedUiInput {
    // Phase 1 transport format. Long-term target is extracted backend-neutral UI primitives.
    RawDrawList(UiDrawList),
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
            frame_index: index,
            flow_registry_revision: 7,
            surface: PreparedSurfaceInfo {
                target_size_px: (1280, 720),
            },
            scene: PreparedSceneInfo {
                world_scene_label: "world".to_string(),
                overlay_scene_label: "overlay".to_string(),
            },
            ui: PreparedUiInput::RawDrawList(crate::plugins::ui::domain::UiDrawList::default()),
            flows: BTreeMap::new(),
            shader: PreparedShaderSnapshot {
                registry_revision: 11,
            },
            ui_rect_shader_id: None,
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
                .frame_index,
            4
        );
        assert_eq!(resource.allocate_frame_index(), 5);

        let taken = resource.take().expect("take should return a frame");
        assert_eq!(taken.frame_index, 4);
        assert!(resource.frame().is_none());
    }
}
