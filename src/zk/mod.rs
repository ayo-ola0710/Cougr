//! Zero-knowledge proof support for Cougr.
//!
//! This module provides ergonomic wrappers around Stellar Protocol 25 (X-Ray)
//! cryptographic host functions for use in on-chain game verification.
//!
//! ## Architecture
//!
//! - **`types`**: Core ZK types (`G1Point`, `G2Point`, `Scalar`, `Groth16Proof`, `VerificationKey`)
//! - **`crypto`**: Low-level BN254 and Poseidon wrappers
//! - **`groth16`**: Groth16 proof verification
//! - **`error`**: ZK-specific error types
//! - **`testing`**: Mock types for unit testing without real proofs
//!
//! ## Usage
//!
//! ```no_run
//! use cougr_core::zk::{verify_groth16, G1Point, G2Point, Groth16Proof, Scalar, VerificationKey};
//! use soroban_sdk::{BytesN, Env, Vec};
//!
//! let env = Env::default();
//! let g1 = G1Point { bytes: BytesN::from_array(&env, &[0u8; 64]) };
//! let g2 = G2Point { bytes: BytesN::from_array(&env, &[0u8; 128]) };
//! let vk = VerificationKey {
//!     alpha: g1.clone(),
//!     beta: g2.clone(),
//!     gamma: g2.clone(),
//!     delta: g2,
//!     ic: Vec::from_array(&env, [g1.clone()]),
//! };
//! let proof = Groth16Proof { a: g1.clone(), b: vk.beta.clone(), c: g1 };
//! let public_inputs: [Scalar; 0] = [];
//! let _result = verify_groth16(&env, &vk, &proof, &public_inputs);
//! ```

pub mod bls12_381;
pub mod circuits;
pub mod commitment;
pub mod components;
pub mod crypto;
pub mod error;
pub mod groth16;
pub mod merkle;
pub mod systems;
pub mod testing;
pub mod traits;
pub mod types;

// Re-export commonly used items
pub use bls12_381::{
    bls12_381_g1_add, bls12_381_g1_msm, bls12_381_g1_mul, bls12_381_pairing_check,
};
pub use circuits::{
    CombatCircuit, CustomCircuit, CustomCircuitBuilder, InventoryCircuit, MovementCircuit,
    TurnSequenceCircuit,
};
pub use commitment::{pedersen_commit, pedersen_verify, PedersenCommitment, PedersenParams};
pub use components::{CommitReveal, HiddenState, ProofSubmission, VerifiedMarker};
#[cfg(feature = "hazmat-crypto")]
pub use crypto::{poseidon2_hash, poseidon2_hash_single, Poseidon2Params};
pub use error::ZKError;
pub use groth16::verify_groth16;
pub use merkle::{verify_inclusion, MerkleProof, MerkleTree, OnChainMerkleProof, SparseMerkleTree};
#[cfg(feature = "hazmat-crypto")]
pub use merkle::{
    verify_poseidon_proof, PoseidonMerkleProof, PoseidonMerkleTree, PoseidonSparseMerkleTree,
};
pub use systems::{
    cleanup_verified_system, commit_reveal_deadline_system, encode_commit_reveal,
    encode_verified_marker, verify_proofs_system,
};
pub use traits::{bytes32_to_scalar, i32_to_scalar, u32_to_scalar, GameCircuit};
pub use types::{
    Bls12381G1Point, Bls12381G2Point, Bls12381Scalar, G1Point, G2Point, Groth16Proof, Scalar,
    VerificationKey,
};
