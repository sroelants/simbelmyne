# Things to try

## Extensions
- [ ] Singular move extension: Extend depth by 1 if there is only one legal move
      (basically a free +1 to depth, because the branching factor is 1) (failed)
- [ ] Actual singular extensions

## Reductions
- [✓] Internal Iterative Reduction (when no TT move is found)
- [ ] Reduce good captures less
- [ ] Reduce when eval is far below alpha (~delta pruning)

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
- [✓] King safety terms (failed)
      - [✓] King zone
      - [✓] Pawn storm
- [ ] Connected rooks (failed)
- [ ] Pinned pieces (failed)
- [=] Pawn structure
      - [ ] Add pawn hash table?
      - [✓] Connected pawns (failed)
      - [✓] Pawn phalanx
      - [ ] Backward pawns
- [✓] Bishop pairs
- [✓] Mobility
- [✓] Parameter tuning
- [ ] Knight outposts
- [ ] King distance to passed pawns: protect our passers, attack enemy passers
- [ ] Threats (how many pieces are attacked by this piece? Distinguish
      minor/major threats?)

## Misc
- [✓] Tighten integer types and table entry sizes to the absolute minimum
- [ ] Store checkers bitboards on board
- [ ] Store checkers bitboards on board (or attacked pieces?)
- [ ] Report mate score
- [ ] Mate in N mode
