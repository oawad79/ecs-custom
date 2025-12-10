use crate::archetype::ArchetypeMap;
use crate::command::Commands;
use crate::component::{Bundle, Component, type_name};
use crate::entity::{Entity, EntityInfo, EntityMeta};
use crate::error::{EcsError, Result};
use crate::query::Query;
use crate::resource::Resources;
use slotmap::SlotMap;
use std::any::TypeId;

pub struct World {
    entities: SlotMap<Entity, EntityLocation>,
    pub(crate) archetypes: ArchetypeMap,
    resources: Resources,
    commands: Commands,
    tick: u64,
}

#[derive(Clone, Copy)]
struct EntityLocation {
    archetype: usize,
    index: usize,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: SlotMap::with_key(),
            archetypes: ArchetypeMap::new(),
            resources: Resources::new(),
            commands: Commands::new(),
            tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        for archetype in self.archetypes.iter_mut() {
            archetype.set_tick(self.tick);
        }
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        let type_ids = B::type_ids();
        let type_names = B::type_names();

        let archetype_index = self.archetypes.get_or_create(type_ids, type_names);
        let archetype = self.archetypes.get_mut(archetype_index).unwrap();

        if archetype.is_empty() {
            B::init_archetype(archetype);
        }

        let entity_index = archetype.len();

        let entity = self.entities.insert(EntityLocation {
            archetype: archetype_index,
            index: entity_index,
        });

        archetype.push_entity(entity);
        bundle.insert_into(archetype, entity_index);

