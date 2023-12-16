use clap::Subcommand;
use self::{presets::Preset, perft::run_perft, bench::run_bench};

pub mod bench;
pub mod presets;
pub mod perft;

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

    Bench {
        /// Set the search depth
        #[arg(short, long, value_name = "DEPTH", default_value = "6")]
        depth: usize,

        /// One or more FEN strings to run the perf test on
        #[arg(short, long, value_name = "FEN")]
        fen: Option<String>,

    },
}

impl Command {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Command::Perft { depth, fen, preset, all } => run_perft(depth, fen, preset, all)?,
            Command::Bench { depth, fen } => run_bench(depth, fen),
        };

        Ok(())
    }
}
