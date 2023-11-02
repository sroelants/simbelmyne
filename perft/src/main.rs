mod perft;
mod bench;
mod perftree;

use bench::{Preset, run_bench};
use clap::{Parser, Subcommand};
use perftree::run_perftree;


pub const BULK: bool = true;

#[derive(Parser)]
#[command(author = "Sam Roelants", version = "0.1", about = "A simple perft tool.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command
}


#[derive(Debug, Subcommand)]
enum Command {
    #[command(arg_required_else_help = true)]
    Perftree {
        depth: usize,
        fen: String,
        moves: Vec<String>
    },

    Bench {
        /// Sets a custom config file
        #[arg(short, long, value_name = "DEPTH", default_value = "5")]
        depth: usize,

        /// One or more FEN strings to run the perf test on
        #[arg(short, long, value_name = "FEN")]
        fen: Option<String>,

        /// The name of a pre-loaded board FEN
        #[arg(short, long, value_name = "PRESET_NAME", default_value="starting-pos")]
        preset: Option<Preset>,
    }
}

impl Command {
    pub fn run(self) -> anyhow::Result<()>{
        match self {
            Command::Bench { depth, fen, preset } => {
                run_bench(depth, fen, preset)
            },
            Command::Perftree { depth, fen, moves } => {
                run_perftree(depth, fen, moves) 
            },
        }
    }
}



fn main() {
    let args = Cli::parse();
    let result = args.command.run();

    if result.is_err() {
        std::process::exit(1);
    }
}
