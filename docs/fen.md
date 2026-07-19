# Forsyth-Edwards Notation

Forsyth-Edwards notation (FEN) is a compact way of representing the state of 
a game of chess. [^repetitions]

An typical FEN string looks as follows:

```
rnbq1rk1/ppp2ppp/4pn2/3p4/1bPP4/2N1PN2/PP3PPP/R1BQKB1R w KQ - 1 6
```

We'll get into what each individual bit means, but at a surface level, each
FEN string breaks down as follows:

```
<pieces> <side-to-move> <castling-rights> <en-passant-square> <halfmoves> <fullmoves>
```

## Pieces
The first part of a FEN string represents the piece layout:

```
rnbq1rk1/ppp2ppp/4pn2/3p4/1bPP4/2N1PN2/PP3PPP/R1BQKB1R
```

The way to read this string is as follows: Seperated by slashes (`/`) are 
the pieces on each individual rank, starting at the top of the board (rank 8)
down to the bottom (rank 1).

Lowercase letters represent black pieces, while uppercase letters represent
white pieces, in typical [algebraic notation](https://en.wikipedia.org/wiki/Algebraic_notation_(chess)).
Empty squares are represented by numbers. So, our example FEN string describes
the 8th rank as:

```
rook, knight, bishop, queen, <empty square>, rook, king, <empty square>
```

## Side-to-move
Pretty straightforward: lowercase `w` for white-to-move, lowercase `b` for
black-to-move.

## Castling rights
Castling rights are encoded by whether or not kingside (`k`) or queenside (`q`)
castling is still available to the player. For example, a string like `KQkq` 
indicates that both sides can castle both kingside and queenside.

A string like `Kq` indicates that white can only castle kingside, and black can
only castle queenside.

Note that these castling rights only track whether or not it is _in principle_
possible to castle. For example, moving the kingside rook will remove the
kingside castling right, while moving the king will remove both. It does _not_
take into account other conditions for legal castling, like whether or not the
king is in check or has an unobstructed path.

[^repetitions]: It does not, however, represent _everything_ there is to know a
about the game state, though! In particular, it is impossible to know, given a
FEN string, whether there have been any repeated positions up to this point.
This is important information, due to the [threefold repetition rule](https://en.wikipedia.org/wiki/Threefold_repetition).
