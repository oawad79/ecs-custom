use crate::world::World;
use std::any::TypeId;

pub trait System: Send {
    fn run(&mut self, world: &mut World);
    fn reads(&self) -> &[TypeId];
    fn writes(&self) -> &[TypeId];
    fn name(&self) -> &str;
}

pub struct QuerySystem<Q, F> {
    func: F,
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
    name: String,
    _marker: std::marker::PhantomData<Q>,
}

impl<Q: crate::query::Query, F> QuerySystem<Q, F>
where
    F: FnMut(Q::Item<'_>) + Send,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            reads: Q::read_types(),
            writes: Q::write_types(),
            name: std::any::type_name::<F>().to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl<Q: crate::query::Query, F> System for QuerySystem<Q, F>
where
    F: FnMut(Q::Item<'_>) + Send,
{
    fn run(&mut self, world: &mut World) {
        for item in world.query::<Q>() {
            (self.func)(item);
        }
    }

    fn reads(&self) -> &[TypeId] {
        &self.reads
    }

    fn writes(&self) -> &[TypeId] {
        &self.writes
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct FunctionSystem<F> {
    func: F,
    name: String,
}

impl<F: FnMut(&mut World) + Send> System for FunctionSystem<F> {
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }

    fn reads(&self) -> &[TypeId] {
        &[]
    }

    fn writes(&self) -> &[TypeId] {
        &[]
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub trait IntoSystem<Marker> {
    type System: System;
    fn into_system(self) -> Self::System;
}

impl<F: FnMut(&mut World) + Send + 'static> IntoSystem<()> for F {
    type System = FunctionSystem<F>;

    fn into_system(self) -> Self::System {
        FunctionSystem {
            func: self,
            name: std::any::type_name::<F>().to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}

pub struct Schedule {
    stages: Vec<(Stage, Vec<Box<dyn System>>)>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            stages: vec![
                (Stage::PreUpdate, Vec::new()),
                (Stage::Update, Vec::new()),
                (Stage::PostUpdate, Vec::new()),
                (Stage::Render, Vec::new()),
            ],
        }
    }

    pub fn add_system(&mut self, stage: Stage, system: impl System + 'static) {
        for (s, systems) in &mut self.stages {
            if *s == stage {
                systems.push(Box::new(system));
                return;
            }
        }
    }

    pub fn add_update_system(&mut self, system: impl System + 'static) {
        self.add_system(Stage::Update, system);
    }

    pub fn run(&mut self, world: &mut World) {
        for (_stage, systems) in &mut self.stages {
            for system in systems {
                system.run(world);
            }
        }
        world.flush_commands();
        world.tick();
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ParallelSchedule {
    systems: Vec<Box<dyn System>>,
}

impl ParallelSchedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: impl System + 'static) {
        self.systems.push(Box::new(system));
    }

    pub fn run(&mut self, world: &mut World) {
        // Group systems by conflicts
        let mut batches: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; self.systems.len()];

        for i in 0..self.systems.len() {
            if assigned[i] {
                continue;
            }

            let mut batch = vec![i];
            assigned[i] = true;

            for j in (i + 1)..self.systems.len() {
                if assigned[j] {
                    continue;
                }

                let conflicts = batch
                    .iter()
                    .any(|&b| self.systems_conflict(&self.systems[b], &self.systems[j]));

                if !conflicts {
                    batch.push(j);
                    assigned[j] = true;
                }
            }

            batches.push(batch);
        }

        // Run each batch (systems in a batch could run in parallel)
        for batch in batches {
            for &system_index in &batch {
                self.systems[system_index].run(world);
            }
        }

        world.flush_commands();
        world.tick();
    }

    fn systems_conflict(&self, a: &Box<dyn System>, b: &Box<dyn System>) -> bool {
        let a_reads = a.reads();
        let a_writes = a.writes();
        let b_reads = b.reads();
        let b_writes = b.writes();

        // Write-
        // Write-write conflict
        for a_write in a_writes {
            if b_writes.contains(a_write) {
                return true;
            }
        }

        // Read-write conflict
        for a_read in a_reads {
            if b_writes.contains(a_read) {
                return true;
            }
        }

        for b_read in b_reads {
            if a_writes.contains(b_read) {
                return true;
            }
        }

        false
    }
}

impl Default for ParallelSchedule {
    fn default() -> Self {
        Self::new()
    }
}
