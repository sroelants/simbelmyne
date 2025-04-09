# Things to try

## Search 
### Extensions
- [x] Check extensions
- [x] Singular extensions
- [x] Double extensions
- [x] Triple extensions
- [x] Negative extensions
- [ ] Cutnode negative extensions

### Reductions
- [x] Internal Iterative Reduction (when no TT move is found)
- [x] Reduce bad captures more
- [x] History based reduction
- [x] Use `improving` in LMR
- [x] Cutnode LMR reductions
- [ ] ttpv
- [ ] Reduce when eval is far below alpha (~delta pruning)
- [ ] ~~IIR when TT entry depth is much more shallow (e.g., `depth - tt_depth > 4`)~~
- [ ] ~IIR more in cutnodes~

### Pruning
- [x] Delta pruning
- [x] More sophisticated null-move pruning, add Zugzwang check
- [x] SEE pruning
- [x] Use `improving` in RFP
- [x] Use `improving` in FP
- [x] Multicut
- [x] History based pruning
- [x] SEE prune quiet moves
- [x] Use history to determine bad tactical threshold
- [ ] Use `improving` in razoring?
- [ ] Use eval in LMP threshold
- [ ] Use capthist in see pruning threshold
- [ ] Use capthist in capture lmr
- [ ] ~QS LMP~
- [ ] ~Razoring~

### Move ordering
- [x] Revisit history scores (subtract scores for moves that fail-low/ didn't fail-high)
- [x] Counter moves?
- [x] Continuation history
- [x] 2-ply continuation history
- [x] 4-ply continuation history
- [x] Capture history (replaces LVA)
- [x] Threat-based history
- [ ] ~Threat-based capthist~
- [ ] Capture conthist?
- [ ] Capture countermoves?

### Time management
- [x] Use less time when bestmove remains stable
- [x] Use less time when eval remains stable
- [x] Use more time when subtree has more nodes? Or less? I don't really get
      this one, tbh.

### Corrhist
- [x] Pawn corrhist
- [x] Non-pawn corrhist
- [x] Material corrhist
- [x] Minor piece corrhist
- [ ] ~Major piece corrhist~
- [ ] ~Threat corrhist~
- [ ] ~Continuation corrhist~ (indexed by prev move/2 prev moves)
- [ ] Plug eval into corrhist, somehow?

## Evaluation
- King safety terms
  - [x] King zone
  - [x] Pawn storm
  - [ ] King threats
  - [ ] Disregard king zone attacks that are (doubly) protected by pawns
- Pawns
  - [x] Passed pawns
  - [x] Doubled Pawns
  - [x] Isolated pawns
  - [x] Connected pawns
  - [x] Phalanx pawns
  - [x] Passer distance to friendly king
  - [x] Passer distance to enemy king
  - [x] Pawn storm
  - [x] Pawn shield
  - [x] Pawn threats
  - [x] Square rule
  - [x] Unblocked passers
  - [x] Protected passers
  - [ ] ~Backward pawns~
- [≡] Knights
  - [x] Knight Mobility
  - [x] Knight outposts
  - [x] Knight threats
  - [x] Knight behind (friendly) pawn
  - [ ] Open/Closed position
- [✓] Bishops
  - [x] Bishop mobility
  - [x] Bishop pair
  - [x] Bishop outposts
  - [x] Bishop threats
  - [x] Bishop behind (friendly) pawn
  - [x] Bad bishop
- [=] Rooks
  - [x] Rook mobility
  - [x] Rooks on open file
  - [x] Rooks on semi-open file
  - [x] Connected rooks (failed) (on the 1st rank?)
  - [x] Rook on the 7th
  - [x] Rook threats
  - [ ] Doubled rooks (on a (semi-) open file)
  - [ ] Rook behind a queen
  - [ ] Rook behind a passed pawn
- [≡] Queens
  - [x] Mobility
  - [x] Queen on 7th
  - [x] Queen on open file
  - [x] Queen on semi-open file
  - [x] Threats
  - [ ] Discovered attacks
  - [ ] ~Queen threats~ (potential attacks on our queen)

- [-] Kings
  - [x] Virtual mobility
  - [x] King zone attacks
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
- [x] Pinned pieces (part of mobility)
- [x] Tempo
- [x] Mobility
- [x] Parameter tuning
- [x] Packed eval
- [x] Endgame scaling (look at stash/weiss)
      -> This requires modifications of the tuner. cf eth tuning paper
- [x] Safe checks: How many squares where I can check the king without being
      under attack myself
- [x] Unsafe checks (probably less important than safe checks, but might still
      be worth something?)
- [x] Add pawn hash table? Not sure how valuable it is, when we're already doing
      incremental.
- [ ] Pieces protected by pawns
- [ ] Horizontally mirrored psqts

## Misc
- [x] Use PEXT bitboards
- [x] Add back in contempt factor
- [x] Tighten integer types and table entry sizes to the absolute minimum
- [x] Store checkers bitboards on board
- [x] Report mate score
- [x] Return early when only one legal move (failed)
- [ ] Do better staging of movegen
  - [x] (non-functional) Try TT first, before even generating moves
  - [x] (non-functional) Generate captures and quiets separately
  - [ ] (functional) Maybe even hold off scoring quiets until we've yielded 
        killers and countermove? (Failed, not sure if we'll get this to
        work)
- [x] Don't replace TT Move with a fail-low (also, should we even be using
   x  fail-low bestmove for _anything_ at all?)
