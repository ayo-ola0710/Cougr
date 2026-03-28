# Tower Defense Example

This example demonstrates a minimal tower defense game implemented as a Soroban smart contract. It includes wave spawning, enemy path progression, tower attacks, health reduction, and win/loss conditions.

## Features
- Deterministic wave and path progression
- Tower targeting and damage resolution
- Base health reduction and survival conditions

## Setup

1. Navigate to the example directory:
   ```bash
   cd examples/tower_defense
   ```

2. Build the contract:
   ```bash
   cargo build --target wasm32-unknown-unknown
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Format and lint the code:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

## Contract API

### `fn init_game(env: Env)`
Initializes the game state.

### `fn place_tower(env: Env, x: u32, y: u32, tower_kind: u32)`
Places a tower at the specified coordinates.

### `fn advance_tick(env: Env)`
Advances the game state by one tick.

### `fn get_state(env: Env) -> GameState`
Returns the current game state as a JSON string.

### `fn is_finished(env: Env) -> bool`
Checks if the game is finished.

## Validation Commands

To validate the example:
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```