use anyhow::anyhow;
use chess::bitboard::Bitboard;
use chess::board::{Board, Piece, Square};
use chess::util::parse;
use colored::*;
use std::fmt::Display;
use std::{io, env};
use std::io::Write;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

struct Game {
    board: Board,
    highlights: Bitboard,
}

//TODO: We really don't need a game struct at the moment, just have a board
// state instead
impl Game {
    fn play_turn(&mut self) -> anyhow::Result<()> {
        println!("{self}");

        let selected_square = get_instruction("Move which piece?\n > ")?;
        let selected_piece = self.try_select(selected_square)?;

        let legal_moves = self.board.legal_moves();

        let mut highlights = Bitboard::EMPTY;
        highlights |= selected_piece.position;
        highlights |= legal_moves
            .iter()
            .filter(|mv| mv.src() == selected_square)
            .map(|mv| Bitboard::from(mv.tgt()))
            .collect();

        self.highlights = highlights;

        println!("{self}");

        let to = get_instruction(&format!(
            "Move where to?\n {} > ",
            selected_square.to_alg().bright_blue()
        ))?;

        self.highlights = Bitboard::EMPTY;

        let mv = legal_moves
            .into_iter()
            .filter(|mv| mv.src() == selected_square)
            .find(|mv| mv.tgt() == to.into())
            .ok_or(anyhow!("Not a legal move!"))?;

        self.board = self.board.play_move(mv);

        Ok(())
    }

    fn try_select(&self, square: Square) -> anyhow::Result<&Piece> {
        let selected = self
            .board
            .get_at(square)
            .ok_or(anyhow!("No piece on square {:?}", square))?;

        if selected.color() != self.board.current {
            Err(anyhow!("Selected piece belongs to the other player"))?;
        }

        Ok(selected)
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "  a b c d e f g h \n".bright_blue())?;

        for rank in (0..8).rev() {
            write!(f, "{}", (rank + 1).to_string().bright_blue())?;
            write!(f, " ")?;

            for file in 0..8 {
                let current_square = Square::new(rank, file);

                let character = self
                    .board
                    .get_at(current_square)
                    .map(|piece| format!("{piece}"))
                    .unwrap_or(".".to_string());

                if self.highlights.is_empty() || self.highlights.contains(current_square.into()) {
                    write!(f, "{}", character)?;
                } else {
                    write!(f, "{}", character.bright_black())?;
                }

                write!(f, " ")?;
            }
            write!(f, "{}", (rank + 1).to_string().bright_blue())?;
            write!(f, "\n")?;
        }
        write!(f, "{}", "  a b c d e f g h \n".bright_blue())?;

        Ok(())
    }
}

fn main() {
    let board: Board = env::args().skip(1).next()
        .unwrap_or(DEFAULT_FEN.to_string())
        .parse()
        .unwrap();

    ////////////////////////////////////////////////////////////////////////////
    // GAME LOOP
    ////////////////////////////////////////////////////////////////////////////

    let mut game = Game {
        board,
        highlights: Bitboard::EMPTY,
    };
    loop {
        println!("FEN: {}", game.board.to_fen());
        if let Err(error) = game.play_turn() {
            eprintln!("[{}]: {error}", "Error".red());
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // Perft
    ////////////////////////////////////////////////////////////////////////////

    // let max_depth = 5;
    //
    // for depth in 1..=max_depth {
    //     let nodes = perft(board, depth);
    //     println!("Nodes at {depth}: {nodes}");
    // }
}

fn get_instruction(prompt: &str) -> anyhow::Result<Square> {
    let mut input = String::default();

    print!("{prompt}");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let (_, square) =
        parse::algebraic_square(&input).map_err(|_| anyhow!("Invalid square {}", input))?;

    Ok(square)
}
