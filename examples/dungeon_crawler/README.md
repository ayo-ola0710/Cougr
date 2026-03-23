# Dungeon Crawler - Gasless Gaming with Session Keys

A single-player dungeon exploration game demonstrating Cougr's SessionBuilder and GameWorld for gasless gameplay sessions.

## The Session Key Pattern

Traditional blockchain games suffer from poor UX due to wallet confirmation prompts for every action. This creates friction that destroys the gaming experience - imagine having to confirm a wallet transaction for every move in chess!

**Session keys solve this problem** by allowing players to pre-authorize a scoped set of actions with time and operation limits. Once created, the session key can execute approved actions without further wallet prompts.

### How It Works

1. **Session Creation**: Player creates a session key scoped to specific actions (`move`, `attack`, `pickup`) with:
   - **Time limit**: Session expires after N ledgers
   - **Operation limit**: Maximum M operations allowed
   - **Action scope**: Only approved actions can be executed

2. **Gasless Gameplay**: Using the session key, players can:
   - Explore dungeon rooms
   - Fight monsters
   - Collect items
   - All without wallet confirmations!

3. **Automatic Expiry**: Sessions end when:
   - Time limit reached
   - Operation limit exhausted
   - Player manually ends session

## Game Overview

Navigate through a dungeon with interconnected rooms, fight monsters, and collect items. Your progress persists between sessions, but you need an active session to perform actions.

### Game Flow

```
START SESSION → EXPLORE → FIGHT → LOOT → SESSION EXPIRES → REPEAT
```

### Dungeon Layout

```
[Room 1: Entrance] ←→ [Room 2: Goblin Lair] ←→ [Room 3: Treasure Chamber]
     ↓ Health Potion        ↓ Goblin + Sword         ↓ Dragon + Shield + Gold
```

## Contract API

### Session Management

```rust
// Create a scoped session key
fn start_session(env: Env, player: Address, duration: u64, max_ops: u32) -> Address

// End the current session
fn end_session(env: Env, player: Address)
```

### Gameplay Actions (Require Active Session)

```rust
// Move between connected rooms
fn move_player(env: Env, session_key: Address, direction: u32) -> Result<GameState, GameError>

// Attack monsters in current room
fn attack(env: Env, session_key: Address, target_id: u32) -> Result<CombatResult, GameError>

// Pick up items in current room
fn pickup(env: Env, session_key: Address, item_id: u32) -> Result<GameState, GameError>
```

### State Queries

```rust
// Initialize the dungeon
fn init_dungeon(env: Env, player: Address) -> GameState

// Get current game state
fn get_state(env: Env, player: Address) -> Result<GameState, GameError>
```

## Technical Implementation

### Core Components

- **SessionBuilder**: Creates scoped session keys with time/operation limits
- **GameWorld**: Bridges ECS entities with account management
- **ECS Architecture**: Components (Position, Health, Inventory) and Systems (Movement, Combat, Loot)

### Session Scoping

```rust
let scope = SessionBuilder::new(&env)
    .allow_action(symbol_short!("move"))
    .allow_action(symbol_short!("attack"))
    .allow_action(symbol_short!("pickup"))
    .max_operations(100)
    .expires_at(env.ledger().timestamp() + 3600)
    .build_scope();
```

### Key Constraints

- **Action Validation**: Only scoped actions (`move`, `attack`, `pickup`) work with session keys
- **Operation Counting**: Each action consumes one operation from the session budget
- **Persistent State**: Game progress persists in `env.storage().persistent()` across sessions
- **Automatic Expiry**: Sessions automatically expire based on time or operation limits

## Building and Testing

```bash
# Build the contract
cargo build
stellar contract build

# Run tests
cargo test

# Deploy (example)
stellar contract deploy --wasm target/wasm32v1-none/release/dungeon_crawler.wasm
```

## Why Session Keys Matter for Gaming

**Without Session Keys:**
- Every move requires wallet confirmation
- Players abandon games due to friction
- Impossible to create fluid gaming experiences

**With Session Keys:**
- Smooth, uninterrupted gameplay
- Players pre-authorize actions they trust
- Games feel like traditional applications
- Maintains security through scoped permissions

This pattern is essential for any on-chain game where user experience matters. Session keys enable the transition from "blockchain applications" to "applications that happen to use blockchain."

## Example Usage

```rust
// 1. Initialize dungeon
let state = DungeonCrawlerContract::init_dungeon(env, player);

// 2. Start 1-hour session with 50 operations
let session_key = DungeonCrawlerContract::start_session(env, player, 3600, 50);

// 3. Play without wallet prompts!
DungeonCrawlerContract::move_player(env, session_key, 0)?; // Move north
DungeonCrawlerContract::attack(env, session_key, 1)?;      // Attack goblin
DungeonCrawlerContract::pickup(env, session_key, 2)?;     // Grab sword

// 4. Session automatically expires after 50 operations or 1 hour
```

The future of blockchain gaming is gasless, and session keys make it possible.
