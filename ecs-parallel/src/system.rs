use crate::query::Query;
use crate::world::World;
use rayon::prelude::*;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// Trait for systems that can run on the world
pub trait System: Send {
    fn run(&mut self, world: &mut World);

    /// Returns the component types this system reads
    fn reads(&self) -> Vec<TypeId> {
        Vec::new()
    }

    /// Returns the component types this system writes
    fn writes(&self) -> Vec<TypeId> {
        Vec::new()
    }

    /// Returns a name for debugging
    fn name(&self) -> &str {
        "unnamed_system"
    }
}

/// Trait for converting functions into systems
pub trait IntoSystem<Params> {
    type System: System;
    fn into_system(self) -> Self::System;
}

/// A system that runs a function with query access
pub struct FunctionSystem<F> {
    func: F,
    name: String,
}

impl<F> FunctionSystem<F> {
    pub fn new(func: F) -> Self {
        Self {
            func,
            name: "function_system".to_string(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl<F> System for FunctionSystem<F>
where
    F: FnMut(&mut World) + Send,
{
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl<F> IntoSystem<()> for F
where
    F: FnMut(&mut World) + Send,
{
    type System = FunctionSystem<F>;

    fn into_system(self) -> Self::System {
        FunctionSystem::new(self)
    }
}

/// System that operates on a query
pub struct QuerySystem<Q, F>
where
    Q: Query,
{
    func: F,
    name: String,
    _marker: std::marker::PhantomData<Q>,
}

impl<Q, F> QuerySystem<Q, F>
where
    Q: Query,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            name: "query_system".to_string(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl<Q, F> System for QuerySystem<Q, F>
where
    Q: Query + Send,
    F: FnMut(Q::Item<'_>) + Send,
{
    fn run(&mut self, world: &mut World) {
        for item in world.query::<Q>() {
            (self.func)(item);
        }
    }

    fn reads(&self) -> Vec<TypeId> {
        Q::read_types()
    }

    fn writes(&self) -> Vec<TypeId> {
        Q::write_types()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Execution stage for grouping systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}

/// Schedule for running systems with parallel execution support
pub struct Schedule {
    stages: HashMap<Stage, StageExecutor>,
    stage_order: Vec<Stage>,
}

impl Schedule {
    pub fn new() -> Self {
        let mut schedule = Self {
            stages: HashMap::new(),
            stage_order: vec![
                Stage::PreUpdate,
                Stage::Update,
                Stage::PostUpdate,
                Stage::Render,
            ],
        };

        // Initialize default stages
        for &stage in &schedule.stage_order {
            schedule.stages.insert(stage, StageExecutor::new());
        }

        schedule
    }

    /// Add a system to a specific stage
    pub fn add_system<S: System + 'static>(&mut self, stage: Stage, system: S) -> &mut Self {
        self.stages
            .entry(stage)
            .or_insert_with(StageExecutor::new)
            .add_system(system);
        self
    }

    /// Add a system to the Update stage (convenience method)
    pub fn add_update_system<S: System + 'static>(&mut self, system: S) -> &mut Self {
        self.add_system(Stage::Update, system)
    }

    /// Run all systems in all stages
    pub fn run(&mut self, world: &mut World) {
        for &stage in &self.stage_order {
            if let Some(executor) = self.stages.get_mut(&stage) {
                executor.run(world);
            }
        }
    }

    /// Run a specific stage
    pub fn run_stage(&mut self, stage: Stage, world: &mut World) {
        if let Some(executor) = self.stages.get_mut(&stage) {
            executor.run(world);
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes systems within a stage, potentially in parallel
struct StageExecutor {
    systems: Vec<Box<dyn System>>,
    batches: Vec<Vec<usize>>, // Indices of systems that can run in parallel
}

impl StageExecutor {
    fn new() -> Self {
        Self {
            systems: Vec::new(),
            batches: Vec::new(),
        }
    }

    fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
        self.rebuild_batches();
    }

    /// Rebuild parallel execution batches based on system dependencies
    fn rebuild_batches(&mut self) {
        self.batches.clear();

        let mut remaining: HashSet<usize> = (0..self.systems.len()).collect();

        while !remaining.is_empty() {
            let mut batch = Vec::new();
            let mut batch_reads = HashSet::new();
            let mut batch_writes = HashSet::new();

            let remaining_vec: Vec<usize> = remaining.iter().copied().collect();

            for &idx in &remaining_vec {
                let system = &self.systems[idx];
                let reads = system.reads();
                let writes = system.writes();

                // Check if this system conflicts with the current batch
                let has_conflict = writes
                    .iter()
                    .any(|w| batch_reads.contains(w) || batch_writes.contains(w))
                    || reads.iter().any(|r| batch_writes.contains(r));

                if !has_conflict {
                    batch.push(idx);
                    batch_reads.extend(reads);
                    batch_writes.extend(writes);
                    remaining.remove(&idx);
                }
            }

            if !batch.is_empty() {
                self.batches.push(batch);
            } else {
                // If we couldn't add anything, there might be a deadlock
                // Just add the first remaining system to break it
                if let Some(&idx) = remaining.iter().next() {
                    self.batches.push(vec![idx]);
                    remaining.remove(&idx);
                }
            }
        }
    }

    fn run(&mut self, world: &mut World) {
        for batch in &self.batches {
            if batch.len() == 1 {
                // Single system, run directly
                self.systems[batch[0]].run(world);
            } else {
                // Multiple systems can run in parallel
                // Note: This is unsafe and requires careful handling
                // For now, we'll run them sequentially as true parallel access
                // to World requires more sophisticated synchronization
                for &idx in batch {
                    self.systems[idx].run(world);
                }
            }
        }
    }
}

/// Parallel schedule that can execute non-conflicting systems in parallel
pub struct ParallelSchedule {
    systems: Vec<Box<dyn System>>,
    dependency_graph: DependencyGraph,
}

impl ParallelSchedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            dependency_graph: DependencyGraph::new(),
        }
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) -> &mut Self {
        let idx = self.systems.len();
        let reads = system.reads();
        let writes = system.writes();

        self.systems.push(Box::new(system));
        self.dependency_graph.add_system(idx, reads, writes);

        self
    }

    /// Execute systems in parallel where possible
    pub fn run(&mut self, world: &mut World) {
        let batches = self.dependency_graph.compute_batches();

        for batch in batches {
            // For true parallelism, we'd need to split World access
            // For now, run batch systems sequentially
            for idx in batch {
                self.systems[idx].run(world);
            }
        }
    }
}

impl Default for ParallelSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks dependencies between systems
struct DependencyGraph {
    systems: Vec<SystemNode>,
}

struct SystemNode {
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    fn add_system(&mut self, _idx: usize, reads: Vec<TypeId>, writes: Vec<TypeId>) {
        self.systems.push(SystemNode { reads, writes });
    }

    /// Compute batches of systems that can run in parallel
    fn compute_batches(&self) -> Vec<Vec<usize>> {
        let mut batches = Vec::new();
        let mut remaining: HashSet<usize> = (0..self.systems.len()).collect();

        while !remaining.is_empty() {
            let mut batch = Vec::new();
            let mut batch_reads = HashSet::new();
            let mut batch_writes = HashSet::new();

            let remaining_vec: Vec<usize> = remaining.iter().copied().collect();

            for &idx in &remaining_vec {
                let node = &self.systems[idx];

                // Check for conflicts
                let has_write_conflict = node
                    .writes
                    .iter()
                    .any(|w| batch_reads.contains(w) || batch_writes.contains(w));

                let has_read_conflict = node.reads.iter().any(|r| batch_writes.contains(r));

                if !has_write_conflict && !has_read_conflict {
                    batch.push(idx);
                    batch_reads.extend(node.reads.iter().copied());
                    batch_writes.extend(node.writes.iter().copied());
                    remaining.remove(&idx);
                }
            }

            if !batch.is_empty() {
                batches.push(batch);
            } else {
                // Break potential deadlock
                if let Some(&idx) = remaining.iter().next() {
                    batches.push(vec![idx]);
                    remaining.remove(&idx);
                }
            }
        }

        batches
    }
}

/// Helper macro to create query systems more easily
#[macro_export]
macro_rules! query_system {
    ($func:expr) => {
        $crate::system::QuerySystem::new($func)
    };
}
