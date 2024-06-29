use std::path::PathBuf;

use clap::Subcommand;
use self::{presets::Preset, perft::run_perft, bench::run_bench, tune::run_tune};

pub mod bench;
pub mod presets;
pub mod perft;
pub mod tune;

#[derive(Debug, Subcommand)]
pub enum Command {
    Perft {
        /// Set the search depth
        #[arg(short, long, value_name = "DEPTH", default_value = "5")]
        depth: usize,

        /// One or more FEN strings to run the perf test on
        #[arg(short, long, value_name = "FEN")]
        fen: Option<String>,

        /// The name of a pre-loaded board FEN
        #[arg(
            short,
            long,
            value_name = "PRESET_NAME",
            default_value = "starting-pos"
        )]
        preset: Option<Preset>,

        #[arg(long)]
        all: bool
    },

    Bench,

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
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Perft { depth, fen, preset, all } => run_perft(depth, fen, preset, all)?,
            Command::Tune { file, positions, epochs, output, interval } => run_tune(file, positions, epochs, output, interval),
            Command::Bench => run_bench(),
        };

        Ok(())
    }
}
