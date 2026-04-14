#![no_std]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

/// Board dimensions: 7 columns x 6 rows
const ROWS: u32 = 6;
const COLS: u32 = 7;

/// Cell value: 0=Empty, 1=Player1, 2=Player2
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Empty = 0,
    Player1 = 1,
    Player2 = 2,
}

/// Board component - stores the 7x6 game board state
#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardComponent {
    pub cells: Vec<u32>, // Flattened 2D array: index = row * COLS + col
    pub entity_id: u32,
}

impl BoardComponent {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        let mut cells = Vec::new(env);
        for _ in 0..(ROWS * COLS) {
            cells.push_back(0u32);
        }
        Self { cells, entity_id }
    }

    /// Get cell value at (row, col)
    pub fn get_cell(&self, _env: &Env, row: u32, col: u32) -> u32 {
        if row >= ROWS || col >= COLS {
            return 0;
        }
        let index = row * COLS + col;
        self.cells.get(index).unwrap_or(0)
    }

    /// Set cell value at (row, col)
    pub fn set_cell(&mut self, _env: &Env, row: u32, col: u32, value: u32) {
        if row >= ROWS || col >= COLS {
            return;
        }
        let index = row * COLS + col;
        self.cells.set(index, value);
    }

    /// Find the lowest empty row in a column (gravity-based placement)
    pub fn get_lowest_empty_row(&self, _env: &Env, col: u32) -> Option<u32> {
        if col >= COLS {
            return None;
        }

        // Start from bottom row and go up
        for row in (0..ROWS).rev() {
            let index = row * COLS + col;
            let cell = self.cells.get(index).unwrap_or(0);
            if cell == 0 {
                return Some(row);
            }
        }
        None // Column is full
    }

    /// Check if column is full
    pub fn is_column_full(&self, env: &Env, col: u32) -> bool {
        self.get_lowest_empty_row(env, col).is_none()
    }
}

impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.cells.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let cell = self.cells.get(i).unwrap_or(0);
            bytes.append(&Bytes::from_array(env, &cell.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let len = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);

        let mut cells = Vec::new(env);
        for i in 0..len {
            let offset = 8 + i * 4;
            if offset + 4 > data.len() {
                break;
            }
            let cell = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            cells.push_back(cell);
        }
        Some(Self { cells, entity_id })
    }
}

/// Player component - stores both players' addresses
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerComponent {
    pub player_one: Address,
    pub player_two: Address,
    pub entity_id: u32,
}

impl PlayerComponent {
    pub fn new(player_one: Address, player_two: Address, entity_id: u32) -> Self {
        Self {
            player_one,
            player_two,
            entity_id,
        }
    }
}

/// Game state component
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStateComponent {
    pub is_player_one_turn: bool,
    pub move_count: u32,
    pub status: u32, // 0=InProgress, 1=Player1Wins, 2=Player2Wins, 3=Draw
    pub last_move_col: Option<u32>,
    pub entity_id: u32,
}

impl GameStateComponent {
    pub fn new(entity_id: u32) -> Self {
        Self {
            is_player_one_turn: true,
            move_count: 0,
            status: 0,
            last_move_col: None,
            entity_id,
        }
    }
}

impl ComponentTrait for GameStateComponent {
    fn component_type() -> Symbol {
        symbol_short!("gstate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &[if self.is_player_one_turn { 1 } else { 0 }],
        ));
        bytes.append(&Bytes::from_array(env, &self.move_count.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.status.to_be_bytes()));

        // Serialize Option<u32> for last_move_col
        match self.last_move_col {
            Some(col) => {
                bytes.append(&Bytes::from_array(env, &[1u8])); // Some
                bytes.append(&Bytes::from_array(env, &col.to_be_bytes()));
            }
            None => {
                bytes.append(&Bytes::from_array(env, &[0u8])); // None
            }
        }

        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 14 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let is_player_one_turn = data.get(4).unwrap() != 0;
        let move_count = u32::from_be_bytes([
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
            data.get(8).unwrap(),
        ]);
        let status = u32::from_be_bytes([
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
            data.get(12).unwrap(),
        ]);

        let has_last_move = data.get(13).unwrap() != 0;
        let last_move_col = if has_last_move && data.len() >= 18 {
            let col = u32::from_be_bytes([
                data.get(14).unwrap(),
                data.get(15).unwrap(),
                data.get(16).unwrap(),
                data.get(17).unwrap(),
            ]);
            Some(col)
        } else {
            None
        };

        Some(Self {
            is_player_one_turn,
            move_count,
            status,
            last_move_col,
            entity_id,
        })
    }
}

