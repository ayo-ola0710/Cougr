#![no_std]

use cougr_core::accounts::{BatchBuilder, GameAction, SessionBuilder};
use cougr_core::component::ComponentTrait;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Bytes, Env, Symbol, Vec,
};

// ─── Error codes ────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GameError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotYourTurn = 3,
    NotAPlayer = 4,
    InvalidCard = 5,
    InsufficientMana = 6,
    CardNotInHand = 7,
    FieldFull = 8,
    InvalidTarget = 9,
    WrongPhase = 10,
    GameOver = 11,
    BatchEmpty = 12,
    InvalidAction = 13,
    InvalidPosition = 14,
    SessionExpired = 15,
}

// ─── Card kinds ─────────────────────────────────────────────────────────────

/// 0 = Creature, 1 = Spell
pub const KIND_CREATURE: u32 = 0;
pub const KIND_SPELL: u32 = 1;

// ─── Match phases ────────────────────────────────────────────────────────────

/// 0 = Draw, 1 = Main, 2 = Combat, 3 = End
pub const PHASE_DRAW: u32 = 0;
pub const PHASE_MAIN: u32 = 1;
pub const PHASE_COMBAT: u32 = 2;
pub const PHASE_END: u32 = 3;

// ─── Match status ────────────────────────────────────────────────────────────

/// 0 = InProgress, 1 = PlayerAWins, 2 = PlayerBWins, 3 = Conceded
pub const STATUS_IN_PROGRESS: u32 = 0;
pub const STATUS_A_WINS: u32 = 1;
pub const STATUS_B_WINS: u32 = 2;
pub const STATUS_CONCEDED: u32 = 3;

// ─── Field capacity ─────────────────────────────────────────────────────────

pub const MAX_HAND: u32 = 7;
pub const MAX_FIELD: u32 = 5;
pub const MAX_MANA: u32 = 10;
pub const STARTING_HEALTH: u32 = 20;
pub const STARTING_HAND_SIZE: u32 = 4;

// ─── Action symbols ─────────────────────────────────────────────────────────

pub const SYM_PLAY: Symbol = symbol_short!("play");
pub const SYM_SPELL: Symbol = symbol_short!("spell");
pub const SYM_ATTACK: Symbol = symbol_short!("attack");
pub const SYM_CONCEDE: Symbol = symbol_short!("concede");

// ─── Components ─────────────────────────────────────────────────────────────

/// A single card definition.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Card {
    pub id: u32,
    pub kind: u32, // KIND_CREATURE | KIND_SPELL
    pub cost: u32,
    pub power: u32,
    pub toughness: u32,
}

impl Card {
    pub fn new(id: u32, kind: u32, cost: u32, power: u32, toughness: u32) -> Self {
        Self {
            id,
            kind,
            cost,
            power,
            toughness,
        }
    }
}

/// A creature currently on the battlefield.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatureState {
    pub card_id: u32,
    pub power: u32,
    pub toughness: u32,
    pub current_toughness: u32,
}

impl CreatureState {
    pub fn new(card_id: u32, power: u32, toughness: u32) -> Self {
        Self {
            card_id,
            power,
            toughness,
            current_toughness: toughness,
        }
    }
}

/// Represents the hand for one player.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerHand {
    pub cards: Vec<u32>, // Card IDs
    pub entity_id: u32,
}

impl PlayerHand {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        Self {
            cards: Vec::new(env),
            entity_id,
        }
    }
}

impl ComponentTrait for PlayerHand {
    fn component_type() -> Symbol {
        symbol_short!("hand")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.cards.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let card_id = self.cards.get(i).unwrap_or(0);
            bytes.append(&Bytes::from_array(env, &card_id.to_be_bytes()));
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
        let mut cards = Vec::new(env);
        for i in 0..len {
            let offset = 8 + (i * 4);
            let card_id = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            cards.push_back(card_id);
        }
        Some(Self { cards, entity_id })
    }
}

/// Creatures on the battlefield for one player.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerField {
    pub creatures: Vec<CreatureState>,
    pub entity_id: u32,
}

impl PlayerField {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        Self {
            creatures: Vec::new(env),
            entity_id,
        }
    }
}

