use std::fmt::Display;
use std::str::FromStr;
use chess::board::Board;
use chess::movegen::moves::BareMove;
use crate::time_control::TimeControl;
use anyhow::anyhow;

/// Messages that can be sent from the client to the engine
#[derive(Debug, Clone)]
pub enum UciClientMessage { Uci,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    UciNewGame,
    Position(Board, Vec<BareMove>),
    Go(TimeControl),
    Stop,
    Quit,
}

impl Display for UciClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UciClientMessage::*;

        match self {
            Uci => writeln!(f, "uci"),
            Debug(flag) => writeln!(f, "debug {}", if *flag { "on" } else { "off" }),
            IsReady => writeln!(f, "isready"),
            SetOption(opt, val) => writeln!(f, "setoption name {opt} value {val}"),
            UciNewGame => writeln!(f, "ucinewgame"),
            Position(board, moves) => {
                write!(f, "position fen {fen}", fen = board.to_fen())?;

                if !moves.is_empty() {
                    write!(f, " moves")?;
                    for mv in moves {
                        write!(f, " {mv}")?;
                    }
                }

                std::fmt::Result::Ok(())
            },
            Go(tc) => writeln!(f, "go {tc}"),
            Stop => writeln!(f, "stop"),
            Quit => writeln!(f, "quit"),
        }
    }
}

impl FromStr for UciClientMessage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        use UciClientMessage::*;
        let s = s.trim();
        let (msg, remainder) = s.split_once(" ").unwrap_or((s, ""));

        match msg {
            "uci" => Ok(Uci),

            "isready" => Ok(IsReady),

            "debug" => {
                if let Some(flag) = remainder.split_whitespace().next() {
                    let flag = if flag == "on" { true } else { false };
                    Ok(Debug(flag))
                } else {
                    Err(anyhow!("Invalid UCI message: {msg}"))?
                }
            },

            "setoption" => {
                let mut parts = remainder.split_whitespace();
                assert_eq!(parts.next(), Some("name"), "Invalidly formed UCI command");
                
                let name = parts
                    .by_ref()
                    .take_while(|&word| word != "value")
                    .collect::<String>();

                let value  = if let Some(value) = parts.next() {
                    value
                } else {
                    Err(anyhow!("Invalid UCI message"))?
                };

                Ok(SetOption(name.to_string(), value.to_string()))
            },

            "ucinewgame" => Ok(UciNewGame),

            "position" => {
                let mut parts = remainder.split(" ");

                let pos_type = parts.next().ok_or(anyhow!("Invalid UCI message {msg}"))?;

                let board = if pos_type == "fen" {
                    let fen_parts = (&mut parts).take(6);
                    let fen = fen_parts.collect::<Vec<_>>().join(" ");

                    fen.parse()?
                } else {
                    Board::new()
                };

                let mut moves = Vec::new();

                if let Some("moves") = parts.next() {
                    for mv in parts {
                        let mv: BareMove = mv.parse()?;
                        moves.push(mv);
                    }
                }

                Ok(Position(board, moves))
            },

            "go" => {
                let tc = remainder.parse()?;
                Ok(Go(tc))
            },

            "stop" => Ok(Stop),
            "quit" => Ok(Quit),

            _ => Err(anyhow!("Invalid UCI message: {msg}"))
        }
    }
}