/// ECS World State - stores all game entities and components
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub board: BoardComponent,
    pub players: PlayerComponent,
    pub game_state: GameStateComponent,
    pub next_entity_id: u32,
}

/// External game state for API consumers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub board: Vec<u32>, // Flattened 7x6 board
    pub rows: u32,
    pub cols: u32,
    pub player_one: Address,
    pub player_two: Address,
    pub is_player_one_turn: bool,
    pub move_count: u32,
    pub status: u32, // 0=InProgress, 1=P1Wins, 2=P2Wins, 3=Draw
    pub last_move_col: Option<u32>,
}

/// Move result returned after each move
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DropResult {
    pub success: bool,
    pub game_state: GameState,
    pub message: Symbol,
    pub row_placed: Option<u32>,
}

const WORLD_KEY: Symbol = symbol_short!("WORLD");

#[contract]
pub struct ConnectFourContract;

#[contractimpl]
impl ConnectFourContract {
    /// Initialize a new game with two players
    pub fn init_game(env: Env, player_one: Address, player_two: Address) -> GameState {
        let mut next_entity_id = 0u32;

        let board = BoardComponent::new(&env, next_entity_id);
        next_entity_id += 1;

        let players = PlayerComponent::new(player_one.clone(), player_two.clone(), next_entity_id);
        next_entity_id += 1;

        let game_state = GameStateComponent::new(next_entity_id);
        next_entity_id += 1;

        let world_state = ECSWorldState {
            board,
            players,
            game_state,
            next_entity_id,
        };

        env.storage().instance().set(&WORLD_KEY, &world_state);
        Self::to_game_state(&env, &world_state)
    }

    /// Drop a piece in a column (0-6)
    /// Gravity automatically places piece in lowest available row
    pub fn drop_piece(env: Env, player: Address, column: u32) -> DropResult {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // Validation system
        let validation = Self::validation_system(&world_state, &player, column);
        if !validation.0 {
            return DropResult {
                success: false,
                game_state: Self::to_game_state(&env, &world_state),
                message: validation.1,
                row_placed: None,
            };
        }

        // Gravity placement system
        let row_placed = Self::gravity_system(&mut world_state, column);

        // Execution system - place the piece
        Self::execution_system(&mut world_state, row_placed, column);

        // Win detection system
        Self::win_detection_system(&mut world_state);

        // Draw detection system
        Self::draw_system(&mut world_state);

        // Turn system
        Self::turn_system(&mut world_state);

        env.storage().instance().set(&WORLD_KEY, &world_state);

        DropResult {
            success: true,
            game_state: Self::to_game_state(&env, &world_state),
            message: symbol_short!("ok"),
            row_placed: Some(row_placed),
        }
    }

    /// Get the current game state
    pub fn get_state(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        Self::to_game_state(&env, &world_state)
    }

    /// Get the board state as a flattened vector
    pub fn get_board(env: Env) -> Vec<u32> {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.board.cells
    }

    /// Check if a column is valid (within bounds and not full)
    pub fn is_valid_column(env: Env, column: u32) -> bool {
        if column >= COLS {
            return false;
        }

        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if world_state.game_state.status != 0 {
            return false;
        }

        !world_state.board.is_column_full(&env, column)
    }

    /// Check if the game is finished
    pub fn is_finished(env: Env) -> bool {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.game_state.status != 0
    }

    /// Get the winner's address if game is over
    pub fn get_winner(env: Env) -> Option<Address> {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        match world_state.game_state.status {
            1 => Some(world_state.players.player_one),
            2 => Some(world_state.players.player_two),
            _ => None,
        }
    }

    /// Reset the game with the same players
    pub fn reset_game(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        Self::init_game(
            env,
            world_state.players.player_one,
            world_state.players.player_two,
        )
    }

    /// Validation System - checks if move is legal
    fn validation_system(world: &ECSWorldState, player: &Address, column: u32) -> (bool, Symbol) {
        // Check if game is over
        if world.game_state.status != 0 {
            return (false, symbol_short!("gameover"));
        }

        // Check column bounds
        if column >= COLS {
            return (false, symbol_short!("invalid"));
        }

        // Check if player is registered
        let is_player_one = *player == world.players.player_one;
        let is_player_two = *player == world.players.player_two;

        if !is_player_one && !is_player_two {
            return (false, symbol_short!("notplay"));
        }

        // Check turn order
        if world.game_state.is_player_one_turn && !is_player_one {
            return (false, symbol_short!("notturn"));
        }
        if !world.game_state.is_player_one_turn && !is_player_two {
            return (false, symbol_short!("notturn"));
        }

        // Check if column is full
        if world.board.is_column_full(&Env::default(), column) {
            return (false, symbol_short!("full"));
        }

        (true, symbol_short!("ok"))
    }