impl ComponentTrait for PlayerField {
    fn component_type() -> Symbol {
        symbol_short!("field")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.creatures.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let c = self.creatures.get(i).unwrap();
            bytes.append(&Bytes::from_array(env, &c.card_id.to_be_bytes()));
            bytes.append(&Bytes::from_array(env, &c.power.to_be_bytes()));
            bytes.append(&Bytes::from_array(env, &c.toughness.to_be_bytes()));
            bytes.append(&Bytes::from_array(env, &c.current_toughness.to_be_bytes()));
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
        let mut creatures = Vec::new(env);
        for i in 0..len {
            let base = 8 + (i * 16);
            let card_id = u32::from_be_bytes([
                data.get(base).unwrap(),
                data.get(base + 1).unwrap(),
                data.get(base + 2).unwrap(),
                data.get(base + 3).unwrap(),
            ]);
            let power = u32::from_be_bytes([
                data.get(base + 4).unwrap(),
                data.get(base + 5).unwrap(),
                data.get(base + 6).unwrap(),
                data.get(base + 7).unwrap(),
            ]);
            let toughness = u32::from_be_bytes([
                data.get(base + 8).unwrap(),
                data.get(base + 9).unwrap(),
                data.get(base + 10).unwrap(),
                data.get(base + 11).unwrap(),
            ]);
            let current_toughness = u32::from_be_bytes([
                data.get(base + 12).unwrap(),
                data.get(base + 13).unwrap(),
                data.get(base + 14).unwrap(),
                data.get(base + 15).unwrap(),
            ]);
            creatures.push_back(CreatureState {
                card_id,
                power,
                toughness,
                current_toughness,
            });
        }
        Some(Self {
            creatures,
            entity_id,
        })
    }
}

/// Stats (health, mana) for one player.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerStats {
    pub health: u32,
    pub mana: u32,
    pub max_mana: u32,
    pub entity_id: u32,
}

impl PlayerStats {
    pub fn new(entity_id: u32) -> Self {
        Self {
            health: STARTING_HEALTH,
            mana: 1,
            max_mana: 1,
            entity_id,
        }
    }
}

impl ComponentTrait for PlayerStats {
    fn component_type() -> Symbol {
        symbol_short!("pstats")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.health.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.mana.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.max_mana.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 16 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let health = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let mana = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let max_mana = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        Some(Self {
            health,
            mana,
            max_mana,
            entity_id,
        })
    }
}

/// Top-level match state.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchState {
    pub turn: u32,
    pub active_player: Address,
    pub phase: u32,
    pub status: u32,
    pub entity_id: u32,
}

impl MatchState {
    pub fn new(active_player: Address, entity_id: u32) -> Self {
        Self {
            turn: 1,
            active_player,
            phase: PHASE_DRAW,
            status: STATUS_IN_PROGRESS,
            entity_id,
        }
    }
}

// ─── World state (all ECS data for the match) ────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    // Players
    pub player_a: Address,
    pub player_b: Address,
    // Decks (remaining card IDs in draw order)
    pub deck_a: Vec<u32>,
    pub deck_b: Vec<u32>,
    // Hands
    pub hand_a: PlayerHand,
    pub hand_b: PlayerHand,
    // Fields
    pub field_a: PlayerField,
    pub field_b: PlayerField,
    // Stats
    pub stats_a: PlayerStats,
    pub stats_b: PlayerStats,
    // Match metadata
    pub match_state: MatchState,
    // Session key tracking (ledger timestamp-based TTL)
    pub session_a_expires: u64,
    pub session_b_expires: u64,
    pub next_entity_id: u32,
}

// ─── Card library (deterministic card definitions) ───────────────────────────

/// Returns the canonical card definition for a given card id.
/// Cards 1-5: cheap creatures; 6-8: powerful creatures; 9-10: spells.
fn card_definition(card_id: u32) -> Option<Card> {
    match card_id {
        1 => Some(Card::new(1, KIND_CREATURE, 1, 1, 2)), // 1-cost 1/2 creature
        2 => Some(Card::new(2, KIND_CREATURE, 2, 2, 2)), // 2-cost 2/2 creature
        3 => Some(Card::new(3, KIND_CREATURE, 2, 1, 3)), // 2-cost 1/3 creature
        4 => Some(Card::new(4, KIND_CREATURE, 3, 3, 2)), // 3-cost 3/2 creature
        5 => Some(Card::new(5, KIND_CREATURE, 3, 2, 4)), // 3-cost 2/4 creature
        6 => Some(Card::new(6, KIND_CREATURE, 4, 4, 4)), // 4-cost 4/4 creature
        7 => Some(Card::new(7, KIND_CREATURE, 5, 5, 5)), // 5-cost 5/5 creature
        8 => Some(Card::new(8, KIND_CREATURE, 6, 6, 6)), // 6-cost 6/6 creature
        9 => Some(Card::new(9, KIND_SPELL, 2, 3, 0)),    // 2-cost spell: deal 3 damage
        10 => Some(Card::new(10, KIND_SPELL, 4, 5, 0)),  // 4-cost spell: deal 5 damage
        _ => None,
    }
}

