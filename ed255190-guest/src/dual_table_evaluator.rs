use crate::hinter::ComputeHintStreamer;
use crate::structs::TEPoint;
use crate::EvaluationError;

pub struct DualTableEvaluator {
    pub s1: [u32; 8],
    pub s2: [u32; 8],
}

impl DualTableEvaluator {
    pub fn new(s1: &[u32; 8], s2: &[u32; 8]) -> Self {
        // Warning: format checking over s1 and s2 and g2 are not implemented yet.
        // TODO: add them

        Self { s1: *s1, s2: *s2 }
    }

    pub fn evaluate(
        &self,
        compute_hint: &mut impl ComputeHintStreamer,
    ) -> Result<[u32; 16], EvaluationError> {
        #[cfg(target_os = "zkvm")]
        let before_cycle = risc0_zkvm::guest::env::cycle_count();

        let mut sum = TEPoint::default();
        for i in 0..8 {
            for j in 0..8 {
                let bits_k1 = ((self.s1[i] >> (j * 4)) & 0xF) as usize;
                let bits_k2 = ((self.s2[i] >> (j * 4)) & 0xF) as usize;

                if bits_k1 != 0 {
                    let point = crate::G_LONG_TABLE[i * 8 + j][bits_k1 - 1];
                    let rhs = TEPoint {
                        x: point.0,
                        y: point.1,
                    };
                    let x3 = compute_hint.next();
                    let y3 = compute_hint.next();
                    let hint = TEPoint { x: x3, y: y3 };

                    sum.check_add_hint(&rhs, &hint)?;
                    sum = hint;
                }

                if bits_k2 != 0 {
                    let point = crate::G2_LONG_TABLE[i * 8 + j][bits_k2 - 1];
                    let rhs = TEPoint {
                        x: point.0,
                        y: point.1,
                    };
                    let x3 = compute_hint.next();
                    let y3 = compute_hint.next();
                    let hint = TEPoint { x: x3, y: y3 };

                    sum.check_add_hint(&rhs, &hint)?;
                    sum = hint;
                }
            }
        }

        let mut res = [0u32; 16];
        res[0..8].copy_from_slice(&sum.x);
        res[8..16].copy_from_slice(&sum.y);

        #[cfg(target_os = "zkvm")]
        println!("{}", risc0_zkvm::guest::env::cycle_count() - before_cycle);

        Ok(res)
    }
}
