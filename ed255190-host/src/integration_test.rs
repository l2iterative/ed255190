use crate::bytes_to_u32_digits;
use crate::eddsa::AffineEdwardsPoint;
use crate::hinter::HintBuilder;
use ark_ec::{AffineRepr, CurveGroup};
use ark_ed25519::{EdwardsAffine, Fr};
use ark_ff::{BigInteger, PrimeField, UniformRand};
use ed255190_guest::{ComputeHintStore, DualTableEvaluator, Evaluator, TEPoint};
use rand::{thread_rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::ops::Mul;

#[test]
fn evaluate_hint() {
    let mut prng = thread_rng();

    for _ in 0..100 {
        let s1_fe = Fr::rand(&mut prng);
        let s2_fe = Fr::rand(&mut prng);

        let s1: [u32; 8] = bytemuck::cast(s1_fe.into_bigint().0);
        let s2: [u32; 8] = bytemuck::cast(s2_fe.into_bigint().0);

        let g2_ge = EdwardsAffine::rand(&mut prng);

        let compute_hint = HintBuilder::build_unknown_g2(
            &s1,
            &s2,
            &AffineEdwardsPoint {
                x: g2_ge.x,
                y: g2_ge.y,
            },
        );

        let mut compute_hint_vec = Vec::new();
        for entry in compute_hint {
            for i in 0..8 {
                compute_hint_vec.push(entry[i]);
            }
        }
        let mut compute_hint_provider = ComputeHintStore::new(&compute_hint_vec);

        let eval = Evaluator::new(
            &s1,
            &s2,
            &TEPoint {
                x: bytemuck::cast(g2_ge.x.into_bigint().0),
                y: bytemuck::cast(g2_ge.y.into_bigint().0),
            },
        );

        let res = eval.evaluate(&mut compute_hint_provider);
        assert!(matches!(res, Ok(_)), "evaluation fails: {:?}", res);

        let sum = match res {
            Ok(v) => v,
            Err(_) => {
                unreachable!()
            }
        };

        let expected = (EdwardsAffine::generator().mul(&s1_fe) + g2_ge.mul(&s2_fe)).into_affine();

        assert_eq!(
            sum[0..8],
            bytes_to_u32_digits(&expected.x.into_bigint().to_bytes_le())
        );
        assert_eq!(
            sum[8..16],
            bytes_to_u32_digits(&expected.y.into_bigint().to_bytes_le())
        );
    }
}

#[test]
fn evaluate_hint_dual_table() {
    let mut prng = thread_rng();

    for _ in 0..100 {
        let s1_fe = Fr::rand(&mut prng);
        let s2_fe = Fr::rand(&mut prng);

        let s1: [u32; 8] = bytemuck::cast(s1_fe.into_bigint().0);
        let s2: [u32; 8] = bytemuck::cast(s2_fe.into_bigint().0);

        let compute_hint = HintBuilder::build_g2_in_table(&s1, &s2);

        let mut compute_hint_vec = Vec::new();
        for entry in compute_hint {
            for i in 0..8 {
                compute_hint_vec.push(entry[i]);
            }
        }
        let mut compute_hint_provider = ComputeHintStore::new(&compute_hint_vec);

        let eval = DualTableEvaluator::new(&s1, &s2);

        let res = eval.evaluate(&mut compute_hint_provider);
        assert!(matches!(res, Ok(_)), "evaluation fails: {:?}", res);

        let sum = match res {
            Ok(v) => v,
            Err(_) => {
                unreachable!()
            }
        };

        let g2 = {
            let mut prng = ChaCha20Rng::seed_from_u64(0u64);
            EdwardsAffine::rand(&mut prng)
        };

        let expected = (EdwardsAffine::generator().mul(&s1_fe) + g2.mul(&s2_fe)).into_affine();

        assert_eq!(
            sum[0..8],
            bytes_to_u32_digits(&expected.x.into_bigint().to_bytes_le())
        );
        assert_eq!(
            sum[8..16],
            bytes_to_u32_digits(&expected.y.into_bigint().to_bytes_le())
        );
    }
}
