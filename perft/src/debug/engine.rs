
use std::{collections::BTreeMap, process::{Child, ChildStdin, ChildStdout, Command, Stdio}, io::{BufReader, self, BufRead}};

use std::io::Write;
use chess::board::Board;

use crate::perft::perft_divide;


type Perft = BTreeMap<String, usize>;


pub trait Engine {
    fn perft(&mut self, board: Board, depth: usize) -> anyhow::Result<Perft>;
}

pub struct Simbelmyne {}

impl Engine for Simbelmyne {
    fn perft(&mut self, board: Board, depth: usize) -> anyhow::Result<Perft> {
        let move_list: Perft = perft_divide::<true>(board, depth)
            .into_iter()
            .map(|(mv, nodes)| (mv.to_string(), nodes))
            .collect();

        Ok(move_list)
    }
}

pub struct Stockfish {
    child: Child,
    inp: BufReader<ChildStdout>,
    out: ChildStdin,
}

impl Stockfish {
    pub fn new() -> io::Result<Stockfish> {
        let mut child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut inp = BufReader::new(child.stdout.take().expect("stdout not captured"));
        // consume/skip header
        let mut buf = String::new();
        inp.read_line(&mut buf)?;

        let out = child.stdin.take().expect("stdin not captured");

        Ok(Stockfish { child, inp, out })
    }
}

impl Engine for Stockfish {
    fn perft(&mut self, board: Board, depth: usize) -> anyhow::Result<Perft> {
        // send command to stockfish
        write!(self.out, "position fen {}", board.fen())?;

        write!(self.out, "\ngo perft {}\n", depth)?;

        let mut buf = String::new();

        // parse child counts
        let mut child_counts = BTreeMap::new();
        loop {
            buf.clear();
            self.inp.read_line(&mut buf)?;
            if buf.trim().is_empty() {
                break;
            }
            let mut parts = buf.trim().split(": ");
            let move_ = parts
                .next()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "unexpected end of line")
                })?
                .to_string();
            let count = parts
                .next()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "unexpected end of line")
                })?
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            let move_ = move_.parse()?;
            child_counts.insert(move_, count);
        }

        // throw away remaining lines
        buf.clear();
        self.inp.read_line(&mut buf)?;
        buf.clear();
        self.inp.read_line(&mut buf)?;

        Ok(child_counts)
    }
}

impl Drop for Stockfish {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
