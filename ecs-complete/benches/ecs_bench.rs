use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use ecs_complete::System;
use ecs_complete::World;

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Health(f32);

#[derive(Debug, Clone, Copy)]
struct Damage(f32);

fn spawn_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn");

    for size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut world = World::new();
                for i in 0..size {
                    world.spawn((
                        Position {
                            x: i as f32,
                            y: 0.0,
                        },
                        Velocity { x: 1.0, y: 1.0 },
                    ));
                }
                black_box(world);
            });
        });
    }

    group.finish();
}

fn query_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("query");

    for size in [100, 1_000, 10_000].iter() {
        let mut world = World::new();
        for i in 0..*size {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                },
                Velocity { x: 1.0, y: 1.0 },
            ));
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for (pos, vel) in world.query::<(&Position, &Velocity)>() {
                    black_box(pos);
                    black_box(vel);
                }
            });
        });
    }

    group.finish();
}

fn query_mut_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_mut");

    for size in [100, 1_000, 10_000].iter() {
        let mut world = World::new();
        for i in 0..*size {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                },
                Velocity { x: 1.0, y: 1.0 },
            ));
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for (pos, vel) in world.query::<(&mut Position, &Velocity)>() {
                    pos.x += vel.x;
                    pos.y += vel.y;
                    black_box(pos);
                }
            });
        });
    }

    group.finish();
}

fn insert_component_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_component");
    group.sample_size(10);

    for size in [100, 1_000, 10_000].iter() {
        group.bench_function(BenchmarkId::from_parameter(size), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let entities: Vec<_> = (0..*size)
                        .map(|i| {
                            world.spawn((Position {
                                x: i as f32,
                                y: 0.0,
                            },))
                        })
                        .collect();
                    (world, entities)
                },
                |(mut world, mut entities)| {
                    // Reverse iteration to avoid index issues
                    entities.reverse();
                    for &entity in &entities {
                        let _ = world.insert(entity, Velocity { x: 1.0, y: 1.0 });
                    }
                    black_box(world);
                },
            );
        });
    }

    group.finish();
}

fn remove_component_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("remove_component");
    group.sample_size(10);

    for size in [100, 1_000, 10_000].iter() {
        group.bench_function(BenchmarkId::from_parameter(size), |b| {
            b.iter_with_setup(
                || {
                    let mut world = World::new();
                    let entities: Vec<_> = (0..*size)
                        .map(|i| {
                            world.spawn((
                                Position {
                                    x: i as f32,
                                    y: 0.0,
                                },
                                Velocity { x: 1.0, y: 1.0 },
                            ))
                        })
                        .collect();
                    (world, entities)
                },
                |(mut world, mut entities)| {
                    // Reverse iteration to avoid index issues
                    entities.reverse();
                    for &entity in &entities {
                        let _ = world.remove::<Velocity>(entity);
                    }
                    black_box(world);
                },
            );
        });
    }

    group.finish();
}

fn despawn_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("despawn");

    for size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut world = World::new();
                let mut entities: Vec<_> = (0..size)
                    .map(|i| {
                        world.spawn((Position {
                            x: i as f32,
                            y: 0.0,
                        },))
                    })
                    .collect();

                // Reverse to avoid index issues with swap_remove
                entities.reverse();
                for entity in entities {
                    world.despawn(entity);
                }
                black_box(world);
            });
        });
    }

    group.finish();
}

fn fragmented_query_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("fragmented_query");

    for size in [100, 1_000, 10_000].iter() {
        let mut world = World::new();

        // Create fragmented archetypes
        for i in 0..*size {
            if i % 4 == 0 {
                world.spawn((Position {
                    x: i as f32,
                    y: 0.0,
                },));
            } else if i % 4 == 1 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                    Velocity { x: 1.0, y: 1.0 },
                ));
            } else if i % 4 == 2 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                    Health(100.0),
                ));
            } else {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                    },
                    Velocity { x: 1.0, y: 1.0 },
                    Health(100.0),
                ));
            }
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for pos in world.query::<&Position>() {
                    black_box(pos);
                }
            });
        });
    }

    group.finish();
}

fn system_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("system");

    for size in [100, 1_000, 10_000].iter() {
        let mut world = World::new();
        for i in 0..*size {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                },
                Velocity { x: 1.0, y: 1.0 },
            ));
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for (pos, vel) in world.query::<(&mut Position, &Velocity)>() {
                    pos.x += vel.x;
                    pos.y += vel.y;
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    spawn_benchmark,
    query_benchmark,
    query_mut_benchmark,
    insert_component_benchmark,
    remove_component_benchmark,
    despawn_benchmark,
    fragmented_query_benchmark,
    system_benchmark,
);
criterion_main!(benches);
