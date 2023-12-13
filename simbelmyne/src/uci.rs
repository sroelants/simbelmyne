use std::io::BufRead;
use std::io::stdout;
use std::io::Write; 
use chess::board::Board;
use crate::time_control::TimeController;
use crate::time_control::TimeControlHandle;
use crate::transpositions::TTable;
use std::sync::mpsc::{channel, Receiver, Sender};
use shared::uci::UciClientMessage;

use crate::position::Position;

const NAME: &str = "Simbelmyne";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Debug)]
pub struct UciListener {
    search_tx: Sender<UciClientMessage>,
}

impl UciListener {
    pub fn new() -> UciListener {
        let (tx, rx) = channel::<UciClientMessage>();
        std::thread::spawn(move || SearchThread::new(rx).run());
        
        UciListener { search_tx: tx }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();

        for input in stdin.lines() {
            let input = input.unwrap();
            match &input.trim().parse::<UciClientMessage>() {
                Ok(command) => {
                    match command {
                        UciClientMessage::Quit => { break; },

                        _ => self.search_tx.send(command.clone())?
                    }
                },

                Err(err) => println!("{err}: {input}")
            };

        stdout().flush()?;
        }

        Ok(())
    }
}

struct SearchThread {
    search_rx: Receiver<UciClientMessage>,
    position: Position,
    debug: bool,
    tc_handle: Option<TimeControlHandle>,
}

impl SearchThread {
    pub fn new(rx: Receiver<UciClientMessage>) -> SearchThread {
        SearchThread { 
            search_rx: rx, 
            position: Position::new(Board::new()),
            debug: false,
            tc_handle: None,
        }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        for cmd in &self.search_rx {
            match cmd {
                UciClientMessage::Uci => {
                 println!("id name {NAME} {VERSION}");
                    stdout().flush()?;
                    println!("id author {AUTHOR}");
                    stdout().flush()?;
                    println!("uciok");
                    stdout().flush()?;
                },

                UciClientMessage::IsReady => println!("readyok"),

                UciClientMessage::UciNewGame => {
                    self.position = Position::new(Board::new());
                    self.tc_handle = None;
                },

                UciClientMessage::Debug(debug) => self.debug = debug,

                UciClientMessage::Position(board, moves) => {
                    self.position = moves.into_iter().fold(
                        Position::new(board),
                        |position, mv| position.play_bare_move(mv)
                    );
                },

                UciClientMessage::Go(tc) => {
                        let side = self.position.board.current;
                    let (tc, tc_handle) = TimeController::new(tc, side);

                    self.tc_handle = Some(tc_handle);

                    let mut tt = TTable::with_capacity(64);
                    let search = self.position.search(&mut tt, tc);
                    let mv = search.pv[0];
                    println!("bestmove {mv}");
                },

                UciClientMessage::Stop => {
                    if let Some(tc_handle) = &self.tc_handle {
                        tc_handle.stop();
                    }
                }

                _ => {}
            }
        }

        Ok(())
    }
}
