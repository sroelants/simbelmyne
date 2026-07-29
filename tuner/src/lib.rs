use std::{path::PathBuf, time::Instant};

use chess::board::Board;

use crate::{
  batches::Batcher,
  data_entry::{Activation, DataEntry},
  loader::load_entries,
  logger::{Logger, Msg},
  optimizers::{Adam, AdamConfig, LossFn},
  schedule::{ConstantWdl, LinearLr, LinearWdl, LrSchedule, WdlSchedule},
  score::Score,
};

pub mod batches;
pub mod data_entry;
pub mod loader;
pub mod logger;
pub mod optimizers;
pub mod schedule;
pub mod score;

pub struct Tuner<Lr, Wdl>
where
  Lr: LrSchedule,
  Wdl: WdlSchedule,
{
  entries: Vec<DataEntry>,
  epochs: usize,
  k: f32,
  batch_size: u32,
  lr_schedule: Lr,
  wdl_schedule: Wdl,
  loss: LossFn,
  logger: Logger,
}

impl Tuner<LinearLr, LinearWdl> {
  pub fn new(epochs: usize, batch_size: u32) -> Self {
    Self {
      entries: vec![],
      epochs,
      batch_size,
      k: 0.01,
      lr_schedule: LinearLr::new(1.0, 0.0),
      wdl_schedule: LinearWdl::new(0.6, 0.9),
      loss: LossFn::MeanSquareError,
      logger: Logger::new(),
    }
  }
}

impl<Lr: LrSchedule, Wdl: WdlSchedule> Tuner<Lr, Wdl> {
  pub fn run<const N: usize>(&mut self, mut w: [Score; N]) -> [Score; N] {
    let epochs = self.epochs;
    let batch_size = self.batch_size;

    for epoch in 0..=epochs {
      let start = Instant::now();
      let mut cfg = AdamConfig::default();
      cfg.lrate = self.lr_schedule.rate(epoch, epochs);
      cfg.wdl = self.wdl_schedule.blend(epoch, epochs);

      let batcher = Batcher::new(&mut self.entries, batch_size);

      for batch in batcher.iter() {
        let optimizer = Adam::new(w, cfg);
        w = optimizer.run(&batch);
      }

      let loss = cfg.loss.batch_loss(&self.entries, &w, cfg.k);
      let ms = start.elapsed().as_millis();
      self.logger.report(epoch as u32, epochs as u32, loss, ms);
    }

    w
  }

  pub fn load(
    &mut self,
    file: PathBuf,
    positions: Option<usize>,
    activations: impl Fn(Board) -> (Vec<Activation>, i32) + Sync,
  ) {
    self.logger.info("Loading entries");
    self.entries = load_entries(file, positions, activations);

    let count = self.entries.len() as u32;
    self.logger.load(count);
  }

  pub fn log(&mut self, msg: Msg) {
    self.logger.log(msg);
  }
}
