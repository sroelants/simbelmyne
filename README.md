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
| v1.5.1  | 2700     |             |           | 2708*     | 2702    |
| v1.6.0  | 2760     |             |           | 2796*     | 2769    |
| v1.7.0  | 2900     |             |           | 2913      | 2926    |
| v1.8.0  | 3050     |             |           |           | 3037    |
| v1.9.0  | 3100     |             |           | 3045*     | 3083    |
| v1.10.0 | 3200     | 3220        |           | 3195      | 3254    |

(* Provisional rating, not enough games played so the error bars are rather large.)
  
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
- Legal move generation
- Bitboard representation
- Magic bitboards
- PEXT bitboards (when supported)

#### Pruning, reductions, extensions
- Iterative deepening
- Aspiration windows
- Alpha-beta pruning
- Principal-variation search
- Check extensions
- Improving heuristic
- Transposition table
- Internal iterative reduction
- Reverse futility pruning
- Null-move pruning
- Futility pruning
- Static Exchange Evaluation pruning
- Late move pruning
- Late move reductions
- Quiescence search
- Singular extensions
- Double extensions
- Triple extensions
- Negative extensions
- Multicut

#### Move ordering
- Hash move
- MVV + Capture history
- Static exchange evaluation for splitting captures into good/bad
- Killer Moves
- Countermoves
- Threat-based quiet history
- 1 ply continuation history
- 2 ply continuation history
- 4 ply continuation history
- Pawn-based correction history
- Nonpawn-based correction history
- Material-based correction history

#### Time management
- Hard/soft time bounds
- Best-move stability based soft-time scaling
- Score-based stability based soft-time scaling
- best-move subtree based soft-time scaling

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

- Material counting
- Piece-square tables
- Pawn structure
  - Isolated pawns
  - Protected pawns
  - Phalanx pawns
- Passed pawns
  - Distance to friendly king
  - Distance to enemy king
  - Square rule
  - Protected passed pawns
  - Unhindered passed pawns
- Pawn structure cache
- Minor piece outposts
- Minor piece shelters
- Bishop pair
- Bad bishops
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
  - Safe checks
  - Unsafe checks
- Endgame scaling

## Acknowledgements
Simbelmyne was inspired, and has drawn a lot from many different people,
resources and codebases, so it is only proper to give thanks where thanks are
due. Especially to the excellent people in the Engine Programming 
and Stockfish discords. Y'all are pretty good eggs.

[license-badge]: https://img.shields.io/github/license/sroelants/simbelmyne?style=for-the-badge&color=blue
[license-link]: https://github.com/sroelants/simbelmyne/blob/main/LICENSE

[build-badge]: https://img.shields.io/github/actions/workflow/status/sroelants/simbelmyne/tests.yml?style=for-the-badge
[build-link]: https://github.com/sroelants/simbelmyne/actions/workflows/tests.yml

[release-badge]: https://img.shields.io/github/v/release/sroelants/simbelmyne?style=for-the-badge&color=violet
[release-link]: https://github.com/sroelants/simbelmyne/releases/latest

[lichess-badge]:https://img.shields.io/badge/Play-latest-yellow?logo=lichess&style=for-the-badge
[lichess-link]: https://lichess.org/@/simbelmyne-bot

[arena]: http://www.playwitharena.de
[cutechess]: https://cutechess.com
[shredder]: https://www.shredderchess.com

[cargo]: https://doc.rust-lang.org/cargo