        entity
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entities.remove(entity) {
            let archetype = self.archetypes.get_mut(location.archetype).unwrap();
            let swapped_entity = archetype.remove_entity(location.index);

            if swapped_entity != entity {
                if let Some(swapped_location) = self.entities.get_mut(swapped_entity) {
                    swapped_location.index = location.index;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains_key(entity)
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let location = self.entities.get(entity)?;
        let archetype = self.archetypes.get(location.archetype)?;
        archetype.get_component::<T>(location.index)
    }

    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let location = self.entities.get(entity)?;
        let archetype = self.archetypes.get_mut(location.archetype)?;
        archetype.get_component_mut::<T>(location.index)
    }

    pub fn try_get<T: Component>(&self, entity: Entity) -> Result<&T> {
        self.get(entity).ok_or(EcsError::EntityNotFound(entity))
    }

    pub fn try_get_mut<T: Component>(&mut self, entity: Entity) -> Result<&mut T> {
        if !self.is_alive(entity) {
            return Err(EcsError::EntityNotFound(entity));
        }
        self.get_mut(entity)
            .ok_or(EcsError::ComponentNotFound(TypeId::of::<T>()))
    }

    pub fn insert<C: Component>(&mut self, entity: Entity, component: C) -> Result<()> {
        let location = self
            .entities
            .get(entity)
            .ok_or(EcsError::EntityNotFound(entity))?;

        let from_archetype = location.archetype;
        let component_type = TypeId::of::<C>();

        // Check if component already exists
        let from_arch = self.archetypes.get(from_archetype).unwrap();
        if from_arch.types().contains(&component_type) {
            // Just update the component
            let archetype = self.archetypes.get_mut(from_archetype).unwrap();
            archetype.set_component(location.index, component);
            return Ok(());
        }

        // Find or create target archetype
        let to_archetype = if let Some(to) = self
            .archetypes
            .find_archetype_with_added(from_archetype, component_type)
        {
            to
        } else {
            let to = self.archetypes.create_archetype_with_added(
                from_archetype,
                component_type,
                type_name::<C>(),
            );

            // Initialize columns in the new archetype
            let (from_arch, to_arch) = self.archetypes.get_pair_mut(from_archetype, to).unwrap();

            // Copy column structure from source
            for col in 0..from_arch.columns.len() {
                if to_arch.columns.len() <= col {
                    let item_size = from_arch.columns[col].item_size;
                    let drop_fn = from_arch.columns[col].drop_fn;
                    to_arch.add_column_raw(item_size, drop_fn);
                }
            }

            // Add column for the new component
            to_arch.add_column::<C>();

            to
        };

        // Move entity to new archetype
        self.move_entity_with_component(entity, from_archetype, to_archetype, component)?;

        Ok(())
    }

    pub fn remove<C: Component>(&mut self, entity: Entity) -> Result<C> {
        let location = self
            .entities
            .get(entity)
            .ok_or(EcsError::EntityNotFound(entity))?;

        let from_archetype = location.archetype;
        let component_type = TypeId::of::<C>();

        // Take the component before moving
        let component = {
            let archetype = self.archetypes.get_mut(from_archetype).unwrap();
            archetype
                .take_component::<C>(location.index)
                .ok_or(EcsError::ComponentNotFound(component_type))?
        };

        // Find or create target archetype
        let to_archetype = if let Some(to) = self
            .archetypes
            .find_archetype_with_removed(from_archetype, component_type)
        {
            to
        } else {
            let to = self
                .archetypes
                .create_archetype_with_removed(from_archetype, component_type);

            // Initialize columns in the new archetype if it's empty
            let (from_arch, to_arch) = self.archetypes.get_pair_mut(from_archetype, to).unwrap();

            if to_arch.columns.is_empty() {
                // Copy column structure from source for all components except the removed one
                for (col_idx, &type_id) in from_arch.types().iter().enumerate() {
                    if type_id != component_type {
                        let item_size = from_arch.columns[col_idx].item_size;
                        let drop_fn = from_arch.columns[col_idx].drop_fn;
                        to_arch.add_column_raw(item_size, drop_fn);
                    }
                }
            }

            to
        };

        // Move entity to new archetype
        self.move_entity(entity, from_archetype, to_archetype)?;

        Ok(component)
    }

    pub(crate) fn remove_by_id(&mut self, entity: Entity, type_id: TypeId) {
        // Simplified version for commands
        if let Some(location) = self.entities.get(entity) {
            let from_archetype = location.archetype;
            if let Some(to_archetype) = self
                .archetypes
                .find_archetype_with_removed(from_archetype, type_id)
            {
                let _ = self.move_entity(entity, from_archetype, to_archetype);
            }
        }
    }

    fn move_entity_with_component<C: Component>(
        &mut self,
        entity: Entity,
        from_archetype: usize,
        to_archetype: usize,
        new_component: C,
    ) -> Result<()> {
        let from_index = self
            .entities
            .get(entity)
            .ok_or(EcsError::EntityNotFound(entity))?
            .index;

        // Get the types from source archetype
        let from_types: Vec<TypeId> = self
            .archetypes
            .get(from_archetype)
            .ok_or(EcsError::ArchetypeNotFound(from_archetype))?
            .types()
            .to_vec();

        let to_index;
        let swapped_entity;

        // Scope for mutable borrow of archetypes
        {
            let (from_arch, to_arch) = self
                .archetypes
                .get_pair_mut(from_archetype, to_archetype)
                .ok_or(EcsError::ArchetypeNotFound(to_archetype))?;

            to_index = to_arch.len();

            // Push entity to target archetype first
            to_arch.push_entity(entity);

            // Copy all matching components from source to destination
            for &type_id in &from_types {
                to_arch.copy_component_from(to_index, from_arch, from_index, type_id);
            }

            // Add the new component
            to_arch.set_component(to_index, new_component);

            // Remove entity from source archetype
            swapped_entity = from_arch.remove_entity(from_index);
        }

        // Update entity location
        if let Some(loc) = self.entities.get_mut(entity) {
            loc.archetype = to_archetype;
            loc.index = to_index;
        }

        // Update swapped entity location if needed
        if swapped_entity != entity {
            if let Some(swapped_location) = self.entities.get_mut(swapped_entity) {
                swapped_location.index = from_index;
            }
        }

        Ok(())
    }

    fn move_entity(
        &mut self,
        entity: Entity,
        from_archetype: usize,
        to_archetype: usize,
    ) -> Result<()> {
        let from_index = self
            .entities
            .get(entity)
            .ok_or(EcsError::EntityNotFound(entity))?
            .index;

        // Get the types from target archetype (which is a subset of source)
        let to_types: Vec<TypeId> = self
            .archetypes
            .get(to_archetype)
            .ok_or(EcsError::ArchetypeNotFound(to_archetype))?
            .types()
            .to_vec();

        let to_index;
        let swapped_entity;

        // Scope for mutable borrow of archetypes
        {
            let (from_arch, to_arch) = self
                .archetypes
                .get_pair_mut(from_archetype, to_archetype)
                .ok_or(EcsError::ArchetypeNotFound(to_archetype))?;

            to_index = to_arch.len();

            // Push entity to target archetype first
            to_arch.push_entity(entity);

            // Copy all components that exist in target archetype
            for &type_id in &to_types {
                to_arch.copy_component_from(to_index, from_arch, from_index, type_id);
            }

            // Remove entity from source archetype
            swapped_entity = from_arch.remove_entity(from_index);
        }

        // Update entity location
        if let Some(loc) = self.entities.get_mut(entity) {
            loc.archetype = to_archetype;
            loc.index = to_index;
        }

        // Update swapped entity location if needed
        if swapped_entity != entity {
            if let Some(swapped_location) = self.entities.get_mut(swapped_entity) {
                swapped_location.index = from_index;
            }
        }

        Ok(())
    }

    pub fn query<Q: Query>(&mut self) -> QueryIter<Q> {
        QueryIter {
            archetypes: &mut self.archetypes,
            archetype_index: 0,
            entity_index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn entity_info(&self, entity: Entity) -> Option<EntityInfo> {
        let location = self.entities.get(entity)?;
        let archetype = self.archetypes.get(location.archetype)?;

        Some(EntityInfo {
            entity,
            archetype_id: archetype.id(),
            component_types: archetype.type_names().to_vec(),
        })
    }

    pub fn entity_meta(&self, entity: Entity) -> Option<EntityMeta> {
        let location = self.entities.get(entity)?;
        Some(EntityMeta {
            generation: 0, // SlotMap handles generations internally
            archetype: location.archetype,
            index: location.index,
        })
    }

    pub fn insert_resource<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    pub fn get_resource<T: 'static>(&self) -> Option<crate::resource::Res<T>> {
        self.resources.get()
    }

    pub fn get_resource_mut<T: 'static>(&self) -> Option<crate::resource::ResMut<T>> {
        self.resources.get_mut()
    }

    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resources.remove()
    }

