#![no_std]
use cougr_core::component::ComponentTrait;
use cougr_core::*;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Bytes, Env, Symbol, Vec};

mod test;

// Game constants
const GRID_WIDTH: usize = 15;
const GRID_HEIGHT: usize = 13;
const INITIAL_LIVES: u32 = 3;
const BOMB_TIMER: u32 = 3;
const EXPLOSION_DURATION: u32 = 1;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    World,
}

// Component definitions for Bomberman game
#[contracttype]
#[derive(Clone)]
pub struct PlayerComponent {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub lives: u32,
    pub bomb_capacity: u32,
    pub score: u32,
}

impl PlayerComponent {
    pub fn new(id: u32, x: i32, y: i32) -> Self {
        Self {
            id,
            x,
            y,
            lives: INITIAL_LIVES,
            bomb_capacity: 1,
            score: 0,
        }
    }
}

impl ComponentTrait for PlayerComponent {
    fn component_type() -> Symbol {
        symbol_short!("player")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.lives.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.bomb_capacity.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.score.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 24 {
            return None;
        }
        let id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let x = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let lives = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let bomb_capacity = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        let score = u32::from_be_bytes([
            data.get(20).unwrap(),
            data.get(21).unwrap(),
            data.get(22).unwrap(),
            data.get(23).unwrap(),
        ]);
        Some(Self {
            id,
            x,
            y,
            lives,
            bomb_capacity,
            score,
        })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct BombComponent {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
    pub power: u32,
    pub owner_id: u32,
}

impl BombComponent {
    pub fn new(x: i32, y: i32, owner_id: u32) -> Self {
        Self {
            x,
            y,
            timer: BOMB_TIMER,
            power: 1,
            owner_id,
        }
    }
}

impl ComponentTrait for BombComponent {
    fn component_type() -> Symbol {
        symbol_short!("bomb")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.timer.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.power.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.owner_id.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 20 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let timer = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let power = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        let owner_id = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);
        Some(Self {
            x,
            y,
            timer,
            power,
            owner_id,
        })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct ExplosionComponent {
    pub x: i32,
    pub y: i32,
    pub timer: u32,
}

impl ExplosionComponent {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            timer: EXPLOSION_DURATION,
        }
    }
}

impl ComponentTrait for ExplosionComponent {
    fn component_type() -> Symbol {
        symbol_short!("explosion")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.x.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.y.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.timer.to_be_bytes()));
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 12 {
            return None;
        }
        let x = i32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let y = i32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let timer = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        Some(Self { x, y, timer })
    }
}

// Grid cell types
#[contracttype]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    Empty = 0,
    Wall = 1,
    Destructible = 2,
    PowerUp = 3,
}

#[contracttype]
#[derive(Clone)]
pub struct GridComponent {
    pub cells: Vec<CellType>,
}

impl GridComponent {
    #[allow(clippy::if_same_then_else)]
    pub fn new(env: &Env) -> Self {
        let mut cells = Vec::new(env);

        for _ in 0..(GRID_WIDTH * GRID_HEIGHT) {
            cells.push_back(CellType::Empty);
        }

        // Initialize walls around the perimeter
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let index = y * GRID_WIDTH + x;
                if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                    cells.set(index as u32, CellType::Wall);
                } else if x % 2 == 0 && y % 2 == 0 {
                    cells.set(index as u32, CellType::Wall);
                }
            }
        }

        // Add some destructible blocks and power-ups
        for x in 1..GRID_WIDTH - 1 {
            for y in 1..GRID_HEIGHT - 1 {
                let index = y * GRID_WIDTH + x;
                if (x + y) % 3 == 0 && cells.get(index as u32).unwrap() == CellType::Empty {
                    cells.set(index as u32, CellType::Destructible);
                } else if (x + y) % 7 == 0 && cells.get(index as u32).unwrap() == CellType::Empty {
                    cells.set(index as u32, CellType::PowerUp);
                }
            }
        }

        Self { cells }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> CellType {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells
                .get((y * GRID_WIDTH + x) as u32)
                .unwrap_or(CellType::Wall)
        } else {
            CellType::Wall
        }
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell_type: CellType) {
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            self.cells.set((y * GRID_WIDTH + x) as u32, cell_type);
        }
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
            return false;
        }
        matches!(
            self.get_cell(x as usize, y as usize),
            CellType::Empty | CellType::PowerUp
        )
    }
}

