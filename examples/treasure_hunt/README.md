# Treasure Hunt (Merkle Map Pattern)

`treasure_hunt` demonstrates a core on-chain game pattern:

1. Build a large map off-chain.
2. Store only a Merkle root on-chain.
3. Require Merkle proofs to reveal cell contents.
4. Track revealed cells as sparse state ("fog-of-war").

## Why this pattern matters

Large maps are too expensive to store fully on-chain. This example keeps
on-chain storage bounded:

- Full map remains off-chain.
- Contract stores only:
  - committed map root
  - player/game state
  - explored-cell sparse state

This enables large worlds while preserving verifiability.

## Cell encoding and map commitment

Each map cell is encoded into 32 bytes:

- bytes `[0..4]`: `x` (big-endian)
- bytes `[4..8]`: `y` (big-endian)
- byte `[8]`: `cell_value` (`0=empty`, `1=treasure`, `2=trap`)
- remaining bytes: zero

The off-chain generator:

- builds all encoded leaves in row-major order (`idx = y * width + x`)
- builds a SHA256 Merkle tree with `cougr_core::zk::merkle::MerkleTree`
- publishes the root and map dimensions via `init_game`

## Contract flow

### `init_game(player, map_root, width, height, total_treasures)`

- stores map commitment metadata
- initializes player at `(0,0)`, health/score defaults
- initializes empty fog-of-war root from `SparseMerkleTree`

### `explore(player, x, y, cell_value, proof)`

- checks auth and active game
- enforces adjacent movement and bounds
- rejects already explored cells
- reconstructs Merkle proof structure and verifies inclusion against committed root
- applies discovery effects:
  - treasure: +score, +treasures_found
  - trap: health deduction
- marks cell explored
- recomputes sparse fog root using `SparseMerkleTree`
- updates game status (`Won`/`Lost`/`Active`)

### `get_state()`

Returns complete game state including map root metadata, player stats,
explored map, game config, and current fog root.

### `is_explored(x, y)`

Returns whether the cell has been revealed.

## Build and test

```bash
cd examples/treasure_hunt
cargo build
stellar contract build
cargo test
```

## Test coverage highlights

- valid exploration with correct Merkle proof
- invalid proof rejection
- treasure scoring and trap damage
- re-exploration rejection
- win condition (all treasures found)
- loss condition (health reaches zero)
- full playable sequence from init to terminal state
- sparse fog root updates after exploration
