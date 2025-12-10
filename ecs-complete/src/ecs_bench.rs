//! Benchmark utilities for the ECS

use crate::World;

/// A simple position component for benchmarking
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl BenchPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// A simple velocity component for benchmarking
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchVelocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl BenchVelocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// A simple data component for benchmarking
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchData {
    pub value: i32,
}

impl BenchData {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

/// A marker component for benchmarking
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchMarker;

/// Helper function to spawn entities with position and velocity
pub fn spawn_entities_with_position_velocity(world: &mut World, count: usize) {
    for i in 0..count {
        let i = i as f32;
        world.spawn((
            BenchPosition::new(i, i * 2.0, i * 3.0),
            BenchVelocity::new(1.0, 2.0, 3.0),
        ));
    }
}

/// Helper function to spawn entities with just position
pub fn spawn_entities_with_position(world: &mut World, count: usize) {
    for i in 0..count {
        let i = i as f32;
        world.spawn((BenchPosition::new(i, i * 2.0, i * 3.0),));
    }
}

/// Helper function to spawn entities with position, velocity, and data
pub fn spawn_entities_complex(world: &mut World, count: usize) {
    for i in 0..count {
        let i_f32 = i as f32;
        world.spawn((
            BenchPosition::new(i_f32, i_f32 * 2.0, i_f32 * 3.0),
            BenchVelocity::new(1.0, 2.0, 3.0),
            BenchData::new(i as i32),
        ));
    }
}

/// Helper function to spawn a mix of entities with different component combinations
pub fn spawn_entities_fragmented(world: &mut World, count: usize) {
    for i in 0..count {
        let i_f32 = i as f32;
        match i % 4 {
            0 => {
                world.spawn((BenchPosition::new(i_f32, i_f32, i_f32),));
            }
            1 => {
                world.spawn((
                    BenchPosition::new(i_f32, i_f32, i_f32),
                    BenchVelocity::new(1.0, 1.0, 1.0),
                ));
            }
            2 => {
                world.spawn((
                    BenchPosition::new(i_f32, i_f32, i_f32),
                    BenchData::new(i as i32),
                ));
            }
            _ => {
                world.spawn((
                    BenchPosition::new(i_f32, i_f32, i_f32),
                    BenchVelocity::new(1.0, 1.0, 1.0),
                    BenchData::new(i as i32),
                    BenchMarker,
                ));
            }
        }
    }
}
