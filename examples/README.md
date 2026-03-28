# Examples

This directory contains standalone game projects built on top of `cougr-core`. The examples are intended to serve two purposes:

- demonstrate how the framework can be applied to different gameplay models
- provide reference implementations for architecture, storage, authorization, and verification patterns

The catalog is expected to grow over time. Documentation in this directory should therefore avoid hard dependencies on exact counts or one-off example narratives.

## How To Use The Examples

Each example lives in its own directory and can be built independently. In most cases, the workflow is:

```bash
cd examples/<example-name>
cargo build
cargo test
```

Some examples also include Soroban-specific build steps:

```bash
stellar contract build
```

## Example Catalog

| Example | Category | Focus |
|---|---|---|
| `angry_birds` | Physics-inspired arcade | Projectile logic and destructible-state style gameplay |
| `arkanoid` | Arcade | Paddle, collision, and brick lifecycle management |
| `asteroids` | Arcade | Entity-heavy movement, collisions, and spawning |
| `battleship` | Board / hidden information | Commit-reveal and selective state disclosure |
| `bomberman` | Grid action | Tile updates, hazards, and timed interactions |
| `chess` | Board / strategy | Rule validation and proof-oriented move flow |
| `dungeon_crawler` | Progression | Stateful exploration and encounter management |
| `flappy_bird` | Arcade | Tight tick-loop updates and obstacle generation |
| `geometry_dash` | Reflex | Deterministic timing and obstacle progression |
| `guild_arena` | Account patterns | Social recovery and multi-device gameplay |
| `pac_man` | Maze action | Grid navigation and adversarial movement patterns |
| `pokemon_mini` | Turn-based battle | Combat sequencing and match state transitions |
| `pong` | Arcade | Minimal competitive loop and ECS fundamentals |
| `proof_of_hunt` | Hidden-state exploration | stellar-zk style proof verification and x402 premium actions |
| `rock_paper_scissors` | Commit-reveal | Hidden choices and reveal resolution |
| `snake` | Arcade | Growth mechanics and collision rules |
| `space_invaders` | Wave shooter | Formation movement and repeated tick systems |
| `tap_battle` | Casual competitive | Lightweight action resolution and progression |
| `tetris` | Puzzle | Piece state, rotation, and board clearing |
| `treasure_hunt` | Hidden-state exploration | Off-chain Merkle map commitments with on-chain proof-gated discovery and sparse fog-of-war |
| `tic_tac_toe` | Board | Small-state deterministic turn handling |
| `trading_card_game` | Card / strategy | Structured turns, card effects, and state composition |

## Choosing A Reference

Use examples by pattern, not only by genre:

| If you need to study | Good starting points |
|---|---|
| Basic ECS structure | `pong`, `snake`, `tetris` |
| Hidden state or commit-reveal | `battleship`, `rock_paper_scissors` |
| Turn-based logic | `tic_tac_toe`, `pokemon_mini`, `chess` |
| Account abstraction patterns | `guild_arena` |
| Larger multi-entity loops | `asteroids`, `space_invaders`, `pac_man` |

## Conventions

- Keep each example self-contained.
- Prefer a clear gameplay loop over framework trickery.
- Document any non-obvious contract behavior in the example's local `README.md`.
- If an example introduces a reusable pattern, reflect that pattern back into the core documentation where appropriate.

## Adding A New Example

Before adding a new example:

1. confirm the example demonstrates a pattern not already covered clearly elsewhere
2. keep the directory standalone and runnable on its own
3. include a local `README.md` with scope, commands, and design notes
4. add or update a CI workflow if the example should be validated automatically

For contribution expectations across the repository, see [CONTRIBUTING.md](../CONTRIBUTING.md).

