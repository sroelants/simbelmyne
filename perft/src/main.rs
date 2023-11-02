use std::str::FromStr;
mod perft;

use chess::{board::Board, movegen::moves::Move};
use clap::{Parser, Subcommand};
use perft::{run_perft, perft_divide};
use colored::*;


const BULK: bool = true;

#[derive(Parser)]
#[command(author = "Sam Roelants", version = "0.1", about = "A simple perft tool.", long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "DEPTH", default_value = "5")]
    depth: usize,

    /// One or more FEN strings to run the perf test on
    #[arg(short, long, value_name = "FEN")]
    fen: Option<String>,

    /// The name of a pre-loaded board FEN
    #[arg(short, long, value_name = "PRESET_NAME", default_value="starting-pos")]
    preset: Option<Preset>,

    #[command(subcommand)]
    command: Commands
}


#[derive(Debug, Subcommand)]
enum Commands {
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

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
enum Preset {
    StartingPos,
    Kiwipete,
}

impl Preset {
    const COUNT: usize = 2;
    const PRESETS: [PerftPreset<'static>; Preset::COUNT] = [
        PerftPreset {
            name: "Starting position",
            description: "All the pieces in their original position",
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            expected: &[1, 20, 400, 8902, 197_281, 4_865_609, 119_060_24],
        },

        PerftPreset {
            name: "Kiwipete",
            description: "An infamous board state to week out any edge cases",
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            expected: &[1, 48, 2_039, 97_862, 4_085_603, 193_690_690 ],
        }
    ];
}

#[derive(Copy, Clone, Debug)]
struct PerftPreset<'a> {
    name: &'a str,
    description: &'a str,
    fen: &'a str,
    expected: &'a [usize]
}


fn main() {
    let args = Cli::parse();
    let mut all_passed = true;

    if let Commands::Perftree {depth, fen, moves} = args.command {
        let mut board: Board = fen.parse().unwrap();
        let moves: Vec<Move> = moves
            .join(" ")
            .split(' ')
            .filter(|s| !s.is_empty())
            .map(|mv| Move::from_str(mv).unwrap())
            .collect();

        for mv in moves.iter() {
            board = board.play_move(*mv);
        }

        let results = perft_divide::<BULK>(board, depth);

        results.iter().for_each(|(mv, nodes)| println!("{mv} {nodes}"));
        println!("");
        println!("{}", results.iter().map(|(_, nodes)| nodes).sum::<usize>());
    } else if let Commands::Bench { fen, depth, preset } = args.command {
        println!("🏃 {}", "Running Perft test\n----------------------------".blue().italic());
        if let Some(preset) = preset {
            let preset = Preset::PRESETS[preset as usize];
            let board: Board = preset.fen.parse().unwrap();

            println!("{}: {}", "Preset".green(), preset.name);
            println!("{}: {}", "Description".green(), preset.description);
            println!("{}: {}","FEN".green(), preset.fen.italic());
            println!("{}:\n\n{board}\n\n", "Board".green());

            for depth in 0..=depth {
                let result = run_perft::<BULK>(board, depth);
                let is_match = match preset.expected.get(depth) {
                    Some(&expected) => result.nodes == expected,
                    None => true
                };

                all_passed &= is_match;

                print!("Depth {}: ", depth.to_string().blue());


                let expected = preset.expected
                    .get(depth)
                    .map(|&n| n.to_string())
                    .unwrap_or("".to_string()).green();

                print!("expected {expected:>10} ");

                let found = if is_match {
                    result.nodes.to_string().green()
                } else {
                    result.nodes.to_string().red()
                };

                print!("found {found:>10} ");

                print!("in {:10}ms ({:.3}Mnps)", result.millis(), result.mega_nps());

                if is_match {
                    print!(" 💚");
                } else {
                    print!(" 🔴");
                }

                println!("");
            }
        }

        if !all_passed {
            std::process::exit(1);
        }
    }
}
