use std::collections::BTreeSet;

use anyhow::{Result, anyhow, ensure};
use runenui_core::StyleEnvironment;

use crate::app::App;
use crate::plugins::render::host::RenderSurfaceId;
use crate::plugins::render::{
    ManagedRenderProducerIds, RenderFrameProducerId, RunenUiPaintSubmissionRegistryResource,
    SurfaceFrameSubmissionRegistryResource, render_integration_is_active,
};

use ui_controls::{ControlPackageRegistry, runenwerk_control_package};

use super::{
    UiPluginStateResource, UiRuntimeDiagnostic, UiRuntimeDiagnosticsResource,
    UiRuntimeFontConfiguration, UiRuntimePresentationAssociationsResource,
    UiRuntimePresentationBinding, UiRuntimeSlotId, UiRuntimeSlotMountFailure,
    UiRuntimeSlotMountReport, UiRuntimeSlotsResource, UiScreen,
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

/// A native-primary convenience mount preserves the original RunenUI owner.
/// On placement failure, the typed runtime remains mounted and identifiable.
#[derive(Debug, thiserror::Error)]
pub enum UiAppPrimaryMountError {
    #[error("native-primary typed UI admission rejected: {0}")]
    Preflight(#[source] anyhow::Error),
    #[error("typed RunenUI mount rejected: {report:?}")]
    MountRejected { report: UiRuntimeSlotMountReport },
    #[error("native-primary placement failed for retained slot {slot_id:?}: {source}")]
    PlacementRetained {
        slot_id: UiRuntimeSlotId,
        #[source]
        source: anyhow::Error,
    },
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

    /// Mounts an ordinary concrete RunenUI application in the same Engine slot
    /// and renderer publication path as the existing Counter UiProgram consumer.
    pub fn mount_app<A>(
        &mut self,
        screen_id: super::UiTypedScreenId,
        state: A::State,
        fonts: &UiRuntimeFontConfiguration,
    ) -> UiRuntimeSlotMountReport
    where
        A: runenui_core::UiApp + 'static,
    {
        self.app.init_resource::<UiRuntimeDiagnosticsResource>();
        self.app.init_resource::<UiRuntimeSlotsResource>();
        let clock = self.app.native_ui_clock();
        let slots = self
            .app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("UiRuntimeSlotsResource initialized before typed app mounting");
        if let Err(failure) = slots.configure_host_clock(clock) {
            let report = UiRuntimeSlotMountReport::rejected(screen_id, failure);
            record_slot_mount_diagnostic(self.app, &report);
            return report;
        }
        let report = self
            .app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("UiRuntimeSlotsResource initialized before typed app mounting")
            .mount_app::<A>(screen_id, state, fonts);
        record_slot_mount_diagnostic(self.app, &report);
        report
    }

    /// Mounts one typed UiApp and requests managed placement on the native
    /// primary Render surface. This does not install plugins or a RenderFlow.
    ///
    /// RunenUI mounting remains independent from physical Present completion.
    /// On placement failure the slot remains mounted and its typed error
    /// exposes the slot ID, so the caller retains all pending Host work and
    /// can retry placement or retire the slot with its full shutdown batch.
    ///
    /// # Errors
    /// Rejects missing native Host or selected owner plugins/resources before
    /// mounting; propagates RunenUI mount failure reports; or retains the
    /// mounted but unplaced slot on managed Render placement failure.
    pub fn mount_app_on_primary<A>(
        &mut self,
        screen_id: super::UiTypedScreenId,
        state: A::State,
        fonts: &UiRuntimeFontConfiguration,
    ) -> std::result::Result<UiRuntimeSlotId, UiAppPrimaryMountError>
    where
        A: runenui_core::UiApp + 'static,
    {
        let admission = (|| -> Result<()> {
            ensure!(
                self.app.is_native_window_host() && self.app.native_ui_clock().is_some(),
                "native-primary typed UI placement requires a native-window Host"
            );
            ensure!(
                self.app
                    .world()
                    .resource::<UiPluginStateResource>()
                    .is_ok_and(UiPluginStateResource::is_installed),
                "native-primary typed UI placement requires UiPlugin"
            );
            ensure!(
                render_integration_is_active(self.app.world()),
                "native-primary typed UI placement requires RenderPlugin"
            );
            self.app.world().resource::<UiRuntimeSlotsResource>()?;
            self.app
                .world()
                .resource::<UiRuntimePresentationAssociationsResource>()?;
            self.app.world().resource::<ManagedRenderProducerIds>()?;
            self.app
                .world()
                .resource::<SurfaceFrameSubmissionRegistryResource>()?;
            self.app
                .world()
                .resource::<RunenUiPaintSubmissionRegistryResource>()?;
            Ok(())
        })();
        admission.map_err(UiAppPrimaryMountError::Preflight)?;

        let report = self.mount_app::<A>(screen_id, state, fonts);
        let Some(slot_id) = report.slot_id() else {
            return Err(UiAppPrimaryMountError::MountRejected { report });
        };
        self.present(slot_id, RenderSurfaceId::primary())
            .map_err(|source| UiAppPrimaryMountError::PlacementRetained { slot_id, source })?;
        Ok(slot_id)
    }

    /// Places an already mounted RunenUI slot on a selected Render surface.
    ///
    /// This is a placement request, not a claim that the surface exists or
    /// that a GPU submission has completed Present. The native Host derives
    /// actual attachment, dimensions and scale during normal RenderPrepare.
    /// The existing typed, target-neutral mount API remains valid headlessly.
    ///
    /// # Errors
    /// Rejects missing selected capabilities, headless Host, unknown slots,
    /// externally owned bindings and producer allocation/binding failures.
    pub fn present(&mut self, slot_id: UiRuntimeSlotId, target: RenderSurfaceId) -> Result<()> {
        ensure!(
            self.app
                .world()
                .resource::<UiPluginStateResource>()
                .is_ok_and(UiPluginStateResource::is_installed),
            "UiPlugin must be installed before presenting a mounted UI"
        );
        ensure!(
            render_integration_is_active(self.app.world()),
            "RenderPlugin must be installed before presenting a mounted UI"
        );
        ensure!(
            self.app
                .world()
                .resource::<UiRuntimeSlotsResource>()?
                .contains(slot_id),
            "cannot present an unknown or retired RunenUI slot"
        );
        ensure!(
            self.app.native_ui_clock().is_some(),
            "the current native-window presentation path requires a native Host"
        );

        let existing = self
            .app
            .world()
            .resource::<UiRuntimePresentationAssociationsResource>()?
            .binding(slot_id)
            .cloned();
        if let Some(mut binding) = existing {
            ensure!(
                self.app
                    .world()
                    .resource::<ManagedRenderProducerIds>()?
                    .is_owned_by(binding.producer_id, slot_id.value()),
                "existing RunenUI presentation binding is externally managed"
            );
            binding.render_surface_id = target;
            self.app
                .world_mut()
                .resource_mut::<UiRuntimePresentationAssociationsResource>()?
                .bind(binding)?;
            return Ok(());
        }

        let mut occupied = BTreeSet::<RenderFrameProducerId>::new();
        occupied.extend(
            self.app
                .world()
                .resource::<UiRuntimePresentationAssociationsResource>()?
                .bindings()
                .map(|binding| binding.producer_id),
        );
        occupied.extend(
            self.app
                .world()
                .resource::<SurfaceFrameSubmissionRegistryResource>()?
                .ordered_submissions()
                .into_iter()
                .map(|submission| submission.producer_id),
        );
        occupied.extend(
            self.app
                .world()
                .resource::<RunenUiPaintSubmissionRegistryResource>()?
                .ordered_submissions()
                .into_iter()
                .map(|submission| submission.producer_id),
        );

        let producer = self
            .app
            .world_mut()
            .resource_mut::<ManagedRenderProducerIds>()?
            .allocate(slot_id.value(), &occupied)?;
        let result = self
            .app
            .world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .bind(UiRuntimePresentationBinding::new(
                slot_id,
                producer,
                target,
                StyleEnvironment::default(),
            ));
        if let Err(error) = result {
            self.app
                .world_mut()
                .resource_mut::<ManagedRenderProducerIds>()?
                .release_owned(producer, slot_id.value());
            return Err(anyhow!("RunenUI placement rejected: {error}"));
        }
        Ok(())
    }

    /// Withdraws only an Engine-managed placement, without unmounting its
    /// independent typed RunenUI runtime.
    pub fn unpresent(&mut self, slot_id: UiRuntimeSlotId) -> Result<bool> {
        let existing = match self
            .app
            .world()
            .resource::<UiRuntimePresentationAssociationsResource>()
        {
            Ok(associations) => associations.binding(slot_id).cloned(),
            Err(_) => None,
        };
        let Some(binding) = existing else {
            // A low-level unbind may already have retired its association.
            // The managed caller may still settle its outstanding producer lease.
            return Ok(self
                .app
                .world_mut()
                .resource_mut::<ManagedRenderProducerIds>()
                .map(|leases| leases.release_for_owner(slot_id.value()))
                .unwrap_or(false));
        };
        let leased = self
            .app
            .world()
            .resource::<ManagedRenderProducerIds>()?
            .is_owned_by(binding.producer_id, slot_id.value());
        ensure!(
            leased,
            "cannot withdraw an externally owned UI presentation binding"
        );
        self.app
            .world_mut()
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .unbind(slot_id);
        self.app
            .world_mut()
            .resource_mut::<ManagedRenderProducerIds>()?
            .release_owned(binding.producer_id, slot_id.value());
        Ok(true)
    }

    /// Completes RunenUI shutdown before withdrawing the managed Render placement.
    /// On a fallible shutdown, both the runtime and its binding remain intact.
    pub fn unmount_presented(
        &mut self,
        slot_id: UiRuntimeSlotId,
    ) -> Result<Option<runenui_runtime::InputShutdownBatch>> {
        // Do not retire an application if its presentation is controlled by
        // another caller; otherwise failure after retirement would orphan paint.
        let existing = self
            .app
            .world()
            .resource::<UiRuntimePresentationAssociationsResource>()
            .ok()
            .and_then(|associations| associations.binding(slot_id).cloned());
        if let Some(binding) = &existing {
            ensure!(
                self.app
                    .world()
                    .resource::<ManagedRenderProducerIds>()?
                    .is_owned_by(binding.producer_id, slot_id.value()),
                "cannot unmount a UI with an externally managed presentation binding"
            );
        }
        let retirement = self
            .app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()?
            .unmount_with_retirement(slot_id)
            .map_err(|error| anyhow!("RunenUI shutdown rejected: {error:?}"))?;
        // A preceding expert unmount can leave its old presentation awaiting the
        // next RenderPrepare. Retire this managed association even if no slot remains.
        self.unpresent(slot_id)?;
        Ok(retirement)
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
    let clock = app.native_ui_clock();
    let report = {
        let slots = app
            .world_mut()
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("UiRuntimeSlotsResource was initialized before typed UI mounting");
        if let Err(failure) = slots.configure_host_clock(clock) {
            let report = UiRuntimeSlotMountReport::rejected(screen_id, failure);
            record_slot_mount_diagnostic(app, &report);
            return report;
        }
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
