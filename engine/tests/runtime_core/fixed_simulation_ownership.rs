use engine::plugins::{FixedStepPlugin, SimulationPlugin};
use engine::prelude::*;

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct FixedCounter(u32);

fn count_fixed(mut counter: ResMut<FixedCounter>) {
    counter.0 = counter.0.saturating_add(1);
}

#[derive(Debug, Default, Component, runen_ecs::Resource)]
struct ObservedTicks(Vec<u64>);

fn observe_simulation_tick(tick: Res<SimulationTick>, mut observed: ResMut<ObservedTicks>) {
    observed.0.push(tick.0);
}

#[test]
fn bare_app_has_no_fixed_cadence_or_simulation_owner_state() {
    let app = App::headless();

    assert!(app.world().resource::<FixedTimeConfig>().is_err());
    assert!(app.world().resource::<CatchupBudget>().is_err());
    assert!(app.world().resource::<FixedTimeState>().is_err());
    assert!(app.world().resource::<SimulationTick>().is_err());
    assert!(app.world().resource::<SimulationProfileConfig>().is_err());
    assert!(app.world().resource::<SimulationSessionId>().is_err());
    assert!(app.world().resource::<SimulationSeed>().is_err());
    assert!(app.world().resource::<SimulationRng>().is_err());
}

#[test]
fn simulation_owner_query_preserves_absent_tick_behavior() {
    let app = App::headless();

    assert_eq!(app.current_tick(), 0);
}

#[test]
fn public_cadence_resources_do_not_activate_fixed_update() {
    let mut app = App::headless();
    app.insert_resource(FixedTimeConfig::default());
    app.insert_resource(CatchupBudget::default());
    app.insert_resource(FixedTimeState::default());
    app.insert_resource(FixedCounter::default());
    app.add_systems(FixedUpdate, count_fixed);

    let app = app
        .run_for_frames(1)
        .expect("ordinary frame advancement should remain valid");

    assert_eq!(app.world().resource::<FixedCounter>().unwrap().0, 0);
    assert_eq!(
        app.world()
            .resource::<FixedTimeState>()
            .unwrap()
            .total_completed_steps,
        0
    );
}

#[test]
fn fixed_step_plugin_provides_and_preserves_cadence_state() {
    let mut app = App::headless();
    let config = FixedTimeConfig { step_seconds: 0.02 };
    let budget = CatchupBudget {
        max_steps_per_frame: 7,
    };
    let state = FixedTimeState {
        accumulator_seconds: 0.01,
        steps_ran_last_frame: 2,
        saturated_frames: 3,
        total_completed_steps: 4,
    };
    app.insert_resource(config);
    app.insert_resource(budget);
    app.insert_resource(state);

    app.add_plugin(FixedStepPlugin);

    assert_eq!(*app.world().resource::<FixedTimeConfig>().unwrap(), config);
    assert_eq!(*app.world().resource::<CatchupBudget>().unwrap(), budget);
    assert_eq!(*app.world().resource::<FixedTimeState>().unwrap(), state);
}

#[test]
fn fixed_step_plugin_runs_without_manufacturing_simulation_identity() {
    let mut app = App::headless();
    app.add_plugin(FixedStepPlugin);
    app.insert_resource(FixedCounter::default());
    app.add_systems(FixedUpdate, count_fixed);

    let app = app
        .run_for_fixed_steps(1)
        .expect("selected fixed cadence should support bounded fixed-step advancement");

    assert_eq!(app.world().resource::<FixedCounter>().unwrap().0, 1);
    assert_eq!(
        app.world()
            .resource::<FixedTimeState>()
            .unwrap()
            .total_completed_steps,
        1
    );
    assert!(app.world().resource::<SimulationTick>().is_err());
}

#[test]
fn simulation_plugin_provides_simulation_state_without_activating_fixed_cadence() {
    let mut app = App::headless();
    app.add_plugin(SimulationPlugin);
    app.insert_resource(FixedCounter::default());
    app.add_systems(FixedUpdate, count_fixed);

    let app = app
        .run_for_frames(1)
        .expect("simulation integration alone should not imply fixed cadence");

    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 0);
    assert!(app.world().resource::<SimulationProfileConfig>().is_ok());
    assert!(app.world().resource::<SimulationSessionId>().is_ok());
    assert!(app.world().resource::<SimulationSeed>().is_ok());
    assert!(app.world().resource::<SimulationRng>().is_ok());
    assert!(app.world().resource::<FixedTimeState>().is_err());
    assert_eq!(app.world().resource::<FixedCounter>().unwrap().0, 0);
}

