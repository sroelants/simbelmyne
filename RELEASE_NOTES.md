### What's new

### Added features

#### Evaluation

- 🔧 Eval refactor
- 🔧 Split eval up into incremental and non-incremental
- ⚖️ Include pawn attacks and pin masks in mobility considerations
- ⚖️ Add evaluation term for king tropism wrt. friendly/enemy passed pawns
- ⚖️ Add evaluation term for connected rooks
- ⚖️ Add evaluation term for rooks on semi-open file
- ⚖️ Add evaluation term for major piece on 7th rank
- ⚖️ Add evaluation term for queen on (semi-) open file
- ⚖️: Add evaluation term for threats (pawn on minor/rook/queen, minor on
  rook/queen, rook on queen)
- ⚖️ Add evaluation term for rook and bishop outposts
- ⚖️ Add contempt factor to reduce number of early draws
- ⚖️ Add Tempo bonus

#### Bugfixes
- 🐛 Fix PV reporting, finally
- 🐞 Don't double-count leaf nodes in negamax _and_ quiescence search
- 🦋 Fix bug where we were wrapping around the board looking for phalanx pawns

#### UCI
- 💅 print properly formatted mate scores in UCI output (`score mate n`)
- 💅 Pretty print UCI messages when attached to a tty
- 💅 Add custom `eval` command that pretty-prints a breakdown of the evaluation

#### Misc
- 🔧 Pre-load a board position through a `-f`/`--fen` cli argument

See the respective PRs for self-play results where relevant

### Choosing a binary
This release comes with precompiled binaries for all major platforms. Because
the engine benefits tremendously from more modern CPU instruction sets, there 
are binaries compiled for major instruction sets, following the x86-64 
[microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) 
as a naming scheme. 

Realistically, on modern hardware (< 10 years old), you should be okay to use the 
binaries labeled `V3`. If the engine crashes within the first seconds, try `V2`, 
and so on.
