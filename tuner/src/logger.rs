use colored::Colorize;
use std::{fmt::Display, time::Instant};

pub struct Logger {
  start: Instant,
  last_report: Instant,
}

impl Logger {
  pub fn new() -> Self {
    Self {
      start: Instant::now(),
      last_report: Instant::now(),
    }
  }

  pub fn log(&mut self, msg: Msg) {
    let s = match msg {
      Msg::Load(Load(entries)) => {
        format!("Loaded {entries} entries")
      }

      _ => format!("Hello"),
    };

    eprintln!("{s}");
  }

  fn print(&self, s: impl Display) {
    let timestamp = self.timestamp();
    eprintln!("{timestamp} {s}");
  }

  pub fn info(&self, info: &str) {
    self.print(info);
  }

  pub fn load(&self, entries: u32) {
    self.print(format!("Loaded {} entries", entries.to_string().blue()));
  }

  pub fn report(&mut self, epoch: u32, epochs: u32, loss: f32, ms: u128) {
    self.print(format!(
      "Epoch: {}/{epochs}, Loss: {} in {}ms",
      epoch.to_string().blue(),
      loss.to_string().blue(),
      ms.to_string().blue(),
    ));

    self.last_report = Instant::now();
  }

  pub fn timestamp(&self) -> String {
    let duration = self.start.elapsed().as_secs();
    let mins = duration / 60;
    let secs = duration % 60;

    format!("[{mins:0>2}:{secs:0>2}]")
      .bright_black()
      .to_string()
  }
}

pub enum Msg {
  Load(Load),
  Report(Report),
}

impl Msg {
  pub fn load(entries: u32) -> Self {
    Self::Load(Load(entries))
  }

  pub fn report(epoch: u32, epochs: u32, loss: f32) -> Self {
    Self::Report(Report {
      epoch,
      epochs,
      loss,
    })
  }
}

struct Load(pub u32);

struct Report {
  pub epoch: u32,
  pub epochs: u32,
  pub loss: f32,
}
