use crate::{perft::run_perft, BULK, presets::Preset};
use anyhow::*;
use chess::board::Board;
use colored::*;

pub fn run_bench(depth: usize, fen: Option<String>, preset: Option<Preset>) -> anyhow::Result<()> {

    println!(
        "🏃 {}",
        "Running Perft test\n----------------------------"
            .blue()
            .italic()
    );

    if let Some(fen) = fen {
        let board: Board = fen.parse().unwrap();

        println!("{}: {}", "FEN".green(), fen.italic());
        println!("{}:\n\n{board}\n\n", "Board".green());

        for depth in 0..=depth {
            let result = run_perft::<BULK>(board, depth);

            print!("Depth {}: ", depth.to_string().blue());

            print!("found {:>12} ", result.nodes.to_string().green());

            print!("in {:5}ms ({:.3}Mnps)", result.millis(), result.mega_nps());

            println!("");
        }

    } else if let Some(preset) = preset {
        let mut all_passed = true;
        let preset = Preset::load_preset(preset);
        let board: Board = preset.fen.parse().unwrap();

        println!("{}: {}", "Preset".green(), preset.name);
        println!("{}: {}", "Description".green(), preset.description);
        println!("{}: {}", "FEN".green(), preset.fen.italic());
        println!("{}:\n\n{board}\n\n", "Board".green());

        for depth in 0..=depth {
            let result = run_perft::<BULK>(board, depth);
            let is_match = match preset.expected.get(depth) {
                Some(&expected) => result.nodes == expected,
                None => true,
            };

            all_passed &= is_match;

            print!("Depth {}: ", depth.to_string().blue());

            let expected = preset
                .expected
                .get(depth)
                .map(|&n| n.to_string())
                .unwrap_or("".to_string())
                .green();

            print!("expected {expected:>12} ");

            let found = if is_match {
                result.nodes.to_string().green()
            } else {
                result.nodes.to_string().red()
            };

            print!("found {found:>12} ");

            let mnps_str = format!("({:.3}Mnps)",result.mega_nps());
            print!("in {:5}ms {:>15}", result.millis(), mnps_str);

            if is_match {
                print!("{:>3}","💚");
            } else {
                print!("{:>3}", "🔴");
            }

            println!("");

        }

        if !all_passed {
            return Err(anyhow!("Some perft results didn't match the expected value."));
        }
    }

    Ok(())
}
