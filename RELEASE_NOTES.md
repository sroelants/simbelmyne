# What's new in v1.4.0
Some decent improvements were made on the evaluation. Several king-safety terms 
were added, and Simbelmyne now has a built-in Gradient Descent tuner for tuning
evaluation parameters.

### Added features
#### Evaluation
- Bishop pair bonus
- Rook on open file bonus
- Texel tuner
- Piece mobility bonus
- Pawn shield bonus
- Virtual mobility

#### Search
- SPSA tuning of parameters
- Switch from Internal Iterative Deepening (IIR) to Internal Iterative Reduction
  (IIR)
- Use TT in QSearch

#### UCI
- Add Hash option for setting TT Size
- Add search parameters as UCI options, for SPSA tuning

#### Bugfixes
- Update rook eval when rook is removed.
- Clear PV at the start of Qsearch, so we don't propagate up illegal moves
- Fix integer overflows in RFP condition

#### Misc
- Make move generation 100% allocation-free
- Store static eval in TT entry
- Prefetch TT entries
- Normalize mate scores when storing in TT

### Estimated rating
Self play agains `v1.3.0` and against several other engines put the estimated
Elo gain at +-160, or around ~2650.

```
Score of Simbelmyne main vs Simbelmyne v1.3.0 (2500): 1535 - 441 - 524 [0.719]
...      Simbelmyne main playing White: 889 - 145 - 216  [0.798] 1250
...      Simbelmyne main playing Black: 646 - 296 - 308  [0.640] 1250
...      White vs Black: 1185 - 791 - 524  [0.579] 2500
Elo difference: 163.0 +/- 13.0, LOS: 100.0 %, DrawRatio: 21.0 %
2500 of 2500 games finished.
```

```
Rank Name                          Elo     +/-   Games   Score    Draw 
   0 Simbelmyne main                26       7    6000   53.8%   27.6% 
   1 Cheers 0.2 (2611)              48      19    1000   56.8%   24.0% 
   2 Leorik 2.0.2 (2536)             6      18    1000   50.8%   27.7% 
   3 Lynx 1.3 (2650)               -18      18    1000   47.4%   31.3% 
   4 Nalwald 1.9 (2625)            -31      18    1000   45.5%   30.0% 
   5 Blunder 7.6.0 (2619)          -58      18    1000   41.8%   30.9% 
   6 Avalanche 0.2.2 (2532)       -108      20    1000   34.9%   21.9% 

6000 of 6000 games finished.
```



### Choosing a binary
This release comes with precompiled binaries for all major platforms. Because
the engine benefits tremendously from more modern CPU instruction sets, there 
are binaries compiled for major instruction sets, following the x86-64 
[microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) 
as a naming scheme. 

Realistically, on modern hardware (< 10 years old), you should be okay to use the 
binaries labeled `V3`. If the engine crashes within the first seconds, try `V2`, 
and so on.
