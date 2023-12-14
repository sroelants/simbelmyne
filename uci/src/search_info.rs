use std::fmt::Display;
use std::str::FromStr;
use chess::movegen::moves::Move;
use anyhow::*;

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
    pub score: Option<i32>,

    /// The move we're currently searching
    pub currmove: Option<Move>,

    /// The number of the move we're currently searching
    pub currmovenumber: Option<u8>,

    /// How full is the transposition table, as a value per mille.
    pub hashfull: Option<u32>,

    /// The number of nodes searched per second
    pub nps: Option<u32>,

    /// The current principal variation
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


