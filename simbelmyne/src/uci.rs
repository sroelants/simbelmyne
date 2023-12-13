//! Simbelmyne's UCI interface.
//!
//! Utilities for creating a UCI "listener" that spins up a search thread 
//! and communicates with it over a channel.
//!
//! Uses the UCI types and definitions defined in `shared::uci`.
//!
//! Only the basic UCI commands needed for typical play are supported, no 
//! extra features (hash table size, etc...) just yet.

use std::io::BufRead;
use std::io::stdout;
use std::io::Write; 
use std::sync::mpsc::{channel, Receiver, Sender};
use chess::board::Board;
use shared::uci::UciClientMessage;
use crate::time_control::TimeController;
use crate::time_control::TimeControlHandle;
use crate::transpositions::TTable;
use crate::position::Position;

// Engine information, printed on `uci`

const NAME: &str = "Simbelmyne";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// A wrapper that spins up a search thread and wires up the stdin/stdout of the
/// process to the search thread.
#[derive(Debug)]
pub struct UciController {
    search_tx: Sender<UciClientMessage>,
}

impl UciController {
    // Create a new UCI listener
    pub fn new() -> Self {
        let (tx, rx) = channel::<UciClientMessage>();
        std::thread::spawn(move || SearchThread::new(rx).run());
        
        Self { search_tx: tx }
    }

    /// Start listening on stdin and transmit any valid UCI messages to the
    /// search thread
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

/// A search thread listens for UCI messages over a channel from it's 
/// parent controller.
struct SearchThread {
    search_rx: Receiver<UciClientMessage>,
    position: Position,
    debug: bool,
    tc_handle: Option<TimeControlHandle>,
}

impl SearchThread {
    /// Create a new search thread
    pub fn new(rx: Receiver<UciClientMessage>) -> Self {
        Self { 
            search_rx: rx, 
            position: Position::new(Board::new()),
            debug: false,
            tc_handle: None,
        }
    }

    /// Start listening for commands on the channel, and respond accordingly
    pub fn run(&mut self) -> anyhow::Result<()> {
        for cmd in &self.search_rx {
            match cmd {
                // Print identifying information
                UciClientMessage::Uci => {
                    println!("id name {NAME} {VERSION}");
                    stdout().flush()?;
                    println!("id author {AUTHOR}");
                    stdout().flush()?;
                    println!("uciok");
                    stdout().flush()?;
                },

                // Let the client know we're ready
                UciClientMessage::IsReady => println!("readyok"),

                // Reset the search state
                UciClientMessage::UciNewGame => {
                    self.position = Position::new(Board::new());
                    self.tc_handle = None;
                },

                // Print additional debug information
                UciClientMessage::Debug(debug) => self.debug = debug,


                // Set up the provided position by applying the moves to the
                // provided board state.
                UciClientMessage::Position(board, moves) => {
                    let mut position = Position::new(board);

                    for mv in moves {
                        self.position = position.play_bare_move(mv);
                    }

                    self.position = position;
                },

                // Start a search on the current board position, with the 
                // requested time control
                UciClientMessage::Go(tc) => {
                    let side = self.position.board.current;
                    let (tc, tc_handle) = TimeController::new(tc, side);

                    self.tc_handle = Some(tc_handle);

                    let mut tt = TTable::with_capacity(64);
                    let search = self.position.search(&mut tt, tc);
                    let mv = search.pv[0];
                    println!("bestmove {mv}");
                },

                _ => {}
            }
        }

        Ok(())
    }
}
