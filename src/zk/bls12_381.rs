//! BLS12-381 curve operation wrappers for Cougr.
//!
//! Provides ergonomic wrappers around Stellar Protocol 22+ BLS12-381
//! host functions. These complement the BN254 wrappers in `crypto.rs`.

use soroban_sdk::crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine, Fr as Bls12381Fr};
use soroban_sdk::{Env, Vec};

use super::error::ZKError;
use super::types::{Bls12381G1Point, Bls12381G2Point, Bls12381Scalar};

/// Add two BLS12-381 G1 points.
///
/// Wraps `env.crypto().bls12_381().g1_add()`.
pub fn bls12_381_g1_add(
    env: &Env,
    p1: &Bls12381G1Point,
    p2: &Bls12381G1Point,
) -> Result<Bls12381G1Point, ZKError> {
    let a = Bls12381G1Affine::from_bytes(p1.bytes.clone());
    let b = Bls12381G1Affine::from_bytes(p2.bytes.clone());
    let result = env.crypto().bls12_381().g1_add(&a, &b);
    Ok(Bls12381G1Point {
        bytes: result.to_bytes(),
    })
}

/// Multiply a BLS12-381 G1 point by a scalar.
///
/// Wraps `env.crypto().bls12_381().g1_mul()`.
pub fn bls12_381_g1_mul(
    env: &Env,
    point: &Bls12381G1Point,
    scalar: &Bls12381Scalar,
) -> Result<Bls12381G1Point, ZKError> {
    let p = Bls12381G1Affine::from_bytes(point.bytes.clone());
    let s = Bls12381Fr::from_bytes(scalar.bytes.clone());
    let result = env.crypto().bls12_381().g1_mul(&p, &s);
    Ok(Bls12381G1Point {
        bytes: result.to_bytes(),
    })
}

/// Multi-scalar multiplication on BLS12-381 G1.
///
/// Computes `sum(points[i] * scalars[i])` efficiently.
pub fn bls12_381_g1_msm(
    env: &Env,
    points: &[Bls12381G1Point],
    scalars: &[Bls12381Scalar],
) -> Result<Bls12381G1Point, ZKError> {
    if points.len() != scalars.len() {
        return Err(ZKError::InvalidInput);
    }
    if points.is_empty() {
        return Err(ZKError::InvalidInput);
    }

    let mut vp: Vec<Bls12381G1Affine> = Vec::new(env);
    let mut vs: Vec<Bls12381Fr> = Vec::new(env);

    for p in points {
        vp.push_back(Bls12381G1Affine::from_bytes(p.bytes.clone()));
    }
    for s in scalars {
        vs.push_back(Bls12381Fr::from_bytes(s.bytes.clone()));
    }

    let result = env.crypto().bls12_381().g1_msm(vp, vs);
    Ok(Bls12381G1Point {
        bytes: result.to_bytes(),
    })
}

/// BLS12-381 multi-pairing check.
///
/// Returns `true` if the pairing equation holds:
///
/// ```text
/// e(g1[0], g2[0]) * e(g1[1], g2[1]) * ... == 1
/// ```
pub fn bls12_381_pairing_check(
    env: &Env,
    g1_points: &[Bls12381G1Point],
    g2_points: &[Bls12381G2Point],
) -> Result<bool, ZKError> {
    if g1_points.len() != g2_points.len() {
        return Err(ZKError::InvalidInput);
    }
    if g1_points.is_empty() {
        return Err(ZKError::InvalidInput);
    }

    let mut vp1: Vec<Bls12381G1Affine> = Vec::new(env);
    let mut vp2: Vec<Bls12381G2Affine> = Vec::new(env);

    for p in g1_points {
        vp1.push_back(Bls12381G1Affine::from_bytes(p.bytes.clone()));
    }
    for p in g2_points {
        vp2.push_back(Bls12381G2Affine::from_bytes(p.bytes.clone()));
    }

    Ok(env.crypto().bls12_381().pairing_check(vp1, vp2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{BytesN, Env};

    #[test]
    fn test_bls12_381_pairing_check_empty_fails() {
        let result = bls12_381_pairing_check(&Env::default(), &[], &[]);
        assert_eq!(result, Err(ZKError::InvalidInput));
    }

    #[test]
    fn test_bls12_381_pairing_check_mismatched_lengths() {
        let env = Env::default();
        let g1 = Bls12381G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 96]),
        };
        let result = bls12_381_pairing_check(&env, &[g1], &[]);
        assert_eq!(result, Err(ZKError::InvalidInput));
    }

    #[test]
    fn test_bls12_381_g1_msm_empty_fails() {
        let result = bls12_381_g1_msm(&Env::default(), &[], &[]);
        assert_eq!(result.unwrap_err(), ZKError::InvalidInput);
    }

    #[test]
    fn test_bls12_381_g1_msm_mismatched_lengths() {
        let env = Env::default();
        let g1 = Bls12381G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 96]),
        };
        let result = bls12_381_g1_msm(&env, &[g1], &[]);
        assert_eq!(result.unwrap_err(), ZKError::InvalidInput);
    }

    #[test]
    fn test_bls12_381_g1_point_type_creation() {
        let env = Env::default();
        let point = Bls12381G1Point {
            bytes: BytesN::from_array(&env, &[0u8; 96]),
        };
        assert_eq!(point.bytes.len(), 96);
    }

    #[test]
    fn test_bls12_381_g2_point_type_creation() {
        let env = Env::default();
        let point = Bls12381G2Point {
            bytes: BytesN::from_array(&env, &[0u8; 192]),
        };
        assert_eq!(point.bytes.len(), 192);
    }

    #[test]
    fn test_bls12_381_scalar_type_creation() {
        let env = Env::default();
        let scalar = Bls12381Scalar {
            bytes: BytesN::from_array(&env, &[0u8; 32]),
        };
        assert_eq!(scalar.bytes.len(), 32);
    }
}
