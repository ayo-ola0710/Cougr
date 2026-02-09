<p align="center">
  <img src="public/Cougr.png" width="120" alt="Cougr logo" />
</p>

<h1 align="center">Cougr</h1>

<p align="center">
  <strong>ECS framework for on-chain games on Stellar</strong>
</p>

<p align="center">
  <a href="https://stellar.org"><img src="https://img.shields.io/badge/Stellar-Soroban-blue?logo=stellar" alt="Stellar" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust" alt="Rust" /></a>
  <img src="https://img.shields.io/badge/tests-391%20passing-brightgreen" alt="Tests" />
  <img src="https://img.shields.io/badge/WASM-~14KB-purple" alt="WASM size" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="License" /></a>
</p>

---

Cougr is a Rust ECS (Entity Component System) framework built on [soroban-sdk](https://soroban.stellar.org/) for building fully on-chain games on the Stellar blockchain. It compiles to a ~14 KB WASM contract and provides everything you need: ECS, zero-knowledge proofs, smart account abstraction, and 11 ready-to-deploy game examples.

## Quick Start

```toml
# Cargo.toml
[dependencies]
cougr-core = { git = "https://github.com/salazarsebas/Cougr.git", branch = "main" }
```

```rust
use cougr_core::simple_world::SimpleWorld;
use cougr_core::component::Position;
use soroban_sdk::Env;

let env = Env::default();
let mut world = SimpleWorld::new(&env);

// Spawn entities and attach components
let player = world.spawn_entity();
world.set_typed(&env, player, &Position::new(0, 10));

// Read them back, fully typed
let pos: Position = world.get_typed(&env, player).unwrap();
```

## Features

### ECS Engine

| Capability | Description |
|---|---|
| **SimpleWorld** | Map-based ECS with O(log n) lookups, dual Table/Sparse storage |
| **ArchetypeWorld** | Entities grouped by component composition for batch queries |
| **GameWorld** | Unified wrapper: ECS + accounts + ZK proofs |
| **Typed API** | `get_typed<T>`, `set_typed<T>`, `has_typed<T>`, `remove_typed<T>` |
| **Query caching** | World-versioned cache that invalidates on mutation |
| **Hooks & Observers** | React to component add/remove/change events |
| **Deferred commands** | Buffer mutations during system execution, flush later |
| **Plugin system** | Modular game logic bundles |
| **Scheduler** | Priority-based system ordering with dependencies |
| **Change detection** | Track which components changed per tick |
| **Incremental serialization** | Only persist dirty entities to storage |

### Zero-Knowledge Proofs

| Capability | Description |
|---|---|
| **Groth16 verification** | On-chain proof verification via BN254 pairing |
| **BLS12-381** | Point addition, scalar multiplication, MSM, pairing checks |
| **Poseidon2** | ZK-friendly hashing (~300 constraints vs 28K for SHA256) |
| **Merkle trees** | SHA256 + Poseidon variants, sparse Merkle trees, on-chain proofs |
| **Pedersen commitments** | Commit-reveal schemes for hidden game state |
| **Game circuits** | Pre-built: Movement, Combat, Inventory, TurnSequence |
| **Custom circuits** | `CustomCircuitBuilder` for defining your own |
| **Commit-reveal system** | ECS components + systems for fog-of-war patterns |

### Smart Accounts

| Capability | Description |
|---|---|
| **Account abstraction** | `CougrAccount` trait with Classic and Contract implementations |
| **Session keys** | Scoped, time-limited keys for gasless gameplay |
| **SessionBuilder** | Fluent API for session key construction |
| **Social recovery** | Guardian-based account recovery with configurable thresholds |
| **Multi-device** | Multiple signing keys per account with device policies |
| **WebAuthn / Passkeys** | secp256r1 signature verification for biometric auth |
| **Graceful degradation** | Automatic fallback from session key to direct auth |
| **Batch authorization** | Authorize multiple game actions in one call |

### Macros

Define components without boilerplate:

```rust
use cougr_core::impl_component;

#[contracttype]
#[derive(Clone)]
pub struct Health {
    pub current: u128,
    pub max: u128,
}

// Generates ComponentTrait: serialize, deserialize, component_type, storage
impl_component!(Health, "health", Table, { current: u128, max: u128 });
```

Supported types: `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `u8`, `bool`, `bytes32` (BytesN<32>).

## Examples

11 fully-implemented game contracts, each a standalone Soroban project:

| Game | Description |
|---|---|
| [Arkanoid](examples/arkanoid) | Brick-breaking arcade |
| [Asteroids](examples/asteroids) | Space shooter |
| [Bomberman](examples/bomberman) | Grid-based bomber |
| [Flappy Bird](examples/flappy_bird) | Side-scrolling obstacle game |
| [Pac-Man](examples/pac_man) | Maze chase |
| [Pokemon Mini](examples/pokemon_mini) | Turn-based battle |
| [Pong](examples/pong) | Classic paddle game |
| [Snake](examples/snake) | Growing snake |
| [Space Invaders](examples/space_invaders) | Alien wave shooter |
| [Tetris](examples/tetris) | Block stacking |
| [Tic-Tac-Toe](examples/tic_tac_toe) | Two-player grid |

Each example compiles to its own WASM contract with `stellar contract build`.

## Architecture

```
src/
  simple_world.rs       Core ECS (entities, components, queries)
  archetype_world/      Archetype-grouped storage + cached queries
  game_world.rs         ECS + accounts + ZK integration wrapper
  component.rs          ComponentTrait, registry, Position, Velocity
  macros.rs             impl_component!, impl_marker_component!, impl_resource!
  hooks.rs              Component lifecycle hooks
  observers.rs          Event-driven observers
  commands.rs           Deferred command buffer
  scheduler.rs          System scheduling with priorities
  change_tracker.rs     Per-component change detection
  plugin.rs             Modular plugin system
  incremental/          Dirty-tracking persistent storage
  zk/
    groth16.rs          Groth16 proof verification
    crypto.rs           BN254 + Poseidon2 wrappers
    bls12_381.rs        BLS12-381 operations
    commitment.rs       Pedersen commitments
    merkle/             SHA256 + Poseidon Merkle trees
    circuits.rs         Pre-built game circuits
    traits.rs           GameCircuit trait
    components.rs       ZK ECS components (CommitReveal, HiddenState, etc.)
    systems.rs          Proof verification systems
  accounts/
    traits.rs           CougrAccount, SessionKeyProvider, RecoveryProvider
    classic.rs          Classic Stellar account
    contract.rs         Contract-based smart account
    session_builder.rs  Fluent session key builder
    recovery.rs         Social recovery
    multi_device.rs     Multi-device key management
    secp256r1_auth.rs   WebAuthn / Passkey support
    degradation.rs      Graceful auth fallback
  debug/                Introspection, metrics, snapshots (behind `debug` feature)
```

## Feature Flags

| Flag | Purpose |
|---|---|
| `hazmat-crypto` | Enables Poseidon2, BN254 curve operations |
| `testutils` | Test helpers and mock accounts |
| `debug` | Runtime introspection, metrics, and state snapshots |

## Development

```bash
cargo test                           # 391 tests
cargo test --features debug          # + debug tooling tests
cargo clippy                         # Lint
cargo fmt --check                    # Format check
stellar contract build               # Build WASM (~14 KB)
```

## Compatibility

- **Rust** 1.70+, Edition 2021
- **soroban-sdk** 25.1.0
- **Targets**: `wasm32-unknown-unknown`, `wasm32v1-none`
- **no_std** + `wee_alloc`

## License

Licensed under [MIT](LICENSE).

## Links

- [Soroban Docs](https://soroban.stellar.org/)
- [Stellar Developers](https://developers.stellar.org/)
