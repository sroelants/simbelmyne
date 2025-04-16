//! Logic for parsing FEN strings
//!
//! A FEN string (short for Forsyth-Edwards Notation) captures an entire board
//! state at a given point in time. This includes more than just the actual
//! pieces: it also includes whose turn it is, what castling rights remain,
//! whether it's possible to capture en-passant on this turn, etc...
//!
//! An example of a FEN-serialized board is:
//!
//!   rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2
//!
//! A FEN string always consists of 6 space-separated parts:
//!
//! 1. rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR
//!  The piece list, read as follows: Starting at the top rank, each character
//!  either represents a piece (in standard algebraic notation), or a number
//!  that represents a number of open squares until the next piece (or the end
//!  of the rank).
//!    
//! 2. w
//!  The player to go next
//!
//! 3. KQkq
//!  The remaining castling rights, read as "White Kingside",
//!  "White Queenside", etc... If no castling rights remain, it's simply
//!  written as a "-". (Note that these castling rights do not include
//!  temporary states like "this square is currently under attack". It only
//!  tracks whether or not the king/rooks have moved, and thus can never
//!  castle.
//!
//! 4. c6
//!  The square that is currently viable for an en-passant capture. This gets
//!  unset on the next move (or updated, if a new square becomes available).
//!  Some as with castling rights, it's simply written as a "-" when unset.
//!
//! 5. 0
//!  The half-move clock. This counts the number of half-turns (i.e, ply)
//!  since the last capture or pawn move. We need this to uphold the 50 move
//!  rule
//!
//! 6. 2
//!  The turn counter. Monotonically increasing counter that keeps track of
//!  how many full turns have gone. Gets incremented at the end of Black's turn.
//!
//! Not doing the best job at having clear errors when passed invalid FEN
//! strings, it'll just scream "Invalid!" and blow up. 💥

use std::str::FromStr;

use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::movegen::castling::CastlingRights;
use crate::piece::Color;
use crate::piece::Piece;
use crate::piece::PieceType;
use crate::square::Square;
use anyhow::anyhow;
use itertools::Itertools;

impl Board {
  // Serialize a board into a FEN string
  pub fn to_fen(&self) -> String {
    let ranks = self.piece_list.into_iter().chunks(8);
    let ranks = ranks.into_iter().collect_vec();
    let mut rank_strs: Vec<String> = Vec::new();

    for rank in ranks.into_iter().rev() {
      let mut elements: Vec<String> = Vec::new();
      let piece_runs = rank.into_iter().group_by(|p| p.is_some());

      for run in &piece_runs {
        match run {
          (true, pieces) => {
            for piece in pieces {
              elements.push(piece.unwrap().to_string())
            }
          }
          (false, gaps) => elements.push(gaps.count().to_string()),
        }
      }

      rank_strs.push(elements.join(""));
    }

    let pieces = rank_strs.into_iter().join("/");
    let next_player = self.current.to_string();
    let castling = self.castling_rights.to_string();
    let en_passant = self
      .en_passant
      .map(|sq| sq.to_string())
      .unwrap_or(String::from("-"));
    let half_moves = self.half_moves;
    let full_moves = self.full_moves;

    format!("{pieces} {next_player} {castling} {en_passant} {half_moves} {full_moves}")
  }

  // Parse a board from a FEN string
  pub fn from_fen(fen: &str) -> anyhow::Result<Board> {
    let mut parts = fen.split(' ');

    let piece_string = parts.next().ok_or(anyhow!("Invalid FEN string"))?;

    // Parse the pieces

    let mut piece_bbs = [Bitboard::EMPTY; PieceType::COUNT];
    let mut occupied_squares = [Bitboard::EMPTY; Color::COUNT];
    let mut piece_list = [None; Square::COUNT];
    let mut square_idx: usize = 0;

    // FEN starts with the 8th rank down, so we need to reverse the ranks
    // to go in ascending order
    for rank in piece_string.split('/').rev() {
      for c in rank.chars() {
        let c = c.to_string();

        if let Ok(gap) = usize::from_str(&c) {
          square_idx += gap;
        } else if let Ok(piece) = Piece::from_str(&c) {
          let square = Square::from(square_idx);
          let bb = Bitboard::from(square);

          piece_list[square_idx] = Some(piece);
          piece_bbs[piece.piece_type()] |= bb;
          occupied_squares[piece.color()] |= bb;

          square_idx += 1;
        }
      }
    }

    // Parse the game state

    let current: Color =
      parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

    let castling_rights: CastlingRights =
      parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

    let en_passant: Option<Square> = parts
      .next()
      .ok_or(anyhow!("Invalid FEN string"))?
      .parse()
      .ok();

    let half_moves =
      parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

    let full_moves =
      parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

    let board = Board::new(
      piece_list,
      piece_bbs,
      occupied_squares,
      current,
      castling_rights,
      en_passant,
      half_moves,
      full_moves,
    );

    Ok(board)
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_to_fen() {
  let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
  let board = Board::from_str(initial_fen).unwrap();
  let fen = board.to_fen();
  assert_eq!(initial_fen, fen);
}
