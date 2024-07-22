use std::fmt::Display;
use std::io::IsTerminal;
use std::str::FromStr;
use chess::movegen::moves::Move;
use anyhow::*;
use colored::Colorize;

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
            let score = wdl.wdl_normalized(score);
            output.push(format!("score {score} "));
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
    pub fn to_pretty(&self, wdl: WdlParams) -> String {
        let mut output = Vec::new();

            if let Some(depth) = self.depth {
                output.push(format!("{}", format!("{depth:>3}").blue()));
            }

            if let Some(seldepth) = self.seldepth {
                output.push(format!("{:<3}", format!("/{seldepth}").bright_black()));
            }

            if let Some(score) = self.score {
                let score = wdl.wdl_normalized(score);

                match score {
                    Score::Cp(score) => {
                        let pawn = score as f32 / 100.0;

                        if score < -200 {
                            output.push(format!("{:>7}", format!("{pawn:+.2}").purple().bold()));
                        } else if score < -10 {
                            output.push(format!("{:>7}", format!("{pawn:+.2}").red()));
                        } else if score > 10 {
                            output.push(format!("{:>7}", format!("{pawn:+.2}").green()));
                        } else if score > 200 {
                            output.push(format!("{:>7}", format!("{pawn:+.2}").blue().bold()));
                        } else {
                            output.push(format!("{:>7}", format!("{pawn:+.2}")));
                        }
                    },

                    Score::Mate(n) => {
                        if n < 0 {
                            output.push(format!("{:>7}", format!("M {n}").purple().bold()));
                        } else {
                            output.push(format!("{:>7}", format!("M {n}").blue().bold()));
                        }
                    }
                }
            }

            if let Some(time) = self.time {
                output.push(format!("{:>8}", format!("{time}ms").bright_black()));
            }

            if let Some(nodes) = self.nodes {
                let kn = nodes as f32 / 1000.0;
                output.push(format!("{:>10}", format!("{kn:.1}kn").bright_black()));
            }

            if let Some(nps) = self.nps {
                let knps = nps / 1000;
                output.push(format!("{:>10}", format!("{knps}knps").bright_black()));
            }

            if let Some(hashfull) = self.hashfull {
                let percent = hashfull as f32 / 10.0;
                output.push(format!("{:>6}", format!("{percent:.1}%").bright_black()));
            }


            if self.pv.len() > 0 {
                output.push(format!(" "));

                for mv in self.pv.iter() {
                    output.push(format!("{} ", format!("{mv}").italic()));
                }
            }

        output.join(" ")
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
                let val = parts.next()
                    .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'score'"))?
                    .parse()?;

                Ok(Self::Cp(val))
            },

            Some("mate") => {
                let val = parts.next()
                    .ok_or(anyhow!("Not a valid info string: {s}. Failed to parse 'score'"))?
                    .parse()?;

                Ok(Self::Mate(val))
            },

            _ => Err(anyhow!("Not a valid info string: {s}. Failed to parse 'score'"))
        }
    }
}
