use chess::{board::Board, square::Square};
use colored::Colorize;

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
                let score = eval.material(piece, None) + eval.psqt(piece, sq, None);
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
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Pawn structure:", white_pawn_structure, black_pawn_structure));

    let white_bishop_pair =  eval.bishop_pair::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_bishop_pair = -eval.bishop_pair::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Bishop pair", white_bishop_pair, black_bishop_pair));

    let white_rook_open_file =  eval.rook_open_file::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_rook_open_file = -eval.rook_open_file::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Rook on open file:", white_rook_open_file, black_rook_open_file));

    let white_rook_semiopen_file =  eval.rook_semiopen_file::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_rook_semiopen_file = -eval.rook_semiopen_file::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Rook on semiopen file:", white_rook_semiopen_file, black_rook_semiopen_file));

    let white_connected_rooks =  eval.connected_rooks::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_connected_rooks = -eval.connected_rooks::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Connected rooks:", white_connected_rooks, black_connected_rooks));

    let white_queen_open_file =  eval.queen_open_file::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_queen_open_file = -eval.queen_open_file::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Queen on open file:", white_queen_open_file, black_queen_open_file));

    let white_queen_semiopen_file =  eval.queen_semiopen_file::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_queen_semiopen_file = -eval.queen_semiopen_file::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Queen on semiopen file:", white_queen_semiopen_file, black_queen_semiopen_file));

    let white_major_on_7th =  eval.major_on_seventh::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_major_on_7th = -eval.major_on_seventh::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Major on 7th:", white_major_on_7th, black_major_on_7th));

    let white_pawn_shield =  eval.pawn_shield::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_shield = -eval.pawn_shield::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Pawn shield:", white_pawn_shield, black_pawn_shield));

    let white_pawn_storm =  eval.pawn_storm::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_storm = -eval.pawn_storm::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Pawn storm:", white_pawn_storm, black_pawn_storm));

    let white_passers_friendly_king =  eval.passers_friendly_king::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_passers_friendly_king = -eval.passers_friendly_king::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Passers - Friendly king:", white_passers_friendly_king, black_passers_friendly_king));

    let white_passers_enemy_king =  eval.passers_enemy_king::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_passers_enemy_king = -eval.passers_enemy_king::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Passers - Enemy king:", white_passers_enemy_king, black_passers_enemy_king));

    let white_mobility =  eval.mobility::<WHITE>(&board, &mut ctx, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_mobility = -eval.mobility::<BLACK>(&board, &mut ctx, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Mobility:", white_mobility, black_mobility));

    let white_virtual_mobility =  eval.virtual_mobility::<WHITE>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_virtual_mobility = -eval.virtual_mobility::<BLACK>(&board, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "Virtual mobility:", white_virtual_mobility, black_virtual_mobility));

    let white_king_zone =  eval.king_zone::<WHITE>(&mut ctx, None).lerp(eval.game_phase) as f32 / 100.0;
    let black_king_zone = -eval.king_zone::<BLACK>(&mut ctx, None).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<25} {:>7.2} {:>7.2}", "King zone:", white_king_zone, black_king_zone));

    lines.push("".to_string());

    lines.push(format!("Total: {}", eval.total(&board)));

    lines.join("\n")
}

