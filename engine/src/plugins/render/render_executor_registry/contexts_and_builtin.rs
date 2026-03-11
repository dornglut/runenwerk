use super::*;

pub struct RenderPassPrepareContext<'a> {
    device: &'a Device,
    queue: &'a Queue,
    frame_data: &'a RenderFrameDataRegistry<'a>,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    builtin_dispatch: Option<&'a mut dyn FnMut(BuiltinRenderPassExecutor) -> Result<()>>,
}

impl<'a> RenderPassPrepareContext<'a> {
    pub fn new(
        device: &'a Device,
        queue: &'a Queue,
        frame_data: &'a RenderFrameDataRegistry<'a>,
        surface_format: TextureFormat,
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            device,
            queue,
            frame_data,
            surface_format,
            surface_size,
            builtin_dispatch: None,
        }
    }

    pub fn with_builtin_dispatch(
        mut self,
        dispatch: &'a mut dyn FnMut(BuiltinRenderPassExecutor) -> Result<()>,
    ) -> Self {
        self.builtin_dispatch = Some(dispatch);
        self
    }

    pub fn device(&self) -> &'a Device {
        self.device
    }

    pub fn queue(&self) -> &'a Queue {
        self.queue
    }

    pub fn frame_data<T: 'static>(&self) -> Option<&'a T> {
        self.frame_data.get::<T>()
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub fn run_builtin(&mut self, builtin: BuiltinRenderPassExecutor) -> Result<()> {
        let Some(dispatch) = &mut self.builtin_dispatch else {
            bail!("builtin prepare dispatch is not available in this context");
        };
        dispatch(builtin)
    }
}

pub struct RenderPassEncodeContext<'a> {
    device: &'a Device,
    encoder: &'a mut CommandEncoder,
    frame_view: &'a TextureView,
    frame_data: &'a RenderFrameDataRegistry<'a>,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
    pipeline: PipelineKey,
    builtin_dispatch:
        Option<&'a mut dyn FnMut(&mut CommandEncoder, BuiltinRenderPassExecutor) -> Result<()>>,
    ui_dispatch: Option<&'a mut dyn FnMut(&mut CommandEncoder) -> Result<()>>,
}

impl<'a> RenderPassEncodeContext<'a> {
    pub fn new(
        device: &'a Device,
        encoder: &'a mut CommandEncoder,
        frame_view: &'a TextureView,
        frame_data: &'a RenderFrameDataRegistry<'a>,
        surface_format: TextureFormat,
        surface_size: (u32, u32),
        pipeline: PipelineKey,
    ) -> Self {
        Self {
            device,
            encoder,
            frame_view,
            frame_data,
            surface_format,
            surface_size,
            pipeline,
            builtin_dispatch: None,
            ui_dispatch: None,
        }
    }

    pub fn with_builtin_dispatch(
        mut self,
        dispatch: &'a mut dyn FnMut(&mut CommandEncoder, BuiltinRenderPassExecutor) -> Result<()>,
    ) -> Self {
        self.builtin_dispatch = Some(dispatch);
        self
    }

    pub fn with_ui_dispatch(
        mut self,
        dispatch: &'a mut dyn FnMut(&mut CommandEncoder) -> Result<()>,
    ) -> Self {
        self.ui_dispatch = Some(dispatch);
        self
    }

    pub fn device(&self) -> &'a Device {
        self.device
    }

    pub fn encoder(&mut self) -> &mut CommandEncoder {
        self.encoder
    }

    pub fn frame_view(&self) -> &'a TextureView {
        self.frame_view
    }

    pub fn frame_data<T: 'static>(&self) -> Option<&'a T> {
        self.frame_data.get::<T>()
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_format
    }

    pub fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub fn pipeline(&self) -> &PipelineKey {
        &self.pipeline
    }

    pub fn run_builtin(&mut self, builtin: BuiltinRenderPassExecutor) -> Result<()> {
        let Some(dispatch) = &mut self.builtin_dispatch else {
            bail!("builtin encode dispatch is not available in this context");
        };
        dispatch(self.encoder, builtin)
    }

    pub fn run_ui(&mut self) -> Result<()> {
        let Some(dispatch) = &mut self.ui_dispatch else {
            bail!("ui encode dispatch is not available in this context");
        };
        dispatch(self.encoder)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinRenderPassExecutor {
    Compute,
    Compose,
    MeshOverlay,
    UiComposite,
}

impl BuiltinRenderPassExecutor {
    pub fn label(self) -> &'static str {
        match self {
            Self::Compute => "builtin_compute",
            Self::Compose => "builtin_compose",
            Self::MeshOverlay => "builtin_mesh_overlay",
            Self::UiComposite => "builtin_ui_composite",
        }
    }

    pub fn from_label(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "builtin_compute" => Some(Self::Compute),
            "builtin_compose" => Some(Self::Compose),
            "builtin_mesh_overlay" => Some(Self::MeshOverlay),
            "builtin_ui_composite" => Some(Self::UiComposite),
            _ => None,
        }
    }
}

// Owner: Grotto Quest Engine - Render Domain
