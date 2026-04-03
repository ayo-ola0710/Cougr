//! Stable privacy surface for Cougr.
//!
//! This module groups the privacy primitives whose contracts are intentionally
//! documented and treated as the stable privacy subset during phase 2.

pub use super::commitment::{pedersen_commit, pedersen_verify, PedersenCommitment, PedersenParams};
pub use super::components::{CommitReveal, HiddenState, COMMIT_REVEAL_TYPE, HIDDEN_STATE_TYPE};
pub use super::interfaces::{
    Bytes32HiddenStateCodec, CommitmentScheme, HiddenStateCodec, MerkleProofVerifier,
    PedersenCommitmentScheme, ProofVerifier, Sha256MerkleProofVerifier,
};
pub use super::merkle::{
    verify_inclusion, MerkleProof, MerkleTree, OnChainMerkleProof, SparseMerkleTree,
};
