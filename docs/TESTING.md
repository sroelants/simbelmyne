# Self-play testing

### `chess` refactor/cleanup
Pretty cool to see a massive inrease in playing strength after the `chess` 
refactor.

_Very_ curious to find out why the asymmetry between playing as White or Black,
though...

Why are we significantly stronger than the previous version when playing as
white, but literally _exactly_ as strong when playing as black? Is there some
bug/mechanism that _significantly_ favors Playing as white?

```
Score of Simbelmyne (Post cleanup) vs Simbelmyne (pre cleanup): 477 - 273 - 250 [0.602]
...      Simbelmyne (Post cleanup) playing White: 291 - 83 - 126  [0.708] 500
...      Simbelmyne (Post cleanup) playing Black: 186 - 190 - 124  [0.496] 500
...      White vs Black: 481 - 269 - 250  [0.606] 1000
Elo difference: 71.9 +/- 18.9, LOS: 100.0 %, DrawRatio: 25.0 %
1000 of 1000 games finished.
```

Out of sheer curiosity: I get a similar imbalance when playing Blunder 3, but
the other way around:

```
Score of Simbelmyne (Post cleanup) vs Blunder 3.0 (~1782): 157 - 305 - 38 [0.352]
...      Simbelmyne (Post cleanup) playing White: 37 - 209 - 4  [0.156] 250
...      Simbelmyne (Post cleanup) playing Black: 120 - 96 - 34  [0.548] 250
...      White vs Black: 133 - 329 - 38  [0.304] 500
Elo difference: -106.0 +/- 30.6, LOS: 0.0 %, DrawRatio: 7.6 %
500 of 500 games finished.
```

Crazy, when I try playing Blunder 3 against the old Simbelmyne (before the
refactor), it plays evenly as both sides, even to the point of scoring slightly
_better_ on average...
```
Score of Blunder 3.0 (~1782) vs Simbelmyne (pre cleanup): 270 - 169 - 61 [0.601]
...      Blunder 3.0 (~1782) playing White: 136 - 85 - 29  [0.602] 250
...      Blunder 3.0 (~1782) playing Black: 134 - 84 - 32  [0.600] 250
...      White vs Black: 220 - 219 - 61  [0.501] 500
Elo difference: 71.2 +/- 29.1, LOS: 100.0 %, DrawRatio: 12.2 %
500 of 500 games finished.
```

So, let me get this straight:
Against the old Simbelmyne, I play vastly better as _White_.
Against blunder, I play vastly better as _Black_.
So. Weird.

Against Stockfish (~1900), things still look pretty asymmetric, playing
significantly weaker as Black.
```
Score of Simbelmyne (Post cleanup) vs Stockfish (1900): 222 - 256 - 22 [0.466]
...      Simbelmyne (Post cleanup) playing White: 124 - 110 - 16  [0.528] 250
...      Simbelmyne (Post cleanup) playing Black: 98 - 146 - 6  [0.404] 250
...      White vs Black: 270 - 208 - 22  [0.562] 500
Elo difference: -23.7 +/- 29.9, LOS: 6.0 %, DrawRatio: 4.4 %
500 of 500 games finished.
```

Really not sure what to make of this

And the same with Rustic: Significantly weaker when playing as _Black_.
```
Score of Rustic alpha 1.1 (1695) vs Simbelmyne (Post cleanup): 60 - 85 - 24 [0.426]
...      Rustic alpha 1.1 (1695) playing White: 48 - 18 - 19  [0.676] 85
...      Rustic alpha 1.1 (1695) playing Black: 12 - 67 - 5  [0.173] 84
...      White vs Black: 115 - 30 - 24  [0.751] 169
Elo difference: -51.8 +/- 49.2, LOS: 1.9 %, DrawRatio: 14.2 %
169 of 500 games finished.
```

Weird. Against Rustic, old Simbelmyne also seems to be vastly better as **WHITE**. 
Man, this has me _stumped_.

