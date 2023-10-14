use board::{Board, Color, Position};
use std::io;
use std::io::Write;

mod parse;
mod fen;
mod board;

struct Game {
    board: Board,
    next: Color,
}

impl Game {
    fn play(&mut self, from: Position, to: Position) -> Result<(), anyhow::Error> {
        // Check if there's actually a piece there
        let piece = self.board
            .get(from)
            .ok_or(anyhow::anyhow!("No piece on square"))?;

        // Check whether the piece is the correct color
        if piece.color != self.next {
            return Err(anyhow::anyhow!("Piece is the wrong color"));
        }

        // Check whether there's a piece of the same color on the other square
        if let Some(target_piece) = self.board.get(to) {
            if target_piece.color == self.next {
                return Err(anyhow::anyhow!("Another one of your pieces is already there"));
            }
        }

        // Remove the other piece
        if let Some(captured) = self.board.remove_at(to) {
            println!("Captured {:?}", captured);
        }

        // Actually move the piece
        let mut piece = self.board
            .get_mut(from)
            .ok_or(anyhow::anyhow!("No piece on square"))?;

        piece.position = to;

        // Update the next player's color
        self.next = match &self.next {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        Ok(())
    }
}

fn main() {
    let board = Board::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let mut game = Game { board, next: Color::White };


    loop {
        println!("{}", game.board);
        print!("Provide next move > ");

        let mut next_move = String::default();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut next_move).unwrap();

        let (_, (from, to)) = parse::instruction(&next_move).unwrap();

        if let Err(msg) = game.play(from, to) {
            println!("Error: {}", msg);
        }

        println!("");
    }
}
