use clap::Subcommand;

use self::{presets::Preset, play::run_play, perft::run_perft, debug::run_debug, bench::run_bench};

pub mod debug;
pub mod bench;
pub mod presets;
pub mod perft;
pub mod play;
pub mod serve;

#[derive(Debug, Subcommand)]
pub enum Command {
    Play {
        ///Start from a FEN string
        #[arg(short, long, default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
        fen: String,

        /// Set the search depth
        #[arg(short, long, value_name = "DEPTH", default_value = "4")]
        depth: usize,
    },

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

    Bench {
        /// Set the search depth
        #[arg(short, long, value_name = "DEPTH", default_value = "6")]
        depth: usize,

        /// One or more FEN strings to run the perf test on
        #[arg(short, long, value_name = "FEN")]
        fen: Option<String>,

    },

    Debug {
        /// The desired search depth, in ply (half-turns)
        #[arg(short, long, default_value = "5")]
        depth: usize,

        #[arg(short, long, default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
        fen: String,
    },
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Play { fen, depth } => run_play(fen, depth)?,
            Command::Perft { depth, fen, preset, all } => run_perft(depth, fen, preset, all)?,
            Command::Bench { depth, fen } => run_bench(depth, fen),
            Command::Debug { depth, fen } => run_debug(depth, fen)?,
        };

        Ok(())
    }
}
