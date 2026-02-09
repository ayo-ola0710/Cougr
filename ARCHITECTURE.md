# Architecture

High-level overview of how Cougr is organized. For usage, see [README.md](README.md).

## Layers

```
┌─────────────────────────────────────────────┐
│              GameWorld                       │  Unified API: ECS + Auth + ZK
├───────────┬───────────────┬─────────────────┤
│  ECS      │  Accounts     │  ZK Proofs      │
├───────────┼───────────────┼─────────────────┤
│  soroban-sdk 25.1.0  (no_std, WASM)         │
└─────────────────────────────────────────────┘
```

**GameWorld** (`src/game_world.rs`) is the top-level integration layer. It wraps a `SimpleWorld` (ECS), a `CougrAccount` (auth), and provides ZK proof submission — one struct for a complete game contract.

## ECS

Two storage backends, same `ComponentTrait` interface:

| Backend | File | Strategy | Best for |
|---|---|---|---|
| **SimpleWorld** | `src/simple_world.rs` | `Map<EntityId, Map<Symbol, Bytes>>` with dual Table/Sparse | General use, small entity counts |
| **ArchetypeWorld** | `src/archetype_world/` | Groups entities by component signature | Large entity counts, batch queries |

Both support typed access (`get_typed<T>`, `set_typed<T>`) and raw access (`get_component`, `add_component`).

Supporting systems:

- **Query cache** (`src/query.rs`) — version-tagged, invalidates on world mutation
- **Hooks** (`src/hooks.rs`) — callbacks on component add/remove
- **Observers** (`src/observers.rs`) — event-driven reactions
- **Commands** (`src/commands.rs`) — deferred mutations during system execution
- **Scheduler** (`src/scheduler.rs`) — priority-based system ordering
- **Change tracker** (`src/change_tracker.rs`) — per-component dirty flags
- **Plugins** (`src/plugin.rs`) — modular game logic bundles
- **Incremental storage** (`src/incremental/`) — only persist dirty entities

### Component definition

The `impl_component!` macro generates `ComponentTrait` (serialize/deserialize/type symbol) from a struct definition. Supported field types: `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `u8`, `bool`, `bytes32`.

## ZK Proofs (`src/zk/`)

All ZK operations use Stellar Protocol 25 (X-Ray) host functions — the heavy crypto runs on the host, not in WASM.

- **Groth16** (`groth16.rs`) — proof verification via BN254 pairing
- **BLS12-381** (`bls12_381.rs`) — G1 add/mul/MSM, pairing checks
- **Poseidon2** (`crypto.rs`) — ZK-friendly hashing, behind `hazmat-crypto` feature
- **Merkle trees** (`merkle/`) — SHA256 and Poseidon variants, sparse trees, on-chain proofs
- **Pedersen** (`commitment.rs`) — commitment scheme for hidden state
- **Game circuits** (`circuits.rs`, `traits.rs`) — `GameCircuit` trait + pre-built circuits (Movement, Combat, Inventory, TurnSequence) + `CustomCircuitBuilder`
- **ECS integration** (`components.rs`, `systems.rs`) — `CommitReveal`, `HiddenState`, `ProofSubmission` components with verification systems

## Accounts (`src/accounts/`)

Account abstraction layer with pluggable implementations:

```
CougrAccount (trait)
├── ClassicAccount      — standard Stellar keypair
└── ContractAccount     — smart contract wallet
     ├── SessionStorage — persistent session keys
     ├── RecoveryStorage — guardian-based recovery
     ├── DeviceStorage  — multi-device key management
     └── Secp256r1Storage — WebAuthn/Passkey keys
```

Key traits: `CougrAccount`, `SessionKeyProvider`, `RecoveryProvider`, `MultiDeviceProvider`.

`SessionBuilder` provides a fluent API for constructing scoped session keys. `authorize_with_fallback` handles graceful degradation from session keys to direct authorization.

## Feature Flags

| Flag | Enables |
|---|---|
| `hazmat-crypto` | Poseidon2 hash, BN254 curve ops (via `soroban-sdk/hazmat-crypto`) |
| `testutils` | `MockAccount`, test helpers (via `soroban-sdk/testutils`) |
| `debug` | Runtime introspection, metrics, state snapshots (`src/debug/`) |

## Build

The contract compiles to ~14 KB WASM with LTO, `opt-level = "z"`, and `overflow-checks = true`.

Target: `wasm32v1-none` (stellar-cli) or `wasm32-unknown-unknown`.
