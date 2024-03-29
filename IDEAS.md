# Things to try

## Extensions
- [ ] Singular move extension: Extend depth by 1 if there is only one legal move
      (basically a free +1 to depth, because the branching factor is 1) (failed)
- [ ] Actual singular extensions

## Reductions
- [✓] Internal Iterative Reduction (when no TT move is found)

## Pruning
- [✓] Delta pruning
- [✓] More sophisticated null-move pruning, add Zugzwang check
- [ ] SEE pruning
- [ ] Razoring

## Move ordering
- [✓] Revisit history scores (subtract scores for moves that fail-low/ didn't fail-high)
- [ ] Incorporate History scores in LMR values
- [ ] Counter moves?
- [ ] Capture history?

## Evaluation
- [ ] King safety terms (failed)
- [ ] Connected rooks (failed)
- [ ] Pinned pieces (failed)
- [ ] Connected pawns
- [✓] Bishop pairs
- [✓] Mobility
- [✓] Parameter tuning
- [ ] Add pawn hash table?

## Misc
- [✓] Tighten integer types and table entry sizes to the absolute minimum
- [ ] Store checkers bitboards on board
- [ ] Store checkers bitboards on board (or attacked pieces?)
- [ ] Report mate score
- [ ] Mate in N mode
