use crate::entity::Entity;
use std::any::TypeId;
use std::collections::HashMap;
use std::ptr::NonNull;

pub(crate) struct Archetype {
    id: usize,
    types: Vec<TypeId>,
    type_names: Vec<&'static str>,
    pub(crate) columns: Vec<Column>,
    entities: Vec<Entity>,
    tick: u64,
}

pub(crate) struct Column {
    pub(crate) data: NonNull<u8>,
    pub(crate) len: usize,
    pub(crate) capacity: usize,
    pub(crate) item_size: usize,
    pub(crate) changed_ticks: Vec<u64>,
    pub(crate) drop_fn: unsafe fn(*mut u8),
}

impl Archetype {
    pub fn new(id: usize, types: Vec<TypeId>, type_names: Vec<&'static str>) -> Self {
        Self {
            id,
            types,
            type_names,
            columns: Vec::new(),
            entities: Vec::new(),
            tick: 0,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn types(&self) -> &[TypeId] {
        &self.types
    }

    pub fn type_names(&self) -> &[&'static str] {
        &self.type_names
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    pub fn add_column<T: 'static>(&mut self) {
        let column = Column {
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            item_size: std::mem::size_of::<T>(),
            changed_ticks: Vec::new(),
            drop_fn: |ptr| unsafe {
                std::ptr::drop_in_place(ptr as *mut T);
            },
        };
        self.columns.push(column);
    }

    pub fn add_column_raw(&mut self, item_size: usize, drop_fn: unsafe fn(*mut u8)) {
        let column = Column {
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            item_size,
            changed_ticks: Vec::new(),
            drop_fn,
        };
        self.columns.push(column);
    }

    pub fn push_entity(&mut self, entity: Entity) {
        self.entities.push(entity);

        for column in &mut self.columns {
            column.len += 1;
            column.changed_ticks.push(self.tick);
            if column.len > column.capacity {
                column.grow();
            }
        }
    }

    pub fn set_component<T: 'static>(&mut self, index: usize, component: T) {
        let type_id = TypeId::of::<T>();
        let column_index = self
            .types
            .iter()
            .position(|&t| t == type_id)
            .expect("Component type not in archetype");

        unsafe {
            let column = &mut self.columns[column_index];
            let ptr = column.data.as_ptr().add(index * column.item_size) as *mut T;
            std::ptr::write(ptr, component);
            column.changed_ticks[index] = self.tick;
        }
    }

    pub fn get_component<T: 'static>(&self, index: usize) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let column_index = self.types.iter().position(|&t| t == type_id)?;

        unsafe {
            let column = &self.columns[column_index];
            if index >= column.len {
                return None;
            }
            let ptr = column.data.as_ptr().add(index * column.item_size) as *const T;
            Some(&*ptr)
        }
    }

    pub fn get_component_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let column_index = self.types.iter().position(|&t| t == type_id)?;

        unsafe {
            let column = &mut self.columns[column_index];
            if index >= column.len {
                return None;
            }
            let ptr = column.data.as_ptr().add(index * column.item_size) as *mut T;
            column.changed_ticks[index] = self.tick;
            Some(&mut *ptr)
        }
    }

    pub fn component_changed<T: 'static>(&self, index: usize, since_tick: u64) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(column_index) = self.types.iter().position(|&t| t == type_id) {
            let column = &self.columns[column_index];
            if index < column.changed_ticks.len() {
                return column.changed_ticks[index] > since_tick;
            }
        }
        false
    }

    pub fn remove_entity(&mut self, index: usize) -> Entity {
        let entity = self.entities.swap_remove(index);

        for column in &mut self.columns {
            unsafe {
                let last = column.len - 1;
                if index != last {
                    let src = column.data.as_ptr().add(last * column.item_size);
                    let dst = column.data.as_ptr().add(index * column.item_size);
                    std::ptr::copy_nonoverlapping(src, dst, column.item_size);
                    column.changed_ticks[index] = column.changed_ticks[last];
                }
                column.len -= 1;
                column.changed_ticks.pop();
            }
        }

        entity
    }

    pub fn take_component<T: 'static>(&mut self, index: usize) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let column_index = self.types.iter().position(|&t| t == type_id)?;

        unsafe {
            let column = &mut self.columns[column_index];
            if index >= column.len {
                return None;
            }
            let ptr = column.data.as_ptr().add(index * column.item_size) as *mut T;
            Some(std::ptr::read(ptr))
        }
    }

    pub fn copy_component_from(
        &mut self,
        to_index: usize,
        from_archetype: &Archetype,
        from_index: usize,
        type_id: TypeId,
    ) {
        // Find column indices in both archetypes
        let to_column_index = self.types.iter().position(|&t| t == type_id);
        let from_column_index = from_archetype.types.iter().position(|&t| t == type_id);

        if let (Some(to_col_idx), Some(from_col_idx)) = (to_column_index, from_column_index) {
            unsafe {
                let to_column = &mut self.columns[to_col_idx];
                let from_column = &from_archetype.columns[from_col_idx];

                // Ensure we have enough space and the indices are valid
                if to_index < to_column.len && from_index < from_column.len {
                    let src = from_column
                        .data
                        .as_ptr()
                        .add(from_index * from_column.item_size);
                    let dst = to_column.data.as_ptr().add(to_index * to_column.item_size);

                    std::ptr::copy_nonoverlapping(src, dst, to_column.item_size);

                    // Update the changed tick - the tick was already added by push_entity
                    // so we just need to update it
                    to_column.changed_ticks[to_index] = from_column.changed_ticks[from_index];
                }
            }
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        for column in &mut self.columns {
            column.reserve(additional);
        }
        self.entities.reserve(additional);
    }
}

