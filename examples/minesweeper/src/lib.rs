#![no_std]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Bytes, Env, Symbol, Vec};

/// Board dimensions: 9×9 (compact for on-chain)
const ROWS: u32 = 9;
const COLS: u32 = 9;

/// Number of mines (10 mines for ~12% density - beginner friendly)
const MINES: u32 = 10;

/// Cell state: 0-8 = revealed with count, 9 = hidden, 10 = mine, 11 = flag (optional)
const CELL_HIDDEN: u32 = 9;
const CELL_MINE: u32 = 10;

/// Game status: 0=Playing, 1=Won, 2=Lost
const STATUS_PLAYING: u32 = 0;
const STATUS_WON: u32 = 1;
const STATUS_LOST: u32 = 2;

/// Board component - stores the mine layout and revealed state
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
            cells.push_back(CELL_HIDDEN);
        }
        Self { cells, entity_id }
    }

    /// Get cell value at (row, col)
    pub fn get_cell(&self, _env: &Env, row: u32, col: u32) -> u32 {
        if row >= ROWS || col >= COLS {
            return CELL_HIDDEN;
        }
        let index = row * COLS + col;
        self.cells.get(index).unwrap_or(CELL_HIDDEN)
    }

    /// Set cell value at (row, col)
    pub fn set_cell(&mut self, _env: &Env, row: u32, col: u32, value: u32) {
        if row >= ROWS || col >= COLS {
            return;
        }
        let index = row * COLS + col;
        self.cells.set(index, value);
    }

    /// Check if cell is hidden
    pub fn is_hidden(&self, env: &Env, row: u32, col: u32) -> bool {
        self.get_cell(env, row, col) == CELL_HIDDEN
    }

    /// Check if cell is a mine
    pub fn is_mine(&self, env: &Env, row: u32, col: u32) -> bool {
        self.get_cell(env, row, col) == CELL_MINE
    }

    /// Check if cell is revealed (0-8 or mine)
    pub fn is_revealed(&self, env: &Env, row: u32, col: u32) -> bool {
        let cell = self.get_cell(env, row, col);
        cell < CELL_HIDDEN || cell == CELL_MINE
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
            let cell = self.cells.get(i).unwrap_or(CELL_HIDDEN);
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
            let offset = 8 + (i * 4);
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

/// Mine layout component - stores where mines are placed (hidden from players)
#[contracttype]
#[derive(Clone, Debug)]
pub struct MineLayoutComponent {
    pub mines: Vec<u32>, // Bit-packed or simple array of mine positions
    pub entity_id: u32,
}

impl MineLayoutComponent {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        let mut mines = Vec::new(env);
        for _ in 0..(ROWS * COLS) {
            mines.push_back(0u32); // 0 = no mine, 1 = mine
        }
        Self { mines, entity_id }
    }

    /// Check if position has a mine
    pub fn has_mine(&self, _env: &Env, row: u32, col: u32) -> bool {
        if row >= ROWS || col >= COLS {
            return false;
        }
        let index = row * COLS + col;
        self.mines.get(index).unwrap_or(0) == 1
    }

    /// Set mine at position
    pub fn set_mine(&mut self, _env: &Env, row: u32, col: u32) {
        if row >= ROWS || col >= COLS {
            return;
        }
        let index = row * COLS + col;
        self.mines.set(index, 1);
    }

    /// Count adjacent mines
    pub fn count_adjacent_mines(&self, env: &Env, row: u32, col: u32) -> u32 {
        let mut count = 0;

        // Check all 8 neighbors
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let nr = row as i32 + dr;
                let nc = col as i32 + dc;

                if nr >= 0
                    && nr < ROWS as i32
                    && nc >= 0
                    && nc < COLS as i32
                    && self.has_mine(env, nr as u32, nc as u32)
                {
                    count += 1;
                }
            }
        }

        count
    }
}

impl ComponentTrait for MineLayoutComponent {
    fn component_type() -> Symbol {
        symbol_short!("minelyt")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.mines.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let mine = self.mines.get(i).unwrap_or(0);
            bytes.append(&Bytes::from_array(env, &mine.to_be_bytes()));
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

        let mut mines = Vec::new(env);
        for i in 0..len {
            let offset = 8 + (i * 4);
            if offset + 4 > data.len() {
                break;
            }
            let mine = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            mines.push_back(mine);
        }
        Some(Self { mines, entity_id })
    }
}

/// Game state component
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStateComponent {
    pub status: u32, // 0=Playing, 1=Won, 2=Lost
    pub revealed_count: u32,
    pub entity_id: u32,
}

impl GameStateComponent {
    pub fn new(entity_id: u32) -> Self {
        Self {
            status: STATUS_PLAYING,
            revealed_count: 0,
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
        bytes.append(&Bytes::from_array(env, &self.status.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.revealed_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let status = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let revealed_count = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        Some(Self {
            status,
            revealed_count,
            entity_id,
        })
    }
}

/// ECS World State
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub board: BoardComponent,
    pub mine_layout: MineLayoutComponent,
    pub game_state: GameStateComponent,
    pub next_entity_id: u32,
}

/// External game state for API consumers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub rows: u32,
    pub cols: u32,
    pub total_mines: u32,
    pub status: u32,
    pub revealed_count: u32,
    pub safe_cells_remaining: u32,
}

/// Reveal result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevealResult {
    pub success: bool,
    pub is_mine: bool,
    pub adjacent_mines: u32,
    pub message: Symbol,
}

