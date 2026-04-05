use crate::simple_world::EntityId as SimpleEntityId;
use crate::simple_world::SimpleWorld;
use soroban_sdk::{Env, Symbol, Vec};

/// Which backing storage a `SimpleQuery` should consider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStorage {
    /// Only scan table-backed components, optimized for gameplay loops.
    Table,
    /// Include both table-backed and sparse components.
    Any,
}
/// Query builder and executable query for `SimpleWorld`.
///
/// `SimpleQuery` is the preferred query surface for Soroban gameplay loops:
/// it supports required, excluded, and optional-match components while
/// selecting the narrowest indexed candidate set available.
#[derive(Debug, Clone)]
pub struct SimpleQuery {
    required_components: Vec<Symbol>,
    excluded_components: Vec<Symbol>,
    any_components: Vec<Symbol>,
    storage: QueryStorage,
}

impl SimpleQuery {
    /// Create an empty query over table-backed components.
    pub fn new(env: &Env) -> Self {
        Self {
            required_components: Vec::new(env),
            excluded_components: Vec::new(env),
            any_components: Vec::new(env),
            storage: QueryStorage::Table,
        }
    }

    /// Require the entity to have this component.
    pub fn with_component(mut self, component_type: Symbol) -> Self {
        self.required_components.push_back(component_type);
        self
    }

    /// Require the entity to have all components from this slice.
    pub fn with_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.required_components.push_back(component_type.clone());
        }
        self
    }

    /// Exclude entities with this component.
    pub fn without_component(mut self, component_type: Symbol) -> Self {
        self.excluded_components.push_back(component_type);
        self
    }

    /// Exclude entities that have any component from this slice.
    pub fn without_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.excluded_components.push_back(component_type.clone());
        }
        self
    }

    /// Require the entity to match at least one component from this set.
    pub fn with_any_component(mut self, component_type: Symbol) -> Self {
        self.any_components.push_back(component_type);
        self
    }

    /// Require the entity to match at least one component from this slice.
    pub fn with_any_components(mut self, component_types: &[Symbol]) -> Self {
        for component_type in component_types {
            self.any_components.push_back(component_type.clone());
        }
        self
    }

    /// Include both table and sparse storage during execution.
    pub fn include_sparse(mut self) -> Self {
        self.storage = QueryStorage::Any;
        self
    }

    /// Returns whether the query has no constraints.
    pub fn is_empty(&self) -> bool {
        self.required_components.is_empty()
            && self.excluded_components.is_empty()
            && self.any_components.is_empty()
    }

    /// Returns the current storage mode for the query.
    pub fn storage(&self) -> QueryStorage {
        self.storage
    }

    /// Execute the query against a `SimpleWorld`.
    pub fn execute(&self, world: &SimpleWorld, env: &Env) -> Vec<SimpleEntityId> {
        let candidates = self.candidate_entities(world, env);
        let mut results = Vec::new(env);

        for i in 0..candidates.len() {
            if let Some(entity_id) = candidates.get(i) {
                if self.matches(world, entity_id) {
                    results.push_back(entity_id);
                }
            }
        }

        results
    }

    /// Returns the number of entities that must be scanned before final filtering.
    pub fn candidate_count(&self, world: &SimpleWorld, env: &Env) -> usize {
        self.candidate_entities(world, env)
            .len()
            .try_into()
            .unwrap()
    }

    fn candidate_entities(&self, world: &SimpleWorld, env: &Env) -> Vec<SimpleEntityId> {
        let mut best: Option<Vec<SimpleEntityId>> = None;

        for i in 0..self.required_components.len() {
            if let Some(component_type) = self.required_components.get(i) {
                let entities = self.entities_for_component(world, &component_type, env);
                if best
                    .as_ref()
                    .map(|current| entities.len() < current.len())
                    .unwrap_or(true)
                {
                    best = Some(entities);
                }
            }
        }

        if let Some(candidates) = best {
            return candidates;
        }

        if !self.any_components.is_empty() {
            let mut union = Vec::new(env);
            for i in 0..self.any_components.len() {
                if let Some(component_type) = self.any_components.get(i) {
                    let entities = self.entities_for_component(world, &component_type, env);
                    for j in 0..entities.len() {
                        if let Some(entity_id) = entities.get(j) {
                            if !contains_entity(&union, entity_id) {
                                union.push_back(entity_id);
                            }
                        }
                    }
                }
            }
            return union;
        }

        let mut all_entities = Vec::new(env);
        for entity_id in world.entity_components.keys().iter() {
            all_entities.push_back(entity_id);
        }
        all_entities
    }

    fn entities_for_component(
        &self,
        world: &SimpleWorld,
        component_type: &Symbol,
        env: &Env,
    ) -> Vec<SimpleEntityId> {
        match self.storage {
            QueryStorage::Table => world.get_table_entities_with_component(component_type, env),
            QueryStorage::Any => world.get_all_entities_with_component(component_type, env),
        }
    }

    fn matches(&self, world: &SimpleWorld, entity_id: SimpleEntityId) -> bool {
        for i in 0..self.required_components.len() {
            if let Some(component_type) = self.required_components.get(i) {
                if !world.has_component(entity_id, &component_type) {
                    return false;
                }
            }
        }

        for i in 0..self.excluded_components.len() {
            if let Some(component_type) = self.excluded_components.get(i) {
                if world.has_component(entity_id, &component_type) {
                    return false;
                }
            }
        }

        if self.any_components.is_empty() {
            return true;
        }

        for i in 0..self.any_components.len() {
            if let Some(component_type) = self.any_components.get(i) {
                if world.has_component(entity_id, &component_type) {
                    return true;
                }
            }
        }

        false
    }
}

