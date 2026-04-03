//! Public API contract smoke tests.
//!
//! These tests are intentionally shallow: they verify that the sanctioned
//! public entrypoints remain available and that stable versus experimental
//! namespaces stay explicit.

use cougr_core::accounts::{
    verify_secp256r1, ClassicAccount, GameAction, Secp256r1Key, Secp256r1Storage, SessionBuilder,
};
use cougr_core::zk::experimental::{
    bytes32_to_scalar, u32_to_scalar, CustomCircuit, GameCircuit, MovementCircuit,
};
use cougr_core::zk::stable::{encode_commit_reveal, CommitReveal, COMMIT_REVEAL_TYPE};
use cougr_core::{Position, SimpleWorld};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

#[test]
fn sanctioned_root_api_supports_basic_ecs_flow() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let entity = world.spawn_entity();

    world.set_typed(&env, entity, &Position::new(3, 4));

    let pos: Position = world.get_typed(&env, entity).unwrap();
    assert_eq!(pos.x, 3);
    assert_eq!(pos.y, 4);
}

#[test]
fn stable_zk_namespace_exposes_stable_commit_reveal_flow() {
    let env = Env::default();
    let commitment = BytesN::from_array(&env, &[9u8; 32]);
    let encoded = encode_commit_reveal(&env, &commitment, 123, false);

    assert!(!COMMIT_REVEAL_TYPE.is_empty());
    assert_eq!(encoded.len(), 41);

    let _stable_component: Option<CommitReveal> = None;
}

#[test]
fn experimental_zk_namespace_exposes_proof_helpers_explicitly() {
    let env = Env::default();
    let g1 = cougr_core::zk::G1Point {
        bytes: BytesN::from_array(&env, &[0u8; 64]),
    };
    let g2 = cougr_core::zk::G2Point {
        bytes: BytesN::from_array(&env, &[0u8; 128]),
    };
    let mut ic = Vec::new(&env);
    for _ in 0..6 {
        ic.push_back(g1.clone());
    }
    let vk = cougr_core::zk::VerificationKey {
        alpha: g1.clone(),
        beta: g2.clone(),
        gamma: g2.clone(),
        delta: g2,
        ic,
    };

    let _movement = MovementCircuit::new(vk.clone(), 10);
    let _game_circuit: &dyn GameCircuit = &_movement;
    let _custom = CustomCircuit::new(
        vk,
        vec![
            u32_to_scalar(&env, 42),
            bytes32_to_scalar(&BytesN::from_array(&env, &[1u8; 32])),
        ],
    );
}

#[test]
fn accounts_namespace_exposes_curated_beta_entrypoints() {
    let env = Env::default();
    let account = Address::generate(&env);
    let _classic = ClassicAccount::new(account.clone());

    let _action = GameAction {
        system_name: symbol_short!("move"),
        data: Bytes::new(&env),
    };

    let _session_builder = SessionBuilder::new(&env).allow_action(symbol_short!("move"));

    let key = Secp256r1Key {
        public_key: BytesN::from_array(&env, &[4u8; 65]),
        label: symbol_short!("passkey"),
        registered_at: 0,
    };
    let _storage_marker = core::mem::size_of::<Secp256r1Storage>();
    let _stored_key = key;

    let _verify_fn: fn(&Env, &BytesN<65>, &Bytes, &BytesN<64>) -> Result<(), cougr_core::accounts::AccountError> =
        verify_secp256r1;
}
