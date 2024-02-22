mod error;
pub use error::EvaluationError;

mod hinter;
pub use hinter::{ComputeHintStore, Hint};

#[cfg(target_os = "zkvm")]
pub use hinter::ComputeHintBuffer;

mod structs;
pub use structs::TEPoint;

mod table;
pub use table::G_TABLE;

mod long_table;
pub use long_table::{G2_LONG_TABLE, G_LONG_TABLE};

mod evaluator;
pub use evaluator::Evaluator;

mod dual_table_evaluator;
pub(crate) mod utils;
pub use dual_table_evaluator::DualTableEvaluator;

static MODULUS_Q: [u32; 8] = [
    0xffffffedu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0x7fffffffu32,
];

static MODULUS_Q_MINUS_ONE: [u32; 8] = [
    0xffffffecu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0xffffffffu32,
    0x7fffffffu32,
];

static MODULUS_N: [u32; 8] = [
    0x5cf5d3edu32,
    0x5812631au32,
    0xa2f79cd6u32,
    0x14def9deu32,
    0u32,
    0u32,
    0u32,
    0x10000000u32,
];

static COEFF_D: [u32; 8] = [
    0x135978a3u32,
    0x75eb4dcau32,
    0x4141d8abu32,
    0x00700a4du32,
    0x7779e898u32,
    0x8cc74079u32,
    0x2b6ffe73u32,
    0x52036ceeu32,
];

static ONE: [u32; 8] = [1u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32];

static TWO: [u32; 8] = [2u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32];
