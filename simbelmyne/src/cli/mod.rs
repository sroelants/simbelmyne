use std::path::PathBuf;

use clap::Subcommand;
use divide::run_divide;
use crate::spsa::{run_openbench, run_weatherfactory};
use self::{perft::run_perft, bench::run_bench, tune::run_tune};

pub mod bench;
pub mod perft;
pub mod divide;
pub mod tune;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the perft test suite
    Perft {
        /// Set the search depth
        #[arg(short, long, value_name = "DEPTH", default_value = "5")]
        depth: usize,

        /// One or more FEN strings to run the perf test on
        #[arg(short, long, value_name = "FEN")]
        fen: Option<String>,

        #[arg(long)]
        all: bool
    },

    /// Run the perft test suite
    Divide {
        /// Set the search depth
        #[arg(short, long, value_name = "DEPTH", default_value = "5")]
        depth: usize,

        /// One or more FEN strings to run the perf test on
        #[arg(short, long, value_name = "FEN", default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" )]
        fen: String,
    },

    /// Run the bench suite and report the total number of nodes and average nps
    Bench,

    /// Start a tuning run of all the evaluation weights
    Tune {
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,

        #[arg(short, long, value_name = "NUMBER")]
        positions: Option<usize>,

        #[arg(short, long, value_name = "EPOCHS", default_value = "100")]
        epochs: usize,

        /// The file to output the tuned weights to
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// The interval of epochs at which to write the intermediate tuned 
        /// parameters.
        #[arg(short, long, value_name = "ITERATIONS", default_value = "100")]
        interval: usize
    },

    /// Output all tunable UCI options in Openbench's SPSA format
    Openbench,

    /// Output all tunable UCI options in WeatherFactory's SPSA format
    WeatherFactory,
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Perft { depth, fen, all } => run_perft(depth, fen, all)?,
            Command::Divide { fen, depth } => run_divide(fen, depth)?,
            Command::Tune { file, positions, epochs, output, interval } => run_tune(file, positions, epochs, output, interval),
            Command::Bench => run_bench(),
            Command::Openbench => run_openbench(),
            Command::WeatherFactory => run_weatherfactory(), 
        };

        Ok(())
    }
}
