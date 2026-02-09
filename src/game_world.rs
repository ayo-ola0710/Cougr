//! Game world that integrates ECS, ZK proofs, and account authorization.
//!
//! `GameWorld` wraps a `SimpleWorld` with an account, providing a unified
//! interface for authorized gameplay with ZK proof verification.
//! Follows the same wrapper pattern as `HookedWorld`, `TrackedWorld`, `ObservedWorld`.
//!
//! # Example
//! ```ignore
//! let world = SimpleWorld::new(&env);
//! let account = ClassicAccount::new(player_address);
//! let mut game = GameWorld::new(world, account);
//!
//! // Execute authorized actions
//! game.execute_authorized(&env, &action)?;
//!
//! // Submit and verify ZK proofs
//! game.submit_proof(&env, entity_id, &vk, &proof, &public_inputs)?;
//! ```

use soroban_sdk::Env;

use crate::accounts::degradation::authorize_with_fallback;
use crate::accounts::error::AccountError;
use crate::accounts::traits::{CougrAccount, SessionKeyProvider};
use crate::accounts::types::{AccountCapabilities, GameAction, SessionKey, SessionScope};
use crate::component::ComponentTrait;
use crate::simple_world::{EntityId, SimpleWorld};
use crate::zk::error::ZKError;
use crate::zk::systems::verify_proofs_system;
use crate::zk::types::{Groth16Proof, Scalar, VerificationKey};

/// Game world that integrates ECS, ZK proofs, and account authorization.
pub struct GameWorld<A: CougrAccount> {
    world: SimpleWorld,
    account: A,
    active_session: Option<SessionKey>,
}

impl<A: CougrAccount> GameWorld<A> {
    /// Create a new game world with the given ECS world and account.
    pub fn new(world: SimpleWorld, account: A) -> Self {
        Self {
            world,
            account,
            active_session: None,
        }
    }

    /// Submit a ZK proof for an entity and verify it.
    ///
    /// On success, adds a `VerifiedMarker` component to the entity.
    pub fn submit_proof(
        &mut self,
        env: &Env,
        entity_id: EntityId,
        vk: &VerificationKey,
        proof: &Groth16Proof,
        public_inputs: &[Scalar],
    ) -> Result<bool, ZKError> {
        let verified =
            verify_proofs_system(&mut self.world, env, entity_id, vk, proof, public_inputs);
        Ok(verified)
    }

    /// Execute an authorized game action.
    ///
    /// Uses the active session if available and valid, otherwise falls back
    /// to direct authorization via the account.
    pub fn execute_authorized(&self, env: &Env, action: &GameAction) -> Result<(), AccountError> {
        authorize_with_fallback(env, &self.account, action, self.active_session.as_ref())
    }

    /// Set an active session key for this game world.
    pub fn set_session(&mut self, session: SessionKey) {
        self.active_session = Some(session);
    }

    /// End the active session.
    pub fn end_session(&mut self) {
        self.active_session = None;
    }

    /// Get the active session, if any.
    pub fn active_session(&self) -> Option<&SessionKey> {
        self.active_session.as_ref()
    }

    /// Get a reference to the inner ECS world.
    pub fn world(&self) -> &SimpleWorld {
        &self.world
    }

    /// Get a mutable reference to the inner ECS world.
    pub fn world_mut(&mut self) -> &mut SimpleWorld {
        &mut self.world
    }

    /// Get a reference to the account.
    pub fn account(&self) -> &A {
        &self.account
    }

    /// Decompose into inner world and account.
    pub fn into_inner(self) -> (SimpleWorld, A) {
        (self.world, self.account)
    }

    // ─── Typed convenience methods ────────────────────────────────

    /// Get a component and deserialize it into the concrete type.
    pub fn get_typed<T: ComponentTrait>(&self, env: &Env, entity_id: EntityId) -> Option<T> {
        self.world.get_typed(env, entity_id)
    }

    /// Serialize a component and store it.
    pub fn set_typed<T: ComponentTrait>(&mut self, env: &Env, entity_id: EntityId, component: &T) {
        self.world.set_typed(env, entity_id, component);
    }

    /// Check if an entity has a component of the given type.
    pub fn has_typed<T: ComponentTrait>(&self, entity_id: EntityId) -> bool {
        self.world.has_typed::<T>(entity_id)
    }

    /// Remove a component of the given type from an entity.
    pub fn remove_typed<T: ComponentTrait>(&mut self, entity_id: EntityId) -> bool {
        self.world.remove_typed::<T>(entity_id)
    }

    // ─── Player management ────────────────────────────────────────

    /// Register a new player entity in the game world.
    ///
    /// Creates a new entity that represents the player.
    pub fn register_player(&mut self) -> EntityId {
        self.world.spawn_entity()
    }

    /// Get the capabilities of the account bound to this game world.
    pub fn player_capabilities(&self) -> AccountCapabilities {
        self.account.capabilities()
    }

    /// Authorize a batch of game actions.
    ///
    /// Each action is authorized individually via the account.
    pub fn batch_execute(&self, env: &Env, actions: &[GameAction]) -> Result<(), AccountError> {
        for action in actions {
            self.execute_authorized(env, action)?;
        }
        Ok(())
    }
}

