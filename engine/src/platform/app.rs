use crate::plugins::default_engine_plugins;
use crate::runtime::{Engine, EnginePlugin, SceneRegistration};
use crate::utils::setup_tracing;
use anyhow::Result;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct App {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    plugins: Vec<Box<dyn EnginePlugin>>,
    scene_registrations: Vec<SceneRegistration>,
    auto_tracing: bool,
    control_flow: ControlFlow,
    title: String,
}

impl App {
    pub fn new() -> Self {
        Self::with_default_plugins()
    }

    pub fn bare() -> Self {
        Self::with_plugins(Vec::new())
    }

    pub fn with_default_plugins() -> Self {
        Self::with_title_and_plugins("Grotto Quest - Engine", default_engine_plugins())
    }

    pub fn with_title_and_default_plugins(title: impl Into<String>) -> Self {
        Self::with_title_and_plugins(title, default_engine_plugins())
    }

    pub fn with_plugins(plugins: Vec<Box<dyn EnginePlugin>>) -> Self {
        Self::with_title_and_plugins("Grotto Quest - Engine", plugins)
    }

    pub fn with_title(title: impl Into<String>, plugins: Vec<Box<dyn EnginePlugin>>) -> Self {
        Self::with_title_and_plugins(title, plugins)
    }

    fn with_title_and_plugins(
        title: impl Into<String>,
        plugins: Vec<Box<dyn EnginePlugin>>,
    ) -> Self {
        Self {
            window: None,
            engine: None,
            plugins,
            scene_registrations: Vec::new(),
            auto_tracing: true,
            control_flow: ControlFlow::Poll,
            title: title.into(),
        }
    }

    pub fn add_plugin<P>(mut self, plugin: P) -> Self
    where
        P: EnginePlugin + 'static,
    {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn add_boxed_plugin(mut self, plugin: Box<dyn EnginePlugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn add_scene<S>(mut self, scene: S) -> Self
    where
        S: Into<SceneRegistration>,
    {
        self.scene_registrations.push(scene.into());
        self
    }

    pub fn add_scene_template(mut self, template_path: impl Into<String>) -> Self {
        let template_path = template_path.into();
        let mut id = SceneRegistration::derive_id_from_template_path(&template_path);
        if self
            .scene_registrations
            .iter()
            .any(|registered| registered.id == id)
        {
            let mut suffix = 2usize;
            let base = id.clone();
            while self
                .scene_registrations
                .iter()
                .any(|registered| registered.id == format!("{base}_{suffix}"))
            {
                suffix = suffix.saturating_add(1);
            }
            id = format!("{base}_{suffix}");
        }
        self.scene_registrations
            .push(SceneRegistration::new(id, template_path));
        self
    }

    pub fn with_auto_tracing(mut self, enabled: bool) -> Self {
        self.auto_tracing = enabled;
        self
    }

    pub fn with_control_flow(mut self, control_flow: ControlFlow) -> Self {
        self.control_flow = control_flow;
        self
    }

    pub fn run(mut self) -> Result<()> {
        let _tracing_guard = if self.auto_tracing {
            setup_tracing()
        } else {
            None
        };
        tracing::info!(title = %self.title, "starting app runtime");

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(self.control_flow);
        event_loop.run_app(&mut self).map_err(Into::into)
    }

    pub fn run_without_tracing(self) -> Result<()> {
        self.with_auto_tracing(false).run()
    }

    pub fn registered_scene_count(&self) -> usize {
        self.scene_registrations.len()
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn clear_plugins(mut self) -> Self {
        self.plugins.clear();
        self
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs: WindowAttributes = Window::default_attributes().with_title(self.title.clone());
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let mut engine = Engine::new_with_plugins_and_scenes(
            window.clone(),
            &self.plugins,
            self.scene_registrations.clone(),
        )
        .expect("failed to create engine");
        // Keep UI sizing aligned with the display scale from the first frame.
        engine.set_ui_scale_from_window_factor(window.scale_factor());
        window.request_redraw();

        self.window = Some(window);
        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };
        let Some(window) = self.window.as_ref() else {
            return;
        };

        engine.data.input.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                engine.resize(size.width, size.height);
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let logical: LogicalSize<f64> = window.inner_size().to_logical(scale_factor);
                let physical = PhysicalSize::new(
                    (logical.width * scale_factor).round() as u32,
                    (logical.height * scale_factor).round() as u32,
                );

                let _ = inner_size_writer.request_inner_size(physical);
                engine.set_ui_scale_from_window_factor(scale_factor);
                engine.resize(physical.width, physical.height);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                engine.update();
                window.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(engine) = self.engine.as_mut() {
            engine.data.input.handle_device_event(&event);
        }
    }
}
