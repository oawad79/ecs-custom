use crate::archetype::ArchetypeMap;
use crate::entity::Entity;
use crate::query::Query;
use slotmap::SlotMap;
use std::any::TypeId;

/// The main ECS container
pub struct World {
    entities: SlotMap<Entity, EntityLocation>,
    archetypes: ArchetypeMap,
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
        }
    }

    /// Spawn a new entity with components
    pub fn spawn<T: ComponentBundle>(&mut self, components: T) -> Entity {
        let type_ids = T::type_ids();

        let archetype_index = self.archetypes.get_or_create(type_ids);
        let archetype = self.archetypes.get_mut(archetype_index).unwrap();

        // Initialize columns if needed
        if archetype.len() == 0 {
            T::init_archetype(archetype);
        }

        let entity_index = archetype.len();

        // Create entity first (slotmap generates the key)
        let entity = self.entities.insert(EntityLocation {
            archetype: archetype_index,
            index: entity_index,
        });

        // Now add to archetype with the generated entity
        archetype.push_entity(entity);
        components.insert_into(archetype, entity_index);

        entity
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entities.remove(entity) {
            let archetype = self.archetypes.get_mut(location.archetype).unwrap();
            let swapped_entity = archetype.remove_entity(location.index);

            // Update the swapped entity's location
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

    /// Check if an entity is alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.contains_key(entity)
    }

    /// Get a component from an entity
    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let location = self.entities.get(entity)?;
        let archetype = self.archetypes.get(location.archetype)?;
        archetype.get_component::<T>(location.index)
    }

    /// Get a mutable component from an entity
    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let location = self.entities.get(entity)?;
        let archetype = self.archetypes.get_mut(location.archetype)?;
        archetype.get_component_mut::<T>(location.index)
    }

    /// Query the world for entities with specific components
    pub fn query<Q: Query>(&mut self) -> QueryIter<Q> {
        QueryIter {
            archetypes: &mut self.archetypes,
            archetype_index: 0,
            entity_index: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for types that can be inserted as a bundle of components
pub trait ComponentBundle {
    fn type_ids() -> Vec<TypeId>;
    fn init_archetype(archetype: &mut crate::archetype::Archetype);
    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize);
}

// Implement ComponentBundle for single components
impl<T: 'static> ComponentBundle for (T,) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn init_archetype(archetype: &mut crate::archetype::Archetype) {
        archetype.add_column::<T>();
    }

    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize) {
        archetype.set_component(index, self.0);
    }
}

// Implement ComponentBundle for tuples of 2-4 components
impl<T1: 'static, T2: 'static> ComponentBundle for (T1, T2) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T1>(), TypeId::of::<T2>()]
    }

    fn init_archetype(archetype: &mut crate::archetype::Archetype) {
        archetype.add_column::<T1>();
        archetype.add_column::<T2>();
    }

    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize) {
        archetype.set_component(index, self.0);
        archetype.set_component(index, self.1);
    }
}

impl<T1: 'static, T2: 'static, T3: 'static> ComponentBundle for (T1, T2, T3) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T1>(), TypeId::of::<T2>(), TypeId::of::<T3>()]
    }

    fn init_archetype(archetype: &mut crate::archetype::Archetype) {
        archetype.add_column::<T1>();
        archetype.add_column::<T2>();
        archetype.add_column::<T3>();
    }

    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize) {
        archetype.set_component(index, self.0);
        archetype.set_component(index, self.1);
        archetype.set_component(index, self.2);
    }
}

impl<T1: 'static, T2: 'static, T3: 'static, T4: 'static> ComponentBundle for (T1, T2, T3, T4) {
    fn type_ids() -> Vec<TypeId> {
        vec![
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
        ]
    }

    fn init_archetype(archetype: &mut crate::archetype::Archetype) {
        archetype.add_column::<T1>();
        archetype.add_column::<T2>();
        archetype.add_column::<T3>();
        archetype.add_column::<T4>();
    }

    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize) {
        archetype.set_component(index, self.0);
        archetype.set_component(index, self.1);
        archetype.set_component(index, self.2);
        archetype.set_component(index, self.3);
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
}
