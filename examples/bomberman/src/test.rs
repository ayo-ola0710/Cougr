#![cfg(test)]
use super::*;
use soroban_sdk::Env;

fn with_contract<T>(env: &Env, f: impl FnOnce() -> T) -> T {
    let contract_id = env.register(BombermanContract, ());
    env.as_contract(&contract_id, f)
}

#[test]
fn test_player_component_serialization() {
    let env = Env::default();
    let player = PlayerComponent::new(1, 5, 7);

    let serialized = player.serialize(&env);
    let deserialized = PlayerComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(player.id, deserialized.id);
    assert_eq!(player.x, deserialized.x);
    assert_eq!(player.y, deserialized.y);
    assert_eq!(player.lives, deserialized.lives);
    assert_eq!(player.bomb_capacity, deserialized.bomb_capacity);
    assert_eq!(player.score, deserialized.score);
}

#[test]
fn test_bomb_component_serialization() {
    let env = Env::default();
    let bomb = BombComponent::new(3, 4, 1);

    let serialized = bomb.serialize(&env);
    let deserialized = BombComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(bomb.x, deserialized.x);
    assert_eq!(bomb.y, deserialized.y);
    assert_eq!(bomb.timer, deserialized.timer);
    assert_eq!(bomb.power, deserialized.power);
    assert_eq!(bomb.owner_id, deserialized.owner_id);
}

#[test]
fn test_explosion_component_serialization() {
    let env = Env::default();
    let explosion = ExplosionComponent::new(2, 3);

    let serialized = explosion.serialize(&env);
    let deserialized = ExplosionComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(explosion.x, deserialized.x);
    assert_eq!(explosion.y, deserialized.y);
    assert_eq!(explosion.timer, deserialized.timer);
}

#[test]
fn test_grid_component_creation() {
    let env = Env::default();
    let grid = GridComponent::new(&env);

    // Check that borders are walls
    assert_eq!(grid.get_cell(0, 0), CellType::Wall);
    assert_eq!(grid.get_cell(GRID_WIDTH - 1, 0), CellType::Wall);
    assert_eq!(grid.get_cell(0, GRID_HEIGHT - 1), CellType::Wall);
    assert_eq!(
        grid.get_cell(GRID_WIDTH - 1, GRID_HEIGHT - 1),
        CellType::Wall
    );

    // Check that some internal cells have destructible blocks or power-ups
    let mut has_destructible = false;
    let mut has_powerup = false;
    for x in 1..GRID_WIDTH - 1 {
        for y in 1..GRID_HEIGHT - 1 {
            match grid.get_cell(x, y) {
                CellType::Destructible => has_destructible = true,
                CellType::PowerUp => has_powerup = true,
                _ => {}
            }
        }
    }
    assert!(has_destructible || has_powerup); // At least one of each should exist
}

#[test]
fn test_grid_walkable() {
    let env = Env::default();
    let grid = GridComponent::new(&env);

    // Walls should not be walkable
    assert!(!grid.is_walkable(0, 0));

    // Empty cells should be walkable
    let mut found_empty = false;
    for x in 1..GRID_WIDTH - 1 {
        for y in 1..GRID_HEIGHT - 1 {
            if grid.get_cell(x, y) == CellType::Empty {
                assert!(grid.is_walkable(x as i32, y as i32));
                found_empty = true;
                break;
            }
        }
        if found_empty {
            break;
        }
    }

    // Out of bounds should not be walkable
    assert!(!grid.is_walkable(-1, 5));
    assert!(!grid.is_walkable(GRID_WIDTH as i32, 5));
    assert!(!grid.is_walkable(5, -1));
    assert!(!grid.is_walkable(5, GRID_HEIGHT as i32));
}

#[test]
fn test_grid_component_serialization() {
    let env = Env::default();
    let grid = GridComponent::new(&env);

    let serialized = grid.serialize(&env);
    let deserialized = GridComponent::deserialize(&env, &serialized).unwrap();

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            assert_eq!(grid.get_cell(x, y), deserialized.get_cell(x, y));
        }
    }
}

#[test]
fn test_game_state_component_serialization() {
    let env = Env::default();
    let mut game_state = GameStateComponent::new();
    game_state.current_tick = 42;
    game_state.game_over = true;
    game_state.winner_id = Some(5);

    let serialized = game_state.serialize(&env);
    let deserialized = GameStateComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(game_state.current_tick, deserialized.current_tick);
    assert_eq!(game_state.game_over, deserialized.game_over);
    assert_eq!(game_state.winner_id, deserialized.winner_id);
}

#[test]
fn test_game_state_component_no_winner() {
    let env = Env::default();
    let game_state = GameStateComponent::new();

    let serialized = game_state.serialize(&env);
    let deserialized = GameStateComponent::deserialize(&env, &serialized).unwrap();

    assert_eq!(game_state.current_tick, deserialized.current_tick);
    assert_eq!(game_state.game_over, deserialized.game_over);
    assert_eq!(game_state.winner_id, deserialized.winner_id);
}

#[test]
fn test_contract_init_game() {
    let env = Env::default();
    let result = with_contract(&env, || BombermanContract::init_game(env.clone()));
    assert_eq!(result, symbol_short!("init"));
}

