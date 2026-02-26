use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, Vec};

fn setup_game() -> (Env, BattleshipContractClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(BattleshipContract, ());
    let client = BattleshipContractClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);

    (env, client, player_a, player_b)
}

fn compute_merkle_root(env: &Env, board: &[u32; 100]) -> (BytesN<32>, Vec<Vec<BytesN<32>>>) {
    let mut leaves = Vec::new(env);
    for (idx, &value) in board.iter().enumerate() {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_array(env, &(idx as u32).to_be_bytes()));
        data.append(&Bytes::from_array(env, &value.to_be_bytes()));
        leaves.push_back(env.crypto().sha256(&data).into());
    }

    // Build Merkle tree (simplified for 100 leaves -> 128 padded)
    let mut tree: Vec<Vec<BytesN<32>>> = Vec::new(env);
    let mut current_level = leaves.clone();

    // Pad to power of 2
    let zero_hash: BytesN<32> = env.crypto().sha256(&Bytes::new(env)).into();
    while current_level.len() < 128 {
        current_level.push_back(zero_hash.clone());
    }

    tree.push_back(current_level.clone());

    // Build tree levels
    while current_level.len() > 1 {
        let mut next_level = Vec::new(env);
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level.get(i).unwrap();
            let right = current_level.get(i + 1).unwrap();

            let mut combined = Bytes::new(env);
            for j in 0..32 {
                combined.push_back(left.get(j).unwrap());
            }
            for j in 0..32 {
                combined.push_back(right.get(j).unwrap());
            }
            next_level.push_back(env.crypto().sha256(&combined).into());
        }
        tree.push_back(next_level.clone());
        current_level = next_level;
    }

    (current_level.get(0).unwrap(), tree)
}

fn get_merkle_proof(env: &Env, tree: &Vec<Vec<BytesN<32>>>, index: u32) -> Vec<BytesN<32>> {
    let mut proof = Vec::new(env);
    let mut idx = index;

    for level in 0..(tree.len() - 1) {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
        let level_vec = tree.get(level).unwrap();
        proof.push_back(level_vec.get(sibling_idx).unwrap());
        idx /= 2;
    }

    proof
}

fn make_commitment(env: &Env, board: &[u32; 100], salt: &BytesN<32>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    for &cell in board.iter() {
        data.append(&Bytes::from_array(env, &cell.to_be_bytes()));
    }
    for i in 0..32 {
        data.push_back(salt.get(i).unwrap());
    }
    env.crypto().sha256(&data).into()
}

#[test]
fn test_new_game() {
    let (_env, client, player_a, player_b) = setup_game();

    client.new_game(&player_a, &player_b);

    let state = client.get_state();
    assert_eq!(state.player_a, player_a);
    assert_eq!(state.player_b, player_b);
    assert_eq!(state.turn_state.phase, Phase::Setup);
}

#[test]
fn test_commit_board() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let mut board_a = [0u32; 100];
    board_a[0] = 1; // Ship at (0,0)
    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let (root_a, _) = compute_merkle_root(&env, &board_a);

    client.commit_board(&player_a, &commitment_a, &root_a);

    let state = client.get_state();
    assert_eq!(state.turn_state.phase, Phase::Setup);

    let mut board_b = [0u32; 100];
    board_b[1] = 1;
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);
    let (root_b, _) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_b, &commitment_b, &root_b);

    let state = client.get_state();
    assert_eq!(state.turn_state.phase, Phase::Attack);
}

#[test]
fn test_attack_and_reveal_miss() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    // Setup boards
    let mut board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[10] = 1; // Ship at (0,1)

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, tree_a) = compute_merkle_root(&env, &board_a);
    let (root_b, tree_b) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Player A attacks (0,0) on B's board - miss
    client.attack(&player_a, &0, &0);

    let state = client.get_state();
    assert!(state.turn_state.has_pending);

    // Player B reveals
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof);

    let state = client.get_state();
    assert!(!state.turn_state.has_pending);
    assert_eq!(state.attack_grid_b.cells.get(0).unwrap(), CellResult::Miss);
}

