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
use chess::board::Board;
use colored::Colorize;
use uci::client::UciClientMessage;
use crate::search_tables::HistoryTable;
use crate::time_control::TimeController;
use crate::time_control::TimeControlHandle;
use crate::transpositions::TTable;
use crate::position::Position;

const BANNER: &str = r"
 ,-.          .       .                  
(   ` o       |       |                  
 `-.  . ;-.-. |-. ,-. | ;-.-. . . ;-. ,-.
.   ) | | | | | | |-' | | | | | | | | |-'
 `-'  ' ' ' ' `-' `-' ' ' ' ' `-| ' ' `-'
                              `-'        ";

const NAME: &str = "Simbelmyne";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const WEBSITE: &str = "https://www.samroelants.com";
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const PROMPT: &str = "> ";


/// A wrapper that spins up a search thread and wires up the stdin/stdout of the
/// process to the search thread.
pub struct SearchController {
    position: Position,
    debug: bool,
    tc_handle: Option<TimeControlHandle>,
    search_thread: SearchThread,
}

impl SearchController {
    // Create a new UCI listener
    pub fn new() -> Self {
        Self { 
            position: Position::new(Board::new()),
            debug: false,
            tc_handle: None,
            search_thread: SearchThread::new(),
        }
    }

    /// Start listening on stdin and transmit any valid UCI messages to the
    /// search thread
    pub fn run(&mut self) -> anyhow::Result<()> {
        let stdin = std::io::stdin().lock();

        eprintln!("{}", BANNER.blue());
        eprintln!("                            {} {}", "Version".blue(), VERSION.blue());
        eprintln!("");
        eprintln!("{}: {NAME} {VERSION}", "Engine".blue()); 
        eprintln!("{}: {AUTHOR}", "Author".blue()); 
        eprintln!("{}: {WEBSITE}", "Website".blue()); 
        eprintln!("{}: {REPOSITORY}", "Source".blue());
        eprintln!("");
        eprint!("{PROMPT}");


        for input in stdin.lines() {
            let input = input.unwrap();

            match input.trim().parse::<UciClientMessage>() {
                Ok(command) => {
                    match command {

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
                            self.search_thread.clear_tables();
                        },

                        // Print additional debug information
                        UciClientMessage::Debug(debug) => self.debug = debug,


                        // Set up the provided position by applying the moves to the
                        // provided board state.
                        UciClientMessage::Position(board, moves) => {
                            let mut position = Position::new(board);

                            for mv in moves {
                                position = position.play_bare_move(mv);
                            }

                            self.position = position;
                        },

                        // Start a search on the current board position, with the 
                        // requested time control
                        UciClientMessage::Go(tc) => {
                            let side = self.position.board.current;
                            let (tc, tc_handle) = TimeController::new(tc, side);
                            self.tc_handle = Some(tc_handle);

                            self.search_thread.search(self.position.clone(), tc);
                        },

                        // Abort the currently running search
                        UciClientMessage::Stop => {
                            if let Some(tc_handle) = &self.tc_handle {
                                tc_handle.stop();
                            }
                        }

                        UciClientMessage::Quit => { break; },

                        _ => {}
                    }
                },

                Err(err) => println!("{err}: {input}")
            };

            eprint!("\n{PROMPT}");
            stdout().flush()?;
        }

        Ok(())
    }
}

enum SearchCommand {
    Search(Position, TimeController),
    Clear,
}

struct SearchThread {
    tx: std::sync::mpsc::Sender<SearchCommand>
}

impl SearchThread {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<SearchCommand>();

        std::thread::spawn(move || {
            let mut tt = TTable::with_capacity(64);
            let mut history = HistoryTable::new();

            for msg in rx.iter() {
                match msg {
                    SearchCommand::Search(position, tc) => {
                        history.age_entries();
                        tt.increment_age();

                        position.search(&mut tt, &mut history, tc);
                    },

                    SearchCommand::Clear => {
                        history = HistoryTable::new();
                        tt = TTable::with_capacity(64);
                    }
                }
            }
        });

        Self { tx }
    }

    pub fn search(&self, position: Position, tc: TimeController) {
        self.tx.send(SearchCommand::Search(position, tc)).unwrap();
    }

    pub fn clear_tables(&self) {
        self.tx.send(SearchCommand::Clear).unwrap();
    }
}
