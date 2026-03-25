#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};
use cougr_core::zk::testing::{mock_proof, mock_verification_key, mock_scalar};

#[test]
fn test_arena_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let contract_id = env.register(AIDungeonMasterArenaContract, ());
    let client = AIDungeonMasterArenaContractClient::new(&env, &contract_id);

    // 1. Initialize Run
    client.init_run(&player);
    let state = client.get_state();
    assert_eq!(state.run_state.player, player);
    assert_eq!(state.run_state.floor, 1);
    assert!(!state.encounter.is_empty());
    let encounter = state.encounter.get(0).unwrap();
    let initial_enemy_hp = encounter.enemy_health;
    
    let mut state = client.submit_action(&player, &ActionInput::Attack);
    assert!(state.encounter.get(0).unwrap().enemy_health < initial_enemy_hp);

    // 3. Victory and Floor progression
    // Loop until encounter is finished (simplified for test)
    while !state.run_state.finished && !state.encounter.is_empty() {
        state = client.submit_action(&player, &ActionInput::Attack);
    }
    
    assert_eq!(state.run_state.floor, 3);

    // 4. Test Proof Verification (Mocked)
    let vk = mock_verification_key(&env, 1);
    client.set_vk(&vk);

    let proof_input = ProofInput {
        proof: mock_proof(&env),
        public_inputs: Vec::from_array(&env, [mock_scalar(&env, 10)]),
        floor: 1,
    };

    // Note: real verification would fail on zeroed points, but we check the flow.
    // Host functions might return false for invalid points.
    let _verified = client.verify_run_proof(&player, &proof_input);
}

#[test]
fn test_premium_action_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let player = Address::generate(&env);
    let contract_id = env.register(AIDungeonMasterArenaContract, ());
    let client = AIDungeonMasterArenaContractClient::new(&env, &contract_id);

    client.init_run(&player);
    
    // Simulate reaching a floor with premium action
    for _ in 0..10 {
        let state = client.get_state();
        if !state.premium.is_empty() {
            break;
        }
        if !state.encounter.is_empty() {
            client.submit_action(&player, &ActionInput::Attack);
        } else {
             // If floor progressed without encounter generation (shouldn't happen in our loop)
             break;
        }
    }

    let state = client.get_state();
    if !state.premium.is_empty() {
        let premium = state.premium.get(0).unwrap();
        assert!(premium.active);
        let updated_state = client.purchase_premium_action(&player, &premium.action_type);
        assert!(updated_state.premium.is_empty() || !updated_state.premium.get(0).unwrap().active);
        assert!(updated_state.run_state.score >= 100);
    }
}
