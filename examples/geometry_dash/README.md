# Geometry Dash Soroban Contract

A rhythm platformer game implemented as a Soroban smart contract using the `Cougr` ECS framework.

## Features

- **ECS Architecture**: Uses `Cougr` for clean separation of components and systems.
- **Tick-based Simulation**: Deterministic game logic that advances in discrete steps.
- **Multiple Player Modes**: Support for Cube, Ship, Wave, and Ball modes, each with unique physics.
- **Collision System**: Detects collisions with spikes, blocks, and mode-switching portals.
- **On-chain State**: Game world and player progress are persisted in Soroban storage.

## Game Physics

The game operates on a fixed-fixed point coordinate system (scaled by 1000).

- **Cube**: Gravity pulls down, tap to jump.
- **Ship**: Constant gravity, hold (multi-tap) to fly up.
- **Wave**: Oscillates at 45 degrees up when held, 45 degrees down when released.
- **Ball**: Tapping switches gravity direction.

## API

- `init_game(player: Address, level_id: u32)`: Initializes the level and player state.
- `jump(player: Address)`: Triggers the "action" input for the current player mode.
- `update_tick(player: Address)`: Advances the game state by one frame.
- `get_state(player: Address) -> GameStatus`: Returns `Playing`, `Crashed`, or `Completed`.
- `get_score(player: Address) -> u32`: Returns current distance-based score.
- `get_mode(player: Address) -> u32`: Returns current player mode identifier.

## Testing

Run unit tests:
```bash
cargo test
```

Build the contract:
```bash
stellar contract build
```

## Level Structure

Obstacles consist of:
- **Spikes**: Trigger `GameStatus::Crashed` on collision.
- **Blocks**: Standard solid obstacles.
- **Portals**: Change the player's `PlayerMode`.
