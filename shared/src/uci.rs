use std::{fmt::Display, str::FromStr, time::Duration};
use anyhow::*;

use chess::{movegen::moves::{Move, BareMove}, board::Board};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SearchInfo {
    pub depth: Option<u8>,
    pub seldepth: Option<u8>,
    pub time: Option<u64>,
    pub nodes: Option<u32>,
    pub score: Option<i32>,
    pub currmove: Option<Move>,
    pub currmovenumber: Option<u8>,
    pub hashfull: Option<u32>,
    pub nps: Option<u32>,
    pub pv: Vec<Move>,
}

impl Display for SearchInfo {
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

        if self.pv.len() > 0 {
            write!(f, "pv ")?;
            for mv in self.pv.iter() {
                write!(f, "{mv} ")?;
            }
        }

        std::fmt::Result::Ok(())
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TCType {
    Infinite,
    Depth(usize),
    Nodes(usize),
    FixedTime(Duration),
    VariableTime { 
        wtime: Duration, 
        btime: Duration, 
        winc: Option<Duration>, 
        binc: Option<Duration>, 
        movestogo: Option<u32> 
    }
}

impl Display for TCType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TCType::*;

        match self {
            Depth(n) => write!(f, "depth {n}"),
            Nodes(n) => write!(f, "nodes {n}"),
            FixedTime(n) => write!(f, "movetime {}", n.as_millis()),
            VariableTime { wtime, btime, winc, binc, movestogo} => {
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

impl FromStr for TCType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut parts = s.split(" ");
        let tc_type = parts.next().ok_or(anyhow!("Invalid time control"))?;

        if tc_type == "infinite" {
            return Ok(TCType::Infinite);
        }

        match tc_type {
            "depth" => {
                let value = parts.next()
                    .ok_or(anyhow!("Invalid time control: {s}"))?
                    .parse()?;

                Ok(TCType::Depth(value))
            },
            "nodes" => {
                let value = parts.next()
                    .ok_or(anyhow!("Invalid time control: {s}"))?
                    .parse()?;

                Ok(TCType::Nodes(value))
            },

            "movetime" => {
                let value: u64 = parts.next()
                    .ok_or(anyhow!("Invalid time control: {s}"))?
                    .parse()?;

                Ok(TCType::FixedTime(Duration::from_millis(value)))
            },

            "wtime" => {
                let wtime: u64 = parts.next()
                    .ok_or(anyhow!("Invalid time control: {s}"))?
                    .parse()?;

                let _ = parts.next();

                let btime: u64 = parts.next()
                    .ok_or(anyhow!("Invalid time control: {s}"))?
                    .parse()?;

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
                            let value: u64 = parts.next()
                                .ok_or(anyhow!("Invalid time control: {s}"))?
                                .parse()?;

                            winc = Some(value);
                        },

                        "binc" => {
                            let value: u64 = parts.next()
                                .ok_or(anyhow!("Invalid time control: {s}"))?
                                .parse()?;

                            binc = Some(value);
                        },

                        "movestogo" => {
                            let value: u32 = parts.next()
                                .ok_or(anyhow!("Invalid time control: {s}"))?
                                .parse()?;

                            movestogo = Some(value);
                        },

                        _ => Err(anyhow!("Invalid time control: {s}"))?
                    }

                    break;
                }

                Ok(TCType::VariableTime {
                    wtime: Duration::from_millis(wtime),
                    btime: Duration::from_millis(btime),
                    winc: winc.map(Duration::from_millis),
                    binc: binc.map(Duration::from_millis),
                    movestogo,
                })
            },

            _ => Err(anyhow!("Invalid time control {s}"))
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
    Position(Board, Vec<BareMove>),
    Go(TCType),
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
    Info(SearchInfo)
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
