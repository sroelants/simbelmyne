use anyhow::*;
use chess::board::Board;
use chess::movegen::moves::Move;
use chess::san::ToSan;
use colored::Colorize;
use std::fmt::Display;
use std::fmt::Write;
use std::str::FromStr;

use crate::wdl::WdlParams;

/// Information we might want to print in a UCI `info` message
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SearchInfo {
  /// The nominal search depth
  pub depth: Option<u8>,

  /// The selective search depth (e.g., max depth in Qsearch)
  pub seldepth: Option<u8>,

  /// The total duration of the search so far
  pub time: Option<u64>,

  /// The number of nodes searched so far. Even though we're doing iterative
  /// deepening, this only includes nodes from the last search iteration.
  pub nodes: Option<u32>,

  /// The highest score we've obtained so far
  pub score: Option<Score>,

  /// The move we're currently searching
  pub currmove: Option<Move>,

  /// The number of the move we're currently searching
  pub currmovenumber: Option<u8>,

  /// How full is the transposition table, as a value per mille.
  pub hashfull: Option<u32>,

  /// The number of nodes searched per second
  pub nps: Option<u64>,

  /// The current principal variation
  pub pv: Vec<Move>,
}

impl Display for SearchInfo {
  /// Format the SearchInfo as a UCI compliant log message
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(depth) = self.depth {
      write!(f, "depth {depth} ")?;
    }

    if let Some(seldepth) = self.seldepth {
      write!(f, "seldepth {seldepth} ")?;
    }

    if let Some(time) = self.time {
      write!(f, "time {time} ")?;
    }

    if let Some(nodes) = self.nodes {
      write!(f, "nodes {nodes} ")?;
    }

    if let Some(score) = self.score {
      write!(f, "score {score} ")?;
    }

    if let Some(currmove) = self.currmove {
      write!(f, "currmove {currmove} ")?;
    }

    if let Some(currmovenumber) = self.currmovenumber {
      write!(f, "currmovenumber {currmovenumber} ")?;
    }

    if let Some(hashfull) = self.hashfull {
      write!(f, "hashfull {hashfull} ")?;
    }

    if let Some(nps) = self.nps {
      write!(f, "nps {nps} ")?;
    }

    if self.pv.len() > 0 {
      write!(f, "pv ")?;

      for mv in self.pv.iter() {
        write!(f, "{mv} ")?;
      }
    }

    std::fmt::Result::Ok(())
  }
}

impl SearchInfo {
  /// Format the SearchInfo as a UCI compliant log message, using the provided
  /// WDL parameters to rescale the score such that an advantage of 100cp
  /// corresponds to a 50% chance of winning.
  pub fn to_uci(&self, wdl: WdlParams) -> String {
    let mut output = Vec::new();

    if let Some(depth) = self.depth {
      output.push(format!("depth {depth} "));
    }

    if let Some(seldepth) = self.seldepth {
      output.push(format!("seldepth {seldepth} "));
    }

    if let Some(time) = self.time {
      output.push(format!("time {time} "));
    }

    if let Some(nodes) = self.nodes {
      output.push(format!("nodes {nodes} "));
    }

    if let Some(score) = self.score {
      match score {
        Score::Cp(score) => {
          let normalized = Score::Cp(wdl.wdl_normalized(score));

          output.push(format!("score {normalized} "));
        }

        score => output.push(format!("score {score} ")),
      }
    }

    if let Some(currmove) = self.currmove {
      output.push(format!("currmove {currmove} "));
    }

    if let Some(currmovenumber) = self.currmovenumber {
      output.push(format!("currmovenumber {currmovenumber} "));
    }

    if let Some(hashfull) = self.hashfull {
      output.push(format!("hashfull {hashfull} "));
    }

    if let Some(nps) = self.nps {
      output.push(format!("nps {nps} "));
    }

    if self.pv.len() > 0 {
      output.push(format!("pv "));

      for mv in self.pv.iter() {
        output.push(format!("{mv} "));
      }
    }

    output.join(" ")
  }

