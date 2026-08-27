use crate::piece::Color;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
  Up = 0,
  UpRight = 1,
  Right = 2,
  DownRight = 3,
  Down = 4,
  DownLeft = 5,
  Left = 6,
  UpLeft = 7,
}

impl Direction {
  /// Rotate directions by 180 degrees to get relative (forward, backward)
  /// directions.
  #[inline(always)]
  pub const fn relative(self, side: Color) -> Self {
    Direction::from(self as u8 ^ ((side as u8) << 2))
  }

  #[inline(always)]
  pub const fn opp(self) -> Self {
    Direction::from(self as u8 ^ 4)
  }
}

const impl From<u8> for Direction {
  #[inline(always)]
  fn from(val: u8) -> Self {
    // SAFETY: val & 7 is guaranteed to be a valid Direction
    unsafe { std::mem::transmute(val & 7) }
  }
}
