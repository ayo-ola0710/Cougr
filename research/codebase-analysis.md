# Cougr Codebase Analysis & Quality Assessment

> Deep analysis of the Cougr ECS framework with improvement opportunities, testing gaps, and a proposed roadmap.

---

## Table of Contents

1. [Current Architecture Overview](#1-current-architecture-overview)
2. [Storage Layer Analysis](#2-storage-layer-analysis)
3. [Unintegrated Infrastructure](#3-unintegrated-infrastructure)
4. [API Surface Analysis](#4-api-surface-analysis)
5. [Critical Improvement Opportunities](#5-critical-improvement-opportunities)
6. [Testing & Quality Assessment](#6-testing--quality-assessment)
7. [Example Games Analysis](#7-example-games-analysis)
8. [Missing Capabilities](#8-missing-capabilities)
9. [Performance Analysis](#9-performance-analysis)
10. [Proposed Improvement Roadmap](#10-proposed-improvement-roadmap)

---

## 1. Current Architecture Overview

### Module Structure

Cougr is organized into **127 modules across 16 directories**, representing an ambitious ECS framework adapted for Soroban/no_std WASM environments.

```
src/
├── lib.rs              # Entry point, re-exports, helper functions
├── entity.rs           # EntityId (u64 + u32 generation) and Entity structs
├── component.rs        # Component struct, ComponentTrait, ComponentRegistry
├── storage.rs          # Simplified flat storage (3 parallel Vecs)
├── world.rs            # Central World container
├── query.rs            # Query system with required/excluded filtering
├── resource.rs         # Global state (ResourceTrait)
├── components.rs       # Built-in Position component
├── archetype.rs        # Archetype concepts (entity grouping)
├── bundle.rs           # Bundle trait documentation
├── lifecycle.rs        # Entity lifecycle hooks
├── change_detection.rs # Change tick tracking
├── hierarchy.rs        # Parent-child relationships
├── system/             # System trait, SystemParam, Query parameter
├── schedule/           # Graph-based dependency resolution scheduler
├── observer/           # Observer pattern for reactive changes
├── event/              # Event system with readers/writers
├── relationship/       # Relationship management
├── storage/            # Advanced storage backends
│   ├── sparse_set.rs   # Sparse set implementation
│   ├── table/          # Table-based storage with columns
│   ├── blob_array.rs   # Binary data containers
│   └── blob_vec.rs     # Binary vector containers
├── error/              # Error types and handlers
└── reflect/            # Reflection infrastructure
```

### Core Traits

```rust
// Component identity and serialization
pub trait ComponentTrait {
    fn component_type() -> Symbol;
    fn serialize(&self, env: &Env) -> Bytes;
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> where Self: Sized;
}

// Resource (global state) management
pub trait ResourceTrait: Send + Sync + 'static {
    fn resource_type() -> Symbol;
    fn serialize(&self, env: &Env) -> Bytes;
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> where Self: Sized;
}

// System execution
pub trait System {
    type In;
    type Out;
    fn run(&mut self, world: &mut World, input: Self::In) -> Self::Out;
}

// System parameter fetching
pub trait SystemParam {
    type Param;
    type Fetch;
    fn fetch(world: &World) -> Self::Fetch;
    fn fetch_mut(world: &mut World) -> Self::Fetch;
}
```

### Built-in Types

- **`GameState`**: Score, level, is_game_over — provided as a default resource
- **`Position`**: Built-in 2D position component (x, y as i32)
- **`Entity`**: ID + generation + component type list
- **`Component`**: Type symbol + serialized bytes data

---

## 2. Storage Layer Analysis

### Current Implementation

The storage layer uses a **flat, parallel vector** design:

```rust
#[contracttype]
pub struct Storage {
    pub entity_ids: Vec<u64>,        // Which entity owns this component
    pub component_types: Vec<Symbol>, // Component type identifier
    pub component_data: Vec<Bytes>,   // Serialized component data
}
```

All three vectors are aligned by index — `entity_ids[i]`, `component_types[i]`, and `component_data[i]` form a single component record.

### Performance Characteristics

| Operation | Complexity | Description |
|-----------|------------|-------------|
| `get_component` | O(n) | Linear scan of all entries |
| `add_component` | O(1) | Append to end of vectors |
| `remove_component` | O(n) | Reconstruct all three vectors |
| `has_component` | O(n) | Linear scan |
| `query` | O(n * m) | n entries x m required components |

### Critical Issues

**1. Linear scan for every read operation**

Every `get_component` call scans the entire storage:
```rust
for i in 0..self.entity_ids.len() {
    if eid == entity_id && ctype == component_type {
        return Some(Component::new(ctype, cdata));
    }
}
```

For a game with 100 entities averaging 5 components each (500 entries), every single component access scans 500 entries.

**2. Vec reconstruction on removal**

Removing a component creates three entirely new vectors:
```rust
let mut new_entity_ids = Vec::new(&env);
let mut new_component_types = Vec::new(&env);
let mut new_component_data = Vec::new(&env);
// Copy everything except the removed entry
```

This is O(n) memory allocation per removal, and removing N components from a world with N entries results in O(n^2) total work.

**3. No indexing or caching**

- No hash maps for entity → components lookup
- No archetype grouping for query optimization
- No query result caching between frames
- Every query re-scans the entire storage

**4. Full world serialization per contract call**

The entire World (all entities, all components) must be loaded from Soroban persistent storage and deserialized at the start of every contract invocation, and re-serialized and stored at the end.

### Existing Advanced Storage (Not Integrated)

The codebase contains sophisticated storage implementations that are **not connected** to the actual Storage struct:

- **`src/storage/sparse_set.rs`**: Full sparse set implementation (O(1) lookup, O(1) insert/remove)
- **`src/storage/table/column.rs`**: Table-based storage with typed columns
- **`src/archetype.rs`**: Archetype grouping logic (entities with same component set)
- **`src/storage/blob_array.rs`**, **`blob_vec.rs`**: Binary data containers

These represent significant engineering work that could dramatically improve performance if integrated.

---

## 3. Unintegrated Infrastructure

A substantial portion of the codebase contains fully-implemented subsystems that are not connected to the core World or used in any examples.

### Scheduling System (`src/schedule/`)

A complete graph-based system scheduling framework with:
- Dependency resolution between systems
- Automatic execution ordering
- Multi-threaded executor infrastructure

**Status**: Fully implemented, never used. All examples manually call systems in sequence.

### Observer System (`src/observer/`)

A reactive pattern for responding to component changes:
- Watch for component add/remove/change events
- Execute callbacks when conditions are met
- Inspired by production ECS observer patterns

**Status**: Fully implemented, no examples demonstrate it.

### Change Detection (`src/change_detection.rs`)

Tick-based tracking of component modifications:
- Track when components were last modified
- Enable systems to only process changed entities
- Reduce unnecessary computation

**Status**: Infrastructure exists, not wired into components or World.

### Commands Module

Deferred command buffers for safe World mutation:
- Queue structural changes (add/remove entities/components) during system execution
- Apply changes after system completes
- Prevent iterator invalidation

**Status**: Module exists, not integrated into system execution flow.

### Relationship/Hierarchy (`src/hierarchy.rs`, `src/relationship/`)

Parent-child entity relationships and arbitrary relationships:
- Scene graph-like hierarchies
- Cascading operations (delete parent → delete children)
- Relationship queries

**Status**: Fully implemented, no examples use it.

### Reflection (`src/reflect/`)

Runtime type inspection and introspection:
- Query component types at runtime
- Serialize/deserialize components dynamically
- Enable tooling and debugging

**Status**: Infrastructure present, not connected to component system.

### Impact Assessment

```
Total modules:     127
Actively used:     ~20 (entity, component, storage, world, query, system basics)
Infrastructure:    ~60 (schedule, observer, change detection, reflection, etc.)
Supporting code:   ~47 (error handling, utilities, storage backends)

Utilization rate:  ~16% of codebase actively used in examples
```

---

## 4. API Surface Analysis

### Public API

The framework exposes a clean but minimal API through `src/lib.rs`:

```rust
// Entity management
create_world() -> World
spawn_entity(world, components) -> Entity
add_component(world, entity_id, component) -> ()
remove_component(world, entity_id, component_type) -> ()
get_component(world, entity_id, component_type) -> Option<Component>

// Query
Query::new()
    .with_component(symbol) -> Query
    .without_component(symbol) -> Query
    .execute(world) -> Vec<EntityId>

// World methods
world.spawn_empty() -> Entity
world.spawn(components) -> Entity
world.despawn(entity_id) -> ()
world.exists(entity_id) -> bool
world.entity_count() -> u64
world.component_count() -> u64
```

### API Gaps

**1. No typed component access**

Components are identified by `Symbol` strings, not Rust types:
```rust
// Current: string-based, error-prone
world.add_component(entity_id, symbol_short!("position"), position_data);

// Desired: type-safe
world.add_component::<Position>(entity_id, position);
```

**2. No query builder pattern**

```rust
// Current: only basic AND/NOT filtering
let query = Query::new()
    .with_component(symbol_short!("position"))
    .without_component(symbol_short!("dead"));

// Missing: complex filters, ordering, pagination
// .with_changed::<Position>()
// .with_added::<Health>()
// .order_by::<Score>(Descending)
// .limit(10)
```

**3. No ergonomic system definition**

```rust
// Current: manual trait implementation
impl System for MySystem {
    type In = ();
    type Out = ();
    fn run(&mut self, world: &mut World, _input: ()) -> () { ... }
}

// Desired: function-based systems
fn my_system(query: Query<(&Position, &mut Velocity)>, time: Res<GameTime>) {
    for (pos, vel) in query.iter_mut() { ... }
}
```

**4. No derive macros**

Every component requires manual `ComponentTrait` implementation with hand-written serialization:
```rust
// 30+ lines of boilerplate per component
impl ComponentTrait for PaddleComponent {
    fn component_type() -> Symbol { symbol_short!("paddle") }
    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.player_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y_position.to_be_bytes()));
        bytes
    }
    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 8 { return None; }
        // ... manual byte parsing
    }
}

// Desired
#[derive(Component)]
pub struct PaddleComponent {
    pub player_id: u32,
    pub y_position: i32,
}
```

---

## 5. Critical Improvement Opportunities

### 5.1 Type Safety

**Current**: Components identified by `Symbol` strings with no compile-time verification.

**Problem**: Easy to make typos or type mismatches that only fail at runtime:
```rust
// These won't be caught at compile time:
world.add_component(id, symbol_short!("positon"), data);  // Typo
world.add_component(id, symbol_short!("position"), wrong_data);  // Type mismatch
```

**Improvement**: Type-parameterized component access with compile-time checks.

### 5.2 Serialization Boilerplate

**Current**: Every component requires ~30 lines of manual serialization code with hardcoded byte offsets and length checks.

**Problem**:
- Error-prone (wrong byte count = silent corruption)
- Repetitive across all 11 examples
- Brittle (changing a field requires updating multiple hardcoded values)

**Improvement**: Derive macro that generates serialization automatically from struct fields.

### 5.3 Error Handling

**Current**: Pervasive use of `.unwrap()` throughout both core library and examples.

**Problem**: Soroban will trap on panic, causing transaction failure with no useful error message. This costs real gas fees in production.

```rust
// Current pattern (throughout codebase)
let game_state: GameState = env
    .storage()
    .persistent()
    .get(&symbol_short!("state"))
    .unwrap();  // Panics with no context
```

**Improvement**: Custom error types with `Result`-based API:
```rust
pub enum CougrError {
    EntityNotFound(EntityId),
    ComponentNotFound(EntityId, Symbol),
    SerializationError(String),
    StorageError(String),
    QueryError(String),
}
```

### 5.4 SimpleWorld Duplication

**Current**: Every example (all 11 games) reimplements its own `SimpleWorld` struct with slight variations.

**Problem**:
- Massive code duplication
- Bug fixes must be applied to 11 copies
- Inconsistent implementations across examples
- New examples must copy-paste boilerplate

**Improvement**: Provide `SimpleWorld` as a core library type or example template that all games import.

### 5.5 Performance

**Current**: O(n) linear scans for every component access, O(n) Vec reconstruction for removals.

**Problem**: Performance degrades quadratically with entity count. A game with 200+ entities will have noticeable gas costs from storage operations alone.

**Improvement**: Integrate existing archetype-based storage and sparse set implementations.

---

## 6. Testing & Quality Assessment

### Current Test Coverage

| Area | Test Count | Quality | Coverage |
|------|-----------|---------|----------|
| Core library (`src/lib.rs`) | 2 | Basic | ~5% |
| Pong example | 19 | Excellent | ~90% |
| Snake example | Present | Good | ~70% |
| Tetris example | Present | Good | ~70% |
| Other examples | Present | Varies | ~50-70% |
| Storage layer | 0 | None | 0% |
| Query system | 0 | None | 0% |
| Event system | 0 | None | 0% |
| Observer system | 0 | None | 0% |
| Scheduling system | 0 | None | 0% |
| Resource system | 0 | None | 0% |

### Overall Coverage Estimate: ~15-20%

### Core Library Testing Gaps

The 2 core library tests cover only:
1. World creation (can create an empty world)
2. Entity spawning (can spawn an entity)

**Not tested:**
- Component add/remove/get operations
- Storage operations under load
- Query execution with various filter combinations
- Entity despawning and cleanup
- Resource management
- Event publishing and consuming
- System execution
- Edge cases (duplicate components, invalid entity IDs, empty queries)

### Example Testing Quality

The **Pong example** stands out as the testing reference with 19 comprehensive tests covering:
- Game initialization state
- Paddle movement (up, down, boundary clamping)
- Ball physics (movement, wall bouncing, velocity)
- Scoring mechanics (player 1, player 2)
- Win condition detection
- Collision detection (paddle-ball)
- Game state transitions (active → game over)
- Score reset on initialization

### Recommendations

1. **Unit tests for storage layer**: get, add, remove, has under various conditions
2. **Query tests**: single filter, multiple filters, exclusion, empty results
3. **Stress tests**: Performance with 100, 500, 1000+ entities
4. **Edge case tests**: Invalid entity IDs, duplicate components, empty world operations
5. **Integration tests**: Full game loop simulation with multiple systems
6. **Regression tests**: For each bug fix, add a test case

---

## 7. Example Games Analysis

### Quality Assessment

| Example | Architecture | Testing | Documentation | Complexity |
|---------|-------------|---------|---------------|------------|
| Pong | Excellent | 19 tests | Well-documented | Medium |
| Snake | Good | Present | Good docs | Medium |
| Flappy Bird | Good | Present | Adequate | Medium |
| Tetris | Excellent | Present | Good | High |
| Pokemon Mini | Good | Present | Adequate | High |
| Space Invaders | Adequate | Present | Minimal | Medium |
| Pac-Man | Adequate | Present | Minimal | High |
| Arkanoid | Adequate | Present | Minimal | Medium |
| Tic-Tac-Toe | Good | Present | Adequate | Low |
| Bomberman | Adequate | Present | Minimal | Medium |
| Asteroids | Adequate | Present | Minimal | Medium |

### Common Pattern Across All Examples

Every example follows the same architecture:

1. **components.rs**: Define game-specific components implementing `ComponentTrait`
2. **systems.rs**: Implement game logic as standalone functions
3. **simple_world.rs**: Re-implement the SimpleWorld storage abstraction
4. **lib.rs**: Contract entry points calling systems in sequence

```rust
// Typical game loop pattern
pub fn update_tick(env: Env) {
    let mut game_state: GameState = env.storage().persistent().get(...).unwrap();
    let mut world: SimpleWorld = env.storage().persistent().get(...).unwrap();

    // Manually call systems in order
    systems::apply_physics(&mut world, &env);
    systems::check_collisions(&mut world, &env);
    systems::update_scoring(&mut world, &env, &mut game_state);

    // Save state
    env.storage().persistent().set(&symbol_short!("state"), &game_state);
    env.storage().persistent().set(&symbol_short!("world"), &world);
}
```

### Observations

1. **No example uses the core `World`** — all use a simplified `SimpleWorld` re-implementation
2. **Systems are functions, not trait implementations** — the `System` trait is never used in examples
3. **No example uses scheduling** — systems called manually in fixed order
4. **No example uses events** — state changes through direct mutation
5. **No example uses observers** — no reactive patterns demonstrated
6. **No example uses resources** — `GameState` stored in Soroban storage, not ECS resources

This suggests the core library API may not be ergonomic enough for practical use, leading developers to bypass it entirely.

---

## 8. Missing Capabilities

The following capabilities are absent from Cougr and would significantly enhance the framework. These represent patterns that are standard in mature ECS frameworks and essential for production-quality on-chain gaming.

### Data Storage & Access

- **Archetype-based storage**: Group entities by their component composition into tables for O(1) archetype-matched queries and cache-friendly iteration
- **Sparse set storage**: Store infrequently-used components separately from dense components, reducing memory waste for optional components
- **Query caching**: Cache query results between frames to avoid re-scanning storage every tick
- **Advanced query filters**: Support for changed-since, added-since, removed-since filters; OR logic; component value filters; ordering; pagination
- **Indexed queries**: O(1) component lookup by entity ID via hash maps instead of linear scans
- **Incremental state updates**: Track dirty entities and only serialize/deserialize changed data instead of the entire world

### System Execution

- **Automatic system scheduling**: Build a dependency graph of systems and execute them in valid order automatically, rather than requiring manual ordering
- **System dependency resolution**: Detect read/write conflicts between systems and schedule non-conflicting systems for concurrent execution
- **System sets and stages**: Group related systems into named stages (e.g., PreUpdate, Update, PostUpdate) for clear execution phases
- **Run conditions**: Conditionally execute systems based on game state (e.g., only run physics when game is active)
- **One-shot systems**: Systems that run exactly once (initialization, cleanup) with automatic removal

### Component Lifecycle

- **Component hooks**: `on_add`, `on_insert`, `on_remove` callbacks when components are added, modified, or removed from entities
- **Reactive observers**: Systems that trigger immediately when specific components change, rather than polling
- **Change detection**: Track which components changed since the last tick, allowing systems to skip unchanged entities
- **Deferred commands / command buffers**: Queue structural changes (entity creation, component removal) during system iteration and apply them safely after the system completes

### Entity Management

- **Entity relationships**: First-class parent-child relationships with cascading operations (despawn parent → despawn children)
- **Component bundles**: Define groups of components that are always added together atomically (e.g., a "PhysicsBundle" = Position + Velocity + Collider)
- **Entity archetypes/prefabs**: Pre-defined entity templates for common game objects (player, enemy, bullet)

### Developer Experience

- **Derive macros**: `#[derive(Component)]`, `#[derive(Bundle)]`, `#[derive(Resource)]` to eliminate serialization boilerplate
- **Function-based systems**: Define systems as plain functions with typed parameters instead of implementing traits
- **Plugin/extension system**: Modular feature sets that can be composed (PhysicsPlugin, NetworkPlugin, ZKPlugin)
- **Custom error types**: Descriptive errors with context instead of panics
- **Result-based API**: All fallible operations return `Result` instead of `Option` or panicking

### On-Chain Optimization

- **Gas-aware scheduling**: Estimate gas cost of systems and prioritize critical ones within budget
- **Lazy loading**: Load only the entities/components needed for the current transaction instead of the entire world
- **State compression**: Compact serialization format for on-chain storage to minimize storage costs
- **Delta encoding**: Store only changes since the last state, reducing storage writes

### Zero-Knowledge Proofs

- **Cryptographic primitives**: Ergonomic Rust wrappers around Stellar Protocol 25 host functions (BN254, Poseidon/Poseidon2, Groth16)
- **ZK components**: First-class ECS components for hidden state (commitments), proof submissions, and verified markers
- **ZK systems**: Verification system integrated into the game loop — batch verify proofs per tick, enforce commit-reveal deadlines
- **Pre-built game circuits**: Movement validation, combat resolution, inventory verification, turn sequencing — common patterns ready to use
- **Merkle tree utilities**: Poseidon-based Merkle trees for fog of war, inventory proofs, and large state space commitments
- **ZK testing utilities**: Mock proofs, mock circuits, and test verification keys for unit testing without actual proof generation

### Account Abstraction

- **Unified account trait**: Single API (`CougrAccount`) abstracting over Classic (G-address) and Contract (C-address) Stellar accounts
- **Session key management**: Scoped temporary credentials for seamless gameplay without wallet popups
- **Batch transactions**: Compose multiple game actions into one atomic operation
- **Social recovery**: Guardian-based recovery with time-locks and cancellation windows
- **Multi-device support**: Per-device key registration, policies, and revocation
- **Capability detection**: Runtime detection of connected account features with graceful degradation
- **Gasless gameplay**: Native Stellar fee sponsorship integration for subsidized game sessions

### Tooling

- **World inspector**: Debug tool to visualize entities, components, and relationships
- **Performance profiler**: Track gas costs per system, identify bottlenecks
- **State diff viewer**: Compare world states between ticks
- **Test utilities**: Helper functions for common test patterns (create test world with entities, simulate N ticks, assert component values)

---

## 9. Performance Analysis

### Current Bottlenecks

#### 1. Storage Access: O(n) per operation

Every component read, write, check, or query performs a linear scan. For a game with E entities and C average components per entity:

- Single `get_component`: O(E * C)
- Single `query` with K filters: O(E * C * K)
- One game tick with S systems, each querying Q components: O(S * Q * E * C)

**Example**: 50 entities, 5 components each, 5 systems, 2 queries each:
```
Per tick: 5 * 2 * 50 * 5 = 2,500 comparisons
```

This grows quadratically with game complexity.

#### 2. Memory Allocation: O(n) per component removal

Removing a single component reconstructs three entire vectors:
```
50 entities * 5 components = 250 entries
Remove 1 component = copy 249 entries to 3 new vectors
Remove 10 components = 10 * ~250 = 2,500 copy operations
```

#### 3. World Serialization: O(total_state) per contract call

Every contract invocation:
1. Deserialize entire world from Soroban storage
2. Execute game logic
3. Serialize entire world back to Soroban storage

For a game with 100 entities, 5 components each, 20 bytes per component:
```
100 * 5 * 20 = 10,000 bytes deserialized AND serialized per call
```

This overhead is constant regardless of how many entities are actually accessed.

#### 4. No Query Caching

If a system queries "all entities with Position and Velocity" on tick 1, and the same query runs on tick 2 with no changes, the entire storage is still scanned.

### Positive Performance Patterns

The codebase does employ several optimization strategies:

```toml
[profile.release]
lto = true               # Link-time optimization
codegen-units = 1         # Single codegen unit for better optimization
opt-level = "z"           # Aggressive size optimization
overflow-checks = true    # Required for Soroban
```

- **wee_alloc**: Custom allocator for WASM memory efficiency
- **#![no_std]**: Avoids standard library overhead
- **Minimal dependencies**: Only soroban-sdk + wee_alloc

### Improvement Impact Estimates

| Optimization | Current | After | Impact |
|-------------|---------|-------|--------|
| Indexed lookup | O(n) | O(1) | 50-100x for large worlds |
| Archetype queries | O(n*m) | O(matching) | 5-20x depending on selectivity |
| Incremental serialization | O(total) | O(changed) | 2-10x per call |
| In-place removal | O(n) copy | O(1) swap | 100x for removals |
| Query caching | O(n) per query | O(1) cached | 10-50x for repeated queries |

---

## 10. Proposed Improvement Roadmap

### Phase 1: Foundation (Immediate Priority)

**Goal**: Make the framework usable for production games without workarounds.

1. **Provide `SimpleWorld` as a core library type**
   - Eliminate copy-paste across 11 examples
   - Standardize the simplified storage pattern for Soroban
   - Files: `src/simple_world.rs`, update all example imports

2. **Add derive macros for Component**
   - `#[derive(CougrComponent)]` generates `ComponentTrait` implementation
   - Automatic serialization/deserialization
   - Files: `src/macros.rs` (proc macros within the single `cougr-core` crate)

3. **Implement custom error types**
   - Replace `.unwrap()` calls with `Result<T, CougrError>`
   - Descriptive error variants for all failure modes
   - Files: `src/error.rs`, update `src/world.rs`, `src/storage.rs`

4. **Add indexed component lookup to SimpleWorld**
   - Hash map from `(entity_id, component_type)` → index
   - Upgrade get/has from O(n) to O(1)
   - Files: `src/simple_world.rs` or `src/storage.rs`

### Phase 2: Core ECS (Short-term)

**Goal**: Activate the existing infrastructure and connect it to the World.

5. **Integrate system scheduling**
   - Connect `src/schedule/` to World
   - Allow `world.add_system(system)` and `world.run_schedule()`
   - Files: `src/world.rs`, `src/schedule/mod.rs`

6. **Integrate query caching**
   - Cache query results, invalidate on component add/remove
   - Files: `src/query.rs`, `src/world.rs`

7. **Implement sparse set storage**
   - Connect `src/storage/sparse_set.rs` to component storage
   - Use for infrequent components (markers, tags)
   - Files: `src/storage.rs`, `src/storage/sparse_set.rs`

8. **Add component lifecycle hooks**
   - `on_add`, `on_remove` callbacks
   - Enable reactive patterns
   - Files: `src/component.rs`, `src/lifecycle.rs`

### Phase 3: Advanced Features (Medium-term)

**Goal**: Mature the framework for complex game development.

9. **Implement change detection**
   - Wire `src/change_detection.rs` into components
   - Enable `Changed<T>` and `Added<T>` query filters
   - Files: `src/change_detection.rs`, `src/query.rs`

10. **Integrate observer system**
    - Connect `src/observer/` to World
    - Demonstrate in examples
    - Files: `src/world.rs`, `src/observer/mod.rs`

11. **Implement plugin/extension system**
    - Modular feature registration within the single crate (e.g., `world.add_plugin(zk_plugin())`)
    - Internal modules (`src/zk/`, `src/accounts/`) registered as plugins
    - Files: new `src/plugin.rs`

12. **Add deferred commands**
    - Command buffers for safe structural changes during iteration
    - Files: `src/commands.rs`, `src/world.rs`

### Phase 4: Optimization & Tooling (Long-term)

**Goal**: Production-ready performance and developer experience.

13. **Implement archetype-based storage**
    - Full integration of `src/archetype.rs` with storage layer
    - Table-based component storage grouped by archetype
    - Files: `src/archetype.rs`, `src/storage.rs`, `src/storage/table/`

14. **Incremental state serialization**
    - Only serialize/deserialize changed entities per contract call
    - Dirty tracking at entity level
    - Files: `src/storage.rs`, `src/world.rs`

15. **World inspector / debugging tools**
    - Print world state, entity counts, component distributions
    - Gas cost estimation per system
    - Files: new `src/debug.rs`

16. **Comprehensive test suite**
    - Unit tests for all core modules
    - Integration tests for system scheduling
    - Performance benchmarks
    - Stress tests for storage under load
    - Files: `src/tests/`, `benches/`

### Phase 5: Native ZK & Account Abstraction (Parallel Track)

**Goal**: Make Cougr the first ECS game engine with built-in zero-knowledge proofs and smart account support — all within the single `cougr-core` crate.

17. **Integrate ZK module (`src/zk/`)**
    - ~25 new source files: crypto wrappers (BN254, BLS12-381, Poseidon/Poseidon2, Groth16), ZK components (HiddenState, Commitment, ProofRequired), ZK systems (verification, commit-reveal, hidden state updates), pre-built game circuits (movement, combat, inventory, turn), Merkle tree utilities, testing mocks
    - Leverages Stellar Protocol 25 X-Ray host functions natively
    - Files: `src/zk/**`
    - See: `research/zk-integration.md` for full architecture

18. **Integrate Account Abstraction module (`src/accounts/`)**
    - ~20 new source files: core traits (CougrAccount, SessionKeyProvider, RecoveryProvider), dual account implementations (Classic G-address + Contract C-address), session key management with scoping, batch transaction builder, social recovery with time-locked guardians, multi-device key management, capability detection, graceful degradation, testing mocks
    - Leverages Stellar native `__check_auth` and fee sponsorship
    - Files: `src/accounts/**`
    - See: `research/smart-accounts.md` for full architecture

19. **Wire ZK and Accounts into the World**
    - `world.submit_proof()`, `world.execute_authorized()`, `world.start_session()`
    - ZK verification as a first-class system in the game loop
    - Account authorization integrated with system execution
    - Files: `src/world.rs`, `src/zk/mod.rs`, `src/accounts/mod.rs`

### Crate Growth Projection

The roadmap above represents a **significant expansion** of `cougr-core` from a single crate:

```
Current state:        ~127 modules across 16 directories
After Phases 1-4:     ~145 modules (integration + new core features)
After Phase 5 (ZK):   ~170 modules (+25 ZK source files)
After Phase 5 (Accts): ~190 modules (+20 account source files)
─────────────────────────────────────────────────────────
Total projected:      ~190 modules — a ~50% growth in crate size
```

All of this ships as a single `cougr-core` dependency. A game developer adds one line to `Cargo.toml` and gets: ECS engine + ZK proofs + smart accounts + session keys + batch transactions + social recovery.

### Priority Matrix

| Improvement | Impact | Effort | Priority |
|------------|--------|--------|----------|
| SimpleWorld as core type | High | Low | P0 |
| Derive macros | High | Medium | P0 |
| Error types | High | Low | P0 |
| Indexed lookup | High | Low | P0 |
| System scheduling | Medium | Medium | P1 |
| Query caching | Medium | Medium | P1 |
| Sparse set storage | Medium | Medium | P1 |
| Component hooks | Medium | Low | P1 |
| **ZK crypto wrappers** | **High** | **Medium** | **P1** |
| **Account core traits** | **High** | **Medium** | **P1** |
| Change detection | Medium | Medium | P2 |
| Observer integration | Medium | Medium | P2 |
| Plugin system | Medium | High | P2 |
| Deferred commands | Medium | Medium | P2 |
| **ZK components & systems** | **High** | **High** | **P2** |
| **Session keys & batching** | **High** | **High** | **P2** |
| **Pre-built game circuits** | **High** | **High** | **P2** |
| **Social recovery & multi-device** | **Medium** | **High** | **P2** |
| Archetype storage | High | High | P3 |
| Incremental serialization | High | High | P3 |
| **Merkle tree utilities** | **Medium** | **Medium** | **P3** |
| Debug tooling | Low | Medium | P3 |
| Comprehensive tests | High | High | P3 |

---

## Summary

Cougr is an ambitious ECS framework with a solid architectural foundation and impressive scope (127 modules, 11 example games). Its key strength is being purpose-built for Stellar/Soroban's constraints (no_std, WASM, on-chain state).

**Current strengths:**
- Clean module organization and clear separation of concerns
- Comprehensive infrastructure already built (scheduling, observers, change detection)
- 11 diverse game examples demonstrating real-world usage
- Proper Soroban integration with correct build profiles

**Primary gaps:**
- Only ~16% of the codebase is actively used — sophisticated infrastructure exists but isn't connected
- Storage layer performance is O(n) for all operations
- No derive macros, requiring ~30 lines of boilerplate per component
- Pervasive `.unwrap()` instead of proper error handling
- Each example re-implements `SimpleWorld` instead of importing from core
- No native ZK proof support despite Stellar Protocol 25 enabling it
- No account abstraction layer despite Stellar's native `__check_auth` support

**Overall quality score: 6.5/10** — Strong architecture with significant integration and polish work remaining before production readiness.

**Planned growth**: The single `cougr-core` crate is projected to grow from ~127 modules to ~190 modules (~50% increase) through integration of existing infrastructure, a complete ZK proof layer (`src/zk/` — ~25 files), and a full account abstraction layer (`src/accounts/` — ~20 files). All of this ships as one dependency, keeping the developer experience simple while making Cougr a comprehensive on-chain gaming framework.

The most impactful immediate improvements are: providing `SimpleWorld` as a core type (eliminates duplication), adding derive macros (eliminates boilerplate), implementing indexed lookup (eliminates O(n) scans), and adding error types (eliminates panics). These four changes would elevate the framework from "proof of concept" to "developer-ready." Following that, the ZK and account abstraction modules will differentiate Cougr as a uniquely capable engine for on-chain gaming on Stellar.
