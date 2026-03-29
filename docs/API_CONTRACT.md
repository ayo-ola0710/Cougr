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
- accounts and privacy capabilities are important differentiators, but they are not yet advertised as fully stable contract surfaces
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
- privacy primitives in `zk`

### Non-contract surfaces

These surfaces are public today, but they must not be interpreted as stable commitments:

- testing-only helpers such as `zk::testing`
- advanced proof-verification APIs whose assumptions are still being hardened
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
- `components`
- `debug` behind feature flag
- `entity`
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
- `storage`
- `system`
- `systems`
- `world`
- `zk`

### Public re-exports

Current top-level re-exports emphasize:

- worlds: `World`, `SimpleWorld`, `ArchetypeWorld`
- ECS data: `Component`, `ComponentId`, `Entity`, `EntityId`, `Resource`
- orchestration: `CommandQueue`, `HookRegistry`, `ObserverRegistry`, `PluginApp`, schedulers
- queries and systems: `Query`, `QueryState`, `System`, `SystemParam`
- accounts and privacy access through their own modules

### Public top-level helper functions

Current top-level helper functions:

- `create_world`
- `spawn_entity`
- `add_component`
- `remove_component`
- `get_component`

## Compatibility Exceptions

### `zk::testing`

`zk::testing` is a support surface for tests and explicit test utility consumers. It is not part of the default product contract and is gated to tests or the `testutils` feature.

## Public API Risks

The main public API risks today are:

- the crate exports more surface area than it can reasonably defend as stable
- some internals-heavy modules are public before their long-term contract is clearly documented
- some privacy and verification surfaces are easy to overread as production guarantees
- accounts and privacy modules still include beta-grade behavior that is intentionally documented outside the stable story

## Direction

Future API cleanup should continue to:

1. define a smaller golden path for `cougr-core`
2. separate stable and experimental privacy surfaces more aggressively
3. reduce exposure of internals-heavy modules where no durable contract exists
4. keep documentation aligned with code reality and compatibility intent
