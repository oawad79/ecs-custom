# Simple ECS

A high-performance Entity Component System (ECS) implementation in Rust with advanced features.

## Features

- ✅ **Archetype-based storage** - Efficient memory layout and cache-friendly iteration
- ✅ **Flexible queries** - Query entities with specific component combinations
- ✅ **Query filters** - `With`, `Without`, `Option`, `Changed` filters
- ✅ **Component add/remove** - Dynamic component manipulation
- ✅ **Resources** - Global singleton data accessible to systems
- ✅ **Events** - Event-driven communication between systems
- ✅ **Hierarchy** - Parent-child entity relationships
- ✅ **Commands** - Deferred entity/component operations
- ✅ **Change detection** - Track component modifications
- ✅ **System scheduling** - Sequential and parallel system execution
- ✅ **Error handling** - Proper error types instead of panics
- ✅ **Benchmarking** - Performance testing suite

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
simple_ecs = { path = "." }
```

## Quick Start

```rust
use simple_ecs::*;

#[derive(Debug, Clone, Copy)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Clone, Copy)]
struct Velocity { x: f32, y: f32 }

fn main() {
    let mut world = World::new();
    
    // Spawn entities
    world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));
    
    // Query entities
    for (pos, vel) in world.query::<(&mut Position, &Velocity)>() {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}
```

## Examples

### Basic Usage

```bash
cargo run --example basic
```

### Hierarchy

```bash
cargo run --example hierarchy
```

### Events

```bash
cargo run --example events
```

### Commands

```bash
cargo run --example commands
```

## Running Tests

```bash
cargo test
```

## Running Benchmarks

````bash
cargo bench