Old simbelmyne playing Rustic:
```
Score of Simbelmyne (pre cleanup) vs Rustic alpha 1.1 (1695): 288 - 163 - 49 [0.625]
...      Simbelmyne (pre cleanup) playing White: 200 - 25 - 25  [0.850] 250
...      Simbelmyne (pre cleanup) playing Black: 88 - 138 - 24  [0.400] 250
...      White vs Black: 338 - 113 - 49  [0.725] 500
Elo difference: 88.7 +/- 29.8, LOS: 100.0 %, DrawRatio: 9.8 %
500 of 500 games finished.
```


So, currently:
Against old Simbelmyne: Stronger as **WHITE**
Against Stockfish:      Stronger as **WHITE**
Against Rustic:         Stronger as **WHITE**
Against Blunder 3:      Stronger as **BLACK**


# Comparing Main (with typo's fixed
All of these are running off `main`, with the `Score::flipped` typo fixed.

## Recomputing from scratch
### Rustic
The trend seems clear enough:
Destroy Rustic when playing as white, but getting forced into a draw pretty
much exactly 50% of the time. Hardly any losses, though, so I wonder if there's 
just an issue with the draw detection that's interacting with this bug in a 
weird way.

```
Score of Simbelmyne vs Rustic alpha 1.1 (1695): 59 - 5 - 20 [0.821]
...      Simbelmyne playing White: 38 - 4 - 0  [0.905] 42
...      Simbelmyne playing Black: 21 - 1 - 20  [0.738] 42
...      White vs Black: 39 - 25 - 20  [0.583] 84
Elo difference: 265.1 +/- 77.4, LOS: 100.0 %, DrawRatio: 23.8 %
84 of 500 games finished.
```

### Blunder
Still win consistently as both sides, but _a lot more_ as white.
Lots of draws when playing as black

```
Score of Simbelmyne vs Blunder 3.0 (~1782): 69 - 15 - 16 [0.770]
...      Simbelmyne playing White: 47 - 2 - 1  [0.950] 50
...      Simbelmyne playing Black: 22 - 13 - 15  [0.590] 50
...      White vs Black: 60 - 24 - 16  [0.680] 100
Elo difference: 209.9 +/- 73.1, LOS: 100.0 %, DrawRatio: 16.0 %
100 of 500 games finished.
```
### Stockfish

### Self
Fascinating. A _lot_ of draws, which I guess isn't super unexpected. But a
_massive_ asymmetry between black and white. Like, wow.
```
Score of Simbelmyne vs Simbelmyne 2: 28 - 26 - 46 [0.510]
...      Simbelmyne playing White: 23 - 6 - 21  [0.670] 50
...      Simbelmyne playing Black: 5 - 20 - 25  [0.350] 50
...      White vs Black: 43 - 11 - 46  [0.660] 100
Elo difference: 6.9 +/- 50.3, LOS: 60.7 %, DrawRatio: 46.0 %
100 of 500 games finished.
```

## Incrementally updating
### Rustic
Similar to blunder below: We seem to be running into way less draws, but almost 
all of those draws seem to be converted into losses instead. Not sure which one
is actually better. 😬

I was operating under the assumption that I was being forced into a draw, but
maybe it's the other way around: maybe I'm managing to force a draw out of an
otherwise lost position?

The asymmetry is still there, but I'm pretty stoked about the 69 win streak when
playing as white!

```
Score of Simbelmyne vs Rustic alpha 1.1 (1695): 117 - 16 - 4 [0.869]
...      Simbelmyne playing White: 69 - 0 - 0  [1.000] 69
...      Simbelmyne playing Black: 48 - 16 - 4  [0.735] 68
...      White vs Black: 85 - 48 - 4  [0.635] 137
Elo difference: 328.1 +/- 87.9, LOS: 100.0 %, DrawRatio: 2.9 %
137 of 500 games finished.
```

