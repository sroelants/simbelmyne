use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::SubAssign;

/// A packed pair of midgame/endgame scores, for more efficient and ergonomic
/// calculation.
#[derive(Debug, Default, Copy, Clone)]
pub struct Score {
    pub mg: f32,
    pub eg: f32
}

impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { mg: self.mg + rhs.mg, eg: self.eg + rhs.eg }
    }
}

impl Add<f32> for Score {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self { mg: self.mg + rhs as f32, eg: self.eg + rhs as f32 }
    }
}

impl AddAssign<f32> for Score {
    fn add_assign(&mut self, rhs: f32) {
        self.mg += rhs as f32;
        self.eg += rhs as f32;
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        self.mg += rhs.mg;
        self.eg += rhs.eg;
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { mg: self.mg - rhs.mg, eg: self.eg - rhs.eg }
    }
}

impl Sub<f32> for Score {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self { mg: self.mg - rhs as f32, eg: self.eg - rhs as f32 }
    }
}

impl SubAssign<f32> for Score {
    fn sub_assign(&mut self, rhs: f32) {
        self.mg -= rhs as f32;
        self.eg -= rhs as f32;
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, rhs: Self) {
        self.mg -= rhs.mg;
        self.eg -= rhs.eg;
    }
}

impl Mul<f32> for Score {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self { mg: rhs * self.mg, eg: rhs * self.eg }
    }
}

impl Mul for Score {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self { mg: self.mg * rhs.mg, eg: self.eg * rhs.eg }
    }
}

impl Div<f32> for Score {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self { mg: self.mg / rhs, eg: self.eg / rhs }
    }
}

impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Score::default(), Score::add)
    }
}
