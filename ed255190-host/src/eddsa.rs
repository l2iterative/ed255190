use crate::COEFF_D;
use ark_ed25519::Fq;
use ark_ff::Field;
use std::ops::Add;
use std::str::FromStr;

pub struct AffineEdwardsPoint {
    pub x: Fq,
    pub y: Fq,
}

impl Add<&AffineEdwardsPoint> for &AffineEdwardsPoint {
    type Output = AffineEdwardsPoint;

    fn add(self, rhs: &AffineEdwardsPoint) -> Self::Output {
        let coeff_d = COEFF_D.get_or_init(|| {
            Fq::from_str(
                "37095705934669439343138083508754565189542113879843219016388785533085940283555",
            )
            .unwrap()
        });

        let x1x2 = &self.x * &rhs.x;
        let y1y2 = &self.y * &rhs.y;

        let x1y2 = &self.x * &rhs.y;
        let x2y1 = &rhs.x * &self.y;

        let dx1x2y1y2 = coeff_d * &x1x2 * &y1y2;
        let one_plus_dx1x2y1y2 = &Fq::ONE - &dx1x2y1y2;
        let one_minus_dx1x2y1y2 = &Fq::ONE + &dx1x2y1y2;

        let x1y2_plus_y1x2 = &x1y2 + &x2y1;
        let x = x1y2_plus_y1x2 * one_plus_dx1x2y1y2.inverse().unwrap();

        let y1y2_plus_x1x2 = &y1y2 + &x1x2;
        let y = y1y2_plus_x1x2 * one_minus_dx1x2y1y2.inverse().unwrap();

        AffineEdwardsPoint { x, y }
    }
}

impl AffineEdwardsPoint {
    pub fn double(&self) -> Self {
        self + self
    }
}
