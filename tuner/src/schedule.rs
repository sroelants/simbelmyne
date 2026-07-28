pub trait LrSchedule {
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

impl LrSchedule for ConstantLr {
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

impl LrSchedule for LinearLr {
  fn rate(&self, epoch: usize, total: usize) -> f32 {
    let t = epoch as f32 / total as f32;
    self.start * (1.0 - t) + self.end * t
  }
}

pub trait WdlSchedule {
  fn blend(&self, epoch: usize, total: usize) -> f32;
}

pub struct ConstantWdl {
  val: f32,
}

impl ConstantWdl {
  pub fn new(val: f32) -> Self {
    Self { val }
  }
}

impl WdlSchedule for ConstantWdl {
  fn blend(&self, _epoch: usize, _total: usize) -> f32 {
    self.val
  }
}

pub struct LinearWdl {
  start: f32,
  end: f32,
}

impl LinearWdl {
  pub fn new(start: f32, end: f32) -> Self {
    Self { start, end }
  }
}

impl WdlSchedule for LinearWdl {
  fn blend(&self, epoch: usize, total: usize) -> f32 {
    let t = epoch as f32 / total as f32;
    self.start * (1.0 - t) + self.end * t
  }
}
