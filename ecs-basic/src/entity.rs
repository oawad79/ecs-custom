use std::fmt;

/// A unique identifier for an entity
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) generation: u32,
}

impl Entity {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.id, self.generation)
    }
}

/// Manages entity allocation and recycling
pub(crate) struct EntityAllocator {
    generations: Vec<u32>,
    free_list: Vec<u32>,
    next_id: u32,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            next_id: 0,
        }
    }

    pub fn allocate(&mut self) -> Entity {
        if let Some(id) = self.free_list.pop() {
            let generation = self.generations[id as usize];
            Entity { id, generation }
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.generations.push(0);
            Entity { id, generation: 0 }
        }
    }

    pub fn free(&mut self, entity: Entity) -> bool {
        if entity.id as usize >= self.generations.len() {
            return false;
        }

        if self.generations[entity.id as usize] != entity.generation {
            return false;
        }

        self.generations[entity.id as usize] += 1;
        self.free_list.push(entity.id);
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        (entity.id as usize) < self.generations.len()
            && self.generations[entity.id as usize] == entity.generation
    }
}