fn contains_entity(entities: &Vec<SimpleEntityId>, entity_id: SimpleEntityId) -> bool {
    for i in 0..entities.len() {
        if let Some(candidate) = entities.get(i) {
            if candidate == entity_id {
                return true;
            }
        }
    }
    false
}

/// Version-aware cache for a `SimpleQuery`.
#[derive(Debug, Clone)]
pub struct SimpleQueryState {
    query: SimpleQuery,
    cached_results: Vec<SimpleEntityId>,
    cached_version: u64,
}

impl SimpleQueryState {
    pub fn new(query: SimpleQuery, env: &Env) -> Self {
        Self {
            query,
            cached_results: Vec::new(env),
            cached_version: 0,
        }
    }

    pub fn execute<'a>(&'a mut self, world: &SimpleWorld, env: &Env) -> &'a Vec<SimpleEntityId> {
        if self.cached_version != world.version() {
            self.cached_results = self.query.execute(world, env);
            self.cached_version = world.version();
        }
        &self.cached_results
    }

    pub fn invalidate(&mut self) {
        self.cached_version = 0;
    }

    pub fn is_valid(&self, world_version: u64) -> bool {
        self.cached_version == world_version
    }

    pub fn query(&self) -> &SimpleQuery {
        &self.query
    }
}

/// Builder for `SimpleQuery`.
pub struct SimpleQueryBuilder {
    query: SimpleQuery,
}

impl SimpleQueryBuilder {
    pub fn new(env: &Env) -> Self {
        Self {
            query: SimpleQuery::new(env),
        }
    }

    pub fn with_component(mut self, component_type: Symbol) -> Self {
        self.query = self.query.with_component(component_type);
        self
    }

    pub fn with_components(mut self, component_types: &[Symbol]) -> Self {
        self.query = self.query.with_components(component_types);
        self
    }

    pub fn without_component(mut self, component_type: Symbol) -> Self {
        self.query = self.query.without_component(component_type);
        self
    }

    pub fn without_components(mut self, component_types: &[Symbol]) -> Self {
        self.query = self.query.without_components(component_types);
        self
    }

    pub fn with_any_component(mut self, component_type: Symbol) -> Self {
        self.query = self.query.with_any_component(component_type);
        self
    }

    pub fn with_any_components(mut self, component_types: &[Symbol]) -> Self {
        self.query = self.query.with_any_components(component_types);
        self
    }

    pub fn include_sparse(mut self) -> Self {
        self.query = self.query.include_sparse();
        self
    }

    pub fn build(self) -> SimpleQuery {
        self.query
    }

    pub fn build_state(self, env: &Env) -> SimpleQueryState {
        SimpleQueryState::new(self.query, env)
    }
}

/// Cached query for `SimpleWorld` that avoids re-scanning when the world hasn't changed.
///
/// Tracks a single component type and caches matching entity IDs.
/// Automatically invalidates when the world version changes.
///
/// # Example
/// ```
/// use cougr_core::query::SimpleQueryCache;
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let entity = world.spawn_entity();
/// world.add_component(entity, symbol_short!("position"), Bytes::new(&env));
///
/// let mut cache = SimpleQueryCache::new(symbol_short!("position"), &env);
/// let entities = cache.execute(&world, &env);
/// assert_eq!(entities.len(), 1);
/// let entities2 = cache.execute(&world, &env);
/// assert_eq!(entities2.len(), 1);
/// ```
pub struct SimpleQueryCache {
    state: SimpleQueryState,
}

impl SimpleQueryCache {
    /// Create a new query cache for a specific component type
    pub fn new(component_type: Symbol, env: &Env) -> Self {
        Self {
            state: SimpleQueryState::new(SimpleQuery::new(env).with_component(component_type), env),
        }
    }

    /// Create a cache from an explicit `SimpleQuery`.
    pub fn from_query(query: SimpleQuery, env: &Env) -> Self {
        Self {
            state: SimpleQueryState::new(query, env),
        }
    }

    /// Execute the query, returning cached results if the world hasn't changed.
    pub fn execute(
        &mut self,
        world: &crate::simple_world::SimpleWorld,
        env: &Env,
    ) -> &Vec<SimpleEntityId> {
        self.state.execute(world, env)
    }

