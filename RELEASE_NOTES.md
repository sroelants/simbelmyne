### What's new

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

#### ⚖️: Evaluation
- Make incremental eval term branchless (#204)

#### 🐛 Bugfixes
- Fix SEE to work with non-zero margins (#202)
- Only report "stop_early" when there's one legal move _in root_ (#205)
- Fix nodecounts and nps reporting (#206)
- Fix LMP move threshold (23.8+/-15.7) (#219)
- 

#### 💬 UCI 

#### :chess_pawn: Lichess Bot
- Tweak Dockerfile to fetch and build source from `main`
- Add deploy script for easy redeploys

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
