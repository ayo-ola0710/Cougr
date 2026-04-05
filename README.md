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

The publishable crate is intentionally narrower than the repository. The core
library is the release artifact; examples, research notes, and CI scaffolding
exist to support development and adoption without enlarging the shipped crate
surface.

## Project Status

Cougr now defines a `1.0.0` stable contract with a deliberately scoped surface. The crate still contains Beta and Experimental namespaces, but they are explicitly separated from the stable guarantee.

Current maturity baseline:

| Area | Current status |
|---|---|
| ECS runtime and storage | Stable |
| Accounts and smart-account patterns | Beta |
| Standards layer (`standards`) | Stable |
| Privacy primitives (`zk::stable`) | Stable |
| Advanced ZK verification and confidential abstractions | Experimental |

Recommended domain namespaces:

- `app`: stable gameplay runtime
- `auth`: beta account/session flows
- `privacy::stable`: stable privacy primitives
- `privacy::experimental`: opt-in advanced proof tooling
- `ops`: stable operational standards

Runtime backend guidance:

- `SimpleWorld` and `ArchetypeWorld` are the supported Soroban-first backends
- `RuntimeWorld` and `RuntimeWorldMut` define their shared stable overlap

## What Cougr Provides

| Area | What it includes |
|---|---|
| ECS | Typed components, multiple world implementations, scheduling, deferred commands, hooks, observers, and change tracking |
| Zero-knowledge tooling | Groth16 verification, curve helpers, commitments, Merkle structures, reusable circuits, and ECS-integrated proof flows |
| Smart account patterns | Session keys, social recovery, multi-device authorization, and fallback authorization flows |
| Contract standards | Ownable, Ownable2Step, AccessControl, Pausable, execution guards, recovery guards, delayed execution, and batch primitives |
| Example contracts | 20+ example game projects and growing, each intended to show concrete patterns rather than isolated snippets |

## Quick Start

```toml
[dependencies]
cougr-core = { git = "https://github.com/salazarsebas/Cougr.git", branch = "main" }
```

```rust
use cougr_core::app::{named_system, GameApp, ScheduleStage};
use cougr_core::{Position, SystemConfig};
use soroban_sdk::Env;

let env = Env::default();
let mut app = GameApp::new(&env);

app.add_systems((
    named_system("spawn_player", |world, env| {
        let player = world.spawn_entity();
        world.set_typed(env, player, &Position::new(0, 10));
    })
    .in_stage(ScheduleStage::Startup),
    named_system("tick", |_world, _env| {})
        .with_config(SystemConfig::new().in_stage(ScheduleStage::Update)),
));

app.run(&env).unwrap();
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

### New Default App Model

For new projects, prefer `GameApp` over wiring `SimpleWorld` and `SimpleScheduler` manually.

The most direct onboarding import is `cougr_core::app`.

`GameApp` provides:

- stage-based execution (`Startup`, `PreUpdate`, `Update`, `PostUpdate`, `Cleanup`)
- dependency-aware scheduling with `before` / `after`
- deferred structural commands through context-aware systems
- grouped registration through `named_system(...)`, `named_context_system(...)`, and `add_systems(...)`
- plugin-based composition without losing direct world access

Use `PluginApp` only as a compatibility name; it aliases `GameApp`.

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

For new integrations, prefer the explicit domain alias `auth` over reaching straight for `accounts` when you want the product-level account surface.

### Standards Layer

| Capability | Description |
|---|---|
| `Ownable` and `Ownable2Step` | Owner-based authority with direct and staged transfers |
| `AccessControl` | Symbol-keyed roles with delegated role admins |
| `Pausable` | Emergency stop primitive for mutating entrypoints |
| `ExecutionGuard` and `RecoveryGuard` | Serialized critical sections and recovery-aware protection |
| `BatchExecutor` and `DelayedExecutionPolicy` | Bounded multi-operation flows and timelocked execution queues |

For new integrations, prefer the explicit domain alias `ops` when importing these standards into application code.

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

- [CHANGELOG.md](CHANGELOG.md) for the current release summary and upgrade framing
- [ARCHITECTURE.md](ARCHITECTURE.md) for the high-level organization of the framework
- [examples/README.md](examples/README.md) for the example catalog and usage notes
- [CONTRIBUTING.md](CONTRIBUTING.md) for contribution standards and workflow expectations
- [SECURITY.md](SECURITY.md) for the current security posture and reporting guidance
- [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) for the current threat model and sensitive subsystem map
- [docs/MATURITY_MODEL.md](docs/MATURITY_MODEL.md) for maturity tiers and promotion criteria
- [docs/STATE_OF_REPO.md](docs/STATE_OF_REPO.md) for a concise snapshot of what is stable, Beta, and still open
- [docs/RELEASE_STATUS.md](docs/RELEASE_STATUS.md) for the shortest release-facing summary of what is actually ship-ready
- [docs/API_CONTRACT.md](docs/API_CONTRACT.md) for the current public API contract and compatibility boundaries
- [docs/API_FREEZE_1_0.md](docs/API_FREEZE_1_0.md) for the frozen `1.0` contract and exclusion list
- [docs/COMPATIBILITY_PROMISES.md](docs/COMPATIBILITY_PROMISES.md) for the compatibility story by maturity tier
- [docs/MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md) for moving older integrations toward the curated surface
- [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md) for the release gate checklist
- [docs/ACCOUNT_KERNEL.md](docs/ACCOUNT_KERNEL.md) for the phase 1 account-kernel model, intents, signers, and replay protection
- [docs/PRIVACY_MODEL.md](docs/PRIVACY_MODEL.md) for the phase 2 privacy split, maturity table, and proof-verification contract
- [docs/STANDARDS_LAYER.md](docs/STANDARDS_LAYER.md) for the reusable standards layer, storage model, and failure semantics
- [docs/UNSAFE_INVARIANTS.md](docs/UNSAFE_INVARIANTS.md) for unsafe boundaries and the invariants that make them sound
- [docs/FEATURE_FLAGS.md](docs/FEATURE_FLAGS.md) for feature flags grouped by maturity and intended usage
- [docs/PUBLIC_GAPS.md](docs/PUBLIC_GAPS.md) for public behaviors that remain outside the stable promise
- [docs/ECS_CORE.md](docs/ECS_CORE.md) for the explicit core ECS runtime model and backend roles
- [docs/PATTERNS.md](docs/PATTERNS.md) for recommended gameplay structuring patterns
- [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for query, scheduler, and storage performance guidance
- [docs/adr/0001-public-surface.md](docs/adr/0001-public-surface.md) for the public-surface curation decision record

## Compatibility

| Item | Value |
|---|---|
| Rust | 1.70+ |
| Edition | 2021 |
| License | MIT |
| Primary SDK | `soroban-sdk` 25.1.0 |
| Targets | Soroban-compatible WASM targets |
