use crate::{COEFF_D, EvaluationError, MODULUS_Q, ONE};
use crate::hinter::DecompressionHint;
use crate::utils::mul_mod;

#[inline(always)]
pub fn add32_and_overflow(a: u32, b: u32, carry: u32) -> (u32, u32) {
    let v = (a as u64).wrapping_add(b as u64).wrapping_add(carry as u64);
    ((v >> 32) as u32, (v & 0xffffffff) as u32)
}

#[inline(always)]
pub fn carry32_and_overflow(a: u32, carry: u32) -> (u32, u32) {
    let (v, carry) = a.overflowing_add(carry);
    (carry as u32, v)
}

#[inline]
pub fn add<const I: usize, const J: usize>(accm: &mut [u32; I], new: &[u32; J]) -> u32 {
    let mut carry = 0;
    (carry, accm[0]) = add32_and_overflow(accm[0], new[0], carry);
    for i in 1..J {
        (carry, accm[i]) = add32_and_overflow(accm[i], new[i], carry);
    }
    for i in J..I {
        (carry, accm[i]) = carry32_and_overflow(accm[i], carry);
    }
    carry
}

#[inline]
pub fn overflow(accm: &mut [u32; 8]) {
    let mut carry;
    (carry, accm[0]) = add32_and_overflow(accm[0], 0x000003d1u32, 0);
    (carry, accm[1]) = add32_and_overflow(accm[1], 0x1u32, carry);
    (carry, accm[2]) = carry32_and_overflow(accm[2], carry);
    (carry, accm[3]) = carry32_and_overflow(accm[3], carry);
    (carry, accm[4]) = carry32_and_overflow(accm[4], carry);
    (carry, accm[5]) = carry32_and_overflow(accm[5], carry);
    (carry, accm[6]) = carry32_and_overflow(accm[6], carry);
    (_, accm[7]) = carry32_and_overflow(accm[7], carry);
}

pub struct CompressedEdwardsY(pub [u32; 8]);

pub struct TEPoint {
    pub x: [u32; 8],
    pub y: [u32; 8],
}

impl CompressedEdwardsY {
    pub fn decompose_with_hints(&self, hint: &DecompressionHint) -> Result<TEPoint, EvaluationError> {
        let mut res = TEPoint {
            x: hint.x,
            y: self.0,
        };
        res.y[7] &= 0x7fffffff;
        
        let x_expected_sign = (self.0[7] >> 31) != 0;
        
        let x_square = mul_mod(&res.x, &res.x, &MODULUS_Q);
        let y_square = mul_mod(&res.y, &res.y, &MODULUS_Q);
        
        let mut rhs = mul_mod(&x_square, &y_square, &MODULUS_Q);
        rhs = mul_mod(&rhs, &COEFF_D, &MODULUS_Q);
        
        // it would not overflow to 2^256 for such additions
        let _ = add::<8, 1>(&mut rhs, &[1u32]);
        let _ = add::<8, 8>(&mut rhs, &x_square);
        
        rhs = mul_mod(&rhs, &ONE, &MODULUS_Q);
        
        let mut lhs_equal_rhs = true;
        for i in 0..8{
            if x_square[i] != rhs[i] {
                lhs_equal_rhs = false;
                break;
            }
        }
        
        if !lhs_equal_rhs {
            return Err(EvaluationError::WrongHint);
        }
        
        let x_actual_sign = (res.x[0] & 1) != 0;
        
        if x_actual_sign != x_expected_sign {
            return Err(EvaluationError::WrongHint);
        }
        
        Ok(res)
    }
}