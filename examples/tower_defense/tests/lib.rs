#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_init_game() {
        let env = Env::default();
        TowerDefenseContract::init_game(env.clone());
        let state = TowerDefenseContract::get_state(env.clone());
        assert!(state.contains("\"base_health\":100"));
        assert!(state.contains("\"current_wave\":1"));
        assert!(state.contains("\"status\":\"active\""));
    }

    #[test]
    fn test_place_tower() {
        let env = Env::default();
        TowerDefenseContract::init_game(env.clone());
        TowerDefenseContract::place_tower(env.clone(), 1, 1, 0);
        // Add assertions to verify tower placement
    }

    #[test]
    fn test_advance_tick() {
        let env = Env::default();
        TowerDefenseContract::init_game(env.clone());
        TowerDefenseContract::advance_tick(env.clone());
        // Add assertions to verify game state progression
    }

    #[test]
    fn test_is_finished() {
        let env = Env::default();
        TowerDefenseContract::init_game(env.clone());
        assert!(!TowerDefenseContract::is_finished(env.clone()));
        // Simulate game end and verify
    }
}