use slotmap::new_key_type;

new_key_type! {
    pub struct Entity;
}

#[derive(Debug, Clone, Copy)]
pub struct EntityMeta {
    pub generation: u32,
    pub archetype: usize,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub entity: Entity,
    pub archetype_id: usize,
    pub component_types: Vec<&'static str>,
}
