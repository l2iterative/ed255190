use ark_ec::{AffineRepr, CurveGroup};
use ark_ed25519::{EdwardsAffine, Fr};
use ark_ff::{BigInteger, PrimeField, UniformRand};
use l2r0_small_serde::to_vec_compact;
use methods::METHOD_ELF;
use risc0_zkvm::{ExecutorEnv, ExecutorImpl};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::ops::Mul;
use std::rc::Rc;

use ed255190_host::{AffineEdwardsPoint, HintBuilder};
use l2r0_profiler_host::CycleTracer;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub s1: [u32; 8],
    pub s2: [u32; 8],
    pub g2_x: [u32; 8],
    pub g2_y: [u32; 8],
}

fn main() {
    let mut prng = rand::thread_rng();

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

    let compute_hint_2 = HintBuilder::build_g2_in_table(&s1, &s2);

    for entry in compute_hint_2 {
        for i in 0..8 {
            compute_hint_vec.push(entry[i]);
        }
    }

    let task = Task {
        s1,
        s2,
        g2_x: bytemuck::cast(g2_ge.x.into_bigint().0),
        g2_y: bytemuck::cast(g2_ge.y.into_bigint().0),
    };

    let task_to_slice = to_vec_compact(&task).unwrap();

    let cycle_tracer = Rc::new(RefCell::new(CycleTracer::default()));

    let env = ExecutorEnv::builder()
        .segment_limit_po2(22)
        .write(&task_to_slice)
        .unwrap()
        .write(&((compute_hint_vec.len() * 4) as u32))
        .unwrap()
        .write_slice(&compute_hint_vec)
        .write_slice(&compute_hint_vec)
        .write_slice(&compute_hint_vec)
        .write_slice(&compute_hint_vec)
        .trace_callback(|e| {
            cycle_tracer.borrow_mut().handle_event(e);
            Ok(())
        })
        .build()
        .unwrap();

    let mut exec = ExecutorImpl::from_elf(env, METHOD_ELF).unwrap();
    let session = exec.run().unwrap();

    cycle_tracer.borrow().print();

    let expected = (EdwardsAffine::generator().mul(&s1_fe) + g2_ge.mul(&s2_fe)).into_affine();
    let g2 = {
        let mut prng = ChaCha20Rng::seed_from_u64(0u64);
        EdwardsAffine::rand(&mut prng)
    };

    let expected_2 = (EdwardsAffine::generator().mul(&s1_fe) + g2.mul(&s2_fe)).into_affine();

    let journal = session.journal.unwrap().bytes;
    assert_eq!(journal[0..32], expected.x.into_bigint().to_bytes_le());
    assert_eq!(journal[32..64], expected.y.into_bigint().to_bytes_le());
    assert_eq!(journal[64..96], expected_2.x.into_bigint().to_bytes_le());
    assert_eq!(journal[96..128], expected_2.y.into_bigint().to_bytes_le());
}
