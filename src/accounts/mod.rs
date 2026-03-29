//! Account abstraction for Cougr game accounts.
//!
//! This module provides a unified interface for both Classic (G-address)
//! and Contract (C-address) Stellar accounts, enabling features like
//! session keys for gasless gameplay.
//!
//! ## Architecture
//!
//! - **`types`**: Core account types (`GameAction`, `SessionScope`, `SessionKey`, etc.)
//! - **`traits`**: `CougrAccount` and `SessionKeyProvider` traits
//! - **`classic`**: Classic Stellar account implementation
//! - **`contract`**: Contract account with session key support
//! - **`error`**: Account-specific error types
//! - **`testing`**: Mock account for unit testing
//!
//! ## Usage
//!
//! ```ignore
//! use cougr_core::accounts::{ClassicAccount, CougrAccount};
//!
//! let account = ClassicAccount::new(player_address);
//! account.authorize(&env, &action)?;
//! ```

pub mod batch;
pub mod classic;
pub mod contract;
pub mod degradation;
pub mod device_storage;
pub mod error;
pub mod intent;
pub mod kernel;
pub mod multi_device;
pub mod policy;
pub mod replay;
pub mod recovery;
pub mod recovery_storage;
pub mod secp256r1_auth;
pub mod session_builder;
pub mod storage;
#[cfg(any(test, feature = "testutils"))]
pub mod testing;
pub mod traits;
pub mod types;
pub mod signer;

// Re-export commonly used items
pub use batch::BatchBuilder;
pub use classic::ClassicAccount;
pub use contract::ContractAccount;
pub use degradation::{authorize_with_fallback, batch_or_sequential, require_capability};
pub use device_storage::DeviceStorage;
pub use error::AccountError;
pub use intent::{AuthMethod, AuthResult, IntentProof, IntentProofKind, IntentSigner, SignedIntent, SignerRef};
pub use kernel::AccountKernel;
pub use multi_device::{DeviceKey, DevicePolicy, MultiDeviceProvider};
pub use policy::{ActiveDevicePolicy, GuardianPolicy, IntentContext, IntentExpiryPolicy, Policy, RecoveryContext, SessionContext, SessionPolicy};
pub use replay::ReplayProtection;
pub use recovery::{Guardian, RecoveryConfig, RecoveryProvider, RecoveryRequest};
pub use recovery_storage::RecoveryStorage;
pub use secp256r1_auth::{Secp256r1Key, Secp256r1Storage};
pub use session_builder::SessionBuilder;
pub use storage::SessionStorage;
#[cfg(any(test, feature = "testutils"))]
pub use testing::MockAccount;
pub use signer::{AccountSigner, DirectAuthSigner, Secp256r1PasskeySigner, SessionAuthSigner};
pub use traits::{CougrAccount, IntentAccount, SessionKeyProvider};
pub use types::{AccountCapabilities, GameAction, SessionKey, SessionScope};