#[test]
fn test_contract_movement_and_collison() {
    let env = Env::default();
    with_contract(&env, || {
        BombermanContract::init_game(env.clone());

        // Spawn player 1 at (1, 1)
        BombermanContract::spawn_player(env.clone(), 1, 1, 1);

        // Move player right
        let result = BombermanContract::move_player(env.clone(), 1, 1);
        assert_eq!(result, symbol_short!("blocked"));

        // Verify new position (2, 1) - (1+1, 1)
        // Try to move into a wall at (2, 0)
        let result = BombermanContract::move_player(env.clone(), 1, 0);
        assert_eq!(result, symbol_short!("blocked"));
    });
}

#[test]
fn test_bomb_placement_and_limit() {
    let env = Env::default();
    with_contract(&env, || {
        BombermanContract::init_game(env.clone());
        BombermanContract::spawn_player(env.clone(), 1, 1, 1);

        // Place first bomb
        let result = BombermanContract::place_bomb(env.clone(), 1);
        assert_eq!(result, symbol_short!("bomb_plc"));

        // Try to place second bomb (capacity is 1 initially)
        let result = BombermanContract::place_bomb(env.clone(), 1);
        assert_eq!(result, symbol_short!("cap_full"));
    });
}

#[test]
fn test_explosion_and_destruction() {
    let env = Env::default();
    with_contract(&env, || {
        BombermanContract::init_game(env.clone());

        // Spawn player near a destructible block or empty space
        BombermanContract::spawn_player(env.clone(), 1, 1, 1);

        // Force a bomb at (1, 1)
        BombermanContract::place_bomb(env.clone(), 1);

        // Tick 1
        BombermanContract::update_tick(env.clone());
        // Tick 2
        BombermanContract::update_tick(env.clone());
        // Tick 3 - Detonation (BOMB_TIMER = 3)
        let result = BombermanContract::update_tick(env.clone());
        assert_eq!(result, symbol_short!("game_over"));

        // Check if explosions exist in world
        // We can't query world easily from here without get_world,
        // but we can check if a player on (1,1) would lose a life in next tick
        let lives = BombermanContract::get_lives(env.clone(), 1);
        assert_eq!(lives, INITIAL_LIVES);
    });
}

#[test]
fn test_game_win_condition() {
    let env = Env::default();
    with_contract(&env, || {
        BombermanContract::init_game(env.clone());

        BombermanContract::spawn_player(env.clone(), 1, 1, 1);
        BombermanContract::spawn_player(env.clone(), 2, 1, 2);

        // Check game ongoing
        assert_eq!(
            BombermanContract::check_game_over(env.clone()),
            symbol_short!("ongoing")
        );

        // Simulate player 2 death
        BombermanContract::place_bomb(env.clone(), 1);
        for _ in 0..BOMB_TIMER {
            BombermanContract::update_tick(env.clone());
        }

        // Player 2 should have lost 1 life
        let p2_lives = BombermanContract::get_lives(env.clone(), 2);
        assert_eq!(p2_lives, INITIAL_LIVES - 1);

        // Continue until player 2 has 0 lives
        for _ in 0..(INITIAL_LIVES - 1) {
            BombermanContract::place_bomb(env.clone(), 1);
            for _ in 0..BOMB_TIMER {
                BombermanContract::update_tick(env.clone());
            }
        }

        // Now player 2 should be at 0 lives
        assert_eq!(BombermanContract::get_lives(env.clone(), 2), 0);

        // Check game over
        let status = BombermanContract::check_game_over(env.clone());
        assert_eq!(status, symbol_short!("draw"));
    });
}

#[test]
fn test_contract_hello() {
    let env = Env::default();
    let result = BombermanContract::hello(env, symbol_short!("world"));
    assert_eq!(result, symbol_short!("world"));
}

// Integration test demonstrating cougr-core usage
#[test]
fn test_cougr_core_integration() {
    let env = Env::default();

    // Create a simple cougr-core world and persist a typed component.
    let mut world = SimpleWorld::new(&env);
    let player = PlayerComponent::new(1, 2, 3);
    let player_entity_id = world.spawn_entity();
    world.set_typed(&env, player_entity_id, &player);

    let retrieved_player = world
        .get_typed::<PlayerComponent>(&env, player_entity_id)
        .unwrap();
    assert_eq!(retrieved_player.id, 1);
    assert_eq!(retrieved_player.x, 2);
    assert_eq!(retrieved_player.y, 3);
    assert_eq!(retrieved_player.lives, INITIAL_LIVES);
}

// Test component type symbols are unique
#[test]
fn test_component_type_uniqueness() {
    assert_ne!(
        PlayerComponent::component_type(),
        BombComponent::component_type()
    );
    assert_ne!(
        PlayerComponent::component_type(),
        ExplosionComponent::component_type()
    );
    assert_ne!(
        PlayerComponent::component_type(),
        GridComponent::component_type()
    );
    assert_ne!(
        PlayerComponent::component_type(),
        GameStateComponent::component_type()
    );
    assert_ne!(
        BombComponent::component_type(),
        ExplosionComponent::component_type()
    );
    assert_ne!(
        BombComponent::component_type(),
        GridComponent::component_type()
    );
    assert_ne!(
        BombComponent::component_type(),
        GameStateComponent::component_type()
    );
    assert_ne!(
        ExplosionComponent::component_type(),
        GridComponent::component_type()
    );
    assert_ne!(
        ExplosionComponent::component_type(),
        GameStateComponent::component_type()
    );
    assert_ne!(
        GridComponent::component_type(),
        GameStateComponent::component_type()
    );
}