    /// Gravity System - finds lowest empty row in column
    fn gravity_system(world: &mut ECSWorldState, column: u32) -> u32 {
        world
            .board
            .get_lowest_empty_row(&Env::default(), column)
            .expect("Column should not be full - validated before calling")
    }

    /// Execution System - places piece on board
    fn execution_system(world: &mut ECSWorldState, row: u32, column: u32) {
        let cell_value = if world.game_state.is_player_one_turn {
            1u32
        } else {
            2u32
        };
        world
            .board
            .set_cell(&Env::default(), row, column, cell_value);
        world.game_state.move_count += 1;
        world.game_state.last_move_col = Some(column);
    }

    /// Win Detection System - checks for 4-in-a-row patterns
    fn win_detection_system(world: &mut ECSWorldState) {
        let board = &world.board;
        let env = Env::default();

        // Check all positions for a winning piece
        for row in 0..ROWS {
            for col in 0..COLS {
                let cell = board.get_cell(&env, row, col);
                if cell == 0 {
                    continue;
                }

                // Check horizontal
                if Self::check_horizontal(board, &env, row, col, cell) {
                    world.game_state.status = cell;
                    return;
                }

                // Check vertical
                if Self::check_vertical(board, &env, row, col, cell) {
                    world.game_state.status = cell;
                    return;
                }

                // Check diagonal (bottom-left to top-right)
                if Self::check_diagonal_positive(board, &env, row, col, cell) {
                    world.game_state.status = cell;
                    return;
                }

                // Check diagonal (top-left to bottom-right)
                if Self::check_diagonal_negative(board, &env, row, col, cell) {
                    world.game_state.status = cell;
                    return;
                }
            }
        }
    }

    /// Check horizontal connection (4 in a row)
    fn check_horizontal(board: &BoardComponent, env: &Env, row: u32, col: u32, cell: u32) -> bool {
        if col + 3 >= COLS {
            return false;
        }

        for i in 0..4 {
            if board.get_cell(env, row, col + i) != cell {
                return false;
            }
        }
        true
    }

    /// Check vertical connection (4 in a column)
    fn check_vertical(board: &BoardComponent, env: &Env, row: u32, col: u32, cell: u32) -> bool {
        if row + 3 >= ROWS {
            return false;
        }

        for i in 0..4 {
            if board.get_cell(env, row + i, col) != cell {
                return false;
            }
        }
        true
    }

    /// Check diagonal with positive slope (bottom-left to top-right)
    fn check_diagonal_positive(
        board: &BoardComponent,
        env: &Env,
        row: u32,
        col: u32,
        cell: u32,
    ) -> bool {
        if row + 3 >= ROWS || col + 3 >= COLS {
            return false;
        }

        for i in 0..4 {
            if board.get_cell(env, row + i, col + i) != cell {
                return false;
            }
        }
        true
    }

    /// Check diagonal with negative slope (top-left to bottom-right)
    fn check_diagonal_negative(
        board: &BoardComponent,
        env: &Env,
        row: u32,
        col: u32,
        cell: u32,
    ) -> bool {
        if row < 3 || col + 3 >= COLS {
            return false;
        }

        for i in 0..4 {
            if board.get_cell(env, row - i, col + i) != cell {
                return false;
            }
        }
        true
    }

    /// Draw System - detects full board with no winner
    fn draw_system(world: &mut ECSWorldState) {
        // Maximum moves = ROWS * COLS = 42
        if world.game_state.move_count >= ROWS * COLS && world.game_state.status == 0 {
            world.game_state.status = 3; // Draw
        }
    }

    /// Turn System - switches turns between players
    fn turn_system(world: &mut ECSWorldState) {
        if world.game_state.status == 0 {
            world.game_state.is_player_one_turn = !world.game_state.is_player_one_turn;
        }
    }

    fn to_game_state(env: &Env, world: &ECSWorldState) -> GameState {
        let mut board = Vec::new(env);
        for i in 0..(ROWS * COLS) {
            board.push_back(world.board.cells.get(i).unwrap_or(0));
        }

        GameState {
            board,
            rows: ROWS,
            cols: COLS,
            player_one: world.players.player_one.clone(),
            player_two: world.players.player_two.clone(),
            is_player_one_turn: world.game_state.is_player_one_turn,
            move_count: world.game_state.move_count,
            status: world.game_state.status,
            last_move_col: world.game_state.last_move_col,
        }
    }
}

#[cfg(test)]
mod tests;