/// Visible cell state for querying
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibleCellState {
    pub is_revealed: bool,
    pub is_mine: bool,
    pub adjacent_mines: u32,
}

const WORLD_KEY: Symbol = symbol_short!("WORLD");

#[contract]
pub struct MinesweeperContract;

#[contractimpl]
impl MinesweeperContract {
    /// Initialize a new game with deterministic mine layout
    pub fn init_game(env: Env) -> GameState {
        let mut next_entity_id = 0u32;

        // Create empty board
        let board = BoardComponent::new(&env, next_entity_id);
        next_entity_id += 1;

        // Create mine layout and place mines deterministically
        let mut mine_layout = MineLayoutComponent::new(&env, next_entity_id);
        Self::place_mines_deterministic(&mut mine_layout, &env);
        next_entity_id += 1;

        // Create game state
        let game_state = GameStateComponent::new(next_entity_id);
        next_entity_id += 1;

        let world_state = ECSWorldState {
            board,
            mine_layout,
            game_state,
            next_entity_id,
        };

        env.storage().instance().set(&WORLD_KEY, &world_state);
        Self::to_game_state(&env, &world_state)
    }

    /// Reveal a cell at (row, col)
    pub fn reveal_cell(env: Env, row: u32, col: u32) -> RevealResult {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // Validation
        if world_state.game_state.status != STATUS_PLAYING {
            return RevealResult {
                success: false,
                is_mine: false,
                adjacent_mines: 0,
                message: symbol_short!("over"),
            };
        }

        if row >= ROWS || col >= COLS {
            return RevealResult {
                success: false,
                is_mine: false,
                adjacent_mines: 0,
                message: symbol_short!("invalid"),
            };
        }

        // Check if already revealed
        if !world_state.board.is_hidden(&env, row, col) {
            return RevealResult {
                success: false,
                is_mine: false,
                adjacent_mines: 0,
                message: symbol_short!("revealed"),
            };
        }

        // Check if mine
        if world_state.mine_layout.has_mine(&env, row, col) {
            // Game over - loss
            world_state.board.set_cell(&env, row, col, CELL_MINE);
            world_state.game_state.status = STATUS_LOST;

            env.storage().instance().set(&WORLD_KEY, &world_state);

            return RevealResult {
                success: true,
                is_mine: true,
                adjacent_mines: 0,
                message: symbol_short!("boom"),
            };
        }

        // Safe cell - reveal it
        let adjacent_count = world_state.mine_layout.count_adjacent_mines(&env, row, col);
        world_state.board.set_cell(&env, row, col, adjacent_count);
        world_state.game_state.revealed_count += 1;

        // Check for win condition
        Self::completion_system(&mut world_state);

        env.storage().instance().set(&WORLD_KEY, &world_state);

        RevealResult {
            success: true,
            is_mine: false,
            adjacent_mines: adjacent_count,
            message: symbol_short!("ok"),
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

    /// Get visible state of a specific cell
    pub fn get_visible_cell(env: Env, row: u32, col: u32) -> VisibleCellState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if row >= ROWS || col >= COLS {
            return VisibleCellState {
                is_revealed: false,
                is_mine: false,
                adjacent_mines: 0,
            };
        }

        let is_revealed = world_state.board.is_revealed(&env, row, col);
        let is_mine = world_state.mine_layout.has_mine(&env, row, col);
        let adjacent_mines = if is_revealed {
            world_state.mine_layout.count_adjacent_mines(&env, row, col)
        } else {
            0
        };

        VisibleCellState {
            is_revealed,
            is_mine: is_revealed && is_mine, // Only show mine if revealed
            adjacent_mines,
        }
    }

    /// Check if the game is finished
    pub fn is_finished(env: Env) -> bool {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.game_state.status != STATUS_PLAYING
    }

    /// Get the board state (for debugging/viewing)
    pub fn get_board(env: Env) -> Vec<u32> {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        world_state.board.cells
    }

    /// Reset the game
    pub fn reset_game(env: Env) -> GameState {
        Self::init_game(env)
    }

    /// Place mines in a deterministic pattern (for proof-friendly implementation)
    /// Uses a fixed pattern that can be verified
    fn place_mines_deterministic(mine_layout: &mut MineLayoutComponent, env: &Env) {
        // Deterministic mine placement - fixed positions for verifiability
        // Using a scattered pattern across the 9x9 grid
        let mine_positions = [
            (1, 1),
            (1, 5),
            (2, 7),
            (3, 3),
            (3, 8),
            (4, 6),
            (5, 2),
            (5, 4),
            (5, 7),
            (7, 0),
            (7, 5),
        ];

        for (row, col) in mine_positions.iter() {
            if *row < ROWS && *col < COLS {
                mine_layout.set_mine(env, *row, *col);
            }
        }
    }

    /// Completion System - checks if all safe cells are revealed
    fn completion_system(world: &mut ECSWorldState) {
        // Total safe cells = total cells - mines
        let total_safe = (ROWS * COLS) - MINES;

        if world.game_state.revealed_count >= total_safe {
            world.game_state.status = STATUS_WON;
        }
    }

    fn to_game_state(_env: &Env, world: &ECSWorldState) -> GameState {
        let total_safe = (ROWS * COLS) - MINES;
        let safe_cells_remaining = total_safe - world.game_state.revealed_count;

        GameState {
            rows: ROWS,
            cols: COLS,
            total_mines: MINES,
            status: world.game_state.status,
            revealed_count: world.game_state.revealed_count,
            safe_cells_remaining,
        }
    }
}

#[cfg(test)]
mod tests;