    pub fn commands(&mut self) -> &mut Commands {
        &mut self.commands
    }

    pub fn flush_commands(&mut self) {
        let mut commands = std::mem::replace(&mut self.commands, Commands::new());
        commands.apply(self);
        self.commands = commands;
    }

    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

pub struct QueryIter<'a, Q: Query> {
    archetypes: &'a mut ArchetypeMap,
    archetype_index: usize,
    entity_index: usize,
    _marker: std::marker::PhantomData<Q>,
}

impl<'a, Q: Query> Iterator for QueryIter<'a, Q> {
    type Item = Q::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let archetypes_ptr = self.archetypes as *mut ArchetypeMap;

        loop {
            let archetype_count = unsafe { (*archetypes_ptr).iter().count() };

            if self.archetype_index >= archetype_count {
                return None;
            }

            let archetype = unsafe {
                (*archetypes_ptr)
                    .iter_mut()
                    .nth(self.archetype_index)
                    .unwrap()
            };

            if !Q::matches_archetype(archetype.types()) {
                self.archetype_index += 1;
                self.entity_index = 0;
                continue;
            }

            if self.entity_index >= archetype.len() {
                self.archetype_index += 1;
                self.entity_index = 0;
                continue;
            }

            let item = unsafe { Q::fetch(archetype, self.entity_index) };
            self.entity_index += 1;

            return Some(unsafe { std::mem::transmute(item) });
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining: usize = unsafe {
            let archetypes_ptr = self.archetypes as *const ArchetypeMap;
            (*archetypes_ptr)
                .iter()
                .skip(self.archetype_index)
                .filter(|a| Q::matches_archetype(a.types()))
                .map(|a| a.len())
                .sum()
        };
        (remaining, Some(remaining))
    }
}
