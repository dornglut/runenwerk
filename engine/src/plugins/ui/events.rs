use ui_program::UiEventPacket;
use ui_surface::SurfaceInstanceId;

#[derive(Clone, Debug, PartialEq)]
pub struct UiActionEvent {
    packet: UiEventPacket,
    surface_instance_id: Option<SurfaceInstanceId>,
}

impl UiActionEvent {
    pub fn new(packet: UiEventPacket) -> Self {
        Self {
            packet,
            surface_instance_id: None,
        }
    }

    pub fn with_surface_instance_id(mut self, surface_instance_id: SurfaceInstanceId) -> Self {
        self.surface_instance_id = Some(surface_instance_id);
        self
    }

    pub fn packet(&self) -> &UiEventPacket {
        &self.packet
    }

    pub fn surface_instance_id(&self) -> Option<SurfaceInstanceId> {
        self.surface_instance_id
    }
}
