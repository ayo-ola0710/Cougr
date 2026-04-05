# Migration Guide

## Purpose

This guide explains how to move existing Cougr integrations toward the curated `1.0` product surface.

It is not a promise that every older pattern disappears immediately. It is the recommended direction for users who want to converge on the defended path.

## Core Direction

Prefer these namespaces in new or updated code:

- `app` for gameplay runtime
- `legacy` only for the older `World` / `System` model
- `auth` for account and session flows
- `privacy::stable` for stable privacy primitives
- `ops` for operational standards

## Runtime Migration

### From direct world/scheduler wiring

If you currently do something like:

```rust
let mut world = SimpleWorld::new(&env);
let mut scheduler = SimpleScheduler::new();
```

prefer:

```rust
let mut app = cougr_core::app::GameApp::new(&env);
```

and register systems through `GameApp`.

When multiple systems belong to the same phase, prefer the declarative path:

```rust
use cougr_core::app::{named_context_system, named_system, GameApp, ScheduleStage};

let mut app = GameApp::new(&env);
app.add_systems((
    named_system("spawn", |world, env| {
        let entity = world.spawn_entity();
        world.set_typed(env, entity, &Position::new(0, 0));
    })
    .in_stage(ScheduleStage::Startup),
    named_context_system("cleanup_tags", |context| {
        let entities = context
            .world()
            .get_entities_with_component(&symbol_short!("expired"), context.env());
        for i in 0..entities.len() {
            let entity = entities.get(i).unwrap();
            context
                .commands()
                .remove_component(entity, symbol_short!("expired"));
        }
    })
    .in_stage(ScheduleStage::Cleanup),
));
```

Why:

- clearer lifecycle
- explicit stages
- one onboarding surface instead of several loose primitives
- a single system registration model for plain and context-aware systems

### From root `World`

If you intentionally stay on the older ECS model, change:

```rust
let mut world = cougr_core::World::new();
```

to:

```rust
let mut world = cougr_core::legacy::World::new();
```

Why:

- it keeps compatibility explicit
- it avoids presenting the legacy path as the default Soroban runtime

## Query Migration

If you still do ad-hoc scans or manual component filtering, prefer:

- `SimpleQueryBuilder`
- `SimpleQueryState`
- `SimpleQueryCache`

Both `SimpleQueryBuilder` and `ArchetypeQueryBuilder` now support:

- `with_components(...)`
- `without_components(...)`
- `with_any_components(...)`

If you need backend-agnostic gameplay helpers across Soroban-first worlds, prefer:

- `RuntimeWorld`
- `RuntimeWorldMut`

These are the shared contracts between `SimpleWorld` and `ArchetypeWorld`.

## Domain Migration

### Accounts

If you currently import from `accounts` directly in application code:

```rust
use cougr_core::accounts::SessionBuilder;
```

prefer:

```rust
use cougr_core::auth::SessionBuilder;
```

The semantics are the same today. The change is about product clarity.

### Privacy

If you rely on stable privacy primitives, prefer:

```rust
use cougr_core::privacy::stable::...
```

instead of:

```rust
use cougr_core::zk::stable::...
```

If you rely on advanced proof tooling, prefer:

```rust
use cougr_core::privacy::experimental::...
```

and treat it as an explicit opt-in to non-frozen APIs.

### Standards

If you currently import standards directly:

```rust
use cougr_core::standards::Pausable;
```

prefer:

```rust
use cougr_core::ops::Pausable;
```

Again, this is a namespace migration for clarity, not a semantic rewrite.

## Example-Level Migration

Use these examples as references:

- `snake` for `app::GameApp` and stage-based gameplay loops
- `battleship` for `privacy::stable` and hidden-information patterns
- `guild_arena` for account/session/recovery patterns

Examples that still use `legacy` are intentionally compatibility-oriented, not the primary direction for new gameplay contracts.

## What Does Not Need Immediate Migration

You do not need to rewrite everything at once if:

- the contract intentionally depends on the old `World` / `System` path
- you are preserving an older example or integration
- your current code already sits behind a stable local abstraction

The main goal is to stop growing new code on top of older default imports.

## Migration Checklist

- [x] move runtime entrypoints to `app` where practical
- [x] move legacy ECS usage to `legacy`
- [x] move account imports to `auth`
- [x] move stable privacy imports to `privacy::stable`
- [x] move standards imports to `ops`
- [x] update local docs/examples to use the curated namespaces
