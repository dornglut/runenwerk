use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ecs::{With, Without};
use engine::prelude::*;
use std::hint::black_box;

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource)]
struct Simulated;

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource)]
struct Disabled;

#[derive(Debug, Default, Component, ecs::Resource)]
struct FrameAccumulator(u64);

#[derive(Debug, Copy, Clone)]
struct PerfEvent(u32);

struct Phase35RuntimeBenchPlugin {
    entity_count: usize,
}

impl Plugin for Phase35RuntimeBenchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameAccumulator>();
        for i in 0..self.entity_count {
            if i % 10 == 0 {
                app.world_mut().spawn((
                    Position {
                        x: i as f32,
                        y: i as f32,
                    },
                    Velocity { x: 1.0, y: -0.5 },
                    Simulated,
                    Disabled,
                ));
            } else {
                app.world_mut().spawn((
                    Position {
                        x: i as f32,
                        y: i as f32,
                    },
                    Velocity { x: 1.0, y: -0.5 },
                    Simulated,
                ));
            }
        }

        app.add_systems(Update, (movement, emit_events, read_broadcast));
    }
}

#[allow(clippy::type_complexity)]
fn movement(mut query: Query<(&mut Position, &Velocity), (With<Simulated>, Without<Disabled>)>) {
    for (position, velocity) in query.iter() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn emit_events(mut writer: BroadcastWriter<PerfEvent>) {
    for i in 0..256_u32 {
        writer.send(PerfEvent(i));
    }
}

fn read_broadcast(reader: BroadcastReader<PerfEvent>, mut frame: ResMut<FrameAccumulator>) {
    let sum = reader
        .iter()
        .fold(0_u64, |acc, event| acc.wrapping_add(event.0 as u64));
    frame.0 = frame.0.wrapping_add(sum);
}

fn workload_engine_runtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_runtime_phase35");

    for &entity_count in &[5_000_usize, 20_000] {
        let mut app = App::headless();
        app.add_plugin(Phase35RuntimeBenchPlugin { entity_count });
        let mut app_slot = Some(app.run_for_frames(1).expect("startup frame should run"));

        group.bench_with_input(
            BenchmarkId::new("headless_mixed_frame", entity_count),
            &entity_count,
            |b, _| {
                b.iter(|| {
                    let mut app = app_slot
                        .take()
                        .expect("bench app state should always be available");
                    app = app.run_for_frames(1).expect("frame should run");
                    app.world_mut().clear_broadcast_admin::<PerfEvent>();
                    let frame = black_box(
                        app.world()
                            .resource::<FrameAccumulator>()
                            .expect("frame stats")
                            .0,
                    );
                    app_slot = Some(app);
                    frame
                });
            },
        );
    }

    group.finish();
}

criterion_group!(phase35_runtime, workload_engine_runtime);
criterion_main!(phase35_runtime);