- [x] Clear killer moves for the next ply in each node
- [x] Yield killers in a fifo way (easy, since we "rotate" the moves out)
- [ ] Performance tweaks in hot loops:
  - [x] Transmute between enums and integers, instead of lookups
  - [x] forego bounds checks
  - [ ] unchecked unwraps?
- [x] Don't clear countermove history between iteration depths (what about
      killers?)
      * I shouldn't need to clear killers anyway, right? Since I clear in every
        node?
      * I can keep countermoves without any issue
- [ ] Don't store killers during null-search
- [ ] Generate check evasions in QSearch? (failed)
- [ ] Tune SEE/MVV-LVA weights
- [ ] Use latest killers/countermoves (by fetching them straight from `history`
      inside `score_quiets` (failed?)
- [ ] Rewrite tuner to be less boilerplatey (or, at least, be more "feature
      based")
- [ ] Profile a search to see where most time is being spent. Eval?
- [ ] Use TT score as a tighter eval

## Small fry (needs longer sprt, but looks promising)
- [ ] Only do full pvs search on first move _in PV node_ (failed)
- [ ] Don't do any pruning when mated
- [ ] Clamp king attacks to 11 (don't use bogus weights)

## Cleanup/refactor goals
- [x] Write a derive macro that generates UCI options for `SearchParams` 
- [x] Figure out a (sane) way to tune MVV/SEE weights
- [x] Refactor (cont)hist to be a little saner
- [x] WDL eval scaling
- [ ] Figure out a way to clean up eval tuning (yet another proc macro?).

## Add as tunable parameters
- [x] MVV/LVA weights
- [x] History bonus/malus parameters
- [x] LMR History divisor
- [x] IIR depth
- [x] IIR reduction
- [x] Time management parameters
- [ ] SEE weights (should these be the same?) (hard to do for now, SEE is part
      of `simbelmyne_chess`

## Fun, entirely unnecessary, features
- [x] SAN
- [x] Lazy-SMP
- [ ] DRFC
- [ ] Multi-pv
- [ ] EG Tablebases
- [ ] Mate in N mode

## Performance improvements
- [x] One giant `play_move`
- [x] Lazy SEE
- [ ] Fully staged movegen (failed. quiet scores bleeding into refutation ranges
      gains more than I get from staging the movegen)
- [ ] Pseudolegal movegen
- [ ] Lazy Evaluation updates
- [ ] One shared repetition history stack

## Bugfixes
- [x] Replace most `mv.is_quiet()` calls with `!mv.is_tactical()`? (or
      equivalent)
- [ ] Does history reductions even work with my killer/countermove bonuses?
      (like, does it effectively kill the reduction, because 
      `1000000 / HIST_DIVISOR` is still quite a lot? Ideally, we'd just use the
      hist score. And even more ideally, we'd not even do history pruning for
      refutation moves...
