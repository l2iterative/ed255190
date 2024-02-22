use ark_ed25519::Fq;
use std::sync::OnceLock;

mod eddsa;
mod table_generation;

static COEFF_D: OnceLock<Fq> = OnceLock::new();
