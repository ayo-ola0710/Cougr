#![no_std]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────

const BOARD_SIZE: u32 = 8;
const EMPTY: u32 = 0;
const BLACK: u32 = 1;
const WHITE: u32 = 2;
const STATUS_ACTIVE: u32 = 0;
const STATUS_FINISHED: u32 = 1;
const WINNER_NONE: u32 = 0;
const WINNER_DRAW: u32 = 3;
const WORLD_KEY: Symbol = symbol_short!("WORLD");
const DIRS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

// ── Components ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardComponent {
    pub cells: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnComponent {
    pub current_player: u32,
    /// Recomputed each turn: 0=normal, 1=opponent currently has no legal move, 2=neither player has a legal move (triggers game end).
    pub pass_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStatusComponent {
    pub status: u32, // 0 = active, 1 = finished
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreComponent {
    pub black_count: u32,
    pub white_count: u32,
}

// ── ComponentTrait implementations ───────────────────────────────────────────

impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("boardcomp")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.width.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.height.to_be_bytes(),
        ));
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let cell = self.cells.get(i).unwrap_or(EMPTY);
            bytes.append(&soroban_sdk::Bytes::from_array(env, &cell.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        // 4 (width) + 4 (height) + 64 * 4 (cells) = 272
        if data.len() != 272 {
            return None;
        }
        let width = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let height = u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        let mut cells = Vec::new(env);
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let offset = 8 + i * 4;
            let cell = u32::from_be_bytes([
                data.get(offset)?,
                data.get(offset + 1)?,
                data.get(offset + 2)?,
                data.get(offset + 3)?,
            ]);
            cells.push_back(cell);
        }
        Some(Self {
            cells,
            width,
            height,
        })
    }
}

impl ComponentTrait for TurnComponent {
    fn component_type() -> Symbol {
        symbol_short!("turncomp")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.current_player.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.pass_count.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let current_player =
            u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let pass_count =
            u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self {
            current_player,
            pass_count,
        })
    }
}

impl ComponentTrait for GameStatusComponent {
    fn component_type() -> Symbol {
        symbol_short!("gamestatu")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.status.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 4 {
            return None;
        }
        let status = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        Some(Self { status })
    }
}

impl ComponentTrait for ScoreComponent {
    fn component_type() -> Symbol {
        symbol_short!("scorecomp")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.black_count.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.white_count.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 8 {
            return None;
        }
        let black_count =
            u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let white_count =
            u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        Some(Self {
            black_count,
            white_count,
        })
    }
}

// ── ECS World ────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub board: BoardComponent,
    pub turn: TurnComponent,
    pub status: GameStatusComponent,
    pub score: ScoreComponent,
    pub player_one: Address, // plays BLACK
    pub player_two: Address, // plays WHITE
}

