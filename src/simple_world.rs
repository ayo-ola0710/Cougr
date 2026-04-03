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
        // Route to the correct map
        match storage {
            ComponentStorage::Table => {
                self.components
                    .set((entity_id, component_type.clone()), data);
            }
            ComponentStorage::Sparse => {
                self.sparse_components
                    .set((entity_id, component_type.clone()), data);
            }
        }

        // Update the entity's component type list
        let mut types = self
            .entity_components
            .get(entity_id)
            .unwrap_or_else(|| Vec::new(self.components.env()));

        // Only add the type if not already present
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
            types.push_back(component_type);
        }
        self.entity_components.set(entity_id, types);
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
            // Update entity_components list
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
            true
        } else {
            false
        }
    }

    /// Check if an entity has a component in either Table or Sparse storage.
    pub fn has_component(&self, entity_id: EntityId, component_type: &Symbol) -> bool {
        self.components
            .contains_key((entity_id, component_type.clone()))
            || self
                .sparse_components
                .contains_key((entity_id, component_type.clone()))
    }

    pub fn get_entities_with_component(&self, component_type: &Symbol, env: &Env) -> Vec<EntityId> {
        let mut entities = Vec::new(env);
        for key in self.entity_components.keys().iter() {
            if let Some(types) = self.entity_components.get(key) {
                for i in 0..types.len() {
                    if let Some(t) = types.get(i) {
                        if &t == component_type {
                            entities.push_back(key);
                            break;
                        }
                    }
                }
            }
        }
        entities
    }

    /// Get entities that have the given component in **Table** storage only.
    /// This is the fast path for querying frequently-iterated components.
    pub fn get_table_entities_with_component(
        &self,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<EntityId> {
        let mut entities = Vec::new(env);
        for key in self.entity_components.keys().iter() {
            if self.components.contains_key((key, component_type.clone())) {
                entities.push_back(key);
            }
        }
        entities
    }

    /// Get entities that have the given component in **either** Table or Sparse storage.
    pub fn get_all_entities_with_component(
        &self,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<EntityId> {
        let mut entities = Vec::new(env);
        for key in self.entity_components.keys().iter() {
            if self.components.contains_key((key, component_type.clone()))
                || self
                    .sparse_components
                    .contains_key((key, component_type.clone()))
            {
                entities.push_back(key);
            }
        }
        entities
    }

    // ─── Typed convenience methods ────────────────────────────────

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
        // Remove all components from both maps
        if let Some(types) = self.entity_components.get(entity_id) {
            for i in 0..types.len() {
                if let Some(t) = types.get(i) {
                    self.components.remove((entity_id, t.clone()));
                    self.sparse_components.remove((entity_id, t));
                }
            }
        }
        self.entity_components.remove(entity_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Position;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_simple_world_creation() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        assert_eq!(world.next_entity_id, 1);
        assert_eq!(world.components.len(), 0);
        assert_eq!(world.version(), 0);
    }

    #[test]
    fn test_spawn_entity() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let id1 = world.spawn_entity();
        let id2 = world.spawn_entity();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_add_and_get_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entity_id = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1, 2, 3, 4]);
        world.add_component(entity_id, symbol_short!("test"), data.clone());

        let retrieved = world.get_component(entity_id, &symbol_short!("test"));
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_has_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entity_id = world.spawn_entity();

        assert!(!world.has_component(entity_id, &symbol_short!("test")));

        let data = Bytes::from_array(&env, &[1]);
        world.add_component(entity_id, symbol_short!("test"), data);

        assert!(world.has_component(entity_id, &symbol_short!("test")));
    }

    #[test]
    fn test_remove_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entity_id = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        world.add_component(entity_id, symbol_short!("test"), data);

        assert!(world.remove_component(entity_id, &symbol_short!("test")));
        assert!(!world.has_component(entity_id, &symbol_short!("test")));
        assert!(!world.remove_component(entity_id, &symbol_short!("test")));
    }

    #[test]
    fn test_get_entities_with_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();
        let e3 = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("pos"), data.clone());
        world.add_component(e2, symbol_short!("pos"), data.clone());
        world.add_component(e3, symbol_short!("vel"), data);

        let entities = world.get_entities_with_component(&symbol_short!("pos"), &env);
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn test_despawn_entity() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entity_id = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        world.add_component(entity_id, symbol_short!("a"), data.clone());
        world.add_component(entity_id, symbol_short!("b"), data);

        world.despawn_entity(entity_id);
        assert!(!world.has_component(entity_id, &symbol_short!("a")));
        assert!(!world.has_component(entity_id, &symbol_short!("b")));
    }

    #[test]
    fn test_version_increments_on_mutations() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        assert_eq!(world.version(), 0);

        let e1 = world.spawn_entity();
        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("test"), data);
        assert_eq!(world.version(), 1);

        world.remove_component(e1, &symbol_short!("test"));
        assert_eq!(world.version(), 2);

        let data2 = Bytes::from_array(&env, &[2]);
        world.add_component(e1, symbol_short!("a"), data2);
        assert_eq!(world.version(), 3);

        world.despawn_entity(e1);
        assert_eq!(world.version(), 4);
    }

    #[test]
    fn test_sparse_component_storage() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        world.add_component_with_storage(
            e1,
            symbol_short!("marker"),
            data.clone(),
            ComponentStorage::Sparse,
        );

        // Sparse component is accessible via get/has
        assert!(world.has_component(e1, &symbol_short!("marker")));
        assert_eq!(
            world.get_component(e1, &symbol_short!("marker")),
            Some(data)
        );

        // Stored in sparse map, not table map
        assert!(!world.components.contains_key((e1, symbol_short!("marker"))));
        assert!(world
            .sparse_components
            .contains_key((e1, symbol_short!("marker"))));
    }

    #[test]
    fn test_sparse_vs_table_queries() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        // e1 has Table "pos" and Sparse "tag"
        world.add_component(e1, symbol_short!("pos"), data.clone());
        world.add_component_with_storage(
            e1,
            symbol_short!("tag"),
            data.clone(),
            ComponentStorage::Sparse,
        );
        // e2 has only Sparse "tag"
        world.add_component_with_storage(e2, symbol_short!("tag"), data, ComponentStorage::Sparse);

        // Table-only query for "pos": only e1
        let table_only = world.get_table_entities_with_component(&symbol_short!("pos"), &env);
        assert_eq!(table_only.len(), 1);

        // get_entities_with_component still scans entity_components (both)
        let all_pos = world.get_entities_with_component(&symbol_short!("pos"), &env);
        assert_eq!(all_pos.len(), 1);

        // All-storage query for "tag": e1 and e2
        let all_tag = world.get_all_entities_with_component(&symbol_short!("tag"), &env);
        assert_eq!(all_tag.len(), 2);

        // Table-only for "tag": neither (both are sparse)
        let table_tag = world.get_table_entities_with_component(&symbol_short!("tag"), &env);
        assert_eq!(table_tag.len(), 0);
    }

    #[test]
    fn test_remove_sparse_component() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        world.add_component_with_storage(e1, symbol_short!("tag"), data, ComponentStorage::Sparse);
        assert!(world.has_component(e1, &symbol_short!("tag")));

        assert!(world.remove_component(e1, &symbol_short!("tag")));
        assert!(!world.has_component(e1, &symbol_short!("tag")));
    }

    #[test]
    fn test_despawn_clears_both_maps() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e1 = world.spawn_entity();

        let data = Bytes::from_array(&env, &[1]);
        world.add_component(e1, symbol_short!("pos"), data.clone());
        world.add_component_with_storage(e1, symbol_short!("tag"), data, ComponentStorage::Sparse);

        world.despawn_entity(e1);
        assert!(!world.has_component(e1, &symbol_short!("pos")));
        assert!(!world.has_component(e1, &symbol_short!("tag")));
    }

    #[test]
    fn test_add_component_replaces_existing() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entity_id = world.spawn_entity();

        let data1 = Bytes::from_array(&env, &[1]);
        let data2 = Bytes::from_array(&env, &[2]);

        world.add_component(entity_id, symbol_short!("test"), data1);
        world.add_component(entity_id, symbol_short!("test"), data2.clone());

        let retrieved = world.get_component(entity_id, &symbol_short!("test"));
        assert_eq!(retrieved, Some(data2));
    }

    // ─── Typed API tests ──────────────────────────────────────────

    #[test]
    fn test_set_and_get_typed() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e = world.spawn_entity();

        let pos = Position::new(10, 20);
        world.set_typed(&env, e, &pos);

        let retrieved: Option<Position> = world.get_typed(&env, e);
        assert!(retrieved.is_some());
        let r = retrieved.unwrap();
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
    }

    #[test]
    fn test_has_typed() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e = world.spawn_entity();

        assert!(!world.has_typed::<Position>(e));
        world.set_typed(&env, e, &Position::new(1, 2));
        assert!(world.has_typed::<Position>(e));
    }

    #[test]
    fn test_remove_typed() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e = world.spawn_entity();

        world.set_typed(&env, e, &Position::new(1, 2));
        assert!(world.remove_typed::<Position>(e));
        assert!(!world.has_typed::<Position>(e));
        assert!(!world.remove_typed::<Position>(e));
    }

    #[test]
    fn test_get_typed_nonexistent() {
        let env = Env::default();
        let world = SimpleWorld::new(&env);
        let result: Option<Position> = world.get_typed(&env, 999);
        assert!(result.is_none());
    }

    #[test]
    fn test_set_typed_overwrites() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let e = world.spawn_entity();

        world.set_typed(&env, e, &Position::new(1, 2));
        world.set_typed(&env, e, &Position::new(50, 60));

        let pos: Position = world.get_typed(&env, e).unwrap();
        assert_eq!(pos.x, 50);
        assert_eq!(pos.y, 60);
    }
}
