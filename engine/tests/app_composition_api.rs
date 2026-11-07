//! #1426 exploratory compile/execution probe over the accepted Engine/RunenUI owner APIs.
//!
//! This test-local construction adapter is NOT a public App contract, a second
//! application runtime, or an implementation of #1423's proposed Host lifetime.
//! The admission point lowers deferred typed declarations into one real App.
//! It tests a viable ownership/error pattern, not acceptance of fluent syntax.

use std::{cell::Cell, rc::Rc};

use anyhow::{Result, anyhow};
use engine::plugins::render::{Gfx, RenderFlowRegistryResource};
use engine::plugins::ui::{
    UiPlugin, UiRuntimeFontConfiguration, UiRuntimeSlotId, UiRuntimeSlotsResource, UiTypedScreenId,
};
use engine::prelude::{App, AppRenderExt, AppUiExt};
use runenui_core::{Effects, NoHostProtocol, ProducerBudget, UiApp, View, text};

type PendingMount = Box<dyn FnOnce(&mut App, bool) -> Result<UiRuntimeSlotId>>;

/// Ephemeral declarations are consumed at admission; no parallel runtime remains.
#[derive(Default)]
struct CompositionProbe {
    native_window: bool,
    mounts: Vec<PendingMount>,
}

impl CompositionProbe {
    fn new() -> Self {
        Self::default()
    }

    fn native_window(mut self) -> Self {
        self.native_window = true;
        self
    }

    fn ui<A>(self, screen_id: UiTypedScreenId) -> Self
    where
        A: UiApp + 'static,
        A::State: Default + 'static,
    {
        self.ui_with_state::<A>(screen_id, A::State::default())
    }

    fn ui_with_state<A>(mut self, screen_id: UiTypedScreenId, state: A::State) -> Self
    where
        A: UiApp + 'static,
        A::State: 'static,
    {
        self.mounts.push(Box::new(move |app, native| {
            let fonts = if native {
                UiRuntimeFontConfiguration::default().with_system_fonts()
            } else {
                UiRuntimeFontConfiguration::default()
            };
            if native {
                let slot = app
                    .ui()
                    .mount_app_on_primary::<A>(screen_id, state, &fonts)?;
                // One ordinary native UI has an explicit real input recipient.
                app.ui().bind_native_primary_input(slot)?;
                Ok(slot)
            } else {
                let report = app.ui().mount_app::<A>(screen_id, state, &fonts);
                report
                    .slot_id()
                    .ok_or_else(|| anyhow!("headless typed UI mount rejected: {report:?}"))
            }
        }));
        self
    }

    /// Errors retain the actual App, including any owner-committed partial
    /// mounts, rather than dropping it behind a consuming Result<()>.
    fn admit(self) -> std::result::Result<App, (Box<App>, anyhow::Error)> {
        let mut app = if self.native_window {
            App::new()
        } else {
            App::headless()
        };
        if self.native_window && self.mounts.len() > 1 {
            return Err((
                Box::new(app),
                anyhow!(
                    "ordinary native-primary composition requires one UI; choose explicit placement for multiple instances"
                ),
            ));
        }
        if !self.mounts.is_empty() {
            if self.native_window {
                if let Err(error) = app.select_native_ui_only() {
                    return Err((Box::new(app), error));
                }
            } else {
                app.add_plugin(UiPlugin);
            }
        }
        for mount in self.mounts {
            if let Err(error) = mount(&mut app, self.native_window) {
                return Err((Box::new(app), error));
            }
        }
        Ok(app)
    }
}

struct DefaultUi;

impl UiApp for DefaultUi {
    type State = ();
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn root(_: &Self::State) -> impl View<Self::Action> {
        text("default")
    }

    fn update(
        _: &mut Self::State,
        (): Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        Effects::none()
    }

    fn initial_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
    }

    fn action_producer_budget(_: &Self::State, _: &Self::Action) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
    }

    fn interaction_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0)
    }
}

struct PersistedUi;

// No Default or Send bound: a real UiApp with local, non-Send state.
struct PersistedState(Rc<Cell<u32>>);

impl UiApp for PersistedUi {
    type State = PersistedState;
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn root(state: &Self::State) -> impl View<Self::Action> {
        text(format!("persisted {}", state.0.get()))
    }

    fn update(
        _: &mut Self::State,
        (): Self::Action,
    ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        Effects::none()
    }

    fn initial_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 0, 1)
    }

    fn action_producer_budget(_: &Self::State, _: &Self::Action) -> ProducerBudget {
        ProducerBudget::new(1, 0).with_reconciliation(1, 1, 1)
    }

    fn interaction_producer_budget(_: &Self::State) -> ProducerBudget {
        ProducerBudget::new(1, 0)
    }
}

fn accepted(probe: CompositionProbe) -> App {
    probe
        .admit()
        .unwrap_or_else(|(_app, error)| panic!("real App admission failed: {error:#}"))
}

#[test]
fn default_typed_ui_declares_in_either_native_order_without_second_flow_or_gpu() {
    for window_first in [true, false] {
        let probe = CompositionProbe::new();
        let probe = if window_first {
            probe
                .native_window()
                .ui::<DefaultUi>(UiTypedScreenId::new("proof.default"))
        } else {
            probe
                .ui::<DefaultUi>(UiTypedScreenId::new("proof.default"))
                .native_window()
        };
        let app = accepted(probe);
        assert_eq!(
            app.world()
                .resource::<UiRuntimeSlotsResource>()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            app.world()
                .resource::<RenderFlowRegistryResource>()
                .unwrap()
                .flow_count(),
            1
        );
        assert!(!app.world().has_resource::<Gfx>());
    }
}

#[test]
fn explicit_nondefault_nonsend_state_mounts_headlessly_without_present_or_gpu() {
    let local_state = PersistedState(Rc::new(Cell::new(7)));
    let app = accepted(
        CompositionProbe::new()
            .ui_with_state::<PersistedUi>(UiTypedScreenId::new("proof.persisted"), local_state),
    );
    assert_eq!(
        app.world()
            .resource::<UiRuntimeSlotsResource>()
            .unwrap()
            .len(),
        1
    );
    assert!(
        app.world()
            .resource::<RenderFlowRegistryResource>()
            .is_err()
    );
    assert!(!app.world().has_resource::<Gfx>());
}

#[test]
fn ambiguous_native_multiple_ui_is_rejected_before_owner_mounting() {
    let probe = CompositionProbe::new()
        .ui::<DefaultUi>(UiTypedScreenId::new("proof.first"))
        .ui::<DefaultUi>(UiTypedScreenId::new("proof.second"))
        .native_window();
    let (app, error) = match probe.admit() {
        Err(failure) => failure,
        Ok(_) => panic!("ambiguous native UI placement must reject"),
    };
    assert!(error.to_string().contains("explicit placement"));
    assert!(app.world().resource::<UiRuntimeSlotsResource>().is_err());
    assert!(
        app.world()
            .resource::<RenderFlowRegistryResource>()
            .is_err()
    );
    assert!(!app.world().has_resource::<Gfx>());
}