### Blunder
Significantly more losses as black, but also significantly less _draws_.
I wonder what that is about... I think I saw something similar against Rustic 
as well... So, recomputing from scratch doesn't really affect the best case
(when we're playing as white), but _strongly_ affects the behavior as black.
Recomputing the score from scratch seems to lead to a lot more draws.

```
Score of Simbelmyne vs Blunder 3.0 (~1782): 76 - 18 - 7 [0.787]
...      Simbelmyne playing White: 46 - 3 - 1  [0.930] 50
...      Simbelmyne playing Black: 30 - 15 - 6  [0.647] 51
...      White vs Black: 61 - 33 - 7  [0.639] 101
Elo difference: 227.2 +/- 81.1, LOS: 100.0 %, DrawRatio: 6.9 %
101 of 500 games finished.
```

#Self
Statistics are a bit weird, but seemingly far less of an asymmetry?
```
Score of Simbelmyne vs Simbelmyne2: 25 - 34 - 41 [0.455]
...      Simbelmyne playing White: 12 - 13 - 25  [0.490] 50
...      Simbelmyne playing Black: 13 - 21 - 16  [0.420] 50
...      White vs Black: 33 - 26 - 41  [0.535] 100
Elo difference: -31.4 +/- 52.7, LOS: 12.1 %, DrawRatio: 41.0 %
100 of 500 games finished.
```

Running it for a bit longer makes the numbers a bit easier to read.

Lots of draws, a bit of asymmetry.
```
Score of Simbelmyne vs Simbelmyne2: 56 - 58 - 88 [0.495]
...      Simbelmyne playing White: 30 - 19 - 52  [0.554] 101
...      Simbelmyne playing Black: 26 - 39 - 36  [0.436] 101
...      White vs Black: 69 - 45 - 88  [0.559] 202
Elo difference: -3.4 +/- 36.1, LOS: 42.6 %, DrawRatio: 43.6 %
202 of 500 games finished.
```

---

Funny, it turns out that Rustic is even more asymmetric than I am...
Blunder is perfectly even, though...

# Comparing after evaluation refactor
## Incrementally updating
### Blunder
Crazy. I seem to be losing to Blunder consistently, **BUT**, I'm playing evenly
as black and white...
```
Score of Simbelmyne vs Blunder 3.0 (~1782): 33 - 68 - 0 [0.327]
...      Simbelmyne playing White: 17 - 34 - 0  [0.333] 51
...      Simbelmyne playing Black: 16 - 34 - 0  [0.320] 50
...      White vs Black: 51 - 50 - 0  [0.505] 101
Elo difference: -125.6 +/- 73.6, LOS: 0.0 %, DrawRatio: 0.0 %
101 of 500 games finished.
```

One weird clue should definitely be the fact that I'm getting back zero scores 
_a lot_ more often, at random times.

Right. We _SHOULD NOT_ be assigning mates or draws when we have = moves in
QSearch. That's like saying it's a stalemate because there's no captures on the
board.

Lol, that seems to have been it. Now we're playing Blunder evenly, and pretty 
much destroying him

Yeah, crazy, it looks like the PST was the offender! Changing that table back
to the old value and I'm getting the awful behavior as black again.

Super weird that it's not symmetric between black and white, honestly...


Sick! Let's do a bigger gauntlet then!

# The bigger gauntlet
```
Rank Name                          Elo     +/-   Games   Score    Draw 
   0 Simbelmyne                    179      13    3004   73.7%   11.0% 
   1 Blunder 5 (2123)              -22      28     500   46.9%   14.6% 
   2 Stockfish (1900)             -108      31     500   35.0%    2.8% 
   3 Rustic Alpha 3.0.0 (1913)    -144      29     500   30.4%   21.2% 
   4 Simbelmyne (pre cleanup)     -213      34     500   22.7%   10.2% 
   5 Zagreus 3 (1800)             -327      40     500   13.2%   10.8% 
   6 Blunder 4 (1757)             -389      47     504    9.6%    6.2% 
```

The Elo endorphins are real!