// ── Public API types ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub current_player: u32,
    pub pass_count: u32,
    pub status: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoardState {
    pub cells: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreState {
    pub black_count: u32,
    pub white_count: u32,
    pub winner: u32, // 0 = ongoing, 1 = black wins, 2 = white wins, 3 = draw
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct ReversiContract;

#[contractimpl]
impl ReversiContract {
    // ── Public API ────────────────────────────────────────────────────────────────

    pub fn init_game(env: Env, player_one: Address, player_two: Address) {
        if env
            .storage()
            .instance()
            .get::<Symbol, ECSWorldState>(&WORLD_KEY)
            .is_some()
        {
            panic!("Game already initialized");
        }
        let board = Self::init_board(&env);
        let score = Self::scoring_system(&board);
        let world = ECSWorldState {
            board,
            turn: TurnComponent {
                current_player: BLACK,
                pass_count: 0,
            },
            status: GameStatusComponent {
                status: STATUS_ACTIVE,
            },
            score,
            player_one,
            player_two,
        };
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn submit_move(env: Env, player: Address, row: u32, col: u32) {
        player.require_auth();
        let mut world = Self::load_world(&env);
        if world.status.status != STATUS_ACTIVE {
            panic!("Game is finished");
        }
        let player_color = Self::player_color(&world, &player);
        if player_color != world.turn.current_player {
            panic!("Not your turn");
        }
        if !Self::move_validation_system(&world.board, row, col, player_color) {
            panic!("Illegal move");
        }
        let opponent = Self::opponent_of(player_color);
        world.board = Self::flip_resolution_system(world.board, row, col, player_color);
        world.score = Self::scoring_system(&world.board);
        world.turn = Self::turn_system(&world.board, player_color, opponent);
        world.status = Self::end_condition_system(&world.board, &world.turn);
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn get_state(env: Env) -> GameState {
        let world = Self::load_world(&env);
        GameState {
            current_player: world.turn.current_player,
            pass_count: world.turn.pass_count,
            status: world.status.status,
        }
    }

    pub fn get_board(env: Env) -> BoardState {
        let world = Self::load_world(&env);
        BoardState {
            cells: world.board.cells,
            width: world.board.width,
            height: world.board.height,
        }
    }

    pub fn get_score(env: Env) -> ScoreState {
        let world = Self::load_world(&env);
        let winner = if world.status.status == STATUS_FINISHED {
            if world.score.black_count > world.score.white_count {
                BLACK
            } else if world.score.white_count > world.score.black_count {
                WHITE
            } else {
                WINNER_DRAW
            }
        } else {
            WINNER_NONE
        };
        ScoreState {
            black_count: world.score.black_count,
            white_count: world.score.white_count,
            winner,
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────────

    fn load_world(env: &Env) -> ECSWorldState {
        env.storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"))
    }

    fn idx(row: u32, col: u32) -> u32 {
        row * BOARD_SIZE + col
    }

    fn get_cell(cells: &Vec<u32>, row: u32, col: u32) -> u32 {
        cells.get(row * BOARD_SIZE + col).unwrap_or(EMPTY)
    }

    fn opponent_of(player: u32) -> u32 {
        if player == BLACK {
            WHITE
        } else {
            BLACK
        }
    }

    fn player_color(world: &ECSWorldState, player: &Address) -> u32 {
        if player == &world.player_one {
            BLACK
        } else if player == &world.player_two {
            WHITE
        } else {
            panic!("Unknown player")
        }
    }

    fn is_board_full(board: &BoardComponent) -> bool {
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            if board.cells.get(i).unwrap_or(EMPTY) == EMPTY {
                return false;
            }
        }
        true
    }

    fn init_board(env: &Env) -> BoardComponent {
        let mut cells = Vec::new(env);
        for _ in 0..(BOARD_SIZE * BOARD_SIZE) {
            cells.push_back(EMPTY);
        }
        cells.set(Self::idx(3, 3), WHITE);
        cells.set(Self::idx(3, 4), BLACK);
        cells.set(Self::idx(4, 3), BLACK);
        cells.set(Self::idx(4, 4), WHITE);
        BoardComponent {
            cells,
            width: BOARD_SIZE,
            height: BOARD_SIZE,
        }
    }

    // ── ScoringSystem ─────────────────────────────────────────────────────────────

    fn scoring_system(board: &BoardComponent) -> ScoreComponent {
        let mut black_count = 0u32;
        let mut white_count = 0u32;
        for i in 0..(BOARD_SIZE * BOARD_SIZE) {
            let cell = board.cells.get(i).unwrap_or(EMPTY);
            if cell == BLACK {
                black_count += 1;
            } else if cell == WHITE {
                white_count += 1;
            }
        }
        ScoreComponent {
            black_count,
            white_count,
        }
    }

    // ── MoveValidationSystem ──────────────────────────────────────────────────────

    fn move_validation_system(board: &BoardComponent, row: u32, col: u32, player: u32) -> bool {
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return false;
        }
        if Self::get_cell(&board.cells, row, col) != EMPTY {
            return false;
        }
        let opp = Self::opponent_of(player);
        for (dr, dc) in DIRS {
            if Self::flips_in_dir(&board.cells, row, col, player, opp, dr, dc) > 0 {
                return true;
            }
        }
        false
    }

    fn flips_in_dir(
        cells: &Vec<u32>,
        row: u32,
        col: u32,
        player: u32,
        opp: u32,
        dr: i32,
        dc: i32,
    ) -> u32 {
        let mut r = row as i32 + dr;
        let mut c = col as i32 + dc;
        let mut count = 0u32;
        while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            let cell = Self::get_cell(cells, r as u32, c as u32);
            if cell == opp {
                count += 1;
                r += dr;
                c += dc;
            } else if cell == player {
                return count;
            } else {
                return 0;
            }
        }
        0
    }

    fn has_legal_moves(board: &BoardComponent, player: u32) -> bool {
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if Self::move_validation_system(board, row, col, player) {
                    return true;
                }
            }
        }
        false
    }

    // ── FlipResolutionSystem ──────────────────────────────────────────────────────

    fn flip_resolution_system(
        mut board: BoardComponent,
        row: u32,
        col: u32,
        player: u32,
    ) -> BoardComponent {
        let opp = Self::opponent_of(player);
        board.cells.set(Self::idx(row, col), player);
        for (dr, dc) in DIRS {
            let n = Self::flips_in_dir(&board.cells, row, col, player, opp, dr, dc);
            if n > 0 {
                let mut r = row as i32 + dr;
                let mut c = col as i32 + dc;
                for _ in 0..n {
                    board.cells.set(Self::idx(r as u32, c as u32), player);
                    r += dr;
                    c += dc;
                }
            }
        }
        board
    }

    // ── TurnSystem ────────────────────────────────────────────────────────────────

    fn turn_system(board: &BoardComponent, current: u32, opponent: u32) -> TurnComponent {
        if Self::has_legal_moves(board, opponent) {
            TurnComponent {
                current_player: opponent,
                pass_count: 0,
            }
        } else {
            Self::pass_system(board, current, opponent)
        }
    }

    // ── PassSystem ────────────────────────────────────────────────────────────────

    /// PassSystem: Handles automatic pass when the next player has no legal move.
    fn pass_system(board: &BoardComponent, current: u32, _opponent: u32) -> TurnComponent {
        if Self::has_legal_moves(board, current) {
            // Current player continues; opponent is auto-passed
            TurnComponent {
                current_player: current,
                pass_count: 1,
            }
        } else {
            // Both players locked; game will end
            TurnComponent {
                current_player: current,
                pass_count: 2,
            }
        }
    }

    // ── EndConditionSystem ────────────────────────────────────────────────────────

    fn end_condition_system(board: &BoardComponent, turn: &TurnComponent) -> GameStatusComponent {
        if turn.pass_count >= 2 || Self::is_board_full(board) {
            GameStatusComponent {
                status: STATUS_FINISHED,
            }
        } else {
            GameStatusComponent {
                status: STATUS_ACTIVE,
            }
        }
    }
}

#[cfg(test)]
mod test;
