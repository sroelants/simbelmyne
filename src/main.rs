use board::Board;

mod parse;
mod fen;
mod board;

fn main() {
    let board = Board::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    println!("{}", board);

}
