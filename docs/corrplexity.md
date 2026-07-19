# Corrplexity

By looking at the size of the Corrhist correction, we have a measure of how
"complex" the position is.

We can use this measure to then influence the amount of pruning/reduction we
do elsewhere in the search: LMR, RFP, etc etc

## Simplest approach: `static_eval - raw_eval`

The simplest approach just looks at the total correction between all the
different corrhists. One the one hand, this is the more "relevant" measure. On
the other hand, it might miss the fact that one or several corrhists are giving
a serious signal, but that signal is washed out by the other corrhists.

```rust
// inside lmr
reduction -= ((static_eval - raw_eval) > lmr_corrplexity_margin()) as i16;
```

### Tests

```
| Margin |           SPRT |                                    Test |
--------------------------------------------------------------------
|    120 |  -7.90 +- 29.63 | https://chess.samroelants.com/test/943/ |
|    120 |  -3.50 +-  6.28 | https://chess.samroelants.com/test/944/ |
|    120 | -14.69 +- 14.15 | https://chess.samroelants.com/test/989/ |
|    100 |   1.20 +-  2.87 | https://chess.samroelants.com/test/942/ |
|     90 |  -9.80 +-  6.79 | https://chess.samroelants.com/test/988/ |
|     80 |  -3.38 +-  4.42 | https://chess.samroelants.com/test/941/ |
|     60 |  -1.27 +-  4.33 | https://chess.samroelants.com/test/990/ |
|     40 |  -4.51 +-  9.83 | https://chess.samroelants.com/test/992/ |
```

## Root-mean-square

One way of taking into account the individual contributions more is by looking
at the RMS value of all the corrhists. That way, the different corrections can't
simply wash out the corrplexity value.

Cons: slightly more work to compute this value. Need to look up all of the
corrhist values a second time

### Tests

```
| Margin |           SPRT |                                    Test |
--------------------------------------------------------------------
|    150 | -6.97 +-  8.99 | https://chess.samroelants.com/test/946/ |
|    140 | -4.88 +-  5.63 | https://chess.samroelants.com/test/955/ |
|    130 | -0.94 +-  3.70 | https://chess.samroelants.com/test/954/ |
|    120 | -3.11 +-  6.26 | https://chess.samroelants.com/test/945/ |
|    100 | -6.92 +- 13.11 | https://chess.samroelants.com/test/951/ |
```

## Max?

If the RMS is the L2 norm of the corrhists, then we might also look at the
L∞ norm. This amounts to simply looking at the maximum correction.

This feels a little more brittle than the L2 norm, though: if all the corrhists
seem to agree there isn't much correction needed, but then one corrhist ends
up being huge, then

1. Maybe we're overestimating the complexity
2. We're pretty close to the situation where we simply use the total delta.

### Tests

```
| Margin |           SPRT |                                    Test |
--------------------------------------------------------------------
|     30 |  -9.04 +- 6.48 | https://chess.samroelants.com/test/961/ |
```
