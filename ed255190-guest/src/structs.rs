use crate::hinter::DecompressionHint;
use crate::utils::mul_mod;
use crate::{EvaluationError, COEFF_D, MODULUS_Q, ONE, TWO};

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

#[derive(Clone)]
pub struct TEPoint {
    pub x: [u32; 8],
    pub y: [u32; 8],
}

impl CompressedEdwardsY {
    pub fn decompose_with_hints(
        &self,
        hint: &DecompressionHint,
    ) -> Result<TEPoint, EvaluationError> {
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
        for i in 0..8 {
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

impl Default for TEPoint {
    fn default() -> Self {
        Self {
            x: [0u32; 8],
            y: [1u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32, 0u32],
        }
    }
}

impl TEPoint {
    pub fn check_add_hint(&self, rhs: &Self, hint: &Self) -> Result<(), EvaluationError> {
        let TEPoint { x: x1, y: y1 } = &self;
        let TEPoint { x: x2, y: y2 } = &rhs;
        let TEPoint { x: x3, y: y3 } = &hint;

        let x1x2 = mul_mod(x1, x2, &MODULUS_Q);
        let y1y2 = mul_mod(y1, y2, &MODULUS_Q);

        let x1y2 = mul_mod(x1, y2, &MODULUS_Q);
        let y1x2 = mul_mod(x2, y1, &MODULUS_Q);

        let mut dx1x2y1y2 = mul_mod(&x1x2, &y1y2, &MODULUS_Q);
        dx1x2y1y2 = mul_mod(&dx1x2y1y2, &COEFF_D, &MODULUS_Q);

        let mut x1y2_plus_y1x2 = x1y2.clone();
        let _ = add::<8, 8>(&mut x1y2_plus_y1x2, &y1x2);
        x1y2_plus_y1x2 = mul_mod(&x1y2_plus_y1x2, &ONE, &MODULUS_Q);

        let mut dx1x2y1y2_plus_one = dx1x2y1y2.clone();
        let _ = add::<8, 1>(&mut dx1x2y1y2_plus_one, &[1u32]);
        let x3_times_dx1x2y1y2_plus_one = mul_mod(&dx1x2y1y2_plus_one, &x3, &MODULUS_Q);

        // x3 * (1 + d * x1 * y1 * x2 * y2) = x1 * y2 + y1 * x2
        let mut first_equation_is_equal = true;
        for i in 0..8 {
            if x3_times_dx1x2y1y2_plus_one[i] != x1y2_plus_y1x2[i] {
                first_equation_is_equal = false;
                break;
            }
        }

        if !first_equation_is_equal {
            return Err(EvaluationError::WrongHint);
        }

        let mut should_be_y3 = mul_mod(&dx1x2y1y2, &y3, &MODULUS_Q);
        let _ = add::<8, 8>(&mut should_be_y3, &y1y2);
        should_be_y3 = mul_mod(&should_be_y3, &ONE, &MODULUS_Q);
        let _ = add::<8, 8>(&mut should_be_y3, &x1x2);
        should_be_y3 = mul_mod(&should_be_y3, &ONE, &MODULUS_Q);

        // y3 =  x1 * x2 +  y1 * y2 + d * x1 * y1 * x2 * y2 * y3
        let mut second_equation_is_equal = true;
        for i in 0..8 {
            if should_be_y3[i] != y3[i] {
                second_equation_is_equal = false;
                break;
            }
        }

        if !second_equation_is_equal {
            return Err(EvaluationError::WrongHint);
        }

        Ok(())
    }

    pub fn check_dbl_hint(&self, hint: &Self) -> Result<(), EvaluationError> {
        let TEPoint { x: x1, y: y1 } = &self;
        let TEPoint { x: x3, y: y3 } = &hint;

        let x1x1 = mul_mod(x1, x1, &MODULUS_Q);
        let y1y1 = mul_mod(y1, y1, &MODULUS_Q);
        let x1y1 = mul_mod(x1, y1, &MODULUS_Q);

        let mut dx1x1y1y1 = mul_mod(&x1x1, &y1y1, &MODULUS_Q);
        dx1x1y1y1 = mul_mod(&dx1x1y1y1, &COEFF_D, &MODULUS_Q);

        let mut x1y1_plus_y1x1 = mul_mod(&x1y1, &TWO, &MODULUS_Q);

        let mut dx1x1y1y1_plus_one = dx1x1y1y1.clone();
        let _ = add::<8, 1>(&mut dx1x1y1y1_plus_one, &[1u32]);

        let x3_times_dx1x1y1y1_plus_one = mul_mod(&dx1x1y1y1_plus_one, &x3, &MODULUS_Q);

        // x3 * (1 + d * x1 * x1 * y1 * y1) = 2 * x1 * y1
        let mut first_equation_is_equal = true;
        for i in 0..8 {
            if x3_times_dx1x1y1y1_plus_one[i] != x1y1_plus_y1x1[i] {
                first_equation_is_equal = false;
                break;
            }
        }

        if !first_equation_is_equal {
            return Err(EvaluationError::WrongHint);
        }

        let mut should_be_y3 = mul_mod(&dx1x1y1y1, &y3, &MODULUS_Q);
        let _ = add::<8, 8>(&mut should_be_y3, &y1y1);
        should_be_y3 = mul_mod(&should_be_y3, &ONE, &MODULUS_Q);
        let _ = add::<8, 8>(&mut should_be_y3, &x1x1);
        should_be_y3 = mul_mod(&should_be_y3, &ONE, &MODULUS_Q);

        // y3 = x1 * x1 + y1 * y1 + d * x1 * y1 * x1 * y1 * y3
        let mut second_equation_is_equal = true;
        for i in 0..8 {
            if should_be_y3[i] != y3[i] {
                second_equation_is_equal = false;
                break;
            }
        }

        if !second_equation_is_equal {
            return Err(EvaluationError::WrongHint);
        }

        Ok(())
    }
}
