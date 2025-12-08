use crate::query::Query;
use crate::world::World;

/// Trait for systems that can run on the world
pub trait System {
    fn run(&mut self, world: &mut World);
}

/// Trait for converting functions into systems
pub trait IntoSystem<Params> {
    type System: System;
    fn into_system(self) -> Self::System;
}

/// A system that runs a function with query access
pub struct FunctionSystem<F> {
    func: F,
}

impl<F> System for FunctionSystem<F>
where
    F: FnMut(&mut World),
{
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }
}

impl<F> IntoSystem<()> for F
where
    F: FnMut(&mut World),
{
    type System = FunctionSystem<F>;

    fn into_system(self) -> Self::System {
        FunctionSystem { func: self }
    }
}

/// System that operates on a query
pub struct QuerySystem<Q, F>
where
    Q: Query,
{
    func: F,
    _marker: std::marker::PhantomData<Q>,
}

impl<Q, F> QuerySystem<Q, F>
where
    Q: Query,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Q, F> System for QuerySystem<Q, F>
where
    Q: Query,
    F: FnMut(Q::Item<'_>),
{
    fn run(&mut self, world: &mut World) {
        for item in world.query::<Q>() {
            (self.func)(item);
        }
    }
}

/// Schedule for running systems in order
pub struct Schedule {
    systems: Vec<Box<dyn System>>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a system to the schedule
    pub fn add_system<S: System + 'static>(&mut self, system: S) -> &mut Self {
        self.systems.push(Box::new(system));
        self
    }

    /// Run all systems in order
    pub fn run(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro to create query systems more easily
#[macro_export]
macro_rules! query_system {
    ($func:expr) => {
        $crate::system::QuerySystem::new($func)
    };
}
