mod indexing;
#[cfg(test)]
mod tests;

use crate::component::{ComponentStorage, ComponentTrait};
use soroban_sdk::{contracttype, Bytes, Env, Map, Symbol, Vec};

/// Simple entity ID type for Soroban-optimized ECS.
pub type EntityId = u32;

/// Simplified game world optimized for Soroban on-chain storage.
///
/// Uses `Map`-based storage for O(log n) component lookups instead of
/// linear scans. This is the recommended ECS container for Soroban contracts.
///
/// ## Dual-Map storage
///
/// Components are split into two maps based on their `ComponentStorage` kind:
/// - **Table** (`components`): Frequently-iterated components (e.g., Position, Velocity).
///   Queried by `get_entities_with_component()`.
/// - **Sparse** (`sparse_components`): Infrequently-accessed marker or tag components.
///   Not included in the default entity query; use `get_all_entities_with_component()` to include them.
///
/// Both maps are transparent to `get_component()`, `has_component()`, and `remove_component()`.
///
/// # Example
/// ```
/// use cougr_core::component::ComponentStorage;
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let entity_id = world.spawn_entity();
/// world.add_component(entity_id, symbol_short!("position"), Bytes::new(&env));
/// world.add_component_with_storage(
///     entity_id,
///     symbol_short!("marker"),
///     Bytes::new(&env),
///     ComponentStorage::Sparse,
/// );
/// assert!(world.has_component(entity_id, &symbol_short!("position")));
/// ```
#[contracttype]
#[derive(Clone, Debug)]
pub struct SimpleWorld {
    pub next_entity_id: u32,
    /// Table component data keyed by (entity_id, component_type).
    pub components: Map<(u32, Symbol), Bytes>,
    /// Sparse component data keyed by (entity_id, component_type).
    pub sparse_components: Map<(u32, Symbol), Bytes>,
    /// Tracks which component types each entity has.
    pub entity_components: Map<u32, Vec<Symbol>>,
    /// Direct index for frequently queried table-backed components.
    pub table_index: Map<Symbol, Vec<u32>>,
    /// Direct index for all components regardless of backing storage.
    pub all_index: Map<Symbol, Vec<u32>>,
    /// Version counter incremented on structural changes (add/remove/despawn).
    /// Used for query cache invalidation.
    pub version: u64,
}

impl SimpleWorld {
    pub fn new(env: &Env) -> Self {
        Self {
            next_entity_id: 1,
            components: Map::new(env),
            sparse_components: Map::new(env),
            entity_components: Map::new(env),
            table_index: Map::new(env),
            all_index: Map::new(env),
            version: 0,
        }
    }

    /// Returns the current world version for cache invalidation.
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn spawn_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    fn has_component_in_table(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.components
            .contains_key((entity_id, component_type.clone()))
    }

    fn has_component_in_sparse(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.sparse_components
            .contains_key((entity_id, component_type.clone()))
    }

    /// Add a component using the default **Table** storage.
    pub fn add_component(&mut self, entity_id: EntityId, component_type: Symbol, data: Bytes) {
        self.add_component_with_storage(entity_id, component_type, data, ComponentStorage::Table);
    }

    /// Add a component, routing to the Table or Sparse map based on `storage`.
    pub fn add_component_with_storage(
        &mut self,
        entity_id: EntityId,
        component_type: Symbol,
        data: Bytes,
        storage: ComponentStorage,
    ) {
        self.version += 1;
        let was_in_table = self.has_component_in_table(entity_id, &component_type);
        let was_in_sparse = self.has_component_in_sparse(entity_id, &component_type);

        match storage {
            ComponentStorage::Table => {
                self.components
                    .set((entity_id, component_type.clone()), data);
                if was_in_sparse {
                    self.sparse_components
                        .remove((entity_id, component_type.clone()));
                }
            }
            ComponentStorage::Sparse => {
                self.sparse_components
                    .set((entity_id, component_type.clone()), data);
                if was_in_table {
                    self.components.remove((entity_id, component_type.clone()));
                }
            }
        }

        let mut types = self
            .entity_components
            .get(entity_id)
            .unwrap_or_else(|| Vec::new(self.components.env()));

        let mut found = false;
        for i in 0..types.len() {
            if let Some(t) = types.get(i) {
                if t == component_type {
                    found = true;
                    break;
                }
            }
        }
        if !found {
            types.push_back(component_type.clone());
        }
        self.entity_components.set(entity_id, types);

        indexing::push_index(&mut self.all_index, &component_type, entity_id);
        match storage {
            ComponentStorage::Table => {
                indexing::push_index(&mut self.table_index, &component_type, entity_id);
            }
            ComponentStorage::Sparse => {
                indexing::remove_from_index(&mut self.table_index, &component_type, entity_id);
            }
        }
    }

