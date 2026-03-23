#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────

const BOARD_SIZE: u32 = 8;
const EMPTY: u32 = 0;
const BLACK: u32 = 1;
const WHITE: u32 = 2;
const WORLD_KEY: Symbol = symbol_short!("WORLD");
const DIRS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1), (0, 1),
    (1, -1), (1, 0), (1, 1),
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
    pub fn init_game(env: Env, player_one: Address, player_two: Address) {
        let _ = (env, player_one, player_two);
        panic!("not implemented")
    }

    pub fn submit_move(env: Env, player: Address, row: u32, col: u32) {
        let _ = (env, player, row, col);
        panic!("not implemented")
    }

    pub fn get_state(env: Env) -> GameState {
        let _ = env;
        panic!("not implemented")
    }

    pub fn get_board(env: Env) -> BoardState {
        let _ = env;
        panic!("not implemented")
    }

    pub fn get_score(env: Env) -> ScoreState {
        let _ = env;
        panic!("not implemented")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};
}
