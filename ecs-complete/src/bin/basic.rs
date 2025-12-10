use ecs_complete::{Schedule, World, system::QuerySystem};

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

#[derive(Debug)]
struct Time {
    delta: f32,
}

fn main() {
    let mut world = World::new();

    // Insert a resource
    world.insert_resource(Time { delta: 0.016 });

    // Spawn some entities
    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));
    world.spawn((Position { x: 10.0, y: 10.0 }, Velocity { x: -1.0, y: 1.0 }));
    world.spawn((Position { x: 5.0, y: 5.0 }, Velocity { x: 0.0, y: -1.0 }));

    // Create a schedule
    let mut schedule = Schedule::new();

    // Add a movement system with explicit type annotation
    schedule.add_update_system(
        QuerySystem::<(&mut Position, &Velocity), _>::new(
            |(pos, vel): (&mut Position, &Velocity)| {
                pos.x += vel.x;
                pos.y += vel.y;
            },
        )
        .with_name("movement_system"),
    );

    // Run the schedule for a few frames
    for frame in 0..5 {
        println!("\n=== Frame {} ===", frame);

        schedule.run(&mut world);

        // Print all positions
        for pos in world.query::<&Position>() {
            println!("Position: ({}, {})", pos.x, pos.y);
        }
    }
}
