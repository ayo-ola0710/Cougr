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
//! ```ignore
//! use cougr_core::zk::{crypto, groth16, types::*};
//!
//! // Verify a Groth16 proof on-chain
//! let result = groth16::verify_groth16(&env, &vk, &proof, &public_inputs);
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
#[cfg(any(test, feature = "testutils"))]
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
