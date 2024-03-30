# What's new in v1.4.0

### Added features
#### Eval
#### Search
- Delta pruning in Quiescence search (+16) [#163](https://github.com/sroelants/simbelmyne/pull/163)
- Switch from fail-hard to fail-soft (+20) [#165](https://github.com/sroelants/simbelmyne/pull/165)
- 
#### UCI
- Add custom `go perft <n>` command that prints divide perft results (cf. stockfish) [#164](https://github.com/sroelants/simbelmyne/pull/164)

### Choosing a binary
This release comes with precompiled binaries for all major platforms. Because
the engine benefits tremendously from more modern CPU instruction sets, there 
are binaries compiled for major instruction sets, following the x86-64 
[microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) 
as a naming scheme. 

Realistically, on modern hardware (< 10 years old), you should be okay to use the 
binaries labeled `V3`. If the engine crashes within the first seconds, try `V2`, 
and so on.
