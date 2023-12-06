use std::{fmt::Display, str::FromStr, time::Duration};
use anyhow::*;

use chess::{movegen::moves::Move, board::Board};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Info {
    pub depth: Option<u8>,
    pub seldepth: Option<u8>,
    pub time: Option<u64>,
    pub nodes: Option<u32>,
    pub score: Option<i32>,
    pub currmove: Option<Move>,
    pub currmovenumber: Option<u8>,
    pub hashfull: Option<u32>,
    pub nps: Option<u32>,
}

impl Display for Info {
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
            write!(f, "score cp {score} ")?;
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

        std::fmt::Result::Ok(())
    }
}

impl FromStr for Info {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut info = Info::default();
        let mut parts = s.split_whitespace();
        
        while let Some(info_type) = parts.next() {
            match info_type {
                "depth" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'depth'."))?;

                    info.depth = Some(info_value.parse()?);
                },

                "seldepth" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'seldepth'."))?;

                    info.seldepth = Some(info_value.parse()?);
                },

                "time" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'time'."))?;

                    info.time = Some(info_value.parse()?);
                },

                "nodes" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'nodes'."))?;

                    info.nodes = Some(info_value.parse()?);
                },

                "score" => { // 'score cp x'
                    parts.next(); // Skip the 'cp' part
                    let info_value = parts
                        .next()
                        .ok_or(anyhow!("Not a valid info string: {s}, failed to parse 'score'"))?;

                    info.score = Some(info_value.parse()?);
                },

                "currmove" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'currmove'."))?;

                    info.currmove = Some(info_value.parse()?);
                },

                "currmovenumber" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'currmovenumber'."))?;

                    info.currmovenumber = Some(info_value.parse()?);
                },

                "hashfull" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'hashfull'."))?;

                    info.hashfull = Some(info_value.parse()?);
                },

                "nps" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'nps'."))?;

                    info.nps = Some(info_value.parse()?);
                },

                // Just skip anything we don't recognize, and keep stepping
                // forward until we come across another token we recognize
                _ => continue,
            };
        }

        Ok(info)
    }
}


#[derive(Debug, Copy, Clone)]
pub enum TimeControl {
    Depth(usize),
    Nodes(usize),
    Time(Duration),
    Infinite,
}

impl Display for TimeControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TimeControl::*;

        match self {
            Depth(n) => write!(f, "depth {n}"),
            Nodes(n) => write!(f, "nodes {n}"),
            Time(n) => write!(f, "movetime {}", n.as_millis()),
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

        let tc_value = parts.next().ok_or(anyhow!("Invalid time control"))?;
        let tc_value: usize = tc_value.parse()?;

        match tc_type {
            "depth" => Ok(TimeControl::Depth(tc_value)),
            "nodes" => Ok(TimeControl::Nodes(tc_value)),
            "movetime" => Ok(TimeControl::Time(Duration::from_millis(tc_value as u64))),
            _ => Err(anyhow!("Invalid time control"))
        }
    }
}

#[derive(Debug, Clone)]
pub enum UciClientMessage {
    Uci,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    UciNewGame,
    Position(Board),
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
            Position(board) => writeln!(f, "position fen {fen}", fen = board.to_fen()),
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
                parts.next(); // Skip "name"
                let opt = if let Some(opt) = parts.next() {
                    opt
                } else {
                    Err(anyhow!("Invalid UCI message: {msg}"))?
                };

                parts.next(); // Skip "value"
                let value  = if let Some(value) = parts.next() {
                    value
                } else {
                    Err(anyhow!("Invalid UCI message"))?
                };

                Ok(SetOption(opt.to_string(), value.to_string()))
            },

            "ucinewgame" => Ok(UciNewGame),

            "position" => {
                let mut parts = remainder.split(" ");

                let pos_type = parts.next().ok_or(anyhow!("Invalid UCI message {msg}"))?;
                let mut board = if pos_type == "fen" {
                    let fen_parts = (&mut parts).take(6);
                    let fen = fen_parts.collect::<Vec<_>>().join(" ");

                    fen.parse()?
                } else {
                    Board::new()
                };

                if let Some("moves") = parts.next() {
                    for mv in parts {
                        let mv: Move = mv.parse()?;
                        board = board.play_move(mv);
                    }
                }

                Ok(Position(board))
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

#[derive(Debug, Clone)]
pub enum IdType {
    Name(String),
    Author(String)
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
        let  (id_type, id_value) = s.split_once(" ").ok_or(anyhow!("Invalid UCI message"))?;

        match id_type {
            "name" => Ok(IdType::Name(id_value.trim().to_owned())),
            "author" => Ok(IdType::Author(id_value.trim().to_owned())),
            _ => Err(anyhow!("Not a valid id string"))
        }
    }
}

#[derive(Debug, Clone)]
pub enum UciEngineMessage {
    Id(IdType),
    UciOk,
    ReadyOk,
    BestMove(Move),
    Info(Info)
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
            },

            "uciok" => Ok(UciOk),

            "readyok" => Ok(ReadyOk),

            "bestmove" => {
                let mv = remainder.split_whitespace()
                    .next()
                    .ok_or(anyhow!("Invalid UCI message"))?
                    .parse()?;

                Ok(BestMove(mv))
            }

            "info" => {
                let info_vals = remainder.parse().unwrap();
                Ok(Info(info_vals))
            }

            _ => Err(anyhow!("Invalid UCI message"))
        }
    }
}

impl Display for UciEngineMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UciEngineMessage::*;

        match self {
            Id(id_option) => write!(f, "id {id_option}"),
            UciOk => write!(f, "uciok"),
            ReadyOk => write!(f, "readyok"),
            BestMove(mv) => write!(f, "bestmove {mv}"),
            Info(info) => write!(f, "info {info}"),
        }
    }
}
