### What's new

This release mostly focuses on improving the search. Simbelmyne prunes much more
accurately (and aggresively!), and the average branching factor across the
bench suite has dropped significantly.

A small gauntlet puts it at ~2900 Elo, around 233 Elo stronger than Simbelmyne
v1.6.0

```
Rank Name                          Elo     +/-   Games   Score    Draw 
   0 Simbelmyne main                79      13    2261   61.1%   25.4% 
   1 Nalwald 15 (2912)              24      28     452   53.4%   24.6% 
   2 Avalanche 1.1.0 (2902)         15      27     452   52.1%   27.2% 
   3 Cheers 0.3.1 (2838)           -68      27     452   40.4%   28.5% 
   4 Princhess 0.13 (2845)        -166      31     452   27.8%   21.9% 
   5 Simbelmyne v1.6.0 (2750?)    -233      31     453   20.8%   24.7% 
```

### Added features

#### 🔍 Search
- SPSA rqetune of search parameters (37.9 +/- 12)
- Tweak 
- Refactor futility pruning (13.1 +/- 8.8) (#198)
- Switch to log-based LMR (32.8 +/- 16.3)
- Reduce soft-time (32.5 +/- 15.6) (#200)
- Enable LMR in check (27.5 +/- 15.6)
- Stop searching if there's only one legal move
- Use LMP formula instead of LMP table (19.5 +/- 12.9)
- Switch to simpler FP formula (14 +/- 10.9) (#201)
- _Another_ SPSA re-tune (66.6 +/- 13.6)

#### ⚖️: Evaluation
- Fix contempt factor 

#### 🐛 Bugfixes
- Actually use soft-time limit instead of hard-time (9.3 +/- 9.4) (#197)
- Fix PVS (50.2 +/- 18) (#199)

#### 💬 UCI 

#### Misc

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
