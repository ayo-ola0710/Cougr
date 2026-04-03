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
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="License" /></a>
</p>

Cougr is a Rust framework for building on-chain games on Stellar with an Entity Component System (ECS) architecture. It combines ECS primitives, zero-knowledge tooling, and account abstraction patterns into a single codebase designed for Soroban-compatible applications.

The repository includes the core library, a growing catalog of standalone game examples, and focused research notes for protocol and architecture work. The goal is to provide a practical foundation for teams building game logic that must remain structured, testable, and efficient under blockchain constraints.

## Project Status

Cougr is still pre-`1.0`. The library has strong test coverage, but not every subsystem should be treated as equally mature.

Current maturity baseline:

| Area | Current status |
|---|---|
| ECS runtime and storage | Beta |
| Accounts and smart-account patterns | Beta |
| Privacy primitives | Beta |
| Advanced ZK verification and confidential abstractions | Experimental |

## What Cougr Provides

| Area | What it includes |
|---|---|
| ECS | Typed components, multiple world implementations, scheduling, deferred commands, hooks, observers, and change tracking |
| Zero-knowledge tooling | Groth16 verification, curve helpers, commitments, Merkle structures, reusable circuits, and ECS-integrated proof flows |
| Smart account patterns | Session keys, social recovery, multi-device authorization, and fallback authorization flows |
| Example contracts | 20+ example game projects and growing, each intended to show concrete patterns rather than isolated snippets |

## Quick Start

```toml
[dependencies]
cougr-core = { git = "https://github.com/salazarsebas/Cougr.git", branch = "main" }
```

```rust
use cougr_core::component::Position;
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::Env;

let env = Env::default();
let mut world = SimpleWorld::new(&env);

let player = world.spawn_entity();
world.set_typed(&env, player, &Position::new(0, 10));

let pos: Position = world.get_typed(&env, player).unwrap();
```

## Core Capabilities

### ECS Runtime

| Capability | Description |
|---|---|
| `SimpleWorld` | General-purpose ECS storage with typed and raw component access |
| `ArchetypeWorld` | Archetype-oriented storage for broader query patterns and batch operations |
| Query support | Query builders, filters, iteration utilities, and mutation-aware access patterns |
| Runtime systems | Scheduling, deferred command buffers, change detection, lifecycle hooks, and observers |
| Storage model | Incremental persistence support and utilities for Soroban-constrained environments |

### Zero-Knowledge and Hidden State

| Capability | Description |
|---|---|
| Proof verification | Groth16 verification and supporting curve operations for on-chain validation |
| Commit-reveal flows | Components and systems for hidden state, commitments, and proof submissions |
| Merkle utilities | SHA256 and Poseidon-oriented tree utilities, including sparse variants |
| Circuit building | Reusable game circuits and custom circuit construction helpers |

### Accounts and Authorization

| Capability | Description |
|---|---|
| Account abstraction | Shared traits for classic and contract-based authorization models |
| Session keys | Scoped authorization for repetitive gameplay actions |
| Recovery flows | Guardian-based recovery mechanisms for safer account management |
| Multi-device usage | Per-device policies and device-scoped access control |
| Fallback authorization | Graceful degradation from advanced auth flows to direct authorization |

## Example Projects

The `examples/` directory contains 20+ standalone game contracts and is expected to keep growing. Examples are useful both as runnable references and as design patterns for structuring new projects on top of Cougr.

| Category | Representative examples | Primary focus |
|---|---|---|
| Arcade and action | `pong`, `snake`, `flappy_bird`, `space_invaders`, `asteroids` | Core ECS gameplay loops and state updates |
| Board and strategy | `tic_tac_toe`, `chess`, `battleship` | Turn management, hidden information, and deterministic rules |
| Puzzle and progression | `tetris`, `geometry_dash`, `dungeon_crawler` | Stateful progression and constrained execution flows |
| Experimental account patterns | `guild_arena`, `rock_paper_scissors` | Recovery, multi-device usage, and commit-reveal mechanics |

For the current catalog, see [examples/README.md](examples/README.md).

## Repository Layout

| Path | Purpose |
|---|---|
| `src/` | Core framework implementation |
| `tests/` | Integration, edge-case, and stress coverage |
| `benches/` | Benchmark targets |
| `examples/` | Standalone example game contracts |
| `research/` | Design and exploration notes |
| `.github/workflows/` | CI workflows for the library and selected examples |

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

Some examples also include Soroban-specific build flows using `stellar contract build`.

## Supporting Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) for the high-level organization of the framework
- [examples/README.md](examples/README.md) for the example catalog and usage notes
- [CONTRIBUTING.md](CONTRIBUTING.md) for contribution standards and workflow expectations
- [SECURITY.md](SECURITY.md) for the current security posture and reporting guidance
- [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) for the current threat model and sensitive subsystem map
- [docs/MATURITY_MODEL.md](docs/MATURITY_MODEL.md) for maturity tiers and promotion criteria
- [docs/API_CONTRACT.md](docs/API_CONTRACT.md) for the current public API contract and compatibility boundaries
- [docs/ACCOUNT_KERNEL.md](docs/ACCOUNT_KERNEL.md) for the phase 1 account-kernel model, intents, signers, and replay protection
- [docs/PRIVACY_MODEL.md](docs/PRIVACY_MODEL.md) for the phase 2 privacy split, maturity table, and proof-verification contract
- [docs/PUBLIC_GAPS.md](docs/PUBLIC_GAPS.md) for public behaviors that remain outside the stable promise

## Compatibility

| Item | Value |
|---|---|
| Rust | 1.70+ |
| Edition | 2021 |
| License | MIT |
| Primary SDK | `soroban-sdk` 25.1.0 |
| Targets | Soroban-compatible WASM targets |
