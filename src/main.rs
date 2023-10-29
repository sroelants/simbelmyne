use bitboard::Bitboard;
use movegen::castling::{CastleType, CastlingRights};
use movegen::moves::Move;
use std::str::FromStr;
use board::{Board, Color, PieceType, Piece, Square};
use std::fmt::Display;
use std::io;
use std::io::Write;
use anyhow::anyhow;
use colored::*;

mod parse;
mod fen;
mod board;
mod bitboard;
mod movegen;

struct Game {
    board: Board,
    highlights: Bitboard,
}

impl Game {
    fn play_turn(&mut self) -> anyhow::Result<()> {
        println!("{self}");

        let selected_square = get_instruction("Move which piece?\n > ")?;
        let selected_piece = self.try_select(selected_square)?;

        let legal_moves = self.board.legal_moves();

        let mut highlights = Bitboard::default();
        highlights |= selected_piece.position;
        highlights |= legal_moves
            .iter()
            .filter(|mv| mv.src() == selected_square)
            .map(|mv| Bitboard::from(mv.tgt()))
            .collect();

        self.highlights = highlights;

        println!("{self}");

        let to = get_instruction(
            &format!("Move where to?\n {} > ", selected_square.to_alg().bright_blue())
        )?;

        self.highlights = Bitboard::default();

        let mv = legal_moves
            .into_iter()
            .filter(|mv| mv.src() == selected_square)
            .find(|mv| mv.tgt() == to.into())
            .ok_or(anyhow!("Not a legal move!"))?;

        self.play(mv)?;

        self.board.current_player = self.board.current_player.opp();
        Ok(())
    }

    fn try_select(&self, square: Square) -> anyhow::Result<&Piece> {
        let selected = self.board.get_at(square)
            .ok_or(anyhow!("No piece on square {:?}", square))?;

        if selected.color() != self.board.current_player {
            Err(anyhow!("Selected piece belongs to the other player"))?;
        }

        Ok(selected)
    }

    fn play(&mut self, mv: Move) -> anyhow::Result<()> {
        // Remove selected piece from board, and update fields
        let mut selected_piece = self.board.remove_at(mv.src().into())
            .expect("We're sure there's a piece on the source square");

        selected_piece.position = mv.tgt().into();

        // Update Castling rights
        // If the piece is a king, revoke that side's castling rights
        if selected_piece.piece_type() == PieceType::King {
            if self.board.current_player == Color::White {
                self.board.castling_rights.remove(CastlingRights::WQ);
                self.board.castling_rights.remove(CastlingRights::WK);
            } else {
                self.board.castling_rights.remove(CastlingRights::BQ);
                self.board.castling_rights.remove(CastlingRights::BK);
            }
        }

        // If any of the rooks moved, revoke their respective castling rights
        if selected_piece.piece_type() == PieceType::Rook {
            match (mv.src().rank(), mv.src().file()) {
                (0,0) => self.board.castling_rights.remove(CastlingRights::WQ),
                (0,7) => self.board.castling_rights.remove(CastlingRights::WK),
                (7,0) => self.board.castling_rights.remove(CastlingRights::BQ),
                (7,7) => self.board.castling_rights.remove(CastlingRights::BK),
                _ => {}
            }
        }

        // play move
        let _captured = self.board.remove_at(mv.tgt().into()); //Captured piece?
        self.board.add_at(mv.tgt().into(), selected_piece);

        if mv.is_en_passant() {
            let capture_sq = mv.tgt().backward(self.board.current_player)
                .expect("En-passant capture target is in bounds");
            self.board.remove_at(capture_sq);
        }

        if mv.is_castle() {
            let ctype = CastleType::from_move(mv).unwrap();
            self.play(ctype.rook_move())?;
        }

        // Update en-passant square
        if mv.is_double_push() {
            self.board.en_passant = mv.src().forward(self.board.current_player)
        } else {
            self.board.en_passant = None;
        }

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
                let current_square = Square::new(rank, file);

                let character = self.board.get_at(current_square)
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
    // let board = Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    // let board = Board::from_str("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    // let board = Board::from_str("1k6/8/8/4b3/8/2Q5/8/K7 w - - 0 1").unwrap();
    let board = Board::from_str("rnbqkbnr/ppppp1pp/8/8/3P1p2/2P5/PP2PPPP/RNBQKBNR w KQkq - 0 3").unwrap();
    let mut game = Game { 
        board, 
        highlights: Bitboard::default()
    };

    loop {
        if let Err(error) = game.play_turn() {
            eprintln!("[{}]: {error}", "Error".red());
        }
    }
}

fn get_instruction(prompt: &str) -> anyhow::Result<Square> {
    let mut input = String::default();

    print!("{prompt}");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let (_, square) = parse::algebraic_square(&input)
        .map_err(|_| anyhow!("Invalid square {}", input))?;

    Ok(square)
}
