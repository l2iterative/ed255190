use crate::hinter::ComputeHintStreamer;
use crate::structs::TEPoint;
use crate::EvaluationError;

pub struct Evaluator {
    pub s1: [u32; 8],
    pub s2: [u32; 8],
    pub g2: TEPoint,
}

impl Evaluator {
    pub fn new(s1: &[u32; 8], s2: &[u32; 8], g2: &TEPoint) -> Self {
        // Warning: format checking over s1 and s2 and g2 are not implemented yet.
        // TODO: add them

        Self {
            s1: *s1,
            s2: *s2,
            g2: g2.clone(),
        }
    }

    pub fn evaluate(
        &self,
        compute_hint: &mut impl ComputeHintStreamer,
    ) -> Result<[u32; 16], EvaluationError> {
        #[cfg(target_os = "zkvm")]
        let before_cycle = risc0_zkvm::guest::env::cycle_count();

        // Step 1: build a table for g2

        let mut table_g2 = [([0u32; 8], [0u32; 8]); 15];

        table_g2[0].0 = self.g2.x;
        table_g2[0].1 = self.g2.y;

        let mut cur = self.g2.clone();

        for i in 1..15 {
            let x3 = compute_hint.next();
            let y3 = compute_hint.next();

            let hint = TEPoint { x: x3, y: y3 };

            cur.check_add_hint(&self.g2, &hint)?;
            cur.x = x3;
            cur.y = y3;

            table_g2[i].0 = x3;
            table_g2[i].1 = y3;
        }

        // Step 2: start the addition

        let mut sum = TEPoint::default();
        for i in 0..8 {
            for j in 0..8 {
                if !(i == 0 && j == 0) {
                    for _ in 0..4 {
                        let x3 = compute_hint.next();
                        let y3 = compute_hint.next();
                        let hint = TEPoint { x: x3, y: y3 };
                        sum.check_dbl_hint(&hint)?;
                        sum = hint;
                    }
                }

                let bits_k1 = ((self.s1[7 - i] >> ((7 - j) * 4)) & 0xF) as usize;
                let bits_k2 = ((self.s2[7 - i] >> ((7 - j) * 4)) & 0xF) as usize;

                if bits_k1 != 0 {
                    let point = crate::G_TABLES[bits_k1 - 1];
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
                    let point = table_g2[bits_k2 - 1];
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
