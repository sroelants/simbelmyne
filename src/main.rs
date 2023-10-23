use bitboard::Bitboard;
use movegen::castling::CastlingRights;
use movegen::moves::Move;
use std::str::FromStr;
use board::{Board, Color, PieceType, Piece};
use std::fmt::Display;
use std::io;
use std::io::Write;
use anyhow::anyhow;
use colored::*;

mod parse;
mod fen;
mod board;
mod moves;
mod bitboard;
mod movegen;

struct Game {
    board: Board,
    next: Color,
    highlights: Bitboard,

}

impl Game {
    fn play_turn(&mut self) -> anyhow::Result<()> {
        println!("{self}");
        let selected_square = get_instruction("Move which piece?\n > ")?;
        let selected_piece = self.try_select(selected_square)?;

        let legal_moves = selected_piece.legal_moves(&self.board);

        let mut highlights = Bitboard::default();
        highlights.add_in_place(selected_piece.position);
        highlights.add_in_place(legal_moves.iter().map(|mv| mv.tgt()).collect());
        self.highlights = highlights;

        println!("{self}");

        let to = get_instruction(
            &format!("Move where to?\n {} > ", selected_square.to_string().bright_blue())
        )?;

        self.highlights = Bitboard::default();

        let mv = legal_moves
            .into_iter()
            .find(|mv| mv.src() == selected_square && mv.tgt() == to)
            .ok_or(anyhow!("Not a legal move!"))?;

        self.next = self.next.opp();
        self.play(mv)?;
        Ok(())
    }

    fn try_select(&self, position: Bitboard) -> anyhow::Result<&Piece> {
        let selected = self.board.get(&position)
            .ok_or(anyhow!("No piece on square {}", position))?;

        if selected.color != self.next {
            Err(anyhow!("Selected piece belongs to the other player"))?;
        }

        Ok(selected)
    }

    fn play(&mut self, mv: Move) -> anyhow::Result<()> {
        // Update CastlingRights
        // TODO: Optimize this by having separate masks for ALL_WHITE
        // and ALL_BLACK
        // Also, not sure if there's a better way than checking on _every_ single
        // move... Maybe check until the castling rights are 0, and then stop
        // checking?
        let selected_piece = self.board.get(&mv.src())
            .expect("Move is legal, so piece is always present");
        
        if selected_piece.piece_type == PieceType::King {
            if self.next == Color::White {
                self.board.castling_rights.remove(CastlingRights::WQ);
                self.board.castling_rights.remove(CastlingRights::WK);
            } else {
                self.board.castling_rights.remove(CastlingRights::BQ);
                self.board.castling_rights.remove(CastlingRights::BK);
            }
        }

        match (mv.src().rank(), mv.src().file()) {
            (0,0) => self.board.castling_rights.remove(CastlingRights::WQ),
            (0,7) => self.board.castling_rights.remove(CastlingRights::WK),
            (7,0) => self.board.castling_rights.remove(CastlingRights::BQ),
            (7,7) => self.board.castling_rights.remove(CastlingRights::BK),
            _ => {}
        }

        // play move
        self.board.remove_at(mv.tgt());

        let mut selected_piece = self.board.get_mut(&mv.src())
            .expect("Move is legal, so piece is always present");

        selected_piece.position = mv.tgt();
        Ok(())
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "  a b c d e f g h \n".bright_blue())?;

        for rank in (0..8).rev() {
            write!(f, "{}", (rank + 1).to_string().bright_blue())?;
            write!(f, " ")?;

            for file in 0..8 {
                let current_square = Bitboard::new(rank, file);

                let character = self.board.get(&current_square)
                    .map(|piece| format!("{piece}"))
                    .unwrap_or(".".to_string());

                if self.highlights.is_empty() || self.highlights.contains(current_square) {
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
        write!(f, "Castling rights: {:?}", self.board.castling_rights)?;

        Ok(())
    }
}


fn main() {
    // let board = Board::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let board = Board::from_str("rnbqkbnr/pppppppp/8/4K3/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut game = Game { 
        board, 
        next: Color::White,
        highlights: Bitboard::default()
    };

    loop {
        if let Err(error) = game.play_turn() {
            eprintln!("{}", error)
        }
    }
}

fn get_instruction(prompt: &str) -> anyhow::Result<Bitboard> {
    let mut input = String::default();

    print!("{prompt}");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let (_, position) = parse::algebraic_square_position(&input)
        .map_err(|_| anyhow!("Invalid square {}", input))?;

    Ok(position)
}
