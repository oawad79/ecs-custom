use crate::entity::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

// Component is automatically implemented via the blanket impl

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Children(pub Vec<Entity>);

// Component is automatically implemented via the blanket impl

impl Children {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, entity: Entity) {
        self.0.push(entity);
    }

    pub fn remove(&mut self, entity: Entity) {
        self.0.retain(|&e| e != entity);
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}
