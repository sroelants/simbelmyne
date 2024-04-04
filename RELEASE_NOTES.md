### What's new

Mostly bugfixes/tweaks that came up as a result of the CCRL D10 and Blitz 
matches

### Added features
#### Bugs
- Stop printing a `>` prompt, it confuses some match runners (arena)
- Don't reset TT size after clearing the TT

### Choosing a binary
This release comes with precompiled binaries for all major platforms. Because
the engine benefits tremendously from more modern CPU instruction sets, there 
are binaries compiled for major instruction sets, following the x86-64 
[microarchitecture levels](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) 
as a naming scheme. 

Realistically, on modern hardware (< 10 years old), you should be okay to use the 
binaries labeled `V3`. If the engine crashes within the first seconds, try `V2`, 
and so on.
