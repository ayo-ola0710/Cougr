# Cougr Public API Contract

## Purpose

This document defines how Cougr presents its public Rust API before `1.0`.

It answers four practical questions:

- which entrypoints are central to the product
- which surfaces are usable but still evolving
- which modules should not be interpreted as production commitments
- which compatibility shims or testing helpers are intentionally outside the long-term contract

## API Positioning

Cougr currently exposes a broad surface for a pre-`1.0` crate. That is acceptable only if the repository is explicit about which parts represent the intended public contract and which parts are still in motion.

The current product story is:

- `cougr-core` is primarily an ECS framework for Soroban-compatible applications
- accounts remain Beta, while privacy is split between a stable primitive subset and experimental proof systems
- helper APIs that exist only for compatibility or transition should remain clearly demoted

## Recommended Public Contract

### Core entrypoints

These are the strongest candidates for the long-term public contract:

- `World`
- `SimpleWorld`
- `ArchetypeWorld`
- typed and raw component operations
- command queues
- scheduling primitives
- events, hooks, and observers
- incremental persistence utilities

### Supported but evolving surfaces

These surfaces are useful and implemented, but should continue to be presented as Beta:

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
- ECS data: `Component`, `ComponentId`, `ComponentStorage`, `ComponentTrait`, `Position`, `Entity`, `EntityId`, `Resource`
- orchestration: `CommandQueue`, `HookRegistry`, `ObserverRegistry`, `PluginApp`, schedulers
- queries and systems: `Query`, `QueryState`, `System`, `SystemParam`
- accounts and privacy access through explicit namespaces: `accounts`, `zk::stable`, `zk::experimental`

### Public top-level helper functions

There are no root-level placeholder helper functions in the supported contract.

The sanctioned onboarding path is the curated root surface itself:

- `SimpleWorld`
- `World`
- `ArchetypeWorld`
- `CommandQueue`
- `accounts`
- `zk::{stable, experimental}`

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

## Direction

The current cleanup direction is:

1. define a smaller golden path for `cougr-core`
2. separate stable and experimental privacy surfaces more aggressively
3. reduce exposure of internals-heavy modules where no durable contract exists
4. keep documentation aligned with code reality and compatibility intent
