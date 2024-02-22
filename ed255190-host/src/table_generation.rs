use ark_ec::{AffineRepr, CurveGroup, Group};
use ark_ed25519::EdwardsAffine;
use ark_ff::{BigInteger, PrimeField, UniformRand};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::ops::Add;

#[allow(unused)]
pub struct TableGeneration {
    pub g_series: [([u32; 8], [u32; 8]); 15],
    pub g_long_series: [[([u32; 8], [u32; 8]); 15]; 64],
    pub g2_long_series: [[([u32; 8], [u32; 8]); 15]; 64],
}

impl TableGeneration {
    pub fn new() -> Self {
        let mut g_series = [([0u32; 8], [0u32; 8]); 15];
        let mut g_long_series = [[([0u32; 8], [0u32; 8]); 15]; 64];
        let mut g2_long_series = [[([0u32; 8], [0u32; 8]); 15]; 64];

        let mut cur = EdwardsAffine::generator();

        let mut table_cur = cur.clone();

        for i in 0..15 {
            g_series[i].0.copy_from_slice(&bytemuck::cast_slice(
                &table_cur.x().unwrap().into_bigint().to_bytes_le(),
            ));
            g_series[i].1.copy_from_slice(&bytemuck::cast_slice(
                &table_cur.y().unwrap().into_bigint().to_bytes_le(),
            ));

            if i != 14 {
                table_cur = table_cur.add(&cur).into_affine();
            }
        }

        for j in 0..15 {
            g_long_series[0][j] = g_series[j];
        }

        for i in 1..64 {
            cur = cur
                .into_group()
                .double()
                .double()
                .double()
                .double()
                .into_affine();

            let mut table_cur = cur.clone();

            for j in 0..15 {
                g_long_series[i][j].0.copy_from_slice(&bytemuck::cast_slice(
                    &table_cur.x().unwrap().into_bigint().to_bytes_le(),
                ));
                g_long_series[i][j].1.copy_from_slice(&bytemuck::cast_slice(
                    &table_cur.y().unwrap().into_bigint().to_bytes_le(),
                ));

                if j != 14 {
                    table_cur = table_cur.add(&cur).into_affine();
                }
            }
        }

        let mut prng = ChaCha20Rng::seed_from_u64(0u64);
        let g2 = EdwardsAffine::rand(&mut prng);
        let mut cur = g2;
        for i in 0..64 {
            if i != 0 {
                cur = cur
                    .into_group()
                    .double()
                    .double()
                    .double()
                    .double()
                    .into_affine();
            }

            let mut table_cur = cur.clone();

            for j in 0..15 {
                g2_long_series[i][j]
                    .0
                    .copy_from_slice(&bytemuck::cast_slice(
                        &table_cur.x().unwrap().into_bigint().to_bytes_le(),
                    ));
                g2_long_series[i][j]
                    .1
                    .copy_from_slice(&bytemuck::cast_slice(
                        &table_cur.y().unwrap().into_bigint().to_bytes_le(),
                    ));

                if j != 14 {
                    table_cur = table_cur.add(&cur).into_affine();
                }
            }
        }

        Self {
            g_series,
            g_long_series,
            g2_long_series,
        }
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

        println!();

        println!("G1 Long table:");
        print!("[");
        for i in 0..64 {
            println!("[");
            for j in 0..15 {
                println!("([");
                for v in self.g_long_series[i][j].0.iter() {
                    print!("{}u32,", v);
                }
                print!("],");
                print!("[");
                for v in self.g_long_series[i][j].1.iter() {
                    print!("{}u32,", v);
                }
                println!("]),");
            }
            println!("],");
        }
        print!("]");

        println!();

        println!("G2 Long table:");
        print!("[");
        for i in 0..64 {
            println!("[");
            for j in 0..15 {
                println!("([");
                for v in self.g2_long_series[i][j].0.iter() {
                    print!("{}u32,", v);
                }
                print!("],");
                print!("[");
                for v in self.g2_long_series[i][j].1.iter() {
                    print!("{}u32,", v);
                }
                println!("]),");
            }
            println!("],");
        }
        print!("]");
    }
}

#[cfg(test)]
mod test {
    use crate::table_generation::TableGeneration;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ed25519::{EdwardsAffine, Fq, Fr};
    use ark_ff::{BigInteger, PrimeField, UniformRand};
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use std::ops::Mul;

    #[test]
    fn check_consistency() {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);
        let g2 = EdwardsAffine::rand(&mut prng);

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

        for i in 0..64 {
            for j in 0..15 {
                let mut r_bigint = Fr::from((j + 1) as u8).into_bigint();
                r_bigint.muln((i as u32) * 4);

                let r = Fr::from_le_bytes_mod_order(&r_bigint.to_bytes_le());
                let point_r = EdwardsAffine::generator().mul(&r).into_affine();

                let x = point_r.x;
                let y = point_r.y;

                let x_reconstructed = Fq::from_le_bytes_mod_order(
                    &bytemuck::cast_slice::<u32, u8>(&hint.g_long_series[i][j].0),
                );
                let y_reconstructed = Fq::from_le_bytes_mod_order(
                    &bytemuck::cast_slice::<u32, u8>(&hint.g_long_series[i][j].1),
                );

                assert_eq!(x, x_reconstructed);
                assert_eq!(y, y_reconstructed);
            }
        }

        for i in 0..64 {
            for j in 0..15 {
                let mut r_bigint = Fr::from((j + 1) as u8).into_bigint();
                r_bigint.muln((i as u32) * 4);

                let r = Fr::from_le_bytes_mod_order(&r_bigint.to_bytes_le());
                let point_r = g2.mul(&r).into_affine();

                let x = point_r.x;
                let y = point_r.y;

                let x_reconstructed = Fq::from_le_bytes_mod_order(
                    &bytemuck::cast_slice::<u32, u8>(&hint.g2_long_series[i][j].0),
                );
                let y_reconstructed = Fq::from_le_bytes_mod_order(
                    &bytemuck::cast_slice::<u32, u8>(&hint.g2_long_series[i][j].1),
                );

                assert_eq!(x, x_reconstructed);
                assert_eq!(y, y_reconstructed);
            }
        }
    }
}
