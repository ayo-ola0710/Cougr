# Proof of Hunt (Stellar + Soroban)

Proof of Hunt is a hidden-map treasure discovery example for Soroban that combines:

- hidden world state committed off-chain
- proof-backed exploration and deterministic progression on-chain
- premium actions modeled for x402 settlement flows

This example is intentionally contract-only: no frontend, no multiplayer layer.

## Why This Is Stellar-Specific

This example demonstrates three Stellar-native patterns working together:

1. Soroban contract state for deterministic gameplay and progression.
2. stellar-zk style Groth16 verifier flow on-chain using Soroban BN254 pairing checks.
3. x402-style premium action credits represented as settled payment units before hint consumption.

References:

- https://crates.io/crates/stellar-zk
- https://github.com/salazarsebas/stellar-zk
- https://developers.stellar.org/docs/build/apps/x402

## Hidden State And Proof Flow

### Off-chain committed data

The hidden map is represented off-chain and committed by root hash:

- map commitment root (`BytesN<32>`)
- map dimensions (`width`, `height`)
- implicit treasure distribution encoded in proof public inputs

### What is proven on-chain

For each exploration:

- `(x, y)` belongs to a valid proof statement bound to the same commitment root
- leaf + sibling path resolves to the committed root
- Groth16 proof verifies through BN254 pairing checks
- nullifier has not been replayed

### Anti-cheat and privacy properties

- Players cannot claim arbitrary discoveries because proof public inputs are tied to coordinates and root.
- Replay is blocked by nullifier storage.
- Full hidden map remains off-chain; only commitment and selective proof metadata are revealed.

## x402 Premium Action Model

`purchase_hint(player, hint_type)` consumes pre-settled premium credits.

This maps to an x402 backend flow where a payment gateway verifies and settles payment off-chain, then credits the user in-contract via `credit_x402_payment(...)`.

- `hint_type = 0`: hint action (cost 1 credit)
- `hint_type = 1`: scan action (cost 2 credits)

## Contract API

Required functions:

- `init_game(env, player, map_commitment, width, height)`
- `explore(env, player, x, y, proof)`
- `purchase_hint(env, player, hint_type)`
- `get_state(env) -> GameState`
- `is_finished(env) -> bool`

Additional helper functions:

- `set_verification_key(env, owner, vk_bytes)`
- `credit_x402_payment(env, owner, player, units, receipt_hash)`

## Architecture Components

- `MapCommitmentComponent`: commitment root, width, height, derived treasure count
- `PlayerStateComponent`: position, score, health, discoveries
- `ExplorationComponent`: explored cell tracking
- `HintUsageComponent`: hints/scans used
- `GameStatusComponent`: active/won/lost

### Systems

- `ExplorationSystem`: coordinate bounds + replay prevention
- `ProofValidationSystem`: public input checks + Merkle path + Groth16 verify
- `DiscoveryResolutionSystem`: score/health/discovery updates
- `HintPurchaseSystem`: x402 credit consumption
- `EndConditionSystem`: won/lost transitions

## Build And Test

All commands are Soroban target aligned (`wasm32v1-none`):

```bash
cd examples/proof_of_hunt
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
stellar contract build
```

## Notes On stellar-zk Integration

The contract uses the same Groth16 verifier model used by stellar-zk templates:

- BN254 proof point decoding
- verification key layout parsing
- multi-pairing check (`env.crypto().bn254().pairing_check`)

For CI tests in this repository, a deterministic zero-proof test path is enabled only under `#[cfg(test)]` so tests remain fully deterministic without external trusted setup artifacts.
