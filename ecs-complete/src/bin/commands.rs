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

fn main() {
    let mut world = World::new();

    // Spawn initial entities
    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

    println!("Initial entities: {}", count_entities(&mut world));

    // Use commands to spawn entities
    {
        let commands = world.commands();
        commands.spawn((Position { x: 5.0, y: 5.0 }, Velocity { x: -1.0, y: -1.0 }));
        commands.spawn((Position { x: 10.0, y: 10.0 }, Velocity { x: 0.0, y: 1.0 }));
    }

    // Commands are not applied yet
    println!("Before flush: {}", count_entities(&mut world));

    // Flush commands
    world.flush_commands();

    // Now entities are spawned
    println!("After flush: {}", count_entities(&mut world));

    // Print all positions
    for pos in world.query::<&Position>() {
        println!("Position: ({}, {})", pos.x, pos.y);
    }
}

fn count_entities(world: &mut World) -> usize {
    world.query::<&Position>().count()
}