    /// Force invalidation of the cache.
    pub fn invalidate(&mut self) {
        self.state.invalidate();
    }

    /// Check if the cache is up-to-date with the given world version.
    pub fn is_valid(&self, world_version: u64) -> bool {
        self.state.is_valid(world_version)
    }

    /// Returns the underlying query.
    pub fn query(&self) -> &SimpleQuery {
        self.state.query()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, Env};

    #[test]
    fn test_simple_query_cache() {
        let env = Env::default();
        let mut world = crate::simple_world::SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        let data = soroban_sdk::Bytes::from_array(&env, &[1, 2, 3, 4]);
        world.add_component(e1, symbol_short!("pos"), data);

        let mut cache = SimpleQueryCache::new(symbol_short!("pos"), &env);

        // First execution populates cache
        let results = cache.execute(&world, &env);
        assert_eq!(results.len(), 1);
        assert!(cache.is_valid(world.version()));

        // Second execution uses cache (world unchanged)
        let results2 = cache.execute(&world, &env);
        assert_eq!(results2.len(), 1);

        // Mutating world invalidates cache
        let e2 = world.spawn_entity();
        let data2 = soroban_sdk::Bytes::from_array(&env, &[5, 6, 7, 8]);
        world.add_component(e2, symbol_short!("pos"), data2);
        assert!(!cache.is_valid(world.version()));

        // Re-execution after mutation returns updated results
        let results3 = cache.execute(&world, &env);
        assert_eq!(results3.len(), 2);
        assert!(cache.is_valid(world.version()));
    }

    #[test]
    fn test_simple_query_cache_invalidate() {
        let env = Env::default();
        let mut cache = SimpleQueryCache::new(symbol_short!("test"), &env);
        let mut world = crate::simple_world::SimpleWorld::new(&env);
        let entity = world.spawn_entity();
        world.add_component(
            entity,
            symbol_short!("test"),
            soroban_sdk::Bytes::from_array(&env, &[1]),
        );
        let _ = cache.execute(&world, &env);
        assert!(cache.is_valid(world.version()));
        cache.invalidate();
        assert!(!cache.is_valid(world.version()));
    }

    #[test]
    fn test_simple_query_builder_with_sparse_and_any() {
        let env = Env::default();
        let mut world = crate::simple_world::SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            soroban_sdk::Bytes::from_array(&env, &[1]),
        );

        let e2 = world.spawn_entity();
        world.add_component_with_storage(
            e2,
            symbol_short!("tag"),
            soroban_sdk::Bytes::from_array(&env, &[2]),
            crate::component::ComponentStorage::Sparse,
        );

        let query = SimpleQueryBuilder::new(&env)
            .with_any_component(symbol_short!("pos"))
            .with_any_component(symbol_short!("tag"))
            .include_sparse()
            .build();

        let results = query.execute(&world, &env);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_simple_query_state_tracks_world_version() {
        let env = Env::default();
        let mut world = crate::simple_world::SimpleWorld::new(&env);
        let query = SimpleQueryBuilder::new(&env)
            .with_component(symbol_short!("pos"))
            .build();
        let mut state = SimpleQueryState::new(query, &env);

        assert_eq!(state.execute(&world, &env).len(), 0);
        assert!(state.is_valid(world.version()));

        let entity = world.spawn_entity();
        world.add_component(
            entity,
            symbol_short!("pos"),
            soroban_sdk::Bytes::from_array(&env, &[3]),
        );

        assert!(!state.is_valid(world.version()));
        assert_eq!(state.execute(&world, &env).len(), 1);
    }

    #[test]
    fn test_simple_query_bulk_filters_and_candidate_count() {
        let env = Env::default();
        let mut world = crate::simple_world::SimpleWorld::new(&env);

        let e1 = world.spawn_entity();
        world.add_component(
            e1,
            symbol_short!("pos"),
            soroban_sdk::Bytes::from_array(&env, &[1]),
        );
        world.add_component(
            e1,
            symbol_short!("vel"),
            soroban_sdk::Bytes::from_array(&env, &[2]),
        );

        let e2 = world.spawn_entity();
        world.add_component(
            e2,
            symbol_short!("pos"),
            soroban_sdk::Bytes::from_array(&env, &[3]),
        );
        world.add_component_with_storage(
            e2,
            symbol_short!("sleep"),
            soroban_sdk::Bytes::from_array(&env, &[4]),
            crate::component::ComponentStorage::Sparse,
        );

        let query = SimpleQueryBuilder::new(&env)
            .with_components(&[symbol_short!("pos")])
            .without_components(&[symbol_short!("sleep")])
            .with_any_components(&[symbol_short!("vel"), symbol_short!("sleep")])
            .include_sparse()
            .build();

        assert_eq!(query.candidate_count(&world, &env), 2);
        let results = query.execute(&world, &env);
        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0), Some(e1));
    }
}
