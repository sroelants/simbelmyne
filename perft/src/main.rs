mod bench;
mod debug;
mod perft;
mod perftree;
mod presets;

use bench::run_bench;
use clap::{Parser, Subcommand};
use debug::run_debug;
use perftree::run_perftree;
use presets::Preset;

pub const BULK: bool = true;

#[derive(Parser)]
#[command(author = "Sam Roelants", version = "0.1", about = "A simple perft tool.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(arg_required_else_help = true)]
    Perftree {
        depth: usize,
        fen: String,
        moves: Vec<String>,
    },

    Bench {
        /// Sets a custom config file
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
            Command::Bench { depth, fen, preset, all } => {
                run_bench(depth, fen, preset, all)
            },

            Command::Perftree { depth, fen, moves } => {
                run_perftree(depth, fen, moves)
            },

            Command::Debug { depth, fen } => run_debug(depth, fen),
        }
    }
}

fn main() -> anyhow::Result<()>{
    let args = Cli::parse();
    args.command.run()?;
    Ok(())
}
