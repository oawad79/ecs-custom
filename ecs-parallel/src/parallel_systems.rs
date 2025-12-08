use simple_ecs::*;
use system::{QuerySystem, Stage};

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

#[derive(Debug)]
struct Damage(u32);

fn main() {
    let mut world = World::new();

    // Spawn entities
    for i in 0..10 {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
            },
            Velocity { dx: 1.0, dy: 0.5 },
            Health {
                current: 100,
                max: 100,
            },
        ));
    }

    world.spawn((Position { x: 5.0, y: 5.0 }, Damage(10)));

    let mut schedule = system::Schedule::new();

    // These systems can run in parallel (no conflicts)
    schedule.add_system(
        Stage::Update,
        QuerySystem::<(&mut Position, &Velocity), _>::new(|(pos, vel)| {
            pos.x += vel.dx;
            pos.y += vel.dy;
        })
        .with_name("movement_system"),
    );

    schedule.add_system(
        Stage::Update,
        QuerySystem::<&mut Health, _>::new(|health| {
            if health.current < health.max {
                health.current = (health.current + 1).min(health.max);
            }
        })
        .with_name("health_regen_system"),
    );

    // This system conflicts with movement (writes Velocity)
    schedule.add_system(
        Stage::Update,
        QuerySystem::<&mut Velocity, _>::new(|vel| {
            vel.dy -= 0.1;
        })
        .with_name("gravity_system"),
    );

    // Print system runs after update
    schedule.add_system(
        Stage::PostUpdate,
        (|w: &mut World| {
            println!("\n=== Entities ===");
            let mut count = 0;
            for (pos, health) in w.query::<(&Position, &Health)>() {
                if count < 3 {
                    println!(
                        "Entity at ({:.1}, {:.1}) - Health: {}/{}",
                        pos.x, pos.y, health.current, health.max
                    );
                }
                count += 1;
            }
            if count > 3 {
                println!("... and {} more entities", count - 3);
            }
        })
        .into_system(),
    );

    println!("Running simulation with parallel-capable scheduling...");

    for frame in 0..5 {
        println!("\n--- Frame {} ---", frame);
        schedule.run(&mut world);
    }
}