impl ComponentTrait for GridComponent {
    fn component_type() -> Symbol {
        symbol_short!("grid")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for cell in self.cells.iter() {
            bytes.append(&Bytes::from_array(env, &[cell as u8]));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != (GRID_WIDTH * GRID_HEIGHT) as u32 {
            return None;
        }
        let mut cells = Vec::new(env);
        for i in 0..GRID_WIDTH * GRID_HEIGHT {
            let cell = match data.get(i as u32).unwrap() {
                0 => CellType::Empty,
                1 => CellType::Wall,
                2 => CellType::Destructible,
                3 => CellType::PowerUp,
                _ => return None,
            };
            cells.push_back(cell);
        }
        Some(Self { cells })
    }
}

#[contracttype]
#[derive(Clone)]
pub struct GameStateComponent {
    pub current_tick: u32,
    pub game_over: bool,
    pub winner_id: Option<u32>,
}

impl GameStateComponent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            current_tick: 0,
            game_over: false,
            winner_id: None,
        }
    }
}

impl ComponentTrait for GameStateComponent {
    fn component_type() -> Symbol {
        symbol_short!("gstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.current_tick.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &[if self.game_over { 1 } else { 0 }],
        ));
        match self.winner_id {
            Some(id) => {
                bytes.append(&Bytes::from_array(env, &[1]));
                bytes.append(&Bytes::from_array(env, &id.to_be_bytes()));
            }
            None => {
                bytes.append(&Bytes::from_array(env, &[0]));
            }
        }
        bytes
    }

    #[allow(unused_variables)]
    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }
        let current_tick = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let game_over = data.get(4).unwrap() != 0;
        let has_winner = data.get(5).unwrap() != 0;
        let winner_id = if has_winner && data.len() >= 10 {
            Some(u32::from_be_bytes([
                data.get(6).unwrap(),
                data.get(7).unwrap(),
                data.get(8).unwrap(),
                data.get(9).unwrap(),
            ]))
        } else {
            None
        };
        Some(Self {
            current_tick,
            game_over,
            winner_id,
        })
    }
}

#[contract]
pub struct BombermanContract;

#[allow(unused_variables)]
#[contractimpl]
impl BombermanContract {
    /// Initialize the game world using cougr-core ECS
    /// This demonstrates how cougr-core simplifies persistent game state management
    /// compared to vanilla Soroban where you'd manually handle storage keys and serialization
    ///
    /// Benefits of using cougr-core:
    /// - Declarative component-based architecture
    /// - Automatic serialization/deserialization through ComponentTrait
    /// - Entity-Component queries for efficient game logic
    /// - Clean separation of game state concerns
    pub fn init_game(env: Env) -> Symbol {
        let mut world = SimpleWorld::new(&env);

        // Spawn grid entity
        let grid = GridComponent::new(&env);
        let grid_entity = world.spawn_entity();
        world.set_typed(&env, grid_entity, &grid);

        // Spawn game state entity
        let game_state = GameStateComponent::new();
        let game_state_entity = world.spawn_entity();
        world.set_typed(&env, game_state_entity, &game_state);

        // Persist world
        env.storage().instance().set(&DataKey::World, &world);

        symbol_short!("init")
    }