#[test]
fn test_attack_and_reveal_hit() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let mut board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1; // Ship at (0,0)

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = compute_merkle_root(&env, &board_a);
    let (root_b, tree_b) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Player A attacks (0,0) - hit
    client.attack(&player_a, &0, &0);

    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &1, &proof);

    let state = client.get_state();
    assert_eq!(state.attack_grid_b.cells.get(0).unwrap(), CellResult::Hit);
    assert_eq!(state.ship_status.remaining_b, 16);
}

#[test]
#[should_panic(expected = "Invalid proof")]
fn test_invalid_proof_rejected() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let mut board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1;

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = compute_merkle_root(&env, &board_a);
    let (root_b, tree_b) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    client.attack(&player_a, &0, &0);

    // Try to reveal with wrong value (claim miss when it's hit)
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof); // Wrong value
}

#[test]
#[should_panic(expected = "Already attacked")]
fn test_cannot_attack_same_cell_twice() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, tree_a) = compute_merkle_root(&env, &board_a);
    let (root_b, tree_b) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // A attacks (0,0) on B's board
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof);

    // B attacks (1,1) on A's board
    client.attack(&player_b, &1, &1);
    let proof = get_merkle_proof(&env, &tree_a, 11);
    client.reveal_cell(&player_a, &1, &1, &0, &proof);

    // A tries to attack (0,0) on B's board again - should panic
    client.attack(&player_a, &0, &0);
}

#[test]
#[should_panic(expected = "Not your turn")]
fn test_turn_enforcement() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = compute_merkle_root(&env, &board_a);
    let (root_b, _) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Player B tries to attack when it's A's turn
    client.attack(&player_b, &0, &0);
}

#[test]
fn test_win_condition() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let mut board_b = [0u32; 100];
    board_b[0] = 1; // Only one ship cell

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, _) = compute_merkle_root(&env, &board_a);
    let (root_b, tree_b) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    // Manually set remaining to 1 for quick test
    let mut state = client.get_state();
    state.ship_status.remaining_b = 1;
    env.as_contract(&client.address, || {
        env.storage().instance().set(&GAME_KEY, &state);
    });

    // Attack and sink last ship
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &1, &proof);

    let state = client.get_state();
    assert_eq!(state.turn_state.phase, Phase::Finished);
    assert_eq!(state.winner, Some(player_a));
}

#[test]
fn test_component_trait() {
    let env = Env::default();

    let commitment = BoardCommitment {
        commitment: BytesN::from_array(&env, &[0u8; 32]),
        merkle_root: BytesN::from_array(&env, &[1u8; 32]),
    };

    let serialized = commitment.serialize(&env);
    assert_eq!(serialized.len(), 64);
    assert_eq!(BoardCommitment::component_type(), symbol_short!("board"));
}

#[test]
fn test_turn_switching() {
    let (env, client, player_a, player_b) = setup_game();
    env.mock_all_auths();

    client.new_game(&player_a, &player_b);

    let board_a = [0u32; 100];
    let board_b = [0u32; 100];

    let salt_a = BytesN::from_array(&env, &[1u8; 32]);
    let salt_b = BytesN::from_array(&env, &[2u8; 32]);

    let commitment_a = make_commitment(&env, &board_a, &salt_a);
    let commitment_b = make_commitment(&env, &board_b, &salt_b);

    let (root_a, tree_a) = compute_merkle_root(&env, &board_a);
    let (root_b, tree_b) = compute_merkle_root(&env, &board_b);

    client.commit_board(&player_a, &commitment_a, &root_a);
    client.commit_board(&player_b, &commitment_b, &root_b);

    let state = client.get_state();
    assert_eq!(state.turn_state.current_player, player_a);

    // A attacks
    client.attack(&player_a, &0, &0);
    let proof = get_merkle_proof(&env, &tree_b, 0);
    client.reveal_cell(&player_b, &0, &0, &0, &proof);

    let state = client.get_state();
    assert_eq!(state.turn_state.current_player, player_b);

    // B attacks
    client.attack(&player_b, &1, &1);
    let proof = get_merkle_proof(&env, &tree_a, 11);
    client.reveal_cell(&player_a, &1, &1, &0, &proof);

    let state = client.get_state();
    assert_eq!(state.turn_state.current_player, player_a);
}
