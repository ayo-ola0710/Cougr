# Project Bootstrap

Use this reference when starting a fresh project that will depend on `cougr-core`.

## Default Goal

Create a small Soroban-compatible Rust project that can:

- compile cleanly
- express game state in a Cougr-friendly structure
- expose a minimal contract API
- support tests from the first iteration

## Recommended Starting Shape

For most prototypes, use a single crate with a small `src/` tree:

```text
my-game/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs
│   ├── components.rs
│   ├── systems.rs
│   ├── state.rs
│   └── test.rs
```

Collapse files when the project is tiny. Split files when game logic becomes harder to scan than to navigate.

## Dependency Strategy

Choose one dependency mode based on the user's situation:

| Situation | Recommended dependency style |
|---|---|
| Building against the latest repository state | Git dependency to the Cougr repository |
| Building against a known published release | Exact crate version if available |
| Working locally across two repos | Local path dependency during active development |

If the user does not specify otherwise, prefer a git dependency to the main repository branch they expect to use.

Example:

```toml
[dependencies]
soroban-sdk = "25.1.0"
cougr-core = { git = "https://github.com/salazarsebas/Cougr.git", branch = "main" }
```

Adjust the source if the user asks for a tag, version, fork, or local path.

## Build Target

Use `wasm32v1-none` as the WASM target for Soroban-oriented builds.

Common commands:

```bash
rustup target add wasm32v1-none
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --target wasm32v1-none --release
```

If `stellar contract build` is available in the project workflow, use it where appropriate.

## Starter Module Responsibilities

| File | Responsibility |
|---|---|
| `src/lib.rs` | Contract entrypoints and top-level wiring |
| `src/components.rs` | Game-facing component definitions |
| `src/systems.rs` | State transitions and gameplay logic |
| `src/state.rs` | Aggregate state shape or ECS world persistence helpers |
| `src/test.rs` | Match-flow and rules tests |

## Starter Design Rules

- Keep the initializer simple and deterministic.
- Keep action methods narrow.
- Return small, useful state snapshots to help tests and clients.
- Avoid building a generalized engine wrapper before the game loop exists.
- Add new files only when they reduce cognitive load.

## Minimal Contract Skeleton

Use this shape as a starting point, then adapt it to the actual game:

```rust
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Env, Symbol, symbol_short};

const STATE_KEY: Symbol = symbol_short!("STATE");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub initialized: bool,
    pub game_over: bool,
}

#[contract]
pub struct GameContract;

#[contractimpl]
impl GameContract {
    pub fn init_game(env: Env) -> GameState {
        let state = GameState {
            initialized: true,
            game_over: false,
        };
        env.storage().instance().set(&STATE_KEY, &state);
        state
    }

    pub fn get_state(env: Env) -> GameState {
        env.storage()
            .instance()
            .get(&STATE_KEY)
            .unwrap_or_else(|| panic!("game not initialized"))
    }
}
```

This is only a shell. Move actual gameplay rules into helper functions or systems quickly.

