//! #1426 exploratory compile/execution probe over the accepted Engine/RunenUI owner APIs.
//!
//! This test-local construction adapter is NOT a public App contract, a second
//! application runtime, or an implementation of #1423's proposed Host lifetime.
//! The admission point lowers deferred typed declarations into one real App.
//! It tests a viable ownership/error pattern, not acceptance of fluent syntax.

use std::{cell::Cell, marker::PhantomData, rc::Rc};

use anyhow::{Result, anyhow};
use engine::plugins::render::{Gfx, RenderFlowRegistryResource};
use engine::plugins::ui::{
    UiPlugin, UiRuntimeFontConfiguration, UiRuntimeSlotId, UiRuntimeSlotsResource, UiTypedScreenId,
};
use engine::prelude::{App, AppRenderExt, AppUiExt};
use runenui_core::{Effects, NoHostProtocol, ProducerBudget, UiApp, View, text};

type PendingMount = Box<dyn FnOnce(&mut App, bool) -> Result<UiRuntimeSlotId>>;

/// Test-local typed intent for the candidate `.ui(A::new(state))` spelling.
/// It stages, but neither mounts a RunenUI runtime nor creates a second owner.
struct UiIntent<A: UiApp> {
    state: A::State,
    _app: PhantomData<fn() -> A>,
}

/// A blanket constructor keeps typed UiApp state separate from the zero-sized
/// application type; the public naming/import collision still needs review.
trait UiConstruct: UiApp + Sized {
    fn new(state: Self::State) -> UiIntent<Self> {
        UiIntent {
            state,
            _app: PhantomData,
        }
    }
}

impl<A: UiApp> UiConstruct for A {}

/// Ephemeral declarations are consumed at admission; no parallel runtime remains.
#[derive(Default)]
struct CompositionProbe {
    native_window: bool,
    mounts: Vec<PendingMount>,
}

/// Test-only scope; declarations are lowered into the real App at admission.
struct CompositionScope<'a> {
    draft: &'a mut CompositionProbe,
}

impl CompositionScope<'_> {
    fn ui<A>(&mut self, intent: UiIntent<A>)
    where
        A: UiApp + 'static,
        A::State: 'static,
    {
        self.draft.register_ui(intent);
    }
}

impl CompositionProbe {
    fn new() -> Self {
        Self::default()
    }

