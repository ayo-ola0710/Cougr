# Changelog

## 1.0.0

### Added

- `app` as the default gameplay runtime surface
- `auth`, `privacy`, and `ops` as product-level domain namespaces
- `RuntimeWorld` and `RuntimeWorldMut` as shared Soroban-first backend contracts
- stronger stage scheduling with ordering, sets, and validation
- `SimpleQueryBuilder`, query state/cache improvements, and richer `ArchetypeWorld` query helpers
- expanded benchmark coverage for backend comparisons and cache invalidation behavior

### Changed

- the recommended onboarding path is now `app::GameApp` + `SimpleWorld` + `SimpleQueryBuilder`
- canonical examples now emphasize the curated runtime story and explicit maturity boundaries
- `battleship` now uses stable privacy primitives from `zk::stable`
- documentation now treats `SimpleWorld` and `ArchetypeWorld` as the defended Soroban-first backends

### Stability Notes

- Stable: ECS onboarding/runtime contract, `app`, `ops`, `standards`, `privacy::stable`, `zk::stable`
- Beta: `auth`, `accounts`, `game_world`
- Experimental: `privacy::experimental`, `zk::experimental`, hazmat cryptographic helpers

### Upgrade Notes

- Prefer `app` over wiring scheduler/world primitives directly for new gameplay code
- If you still have pre-1.0 code built around removed runtime abstractions, port directly to `GameApp`, `SimpleWorld`, and `SimpleQuery`
- Prefer `ops`, `privacy`, and `auth` in application code when you want domain-oriented imports
- Treat root-level advanced re-exports as compatibility/advanced surfaces rather than the default learning path
- See [docs/MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md) for concrete migration mappings
