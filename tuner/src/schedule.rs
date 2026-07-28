pub trait LrScheduler {
  fn rate(&self, epoch: usize, total: usize) -> f32;
}

pub struct ConstantLr {
  val: f32,
}

impl ConstantLr {
  pub fn new(val: f32) -> Self {
    ConstantLr { val }
  }
}

impl LrScheduler for ConstantLr {
  fn rate(&self, _epoch: usize, _total: usize) -> f32 {
    self.val
  }
}

pub struct LinearLr {
  start: f32,
  end: f32,
}

impl LinearLr {
  pub fn new(start: f32, end: f32) -> Self {
    Self { start, end }
  }
}

impl LrScheduler for LinearLr {
  fn rate(&self, epoch: usize, total: usize) -> f32 {
    let t = epoch as f32 / total as f32;
    self.start * (1.0 - t) + self.end * t
  }
}
