# Things to try

## Time control
- [ ] Have a smarter time control than simply `total_time / 30`.
- [ ] Only check in every 4096 nodes, rather than _every_ single node. Not sure
      what the overhead is for checking the atomicbool every single time, but
      I'm sure there's _some_ overhead.

## Extensions
- [ ] Singular move extension: Extend depth by 1 if there is only one legal move
      (basically a free +1 to depth, because the branching factor is 1)

## Reductions
- [ ] Internal Iterative Reduction (when no TT move is found)

## Pruning
- [ ] Delta pruning
- [ ] More sophisticated null-move pruning, add Zugzwang check
- [ ] SEE pruning
- [ ] Razoring

## Move ordering
- [ ] Revisit history scores (subtract scores for moves that fail-low/ didn't fail-high)
- [ ] Incorporate History scores in LMR values
- [ ] Counter moves?
- [ ] Capture history?

## Evaluation
- [ ] King safety terms
- [ ] Connected rooks
- [ ] Bishop pairs
- [ ] Mobility
- [ ] Parameter tuning
- [ ] Add pawn hash table?

## Misc
- [ ] Tighten integer types and table entry sizes to the absolute minimum
