#![no_std]
#![allow(unsafe_code)]
#![doc = r#"
Cougr is a monolithic-on-the-outside ECS framework for Soroban-compatible applications.

The public API is intentionally split into:

- root re-exports for the onboarding path
- `accounts` for account abstraction and session flows
- `zk::stable` for stable privacy primitives
- `zk::experimental` for fast-moving proof-verification APIs

# Golden Path

```rust
use cougr_core::{ComponentTrait, Position, SimpleWorld};
use soroban_sdk::Env;

let env = Env::default();
let mut world = SimpleWorld::new(&env);
let entity = world.spawn_entity();
world.set_typed(&env, entity, &Position::new(1, 2));

let pos: Position = world.get_typed(&env, entity).unwrap();
assert_eq!(pos.x, 1);
```

# Stability

- ECS runtime and storage: Beta
- Accounts: Beta
- `zk::stable`: Stable subset
- `zk::experimental`: Experimental
"#]

extern crate alloc;

// Global allocator for WASM
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Macros must be declared before modules that use them
#[macro_use]
pub mod macros;

// Public product domains
pub mod accounts;
pub mod archetype_world;
pub mod change_tracker;
pub mod commands;
pub mod component;
#[cfg(feature = "debug")]
#[doc(hidden)]
pub mod debug;
pub mod error;
pub mod event;
pub mod game_world;
pub mod hooks;
pub mod incremental;
pub mod observers;
pub mod plugin;
pub mod query;
pub mod resource;
pub mod scheduler;
pub mod simple_world;
pub mod system;
pub mod world;
pub mod zk;

// Internal implementation modules kept out of the default public surface.
mod entity;
mod storage;

// Root-level golden path re-exports.
pub use archetype_world::{ArchetypeQueryCache, ArchetypeWorld};
pub use change_tracker::{ChangeTracker, TrackedWorld};
pub use commands::CommandQueue;
pub use component::{Component, ComponentId, ComponentStorage, ComponentTrait, Position};
pub use entity::{Entity, EntityId};
pub use error::{CougrError, CougrResult};
pub use event::{Event, EventReader, EventWriter};
pub use game_world::GameWorld;
pub use hooks::{HookRegistry, HookedWorld};
pub use incremental::{StorageWorld, WorldMetadata};
pub use observers::{ObservedWorld, ObserverRegistry};
pub use plugin::{Plugin, PluginApp};
pub use query::{Query, QueryState, SimpleQueryCache};
pub use resource::Resource;
pub use scheduler::{SimpleScheduler, SystemScheduler};
pub use simple_world::SimpleWorld;
pub use storage::{SparseStorage, Storage, TableStorage};
pub use system::{IntoSystem, MovementSystem, System, SystemParam};
pub use world::World;

/// Common ECS imports for the default onboarding path.
pub mod prelude {
    pub use super::{
        ArchetypeWorld, CommandQueue, Component, ComponentStorage, ComponentTrait, EntityId,
        Position, Query, Resource, SimpleWorld, World,
    };
}

/// Advanced runtime primitives that remain supported but are not part of the
/// smallest onboarding surface.
pub mod runtime {
    pub use super::{
        resource::Resource, ChangeTracker, Event, EventReader, EventWriter, HookRegistry,
        HookedWorld, ObservedWorld, ObserverRegistry, Plugin, PluginApp, QueryState,
        SimpleQueryCache, SimpleScheduler, StorageWorld, System, SystemParam, SystemScheduler,
        TrackedWorld,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_world_creation() {
        let _env = Env::default();
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_entity_spawn() {
        let _env = Env::default();
        let mut world = World::new();
        let _entity = world.spawn_empty();
        assert_eq!(world.entity_count(), 1);
    }
}
