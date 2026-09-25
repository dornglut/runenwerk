use crate::app::domain::lifecycle::AppLifecycle;
use crate::app::domain::mode::AppMode;
use crate::app::domain::runner::{AppRunner, FixedFramesRunner};
use crate::app::domain::state::WindowedAppState;
use crate::prelude::IntoPlugins;
use crate::runtime::system::IntoSystemConfigs;
use crate::*;
use anyhow::Result;
use runen_ecs::{Resource, Runtime, RuntimeError, ScheduleLabel, World};
use std::error::Error;
use std::fmt;

const DEFAULT_WINDOW_TITLE: &str = "Runenwerk - Engine";

pub struct App {
    pub(crate) world: World,
    pub(crate) scheduler: Runtime,
    pub(crate) runner: Box<dyn AppRunner>,
    pub(crate) lifecycle: AppLifecycle,
    pub(crate) mode: AppMode,
    pub(crate) title: String,
    composition_errors: Vec<AppCompositionError>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_mode(AppMode::Windowed)
    }

    pub fn headless() -> Self {
        Self::with_mode(AppMode::Headless)
    }

    fn with_mode(mode: AppMode) -> Self {
        let title = DEFAULT_WINDOW_TITLE.to_string();
        let mut app = Self {
            world: World::new(),
            scheduler: Runtime::new(),
            runner: Box::new(FixedFramesRunner::new(1)),
            lifecycle: AppLifecycle::default(),
            mode,
            title: title.clone(),
            composition_errors: Vec::new(),
        };
        app
    }

    pub fn add_plugin<P>(&mut self, plugin: P) -> &mut Self
    where
        P: Plugin + 'static,
    {
        if !self.allow_topology_mutation("add_plugin", None) {
            return self;
        }
        plugin.build(self);
        self
    }

    pub fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        if !self.allow_topology_mutation("add_boxed_plugin", None) {
            return self;
        }
        plugin.build(self);
        self
    }

    pub fn add_plugins<P>(&mut self, plugins: P) -> &mut Self
    where
        P: IntoPlugins,
    {
        if !self.allow_topology_mutation("add_plugins", None) {
            return self;
        }
        plugins.add_to_app(self);
        self
    }

    pub fn add_systems<L, S, Marker>(&mut self, _schedule: L, systems: S) -> &mut Self
    where
        L: ScheduleLabel,
        S: IntoSystemConfigs<Marker>,
    {
        if !self.allow_topology_mutation("add_systems", Some(L::name())) {
            return self;
        }
        if let Err(source) = self
            .scheduler
            .add_systems::<L, S, Marker>(_schedule, systems)
        {
            self.composition_errors
                .push(AppCompositionError::SystemRegistration {
                    schedule: L::name(),
                    source,
                });
        }
        self
    }

    pub fn init_resource<R>(&mut self) -> &mut Self
    where
        R: Resource + Default,
    {
        if self.world.resource::<R>().is_err() {
            self.world.insert_resource(R::default());
        }
        self
    }

    pub fn insert_resource<R>(&mut self, value: R) -> &mut Self
    where
        R: Resource,
    {
        self.world.insert_resource(value);
        self
    }

    pub fn set_runner<R>(&mut self, runner: R) -> &mut Self
    where
        R: AppRunner + 'static,
    {
        self.runner = Box::new(runner);
        self
    }

    pub fn set_title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = title.into();
        self
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub(crate) fn into_windowed_state(mut self) -> WindowedAppState {
        if self.lifecycle.is_configuring() {
            self.admit_composition()
                .expect("windowed Host-state transfer requires admitted App composition");
            self.lifecycle
                .prepare_for_execution()
                .expect("Configuring App lifecycle should prepare for windowed Host transfer");
        }
        self.prepare_windowed_frame_pacing();
        WindowedAppState {
            world: self.world,
            scheduler: self.scheduler,
            startup_ran: self.lifecycle,
            title: self.title,
        }
    }

    pub(crate) fn admit_composition(&mut self) -> Result<()> {
        if !self.composition_errors.is_empty() {
            let errors = std::mem::take(&mut self.composition_errors);
            return Err(anyhow::Error::new(
                AppCompositionAdmissionError::Registration(errors),
            ));
        }

        self.scheduler
            .validate()
            .map_err(|source| anyhow::Error::new(AppCompositionAdmissionError::Topology(source)))
    }

    pub(crate) fn prepare_lifecycle_for_execution(&mut self) -> Result<()> {
        self.lifecycle.prepare_for_execution()
    }

    pub(crate) fn record_missing_capability(
        &mut self,
        operation: &'static str,
        capability: &'static str,
    ) {
        self.composition_errors
            .push(AppCompositionError::MissingCapability {
                operation,
                capability,
            });
    }

    pub(crate) fn allow_topology_mutation(
        &mut self,
        operation: &'static str,
        target: Option<&'static str>,
    ) -> bool {
        if self.lifecycle.is_configuring() {
            return true;
        }

        self.composition_errors
            .push(AppCompositionError::LateTopologyMutation {
                operation,
                target,
                lifecycle: self.lifecycle,
            });
        false
    }
}

#[derive(Debug)]
enum AppCompositionError {
    MissingCapability {
        operation: &'static str,
        capability: &'static str,
    },
    SystemRegistration {
        schedule: &'static str,
        source: RuntimeError,
    },
    LateTopologyMutation {
        operation: &'static str,
        target: Option<&'static str>,
        lifecycle: AppLifecycle,
    },
}

impl fmt::Display for AppCompositionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCapability {
                operation,
                capability,
            } => write!(
                formatter,
                "{operation} requires selected capability '{capability}'"
            ),
            Self::SystemRegistration { schedule, source } => write!(
                formatter,
                "failed to register systems in schedule '{}': {}",
                schedule, source
            ),
            Self::LateTopologyMutation {
                operation,
                target,
                lifecycle,
            } => {
                write!(
                    formatter,
                    "{operation} rejected after App composition was sealed (lifecycle: {})",
                    lifecycle.name()
                )?;
                if let Some(target) = target {
                    write!(formatter, "; target: {target}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for AppCompositionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SystemRegistration { source, .. } => Some(source),
            Self::MissingCapability { .. } | Self::LateTopologyMutation { .. } => None,
        }
    }
}

#[derive(Debug)]
enum AppCompositionAdmissionError {
    Registration(Vec<AppCompositionError>),
    Topology(RuntimeError),
}

impl fmt::Display for AppCompositionAdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Registration(errors) => {
                write!(formatter, "App composition admission rejected")?;
                for (index, error) in errors.iter().enumerate() {
                    write!(formatter, "; registration error {}: {error}", index + 1)?;
                }
                Ok(())
            }
            Self::Topology(source) => {
                write!(formatter, "App topology admission rejected: {source}")
            }
        }
    }
}

impl Error for AppCompositionAdmissionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registration(_) => None,
            Self::Topology(source) => Some(source),
        }
    }
}
