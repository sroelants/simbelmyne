# Changes in v1.0.0
- Added pawn structure terms to evaluation
Evaluation now takes into consideration:
  - [x] Passed pawns
  - [x] Isolated pawns
  - [x] Doubled pawns

+86 Elo at 10/0.1s

```
Score of Simbelmyne vs Simbelmyne v1.0.0: 270 - 148 - 82 [0.622]
...      Simbelmyne playing White: 166 - 36 - 48  [0.760] 250
...      Simbelmyne playing Black: 104 - 112 - 34  [0.484] 250
...      White vs Black: 278 - 140 - 82  [0.638] 500
Elo difference: 86.5 +/- 28.6, LOS: 100.0 %, DrawRatio: 16.4 %
500 of 500 games finished.
```

- Added magic bitboards
Use (fancy) magic bitboards throughout the move generation

+ 37 Elo at 10/0.1s
```
Score of Simbelmyne vs Simbelmyne main: 225 - 172 - 103 [0.553]
...      Simbelmyne playing White: 129 - 69 - 52  [0.620] 250
...      Simbelmyne playing Black: 96 - 103 - 51  [0.486] 250
...      White vs Black: 232 - 165 - 103  [0.567] 500
Elo difference: 37.0 +/- 27.3, LOS: 99.6 %, DrawRatio: 20.6 %
500 of 500 games finished.
```

# Against v1.0.0
Around +110 Elo against v1.0.0

```
Score of Simbelmyne vs Simbelmyne v1.0.0: 585 - 278 - 137 [0.653]
...      Simbelmyne playing White: 329 - 96 - 75  [0.733] 500
...      Simbelmyne playing Black: 256 - 182 - 62  [0.574] 500
...      White vs Black: 511 - 352 - 137  [0.580] 1000
Elo difference: 110.2 +/- 20.8, LOS: 100.0 %, DrawRatio: 13.7 %
1000 of 1000 games finished.
```
