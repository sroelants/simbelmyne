use crate::options::UciOption;
use crate::search_info::SearchInfo;
use anyhow::anyhow;
use chess::movegen::moves::Move;
use colored::Colorize;
use std::fmt::Display;
use std::io::IsTerminal;
use std::str::FromStr;

/// Messages that can be sent from the engine back to the client
#[derive(Debug, Clone)]
pub enum UciEngineMessage {
  Id(IdType),
  UciOk,
  ReadyOk,
  BestMove(Move),
  Info(SearchInfo),
  UciOption(UciOption),
}

impl FromStr for UciEngineMessage {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    use UciEngineMessage::*;
    let s = s.trim();
    let (msg, remainder) = s.split_once(" ").unwrap_or((s, ""));

    match msg {
      "id" => {
        let id_val = remainder.parse()?;
        Ok(Id(id_val))
      }

      "uciok" => Ok(UciOk),

      "readyok" => Ok(ReadyOk),

      "bestmove" => {
        let mv = remainder
          .split_whitespace()
          .next()
          .ok_or(anyhow!("Invalid UCI message"))?
          .parse()?;

        Ok(BestMove(mv))
      }

      "info" => {
        let info_vals = remainder.parse().unwrap();
        Ok(Info(info_vals))
      }

      _ => Err(anyhow!("Invalid UCI message")),
    }
  }
}

impl Display for UciEngineMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use UciEngineMessage::*;

    if std::io::stdout().is_terminal() {
      match self {
        Id(id_option) => {
          write!(f, "{}", format!("id {id_option}").bright_black())
        }
        UciOk => write!(f, "{}", "uciok".bright_black()),
        ReadyOk => write!(f, "{}", "readyok".bright_black()),
        BestMove(mv) => write!(
          f,
          "{} {}",
          "bestmove".bright_black(),
          format!("{mv}").italic()
        ),
        Info(info) => write!(f, "{}", info),
        UciOption(option) => {
          write!(f, "{} {}", "option".bright_black(), option)
        }
      }
    } else {
      match self {
        Id(id_option) => write!(f, "id {id_option}"),
        UciOk => write!(f, "uciok"),
        ReadyOk => write!(f, "readyok"),
        BestMove(mv) => write!(f, "bestmove {mv}"),
        Info(info) => write!(f, "info {info}"),
        UciOption(option) => write!(f, "option {option}"),
      }
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
//
// ID Information
//
////////////////////////////////////////////////////////////////////////////////

/// A type of identifying information to output to the user
#[derive(Debug, Clone)]
pub enum IdType {
  /// The name and version of the engine
  Name(String),

  /// THe author of the engine
  Author(String),
}

impl Display for IdType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use IdType::*;

    match self {
      Name(name) => write!(f, "name {name}"),
      Author(author) => write!(f, "author {author}"),
    }
  }
}

impl FromStr for IdType {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Self> {
    let (id_type, id_value) =
      s.split_once(" ").ok_or(anyhow!("Invalid UCI message"))?;

    match id_type {
      "name" => Ok(IdType::Name(id_value.trim().to_owned())),
      "author" => Ok(IdType::Author(id_value.trim().to_owned())),
      _ => Err(anyhow!("Not a valid id string")),
    }
  }
}
