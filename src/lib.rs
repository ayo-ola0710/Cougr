#![no_std]
#![doc = r#"
Cougr is a monolithic-on-the-outside ECS framework for Soroban-compatible applications.

The public API is intentionally split into:

- root re-exports for the onboarding path
- `app` for the default gameplay runtime surface
- `auth` for beta account and session flows
- `privacy` for stable and experimental privacy surfaces
- `ops` for stable operational standards
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

- ECS runtime and storage: Stable
- `app`: Stable
- `standards`: Stable
- Accounts: Beta
- `zk::stable`: Stable
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
mod change_tracker;
pub mod commands;
pub mod component;
#[cfg(feature = "debug")]
#[doc(hidden)]
pub mod debug;
pub mod ecs;
pub mod error;
pub mod event;
mod game_world;
mod hooks;
mod incremental;
mod observers;
pub mod plugin;
pub mod query;
pub mod resource;
pub mod scheduler;
pub mod simple_world;
pub mod standards;
mod system;
pub mod zk;

// Root-level golden path re-exports.
pub use archetype_world::{
    ArchetypeQuery, ArchetypeQueryBuilder, ArchetypeQueryCache, ArchetypeQueryState, ArchetypeWorld,
};
pub use change_tracker::{ChangeTracker, TrackedWorld};
pub use commands::CommandQueue;
pub use component::{Component, ComponentId, ComponentStorage, ComponentTrait, Position};
pub use ecs::{RuntimeWorld, RuntimeWorldMut, WorldBackend};
pub use error::{CougrError, CougrResult};
pub use event::{Event, EventReader, EventWriter};
#[doc(hidden)]
pub use game_world::GameWorld;
#[doc(hidden)]
pub use hooks::{HookRegistry, HookedWorld};
#[doc(hidden)]
pub use incremental::{StorageWorld, WorldMetadata};
#[doc(hidden)]
pub use observers::{ObservedWorld, ObserverRegistry};
pub use plugin::{GameApp, Plugin, PluginApp, PluginGroup};
pub use query::{
    QueryStorage, SimpleQuery, SimpleQueryBuilder, SimpleQueryCache, SimpleQueryState,
};
pub use resource::Resource;
pub use resource::ResourceTrait;
pub use scheduler::{ScheduleError, ScheduleStage, SimpleScheduler, SystemConfig, SystemGroup};
pub use simple_world::SimpleWorld;
pub use system::{
    context_system, named_app_system, named_context_system, named_system, world_system, AppSystem,
    SimpleSystem, SystemContext, SystemSpec,
};

/// Default gameplay runtime surface for new Cougr projects.
pub mod app {
    pub use super::{
        context_system, named_app_system, named_context_system, named_system, world_system,
        AppSystem, CommandQueue, GameApp, Plugin, PluginApp, PluginGroup, Resource, ResourceTrait,
        RuntimeWorld, RuntimeWorldMut, ScheduleError, ScheduleStage, SimpleQuery,
        SimpleQueryBuilder, SimpleScheduler, SimpleSystem, SimpleWorld, SystemConfig,
        SystemContext, SystemGroup, SystemSpec,
    };
}

/// Beta account and session surface.
///
/// This namespace mirrors [`accounts`] but makes its product role explicit.
pub mod auth {
    pub use super::accounts::*;
}

/// Privacy surface split by maturity tier.
///
/// New code should prefer [`privacy::stable`] for defended contracts and only
/// opt into [`privacy::experimental`] knowingly.
pub mod privacy {
    pub use super::zk::{
        experimental, stable, G1Point, G2Point, Groth16Proof, Scalar, VerificationKey, ZKError,
    };
}

/// Stable operational and contract standards.
///
/// This namespace mirrors [`standards`] while making the adoption boundary
/// clearer for application code.
pub mod ops {
    pub use super::standards::*;
}

/// Common ECS imports for the default onboarding path.
pub mod prelude {
    pub use super::simple_world::EntityId;
    pub use super::{
        ArchetypeWorld, CommandQueue, Component, ComponentStorage, ComponentTrait, GameApp,
        PluginGroup, Position, QueryStorage, Resource, RuntimeWorld, RuntimeWorldMut, SimpleQuery,
        SimpleQueryBuilder, SimpleWorld, SystemContext, WorldBackend,
    };
}

/// Advanced runtime primitives that remain supported but are not part of the
/// smallest onboarding surface.
pub mod runtime {
    pub use super::observers::ComponentEvent;
    pub use super::{
        resource::Resource, ChangeTracker, Event, EventReader, EventWriter, HookRegistry,
        HookedWorld, ObservedWorld, ObserverRegistry, Plugin, PluginApp, PluginGroup, QueryStorage,
        RuntimeWorld, RuntimeWorldMut, ScheduleError, ScheduleStage, SimpleQuery,
        SimpleQueryBuilder, SimpleQueryCache, SimpleQueryState, SimpleScheduler, StorageWorld,
        SystemConfig, TrackedWorld, WorldBackend, WorldMetadata,
    };
}