#[test]
fn simulation_session_identity_is_scoped_to_each_app() {
    let mut first = App::headless();
    first.add_plugin(SimulationPlugin);
    let mut second = App::headless();
    second.add_plugin(SimulationPlugin);

    let first_session = *first.world().resource::<SimulationSessionId>().unwrap();
    let second_session = *second.world().resource::<SimulationSessionId>().unwrap();

    assert_eq!(
        first_session, second_session,
        "independent Apps must not consume process-global session allocation state"
    );
}

#[test]
fn simulation_plugin_preserves_explicit_owner_state() {
    let mut app = App::headless();
    let tick = SimulationTick(41);
    let profile = SimulationProfileConfig {
        profile: SimulationProfile::RollbackSession,
        authority: AuthorityRole::Server,
        determinism: DeterminismLevel::Strict,
    };
    let session = SimulationSessionId(99);
    let seed = SimulationSeed(7);
    let mut rng = SimulationRng::from_seed(seed);
    let _ = rng.next_u64();
    let expected_rng = rng.clone();

    app.insert_resource(tick);
    app.insert_resource(profile);
    app.insert_resource(session);
    app.insert_resource(seed);
    app.insert_resource(rng);
    app.add_plugin(SimulationPlugin);

    assert_eq!(*app.world().resource::<SimulationTick>().unwrap(), tick);
    assert_eq!(
        *app.world().resource::<SimulationProfileConfig>().unwrap(),
        profile
    );
    assert_eq!(
        *app.world().resource::<SimulationSessionId>().unwrap(),
        session
    );
    assert_eq!(*app.world().resource::<SimulationSeed>().unwrap(), seed);
    assert_eq!(
        *app.world().resource::<SimulationRng>().unwrap(),
        expected_rng
    );
}

#[test]
fn simulation_tick_advances_in_fixed_step_begin_before_fixed_update() {
    let mut app = App::headless();
    app.add_plugins((FixedStepPlugin, SimulationPlugin));
    app.insert_resource(ObservedTicks::default());
    app.add_systems(FixedUpdate, observe_simulation_tick);

    let app = app
        .run_for_fixed_steps(3)
        .expect("fixed and simulation integration should advance together");

    assert_eq!(
        app.world().resource::<ObservedTicks>().unwrap().0,
        vec![1, 2, 3]
    );
    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 3);
    assert_eq!(
        app.world()
            .resource::<FixedTimeState>()
            .unwrap()
            .total_completed_steps,
        3
    );
}

#[test]
fn run_for_fixed_steps_rejects_when_fixed_cadence_is_not_selected() {
    let result = App::headless().run_for_fixed_steps(1);
    assert!(result.is_err());
}

#[test]
fn repeated_bounded_fixed_step_runs_use_cadence_progress_not_simulation_tick_target() {
    let mut app = App::headless();
    app.add_plugins((FixedStepPlugin, SimulationPlugin));

    let mut app = app
        .run_for_fixed_steps(2)
        .expect("first bounded run should complete two fixed steps");
    app.world_mut().resource_mut::<SimulationTick>().unwrap().0 = 999;

    let app = app
        .run_for_fixed_steps(1)
        .expect("second bounded run should be relative to cadence progress");

    assert_eq!(
        app.world()
            .resource::<FixedTimeState>()
            .unwrap()
            .total_completed_steps,
        3
    );
    assert_eq!(app.world().resource::<SimulationTick>().unwrap().0, 1000);
}

#[test]
fn explicit_app_simulation_configuration_is_preserved_by_simulation_plugin() {
    let mut app = App::headless();
    let seed = SimulationSeed(0x1234);
    app.set_simulation_profile(SimulationProfile::DeterministicLockstep);
    app.set_authority_role(AuthorityRole::Server);
    app.set_simulation_seed(seed);

    assert!(app.world().resource::<SimulationTick>().is_err());
    assert!(app.world().resource::<SimulationSessionId>().is_err());
    assert_eq!(
        app.world()
            .resource::<SimulationProfileConfig>()
            .unwrap()
            .authority,
        AuthorityRole::Server
    );
    assert_eq!(*app.world().resource::<SimulationSeed>().unwrap(), seed);

    let rng_before = app.world().resource::<SimulationRng>().unwrap().clone();
    app.add_plugin(SimulationPlugin);

    let profile = app.world().resource::<SimulationProfileConfig>().unwrap();
    assert_eq!(profile.profile, SimulationProfile::DeterministicLockstep);
    assert_eq!(profile.authority, AuthorityRole::Server);
    assert_eq!(profile.determinism, DeterminismLevel::Strict);
    assert_eq!(*app.world().resource::<SimulationSeed>().unwrap(), seed);
    assert_eq!(
        *app.world().resource::<SimulationRng>().unwrap(),
        rng_before
    );
    assert!(app.world().resource::<SimulationTick>().is_ok());
    assert!(app.world().resource::<SimulationSessionId>().is_ok());
}
