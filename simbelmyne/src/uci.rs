use std::str::FromStr;
use std::io::BufRead;
use std::io::stdout;
use std::io::Write; 
use anyhow::anyhow;
use chess::{board::{Board, Square}, movegen::moves::{MoveType, Move}};
use std::sync::mpsc::{channel, Receiver, Sender};

const NAME: &str = "Simbelmyne";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
pub enum UciCommand {
    Uci,
    Debug(bool),
    IsReady,
    SetOption(String, String),
    UciNewGame,
    Position(String), // TODO: Accept both FEN and startpos+moves
    Go, // Don't bother with any go options for now
    Stop,
    PonderHit,
    Quit,
}

impl FromStr for UciCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chunks = s
            .split(' ')
            .map(|s| s.trim());
        

        match chunks.next() {
            Some("uci") => Ok(UciCommand::Uci),

            Some("debug") => {
                match chunks.next() {
                    Some("on") => Ok(UciCommand::Debug(true)),
                    Some("off") => Ok(UciCommand::Debug(false)),
                    _ => Err(anyhow!("Not a valid UCI command!")),
                }
            },

            Some("setoption") => Ok(UciCommand::SetOption(
                String::from("hello"), 
                String::from("World")
            )),

            Some("isready") => Ok(UciCommand::IsReady),

            Some("ucinewgame") => Ok(UciCommand::UciNewGame),

            Some("position") => {
                match chunks.next() {
                    Some("fen") => {
                        let fen_str = chunks.next()
                            .ok_or(anyhow!("Not a valid UCI command!"))?;
                        Ok(UciCommand::Position(fen_str.to_string()))
                    },
                    _ => Err(anyhow!("Not a valid UCI command!")),
                }
            }

            Some("go") => Ok(UciCommand::Go),

            Some("stop") => Ok(UciCommand::Stop),

            Some("ponderhip") => Ok(UciCommand::PonderHit),

            Some("quit") => Ok(UciCommand::Quit),

            _ => Err(anyhow!("Not a valid UCI command!"))
        }
    }
}

#[derive(Debug)]
pub struct UciListener {
    search_tx: Sender<UciCommand>,
}


impl UciListener {
    pub fn new() -> UciListener {
        let (tx, rx) = channel::<UciCommand>();
        std::thread::spawn(move || SearchThread::new(rx).run());
        
        UciListener { search_tx: tx }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();

        for input in stdin.lines() {
            match input?.parse::<UciCommand>() {
                Ok(command) => {
                    match command {
                        UciCommand::Uci => {
                         println!("id name {NAME} {VERSION}");
                            stdout().flush()?;
                            println!("id author {AUTHOR}");
                            stdout().flush()?;
                            println!("uciok");
                            stdout().flush()?;
                        },

                        UciCommand::IsReady => println!("readyok"),

                        UciCommand::Quit => { break; },

                        _ => self.search_tx.send(command)?
                    }
                },

                Err(err) => eprintln!("{err}")
            };

        stdout().flush()?;
        }

        Ok(())
    }
}

struct SearchThread {
    search_rx: Receiver<UciCommand>,
    board: Board,
    debug: bool
}

impl SearchThread {
    pub fn new(rx: Receiver<UciCommand>) -> SearchThread {
        SearchThread { 
            search_rx: rx, 
            board: Board::new(),
            debug: false,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        for cmd in &self.search_rx {
            match cmd {
                UciCommand::UciNewGame => {
                    self.board = Board::new();
                },

                UciCommand::Debug(debug) => self.debug = debug,

                UciCommand::Stop | UciCommand::Go => {
                    let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
                    println!("bestmove {mv}");
                },

                _ => {}
            }
        }

        Ok(())
    }
}
