//!
//! # Precompiled components(contracts)
//!

use ethereum_types::H160;
use evm::executor::stack::PrecompileFn;
use once_cell::sync::Lazy;
use ovr_evm_precompile_blake2::Blake2F;
use ovr_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use ovr_evm_precompile_curve25519::{Curve25519Add, Curve25519ScalarMul};
use ovr_evm_precompile_ed25519::Ed25519Verify;
use ovr_evm_precompile_modexp::Modexp;
use ovr_evm_precompile_sha3fips::Sha3FIPS256;
use ovr_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use ovr_fp_evm::Precompile;
use ruc::*;
use std::collections::BTreeMap;

pub(crate) static PRECOMPILE_SET: Lazy<BTreeMap<H160, PrecompileFn>> = Lazy::new(|| {
    map! {B
        idx_to_h160(1) => ECRecover::execute as PrecompileFn,
        idx_to_h160(2) => Sha256::execute,
        idx_to_h160(3) => Ripemd160::execute,
        idx_to_h160(4) => Identity::execute,
        idx_to_h160(5) => Modexp::execute,
        idx_to_h160(6) => ECRecoverPublicKey::execute,
        idx_to_h160(7) => Sha3FIPS256::execute,
        idx_to_h160(1024) => Blake2F::execute,
        idx_to_h160(1025) => Bn128Pairing::execute,
        idx_to_h160(1026) => Bn128Add::execute,
        idx_to_h160(1027) => Bn128Mul::execute,
        idx_to_h160(1028) => Curve25519Add::execute,
        idx_to_h160(1029) => Curve25519ScalarMul::execute,
        idx_to_h160(1030) => Ed25519Verify::execute,
    }
});

#[inline(always)]
pub(crate) fn idx_to_h160(i: u64) -> H160 {
    H160::from_low_u64_be(i)
}
