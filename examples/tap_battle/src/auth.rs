//! Authentication and session management for Tap Battle.
//!
//! This module wraps cougr-core's secp256r1 authentication and SessionBuilder
//! to provide the mobile-first auth flow:
//! 1. Player registers a passkey (secp256r1 public key)
//! 2. Player authenticates with biometrics → signature verified on-chain
//! 3. SessionBuilder creates a scoped gameplay session (no per-tx wallet prompts)

use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, Symbol};

use cougr_core::accounts::secp256r1_auth::{verify_secp256r1, Secp256r1Key, Secp256r1Storage};
use cougr_core::accounts::session_builder::SessionBuilder;

use crate::types::*;

// ============================================================================
// AuthSystem — Passkey registration and verification
// ============================================================================

/// Register a secp256r1 passkey for a player.
///
/// Stores the public key via `Secp256r1Storage` and creates a `PasskeyIdentity`
/// component. This replaces seed phrases with biometric authentication.
pub fn register_passkey(env: &Env, player: &Address, pubkey: &BytesN<65>) {
    // Require player authorization
    player.require_auth();

    // Store key via cougr-core's Secp256r1Storage
    let key = Secp256r1Key {
        public_key: pubkey.clone(),
        label: symbol_short!("passkey"),
        registered_at: env.ledger().sequence() as u64,
    };
    Secp256r1Storage::store(env, player, &key);

    // Store PasskeyIdentity component
    let identity = PasskeyIdentity {
        pubkey: pubkey.clone(),
        registered_at: env.ledger().sequence() as u64,
    };
    env.storage()
        .persistent()
        .set(&DataKey::Passkey(player.clone()), &identity);
}

/// Authenticate a player via passkey and create a gameplay session.
///
/// Verifies the secp256r1 signature against the stored public key, then
/// uses `SessionBuilder` to create a session scoped to `tap` and `use_power`
/// actions. The session key allows gasless gameplay.
///
/// # Returns
/// The session key address for subsequent gameplay calls.
pub fn authenticate_and_create_session(
    env: &Env,
    player: &Address,
    signature: &Bytes,
    challenge: &BytesN<32>,
    duration: u64,
) -> Address {
    // Load stored passkey
    let identity: PasskeyIdentity = env
        .storage()
        .persistent()
        .get(&DataKey::Passkey(player.clone()))
        .expect("player has no registered passkey");

    // Verify the secp256r1 signature against the stored public key
    let message = Bytes::from_slice(env, &challenge.to_array());
    // Convert Bytes signature to BytesN<64> for verify_secp256r1
    let sig_array: BytesN<64> = signature.try_into().expect("signature must be 64 bytes");
    verify_secp256r1(env, &identity.pubkey, &message, &sig_array)
        .expect("invalid passkey signature");

    // Create a scoped gameplay session via SessionBuilder
    let session_scope = SessionBuilder::new(env)
        .allow_action(symbol_short!("tap"))
        .allow_action(Symbol::new(env, "use_power_up"))
        .max_operations(DEFAULT_MAX_OPS)
        .expires_at(env.ledger().sequence() as u64 + duration)
        .build_scope();

    // Store session state on-chain
    let session = SessionState {
        player: player.clone(),
        expires_at: session_scope.expires_at,
        ops_remaining: session_scope.max_operations,
    };
    env.storage()
        .persistent()
        .set(&DataKey::Session(player.clone()), &session);

    // Initialize tap counter for this player
    env.storage()
        .persistent()
        .set(&DataKey::TapState(player.clone()), &TapCounter::new());

    // Initialize power-up (DoubleTap by default)
    let power_up = PowerUp::new(PowerUpKind::DoubleTap as u32);
    env.storage()
        .persistent()
        .set(&DataKey::PowerUpState(player.clone()), &power_up);

    player.clone()
}

// ============================================================================
// SessionSystem — Session validation
// ============================================================================

/// Validate that a session is still active and decrement operations.
///
/// Checks expiration and remaining operations. Returns the player address
/// associated with the session.
///
/// # Panics
/// Panics if the session has expired or has no remaining operations.
pub fn validate_session(env: &Env, session_key: &Address) -> Address {
    let mut session: SessionState = env
        .storage()
        .persistent()
        .get(&DataKey::Session(session_key.clone()))
        .expect("no active session");

    // Check expiration (ledger-based)
    assert!(
        (env.ledger().sequence() as u64) < session.expires_at,
        "session expired"
    );

    // Check operations remaining
    assert!(session.ops_remaining > 0, "session operations exhausted");

    // Decrement operations
    session.ops_remaining -= 1;
    env.storage()
        .persistent()
        .set(&DataKey::Session(session_key.clone()), &session);

    session.player.clone()
}
