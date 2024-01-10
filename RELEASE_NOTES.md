# What's new in v1.2.0

### Added features
- Aspiration window search (39 +/- 19)
- Reverse futility pruning (78 +/- 20)
- Futility pruning (13 +/- 20)
- Remove aggressive contempt factor (41 +/- 30)
- SEE-based move ordering of captures (43 +/- 18)
- Principal variation search (37 +/- 20)
- Reuse history tables between searches (23.3 +/- 19)
- Late move pruning (18 +/- 17)

### Estimated strength
Playing a gauntlet with a handful of engines with more established ratings
(ratings taken from the [CCRL Blitz ranking](https://www.computerchess.org.uk/ccrl/404/)), Simbelmyne v1.2 appears to score
~240 Elo higher than Simbelmyne v1.1, at an approximate rating of 2350 Elo.

```
Rank Name                          Elo     +/-   Games   Score    Draw 
   0 Simbelmyne                    131       8    7000   68.0%   17.2% 
   1 Blunder 7.1.0 (2461)          109      19    1000   65.2%   23.6% 
   2 GopherCheck 0.2.3 (2254)     -104      20    1000   35.4%   20.9% 
   3 Halcyon 1.0 (2203)           -142      21    1000   30.6%   17.8% 
   4 Blunder 6.1.0 (2207)         -156      21    1000   28.9%   15.0% 
   5 Zagreus 4.1 (2140)           -189      22    1000   25.1%   16.7% 
   6 Simbelmyne v1.1.0            -242      24    1000   19.9%   14.7% 
   7 Blunder 5 (2123)             -251      25    1000   19.1%   11.7% 
```

### Choosing a binary
This release comes with precompiled binaries for all major platforms. Because
the engine benefits tremendously from more modern CPU instruction sets, there 
are binaries compiled for major instruction sets, following the x86-64 
[microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) 
as a naming scheme. 

Realistically, on modern hardware (< 10 years old), you should be okay to use the 
binaries labeled `V3`. If the engine crashes within the first seconds,  try `V2`, 
and so on.
