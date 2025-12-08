use std::any::TypeId;

/// Trait for querying components from the world
pub trait Query: Send {
    type Item<'a>;

    fn matches_archetype(types: &[TypeId]) -> bool;
    unsafe fn fetch<'a>(
        archetype: &'a mut crate::archetype::Archetype,
        index: usize,
    ) -> Self::Item<'a>;

    /// Returns TypeIds of components this query reads
    fn read_types() -> Vec<TypeId> {
        Vec::new()
    }

    /// Returns TypeIds of components this query writes
    fn write_types() -> Vec<TypeId> {
        Vec::new()
    }
}

pub trait QueryBorrow {
    type Item<'a>;
}

// Implement Query for single component references
impl<T: 'static + Send + Sync> Query for &T {
    type Item<'a> = &'a T;

    fn matches_archetype(types: &[TypeId]) -> bool {
        types.contains(&TypeId::of::<T>())
    }

    unsafe fn fetch<'a>(
        archetype: &'a mut crate::archetype::Archetype,
        index: usize,
    ) -> Self::Item<'a> {
        unsafe { archetype.get_component::<T>(index).unwrap() }
    }

    fn read_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

impl<T: 'static + Send> Query for &mut T {
    type Item<'a> = &'a mut T;

    fn matches_archetype(types: &[TypeId]) -> bool {
        types.contains(&TypeId::of::<T>())
    }

    unsafe fn fetch<'a>(
        archetype: &'a mut crate::archetype::Archetype,
        index: usize,
    ) -> Self::Item<'a> {
        let ptr = archetype as *mut crate::archetype::Archetype;
        unsafe { (*ptr).get_component_mut::<T>(index).unwrap() }
    }

    fn write_types() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}

// Implement Query for tuples of queries
impl<Q1: Query, Q2: Query> Query for (Q1, Q2) {
    type Item<'a> = (Q1::Item<'a>, Q2::Item<'a>);

    fn matches_archetype(types: &[TypeId]) -> bool {
        Q1::matches_archetype(types) && Q2::matches_archetype(types)
    }

    unsafe fn fetch<'a>(
        archetype: &'a mut crate::archetype::Archetype,
        index: usize,
    ) -> Self::Item<'a> {
        let ptr = archetype as *mut crate::archetype::Archetype;
        unsafe { (Q1::fetch(&mut *ptr, index), Q2::fetch(&mut *ptr, index)) }
    }

    fn read_types() -> Vec<TypeId> {
        let mut types = Q1::read_types();
        types.extend(Q2::read_types());
        types
    }

    fn write_types() -> Vec<TypeId> {
        let mut types = Q1::write_types();
        types.extend(Q2::write_types());
        types
    }
}

impl<Q1: Query, Q2: Query, Q3: Query> Query for (Q1, Q2, Q3) {
    type Item<'a> = (Q1::Item<'a>, Q2::Item<'a>, Q3::Item<'a>);

    fn matches_archetype(types: &[TypeId]) -> bool {
        Q1::matches_archetype(types) && Q2::matches_archetype(types) && Q3::matches_archetype(types)
    }

    unsafe fn fetch<'a>(
        archetype: &'a mut crate::archetype::Archetype,
        index: usize,
    ) -> Self::Item<'a> {
        let ptr = archetype as *mut crate::archetype::Archetype;
        unsafe {
            (
                Q1::fetch(&mut *ptr, index),
                Q2::fetch(&mut *ptr, index),
                Q3::fetch(&mut *ptr, index),
            )
        }
    }

    fn read_types() -> Vec<TypeId> {
        let mut types = Q1::read_types();
        types.extend(Q2::read_types());
        types.extend(Q3::read_types());
        types
    }

    fn write_types() -> Vec<TypeId> {
        let mut types = Q1::write_types();
        types.extend(Q2::write_types());
        types.extend(Q3::write_types());
        types
    }
}

impl<Q1: Query, Q2: Query, Q3: Query, Q4: Query> Query for (Q1, Q2, Q3, Q4) {
    type Item<'a> = (Q1::Item<'a>, Q2::Item<'a>, Q3::Item<'a>, Q4::Item<'a>);

    fn matches_archetype(types: &[TypeId]) -> bool {
        Q1::matches_archetype(types)
            && Q2::matches_archetype(types)
            && Q3::matches_archetype(types)
            && Q4::matches_archetype(types)
    }

    unsafe fn fetch<'a>(
        archetype: &'a mut crate::archetype::Archetype,
        index: usize,
    ) -> Self::Item<'a> {
        let ptr = archetype as *mut crate::archetype::Archetype;
        unsafe {
            (
                Q1::fetch(&mut *ptr, index),
                Q2::fetch(&mut *ptr, index),
                Q3::fetch(&mut *ptr, index),
                Q4::fetch(&mut *ptr, index),
            )
        }
    }

    fn read_types() -> Vec<TypeId> {
        let mut types = Q1::read_types();
        types.extend(Q2::read_types());
        types.extend(Q3::read_types());
        types.extend(Q4::read_types());
        types
    }

    fn write_types() -> Vec<TypeId> {
        let mut types = Q1::write_types();
        types.extend(Q2::write_types());
        types.extend(Q3::write_types());
        types.extend(Q4::write_types());
        types
    }
}
