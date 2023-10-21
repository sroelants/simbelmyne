use board::{Board, Color, Bitboard};
use std::fmt::Display;
use std::io;
use std::io::Write;
use anyhow::anyhow;
use colored::*;

mod parse;
mod fen;
mod board;
mod moves;

struct Game {
    board: Board,
    next: Color,
    selected: Option<Bitboard>
}

impl Game {
    fn play_turn(&mut self) -> anyhow::Result<()> {
        println!("{self}");
        let from = get_instruction("Move which piece?\n > ")?;
        self.try_select(from)?;

        println!("{self}");
        let to = get_instruction(
            &format!("Move where to?\n {} > ", from.to_string().bright_blue())
        )?;
        self.try_play(to)?;

        self.selected = None;
        self.next = match &self.next {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }

    fn try_select(&mut self, position: Bitboard) -> anyhow::Result<()> {
        let selected = self.board.get(&position)
            .ok_or(anyhow!("No piece on square {}", position))?;

        if selected.color != self.next {
            Err(anyhow!("Selected piece belongs to the other player"))?;
        }

        self.selected = Some(position);

        Ok(())
    }

    fn try_play(&mut self, position: Bitboard) -> anyhow::Result<()> {
        // Check whether destination is blocked
        // TODO: At some point, we'll use the actual legal moves to verify this
        if let Some(true) = self.board.get(&position)
            .map(|piece| piece.color == self.next) {
            Err(anyhow!("There's one of your pieces on {}", position))?
        }

        // play move
        if let Some(captured) = self.board.remove_at(position) {
            println!("Captured {:?}", captured);
        }

        let selected = self.selected
            .ok_or(anyhow!("No piece selected to move"))?;

        if let Some(mut selected) = self.board.get_mut(selected) {
            selected.position = position;
            selected.has_moved = true;
        };

        Ok(())
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut highlights = Bitboard::default();
        let mut legal_moves = Bitboard::default();

        if let Some(selected) = self.selected {
            highlights.add_in_place(selected);

            let pushes = self.board.get(&selected)
                .map(|piece| moves::pushes(piece, &self.board))
                .unwrap_or_default()
                .into();
            legal_moves.add_in_place(pushes);

            let attacks = self.board.get(&selected)
                .map(|piece| moves::attacks(piece, &self.board))
                .unwrap_or_default()
                .into();
            legal_moves.add_in_place(attacks);

            highlights.add_in_place(legal_moves);
        } 

        write!(f, "{}", "  a b c d e f g h \n".bright_blue())?;

        for rank in (0..8).rev() {
            write!(f, "{}", (rank + 1).to_string().bright_blue())?;
            write!(f, " ")?;

            for file in 0..8 {
                let current_square = Bitboard::new(rank, file);

                let character = self.board.get(&current_square)
                    .map(|piece| format!("{piece}"))
                    .unwrap_or(".".to_string());

                // If there are no highlights, render everything as normal
                if highlights.bits() == 0 {
                    write!(f, "{}", character)?;

                // If there's a highlight, render the current square white if 
                // it's in the highlight.
                } else if highlights.contains(current_square) {
                    write!(f, "{}", character)?;

                // If there's a highlight, but the current square isn't in it,
                // render it dimmed.
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
    // let board = Board::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let board = Board::try_from("rnbqkbnr/pppppppp/8/4K3/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut game = Game { 
        board, 
        next: Color::White,
        selected: None,
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