impl Column {
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity * 2
        };
        self.reserve(new_capacity - self.capacity);
    }

    fn reserve(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }

        let new_capacity = self.capacity + additional;

        unsafe {
            let new_layout = std::alloc::Layout::from_size_align_unchecked(
                new_capacity * self.item_size,
                std::mem::align_of::<u8>(),
            );

            let new_ptr = if self.capacity == 0 {
                std::alloc::alloc(new_layout)
            } else {
                let old_layout = std::alloc::Layout::from_size_align_unchecked(
                    self.capacity * self.item_size,
                    std::mem::align_of::<u8>(),
                );
                std::alloc::realloc(
                    self.data.as_ptr(),
                    old_layout,
                    new_capacity * self.item_size,
                )
            };

            self.data = NonNull::new(new_ptr).expect("Allocation failed");
            self.capacity = new_capacity;
        }

        self.changed_ticks.reserve(additional);
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        if self.capacity > 0 {
            unsafe {
                for i in 0..self.len {
                    let ptr = self.data.as_ptr().add(i * self.item_size);
                    (self.drop_fn)(ptr);
                }

                let layout = std::alloc::Layout::from_size_align_unchecked(
                    self.capacity * self.item_size,
                    std::mem::align_of::<u8>(),
                );
                std::alloc::dealloc(self.data.as_ptr(), layout);
            }
        }
    }
}

pub(crate) struct ArchetypeMap {
    archetypes: Vec<Archetype>,
    type_map: HashMap<Vec<TypeId>, usize>,
    graph: ArchetypeGraph,
}

impl ArchetypeMap {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            type_map: HashMap::new(),
            graph: ArchetypeGraph::new(),
        }
    }

    pub fn get_or_create(
        &mut self,
        mut types: Vec<TypeId>,
        type_names: Vec<&'static str>,
    ) -> usize {
        types.sort_unstable();

        if let Some(&index) = self.type_map.get(&types) {
            return index;
        }

        let index = self.archetypes.len();
        self.archetypes
            .push(Archetype::new(index, types.clone(), type_names));
        self.type_map.insert(types, index);
        index
    }

    pub fn get(&self, index: usize) -> Option<&Archetype> {
        self.archetypes.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Archetype> {
        self.archetypes.get_mut(index)
    }

    pub fn get_pair_mut(
        &mut self,
        index1: usize,
        index2: usize,
    ) -> Option<(&mut Archetype, &mut Archetype)> {
        if index1 == index2 {
            return None;
        }

        let (first, second) = if index1 < index2 {
            let (left, right) = self.archetypes.split_at_mut(index2);
            (&mut left[index1], &mut right[0])
        } else {
            let (left, right) = self.archetypes.split_at_mut(index1);
            (&mut right[0], &mut left[index2])
        };

        Some((first, second))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.iter_mut()
    }

    pub fn find_archetype_with_added(&mut self, from: usize, add: TypeId) -> Option<usize> {
        self.graph.get_edge(from, add, true)
    }

    pub fn find_archetype_with_removed(&mut self, from: usize, remove: TypeId) -> Option<usize> {
        self.graph.get_edge(from, remove, false)
    }

    pub fn create_archetype_with_added(
        &mut self,
        from: usize,
        add: TypeId,
        add_name: &'static str,
    ) -> usize {
        let from_arch = &self.archetypes[from];
        let mut new_types = from_arch.types.clone();
        let mut new_names = from_arch.type_names.clone();

        new_types.push(add);
        new_names.push(add_name);

        let to = self.get_or_create(new_types, new_names);
        self.graph.add_edge(from, to, add, true);
        to
    }

    pub fn create_archetype_with_removed(&mut self, from: usize, remove: TypeId) -> usize {
        let from_arch = &self.archetypes[from];
        let mut new_types = from_arch.types.clone();
        let mut new_names = from_arch.type_names.clone();

        if let Some(pos) = new_types.iter().position(|&t| t == remove) {
            new_types.remove(pos);
            new_names.remove(pos);
        }

        let to = self.get_or_create(new_types, new_names);
        self.graph.add_edge(from, to, remove, false);
        to
    }
}

struct ArchetypeGraph {
    edges: HashMap<(usize, TypeId, bool), usize>,
}

impl ArchetypeGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, component: TypeId, is_add: bool) {
        self.edges.insert((from, component, is_add), to);
    }

    fn get_edge(&self, from: usize, component: TypeId, is_add: bool) -> Option<usize> {
        self.edges.get(&(from, component, is_add)).copied()
    }
}
