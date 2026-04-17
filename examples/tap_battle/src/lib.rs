//! Tap Battle — On-Chain Mobile-First Competitive Tapping Game
//!
//! This smart contract implements a competitive tapping game on the Stellar
//! blockchain using cougr-core's ECS framework and passkey authentication.
//!
//! # Mobile-First Authentication Flow
//! This example demonstrates the complete passkey → session → gameplay flow:
//! 1. **Registration**: Player registers a passkey (secp256r1 public key)
//!    → No seed phrases, no mnemonics — just Face ID / Touch ID
//! 2. **Authentication + Session**: Player authenticates via passkey
//!    → `verify_secp256r1()` validates the biometric signature
//!    → `SessionBuilder` creates a gameplay session scoped to `tap` + `use_power`
//! 3. **Gameplay (gasless via session key)**: Rapid tapping is processed
//!    through the session key with no per-transaction wallet prompts
//! 4. **Result**: Scores compared, winner declared, stats recorded on-chain
//!
//! # Cougr-Core Integration
//! - `secp256r1_auth`: Passkey registration and signature verification
//! - `SessionBuilder`: Scoped session creation for gasless gameplay
//! - ECS Components: `PasskeyIdentity`, `TapCounter`, `PowerUp`, `RoundState`
//! - ECS World: Entity management for game objects

#![no_std]

mod auth;
mod game;
mod types;

#[cfg(test)]
mod test;

use cougr_core::SimpleWorld;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env};

// Re-export types for external use
pub use types::*;

#[contract]
pub struct TapBattleContract;

#[contractimpl]
impl TapBattleContract {
    /// Register a secp256r1 passkey for a player.
    ///
    /// This replaces seed phrases with biometric authentication (Face ID,
    /// Touch ID, or hardware security keys). The public key is stored
    /// on-chain via `Secp256r1Storage`.
    ///
    /// # Arguments
    /// * `player` - The player's Stellar address
    /// * `pubkey` - SEC-1 uncompressed secp256r1 public key (65 bytes)
    pub fn register_passkey(env: Env, player: Address, pubkey: BytesN<65>) {
        // Create cougr-core ECS World for entity management
        let mut world = SimpleWorld::new(&env);
        let _player_entity = world.spawn_entity();

        auth::register_passkey(&env, &player, &pubkey);

        // Store ECS entity count
        env.storage()
            .instance()
            .set(&DataKey::EntityCount, &(world.next_entity_id() - 1));
    }

    /// Authenticate via passkey and create a gameplay session.
    ///
    /// Verifies the secp256r1 signature (biometric challenge), then creates
    /// a `SessionBuilder` session scoped to `tap` and `use_power` actions.
    /// After this call, the player can submit rapid tap actions without
    /// per-transaction wallet prompts.
    ///
    /// # Arguments
    /// * `player` - The player's Stellar address
    /// * `signature` - secp256r1 signature (64 bytes) of the challenge
    /// * `challenge` - Random challenge that was signed (32 bytes)
    /// * `duration` - Session duration in ledger sequences
    ///
    /// # Returns
    /// The session key address for subsequent gameplay calls
    pub fn authenticate_and_start_session(
        env: Env,
        player: Address,
        signature: Bytes,
        challenge: BytesN<32>,
        duration: u64,
    ) -> Address {
        auth::authenticate_and_create_session(&env, &player, &signature, &challenge, duration)
    }

    /// Submit a tap action through the session key.
    ///
    /// Validates the session, increments the tap counter, calculates combo
    /// streak (consecutive taps within N ledgers = multiplier), and updates
    /// the round score. This function is optimized to be as lightweight as
    /// possible for mobile-first gameplay.
    ///
    /// # Arguments
    /// * `session_key` - The player's session address
    ///
    /// # Returns
    /// `TapResult` with current count, combo, multiplier, and score earned
    pub fn tap(env: Env, session_key: Address) -> TapResult {
        let player = auth::validate_session(&env, &session_key);
        game::process_tap(&env, &player)
    }

    /// Activate a power-up through the session key.
    ///
    /// Consumes one power-up charge (earned through combo streaks) and
    /// applies the effect:
    /// - `0` (DoubleTap): +10 bonus points
    /// - `1` (Shield): Defensive effect
    /// - `2` (Burst): +25 bonus points
    ///
    /// # Arguments
    /// * `session_key` - The player's session address
    /// * `power_up` - Power-up kind (0 = DoubleTap, 1 = Shield, 2 = Burst)
    pub fn use_power_up(env: Env, session_key: Address, power_up: u32) {
        let player = auth::validate_session(&env, &session_key);
        game::activate_power_up(&env, &player, power_up);
    }

    /// Start a competitive round between two players.
    ///
    /// Both players must have registered passkeys. Round duration is measured
    /// in ledger sequences (not wall-clock time). Tap counters and power-ups
    /// are reset for both players.
    ///
    /// # Arguments
    /// * `player_a` - First player's address
    /// * `player_b` - Second player's address
    /// * `duration` - Round duration in ledger sequences
    pub fn start_round(env: Env, player_a: Address, player_b: Address, duration: u64) {
        game::start_round(&env, &player_a, &player_b, duration);
    }

    /// Get the current round state.
    ///
    /// If the round duration has elapsed, this auto-finalizes the round,
    /// declares the winner, and updates persistent player profiles.
    pub fn get_round(env: Env) -> RoundState {
        game::get_round(&env)
    }

    /// Get a player's persistent profile stats.
    ///
    /// Returns total wins, total taps, and best combo streak across
    /// all rounds played.
    pub fn get_profile(env: Env, player: Address) -> PlayerProfile {
        game::get_profile(&env, &player)
    }

    /// Get the cougr-core entity count (demonstrates ECS integration).
    pub fn get_entity_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::EntityCount)
            .unwrap_or(0)
    }
}
