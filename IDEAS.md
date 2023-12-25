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
- [ ] Late move reductions

## Pruning
- [ ] Futility pruning
- [ ] Extended Futility Pruning
- [ ] Late move pruning
- [ ] Delta pruning
- [ ] SEE pruning in QSearch

## Move ordering
- [ ] Static Exchange Evaluation
- [ ] Internal iterative deepening

## Evaluation
- [ ] King safety terms
- [ ] Mobility
- [ ] Parameter tuning

## Misc
- [ ] Share TT between searches (wrap in an `Arc` and pass it around?)
- [ ] Revisit PVS when we have better move ordering
