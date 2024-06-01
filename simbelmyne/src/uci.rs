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
use uci::options::OptionType;
use uci::options::UciOption;
use crate::evaluate::pretty_print::print_eval;
use crate::evaluate::Score;
use crate::history_tables::conthist::ContHist;
use crate::history_tables::history::HistoryTable;
use crate::search::params::ASPIRATION_BASE_WINDOW;
use crate::search::params::ASPIRATION_MAX_WINDOW;
use crate::search::params::ASPIRATION_MIN_DEPTH;
use crate::search::params::DEFAULT_TT_SIZE;
use crate::search::params::DELTA_PRUNING_MARGIN;
use crate::search::params::FP_BASE;
use crate::search::params::FP_MARGIN;
use crate::search::params::FP_THRESHOLD;
use crate::search::params::LMP_BASE;
use crate::search::params::LMP_FACTOR;
use crate::search::params::LMP_THRESHOLD;
use crate::search::params::LMR_MIN_DEPTH;
use crate::search::params::LMR_THRESHOLD;
use crate::search::params::NMP_BASE_REDUCTION;
use crate::search::params::NMP_REDUCTION_FACTOR;
use crate::search::params::RFP_MARGIN;
use crate::search::params::RFP_THRESHOLD;
use chess::perft::perft_divide;
use crate::search::params::SearchParams;
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
    search_params: SearchParams,
}

const UCI_OPTIONS: [UciOption; 17] = [
    UciOption { 
        name: "Hash",
        option_type: OptionType::Spin { 
            min: 4,
            max: 1024,
            default: DEFAULT_TT_SIZE as i32
        }
    },

    UciOption { 
        name: "nmp_base_reduction",
        option_type: OptionType::Spin {
            min: 0,
            max: 8,
            default: NMP_BASE_REDUCTION as i32,
        }
    },

    UciOption { 
        name: "nmp_reduction_factor",
        option_type: OptionType::Spin {
            min: 0,
            max: 8,
            default: NMP_REDUCTION_FACTOR as i32, 
        }
    },

    UciOption { 
        name: "aspiration_min_depth",
        option_type: OptionType::Spin {
            min: 1,
            max: 8,
            default: ASPIRATION_MIN_DEPTH as i32,
        }
    },

    UciOption { 
        name: "aspiration_base_window",
        option_type: OptionType::Spin {
            min: 10,
            max: 50, 
            default: ASPIRATION_BASE_WINDOW as i32,
        }
    },
    UciOption { 
        name: "aspiration_max_window",
        option_type: OptionType::Spin {
            min: 500,  
            max: 1300, 
            default: ASPIRATION_MAX_WINDOW as i32,
        } 
    },

    UciOption {
        name: "fp_threshold",
        option_type: OptionType::Spin {
            min: 2,
            max: 12,
            default: FP_THRESHOLD as i32,
        }
    },

    UciOption {
        name: "fp_base",
        option_type: OptionType::Spin {
            min: 0,
            max: 150,
            default: FP_BASE as i32,
        }
    },

    UciOption {
        name: "fp_margin",
        option_type: OptionType::Spin {
            min: 0,
            max: 150,
            default: FP_MARGIN as i32,
        }
    },

    UciOption { 
        name: "rfp_threshold",
        option_type: OptionType::Spin {
            min: 2,
            max: 12,
            default: RFP_THRESHOLD as i32,
        }
    },

    UciOption {
        name: "rfp_margin",
        option_type: OptionType::Spin {
            min: 20,
            max: 140,
            default: RFP_MARGIN
        }
    },

    UciOption { 
        name: "lmp_threshold",
        option_type: OptionType::Spin {
            min: 2, 
            max: 12,
            default: LMP_THRESHOLD as i32,
        }
    },

    UciOption { 
        name: "lmp_base",
        option_type: OptionType::Spin {
            min: 0, 
            max: 5,
            default: LMP_BASE as i32,
        }
    },

    UciOption { 
        name: "lmp_factor",
        option_type: OptionType::Spin {
            min: 0, 
            max: 5,
            default: LMP_FACTOR as i32,
        }
    },

    UciOption { 
        name: "lmr_min_depth", 
        option_type: OptionType::Spin { 
            min: 1,
            max: 5,
            default: LMR_MIN_DEPTH as i32,
        }
    },

    UciOption { 
        name: "lmr_threshold",
        option_type: OptionType::Spin {
            min: 1,
            max: 5,
            default: LMR_THRESHOLD as i32,
        }
    },

    UciOption { 
        name: "delta_pruning_margin",
        option_type: OptionType::Spin {
            min: 100,
            max: 250,
            default: DELTA_PRUNING_MARGIN,
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
            search_params: SearchParams::default(),
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
                            let (tc, tc_handle) = TimeController::new(tc, self.position.board);
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

                                // Internal options, for SPSA tuning
                                "nmp_base_reduction" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.nmp_base_reduction = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "nmp_reduction_factor" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.nmp_reduction_factor = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "aspiration_min_depth" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.aspiration_min_depth = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "aspiration_base_window" => {
                                    let value: Score = value.parse()?;
                                    self.search_params.aspiration_base_window = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "aspiration_max_window" => {
                                    let value: Score = value.parse()?;
                                    self.search_params.aspiration_max_window = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "fp_threshold" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.fp_threshold = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "rfp_threshold" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.rfp_threshold = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "rfp_margin" => {
                                    let value: Score = value.parse()?;
                                    self.search_params.rfp_margin = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "lmp_threshold" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.lmp_threshold = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "lmr_min_depth" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.lmr_min_depth = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "lmr_threshold" => {
                                    let value: usize = value.parse()?;
                                    self.search_params.lmr_threshold = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                "delta_pruning_margin" => {
                                    let value: Score = value.parse()?;
                                    self.search_params.delta_pruning_margin = value;
                                    self.search_thread.set_search_params(self.search_params.clone())
                                },

                                _ => {}
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
    SetSearchParams(SearchParams),
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
            let mut history = HistoryTable::new();
            let mut conthist = ContHist::boxed();
            let mut search_params = SearchParams::default();

            for msg in rx.iter() {
                match msg {
                    SearchCommand::Search(position, mut tc) => {
                        history.age_entries();
                        tt.increment_age();
                        position.search::<DEBUG>(
                            &mut tt, 
                            &mut history, 
                            &mut conthist,
                            &mut tc, 
                            &search_params
                        );
                    },

                    SearchCommand::Clear => {
                        history = HistoryTable::new();
                        tt = TTable::with_capacity(tt_size);
                    },

                    SearchCommand::ResizeTT(size) => {
                        tt_size = size;
                        tt = TTable::with_capacity(size);
                    }

                    SearchCommand::SetSearchParams(params) => {
                        search_params = params;
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

    pub fn set_search_params(&self, search_params: SearchParams) {
        self.tx.send(SearchCommand::SetSearchParams(search_params)).unwrap();
    }
}
