use ark_ec::{AffineRepr, CurveGroup, Group};
use ark_ed25519::EdwardsAffine;
use ark_ff::{BigInteger, PrimeField};
use std::ops::Add;

#[allow(unused)]
pub struct TableGeneration {
    pub g_series: [([u32; 8], [u32; 8]); 15],
}

impl TableGeneration {
    pub fn new() -> Self {
        let mut res = [([0u32; 8], [0u32; 8]); 15];

        let mut cur = EdwardsAffine::generator();

        let mut table_cur = cur.clone();

        for i in 0..15 {
            res[i].0.copy_from_slice(&bytemuck::cast_slice(
                &table_cur.x().unwrap().into_bigint().to_bytes_le(),
            ));
            res[i].1.copy_from_slice(&bytemuck::cast_slice(
                &table_cur.y().unwrap().into_bigint().to_bytes_le(),
            ));

            if i != 14 {
                table_cur = table_cur.add(&cur).into_affine();
            }
        }

        Self { g_series: res }
    }

    pub fn print(&self) {
        println!("Main table:");

        print!("[");
        for i in 0..15 {
            println!("([");
            for v in self.g_series[i].0.iter() {
                print!("{}u32,", v);
            }
            print!("],");
            print!("[");
            for v in self.g_series[i].1.iter() {
                print!("{}u32,", v);
            }
            println!("]),");
        }
        print!("]");
    }
}

#[cfg(test)]
mod test {
    use crate::table_generation::TableGeneration;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ed25519::{EdwardsAffine, Fq, Fr};
    use ark_ff::{BigInteger, PrimeField};
    use std::ops::Mul;

    #[test]
    fn check_consistency() {
        let hint = TableGeneration::new();
        // hint.print();

        for i in 0..15 {
            let r_bigint = Fr::from((i + 1) as u8).into_bigint();

            let r = Fr::from_le_bytes_mod_order(&r_bigint.to_bytes_le());
            let point_r = EdwardsAffine::generator().mul(&r).into_affine();

            let x = point_r.x;
            let y = point_r.y;

            let x_reconstructed =
                Fq::from_le_bytes_mod_order(&bytemuck::cast_slice::<u32, u8>(&hint.g_series[i].0));
            let y_reconstructed =
                Fq::from_le_bytes_mod_order(&bytemuck::cast_slice::<u32, u8>(&hint.g_series[i].1));

            assert_eq!(x, x_reconstructed);
            assert_eq!(y, y_reconstructed);
        }
    }
}