  /// Format the SearchInfo as a pretty-printed log message, using the
  /// provided WDL parameters to rescale the score such that an advantage of
  /// 100cp corresponds to a 50% chance of winning.
  pub fn to_pretty(&self, board: &Board, wdl: WdlParams) -> String {
    let mut output = String::new();

    if let Some(depth) = self.depth {
      write!(
        output,
        "{} {:<2}",
        "iteration".black(),
        depth.to_string().blue().bold()
      )
      .unwrap();
    }

    if let Some(time) = self.time {
      let time_str = if time < 1000 {
        format!(
          "{:>8} {}",
          time.to_string().bold().bright_black(),
          "ms".black()
        )
      } else if time < 60_000 {
        let seconds = format!("{:>9.2}", time as f32 / 1000.0)
          .bright_black()
          .bold();
        format!("{seconds} {}", "s".black())
      } else {
        let minutes = time / 60_000;
        let seconds = (time % 60_000) / 1000;

        format!(
          "{:>6} {}{:0>2} {}",
          minutes.to_string().bold(),
          "m".black(),
          seconds.to_string().bold(),
          "s".black(),
        )
      };

      write!(output, "{time_str}").unwrap();
    }

    if let Some(nodes) = self.nodes {
      let nodes_str = if nodes < 1000 {
        format!(
          "{:>11} {} ",
          nodes.to_string().bold().bright_black(),
          "n".black()
        )
      } else if nodes < 1_000_000 {
        let kn = format!("{:.2}", nodes as f32 / 1000.0);
        format!("{:>10} {}", kn.bold().bright_black(), "kn".black())
      } else {
        let mn = format!("{:.2}", nodes as f32 / 1_000_000.0);
        format!("{:>10} {}", mn.bold().bright_black(), "Mn".black())
      };

      write!(output, "{nodes_str}").unwrap();
    }

    if let Some(nps) = self.nps {
      let nps_str = if nps < 1000 {
        format!(
          "{:>9} {}",
          nps.to_string().bold().bright_black(),
          "nps".black()
        )
      } else if nps < 1_000_000 {
        let knps = format!("{:.2}", nps as f32 / 1000.0);
        format!("{:>9} {}", knps.bold().bright_black(), "knps".black())
      } else {
        let mnps = format!("{:.2}", nps as f32 / 1_000_000.0);
        format!("{:>9} {}", mnps.bold().bright_black(), "Mnps".black())
      };

      write!(output, "{nps_str}").unwrap();
    }

    if let Some(hashfull) = self.hashfull {
      let percent = hashfull as f32 / 10.0;
      let percent_str = format!("{percent:>8.1}").bright_black().bold();
      write!(output, "{percent_str}{}", "%".black()).unwrap();
    }
    if let Some(score) = self.score {
      match score {
        Score::Cp(score) => {
          let (w, d, l) = wdl.get_wdl(score);
          let wdl_string = format!(
            "({} {} {})",
            format!("W: {:>2}%", w / 10),
            format!("D: {:>2}%", d / 10),
            format!("L: {:>2}%", l / 10),
          );

          write!(output, " {} ", wdl_string.bright_black()).unwrap();

          let normalized = wdl.wdl_normalized(score);
          let pawn = normalized as f32 / 100.0;

          if score < -200 {
            write!(output, "{:>7}", format!("{pawn:+.2}").purple().bold())
              .unwrap();
          } else if score < -10 {
            write!(output, "{:>7}", format!("{pawn:+.2}").red()).unwrap();
          } else if score > 10 {
            write!(output, "{:>7}", format!("{pawn:+.2}").green()).unwrap();
          } else if score > 200 {
            write!(output, "{:>7}", format!("{pawn:+.2}").blue().bold())
              .unwrap();
          } else {
            write!(output, "{:>7}", format!("{pawn:+.2}")).unwrap();
          }
        }

        Score::Mate(n) => {
          if n < 0 {
            write!(output, "{:>7}", format!("M {n}").purple().bold()).unwrap();
          } else {
            write!(output, "{:>7}", format!("M {n}").blue().bold()).unwrap();
          }
        }
      }
    }

    if self.pv.len() > 0 {
      let mut board = board.clone();
      write!(output, "   ").unwrap();

      for (i, &mv) in self.pv.iter().take(10).enumerate() {
        if board.current.is_white() {
          let turn = format!("{}. ", board.full_moves).bright_black();
          write!(output, "{turn}").unwrap();
        } else if i == 0 {
          let turn = format!("{}. ", board.full_moves).bright_black();
          write!(output, "{turn}").unwrap();
          write!(output, "{} ", "...".bright_black().bold().italic()).unwrap();
        }

        write!(output, "{} ", mv.to_san(&board).bold()).unwrap();
        board = board.play_move(mv);
      }

      if self.pv.len() > 10 {
        write!(output, "{}", " ...".bold().bright_black()).unwrap();
      }
    }

    output
  }
}

impl FromStr for SearchInfo {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let mut info = SearchInfo::default();
    let mut parts = s.split_whitespace();

    while let Some(info_type) = parts.next() {
      match info_type {
        "depth" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'depth'."
          ))?;

          info.depth = Some(info_value.parse()?);
        }

        "seldepth" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'seldepth'."
          ))?;

          info.seldepth = Some(info_value.parse()?);
        }

        "time" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'time'."
          ))?;

          info.time = Some(info_value.parse()?);
        }

        "nodes" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'nodes'."
          ))?;

          info.nodes = Some(info_value.parse()?);
        }

        "score" => {
          // 'score cp x'
          parts.next(); // Skip the 'cp' part
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}, failed to parse 'score'"
          ))?;

          info.score = Some(info_value.parse()?);
        }

        "currmove" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'currmove'."
          ))?;

          info.currmove = Some(info_value.parse()?);
        }

        "currmovenumber" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'currmovenumber'."
          ))?;

          info.currmovenumber = Some(info_value.parse()?);
        }

        "hashfull" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'hashfull'."
          ))?;

          info.hashfull = Some(info_value.parse()?);
        }

        "nps" => {
          let info_value = parts.next().ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'nps'."
          ))?;

          info.nps = Some(info_value.parse()?);
        }

        // Just skip anything we don't recognize, and keep stepping
        // forward until we come across another token we recognize
        _ => continue,
      };
    }

    Ok(info)
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Score {
  Cp(i32),
  Mate(i32),
}

impl Display for Score {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Cp(score) => write!(f, "cp {score}"),
      Self::Mate(score) => write!(f, "mate {score}"),
    }
  }
}

impl FromStr for Score {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let mut parts = s.split(" ");

    match parts.next() {
      Some("cp") => {
        let val = parts
          .next()
          .ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'score'"
          ))?
          .parse()?;

        Ok(Self::Cp(val))
      }

      Some("mate") => {
        let val = parts
          .next()
          .ok_or(anyhow!(
            "Not a valid info string: {s}. Failed to parse 'score'"
          ))?
          .parse()?;

        Ok(Self::Mate(val))
      }

      _ => Err(anyhow!(
        "Not a valid info string: {s}. Failed to parse 'score'"
      )),
    }
  }
}
