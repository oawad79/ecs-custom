use std::any::TypeId;

pub trait Component: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> Component for T {}

pub fn type_name<T: 'static>() -> &'static str {
    std::any::type_name::<T>()
}

pub trait Bundle: Send + Sync + 'static {
    fn type_ids() -> Vec<TypeId>;
    fn type_names() -> Vec<&'static str>;
    fn init_archetype(archetype: &mut crate::archetype::Archetype);
    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize);
}

// Implement Bundle for single components
impl<T: Component> Bundle for (T,) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn type_names() -> Vec<&'static str> {
        vec![type_name::<T>()]
    }

    fn init_archetype(archetype: &mut crate::archetype::Archetype) {
        archetype.add_column::<T>();
    }

    fn insert_into(self, archetype: &mut crate::archetype::Archetype, index: usize) {
        archetype.set_component(index, self.0);
    }
}

// Implement Bundle for tuples
impl<T1: Component, T2: Component> Bundle for (T1, T2) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T1>(), TypeId::of::<T2>()]
    }

    fn type_names() -> Vec<&'static str> {
        vec![type_name::<T1>(), type_name::<T2>()]
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

impl<T1: Component, T2: Component, T3: Component> Bundle for (T1, T2, T3) {
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T1>(), TypeId::of::<T2>(), TypeId::of::<T3>()]
    }

    fn type_names() -> Vec<&'static str> {
        vec![type_name::<T1>(), type_name::<T2>(), type_name::<T3>()]
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

impl<T1: Component, T2: Component, T3: Component, T4: Component> Bundle for (T1, T2, T3, T4) {
    fn type_ids() -> Vec<TypeId> {
        vec![
            TypeId::of::<T1>(),
            TypeId::of::<T2>(),
            TypeId::of::<T3>(),
            TypeId::of::<T4>(),
        ]
    }

    fn type_names() -> Vec<&'static str> {
        vec![
            type_name::<T1>(),
            type_name::<T2>(),
            type_name::<T3>(),
            type_name::<T4>(),
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