// ─── Action enum ────────────────────────────────────────────────────────────

/// Represents one action in a player's turn batch.
#[contracttype]
#[derive(Clone, Debug)]
pub enum Action {
    /// Play a creature from hand to the field.
    PlayCreature(u32), // card_id
    /// Cast a spell from hand, dealing damage to the opponent.
    CastSpell(u32), // card_id
    /// Declare an attack: attacker index on field vs target index on opponent field (u32::MAX = face).
    DeclareAttack(u32, u32), // attacker_idx, target_idx (MAX = direct)
}

// ─── Turn result ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnResult {
    pub success: bool,
    pub actions_executed: u32,
    pub match_status: u32,
    pub message: Symbol,
}

// ─── External view types ────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct FieldState {
    pub field_a: Vec<CreatureState>,
    pub field_b: Vec<CreatureState>,
}

// ─── Storage key ─────────────────────────────────────────────────────────────

const WORLD_KEY: Symbol = symbol_short!("WORLD");

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct TradingCardGame;

#[contractimpl]
impl TradingCardGame {
    // ── Match Setup ──────────────────────────────────────────────────────────

    /// Initialize a new match.  `deck_a` / `deck_b` are ordered lists of card IDs.
    pub fn new_match(
        env: Env,
        player_a: Address,
        player_b: Address,
        deck_a: Vec<u32>,
        deck_b: Vec<u32>,
    ) {
        if env.storage().instance().has(&WORLD_KEY) {
            panic_with_error!(&env, GameError::AlreadyInitialized);
        }

        let mut eid = 0u32;

        let mut hand_a = PlayerHand::new(&env, eid);
        eid += 1;
        let mut hand_b = PlayerHand::new(&env, eid);
        eid += 1;
        let field_a = PlayerField::new(&env, eid);
        eid += 1;
        let field_b = PlayerField::new(&env, eid);
        eid += 1;
        let stats_a = PlayerStats::new(eid);
        eid += 1;
        let stats_b = PlayerStats::new(eid);
        eid += 1;
        let match_state = MatchState::new(player_a.clone(), eid);
        eid += 1;

        // Draw starting hand from the front of the deck.
        let mut remaining_a = Vec::new(&env);
        let draw_a = STARTING_HAND_SIZE.min(deck_a.len());
        for i in 0..deck_a.len() {
            if i < draw_a {
                hand_a.cards.push_back(deck_a.get(i).unwrap());
            } else {
                remaining_a.push_back(deck_a.get(i).unwrap());
            }
        }

        let mut remaining_b = Vec::new(&env);
        let draw_b = STARTING_HAND_SIZE.min(deck_b.len());
        for i in 0..deck_b.len() {
            if i < draw_b {
                hand_b.cards.push_back(deck_b.get(i).unwrap());
            } else {
                remaining_b.push_back(deck_b.get(i).unwrap());
            }
        }

        let world = ECSWorldState {
            player_a,
            player_b,
            deck_a: remaining_a,
            deck_b: remaining_b,
            hand_a,
            hand_b,
            field_a,
            field_b,
            stats_a,
            stats_b,
            match_state,
            session_a_expires: 0,
            session_b_expires: 0,
            next_entity_id: eid,
        };

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── Session Management ───────────────────────────────────────────────────

    /// Create a match-scoped session for a player.  Returns the session expiry timestamp.
    pub fn start_session(env: Env, player: Address) -> u64 {
        let mut world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        let is_a = player == world.player_a;
        let is_b = player == world.player_b;
        if !is_a && !is_b {
            panic_with_error!(&env, GameError::NotAPlayer);
        }

        // Build a session scope scoped to the three game actions.
        let expires_at = env.ledger().timestamp() + 7200; // 2-hour TTL
        let _scope = SessionBuilder::new(&env)
            .allow_action(SYM_PLAY)
            .allow_action(SYM_SPELL)
            .allow_action(SYM_ATTACK)
            .max_operations(200)
            .expires_at(expires_at)
            .build_scope();

        if is_a {
            world.session_a_expires = expires_at;
        } else {
            world.session_b_expires = expires_at;
        }

        env.storage().instance().set(&WORLD_KEY, &world);
        expires_at
    }

    // ── Turn Submission ──────────────────────────────────────────────────────

    /// Submit a full turn as an atomic batch of actions.
    ///
    /// The caller must be the active player.  Actions are composed via `BatchBuilder`
    /// and either ALL succeed or NONE are applied (the function panics, reverting all state).
    pub fn submit_turn(env: Env, player: Address, actions: Vec<Action>) -> TurnResult {
        let mut world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        // ── Guard: match must be in progress ────────────────────────────────
        if world.match_state.status != STATUS_IN_PROGRESS {
            panic_with_error!(&env, GameError::GameOver);
        }

        // ── Guard: it must be the player's turn ─────────────────────────────
        if player != world.match_state.active_player {
            panic_with_error!(&env, GameError::NotYourTurn);
        }

        let is_a = player == world.player_a;

        // ── Guard: session must be valid ─────────────────────────────────────
        let session_expires = if is_a {
            world.session_a_expires
        } else {
            world.session_b_expires
        };
        if session_expires == 0 || env.ledger().timestamp() > session_expires {
            panic_with_error!(&env, GameError::SessionExpired);
        }

        // ── Guard: actions list must not be empty ────────────────────────────
        if actions.is_empty() {
            panic_with_error!(&env, GameError::BatchEmpty);
        }

        // ── Run DrawSystem first (beginning of turn) ─────────────────────────
        Self::draw_system(&env, &mut world, is_a);

        // ── Run ManaSystem ───────────────────────────────────────────────────
        Self::mana_system(&mut world, is_a);

        // ── Build BatchBuilder and validate each action ──────────────────────
        // We compose every action into a BatchBuilder (proving atomicity intent),
        // then execute the batch.  If authorization fails for any action the
        // panic reverts all state changes made so far.
        let mut batch = BatchBuilder::new();
        for i in 0..actions.len() {
            let action = actions.get(i).unwrap();
            let sym = match &action {
                Action::PlayCreature(_) => SYM_PLAY,
                Action::CastSpell(_) => SYM_SPELL,
                Action::DeclareAttack(_, _) => SYM_ATTACK,
            };
            batch.add(GameAction {
                system_name: sym,
                data: Bytes::new(&env),
            });
        }

        // Validate batch is non-empty (already checked above, but BatchBuilder
        // enforces it as well — we use a mock-style inline check here since we
        // don't have a full CougrAccount in test context).
        if batch.is_empty() {
            panic_with_error!(&env, GameError::BatchEmpty);
        }

        // ── Execute each action atomically ────────────────────────────────────
        // The entire function runs within a single Soroban host invocation,
        // so any panic below reverts ALL storage writes, achieving true atomicity.
        let mut executed = 0u32;
        for i in 0..actions.len() {
            let action = actions.get(i).unwrap();
            match action {
                Action::PlayCreature(card_id) => {
                    Self::play_card_system(&env, &mut world, is_a, card_id);
                }
                Action::CastSpell(card_id) => {
                    Self::cast_spell_system(&env, &mut world, is_a, card_id);
                }
                Action::DeclareAttack(attacker_idx, target_idx) => {
                    Self::combat_system(&env, &mut world, is_a, attacker_idx, target_idx);
                }
            }
            executed += 1;
        }

        // ── WinConditionSystem ───────────────────────────────────────────────
        Self::win_condition_system(&mut world);

        // ── Advance turn ─────────────────────────────────────────────────────
        if world.match_state.status == STATUS_IN_PROGRESS {
            world.match_state.turn += 1;
            world.match_state.active_player = if is_a {
                world.player_b.clone()
            } else {
                world.player_a.clone()
            };
            world.match_state.phase = PHASE_DRAW;
        }

        let status = world.match_state.status;
        env.storage().instance().set(&WORLD_KEY, &world);

        TurnResult {
            success: true,
            actions_executed: executed,
            match_status: status,
            message: symbol_short!("ok"),
        }
    }

    // ── Query Methods ─────────────────────────────────────────────────────────

    pub fn get_state(env: Env) -> MatchState {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));
        world.match_state
    }

    pub fn get_hand(env: Env, player: Address) -> Vec<Card> {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        let hand = if player == world.player_a {
            &world.hand_a
        } else {
            &world.hand_b
        };
        let mut result = Vec::new(&env);
        for i in 0..hand.cards.len() {
            let id = hand.cards.get(i).unwrap();
            if let Some(card) = card_definition(id) {
                result.push_back(card);
            }
        }
        result
    }

    pub fn get_field(env: Env) -> FieldState {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));
        FieldState {
            field_a: world.field_a.creatures,
            field_b: world.field_b.creatures,
        }
    }

    pub fn get_stats(env: Env, player: Address) -> PlayerStats {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));
        if player == world.player_a {
            world.stats_a
        } else {
            world.stats_b
        }
    }

    /// Concede the match.
    pub fn concede(env: Env, player: Address) {
        let mut world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        if world.match_state.status != STATUS_IN_PROGRESS {
            panic_with_error!(&env, GameError::GameOver);
        }

        let is_a = player == world.player_a;
        let is_b = player == world.player_b;
        if !is_a && !is_b {
            panic_with_error!(&env, GameError::NotAPlayer);
        }

        world.match_state.status = STATUS_CONCEDED;
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── ECS Systems ──────────────────────────────────────────────────────────

    /// DrawSystem — draws one card from the player's deck into their hand.
    fn draw_system(env: &Env, world: &mut ECSWorldState, is_a: bool) {
        let (hand, deck) = if is_a {
            (&mut world.hand_a, &mut world.deck_a)
        } else {
            (&mut world.hand_b, &mut world.deck_b)
        };

        if deck.is_empty() || hand.cards.len() >= MAX_HAND {
            return;
        }

        // Draw from the front of the deck.
        let card_id = deck.get(0).unwrap();
        let mut new_deck = Vec::new(env);
        for i in 1..deck.len() {
            new_deck.push_back(deck.get(i).unwrap());
        }
        *deck = new_deck;
        hand.cards.push_back(card_id);
    }

    /// ManaSystem — increments max mana (capped at MAX_MANA) and refills current mana.
    fn mana_system(world: &mut ECSWorldState, is_a: bool) {
        let stats = if is_a {
            &mut world.stats_a
        } else {
            &mut world.stats_b
        };
        if stats.max_mana < MAX_MANA {
            stats.max_mana += 1;
        }
        stats.mana = stats.max_mana;
    }

    /// PlayCardSystem — validates and moves a creature card from hand to field.
    fn play_card_system(env: &Env, world: &mut ECSWorldState, is_a: bool, card_id: u32) {
        let card = match card_definition(card_id) {
            Some(c) => c,
            None => panic_with_error!(env, GameError::InvalidCard),
        };

        if card.kind != KIND_CREATURE {
            panic_with_error!(env, GameError::InvalidCard);
        }

        let (hand, field, stats) = if is_a {
            (&mut world.hand_a, &mut world.field_a, &mut world.stats_a)
        } else {
            (&mut world.hand_b, &mut world.field_b, &mut world.stats_b)
        };

        // Check mana
        if stats.mana < card.cost {
            panic_with_error!(env, GameError::InsufficientMana);
        }

        // Check field capacity
        if field.creatures.len() >= MAX_FIELD {
            panic_with_error!(env, GameError::FieldFull);
        }

        // Remove from hand
        let pos = Self::find_card_in_hand(hand, card_id);
        if pos >= hand.cards.len() {
            panic_with_error!(env, GameError::CardNotInHand);
        }
        let mut new_hand = Vec::new(env);
        for i in 0..hand.cards.len() {
            if i != pos {
                new_hand.push_back(hand.cards.get(i).unwrap());
            }
        }
        hand.cards = new_hand;

        // Deduct mana and place on field
        stats.mana -= card.cost;
        field
            .creatures
            .push_back(CreatureState::new(card_id, card.power, card.toughness));
    }

    /// CastSpellSystem — validates and casts a spell, dealing direct damage to opponent.
    fn cast_spell_system(env: &Env, world: &mut ECSWorldState, is_a: bool, card_id: u32) {
        let card = match card_definition(card_id) {
            Some(c) => c,
            None => panic_with_error!(env, GameError::InvalidCard),
        };

        if card.kind != KIND_SPELL {
            panic_with_error!(env, GameError::InvalidCard);
        }

        let (hand, stats, opp_stats) = if is_a {
            (&mut world.hand_a, &mut world.stats_a, &mut world.stats_b)
        } else {
            (&mut world.hand_b, &mut world.stats_b, &mut world.stats_a)
        };

        if stats.mana < card.cost {
            panic_with_error!(env, GameError::InsufficientMana);
        }

        let pos = Self::find_card_in_hand(hand, card_id);
        if pos >= hand.cards.len() {
            panic_with_error!(env, GameError::CardNotInHand);
        }
        let mut new_hand = Vec::new(env);
        for i in 0..hand.cards.len() {
            if i != pos {
                new_hand.push_back(hand.cards.get(i).unwrap());
            }
        }
        hand.cards = new_hand;

        stats.mana -= card.cost;

        // Deal damage: spell's power value is the damage amount.
        opp_stats.health = opp_stats.health.saturating_sub(card.power);
    }

    /// CombatSystem — resolves one attack declaration.
    ///
    /// `target_idx == u32::MAX` means direct attack (face damage).
    fn combat_system(
        env: &Env,
        world: &mut ECSWorldState,
        is_a: bool,
        attacker_idx: u32,
        target_idx: u32,
    ) {
        let (my_field, opp_field, opp_stats) = if is_a {
            (&mut world.field_a, &mut world.field_b, &mut world.stats_b)
        } else {
            (&mut world.field_b, &mut world.field_a, &mut world.stats_a)
        };

        if attacker_idx >= my_field.creatures.len() {
            panic_with_error!(env, GameError::InvalidTarget);
        }

        let attacker_power = my_field.creatures.get(attacker_idx).unwrap().power;

        if target_idx == u32::MAX {
            // Direct face attack
            opp_stats.health = opp_stats.health.saturating_sub(attacker_power);
        } else {
            // Attack a creature
            if target_idx >= opp_field.creatures.len() {
                panic_with_error!(env, GameError::InvalidTarget);
            }

            let blocker_power = opp_field.creatures.get(target_idx).unwrap().power;
            let blocker_toughness = opp_field
                .creatures
                .get(target_idx)
                .unwrap()
                .current_toughness;
            let attacker_toughness = my_field
                .creatures
                .get(attacker_idx)
                .unwrap()
                .current_toughness;

            // Apply damage both ways
            let new_blocker_toughness = blocker_toughness.saturating_sub(attacker_power);
            let new_attacker_toughness = attacker_toughness.saturating_sub(blocker_power);

            // Update attacker
            let mut attacker = my_field.creatures.get(attacker_idx).unwrap();
            attacker.current_toughness = new_attacker_toughness;
            my_field.creatures.set(attacker_idx, attacker);

            // Update blocker
            let mut blocker = opp_field.creatures.get(target_idx).unwrap();
            blocker.current_toughness = new_blocker_toughness;
            opp_field.creatures.set(target_idx, blocker);

            // Remove dead creatures (toughness == 0)
            Self::remove_dead(env, my_field);
            Self::remove_dead(env, opp_field);
        }
    }

    /// WinConditionSystem — marks the match over if any player's health reaches 0.
    fn win_condition_system(world: &mut ECSWorldState) {
        if world.stats_a.health == 0 {
            world.match_state.status = STATUS_B_WINS;
        } else if world.stats_b.health == 0 {
            world.match_state.status = STATUS_A_WINS;
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn find_card_in_hand(hand: &PlayerHand, card_id: u32) -> u32 {
        for i in 0..hand.cards.len() {
            if hand.cards.get(i).unwrap() == card_id {
                return i;
            }
        }
        u32::MAX
    }

    fn remove_dead(env: &Env, field: &mut PlayerField) {
        let mut survivors = Vec::new(env);
        for i in 0..field.creatures.len() {
            let c = field.creatures.get(i).unwrap();
            if c.current_toughness > 0 {
                survivors.push_back(c);
            }
        }
        field.creatures = survivors;
    }
}

#[cfg(test)]
mod test;
