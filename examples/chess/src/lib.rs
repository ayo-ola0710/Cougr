#![no_std]

use cougr_core::component::ComponentTrait;
use cougr_core::privacy::experimental::CustomCircuit;
use cougr_core::privacy::{Groth16Proof, VerificationKey};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Map, Symbol,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardState {
    pub state_hash: BytesN<32>,
    pub pieces: Map<u32, Piece>,
}

impl ComponentTrait for BoardState {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for i in 0..32 {
            bytes.push_back(self.state_hash.get(i).unwrap());
        }
        bytes
    }

    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Checkmate,
    Draw,
    Resigned,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnState {
    pub current: Address,
    pub move_count: u32,
    pub status: GameStatus,
}

impl ComponentTrait for TurnState {
    fn component_type() -> Symbol {
        symbol_short!("turn")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.move_count.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProofRecord {
    pub last_proof: Bytes,
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub white: Address,
    pub black: Address,
    pub board: BoardState,
    pub turn: TurnState,
    pub proof_record: ProofRecord,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoveResult {
    Success,
    InvalidProof,
    WrongTurn,
    GameOver,
}

const GAME_KEY: Symbol = symbol_short!("GAME");
const VK_KEY: Symbol = symbol_short!("VK");

#[contract]
pub struct ChessContract;

#[contractimpl]
impl ChessContract {
    /// Initialize a new chess game
    pub fn new_game(env: Env, white: Address, black: Address) {
        let board = Self::init_board(&env);
        let state_hash = Self::compute_state_hash(&env, &board);

        let game_state = GameState {
            white: white.clone(),
            black: black.clone(),
            board: BoardState {
                state_hash,
                pieces: board,
            },
            turn: TurnState {
                current: white,
                move_count: 0,
                status: GameStatus::Playing,
            },
            proof_record: ProofRecord {
                last_proof: Bytes::new(&env),
                verified: false,
            },
        };

        env.storage().instance().set(&GAME_KEY, &game_state);
    }

    /// Submit a move with ZK proof
    pub fn submit_move(env: Env, player: Address, from: u32, to: u32, proof: Bytes) -> MoveResult {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        // TurnSystem: validate player
        if game.turn.current != player {
            return MoveResult::WrongTurn;
        }

        if game.turn.status != GameStatus::Playing {
            return MoveResult::GameOver;
        }

        // ProofVerificationSystem: verify the move proof
        let vk: VerificationKey = env
            .storage()
            .instance()
            .get(&VK_KEY)
            .unwrap_or_else(|| panic!("VK not set"));

        let groth16_proof = Self::decode_proof(&env, &proof);
        let circuit = Self::build_move_circuit(&env, &vk, &game.board.state_hash, from, to);

        let verified = circuit.verify(&env, &groth16_proof).unwrap_or(false);

        if !verified {
            return MoveResult::InvalidProof;
        }

        // BoardUpdateSystem: apply the move
        Self::apply_move(&mut game, from, to);

        // Update state hash
        game.board.state_hash = Self::compute_state_hash(&env, &game.board.pieces);

        // Update proof record
        game.proof_record.last_proof = proof;
        game.proof_record.verified = true;

        // TurnSystem: switch turn
        game.turn.move_count += 1;
        game.turn.current = if game.turn.current == game.white {
            game.black.clone()
        } else {
            game.white.clone()
        };

        // EndGameSystem: check for checkmate (simplified: king captured)
        Self::check_endgame(&mut game);

        env.storage().instance().set(&GAME_KEY, &game);
        MoveResult::Success
    }

    /// Resign the game
    pub fn resign(env: Env, player: Address) {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if player == game.white || player == game.black {
            game.turn.status = GameStatus::Resigned;
            env.storage().instance().set(&GAME_KEY, &game);
        }
    }

    /// Get the current board state
    pub fn get_board(env: Env) -> BoardState {
        let game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));
        game.board
    }

    /// Get the current game state
    pub fn get_state(env: Env) -> GameState {
        env.storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"))
    }

    /// Set the verification key (admin function)
    pub fn set_vk(env: Env, vk: VerificationKey) {
        env.storage().instance().set(&VK_KEY, &vk);
    }

    // Internal helper functions

    fn init_board(env: &Env) -> Map<u32, Piece> {
        let mut board = Map::new(env);

        // White pieces (bottom)
        board.set(
            0,
            Piece {
                kind: PieceKind::Rook,
                color: Color::White,
            },
        );
        board.set(
            1,
            Piece {
                kind: PieceKind::Knight,
                color: Color::White,
            },
        );
        board.set(
            2,
            Piece {
                kind: PieceKind::Bishop,
                color: Color::White,
            },
        );
        board.set(
            3,
            Piece {
                kind: PieceKind::Queen,
                color: Color::White,
            },
        );
        board.set(
            4,
            Piece {
                kind: PieceKind::King,
                color: Color::White,
            },
        );
        board.set(
            5,
            Piece {
                kind: PieceKind::Bishop,
                color: Color::White,
            },
        );
        board.set(
            6,
            Piece {
                kind: PieceKind::Knight,
                color: Color::White,
            },
        );
        board.set(
            7,
            Piece {
                kind: PieceKind::Rook,
                color: Color::White,
            },
        );
        for i in 8..16 {
            board.set(
                i,
                Piece {
                    kind: PieceKind::Pawn,
                    color: Color::White,
                },
            );
        }

        // Black pieces (top)
        board.set(
            56,
            Piece {
                kind: PieceKind::Rook,
                color: Color::Black,
            },
        );
        board.set(
            57,
            Piece {
                kind: PieceKind::Knight,
                color: Color::Black,
            },
        );
        board.set(
            58,
            Piece {
                kind: PieceKind::Bishop,
                color: Color::Black,
            },
        );
        board.set(
            59,
            Piece {
                kind: PieceKind::Queen,
                color: Color::Black,
            },
        );
        board.set(
            60,
            Piece {
                kind: PieceKind::King,
                color: Color::Black,
            },
        );
        board.set(
            61,
            Piece {
                kind: PieceKind::Bishop,
                color: Color::Black,
            },
        );
        board.set(
            62,
            Piece {
                kind: PieceKind::Knight,
                color: Color::Black,
            },
        );
        board.set(
            63,
            Piece {
                kind: PieceKind::Rook,
                color: Color::Black,
            },
        );
        for i in 48..56 {
            board.set(
                i,
                Piece {
                    kind: PieceKind::Pawn,
                    color: Color::Black,
                },
            );
        }

        board
    }

    fn compute_state_hash(env: &Env, board: &Map<u32, Piece>) -> BytesN<32> {
        let mut data = Bytes::new(env);
        for pos in 0..64u32 {
            if let Some(piece) = board.get(pos) {
                data.append(&Bytes::from_array(env, &pos.to_be_bytes()));
                let kind_byte = match piece.kind {
                    PieceKind::King => 1u8,
                    PieceKind::Queen => 2u8,
                    PieceKind::Rook => 3u8,
                    PieceKind::Bishop => 4u8,
                    PieceKind::Knight => 5u8,
                    PieceKind::Pawn => 6u8,
                };
                let color_byte = match piece.color {
                    Color::White => 0u8,
                    Color::Black => 1u8,
                };
                data.append(&Bytes::from_array(env, &[kind_byte, color_byte]));
            }
        }
        env.crypto().sha256(&data).into()
    }

    fn build_move_circuit(
        env: &Env,
        vk: &VerificationKey,
        state_hash: &BytesN<32>,
        from: u32,
        to: u32,
    ) -> CustomCircuit {
        CustomCircuit::builder(vk.clone())
            .add_bytes32(state_hash)
            .add_u32(env, from)
            .add_u32(env, to)
            .build()
    }

    fn decode_proof(_env: &Env, _proof_bytes: &Bytes) -> Groth16Proof {
        // In a real implementation, this would decode the proof from bytes
        // For now, we create a dummy proof structure
        // The actual proof would be serialized from the off-chain prover
        panic!("Proof decoding not implemented - use mock in tests")
    }

    fn apply_move(game: &mut GameState, from: u32, to: u32) {
        if let Some(piece) = game.board.pieces.get(from) {
            game.board.pieces.set(to, piece);
            game.board.pieces.remove(from);
        }
    }

    fn check_endgame(game: &mut GameState) {
        let mut white_king_exists = false;
        let mut black_king_exists = false;

        for pos in 0..64u32 {
            if let Some(piece) = game.board.pieces.get(pos) {
                if piece.kind == PieceKind::King {
                    match piece.color {
                        Color::White => white_king_exists = true,
                        Color::Black => black_king_exists = true,
                    }
                }
            }
        }

        if !white_king_exists || !black_king_exists {
            game.turn.status = GameStatus::Checkmate;
        }
    }
}

#[cfg(test)]
mod test;
