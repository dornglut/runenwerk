use anyhow::{Result, anyhow};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub(crate) enum AppLifecycle {
    #[default]
    Configuring,
    Prepared,
    Starting,
    Running,
    Failed,
}

impl AppLifecycle {
    pub(crate) const fn is_configuring(self) -> bool {
        matches!(self, Self::Configuring)
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Configuring => "Configuring",
            Self::Prepared => "Prepared",
            Self::Starting => "Starting",
            Self::Running => "Running",
            Self::Failed => "Failed",
        }
    }

    pub(crate) fn prepare_for_execution(&mut self) -> Result<()> {
        match *self {
            Self::Configuring => {
                *self = Self::Prepared;
                Ok(())
            }
            Self::Prepared | Self::Running => Ok(()),
            Self::Starting => Err(anyhow!(
                "App lifecycle is Starting; the same Startup attempt cannot be prepared again"
            )),
            Self::Failed => Err(anyhow!(
                "App lifecycle is Failed; the same runtime instance cannot be advanced again"
            )),
        }
    }

    pub(crate) fn require_running(self) -> Result<()> {
        if matches!(self, Self::Running) {
            return Ok(());
        }

        Err(anyhow!(
            "App frame advancement requires Running lifecycle state; current lifecycle is {}",
            self.name()
        ))
    }
}
