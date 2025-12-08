use std::any::TypeId;
use std::collections::HashMap;
use std::ptr::NonNull;

/// Stores component data for entities with the same component types
pub(crate) struct Archetype {
    types: Vec<TypeId>,
    columns: Vec<Column>,
    entities: Vec<crate::Entity>,
}

struct Column {
    data: NonNull<u8>,
    len: usize,
    capacity: usize,
    item_size: usize,
    drop_fn: unsafe fn(*mut u8),
}

impl Archetype {
    pub fn new(types: Vec<TypeId>) -> Self {
        Self {
            types,
            columns: Vec::new(),
            entities: Vec::new(),
        }
    }

    pub fn types(&self) -> &[TypeId] {
        &self.types
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn entities(&self) -> &[crate::Entity] {
        &self.entities
    }

    pub fn add_column<T: 'static>(&mut self) {
        let column = Column {
            data: NonNull::dangling(),
            len: 0,
            capacity: 0,
            item_size: std::mem::size_of::<T>(),
            drop_fn: |ptr| unsafe {
                std::ptr::drop_in_place(ptr as *mut T);
            },
        };
        self.columns.push(column);
    }

    pub fn push_entity(&mut self, entity: crate::Entity) -> usize {
        let index = self.entities.len();
        self.entities.push(entity);

        for column in &mut self.columns {
            column.len += 1;
            if column.len > column.capacity {
                column.grow();
            }
        }

        index
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
            Some(&mut *ptr)
        }
    }

    pub fn remove_entity(&mut self, index: usize) -> crate::Entity {
        let entity = self.entities.swap_remove(index);

        for column in &mut self.columns {
            unsafe {
                let last = column.len - 1;
                if index != last {
                    let src = column.data.as_ptr().add(last * column.item_size);
                    let dst = column.data.as_ptr().add(index * column.item_size);
                    std::ptr::copy_nonoverlapping(src, dst, column.item_size);
                }
                column.len -= 1;
            }
        }

        entity
    }
}

impl Column {
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            4
        } else {
            self.capacity * 2
        };

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
}

impl ArchetypeMap {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            type_map: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, mut types: Vec<TypeId>) -> usize {
        types.sort_unstable();

        if let Some(&index) = self.type_map.get(&types) {
            return index;
        }

        let index = self.archetypes.len();
        self.archetypes.push(Archetype::new(types.clone()));
        self.type_map.insert(types, index);
        index
    }

    pub fn get(&self, index: usize) -> Option<&Archetype> {
        self.archetypes.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Archetype> {
        self.archetypes.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.iter_mut()
    }
}