    /// One fallible composition scope: on error return unadmitted typed
    /// declarations to the caller, without claiming owner-side rollback.
    /// Does not prove failures after real Engine mount or consuming run.
    fn compose<E>(
        mut self,
        configure: impl FnOnce(&mut CompositionScope<'_>) -> std::result::Result<(), E>,
    ) -> std::result::Result<Self, (Self, E)> {
        let outcome = configure(&mut CompositionScope { draft: &mut self });
        match outcome {
            Ok(()) => Ok(self),
            Err(error) => Err((self, error)),
        }
    }

    fn native_window(mut self) -> Self {
        self.native_window = true;
        self
    }

    fn ui_default<A>(self, screen_id: UiTypedScreenId) -> Self
    where
        A: UiApp + 'static,
        A::State: Default + 'static,
    {
        self.ui_with_state::<A>(screen_id, A::State::default())
    }

    /// Test-only intent registration. Runtime slot identity is allocated by the
    /// existing Engine owner on admission; this provisional screen ID is
    /// for the local proof and is not a durable or cross-session identity.
    fn ui<A>(mut self, intent: UiIntent<A>) -> Self
    where
        A: UiApp + 'static,
        A::State: 'static,
    {
        self.register_ui(intent);
        self
    }

    fn register_ui<A>(&mut self, intent: UiIntent<A>)
    where
        A: UiApp + 'static,
        A::State: 'static,
    {
        let screen_id = UiTypedScreenId::new(format!("proof.intent.{}", self.mounts.len()));
        let draft = std::mem::take(self).ui_with_state::<A>(screen_id, intent.state);
        *self = draft;
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
                .ui_default::<DefaultUi>(UiTypedScreenId::new("proof.default"))
        } else {
            probe
                .ui_default::<DefaultUi>(UiTypedScreenId::new("proof.default"))
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
        .ui_default::<DefaultUi>(UiTypedScreenId::new("proof.first"))
        .ui_default::<DefaultUi>(UiTypedScreenId::new("proof.second"))
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

#[test]
fn constructor_style_ui_intent_preserves_nondefault_nonsend_typed_state() {
    let state = PersistedState(Rc::new(Cell::new(37)));
    let app = accepted(CompositionProbe::new().ui(PersistedUi::new(state)));

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
fn constructor_style_ui_intent_lowers_into_existing_native_ui_only_owner() {
    let app = accepted(
        CompositionProbe::new()
            .ui(DefaultUi::new(()))
            .native_window(),
    );

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

/// Real Rust method resolution favors an inherent new() over a blanket
/// UiConstruct::new(state). The preferred spelling is not universally valid.
struct ExistingNewUi;

impl ExistingNewUi {
    fn new() -> Self {
        Self
    }
}

impl UiApp for ExistingNewUi {
    type State = PersistedState;
    type Action = ();
    type HostProtocol = NoHostProtocol;

    fn root(state: &Self::State) -> impl View<Self::Action> {
        text(format!("existing {}", state.0.get()))
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

#[test]
fn inherent_new_requires_explicit_trait_constructor_to_register_typed_state() {
    let _existing_inherent_constructor = ExistingNewUi::new();
    let state = PersistedState(Rc::new(Cell::new(49)));
    let app = accepted(CompositionProbe::new().ui(<ExistingNewUi as UiConstruct>::new(state)));
    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 1);
    let slot = slots.slot_ids().next().unwrap();
    assert_eq!(
        slots
            .observe_app::<ExistingNewUi, _>(slot, |view| view.state().0.get())
            .unwrap(),
        49
    );
}

#[test]
fn fallible_compose_returns_staged_state_and_original_error_before_owner_admission() {
    let shared = Rc::new(Cell::new(73));
    let attempted = CompositionProbe::new().compose(|scope| {
        scope.ui(PersistedUi::new(PersistedState(Rc::clone(&shared))));
        // A later fallible state load propagates with ?. The earlier
        // registration was staged, not mounted into a partial Engine App.
        let unavailable: Option<PersistedState> = None;
        scope.ui(PersistedUi::new(
            unavailable.ok_or_else(|| anyhow!("additional state unavailable"))?,
        ));
        Ok::<(), anyhow::Error>(())
    });
    let (draft, error) = match attempted {
        Ok(_) => panic!("failed composed state load must not claim success"),
        Err(failure) => failure,
    };
    assert_eq!(error.to_string(), "additional state unavailable");
    assert_eq!(draft.mounts.len(), 1);

    // Explicit recovery mounts the still-owned draft into one real owner.
    let app = accepted(draft);
    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 1);
    let slot = slots.slot_ids().next().unwrap();
    assert_eq!(
        slots
            .observe_app::<PersistedUi, _>(slot, |view| view.state().0.get())
            .unwrap(),
        73
    );
    assert!(app.world().resource::<RenderFlowRegistryResource>().is_err());
    assert!(!app.world().has_resource::<Gfx>());
}

#[test]
fn fallible_compose_admits_two_independent_headless_ui_states_without_render() {
    let first = Rc::new(Cell::new(11));
    let second = Rc::new(Cell::new(29));
    let draft = CompositionProbe::new()
        .compose(|scope| {
            scope.ui(PersistedUi::new(PersistedState(Rc::clone(&first))));
            scope.ui(PersistedUi::new(PersistedState(Rc::clone(&second))));
            Ok::<(), anyhow::Error>(())
        })
        .unwrap_or_else(|(_, error)| panic!("valid typed declarations rejected: {error:#}"));

    let app = accepted(draft);
    let slots = app.world().resource::<UiRuntimeSlotsResource>().unwrap();
    assert_eq!(slots.len(), 2);
    let values = slots
        .slot_ids()
        .map(|slot| {
            slots
                .observe_app::<PersistedUi, _>(slot, |view| view.state().0.get())
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(values, vec![11, 29]);
    assert!(app.world().resource::<RenderFlowRegistryResource>().is_err());
    assert!(!app.world().has_resource::<Gfx>());
}