    /// Spawn a new player at a given position
    pub fn spawn_player(env: Env, player_id: u32, x: i32, y: i32) -> Symbol {
        let mut world = Self::get_world(&env);

        // Check if player ID already exists
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    return symbol_short!("exists");
                }
            }
        }

        let player = PlayerComponent::new(player_id, x, y);
        let player_entity = world.spawn_entity();
        world.set_typed(&env, player_entity, &player);

        env.storage().instance().set(&DataKey::World, &world);
        symbol_short!("spawned")
    }

    /// Move a player in the specified direction
    /// Directions: 0=up, 1=right, 2=down, 3=left
    pub fn move_player(env: Env, player_id: u32, direction: u32) -> Symbol {
        let mut world = Self::get_world(&env);

        // Find player entity
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        let mut player_entity_opt = None;
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    player_entity_opt = Some((entity_id, player));
                    break;
                }
            }
        }

        let (player_entity, mut player) = match player_entity_opt {
            Some(p) => p,
            None => return symbol_short!("no_player"),
        };

        // Find grid entity
        let grid_entities =
            world.get_entities_with_component(&GridComponent::component_type(), &env);
        let grid_entity = match grid_entities.get(0) {
            Some(e) => e,
            None => return symbol_short!("no_grid"),
        };
        let grid = world.get_typed::<GridComponent>(&env, grid_entity).unwrap();

        // Calculate new position
        let (mut next_x, mut next_y) = (player.x, player.y);
        match direction {
            0 => next_y -= 1, // Up
            1 => next_x += 1, // Right
            2 => next_y += 1, // Down
            3 => next_x -= 1, // Left
            _ => return symbol_short!("inv_dir"),
        }

        // Validate move
        if !grid.is_walkable(next_x, next_y) {
            return symbol_short!("blocked");
        }

        // Check for explosions at target position
        let explosion_entities =
            world.get_entities_with_component(&ExplosionComponent::component_type(), &env);
        for e_id in explosion_entities.iter() {
            if let Some(explosion) = world.get_typed::<ExplosionComponent>(&env, e_id) {
                if explosion.x == next_x && explosion.y == next_y {
                    // Player died
                    if player.lives > 0 {
                        player.lives -= 1;
                    }
                    // Reset position to safe spot (optional, but requested logic is "death")
                    // For now, let's just decrement lives and stay put or move and lose life
                }
            }
        }

        player.x = next_x;
        player.y = next_y;
        world.set_typed(&env, player_entity, &player);

        // Save world
        env.storage().instance().set(&DataKey::World, &world);

        symbol_short!("moved")
    }

    fn get_world(env: &Env) -> SimpleWorld {
        env.storage()
            .instance()
            .get(&DataKey::World)
            .expect("Game not initialized")
    }

    /// Place a bomb at the player's current position
    pub fn place_bomb(env: Env, player_id: u32) -> Symbol {
        let mut world = Self::get_world(&env);

        // Find player position
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        let mut player_opt = None;
        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    player_opt = Some(player);
                    break;
                }
            }
        }

        let player = match player_opt {
            Some(p) => p,
            None => return symbol_short!("no_player"),
        };

        // Check current bomb count for this player
        let bomb_entities =
            world.get_entities_with_component(&BombComponent::component_type(), &env);
        let mut owned_bombs = 0;
        for b_id in bomb_entities.iter() {
            if let Some(bomb) = world.get_typed::<BombComponent>(&env, b_id) {
                if bomb.owner_id == player_id {
                    owned_bombs += 1;
                }
            }
        }

        if owned_bombs >= player.bomb_capacity {
            return symbol_short!("cap_full");
        }

        // Check if a bomb already exists at this position
        for b_id in bomb_entities.iter() {
            if let Some(bomb) = world.get_typed::<BombComponent>(&env, b_id) {
                if bomb.x == player.x && bomb.y == player.y {
                    return symbol_short!("exists");
                }
            }
        }

        // Create bomb entity
        let bomb = BombComponent::new(player.x, player.y, player_id);
        let bomb_entity = world.spawn_entity();
        world.set_typed(&env, bomb_entity, &bomb);

        // Save world
        env.storage().instance().set(&DataKey::World, &world);

        symbol_short!("bomb_plc")
    }

    /// Advance the game tick - handle timers, explosions, collisions
    /// This is where cougr-core's ECS shines for complex game logic
    pub fn update_tick(env: Env) -> Symbol {
        let mut world = Self::get_world(&env);

        // Update game state tick
        let state_entities =
            world.get_entities_with_component(&GameStateComponent::component_type(), &env);
        let state_entity = state_entities.get(0).expect("No game state");
        let mut game_state = world
            .get_typed::<GameStateComponent>(&env, state_entity)
            .unwrap();

        if game_state.game_over {
            return symbol_short!("game_over");
        }

        game_state.current_tick += 1;

        // Find grid
        let grid_entities =
            world.get_entities_with_component(&GridComponent::component_type(), &env);
        let grid_entity = grid_entities.get(0).expect("No grid");
        let mut grid = world.get_typed::<GridComponent>(&env, grid_entity).unwrap();

        // 1. Process explosion timers
        let explosion_entities =
            world.get_entities_with_component(&ExplosionComponent::component_type(), &env);
        for e_id in explosion_entities.iter() {
            if let Some(mut explosion) = world.get_typed::<ExplosionComponent>(&env, e_id) {
                if explosion.timer > 0 {
                    explosion.timer -= 1;
                }
                if explosion.timer == 0 {
                    world.despawn_entity(e_id);
                } else {
                    world.set_typed(&env, e_id, &explosion);
                }
            }
        }

        // 2. Process bomb timers and trigger new explosions
        let bomb_entities =
            world.get_entities_with_component(&BombComponent::component_type(), &env);
        for b_id in bomb_entities.iter() {
            if let Some(mut bomb) = world.get_typed::<BombComponent>(&env, b_id) {
                if bomb.timer > 0 {
                    bomb.timer -= 1;
                }
                if bomb.timer == 0 {
                    // Detonate!
                    Self::detonate_bomb(&env, &mut world, &mut grid, &bomb);
                    world.despawn_entity(b_id);
                } else {
                    world.set_typed(&env, b_id, &bomb);
                }
            }
        }

        // 3. Check for player deaths (collision with explosions)
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);
        let active_explosions =
            world.get_entities_with_component(&ExplosionComponent::component_type(), &env);

        for p_id in player_entities.iter() {
            if let Some(mut player) = world.get_typed::<PlayerComponent>(&env, p_id) {
                if player.lives == 0 {
                    continue;
                }

                let mut hit = false;
                for e_id in active_explosions.iter() {
                    if let Some(explosion) = world.get_typed::<ExplosionComponent>(&env, e_id) {
                        if explosion.x == player.x && explosion.y == player.y {
                            hit = true;
                            break;
                        }
                    }
                }

                if hit {
                    player.lives -= 1;
                    world.set_typed(&env, p_id, &player);
                }
            }
        }

        // 4. Update Grid and GameState
        world.set_typed(&env, grid_entity, &grid);

        // Win/loss detection
        let mut alive_players = 0;
        let mut last_alive_id = 0;
        for p_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, p_id) {
                if player.lives > 0 {
                    alive_players += 1;
                    last_alive_id = player.id;
                }
            }
        }

        if alive_players <= 1 {
            game_state.game_over = true;
            if alive_players == 1 {
                game_state.winner_id = Some(last_alive_id);
            }
        }

        world.set_typed(&env, state_entity, &game_state);

        // Save world
        env.storage().instance().set(&DataKey::World, &world);

        symbol_short!("tick_upd")
    }

    fn detonate_bomb(
        env: &Env,
        world: &mut SimpleWorld,
        grid: &mut GridComponent,
        bomb: &BombComponent,
    ) {
        // Spawn center explosion
        let center_exp = ExplosionComponent::new(bomb.x, bomb.y);
        let center_id = world.spawn_entity();
        world.set_typed(env, center_id, &center_exp);

        // Propagate in 4 directions
        let dirs = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        for (dx, dy) in dirs {
            for dist in 1..=bomb.power {
                let x = bomb.x + dx * dist as i32;
                let y = bomb.y + dy * dist as i32;

                if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
                    break;
                }

                let cell = grid.get_cell(x as usize, y as usize);
                if cell == CellType::Wall {
                    break;
                }

                // Spawn explosion
                let exp = ExplosionComponent::new(x, y);
                let exp_id = world.spawn_entity();
                world.set_typed(env, exp_id, &exp);

                if cell == CellType::Destructible {
                    grid.set_cell(x as usize, y as usize, CellType::Empty);
                    break; // Blocked by destructible but destroys it
                }

                if cell == CellType::PowerUp {
                    grid.set_cell(x as usize, y as usize, CellType::Empty);
                    // Power-up destroyed by explosion
                }
            }
        }
    }

    /// Get the current score for a player
    pub fn get_score(env: Env, player_id: u32) -> u32 {
        let world = Self::get_world(&env);
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);

        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    return player.score;
                }
            }
        }
        0
    }

    /// Get the current lives for a player
    pub fn get_lives(env: Env, player_id: u32) -> u32 {
        let world = Self::get_world(&env);
        let player_entities =
            world.get_entities_with_component(&PlayerComponent::component_type(), &env);

        for entity_id in player_entities.iter() {
            if let Some(player) = world.get_typed::<PlayerComponent>(&env, entity_id) {
                if player.id == player_id {
                    return player.lives;
                }
            }
        }
        0
    }

    /// Check if the game is over and return winner if any
    pub fn check_game_over(env: Env) -> Symbol {
        let world = Self::get_world(&env);
        let state_entities =
            world.get_entities_with_component(&GameStateComponent::component_type(), &env);
        let state_entity = match state_entities.get(0) {
            Some(e) => e,
            None => return symbol_short!("no_state"),
        };
        let game_state = world
            .get_typed::<GameStateComponent>(&env, state_entity)
            .unwrap();

        if game_state.game_over {
            match game_state.winner_id {
                Some(_) => symbol_short!("winner"),
                None => symbol_short!("draw"),
            }
        } else {
            symbol_short!("ongoing")
        }
    }

    pub fn hello(env: Env, to: Symbol) -> Symbol {
        to
    }
}
