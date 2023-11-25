use std::{fmt::Display, str::FromStr};
use anyhow::*;

use chess::movegen::moves::Move;

#[derive(Default)]
struct Info {
    depth: Option<u8>,
    seldepth: Option<u8>,
    time: Option<u32>,
    nodes: Option<u32>,
    score: Option<i32>,
    currmove: Option<Move>,
    currmovenumber: Option<u8>,
    hashfull: Option<u32>,
    nps: Option<u32>,
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

        write!(f, "\n")
    }
}

impl FromStr for Info {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut info = Info::default();
        let mut parts = s.split(" ");
        
        while let Some(info_type) = parts.next() {
            match info_type {
                "depth" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.depth = Some(info_value.parse()?)
                },

                "seldepth" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.seldepth = Some(info_value.parse()?)
                },

                "time" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.time = Some(info_value.parse()?)
                },

                "nodes" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.nodes = Some(info_value.parse()?)
                },

                "score" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.score = Some(info_value.parse()?)
                },

                "currmove" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.currmove = Some(info_value.parse()?)
                },

                "currmovenumber" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.currmovenumber = Some(info_value.parse()?)
                },

                "hashfull" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.hashfull = Some(info_value.parse()?)
                },

                "nps" => {
                    let info_value = parts.next()
                        .ok_or(anyhow!("Not a valid info string"))?;

                    info.nps = Some(info_value.parse()?)
                },

                // Just skip anything we don't recognize, and keep stepping
                // forward until we come across another token we recognize
                _ => continue
            };
        }

        Ok(info)
    }
}


enum TimeControl {
    Depth(usize),
    Nodes(usize),
    Time(usize),
    Infinite,
}

impl Display for TimeControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TimeControl::*;

        match self {
            Depth(n) => write!(f, "depth {n}"),
            Nodes(n) => write!(f, "nodes {n}"),
            Time(n) => write!(f, "movetime {n}"),
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
            "movetime" => Ok(TimeControl::Time(tc_value)),
            _ => Err(anyhow!("Invalid time control"))
        }
    }
}

enum UciClientMessage {
    Uci,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    UciNewGame,
    Position(String),
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
            Position(pos) => writeln!(f, "position {pos}"),
            Go(tc) => writeln!(f, "go {tc}"),
            Stop => writeln!(f, "stop"),
            Quit => writeln!(f, "quit"),
        }
    }
}

enum IdOption {
    Name(String),
    Author(String)
}

impl FromStr for IdOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut parts = s.split(" ");
        let id_type = parts.next().ok_or(anyhow!("Not a valid id string"))?;
        let id_value = parts.next().ok_or(anyhow!("Not a valid id string"))?;

        match id_type {
            "name" => Ok(IdOption::Name(id_value.to_owned())),
            "author" => Ok(IdOption::Author(id_value.to_owned())),
            _ => Err(anyhow!("Not a valid id string"))
        }
    }
}

enum UciEngineMessage {
    Id(IdOption),
    UciOk,
    ReadyOk,
    BestMove(Move),
    Info(Info)
}

impl FromStr for UciEngineMessage {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        use UciEngineMessage::*;
        let mut parts = s.split(" ");
        let msg = parts.next().ok_or(anyhow!("Invalid UCI message"))?;

        match msg {
            "id" => {
                let id_opt = parts.next().ok_or(anyhow!("Invalid UCI message"))?;
                Ok(Id(id_opt.parse()?))
            },

            "uciok" => Ok(UciOk),

            "readyok" => Ok(ReadyOk),

            "bestmove" => {
                let mv = parts.next().ok_or(anyhow!("Invalid UCI message"))?.parse()?;
                Ok(BestMove(mv))
            }

            "info" => {
                let info = parts.next().ok_or(anyhow!("Invalid UCI message"))?.parse()?;
                Ok(Info(info))
            }

            _ => Err(anyhow!("Invalid UCI message"))
        }
    }
}
