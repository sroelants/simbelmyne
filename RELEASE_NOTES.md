### What's new
Version 1.10 is a pretty huge grab-bag of changes. Lots of re-factors that make 
the codebase easier to maintain, a lot of search improvements, and &mdash; for 
the first time in several versions &mdash; a big round of improvements to the
evaluation function. I think Simbelmyne is slowly but surely getting to the 
point where a NNUE-based evaluation becomes inevitable, but let's see how far 
we can take her, first!

No gauntlet just yet, so I don't have a very accurate guess at the playing
strength, but self play tests say [+215 STC](https://chess.samroelants.com/test/354/) and [+200 LTC](https://chess.samroelants.com/test/355/).

### Added features

#### 🔍 Search
- Add pawn based correction history (46+-13.62) (#261)
- Non-pawn based correction history (19.73+-8.64) (#298)
- Material based correction history (6.99+-4.69) (#300)
- Don't clear countermoves between searches (2.96+-2.36) (#263)
- Negative extensions (2.29+-1.59) (#262)
- Multicut (3.84+-3.07) (#264)
- Triple extensions (3.30+-2.64) (#265)
- Remove History aging (6.85+-6.05) (#268)
- Remove delta pruning (3.51+-4.55) (#283)


#### ⚖️: Evaluation
- Pawn cache table (18.57+-8.21) (#282)
- Bonus for knight/bishop shelters (5.26+-3.88) (#286)
- Bonus for safe check options (12.52+-6.66) (#284)
- Bonus for unsafe check options (15.32+-7.39) (#285)
- Penalty for bad bishops (16.14+-7.65) (#287)
- Square rule for passed pawns (7.40+-4.83) (#294)
- Endgame scaling (11.37+-6.17) (#296)
- Bonus for free/unhindered passed pawns (8.73+-5.39) (#295)
- Include attacked stop square in free passed pawn condition (6.36+-4.43) (#299)
- Bonus for protected passed pawns (3.73+-2.97) (#297)

#### 🐛 Bugfixes
- Pass conthist table to move picker by value, instead of copy (11.69+-7.72)
  (#256)

#### ⌛ Time management
- Better hard/soft time limits (22.08+-9.26) (#272)
- Node based scaling of soft time limit (9.24+-5.53) (#273)
- `best_move` stability scaling of soft time limit (9.83+-5.80) (#270)

#### Misc
- Big refactor of the SPSA tuning architecture (#254, #255)
- Big refactor of the search history (#257)
- Big eval refactor (#274, #275, #278, #280, #281, #288, #292)
- Normalized eval using a logistic WDL model (#259)
- Shrink TT entries (16.38+-7.96) (#260)
- Use TT eval in Quiescence search (4.81+-3.60) (#267)
- Make `mv.is_quiet()` return true for all non-tacticals (24.67+-10.53) (#249)

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
