### What's new
This version brings major improvements to the search (getting close to being a
somewhat mature search!), and has seen a decent amount of work go into speeding
up the underlying move generation.

Probably the bigger and more impactful change has been that I've set up a 
personal [OpenBench](https://github.com/AndyGrant/OpenBench) instance, powered
by 4 dedicated GCP workers. This has massively improvod the speed and ergonomics
of testing, and means I can finally test smaller improvements that would have
been impossible until now. 

You can find the OB instance at https://chess.samroelants.com, if you want a 
recorded history of my many failures, and sporadic successes!

Simbelmyne v1.9 is around 140 Elo stronger than v1.8 in STC self-play,
and a small gauntlet places it at around 3114 Elo.

```
   # PLAYER              : RATING    POINTS  PLAYED    (%)
   1 Avalanche 1.4       : 3214.0     248.5     425   58.5%
   2 Black Marlin 4.0    : 3206.0     244.0     336   72.6%
   3 Nalwald 17          : 3199.0     281.0     425   66.1%
   4 Stash 30            : 3162.0     270.5     424   63.8%
   5 Princhess 0.16      : 3153.0     261.5     424   61.7%
   6 Koivisto 4.0        : 3138.0     194.0     425   45.6%
   7 Patricia 2.0        : 3135.0     249.5     424   58.8%
   8 Simbelmyne 1.9      : 3114.4    1939.0    4156   46.7%
   9 Avalanche 1.3       : 3085.0     160.5     425   37.8%
  10 Patricia 1.0        : 3053.0     146.5     424   34.6%
  11 Polaris 1.8.1       : 3052.0     161.0     424   38.0%
```

### Added features

#### 🔍 Search
- PVS SEE Pruning (7.5 +/- 5.7) (#154)
- 2-ply conthist (11.89 +/- 6.84) (#236)
- Clear next ply's killers (10.90 +/- 6.45) (#239)
- Use `improving` in RFP (5.42 +/- 4.09) (#242)
- Use `improving` in FP (5.43 +/- 4.10) (#245)
- Add tactical history (19.86 +/- 9.10) (#244)
- Singular extensions (9.14 +/- 5.80) (#250)
- Double extensions (11.86 +/- 6.68) (#251)
- 4-ply conthist (5.40 +/- 4.07) (#252)
- Use threat-based history (6.70 +/- 4.73) (#253)

#### ⚖️: Evaluation
- Use packed eval (22.7 +/- 15.2) (#233)

#### 🐛 Bugfixes
- Stop lmr reduction from overflowing (#241)
- Clear conthist beteen games (#247)

#### Misc
- Movegen refactor (13.7 +/- 10.7) (#235)
- Speed improvements (19.7 +/- 13.7) (#234)
- Generate quiets lazily (#227)
- Lazily yield TT move before generating captures (22.8 +/- 15.2) (#225)

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
