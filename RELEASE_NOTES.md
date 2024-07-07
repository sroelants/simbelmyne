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

#### ⚖️: Evaluation
- Use packed eval (22.7 +/- 15.2) (#233)

#### 🐛 Bugfixes
- Stop lmr reduction from overflowing (#241)

#### :chess_pawn: Lichess Bot

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
