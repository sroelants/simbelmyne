### What's new
This version mostly focuses on search tweaks. Most every tweak I could get to
gain has been included, so I imagine we're getting into diminishing returns and
~5 Elo patches.

Simbelmyne 1.8 is around 180 Elo stronger than 1.7 in STC self-play.
A small gauntlet places Simbelmyne at around 3050 Blitz 🎉

```
   # PLAYER                     : RATING    POINTS  PLAYED    (%)
   1 Avalanche 1.3              : 3085.0     322.0     727   44.3%
   2 Princhess 0.15             : 3073.0     330.5     726   45.5%
   3 Akimbo 0.5                 : 3058.0     405.5     728   55.7%
   4 Simbelmyne 1.8.0           : 3052.5    3221.5    5811   55.4%
   5 Koivisto 3.0               : 3027.0     328.0     726   45.2%
   6 Nalwald 16                 : 3001.0     389.5     726   53.7%
   7 Viridithas 3.0             : 2985.0     296.0     726   40.8%
   8 Black Marlin 2.0           : 2938.0     265.0     726   36.5%
   9 Polaris 1.7.0              : 2937.0     253.0     726   34.8%
```

### Added features

#### 🔍 Search
- Tweak aspiration window implementation (20.5+/-14.2) (#207)
- Reduce PV nodes less instead of non-pv nodes more (10.1+/-8.1) (#208)
- History table refactor (#209) (#210)
- Tweak quiet history penalties and ageing (35.9+/-20.1) (#211)
- 1 ply continuation history (21.6+/-14.6) (#215)
- Countermove table (13.4+/-10.4) (#216)
- Don't overwrite TT move in all-nodes (21.6+/-14.7) (#217)
- Include promotions in tacticals in (somewhat) staged movegen (#220)
- Use "improving" heuristic in LMP (18.8+/-13.4) (#221)
- Use "improving" in NMP (11.1+/-7.5) (#222)
- Reduce more/less according to history score (16.7+/-12.3) (#223)

#### ⚖️: Evaluation
- Make incremental eval term branchless (#204)

#### 🐛 Bugfixes
- Fix SEE to work with non-zero margins (#202)
- Only report "stop_early" when there's one legal move _in root_ (#205)
- Fix nodecounts and nps reporting (#206)
- Fix LMP move threshold (23.8+/-15.7) (#219)

#### :chess_pawn: Lichess Bot
- Tweak Dockerfile to fetch and build source from `main`
- Add deploy script for easy redeploys
- Tweak deploy script to GC old containers/images

#### Misc
- Use fixed-point multiplication trick instead of module based indexing of TT (12+/-8) (#203)

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
