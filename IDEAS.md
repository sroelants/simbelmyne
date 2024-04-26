# Things to try

## Extensions
- [ ] Singular move extension: Extend depth by 1 if there is only one legal move
      (basically a free +1 to depth, because the branching factor is 1) (failed)
- [ ] Actual singular extensions

## Reductions
- [✓] Internal Iterative Reduction (when no TT move is found)
- [✓] Reduce good captures less
- [ ] Reduce when eval is far below alpha (~delta pruning)
- [ ] Reduce more when "improving"

## Pruning
- [✓] Delta pruning
- [✓] More sophisticated null-move pruning, add Zugzwang check
- [ ] SEE pruning
- [✓] Razoring

## Move ordering
- [✓] Revisit history scores (subtract scores for moves that fail-low/ didn't fail-high)
- [ ] Incorporate History scores in LMR values
- [ ] Counter moves?
- [ ] Continuation history
- [ ] Capture history?

## Evaluation
- [=] King safety terms
      - [✓] King zone
      - [✓] Pawn storm
- [≡] Pawns
      - [✓] Passed pawns
      - [✓] Doubled Pawns
      - [✓] Isolated pawns
      - [✓] Connected pawns
      - [✓] Phalanx pawns
      - [ ] Backward pawns
      - [✓] Passer distance to friendly king
      - [✓] Passer distance to enemy king
      - [✓] Pawn storm
      - [✓] Pawn shield
      - [✓] Pawn threats
- [≡] Knights
      - [✓] Knight Mobility
      - [✓] Knight outposts
      - [ ] Knight behind (friendly) pawn
      - [✓] Knight threats
- [≡] Bishops
      - [✓] Bishop mobility
      - [✓] Bishop pair
      - [✓] Bishop outposts
      - [ ] Bishop behind (friendly) pawn
      - [✓] Bishop threats
- [=] Rooks
      - [✓] Rook mobility
      - [✓] Rooks on open file
      - [✓] Rooks on semi-open file
      - [✓] Connected rooks (failed) (on the 1st rank?)
      - [✓] Rook on the 7th
      - [ ] Doubled rooks (on a (semi-) open file)
      - [ ] Rook behind a queen
      - [ ] Rook behind a passed pawn
      - [✓] Rook threats
- [≡] Queens
      - [✓] Mobility
      - [✓] Queen on 7th
      - [✓] Queen on open file
      - [✓] Queen on semi-open file
      - [✓] Threats
      - [ ] Discovered attacks
- [=] Kings
      - [✓] Virtual mobility
      - [✓] King zone attacks
- [✓] Pinned pieces (part of mobility)
- [ ] Hanging pieces (failed)
- [ ] Add pawn hash table?
- [✓] Mobility
- [✓] Parameter tuning
- [ ] Scale down drawish positions (failed)

## Misc
- [✓] Add back in contempt factor
- [✓] Tighten integer types and table entry sizes to the absolute minimum
- [✓] Store checkers bitboards on board
- [✓] Report mate score
- [ ] Mate in N mode
- [ ] Rewrite tuner to be less boilerplatey (or, at least, be more "feature
      based")
