# First release :tada: 

There's still a long way to go, but Simbelmyne, both as an engine and as a codebase, is in a place I'm happy with.

Haven't placed it in any tournaments so far, so can only give an approximate rating, but it seems to play at around a 2000 Elo level.

A quick overview of what's currently included:
### Search
#### Move generation
- [x] [Legal move generation][legal-moves]
- [x] [Bitboard representation][bitboards]
- [x] Piece movement lookup tables

#### Pruning
- [x]  [Alpha-beta pruning][alpha-beta]
- [x]  [Null-move pruning][null-move]
- [x] [Transposition table][transposition-table]

#### Extensions
- [x] [Check extensions][check-extensions]
- [x] [Quiescence search][quiescence-search]

#### Move ordering
- [x] [MVV-LVA move ordering][mvv-lva]
- [x] [Killer move ordering][killer-move]
- [x] [History tables][history-tables]
- [x] [Hash move][tt-move]
- [ ] [Static exchange evaluation][see]

### Evaluation
- [x] [Material counting][material-counting]
- [x] [Piece-square tables][pst]

Currently only linux binaries are available, but will look into cross-compilation when I get a chance. If you're not on a linux platform, then your best bet for the moment would be to simply compile the source code.

[negamax]: https://en.wikipedial.com/wiki/Negamax
[legal-moves]: https://www.chessprogramming.org/Move_Generation#Legal
[bitboards]: https://www.chessprogramming.org/Bitboards
[magic-bitboards]: https://www.chessprogramming.org/Magic_Bitboards
[alpha-beta]: https://www.chessprogramming.org/Alpha-Beta
[null-move]: https://www.chessprogramming.org/Null_Move_Pruning
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
