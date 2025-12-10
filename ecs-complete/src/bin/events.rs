use ecs_complete::{Entity, events::Events};

#[derive(Debug, Clone)]
struct CollisionEvent {
    entity_a: Entity,
    entity_b: Entity,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

fn main() {
    let mut world = ecs_complete::World::new();

    // Create events resource
    world.insert_resource(Events::<CollisionEvent>::new());

    // Spawn some entities
    let e1 = world.spawn((Position { x: 0.0, y: 0.0 },));
    let e2 = world.spawn((Position { x: 0.5, y: 0.5 },));
    let e3 = world.spawn((Position { x: 10.0, y: 10.0 },));

    // Simulate collision detection
    {
        let mut events = world.get_resource_mut::<Events<CollisionEvent>>().unwrap();
        events.send(CollisionEvent {
            entity_a: e1,
            entity_b: e2,
        });
    }

    // Read events
    {
        let events = world.get_resource::<Events<CollisionEvent>>().unwrap();
        println!("Collisions this frame:");
        for event in events.iter() {
            println!("  {:?} collided with {:?}", event.entity_a, event.entity_b);
        }
    }

    // Update events (clear old events)
    {
        let mut events = world.get_resource_mut::<Events<CollisionEvent>>().unwrap();
        events.update();
    }

    // No events after update
    {
        let events = world.get_resource::<Events<CollisionEvent>>().unwrap();
        println!("\nEvents after update: {}", events.len());
    }
}
