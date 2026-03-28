#![no_std]
#![allow(unsafe_code)]

extern crate alloc;

use soroban_sdk::{Symbol, Vec};

// Global allocator for WASM
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Macros must be declared before modules that use them
#[macro_use]
pub mod macros;

// Core ECS types adapted for Soroban
pub mod accounts;
pub mod archetype_world;
pub mod change_tracker;
pub mod commands;
pub mod component;
pub mod components;
#[cfg(feature = "debug")]
pub mod debug;
pub mod entity;
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
pub mod storage;
pub mod system;
pub mod systems;
pub mod world;
pub mod zk;

// Re-export core types
pub use archetype_world::{ArchetypeQueryCache, ArchetypeWorld};
pub use change_tracker::{ChangeTracker, TrackedWorld};
pub use commands::CommandQueue;
pub use component::{Component, ComponentId, ComponentStorage, ComponentTrait};
pub use components::Position;
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
pub use system::{IntoSystem, System, SystemParam};
pub use systems::MovementSystem;
pub use world::World;

// Library functions for ECS operations
pub fn create_world() -> World {
    World::new()
}

pub fn spawn_entity(world: &mut World, components: Vec<Component>) -> EntityId {
    let entity = world.spawn(components);
    entity.id()
}

pub fn add_component(world: &mut World, entity_id: EntityId, component: Component) -> bool {
    world.add_component_to_entity(entity_id, component);
    true
}

pub fn remove_component(world: &mut World, entity_id: EntityId, component_type: Symbol) -> bool {
    world.remove_component_from_entity(entity_id, &component_type)
}

pub fn get_component(
    world: &World,
    entity_id: EntityId,
    component_type: Symbol,
) -> Option<Component> {
    world.get_component(entity_id, &component_type)
}

/// Deprecated placeholder API.
///
/// This function does not currently execute a real query and always returns
/// an empty result. It remains temporarily available for compatibility while
/// Cougr's Phase 0 API cleanup is in progress.
#[deprecated(
    since = "0.0.1",
    note = "query_entities is a placeholder and not part of Cougr's stable surface; use World::query_entities or SimpleWorld query APIs instead"
)]
pub fn query_entities(
    _world: &World,
    _component_types: Vec<Symbol>,
    env: &soroban_sdk::Env,
) -> Vec<EntityId> {
    // This placeholder is intentionally retained only as a temporary
    // compatibility shim while the public API is narrowed.
    Vec::new(env)
}

// Predule for common types
pub mod prelude {
    pub use super::{
        component::{Component, ComponentId, ComponentStorage},
        entity::{Entity, EntityId},
        event::{Event, EventReader, EventWriter},
        query::{Query, QueryState},
        resource::Resource,
        storage::{SparseStorage, Storage, TableStorage},
        system::{IntoSystem, System, SystemParam},
        world::World,
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
