# Things to try

## Search 
### Extensions
- [✓] Check extensions
- [✓] Singular extensions
- [✓] Double extensions
- [✓] Triple extensions
- [✓] Negative extensions
- [ ] Cutnode negative extensions

### Reductions
- [✓] Internal Iterative Reduction (when no TT move is found)
- [✓] Reduce bad captures more
- [✓] History based reduction
- [✓] Use `improving` in LMR
- [ ] Reduce when eval is far below alpha (~delta pruning)
- [ ] IIR when TT entry depth is much more shallow (e.g., `depth - tt_depth > 4`)
- [ ] IIR more in cutnodes
- [ ] Cutnode LMR reductions
- [ ] ttpv

### Pruning
- [✓] Delta pruning
- [✓] More sophisticated null-move pruning, add Zugzwang check
- [✓] SEE pruning
- [✓] Use `improving` in RFP
- [✓] Use `improving` in FP
- [✓] Multicut
- [ ] Razoring
- [ ] Use `improving` in razoring?
- [ ] History based pruning
- [ ] Use eval in LMP threshold
- [ ] QS LMP
- [ ] SEE prune quiet moves
- [ ] Use capthist in see pruning threshold
- [ ] Use capthist in capture lmr
- [ ] Use history to determine bad tactical threshold
      Maybe even forego SEE if history is really bad?

### Move ordering
- [✓] Revisit history scores (subtract scores for moves that fail-low/ didn't fail-high)
- [✓] Counter moves?
- [✓] Continuation history
- [✓] 2-ply continuation history
- [✓] 4-ply continuation history
- [✓] Capture history (replaces LVA)
- [✓] Threat-based history
- [ ] Threat-based capthist
- [ ] Capture conthist?
- [ ] Capture countermoves?

### Time management
- [✓] Use less time when bestmove remains stable
- [✓] Use less time when eval remains stable
- [✓] Use more time when subtree has more nodes? Or less? I don't really get
      this one, tbh.

### Corrhist
- [✓] Pawn corrhist
- [✓] Non-pawn corrhist
- [✓] Material corrhist
- [ ] Major piece corrhist
- [✓] Minor piece corrhist
- [ ] Continuation corrhist (indexed by prev move/2 prev moves
- [ ] Threat corrhist
- [ ] Plug eval into corrhist, somehow?

## Evaluation
- [=] King safety terms
      - [✓] King zone
      - [✓] Pawn storm
      - [ ] King threats
      - [ ] Disregard king zone attacks that are (doubly) protected by pawns
- [≡] Pawns
      - [✓] Passed pawns
      - [✓] Doubled Pawns
      - [✓] Isolated pawns
      - [✓] Connected pawns
      - [✓] Phalanx pawns
      - [✓] Passer distance to friendly king
      - [✓] Passer distance to enemy king
      - [✓] Pawn storm
      - [✓] Pawn shield
      - [✓] Pawn threats
      - [✓] Square rule
      - [✓] Unblocked passers
      - [✓] Protected passers
      - [ ] Backward pawns
- [≡] Knights
      - [✓] Knight Mobility
      - [✓] Knight outposts
      - [✓] Knight threats
      - [✓] Knight behind (friendly) pawn
      - [ ] Open/Closed position
- [✓] Bishops
      - [✓] Bishop mobility
      - [✓] Bishop pair
      - [✓] Bishop outposts
      - [✓] Bishop threats
      - [✓] Bishop behind (friendly) pawn
      - [✓] Bad bishop
- [=] Rooks
      - [✓] Rook mobility
      - [✓] Rooks on open file
      - [✓] Rooks on semi-open file
      - [✓] Connected rooks (failed) (on the 1st rank?)
      - [✓] Rook on the 7th
      - [✓] Rook threats
      - [ ] Doubled rooks (on a (semi-) open file)
      - [ ] Rook behind a queen
      - [ ] Rook behind a passed pawn
- [≡] Queens
      - [✓] Mobility
      - [✓] Queen on 7th
      - [✓] Queen on open file
      - [✓] Queen on semi-open file
      - [✓] Threats
      - [ ] Discovered attacks
- [-] Kings
      - [✓] Virtual mobility
      - [✓] King zone attacks
      - [ ] King zone attackers
      - [ ] Per-piece king zone attack bonus?
      - [ ] Exclude (2xPawn) defended squares?
      - [ ] Bigger king zone?
      - [ ] King threats
- [ ] Threats & mobility
      - [ ] Incorporate pins (failed for threats)
      - [ ] Xrays?
      - [ ] Hanging pieces (failed)
      - [ ] Index threats scores by defended/undefended
- [✓] Pinned pieces (part of mobility)
- [✓] Tempo
- [✓] Mobility
- [✓] Parameter tuning
- [✓] Packed eval
- [✓] Endgame scaling (look at stash/weiss)
      -> This requires modifications of the tuner. cf eth tuning paper
- [✓] Safe checks: How many squares where I can check the king without being
      under attack myself
- [✓] Unsafe checks (probably less important than safe checks, but might still
      be worth something?)
- [✓] Add pawn hash table? Not sure how valuable it is, when we're already doing
      incremental.
- [ ] Pieces protected by pawns
- [ ] Horizontally mirrored psqts

## Misc
- [✓] Use PEXT bitboards
- [✓] Add back in contempt factor
- [✓] Tighten integer types and table entry sizes to the absolute minimum
- [✓] Store checkers bitboards on board
- [✓] Report mate score
- [✓] Return early when only one legal move (failed)
- [=] Do better staging of movegen
      - [✓] (non-functional) Try TT first, before even generating moves
      - [✓] (non-functional) Generate captures and quiets separately
      - [ ] (functional) Maybe even hold off scoring quiets until we've yielded 
            killers and countermove? (Failed, not sure if we'll get this to
            work)
- [✓] Don't replace TT Move with a fail-low (also, should we even be using
      fail-low bestmove for _anything_ at all?)
- [✓] Clear killer moves for the next ply in each node
- [✓] Yield killers in a fifo way (easy, since we "rotate" the moves out)
- [=] Performance tweaks in hot loops:
      - [✓] Transmute between enums and integers, instead of lookups
      - [✓] forego bounds checks
      - [ ] unchecked unwraps?
- [✓] Don't clear countermove history between iteration depths (what about
      killers?)
      * I shouldn't need to clear killers anyway, right? Since I clear in every
        node?
      * I can keep countermoves without any issue
