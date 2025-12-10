pub mod archetype;
pub mod command;
pub mod component;
pub mod ecs_bench;
pub mod entity;
pub mod error;
pub mod events;
pub mod hierarchy;
pub mod query;
pub mod resource;
pub mod system;
pub mod world;

pub use command::Commands;
pub use component::{Bundle, Component};
pub use ecs_bench::*;
pub use entity::Entity;
pub use error::{EcsError, Result};
pub use hierarchy::{Children, Parent};
pub use query::{Changed, Query, With, Without};
pub use resource::{Res, ResMut, Resources};
pub use system::{IntoSystem, ParallelSchedule, Schedule, Stage, System};
pub use world::World;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventReader, Events};

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Health(f32);

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Player;

    #[test]
    fn test_spawn_and_query() {
        let mut world = World::new();

        let e1 = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
        let e2 = world.spawn((Position { x: 5.0, y: 5.0 }, Velocity { x: -1.0, y: -1.0 }));

        let mut count = 0;
        for (pos, vel) in world.query::<(&Position, &Velocity)>() {
            count += 1;
            assert!(pos.x == 0.0 || pos.x == 5.0);
            assert!(vel.x == 1.0 || vel.x == -1.0);
        }
        assert_eq!(count, 2);

        assert!(world.is_alive(e1));
        assert!(world.is_alive(e2));
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();

        let e1 = world.spawn((Position { x: 0.0, y: 0.0 },));
        let e2 = world.spawn((Position { x: 5.0, y: 5.0 },));

        assert!(world.despawn(e1));
        assert!(!world.is_alive(e1));
        assert!(world.is_alive(e2));

        let mut count = 0;
        for _pos in world.query::<&Position>() {
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_component() {
        let mut world = World::new();

        let entity = world.spawn((Position { x: 10.0, y: 20.0 }, Velocity { x: 1.0, y: 2.0 }));

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);

        let vel = world.get_mut::<Velocity>(entity).unwrap();
        vel.x = 5.0;

        let vel = world.get::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 5.0);
    }

    #[test]
    fn test_insert_component() {
        let mut world = World::new();

        let entity = world.spawn((Position { x: 0.0, y: 0.0 },));
        assert!(world.get::<Velocity>(entity).is_none());

        world.insert(entity, Velocity { x: 1.0, y: 1.0 }).unwrap();
        assert!(world.get::<Velocity>(entity).is_some());

        let vel = world.get::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 1.0);
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();

        let entity = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
        assert!(world.get::<Velocity>(entity).is_some());

        let vel = world.remove::<Velocity>(entity).unwrap();
        assert_eq!(vel.x, 1.0);
        assert!(world.get::<Velocity>(entity).is_none());
        assert!(world.get::<Position>(entity).is_some());
    }

    #[test]
    fn test_resources() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct Time(f32);

        world.insert_resource(Time(0.0));

        {
            let time = world.get_resource::<Time>().unwrap();
            assert_eq!(time.0, 0.0);
        }

        {
            let mut time = world.get_resource_mut::<Time>().unwrap();
            time.0 = 1.5;
        }

        {
            let time = world.get_resource::<Time>().unwrap();
            assert_eq!(time.0, 1.5);
        }

        let time = world.remove_resource::<Time>().unwrap();
        assert_eq!(time.0, 1.5);
        assert!(world.get_resource::<Time>().is_none());
    }

    #[test]
    fn test_events() {
        let mut events = Events::<i32>::new();

        events.send(1);
        events.send(2);
        events.send(3);

        let collected: Vec<_> = events.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);

        events.update();
        assert!(events.is_empty());
    }

    #[test]
    fn test_event_reader() {
        let mut events = Events::<i32>::new();

        events.send(1);
        events.send(2);

        let mut reader = EventReader::new(&events);
        let collected: Vec<_> = reader.iter().copied().collect();
        assert_eq!(collected, vec![1, 2]);

        // Drop the reader before mutating events
        drop(reader);

        events.send(3);

        // Create a new reader that will see all events
        let mut reader = EventReader::new(&events);
        let collected: Vec<_> = reader.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_hierarchy() {
        let mut world = World::new();

        let parent = world.spawn((Position { x: 0.0, y: 0.0 }, Children::new()));
        let child = world.spawn((Position { x: 1.0, y: 1.0 }, Parent(parent)));

        if let Some(children) = world.get_mut::<Children>(parent) {
            children.add(child);
        }

        let children = world.get::<Children>(parent).unwrap();
        assert_eq!(children.0.len(), 1);
        assert_eq!(children.0[0], child);

        let parent_comp = world.get::<Parent>(child).unwrap();
        assert_eq!(parent_comp.0, parent);
    }

    #[test]
    fn test_commands() {
        let mut world = World::new();

        let mut commands = Commands::new();
        commands.spawn((Position { x: 0.0, y: 0.0 },));
        commands.spawn((Position { x: 1.0, y: 1.0 },));

        commands.apply(&mut world);

        let mut count = 0;
        for _pos in world.query::<&Position>() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_option_query() {
        let mut world = World::new();

        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
        world.spawn((Position { x: 1.0, y: 1.0 },));

        let mut count_with_vel = 0;
        let mut count_without_vel = 0;

        for (_pos, vel) in world.query::<(&Position, Option<&Velocity>)>() {
            if vel.is_some() {
                count_with_vel += 1;
            } else {
                count_without_vel += 1;
            }
        }

        assert_eq!(count_with_vel, 1);
        assert_eq!(count_without_vel, 1);
    }

    #[test]
    fn test_system() {
        let mut world = World::new();

        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
        world.spawn((Position { x: 5.0, y: 5.0 }, Velocity { x: -1.0, y: -1.0 }));

        let mut system = system::QuerySystem::<(&mut Position, &Velocity), _>::new(
            |(pos, vel): (&mut Position, &Velocity)| {
                pos.x += vel.x;
                pos.y += vel.y;
            },
        );
        system.run(&mut world);

        for pos in world.query::<&Position>() {
            assert!(pos.x == 1.0 || pos.x == 4.0);
        }
    }

    #[test]
    fn test_schedule() {
        let mut world = World::new();
        let mut schedule = Schedule::new();

        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

        schedule.add_update_system(system::QuerySystem::<(&mut Position, &Velocity), _>::new(
            |(pos, vel): (&mut Position, &Velocity)| {
                pos.x += vel.x;
                pos.y += vel.y;
            },
        ));
        schedule.run(&mut world);

        let pos = world.query::<&Position>().next().unwrap();
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 1.0);
    }

    #[test]
    fn test_change_detection() {
        let mut world = World::new();

        let entity = world.spawn((Position { x: 0.0, y: 0.0 },));

        world.tick();

        // Modify the component
        if let Some(pos) = world.get_mut::<Position>(entity) {
            pos.x = 10.0;
        }

        // Check if it was changed
        let location = world.entity_meta(entity).unwrap();
        let archetype = world.archetypes.get(location.archetype).unwrap();
        assert!(archetype.component_changed::<Position>(location.index, 0));
    }

    #[test]
    fn test_entity_info() {
        let mut world = World::new();

        let entity = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

        let info = world.entity_info(entity).unwrap();
        assert_eq!(info.entity, entity);
        assert_eq!(info.component_types.len(), 2);
    }

    #[test]
    fn test_error_handling() {
        let mut world = World::new();

        let entity = world.spawn((Position { x: 0.0, y: 0.0 },));
        world.despawn(entity);

        let result = world.try_get::<Position>(entity);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EcsError::EntityNotFound(_)));
    }

    #[test]
    fn test_parallel_schedule() {
        let mut world = World::new();
        let mut schedule = ParallelSchedule::new();

        world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
        world.spawn((Position { x: 5.0, y: 5.0 }, Velocity { x: -1.0, y: -1.0 }));

        schedule.add_system(system::QuerySystem::<(&mut Position, &Velocity), _>::new(
            |(pos, vel): (&mut Position, &Velocity)| {
                pos.x += vel.x;
                pos.y += vel.y;
            },
        ));
        schedule.run(&mut world);

        for pos in world.query::<&Position>() {
            assert!(pos.x == 1.0 || pos.x == 4.0);
        }
    }

    #[test]
    fn test_reserve() {
        let mut world = World::new();
        world.reserve(1000);

        for i in 0..1000 {
            world.spawn((Position {
                x: i as f32,
                y: 0.0,
            },));
        }

        let mut count = 0;
        for _pos in world.query::<&Position>() {
            count += 1;
        }
        assert_eq!(count, 1000);
    }

    #[test]
    fn test_insert_multiple_entities() {
        let mut world = World::new();

        let entities: Vec<_> = (0..100)
            .map(|i| {
                world.spawn((Position {
                    x: i as f32,
                    y: 0.0,
                },))
            })
            .collect();

        for &entity in &entities {
            world.insert(entity, Velocity { x: 1.0, y: 1.0 }).unwrap();
        }

        // Verify all entities have both components
        for &entity in &entities {
            assert!(world.get::<Position>(entity).is_some());
            assert!(world.get::<Velocity>(entity).is_some());
        }
    }
}
