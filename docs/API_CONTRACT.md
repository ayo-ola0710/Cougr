# Cougr Public API Contract

## Purpose

This document defines how Cougr presents its public Rust API for `1.0`.

It answers four practical questions:

- which entrypoints are central to the product
- which surfaces are usable but still evolving
- which modules should not be interpreted as production commitments
- which compatibility shims or testing helpers are intentionally outside the long-term contract

## API Positioning

Cougr exposes a broad crate surface, but only a scoped subset is part of the defended `1.0` contract.

The current product story is:

- `cougr-core` is primarily an ECS framework for Soroban-compatible applications
- `app` is the default gameplay runtime surface for new projects
- `auth`, `privacy`, and `ops` are the clearest product-level domain namespaces
- accounts remain Beta, while privacy is split between a stable primitive subset and experimental proof systems
- ECS onboarding/runtime surfaces and `standards` are part of the `1.0` stable contract
- helper APIs that exist only for compatibility or transition should remain clearly demoted

## Recommended Public Contract

This file now serves as the explicit `1.0` stable API list for `cougr-core`.

### Core entrypoints

These are the frozen entrypoints for the `1.0` stable contract:

- `SimpleWorld`
- `ArchetypeWorld`
- `ecs::{RuntimeWorld, RuntimeWorldMut, WorldBackend}`
- typed and raw component operations
- command queues
- scheduling primitives
- events, hooks, and observers
- incremental persistence utilities

Concrete frozen root-level contract:

- `SimpleWorld`
- `ArchetypeWorld`
- `CommandQueue`
- `Component`, `ComponentTrait`, `ComponentStorage`, `ComponentId`
- `Entity`, `EntityId`
- `Query`, `QueryState`
- `SimpleQuery`, `SimpleQueryBuilder`, `SimpleQueryState`
- `RuntimeWorld`, `RuntimeWorldMut`, `WorldBackend`
- `SystemContext`
- `SystemSpec`
- `Resource`
- `ChangeTracker`, `TrackedWorld`
- `Plugin`, `PluginGroup`, `GameApp`, `PluginApp`
- `ScheduleStage`, `SystemConfig`, `SimpleScheduler`, `SystemGroup`
- `prelude`
- `runtime`
- `app`
- `legacy` as a compatibility-preserving namespace
- `ops` as the clearest Stable standards namespace
- `standards` as a Stable namespace
- `privacy::stable` as the clearest stable privacy namespace
- `zk::stable` as the stable privacy namespace
- `auth` as the clearest Beta account namespace
- `accounts` as a Beta namespace
- `privacy::experimental` as an explicitly non-contract namespace
- `zk::experimental` as an explicitly non-contract namespace

### Supported but evolving surfaces

These surfaces are useful and implemented, but should continue to be presented as Beta:

- `legacy`
- `accounts`
- `game_world`
- higher-level query helpers
- higher-level scheduler helpers
- proof-submission helpers in `zk`

### Stable privacy subset

These privacy surfaces are intentionally narrower and can be presented as Stable:

- commitments
- commit-reveal
- hidden-state codec interfaces
- Merkle inclusion and sparse Merkle utilities
- `zk::stable`

### Non-contract surfaces

These surfaces are public today, but they must not be interpreted as stable commitments:

- testing-only helpers such as `zk::testing`
- advanced proof-verification APIs whose assumptions are still being hardened
- `zk::experimental`
- compatibility shims retained for transition
- internals-heavy modules whose invariants are not yet documented as stable guarantees

## Top-Level Surface in `src/lib.rs`

### Public modules

Current top-level modules:

- `app`
- `auth`
- `accounts`
- `archetype_world`
- `change_tracker`
- `commands`
- `component`
- `debug` behind feature flag
- `error`
- `event`
- `game_world`
- `hooks`
- `incremental`
- `observers`
- `ops`
- `privacy`
- `plugin`
- `query`
- `resource`
- `scheduler`
- `simple_world`
- `system`
- `world`
- `zk`

Internal implementation modules such as legacy demo components, duplicate
system helpers, storage internals, and entity internals are no longer part of
the intended default public surface. They may still exist in the repository,
but the root crate is not meant to advertise them as onboarding entrypoints.

### Public re-exports

Current top-level re-exports emphasize:

- worlds: `World`, `SimpleWorld`, `ArchetypeWorld`
- backend contracts: `RuntimeWorld`, `RuntimeWorldMut`, `WorldBackend`
- ECS data: `Component`, `ComponentId`, `ComponentStorage`, `ComponentTrait`, `Position`, `Entity`, `EntityId`, `Resource`
- orchestration: `CommandQueue`, `HookRegistry`, `ObserverRegistry`, `PluginApp`, schedulers
- queries and systems: `Query`, `QueryState`, `SimpleQuery`, `SimpleQueryState`, `SystemContext`, `SystemSpec`, declarative runtime system helpers
- domain access through explicit namespaces: `auth`, `privacy`, `ops`, `accounts`, `zk::stable`, `zk::experimental`

### Public top-level helper functions

There are no root-level placeholder helper functions in the supported contract.

The sanctioned onboarding path is the curated root surface itself:

- `app`
- `auth`
- `privacy`
- `ops`
- `SimpleWorld`
- `ArchetypeWorld`
- `legacy`
- `CommandQueue`
- `GameApp`
- `named_system`, `named_context_system`, `add_systems`

## Compatibility Exceptions

### `zk::testing`

`zk::testing` is a support surface for tests and explicit test utility consumers.
It is not part of the default product contract and is gated to tests or the
`testutils` feature.

## Public API Risks

The main public API risks before this cleanup were:

- the crate exports more surface area than it can reasonably defend as stable
- some internals-heavy modules are public before their long-term contract is clearly documented
- some privacy and verification surfaces are easy to overread as production guarantees
- accounts and privacy modules still include beta-grade behavior that is intentionally documented outside the stable story

## Freeze Direction

The `1.0` freeze is intentionally narrower than the full public module graph:

- `app` is the clearest default runtime namespace for new gameplay code
- `auth` is the clearest Beta auth namespace for application code
- `privacy` is the clearest domain namespace for privacy adoption, with stability determined by submodule
- `ops` is the clearest stable namespace for operational standards in application code
- root re-exports and `prelude` are the default onboarding path
- `runtime` is a supported stable namespace for advanced ECS integrations
- `legacy` is compatibility-preserving, Beta, and intentionally not the default onboarding path
- `standards` is a supported stable namespace
- `accounts` remains a public Beta namespace
- `zk::stable` is the only privacy namespace treated as Stable
- `zk::experimental` remains public for explicit opt-in use, but outside compatibility guarantees
