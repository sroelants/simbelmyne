use chess::board::Board;
use crate::{perft::run_perft, BULK};
use colored::*;
use anyhow::*;

#[derive(Debug, clap::ValueEnum, Clone, Copy)]
pub enum Preset {
    StartingPos,
    Kiwipete,
}

#[derive(Copy, Clone, Debug)]
struct PerftPreset<'a> {
    name: &'a str,
    description: &'a str,
    fen: &'a str,
    expected: &'a [usize]
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

pub fn run_bench(depth: usize, fen: Option<String>, preset: Option<Preset>) -> anyhow::Result<()> {
    let mut all_passed = true;

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

    Ok(())
}
