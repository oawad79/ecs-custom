pub mod archetype;
pub mod entity;
pub mod query;
pub mod system;
pub mod world;

pub use entity::Entity;
pub use query::{Query, QueryBorrow};
pub use system::{IntoSystem, Schedule, System};
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use system::QuerySystem;

    #[derive(Debug, PartialEq)]
    struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Velocity {
        pub dx: f32,
        pub dy: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Health(u32);

    #[test]
    fn test_spawn_entity() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 0.0, y: 0.0 },));

        assert!(world.is_alive(entity));
    }

    #[test]
    fn test_get_component() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 10.0, y: 20.0 },));

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
    }

    #[test]
    fn test_get_component_mut() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 10.0, y: 20.0 },));

        {
            let pos = world.get_mut::<Position>(entity).unwrap();
            pos.x = 30.0;
        }

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 30.0);
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 0.0, y: 0.0 },));

        assert!(world.despawn(entity));
        assert!(!world.is_alive(entity));
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let entity = world.spawn((Position { x: 1.0, y: 2.0 }, Velocity { dx: 3.0, dy: 4.0 }));

        let pos = world.get::<Position>(entity).unwrap();
        let vel = world.get::<Velocity>(entity).unwrap();

        assert_eq!(pos.x, 1.0);
        assert_eq!(vel.dx, 3.0);
    }

    #[test]
    fn test_query_single() {
        let mut world = World::new();

        world.spawn((Position { x: 1.0, y: 2.0 },));
        world.spawn((Position { x: 3.0, y: 4.0 },));

        let mut count = 0;
        for pos in world.query::<&Position>() {
            assert!(pos.x > 0.0);
            count += 1;
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_multiple() {
        let mut world = World::new();

        world.spawn((Position { x: 1.0, y: 2.0 }, Velocity { dx: 0.1, dy: 0.2 }));
        world.spawn((Position { x: 3.0, y: 4.0 }, Velocity { dx: 0.3, dy: 0.4 }));
        world.spawn((Position { x: 5.0, y: 6.0 },)); // No velocity

        let mut count = 0;
        for (pos, vel) in world.query::<(&Position, &Velocity)>() {
            assert!(pos.x > 0.0);
            assert!(vel.dx > 0.0);
            count += 1;
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn test_query_mut() {
        let mut world = World::new();

        world.spawn((Position { x: 1.0, y: 2.0 },));
        world.spawn((Position { x: 3.0, y: 4.0 },));

        for pos in world.query::<&mut Position>() {
            pos.x += 10.0;
        }

        for pos in world.query::<&Position>() {
            assert!(pos.x >= 11.0);
        }
    }

    #[test]
    fn test_query_system() {
        let mut world = World::new();

        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 2.0 }));
        world.spawn((
            Position { x: 10.0, y: 10.0 },
            Velocity { dx: -1.0, dy: -2.0 },
        ));

        // Create a movement system with explicit type annotation
        let mut movement_system = QuerySystem::<(&mut Position, &Velocity), _>::new(
            |(pos, vel): (&mut Position, &Velocity)| {
                pos.x += vel.dx;
                pos.y += vel.dy;
            },
        );

        movement_system.run(&mut world);

        // Check positions were updated
        let mut found_first = false;
        let mut found_second = false;

        for pos in world.query::<&Position>() {
            if (pos.x - 1.0).abs() < 0.001 && (pos.y - 2.0).abs() < 0.001 {
                found_first = true;
            }
            if (pos.x - 9.0).abs() < 0.001 && (pos.y - 8.0).abs() < 0.001 {
                found_second = true;
            }
        }

        assert!(found_first);
        assert!(found_second);
    }

    #[test]
    fn test_schedule() {
        let mut world = World::new();

        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 1.0 }));

        let mut schedule = Schedule::new();

        // Add movement system with explicit type annotation
        schedule.add_system(QuerySystem::<(&mut Position, &Velocity), _>::new(
            |(pos, vel): (&mut Position, &Velocity)| {
                pos.x += vel.dx;
                pos.y += vel.dy;
            },
        ));

        // Add velocity damping system
        schedule.add_system(QuerySystem::<&mut Velocity, _>::new(
            |vel: &mut Velocity| {
                vel.dx *= 0.9;
                vel.dy *= 0.9;
            },
        ));

        // Run schedule multiple times
        for _ in 0..3 {
            schedule.run(&mut world);
        }

        // Verify systems ran
        for pos in world.query::<&Position>() {
            assert!(pos.x > 0.0);
            assert!(pos.y > 0.0);
        }
    }

    #[test]
    fn test_function_system() {
        let mut world = World::new();

        world.spawn((Position { x: 0.0, y: 0.0 },));

        let mut count = 0;
        let mut counting_system = (|w: &mut World| {
            for _ in w.query::<&Position>() {
                count += 1;
            }
        })
        .into_system();

        counting_system.run(&mut world);

        assert_eq!(count, 1);
    }
}
