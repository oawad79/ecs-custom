use parking_lot::RwLock;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Resources {
    data: HashMap<TypeId, Arc<RwLock<Box<dyn Any + Send + Sync>>>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.data
            .insert(TypeId::of::<T>(), Arc::new(RwLock::new(Box::new(resource))));
    }

    pub fn get<T: 'static>(&self) -> Option<Res<T>> {
        self.data.get(&TypeId::of::<T>()).map(|r| Res {
            inner: r.clone(),
            _marker: std::marker::PhantomData,
        })
    }

    pub fn get_mut<T: 'static>(&self) -> Option<ResMut<T>> {
        self.data.get(&TypeId::of::<T>()).map(|r| ResMut {
            inner: r.clone(),
            _marker: std::marker::PhantomData,
        })
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.data.remove(&TypeId::of::<T>()).and_then(|r| {
            Arc::try_unwrap(r)
                .ok()
                .and_then(|lock| lock.into_inner().downcast::<T>().ok())
                .map(|boxed| *boxed)
        })
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Res<'a, T: 'static> {
    inner: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: 'static> std::ops::Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let guard = self.inner.read();
            let ptr = &**guard as *const (dyn Any + Send + Sync) as *const T;
            &*ptr
        }
    }
}

pub struct ResMut<'a, T: 'static> {
    inner: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T: 'static> std::ops::Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let guard = self.inner.read();
            let ptr = &**guard as *const (dyn Any + Send + Sync) as *const T;
            &*ptr
        }
    }
}

impl<'a, T: 'static> std::ops::DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let mut guard = self.inner.write();
            let ptr = &mut **guard as *mut (dyn Any + Send + Sync) as *mut T;
            &mut *ptr
        }
    }
}
