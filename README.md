<div align="center">
  <img src="./assets/simbelmyne_logo.svg" />
</div>

# <div align="center">Simbelmyne</div>

<div align="center">

[![License][license-badge]][license-link]
[![Buid][build-badge]][build-link]
[![release][release-badge]][release-link]

[![lichess-badge]][lichess-link]

</div>

## About
Simbelmyne is a UCI-compliant chess engine. It uses a bitboard-based board
representation, a traditional hand-crafted evaluation function, and is powered
by an optimized alpha-beta search. See [Details](#Details) for more information
on the optimizations that are performed.

A main motivation for this project was to get more familiar with writing Rust,
so let that be a warning that anyone reading the code might find the odd
non-idiomatic, or downright stupid implementation. 

## Rating
Assigning an objective rating to a chess engine is tough. Values will change wildly depending on the machine the engine is running on or what time control is used.

Below is a table of different Elo estimates obtained by having Simbelmyne play against other engines. The used time-controls are listed, as `time / increment`, in seconds.

| Version | Estimate (10/0.1) | [MCERL](https://www.chessengeria.eu/mcerl) (60/0.6) | [CEDR](https://chessengines.blogspot.com/p/rating-jcer.html) (180/3) | [CCRL](https://computerchess.org.uk/ccrl/4040/) (40/15) | [CCRL Blitz](https://computerchess.org.uk/ccrl/4040/) (2/1)
|---------|----------|-------------|-----------|-----------|---------|
| v1.0.0  | 2000     | 2247        |           |           |         |
| v1.1.0  | 2100     | 2293        |           |           |         |
| v1.2.0  | 2350     | 2457        | 2393      |           |         |
| v1.3.0  | 2500     | 2567        | 2505      |           |         |
| v1.3.1  | 2500     |             |           | 2465      |         |
| v1.4.0  | 2650     |             |           |           |         |
| v1.5.0  | 2700     |             |           |           |         |
| v1.5.1  | 2700     |             |           |           | 2702    |
| v1.6.0  | 2760     |             |           |           |         |

A huge thank you goes out to the people kind enough to have gone out of their way to test Simbelmyne!

## Play
Like most chess engines, Simbelmyne is mostly designed to be used through the
UCI protocol. Simply running `simbelmyne` from the command line will drop you
into a UCI prompt that you can use to interact with the engine, if you so want.
The saner option is to use a dedicated UCI-compatible frontend. Some examples
are:
- [Arena][arena]
- [Cutechess][cutechess]
- [Shredder][shredder]

If that feels like too much effort, Simbelmyne is also available for play [as a 
lichess bot][lichess-link].

## Building the project
Simbelmyne is developed with Rust v1.73, and most easily built using the
[Cargo][cargo] toolchain.

From the project root, run `cargo build --release`, and the resulting binary 
will be found at `target/release/simbelmyne`

## Details
Simbelmyne follows a fairly traditional chess engine architecture. The two main
pillars underpinning everything are the Search and Evaluation. 

### Search
The search subsystem is all about visiting as many board positions as possible,
and looking as many moves ahead as possible, in the least amount of time. The
algorithm used for this is a classical [Negamax search][negamax]. The following
optimizations are added on top to improve the search speed and quality:

#### Move generation
- [Legal move generation][legal-moves]
- [Bitboard representation][bitboards]
- [Magic bitboards][magic-bitboards]

#### Pruning, reductions, extensions
- [Alpha-beta pruning][alpha-beta]
- [Null-move pruning][null-move]
- [Transposition table][transposition-table]
- Internal iterative reduction
- [Futility pruning][futility-pruning]
- [Reverse futility pruning][reverse-futility-pruning]
- [Late move pruning][late-move-pruning]
- Late move reductions
- [Check extensions][check-extensions]
- [Quiescence search][quiescence-search]

#### Move ordering
- [MVV-LVA move ordering][mvv-lva]
- [Killer move ordering][killer-move]
- [History tables][history-tables]
- [Hash move][tt-move]
- [Static exchange evaluation][see]

### Evaluation
If the search part of the engine is all about "try and search as deep as
possible", then the evaluation is all about making sense of what is found there.
The engine needs to figure out, by some metric, what board positions are more 
favorable than others. This is where a lot of the hard-earned experience of 
chess-players throughout the ages gets codified into computer-understandable 
heuristics. 

As much as possible, the evaluation function tries to compute evaluation terms
incrementally, and retrieving non-incremental values from the Transposition
table when possible.

- [Material counting][material-counting]
- [Piece-square tables][pst]
- Pawn structure
  - Passed pawns
  - Isolated pawns
  - Protected pawns
  - Phalanx pawns
  - passed pawn tropism to friendly king
  - passed pawn tropism to enemy king king
- Minor piece outposts
- Bishop pair
- Rook on (semi-) open file
- Connected rooks
- Queen on (semi-) open file
- Major piece on 7th rank
- Mobility, taking into consideration pawn attacks and pins
- Threats
- King safety
  - Virtual mobility
  - King zone attacks
  - Pawn shield
  - Pawn storm

## Acknowledgements
Simbelmyne was inspired, and has drawn a lot from many different people,
resources and codebases, so it is only proper to give thanks where thanks are
due.

### Resources
- Sebastian Lague's [excellent chess programming videos][lague] that sparked the idea to 
  write my own engine.
- The Vice [video series][vice]
- The [chess programming wiki][cpw]. Not always the most digestible, but truly a a
  vast wealth of knowledge is to be found there.
- The [TalkChess][talk-chess] forums

### Engines
- [Carp][carp]: especially when starting off, it was good to have a reference
  implementation in Rust to compare notes with
- [Viridithas][viri]: Same. Especially appreciated the [viri wiki][viri-wiki] that briefly
- outlines the high-level features of the engine.
- [Rustic][rustic]: Especially starting out, the Rustic [book][rustic-book] was 
  a great resource. Far more coherent and digestible, if less comprehensive,
  than the CPW.
- [Blunder][blunder]: Delightfully readable and well-documented codebase!
- [Stockfish][stockfish]: OF course, the uber-reference. When in doubt, do as
  Stockfish does.

[license-badge]: https://img.shields.io/github/license/sroelants/simbelmyne?style=for-the-badge&color=blue
[license-link]: https://github.com/sroelants/simbelmyne/blob/main/LICENSE

[build-badge]: https://img.shields.io/github/actions/workflow/status/sroelants/simbelmyne/tests.yml?style=for-the-badge
[build-link]: https://github.com/sroelants/simbelmyne/actions/workflows/tests.yml

[release-badge]: https://img.shields.io/github/v/release/sroelants/simbelmyne?style=for-the-badge&color=violet
[release-link]: https://github.com/sroelants/simbelmyne/releases/latest

[lichess-badge]:https://img.shields.io/badge/Play-v1.6.0-yellow?logo=lichess&style=for-the-badge
[lichess-link]: https://lichess.org/@/simbelmyne-bot

[arena]: http://www.playwitharena.de
[cutechess]: https://cutechess.com
[shredder]: https://www.shredderchess.com

[cargo]: https://doc.rust-lang.org/cargo

[negamax]: https://en.wikipedial.com/wiki/Negamax
[legal-moves]: https://www.chessprogramming.org/Move_Generation#Legal
[bitboards]: https://www.chessprogramming.org/Bitboards
[magic-bitboards]: https://www.chessprogramming.org/Magic_Bitboards
[alpha-beta]: https://www.chessprogramming.org/Alpha-Beta
[null-move]: https://www.chessprogramming.org/Null_Move_Pruning
[futility-pruning]: https://www.chessprogramming.org/Futility_Pruning
[reverse-futility-pruning]: https://www.chessprogramming.org/Reverse_Futility_Pruning
[late-move-pruning]: https://www.chessprogramming.org/Futility_Pruning#MoveCountBasedPruning
[transposition-table]: https://www.chessprogramming.org/Transposition_Table
[check-extensions]: https://www.chessprogramming.org/Check_extensions
[quiescence-search]: https://www.chessprogramming.org/Quiescence_Search
[mvv-lva]: https://www.chessprogramming.org/MVV-LVA
[killer-move]: https://www.chessprogramming.org/Killer_Heuristic
[history-tables]: https://www.chessprogramming.org/History_Heuristic
[tt-move]: https://www.chessprogramming.org/Hash_Move
[see]: https://www.chessprogramming.org/Static_Exchange_Evaluation
[material-counting]: https://www.chessprogramming.org/Material
[pst]: https://www.chessprogramming.org/Piece-Square_Tables
[nnue]: https://www.chessprogramming.org/Neural_Networks#NNUE

[lague]: https://www.youtube.com/watch?v=U4ogK0MIzqk
[vice]: https://www.youtube.com/watch?v=bGAfaepBco4&list=PLZ1QII7yudbc-Ky058TEaOstZHVbT-2hg
[cpw]: https://www.chessprogramming.org/Main_Page
[talk-chess]: https://talkchess.com/forum3/viewforum.php?f=7&sid=ffef1434f6a9dcb18141af3148d4b1ea
[carp]: https://github.com/dede1751/carp
[viri]: https://github.com/cosmobobak/viridithas
[viri-wiki]: https://github.com/cosmobobak/viridithas/blob/master/wiki.md
[rustic]: https://github.com/mvanthoor/rustic
[rustic-book]: https://rustic-chess.org/
[blunder]: https://github.com/algerbrex/blunder/
[stockfish]: https://stockfishchess.org/
