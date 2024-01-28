# What's new in v1.3.0

### Added features
- Internal iterative deepening (12.2 +/- 17.7) (#132)
- Late move reductions (92 +/- 18) (#133)
- Improved (dynamic) time control (33.6 +/- 21.4) (#135)
- Dynamic null move pruning thresholds (34.1 +/- 22.7) (#136)
- Improve quiet history score (26.8 +/- 16.9) (#137)

### Estimated strength
Playing a gauntlet with a handful of engines with more established ratings
(ratings taken from the [CCRL Blitz ranking](https://www.computerchess.org.uk/ccrl/404/)), Simbelmyne v1.3 appears to score
~160 Elo higher than Simbelmyne v1.2 at short time controls, putting it at an 
approximate rating of 2500 Elo.

```
Blunder 7.1.0 (2461)           -1      26     500   49.9%   26.6% 
Avalanche 0.2.2 (2532)        -11      27     500   48.4%   20.8% 
Lynx 1.1.0 (2429)             -24      27     500   46.5%   23.8% 
Mess 0.1.0 (2420)             -45      25     500   43.6%   32.0% 
Simbelmyne v1.2.0 (2350)     -160      27     500   28.5%   31.0% 
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
