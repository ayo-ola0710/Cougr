//! Experimental privacy and proof-verification surface for Cougr.
//!
//! These exports remain intentionally outside Cougr's stable privacy promise.

pub use super::bls12_381::{
    bls12_381_g1_add, bls12_381_g1_msm, bls12_381_g1_mul, bls12_381_pairing_check,
};
pub use super::circuits::{
    CombatCircuit, CustomCircuit, CustomCircuitBuilder, InventoryCircuit, MovementCircuit,
    TurnSequenceCircuit,
};
pub use super::components::{
    ProofSubmission, VerifiedMarker, PROOF_SUBMISSION_TYPE, VERIFIED_MARKER_TYPE,
};
#[cfg(feature = "hazmat-crypto")]
pub use super::crypto::{
    poseidon2_hash, poseidon2_hash_single, poseidon_permutation, Poseidon2Params,
};
pub use super::groth16::{validate_groth16_contract, verify_groth16};
pub use super::interfaces::Groth16ProofVerifier;
#[cfg(feature = "hazmat-crypto")]
pub use super::merkle::{
    verify_poseidon_proof, PoseidonMerkleProof, PoseidonMerkleTree, PoseidonSparseMerkleTree,
};
pub use super::systems::{
    cleanup_verified_system, commit_reveal_deadline_system, encode_commit_reveal,
    decode_verified_at, encode_verified_marker, verify_proofs_system, verify_proofs_with,
};
pub use super::traits::{bytes32_to_scalar, i32_to_scalar, u32_to_scalar, GameCircuit};
pub use super::types::{Groth16Proof, VerificationKey};
