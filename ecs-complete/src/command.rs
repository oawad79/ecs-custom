use crate::component::{Bundle, Component};
use crate::entity::Entity;
use std::any::TypeId;

pub enum Command {
    Spawn(Box<dyn FnOnce(&mut crate::world::World) -> Entity + Send>),
    Despawn(Entity),
    Insert(
        Entity,
        Box<dyn FnOnce(&mut crate::world::World, Entity) + Send>,
    ),
    Remove(Entity, TypeId),
}

pub struct Commands {
    queue: Vec<Command>,
}

impl Commands {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        let index = self.queue.len();
        self.queue
            .push(Command::Spawn(Box::new(move |world| world.spawn(bundle))));
        EntityCommands {
            commands: self,
            index,
        }
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.queue.push(Command::Despawn(entity));
    }

    pub fn entity(&mut self, entity: Entity) -> EntityCommands {
        EntityCommands {
            commands: self,
            index: usize::MAX, // Existing entity
        }
    }

    pub fn insert<C: Component>(&mut self, entity: Entity, component: C) {
        self.queue.push(Command::Insert(
            entity,
            Box::new(move |world, entity| {
                world.insert(entity, component).ok();
            }),
        ));
    }

    pub fn remove<C: Component>(&mut self, entity: Entity) {
        self.queue.push(Command::Remove(entity, TypeId::of::<C>()));
    }

    pub(crate) fn apply(&mut self, world: &mut crate::world::World) {
        for command in self.queue.drain(..) {
            match command {
                Command::Spawn(f) => {
                    f(world);
                }
                Command::Despawn(entity) => {
                    world.despawn(entity);
                }
                Command::Insert(entity, f) => {
                    f(world, entity);
                }
                Command::Remove(entity, type_id) => {
                    world.remove_by_id(entity, type_id);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EntityCommands<'a> {
    commands: &'a mut Commands,
    index: usize,
}

impl<'a> EntityCommands<'a> {
    pub fn insert<C: Component>(self, component: C) -> Self {
        // This is simplified - in a real implementation, we'd track the entity
        self
    }

    pub fn remove<C: Component>(self) -> Self {
        self
    }
}
