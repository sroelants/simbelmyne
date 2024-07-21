# Things to try

## Search 
### Extensions
- [✓] Check extensions
- [✓] Singular extensions
- [✓] Double extensions
- [ ] Triple extensions
- [ ] Negative extensions
- [ ] Cutnode negative extensions

### Reductions
- [✓] Internal Iterative Reduction (when no TT move is found)
- [✓] Reduce bad captures more
- [ ] Reduce when eval is far below alpha (~delta pruning)
- [✓] History based reduction
- [✓] Use `improving` in LMR
- [ ] IIR when TT entry depth is much more shallow (e.g., `depth - tt_depth > 4`)
- [ ] Cutnode reductions

### Pruning
- [✓] Delta pruning
- [✓] More sophisticated null-move pruning, add Zugzwang check
- [✓] SEE pruning
- [ ] Razoring
- [ ] History based pruning
- [✓] Use `improving` in RFP
- [✓] Use `improving` in FP
- [ ] Use `improving` in razoring?
- [ ] Multicut

### Move ordering
- [✓] Revisit history scores (subtract scores for moves that fail-low/ didn't fail-high)
- [✓] Counter moves?
- [✓] Continuation history
- [✓] 2-ply continuation history
- [✓] 4-ply continuation history
- [✓] Capture history (replaces LVA)
- [✓] Threat-based history

### Time management
- [ ] Use less time when bestmove remains stable
- [ ] Use less time when eval remains stable
- [ ] Use more time when subtree has more nodes? Or less? I don't really get
      this one, tbh.

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
- [ ] Add pawn hash table? Not sure how valuable it is, when we're already doing
      incremental.
- [✓] Tempo
- [✓] Mobility
- [✓] Parameter tuning
- [ ] Scale down drawish positions (failed)
- [ ] Pieces protected by pawns
- [✓] Packed eval
- [ ] Safe checks: How many squares where I can check the king without being
      under attack myself

## Misc
- [✓] Use PEXT bitboards
- [ ] Tune SEE/MVV-LVA weights
- [✓] Add back in contempt factor
- [✓] Tighten integer types and table entry sizes to the absolute minimum
- [✓] Store checkers bitboards on board
- [✓] Report mate score
- [ ] Mate in N mode
- [ ] Rewrite tuner to be less boilerplatey (or, at least, be more "feature
      based")
- [ ] Profile a search to see where most time is being spent. Eval?
- [✓] Return early when only one legal move (failed)
- [=] Do better staging of movegen
      - [✓] (non-functional) Try TT first, before even generating moves
      - [✓] (non-functional) Generate captures and quiets separately
      - [ ] (functional) Maybe even hold off scoring quiets until we've yielded 
            killers and countermove? (Failed, not sure if we'll get this to
            work)
- [ ] Use TT score as a tighter eval
- [✓] Don't replace TT Move with a fail-low (also, should we even be using
      fail-low bestmove for _anything_ at all?)
- [✓] Clear killer moves for the next ply in each node
- [ ] Don't store killers during null-search
- [✓] Yield killers in a fifo way (easy, since we "rotate" the moves out)
- [ ] Have "short moves" and "long moves", where the long move includes extra
      information (like the moved piece), so we can index all of our history 
      table using long moves instead. (This essentially becomes our `HistIndex`.)
- [=] Performance tweaks in hot loops:
      - [✓] Transmute between enums and integers, instead of lookups
      - [✓] forego bounds checks
      - [ ] unchecked unwraps?
- [ ] Generate check evasions in QSearch? (As in, when in check, use _all_ legal
      moves. Feels dicey)
- [ ] Don't clear countermove history between iteration depths (what about
      killers?)
      * I shouldn't need to clear killers anyway, right? Since I clear in every
        node?
      * I can keep countermoves without any issue
- [ ] Use latest killers/countermoves (by fetching them straight from `history`
      inside `score_quiets`

## Small fry (needs longer sprt, but looks promising)
- [ ] Only do full pvs search on first move _in PV node_ (failed)
- [ ] Don't do any pruning when mated
- [ ] Clamp king attacks to 11 (don't use bogus weights)

## Cleanup/refactor goals
- [✓] Write a derive macro that generates UCI options for `SearchParams` 
- [✓] Figure out a (sane) way to tune MVV/SEE weights
- [✓] Refactor (cont)hist to be a little saner
- [ ] Figure out a way to clean up eval tuning (yet another proc macro?).
- [ ] WDL eval scaling

## Add as tunable parameters
- [✓] MVV/LVA weights
- [ ] SEE weights (should these be the same?) (hard to do for now, SEE is part
      of `simbelmyne_chess`
- [✓] History bonus/malus parameters
- [✓] LMR History divisor
- [✓] IIR depth
- [✓] IIR reduction
- [✓] Time management parameters

## Bugfixes
- [ ] Does history reductions even work with my killer/countermove bonuses?
      (like, does it effectively kill the reduction, because 
      `1000000 / HIST_DIVISOR` is still quite a lot? Ideally, we'd just use the
      hist score. And even more ideally, we'd not even do history pruning for
      refutation moves...
