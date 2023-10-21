use nom::IResult;
use nom::bytes::complete::tag;
use nom::character::complete::{char, anychar};
use nom::character::complete::u64;
use nom::multi::many1;
use nom::multi::separated_list1;
use nom::Err;
use nom::error::Error;
use nom::error::ErrorKind;
use nom::sequence::separated_pair;
use crate::board::Bitboard;
use crate::fen::FENAtom;
use crate::board::{Color, PieceType};


type ParseResult<'a, O> = IResult<&'a str, O>;

fn generic_error(input: &str) -> nom::Err<nom::error::Error<&str>> {
    Err::Error(Error {input, code: ErrorKind::Fail})
}

pub fn algebraic_piece(input: &str) -> ParseResult<(Color, PieceType)> {
    let (rest, ch) = anychar(input)?;

    match ch {
        'p' => Ok((rest, (Color::Black, PieceType::Pawn))),
        'r' => Ok((rest, (Color::Black, PieceType::Rook))),
        'n' => Ok((rest, (Color::Black, PieceType::Knight))),
        'b' => Ok((rest, (Color::Black, PieceType::Bishop))),
        'q' => Ok((rest, (Color::Black, PieceType::Queen))),
        'k' => Ok((rest, (Color::Black, PieceType::King))),

        'P' => Ok((rest, (Color::White, PieceType::Pawn))),
        'R' => Ok((rest, (Color::White, PieceType::Rook))),
        'N' => Ok((rest, (Color::White, PieceType::Knight))),
        'B' => Ok((rest, (Color::White, PieceType::Bishop))),
        'Q' => Ok((rest, (Color::White, PieceType::Queen))),
        'K' => Ok((rest, (Color::White, PieceType::King))),

        _ => Err(generic_error(input))
    }
}

pub fn algebraic_rank(input: &str) -> ParseResult<u64> {
    let (rest, num) = u64(input)?;

    if num <= 8 {
        Ok((rest, num - 1))
    } else {
        Err(generic_error(input))
    }
}

pub fn algebraic_file(input: &str) -> ParseResult<u64> {
    let (rest, ch) = anychar(input)?;

    match ch {
        'a' => Ok((rest, 0)),
        'b' => Ok((rest, 1)),
        'c' => Ok((rest, 2)),
        'd' => Ok((rest, 3)),
        'e' => Ok((rest, 4)),
        'f' => Ok((rest, 5)),
        'g' => Ok((rest, 6)),
        'h' => Ok((rest, 7)),

        _ => Err(generic_error(input))
    }
}

pub fn algebraic_square(input: &str) -> ParseResult<(u64, u64)> {
    nom::sequence::pair(algebraic_file, algebraic_rank)(input)
}

pub fn algebraic_square_position(input: &str) -> ParseResult<Bitboard> {
    algebraic_square(input)
        .map(|(rest, (file, rank))| (rest, Bitboard::new(rank, file)))
}

pub fn fen_gap(input: &str) -> ParseResult<u64> {
    let (rest, num) = u64(input)?;

    if 0 < num && num <= 8 {
        Ok((rest, num))
    } else {
        Err(generic_error(input))
    }
}

pub fn fen_atom(input: &str) -> ParseResult<FENAtom> {
    let result = algebraic_piece(input);

    if result.is_ok() {
        result.map(|(rest, (color, piece))| (rest, FENAtom::Piece(color, piece)))
    } else {
        fen_gap(input).map(|(rest, gap)| (rest, FENAtom::Gap(gap)))
    }
}

pub fn fen_rank(input: &str) -> ParseResult<Vec<FENAtom>> {
    many1(fen_atom)(input)
}

pub fn fen_board(input: &str) -> ParseResult<Vec<Vec<FENAtom>>> {
    separated_list1(tag("/"), fen_rank)(input)
}

pub fn fen_color(input: &str) -> ParseResult<Color> {
    let (rest, ch) = anychar(input)?;

    match ch {
        'w' => Ok((rest, Color::White)),
        'b' => Ok((rest, Color::Black)),

        _ => Err(generic_error(input))
    }
}

pub fn instruction(input: &str) -> ParseResult<(Bitboard, Bitboard)> {
    separated_pair(
        algebraic_square_position, 
        char(' '), 
        algebraic_square_position
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_piece_white_king() {
        let result = algebraic_piece("King");
        assert_eq!(result, Ok(("ing", (Color::White, PieceType::King))));
    }

    #[test]
    fn algebraic_piece_black_knight() {
        let result = algebraic_piece("night");
        assert_eq!(result, Ok(("ight", (Color::Black, PieceType::Knight))));
    }

    #[test]
    fn algebraic_piece_white_queen() {
        let result = algebraic_piece("Quick");
        assert_eq!(result, Ok(("uick", (Color::White, PieceType::Queen))));
    }

    #[test]
    fn algebraic_piece_fail() {
        let result = algebraic_piece("Hello");
        assert_eq!(
            result, 
            Err(Err::Error(Error { input: "Hello", code: ErrorKind::Fail }))
        );
    }

    #[test]
    fn algebraic_rank_works() {
        assert_eq!(algebraic_rank("7"), Ok(("", 6)));
    }

    #[test]
    fn algebraic_rank_no_alpha() {
        assert_eq!(
            algebraic_rank("a"), 
            Err(Err::Error(Error { input: "a", code: ErrorKind::Digit } ))
        );
    }

    #[test]
    fn algebraic_rank_bounds() {
        assert_eq!(
            algebraic_rank("9"), 
            Err(Err::Error(Error { input: "9", code: ErrorKind::Fail } ))
        );
    }

    #[test]
    fn algebraic_rank_multiple_digits() {
        assert_eq!(
            algebraic_rank("10"), 
            Err(Err::Error(Error { input: "10", code: ErrorKind::Fail } ))
        );
    }

    #[test]
    fn algebraic_file_works() {
        assert_eq!(algebraic_file("c"), Ok(("", 2)));
        assert!(algebraic_file("x").is_err());
    }

    #[test]
    fn algebraic_square_e4() {
      assert_eq!(algebraic_square("e4"), Ok(("", (4,3))));
    }

    #[test]
    fn algebraic_square_e10() {
      assert!(algebraic_square("e10").is_err());
    }

    #[test]
    fn test_fen_rank() {
      assert_eq!(
      fen_rank("Kn4QS"), 
      Ok(("S", vec![
          FENAtom::Piece(Color::White, PieceType::King),
          FENAtom::Piece(Color::Black, PieceType::Knight),
          FENAtom::Gap(4),
          FENAtom::Piece(Color::White, PieceType::Queen),
      ])))
    }

    #[test]
    fn test_fen_board() {
      assert_eq!(
      fen_board("Kn4Q/pppppppp/8/8/8/8/pppppppp and some other stuff"), 
      Ok((" and some other stuff", vec![
          vec![
              FENAtom::Piece(Color::White, PieceType::King),
              FENAtom::Piece(Color::Black, PieceType::Knight),
              FENAtom::Gap(4),
              FENAtom::Piece(Color::White, PieceType::Queen),
          ],
          vec![FENAtom::Piece(Color::Black, PieceType::Pawn); 8],
          vec![FENAtom::Gap(8)],
          vec![FENAtom::Gap(8)],
          vec![FENAtom::Gap(8)],
          vec![FENAtom::Gap(8)],
          vec![FENAtom::Piece(Color::Black, PieceType::Pawn); 8],
      ])))
    }
}
