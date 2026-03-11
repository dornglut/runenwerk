use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ecs::{With, Without};
use engine::prelude::*;
use std::hint::black_box;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Simulated;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
struct Disabled;

#[derive(Debug, Default, Component)]
struct FrameAccumulator(u64);

#[derive(Debug, Copy, Clone)]
struct PerfEvent(u32);

struct Phase4RuntimeBenchPlugin {
    entity_count: usize,
}

impl Plugin for Phase4RuntimeBenchPlugin {
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

        app.add_systems(Update, (movement, emit_events, read_events));
    }
}

fn movement(mut query: Query<(&mut Position, &Velocity), (With<Simulated>, Without<Disabled>)>) {
    for (position, velocity) in query.iter() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn emit_events(mut writer: EventWriter<PerfEvent>) {
    for i in 0..256_u32 {
        writer.send(PerfEvent(i));
    }
}

fn read_events(reader: EventReader<PerfEvent>, mut frame: ResMut<FrameAccumulator>) {
    let sum = reader
        .iter()
        .fold(0_u64, |acc, event| acc.wrapping_add(event.0 as u64));
    (*frame).0 = (*frame).0.wrapping_add(sum);
}

fn workload_engine_runtime(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_runtime_phase4");

    for &entity_count in &[5_000_usize, 20_000] {
        let mut app = App::headless();
        app.add_plugin(Phase4RuntimeBenchPlugin { entity_count });
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
                    app.world_mut().clear_events::<PerfEvent>();
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

criterion_group!(phase4_runtime, workload_engine_runtime);
criterion_main!(phase4_runtime);
