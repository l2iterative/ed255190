use crate::eddsa::AffineEdwardsPoint;
use ark_ed25519::Fq;
use ark_ff::{Field, PrimeField};
use std::ops::Add;

pub struct HintBuilder {}

impl HintBuilder {
    pub fn build_unknown_g2(
        s1: &[u32; 8],
        s2: &[u32; 8],
        g2: &AffineEdwardsPoint,
    ) -> Vec<[u32; 8]> {
        let mut hints = Vec::<[u32; 8]>::new();

        // Step 1: build a table for g2

        let mut table_g2 = [(Fq::ZERO, Fq::ZERO); 15];

        table_g2[0].0 = g2.x.clone();
        table_g2[0].1 = g2.y.clone();

        let mut cur = g2.clone();

        for i in 1..15 {
            cur = cur.add(&g2);
            hints.push(bytemuck::cast(cur.x.into_bigint().0));
            hints.push(bytemuck::cast(cur.y.into_bigint().0));

            table_g2[i] = (cur.x, cur.y);
        }

        // Step 2: start the addition

        let mut sum = AffineEdwardsPoint::default();

        for i in 0..8 {
            for j in 0..8 {
                if !(i == 0 && j == 0) {
                    for _ in 0..4 {
                        sum = sum.double();
                        hints.push(bytemuck::cast(sum.x.into_bigint().0));
                        hints.push(bytemuck::cast(sum.y.into_bigint().0));
                    }
                }

                let bits_k1 = ((s1[7 - i] >> ((7 - j) * 4)) & 0xF) as usize;
                let bits_k2 = ((s2[7 - i] >> ((7 - j) * 4)) & 0xF) as usize;

                if bits_k1 != 0 {
                    let point = ed255190_guest::G_TABLE[bits_k1 - 1];

                    let x2 = Fq::from_le_bytes_mod_order(&bytemuck::cast_slice(&point.0));
                    let y2 = Fq::from_le_bytes_mod_order(&bytemuck::cast_slice(&point.1));

                    let rhs = AffineEdwardsPoint { x: x2, y: y2 };

                    sum = sum.add(&rhs);
                    hints.push(bytemuck::cast(sum.x.into_bigint().0));
                    hints.push(bytemuck::cast(sum.y.into_bigint().0));
                }

                if bits_k2 != 0 {
                    let point = table_g2[bits_k2 - 1];
                    sum = sum.add(&AffineEdwardsPoint {
                        x: point.0,
                        y: point.1,
                    });
                    hints.push(bytemuck::cast(sum.x.into_bigint().0));
                    hints.push(bytemuck::cast(sum.y.into_bigint().0));
                }
            }
        }

        hints
    }

    pub fn build_g2_in_table(s1: &[u32; 8], s2: &[u32; 8]) -> Vec<[u32; 8]> {
        let mut hints = Vec::<[u32; 8]>::new();
        let mut sum = AffineEdwardsPoint::default();

        for i in 0..8 {
            for j in 0..8 {
                let bits_k1 = ((s1[i] >> (j * 4)) & 0xF) as usize;
                let bits_k2 = ((s2[i] >> (j * 4)) & 0xF) as usize;

                if bits_k1 != 0 {
                    let point = ed255190_guest::G_LONG_TABLE[i * 8 + j][bits_k1 - 1];

                    let x2 = Fq::from_le_bytes_mod_order(&bytemuck::cast_slice(&point.0));
                    let y2 = Fq::from_le_bytes_mod_order(&bytemuck::cast_slice(&point.1));

                    let rhs = AffineEdwardsPoint { x: x2, y: y2 };

                    sum = sum.add(&rhs);
                    hints.push(bytemuck::cast(sum.x.into_bigint().0));
                    hints.push(bytemuck::cast(sum.y.into_bigint().0));
                }

                if bits_k2 != 0 {
                    let point = ed255190_guest::G2_LONG_TABLE[i * 8 + j][bits_k2 - 1];

                    let x2 = Fq::from_le_bytes_mod_order(&bytemuck::cast_slice(&point.0));
                    let y2 = Fq::from_le_bytes_mod_order(&bytemuck::cast_slice(&point.1));

                    sum = sum.add(&AffineEdwardsPoint { x: x2, y: y2 });
                    hints.push(bytemuck::cast(sum.x.into_bigint().0));
                    hints.push(bytemuck::cast(sum.y.into_bigint().0));
                }
            }
        }

        hints
    }
}
