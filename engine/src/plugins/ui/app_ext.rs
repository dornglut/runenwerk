use crate::app::App;

use ui_controls::{ControlPackageRegistry, runenwerk_control_package};

use super::{
    UiRuntimeDiagnostic, UiRuntimeDiagnosticsResource, UiRuntimeFontConfiguration,
    UiRuntimeSlotMountFailure, UiRuntimeSlotMountReport, UiRuntimeSlotsResource, UiScreen,
};

pub trait AppUiExt {
    fn mount_ui<S>(&mut self, screen: S) -> &mut Self
    where
        S: UiScreen;

    fn ui(&mut self) -> UiAppMounting<'_>;
}

impl AppUiExt for App {
    fn mount_ui<S>(&mut self, screen: S) -> &mut Self
    where
        S: UiScreen,
    {
        let _ = mount_typed_ui(self, screen);
        self
    }

    fn ui(&mut self) -> UiAppMounting<'_> {
        UiAppMounting { app: self }
    }
}

pub struct UiAppMounting<'a> {
    app: &'a mut App,
}

impl UiAppMounting<'_> {
    pub fn mount<S>(&mut self, screen: S) -> UiRuntimeSlotMountReport
    where
        S: UiScreen,
    {
        mount_typed_ui(self.app, screen)
    }

    /// Mounts with explicit caller-provided font sources, without an Engine proof-font default.
    pub fn mount_with_fonts<S>(
        &mut self,
        screen: S,
        fonts: &UiRuntimeFontConfiguration,
    ) -> UiRuntimeSlotMountReport
    where
        S: UiScreen,
    {
        mount_typed_ui_with_fonts(self.app, screen, Some(fonts))
    }
}

fn mount_typed_ui<S>(app: &mut App, screen: S) -> UiRuntimeSlotMountReport
where
    S: UiScreen,
{
    mount_typed_ui_with_fonts(app, screen, None)
}

fn mount_typed_ui_with_fonts<S>(
    app: &mut App,
    screen: S,
    fonts: Option<&UiRuntimeFontConfiguration>,
) -> UiRuntimeSlotMountReport
where
    S: UiScreen,
{
    let source = screen.build_source();
    let screen_id = source.screen_id().clone();

    app.init_resource::<UiRuntimeDiagnosticsResource>();

    let registry = match ControlPackageRegistry::new().with_package(runenwerk_control_package()) {
        Ok(registry) => registry,
        Err(_) => {
            let report = UiRuntimeSlotMountReport::rejected(
                screen_id,
                UiRuntimeSlotMountFailure::ControlRegistry,
            );
            record_slot_mount_diagnostic(app, &report);
            return report;
        }
    };
    let snapshot = registry.snapshot();

    app.init_resource::<UiRuntimeSlotsResource>();
    let report = {
        let slots = app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("UiRuntimeSlotsResource was initialized before typed UI mounting");
        match fonts {
            Some(fonts) => slots.mount_with_fonts(source, &snapshot, fonts),
            None => slots.mount(source, &snapshot),
        }
    };
    record_slot_mount_diagnostic(app, &report);
    report
}

fn record_slot_mount_diagnostic(app: &mut App, report: &UiRuntimeSlotMountReport) {
    let Some(failure) = report.failure() else {
        return;
    };
    if let Ok(diagnostics) = app
        .world_mut()
        .resource_mut::<UiRuntimeDiagnosticsResource>()
    {
        diagnostics.push(UiRuntimeDiagnostic::runtime_slot_mount_rejected(
            report.screen_id().as_str(),
            failure,
        ));
    }
}
