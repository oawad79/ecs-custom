use ecs_complete::{Children, Parent, World};

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Name(&'static str);

fn main() {
    let mut world = World::new();

    // Create a parent entity
    let parent = world.spawn((Position { x: 0.0, y: 0.0 }, Name("Parent"), Children::new()));

    // Create child entities
    let child1 = world.spawn((Position { x: 1.0, y: 1.0 }, Name("Child 1"), Parent(parent)));

    let child2 = world.spawn((Position { x: 2.0, y: 2.0 }, Name("Child 2"), Parent(parent)));

    // Add children to parent
    if let Some(children) = world.get_mut::<Children>(parent) {
        children.add(child1);
        children.add(child2);
    }

    // Query parent and children
    println!("Parent:");
    if let Some(name) = world.get::<Name>(parent) {
        println!("  Name: {}", name.0);
    }

    if let Some(children) = world.get::<Children>(parent) {
        println!("  Children:");
        for &child in &children.0 {
            if let Some(name) = world.get::<Name>(child) {
                println!("    - {}", name.0);
            }
        }
    }
}
