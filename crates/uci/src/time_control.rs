use anyhow::anyhow;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

/// A time control represents the time constraints placed on the search, whether
/// that's an actual time, a cutoff search depth, or a cutoff node count.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeControl {
  /// Keep going until we get an explicit `stop` signal
  Infinite,

  /// Search up to the requested search depth
  Depth(usize),

  /// Search a requested amount of nodes
  Nodes(usize),

  /// Search for a fixed amount of time
  FixedTime(Duration),

  /// Given a remaining amount of time on the clock, choose your own time
  /// cutoff to maximally optimize that time.
  Clock {
    wtime: Duration,
    btime: Duration,
    winc: Option<Duration>,
    binc: Option<Duration>,
    movestogo: Option<u32>,
  },
}

impl Display for TimeControl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use TimeControl::*;

    match self {
      Depth(n) => write!(f, "depth {n}"),
      Nodes(n) => write!(f, "nodes {n}"),
      FixedTime(n) => write!(f, "movetime {}", n.as_millis()),
      Clock {
        wtime,
        btime,
        winc,
        binc,
        movestogo,
      } => {
        write!(f, "wtime {} btime {}", wtime.as_millis(), btime.as_millis())?;

        if let Some(winc) = winc {
          write!(f, " winc {}", winc.as_millis())?;
        }

        if let Some(binc) = binc {
          write!(f, " binc {}", binc.as_millis())?;
        }

        if let Some(movestogo) = movestogo {
          write!(f, " movestogo {}", movestogo)?;
        }

        std::fmt::Result::Ok(())
      }
      Infinite => write!(f, "infinite"),
    }
  }
}

impl FromStr for TimeControl {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let mut parts = s.split(" ");
    let tc_type = parts.next().ok_or(anyhow!("Invalid time control"))?;

    if tc_type == "infinite" {
      return Ok(TimeControl::Infinite);
    }

    match tc_type {
      "depth" => {
        let value = parts
          .next()
          .ok_or(anyhow!("Invalid time control: {s}"))?
          .parse()?;

        Ok(TimeControl::Depth(value))
      }
      "nodes" => {
        let value = parts
          .next()
          .ok_or(anyhow!("Invalid time control: {s}"))?
          .parse()?;

        Ok(TimeControl::Nodes(value))
      }

      "movetime" => {
        let value: u64 = parts
          .next()
          .ok_or(anyhow!("Invalid time control: {s}"))?
          .parse()?;

        Ok(TimeControl::FixedTime(Duration::from_millis(value)))
      }

      "wtime" => {
        let wtime: u64 = parts
          .next()
          .ok_or(anyhow!("Invalid time control: {s}"))?
          .parse()
          .unwrap_or(100);

        let _ = parts.next();

        let btime: u64 = parts
          .next()
          .ok_or(anyhow!("Invalid time control: {s}"))?
          .parse()
          .unwrap_or(100);

        let mut winc = None;
        let mut binc = None;
        let mut movestogo = None;

        loop {
          let command = parts.next();
          if command.is_none() {
            break;
          }

          match command.unwrap() {
            "winc" => {
              let value: u64 = parts
                .next()
                .ok_or(anyhow!("Invalid time control: {s}"))?
                .parse()?;

              winc = Some(value);
            }

            "binc" => {
              let value: u64 = parts
                .next()
                .ok_or(anyhow!("Invalid time control: {s}"))?
                .parse()?;

              binc = Some(value);
            }

            "movestogo" => {
              let value: u32 = parts
                .next()
                .ok_or(anyhow!("Invalid time control: {s}"))?
                .parse()?;

              movestogo = Some(value);
            }

            _ => Err(anyhow!("Invalid time control: {s}"))?,
          }

          break;
        }

        Ok(TimeControl::Clock {
          wtime: Duration::from_millis(wtime),
          btime: Duration::from_millis(btime),
          winc: winc.map(Duration::from_millis),
          binc: binc.map(Duration::from_millis),
          movestogo,
        })
      }

      _ => Err(anyhow!("Invalid time control {s}")),
    }
  }
}
