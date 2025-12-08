use ecs_slotmap::{Schedule, World, system::QuerySystem};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Debug)]
struct Health {
    current: u32,
    max: u32,
}

fn main() {
    let mut world = World::new();

    // Spawn some entities
    world.spawn((
        Position { x: 0.0, y: 0.0 },
        Velocity { dx: 1.0, dy: 0.5 },
        Health {
            current: 100,
            max: 100,
        },
    ));

    world.spawn((
        Position { x: 10.0, y: 10.0 },
        Velocity { dx: -0.5, dy: 1.0 },
        Health {
            current: 50,
            max: 100,
        },
    ));

    world.spawn((
        Position { x: 5.0, y: 5.0 },
        Health {
            current: 75,
            max: 100,
        },
    ));

    // Create systems
    let mut schedule = Schedule::new();

    // Movement system - applies velocity to position
    schedule.add_system(QuerySystem::<(&mut Position, &Velocity), _>::new(
        |(pos, vel)| {
            pos.x += vel.dx;
            pos.y += vel.dy;
        },
    ));

    // Health regeneration system
    schedule.add_system(QuerySystem::<&mut Health, _>::new(|health| {
        if health.current < health.max {
            health.current = (health.current + 1).min(health.max);
        }
    }));

    // Gravity system - applies downward velocity
    schedule.add_system(QuerySystem::<&mut Velocity, _>::new(|vel| {
        vel.dy -= 0.1; // gravity
    }));

    // Print system
    schedule.add_system(
        (|w: &mut World| {
            println!("\n=== Frame ===");
            for (pos, health) in w.query::<(&Position, &Health)>() {
                println!(
                    "Entity at ({:.1}, {:.1}) - Health: {}/{}",
                    pos.x, pos.y, health.current, health.max
                );
            }
        })
        .into_system(),
    );

    // Run simulation for 5 frames
    for frame in 0..5 {
        println!("\n--- Frame {} ---", frame);
        schedule.run(&mut world);
    }
}