    /// Get a component's data, checking both Table and Sparse maps transparently.
    pub fn get_component(&self, entity_id: EntityId, component_type: &Symbol) -> Option<Bytes> {
        self.components
            .get((entity_id, component_type.clone()))
            .or_else(|| {
                self.sparse_components
                    .get((entity_id, component_type.clone()))
            })
    }

    /// Remove a component from both Table and Sparse maps transparently.
    pub fn remove_component(&mut self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.version += 1;
        let removed = self
            .components
            .remove((entity_id, component_type.clone()))
            .or_else(|| {
                self.sparse_components
                    .remove((entity_id, component_type.clone()))
            });

        if removed.is_some() {
            if let Some(types) = self.entity_components.get(entity_id) {
                let env = self.components.env();
                let mut new_types = Vec::new(env);
                for i in 0..types.len() {
                    if let Some(t) = types.get(i) {
                        if &t != component_type {
                            new_types.push_back(t);
                        }
                    }
                }
                if new_types.is_empty() {
                    self.entity_components.remove(entity_id);
                } else {
                    self.entity_components.set(entity_id, new_types);
                }
            }
            indexing::remove_from_index(&mut self.all_index, component_type, entity_id);
            indexing::remove_from_index(&mut self.table_index, component_type, entity_id);
            true
        } else {
            false
        }
    }

    /// Check if an entity has a component in either Table or Sparse storage.
    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.has_component_in_table(entity_id, component_type)
            || self.has_component_in_sparse(entity_id, component_type)
    }

    pub fn get_entities_with_component(&self, component_type: &Symbol, env: &Env) -> Vec<EntityId> {
        self.table_index
            .get(component_type.clone())
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get entities that have the given component in **Table** storage only.
    /// This is the fast path for querying frequently-iterated components.
    pub fn get_table_entities_with_component(
        &self,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<EntityId> {
        self.table_index
            .get(component_type.clone())
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get entities that have the given component in **either** Table or Sparse storage.
    pub fn get_all_entities_with_component(
        &self,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<EntityId> {
        self.all_index
            .get(component_type.clone())
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Returns the number of entities indexed for a component in table storage only.
    pub fn table_component_count(&self, component_type: &Symbol) -> usize {
        self.table_index
            .get(component_type.clone())
            .map(|entities| entities.len())
            .unwrap_or(0)
            .try_into()
            .unwrap()
    }

    /// Returns the number of entities indexed for a component across both storage classes.
    pub fn component_count(&self, component_type: &Symbol) -> usize {
        self.all_index
            .get(component_type.clone())
            .map(|entities| entities.len())
            .unwrap_or(0)
            .try_into()
            .unwrap()
    }

    /// Get a component and deserialize it into the concrete type.
    ///
    /// # Example
    /// ```
    /// use cougr_core::component::Position;
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::Env;
    ///
    /// let env = Env::default();
    /// let mut world = SimpleWorld::new(&env);
    /// let entity_id = world.spawn_entity();
    /// world.set_typed(&env, entity_id, &Position::new(10, 20));
    /// let pos: Option<Position> = world.get_typed::<Position>(&env, entity_id);
    /// assert_eq!(pos.unwrap().x, 10);
    /// ```
    pub fn get_typed<T: ComponentTrait>(&self, env: &Env, entity_id: EntityId) -> Option<T> {
        let bytes = self.get_component(entity_id, &T::component_type())?;
        T::deserialize(env, &bytes)
    }

    /// Serialize a component and store it, using the type's default storage kind.
    ///
    /// # Example
    /// ```
    /// use cougr_core::component::Position;
    /// use cougr_core::simple_world::SimpleWorld;
    /// use soroban_sdk::Env;
    ///
    /// let env = Env::default();
    /// let mut world = SimpleWorld::new(&env);
    /// let entity_id = world.spawn_entity();
    /// world.set_typed(&env, entity_id, &Position::new(10, 20));
    /// assert!(world.has_typed::<Position>(entity_id));
    /// ```
    pub fn set_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId, component: &T) {
        let symbol = T::component_type();
        let data = component.serialize(env);
        let storage = T::default_storage();
        self.add_component_with_storage(entity_id, symbol, data, storage);
    }

    /// Check if an entity has a component of the given type.
    pub fn has_typed<T: ComponentTrait>(&self, entity_id: EntityId) -> bool {
        self.has_component(entity_id, &T::component_type())
    }

    /// Remove a component of the given type from an entity.
    pub fn remove_typed<T: ComponentTrait>(&mut self, entity_id: EntityId) -> bool {
        self.remove_component(entity_id, &T::component_type())
    }

    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        self.version += 1;
        if let Some(types) = self.entity_components.get(entity_id) {
            for i in 0..types.len() {
                if let Some(t) = types.get(i) {
                    self.components.remove((entity_id, t.clone()));
                    self.sparse_components.remove((entity_id, t.clone()));
                    indexing::remove_from_index(&mut self.all_index, &t, entity_id);
                    indexing::remove_from_index(&mut self.table_index, &t, entity_id);
                }
            }
        }
        self.entity_components.remove(entity_id);
    }
}