- [ ] Don't store killers during null-search
- [ ] Generate check evasions in QSearch? (failed)
- [ ] Tune SEE/MVV-LVA weights
- [ ] Use latest killers/countermoves (by fetching them straight from `history`
      inside `score_quiets` (failed?)
- [ ] Mate in N mode
- [ ] Rewrite tuner to be less boilerplatey (or, at least, be more "feature
      based")
- [ ] Profile a search to see where most time is being spent. Eval?
- [ ] Use TT score as a tighter eval

## Small fry (needs longer sprt, but looks promising)
- [ ] Only do full pvs search on first move _in PV node_ (failed)
- [ ] Don't do any pruning when mated
- [ ] Clamp king attacks to 11 (don't use bogus weights)

## Cleanup/refactor goals
- [✓] Write a derive macro that generates UCI options for `SearchParams` 
- [✓] Figure out a (sane) way to tune MVV/SEE weights
- [✓] Refactor (cont)hist to be a little saner
- [✓] WDL eval scaling
- [ ] Figure out a way to clean up eval tuning (yet another proc macro?).

## Add as tunable parameters
- [✓] MVV/LVA weights
- [✓] History bonus/malus parameters
- [✓] LMR History divisor
- [✓] IIR depth
- [✓] IIR reduction
- [✓] Time management parameters
- [ ] SEE weights (should these be the same?) (hard to do for now, SEE is part
      of `simbelmyne_chess`

## Fun, entirely unnecessary, features
- [✓] SAN
- [✓] Lazy-SMP
- [ ] DRFC
- [ ] Multi-pv
- [ ] EG Tablebases

## Performance improvements
- [✓] One giant `play_move`
- [✓] Lazy SEE
- [ ] Fully staged movegen (failed. quiet scores bleeding into refutation ranges
      gains more than I get from staging the movegen)
- [ ] Pseudolegal movegen
- [ ] Lazy Evaluation updates
- [ ] One shared repetition history stack

## Bugfixes
- [✓] Replace most `mv.is_quiet()` calls with `!mv.is_tactical()`? (or
      equivalent)
- [ ] Does history reductions even work with my killer/countermove bonuses?
      (like, does it effectively kill the reduction, because 
      `1000000 / HIST_DIVISOR` is still quite a lot? Ideally, we'd just use the
      hist score. And even more ideally, we'd not even do history pruning for
      refutation moves...
