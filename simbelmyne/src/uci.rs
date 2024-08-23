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
use uci::engine::UciEngineMessage;
use uci::options::OptionType;
use uci::options::UciOption;
use crate::evaluate::pawn_cache::PawnCache;
use crate::evaluate::pretty_print::print_eval;
use crate::history_tables::History;
use crate::search::params::DEFAULT_TT_SIZE;
use chess::perft::perft_divide;
use crate::time_control::TimeController;
use crate::time_control::TimeControlHandle;
use crate::transpositions::TTable;
use crate::position::Position;

const DEBUG: bool = true;

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


/// A wrapper that spins up a search thread and wires up the stdin/stdout of the
/// process to the search thread.
pub struct SearchController {
    position: Position,
    debug: bool,
    tc_handle: Option<TimeControlHandle>,
    search_thread: SearchThread,
}

const UCI_OPTIONS: [UciOption; 2] = [
    UciOption { 
        name: "Hash",
        option_type: OptionType::Spin { 
            min: 4,
            max: 1024,
            default: DEFAULT_TT_SIZE as i32,
            step: 1
        }
    },

    UciOption { 
        name: "Threads",
        option_type: OptionType::Spin { 
            min: 1,
            max: 1,
            default: 1,
            step: 1
        }
    },
];

impl SearchController {
    // Create a new UCI listener
    pub fn new(board: Board) -> Self {
        Self { 
            position: Position::new(board),
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


        for input in stdin.lines() {
            let input = input.unwrap();

            match input.trim().parse::<UciClientMessage>() {
                Ok(command) => {
                    match command {

                        // Print identifying information
                        UciClientMessage::Uci => {
                            println!("id name {NAME} {VERSION}");
                            println!("id author {AUTHOR}");

                            for option in UCI_OPTIONS {
                                println!("option {option}");
                            }

                            #[cfg(feature = "spsa")] {
                                use crate::search::params::SPSA_UCI_OPTIONS;
                                for option in SPSA_UCI_OPTIONS {
                                    println!("option {option}");
                                }
                            }

                            println!("uciok");
                        },

                        UciClientMessage::Show => {
                            println!("{}", self.position.board);
                        },

                        UciClientMessage::Eval => {
                            println!("{}", print_eval(&self.position.board));
                        },

                        // Let the client know we're ready
                        UciClientMessage::IsReady => println!("readyok"),

                        // Reset the search state
                        UciClientMessage::UciNewGame => {
                            self.position = Position::new(Board::default());
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
                            let (tc, tc_handle) = TimeController::new(tc, self.position.board.current);
                            self.tc_handle = Some(tc_handle);

                            self.search_thread.search(self.position.clone(), tc);
                        },

                        UciClientMessage::GoPerft(depth) => {
                            let perft_result = perft_divide(self.position.board, depth);
                            let total: usize = perft_result.iter().map(|(_, nodes)| nodes).sum();

                            for (mv, nodes) in perft_result.iter() {
                                println!("{mv}: {nodes}");
                            }

                            println!("\n{total}");
                        }

                        // Abort the currently running search
                        UciClientMessage::Stop => {
                            if let Some(tc_handle) = &self.tc_handle {
                                tc_handle.stop();
                            }
                        }

                        // Set an option
                        UciClientMessage::SetOption(name, value) => {
                            match name.as_str() {
                                // Advertized options
                                "Hash" => {
                                    let size: usize = value.parse()?;
                                    self.search_thread.resize_tt(size);
                                },

                                // Treat any other options as search params
                                // for SPSA purposes.
                                _ => {
                                    if let Ok(_value) = value.parse::<i32>() {
                                        #[cfg(feature = "spsa")] {
                                            use crate::search::params::set_param;
                                            set_param(&name, _value);
                                        }
                                    } else {
                                        eprintln!("Invalid value {value}");
                                    }
                                }
                            }

                        }

                        UciClientMessage::Quit => { break; },
                    }
                },

                Err(err) => println!("{err}: {input}")
            };

            stdout().flush()?;
        }

        Ok(())
    }
}

/// Commands that can be sent from the UCI listener thread to the SearchThread
enum SearchCommand {
    Search(Position, TimeController),
    Clear,
    ResizeTT(usize),
}

/// A handle to a long-running thread that's in charge of searching for the best
/// move, given a position and time control.
struct SearchThread {
    tx: std::sync::mpsc::Sender<SearchCommand>
}

impl SearchThread {
    /// Spawn a new search thread, and return a handle to it as a SearchThread
    /// struct.
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<SearchCommand>();

        std::thread::spawn(move || {
            let mut tt_size = DEFAULT_TT_SIZE;
            let mut tt = TTable::with_capacity(tt_size);
            let mut pawn_cache = PawnCache::with_capacity(8);
            let mut history = History::new();

            for msg in rx.iter() {
                match msg {
                    SearchCommand::Search(position, mut tc) => {
                        tt.increment_age();

                        // let report = position.search::<DEBUG>(
                        //     &mut tt, 
                        //     &mut pawn_cache,
                        //     &mut history,
                        //     &mut tc, 
                        // );

                    // println!("{}", UciEngineMessage::BestMove(report.pv[0]));
                    },

                    SearchCommand::Clear => {
                        history = History::new();
                        tt = TTable::with_capacity(tt_size);
                    },

                    SearchCommand::ResizeTT(size) => {
                        tt_size = size;
                        tt = TTable::with_capacity(size);
                    }

                }
            }
        });

        Self { tx }
    }

    /// Initiate a new search on this thread
    pub fn search(&self, position: Position, tc: TimeController) {
        self.tx.send(SearchCommand::Search(position, tc)).unwrap();
    }

    /// Clear the history and transposition tables for this search thread
    pub fn clear_tables(&self) {
        self.tx.send(SearchCommand::Clear).unwrap();
    }

    pub fn resize_tt(&self, size: usize) {
        self.tx.send(SearchCommand::ResizeTT(size)).unwrap();
    }

    // pub fn set_search_params(&self, search_params: SearchParams) {
    //     self.tx.send(SearchCommand::SetSearchParams(search_params)).unwrap();
    // }
}
