use crate::runtime::fixed_step_executor::run_fixed_update_frame;
use crate::runtime::schedules::{
    FrameEnd, PreUpdate, RenderPrepare, RenderSubmit, Startup, Update,
};
use crate::runtime::window::WindowState;
use anyhow::Result;
use runen_ecs::{Runtime, World};

/// Applies builtin runtime run-state before startup/frame execution.
///
/// This does not install resources. Builtin resources are installed by
/// `App::install_builtin_resources` during app construction.
pub(crate) fn prepare_world_for_run(world: &mut World, title: &str, headless: bool) {
    if let Ok(window) = world.resource_mut::<WindowState>() {
        window.set_headless(headless);
        window.redraw_requested = false;
        window.close_requested = false;
        window.title = title.to_string();
    }
}

/// Runs `Startup` at most once for a runtime state.
pub(crate) fn run_startup_if_needed(
    world: &mut World,
    scheduler: &mut Runtime,
    startup_ran: &mut bool,
) -> Result<()> {
    if *startup_ran {
        return Ok(());
    }

    scheduler.run_schedule::<Startup>(world)?;
    *startup_ran = true;
    Ok(())
}

/// Runs one runtime frame using the canonical Engine-owned lifecycle order:
///
/// 1. `PreUpdate`
/// 2. fixed-step loop (`FixedUpdate` zero or more times)
/// 3. `Update`
/// 4. `RenderPrepare`
/// 5. `RenderSubmit`
/// 6. `FrameEnd`
///
/// RunenECS executes each generic schedule and owns its ECS deferred visibility.
/// Runenwerk product/query publication is installed explicitly by application
/// systems at the lifecycle positions that require it.
pub(crate) fn run_frame(world: &mut World, scheduler: &mut Runtime) -> Result<()> {
    scheduler.run_schedule::<PreUpdate>(world)?;
    run_fixed_update_frame(world, scheduler)?;
    scheduler.run_schedule::<Update>(world)?;
    scheduler.run_schedule::<RenderPrepare>(world)?;
    scheduler.run_schedule::<RenderSubmit>(world)?;
    scheduler.run_schedule::<FrameEnd>(world)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::time::domain::Time;
    use crate::runtime::fixed_time::{
        CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick,
    };
    use crate::runtime::publication::{
        PublicationHandlers, dispatch_product_publication_system,
        dispatch_query_snapshot_publication_system,
    };
    use crate::runtime::system::SystemConfigExt;
    use anyhow::anyhow;
    use runen_ecs::{Commands, Res, SystemSet};

    fn test_world() -> World {
        let mut world = World::new();
        let mut time = Time::default();
        time.delta_seconds = 0.0;
        world.insert_resource(time);
        world.insert_resource(FixedTimeConfig::default());
        world.insert_resource(CatchupBudget::default());
        world.insert_resource(FixedTimeState::default());
        world.insert_resource(SimulationTick(0));
        world
    }

    #[test]
    fn frame_end_failure_is_propagated() {
        fn fail_frame_end() -> anyhow::Result<()> {
            Err(anyhow!("frame end failure"))
        }

        let mut world = test_world();
        let mut runtime = Runtime::new();
        runtime.add_systems::<FrameEnd, _, _>(&mut world, fail_frame_end);

        let err = run_frame(&mut world, &mut runtime).expect_err("frame should fail");
        assert!(format!("{err:#}").contains("frame end failure"));
    }

    #[test]
    fn generic_lifecycle_schedules_do_not_dispatch_publication() {
        let mut world = test_world();
        world.insert_resource(PublicationHandlers::default());
        let product = std::rc::Rc::new(std::cell::RefCell::new(0));
        let product_observations = product.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |_, _| {
                *product_observations.borrow_mut() += 1;
                Ok(())
            });
        let query = std::rc::Rc::new(std::cell::RefCell::new(0));
        let query_observations = query.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |_, _| {
                *query_observations.borrow_mut() += 1;
                Ok(())
            });

        let mut runtime = Runtime::new();
        let mut startup_ran = false;
        run_startup_if_needed(&mut world, &mut runtime, &mut startup_ran).unwrap();
        run_frame(&mut world, &mut runtime).unwrap();

        assert_eq!(*product.borrow(), 0);
        assert_eq!(*query.borrow(), 0);
    }

    #[derive(Copy, Clone)]
    struct DeferredProducerSet;

    impl SystemSet for DeferredProducerSet {
        fn name() -> &'static str {
            "DeferredProducerSet"
        }
    }

    #[derive(Copy, Clone)]
    struct ProductPublicationSet;

    impl SystemSet for ProductPublicationSet {
        fn name() -> &'static str {
            "ProductPublicationSet"
        }
    }

    #[derive(Copy, Clone)]
    struct QueryPublicationSet;

    impl SystemSet for QueryPublicationSet {
        fn name() -> &'static str {
            "QueryPublicationSet"
        }
    }

    #[derive(Copy, Clone)]
    struct EmitDeferred(bool);

    impl runen_ecs::Resource for EmitDeferred {}

    #[derive(runen_ecs::Component)]
    struct FrontierMarker;

    fn queue_optional_deferred_commands(emit: Res<EmitDeferred>, mut commands: Commands) {
        if emit.0 {
            commands.spawn(FrontierMarker);
        }
    }

    fn explicit_publication_sequences(emit_deferred: bool) -> (Vec<u64>, Vec<u64>) {
        let mut world = test_world();
        world.insert_resource(EmitDeferred(emit_deferred));
        world.insert_resource(PublicationHandlers::default());
        let products = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let product_observations = products.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_product(move |occurrence, _| {
                product_observations
                    .borrow_mut()
                    .push(occurrence.sequence());
                Ok(())
            });
        let queries = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let query_observations = queries.clone();
        world
            .resource_mut::<PublicationHandlers>()
            .unwrap()
            .add_query_snapshot(move |occurrence, _| {
                query_observations.borrow_mut().push(occurrence.sequence());
                Ok(())
            });

        let mut runtime = Runtime::new();
        runtime.add_systems::<Update, _, _>(
            &mut world,
            queue_optional_deferred_commands.in_set(DeferredProducerSet),
        );
        runtime.add_systems::<Update, _, _>(
            &mut world,
            dispatch_product_publication_system
                .in_set(ProductPublicationSet)
                .after(DeferredProducerSet),
        );
        runtime.add_systems::<Update, _, _>(
            &mut world,
            dispatch_query_snapshot_publication_system
                .in_set(QueryPublicationSet)
                .after(ProductPublicationSet),
        );
        runtime.run_schedule::<Update>(&mut world).unwrap();

        (products.take(), queries.take())
    }

    #[test]
    fn explicit_publication_cardinality_is_independent_of_ecs_frontiers() {
        let without_deferred_activity = explicit_publication_sequences(false);
        let with_deferred_activity = explicit_publication_sequences(true);

        assert_eq!(without_deferred_activity, (vec![0], vec![0]));
        assert_eq!(with_deferred_activity, (vec![0], vec![0]));
    }
}
