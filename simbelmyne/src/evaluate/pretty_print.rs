use chess::{board::Board, square::Square};
use colored::Colorize;

use crate::evaluate::Evaluate;

use super::{Eval, EvalContext};

const WHITE: bool = true;
const BLACK: bool = false;

fn blank_line(rank: usize) -> String {
        let mut line: Vec<String> = Vec::new();
        line.push("  ║".to_string());
    if rank % 2 == 0 {
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
    } else {
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
    }

    line.push("║ ".to_string());
    line.join("")
}

pub fn print_eval(board: &Board) -> String {
    let eval = Eval::new(board);

    let mut lines: Vec<String> = vec![];
    lines.push("      a      b      c      d      e      f      g      h    ".to_string());
    lines.push("  ╔════════════════════════════════════════════════════════╗".to_string());

    for (rank, squares) in Square::RANKS.into_iter().enumerate() {
        lines.push(blank_line(rank));

        // Piece character
        let mut line: Vec<String> = vec![];
        line.push((8 - rank).to_string());
        line.push(" ║".to_string());
        for (file, sq) in squares.into_iter().enumerate() {
            let bg = if (rank + file) % 2 == 0 { "white" } else { "black" };
            let fg = if (rank + file) % 2 == 0 { "black" } else { "white" };

            let square = match board.get_at(sq) {
                Some(piece) => format!("   {}   ", piece).color(fg).on_color(bg),
                None => "       ".to_string().on_color(bg),
            };

            line.push(square.to_string());
        }
        line.push("║ ".to_string());
        line.push((8 - rank).to_string());
        lines.push(line.join(""));

        lines.push(blank_line(rank));

        // Piece score
        let mut line: Vec<String> = vec![];
        line.push("  ║".to_string());
        for (file, sq) in squares.into_iter().enumerate() {
            let bg = if (rank + file) % 2 == 0 { "white" } else { "black" };
            let fg = if (rank + file) % 2 == 0 { "black" } else { "white" };
            let score = if let Some(piece) = board.get_at(sq) {
                // Get score for piece
                let score = board.material(piece) + board.psqt(piece, sq);
                let pawn_score = score.lerp(eval.game_phase) as f32 / 100.0;

                format!("{:.2}", pawn_score)
            } else {
                "".to_string()
            };

            line.push(format!("{:^7}", score.color(fg).on_color(bg)));
            
        }
        line.push("║  ".to_string());
        let line = line.join("");

        lines.push(line);


    }
    lines.push("  ╚════════════════════════════════════════════════════════╝".to_string());
    lines.push("      a      b      c      d      e      f      g      h    ".to_string());

    lines.push("\n".to_string());
    lines.push("Evaluation features:".blue().to_string());
    lines.push("--------------------".blue().to_string());

    let mut ctx = EvalContext::new(board);

    let white_pawn_structure =  eval.pawn_structure.compute_score::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_structure = -eval.pawn_structure.compute_score::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn structure:", white_pawn_structure, black_pawn_structure));

    let white_bishop_pair =  board.bishop_pair::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_bishop_pair = -board.bishop_pair::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Bishop pair", white_bishop_pair, black_bishop_pair));

    let white_rook_open_file =  board.rook_open_file::<WHITE>(&eval.pawn_structure).lerp(eval.game_phase) as f32 / 100.0;
    let black_rook_open_file = -board.rook_open_file::<BLACK>(&eval.pawn_structure).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Rook on open file:", white_rook_open_file, black_rook_open_file));

    let white_pawn_shield =  board.pawn_shield::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_shield = -board.pawn_shield::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn shield:", white_pawn_shield, black_pawn_shield));

    let white_pawn_storm =  board.pawn_storm::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_storm = -board.pawn_storm::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn storm:", white_pawn_storm, black_pawn_storm));

    let white_mobility =  board.mobility::<WHITE>(&mut ctx, &eval.pawn_structure).lerp(eval.game_phase) as f32 / 100.0;
    let black_mobility = -board.mobility::<BLACK>(&mut ctx, &eval.pawn_structure).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Mobility:", white_mobility, black_mobility));

    let white_virtual_mobility =  board.virtual_mobility::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_virtual_mobility = -board.virtual_mobility::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Virtual mobility:", white_virtual_mobility, black_virtual_mobility));

    let white_king_zone =  board.king_zone::<WHITE>(&mut ctx).lerp(eval.game_phase) as f32 / 100.0;
    let black_king_zone = -board.king_zone::<BLACK>(&mut ctx).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "King zone:", white_king_zone, black_king_zone));

    lines.push("".to_string());

    lines.push(format!("Total: {}", eval.total(&board)));

    lines.join("\n")
}

