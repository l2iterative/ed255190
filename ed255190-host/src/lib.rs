use ark_ed25519::Fq;
use std::sync::OnceLock;

mod eddsa;
pub use eddsa::AffineEdwardsPoint;

mod hinter;
pub use hinter::HintBuilder;

mod table_generation;

#[cfg(test)]
mod integration_test;

static COEFF_D: OnceLock<Fq> = OnceLock::new();

#[inline]
pub fn bytes_to_u32_digits(fe: &[u8]) -> [u32; 8] {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(fe);
    bytemuck::cast::<[u8; 32], [u32; 8]>(bytes)
}