/// Extension methods for `GameWorld` when the account supports session keys.
impl<A: CougrAccount + SessionKeyProvider> GameWorld<A> {
    /// Start a new session for gasless gameplay.
    ///
    /// Creates a session key via the provider and sets it as the active session.
    pub fn start_session(
        &mut self,
        env: &Env,
        scope: SessionScope,
    ) -> Result<SessionKey, AccountError> {
        let key = self.account.create_session(env, scope)?;
        self.active_session = Some(key.clone());
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::testing::MockAccount;
    use crate::accounts::types::SessionScope;
    use crate::component::Position;
    use crate::zk::types::{G1Point, G2Point};
    use soroban_sdk::{symbol_short, vec, Bytes, BytesN, Env, Vec};

    fn make_game_world(env: &Env) -> GameWorld<MockAccount> {
        let world = SimpleWorld::new(env);
        let account = MockAccount::new(env);
        GameWorld::new(world, account)
    }

    fn make_vk(env: &Env) -> VerificationKey {
        let g1 = G1Point {
            bytes: BytesN::from_array(env, &[0u8; 64]),
        };
        let g2 = G2Point {
            bytes: BytesN::from_array(env, &[0u8; 128]),
        };
        VerificationKey {
            alpha: g1.clone(),
            beta: g2.clone(),
            gamma: g2.clone(),
            delta: g2,
            ic: Vec::new(env),
        }
    }

    #[test]
    fn test_game_world_creation() {
        let env = Env::default();
        let game = make_game_world(&env);
        assert_eq!(game.world().next_entity_id, 1);
        assert!(game.active_session().is_none());
    }

    #[test]
    fn test_game_world_world_access() {
        let env = Env::default();
        let mut game = make_game_world(&env);
        let entity_id = game.world_mut().spawn_entity();
        assert_eq!(entity_id, 1);
        assert_eq!(game.world().next_entity_id, 2);
    }

    #[test]
    fn test_game_world_execute_authorized() {
        let env = Env::default();
        let game = make_game_world(&env);
        let action = GameAction {
            system_name: symbol_short!("move"),
            data: Bytes::new(&env),
        };
        // MockAccount always succeeds
        assert!(game.execute_authorized(&env, &action).is_ok());
    }

    #[test]
    fn test_game_world_session_lifecycle() {
        let env = Env::default();
        let mut game = make_game_world(&env);

        assert!(game.active_session().is_none());

        let session = SessionKey {
            key_id: BytesN::from_array(&env, &[1u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("move")],
                max_operations: 100,
                expires_at: 5000,
            },
            created_at: 0,
            operations_used: 0,
        };

        game.set_session(session);
        assert!(game.active_session().is_some());

        game.end_session();
        assert!(game.active_session().is_none());
    }

    #[test]
    fn test_game_world_execute_with_session() {
        let env = Env::default();
        let mut game = make_game_world(&env);

        let session = SessionKey {
            key_id: BytesN::from_array(&env, &[1u8; 32]),
            scope: SessionScope {
                allowed_actions: vec![&env, symbol_short!("move")],
                max_operations: 100,
                expires_at: 5000,
            },
            created_at: 0,
            operations_used: 0,
        };
        game.set_session(session);

        let action = GameAction {
            system_name: symbol_short!("move"),
            data: Bytes::new(&env),
        };
        // Uses session key path
        assert!(game.execute_authorized(&env, &action).is_ok());
    }

    #[test]
    fn test_game_world_submit_proof_invalid() {
        let env = Env::default();
        let mut game = make_game_world(&env);
        let entity_id = game.world_mut().spawn_entity();

        let vk = make_vk(&env);
        let proof = Groth16Proof {
            a: G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
            b: G2Point {
                bytes: BytesN::from_array(&env, &[0u8; 128]),
            },
            c: G1Point {
                bytes: BytesN::from_array(&env, &[0u8; 64]),
            },
        };

        // With invalid VK (empty IC), verification should fail gracefully
        let result = game.submit_proof(&env, entity_id, &vk, &proof, &[]);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // verification fails but no error
    }

    #[test]
    fn test_game_world_into_inner() {
        let env = Env::default();
        let game = make_game_world(&env);
        let (world, _account) = game.into_inner();
        assert_eq!(world.next_entity_id, 1);
    }

    #[test]
    fn test_game_world_account_access() {
        let env = Env::default();
        let game = make_game_world(&env);
        let _caps = game.account().capabilities();
    }

    // ─── Typed API tests ──────────────────────────────────────────

    #[test]
    fn test_game_world_typed_component() {
        let env = Env::default();
        let mut game = make_game_world(&env);
        let e = game.world_mut().spawn_entity();

        game.set_typed(&env, e, &Position::new(5, 10));
        assert!(game.has_typed::<Position>(e));

        let pos: Position = game.get_typed(&env, e).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);

        assert!(game.remove_typed::<Position>(e));
        assert!(!game.has_typed::<Position>(e));
    }

    // ─── Player management tests ──────────────────────────────────

    #[test]
    fn test_register_player() {
        let env = Env::default();
        let mut game = make_game_world(&env);
        let p1 = game.register_player();
        let p2 = game.register_player();
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_player_capabilities() {
        let env = Env::default();
        let game = make_game_world(&env);
        let caps = game.player_capabilities();
        // MockAccount returns default capabilities
        assert!(caps.can_batch || !caps.can_batch); // just verify it doesn't panic
    }

    #[test]
    fn test_batch_execute() {
        let env = Env::default();
        let game = make_game_world(&env);
        let actions = [
            GameAction {
                system_name: symbol_short!("move"),
                data: Bytes::new(&env),
            },
            GameAction {
                system_name: symbol_short!("attack"),
                data: Bytes::new(&env),
            },
        ];
        // MockAccount always succeeds
        assert!(game.batch_execute(&env, &actions).is_ok());
    }

    #[test]
    fn test_batch_execute_empty() {
        let env = Env::default();
        let game = make_game_world(&env);
        assert!(game.batch_execute(&env, &[]).is_ok());
    }
}
