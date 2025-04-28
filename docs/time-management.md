# Time management

There's a couple of questions that come up when making your engine play another
engine, or human, at a particular time control.

1. How long should the engine search any particular move for?
2. Should the engine search equally long for every move? Or should it take more
   time in the opening/endgame?
3. Even if I figure out how much time I want to spend on a move, how do I know
   what depth to search to that would (roughly) correspond to that time?

## What depth to search to (Iterative deepening)

We'll start with the latter question first. There is no real way to figure out
what depth you'll want to search to to satisfy your time constraint: it's
impossible to predict how many more nodes you will need to search by going one
ply deeper (see [Search explosion](search.md#search-explosion)).

The only real way to solve this problem is to restart the search over and over.
If at the end of each search, there's time left, you can start a new search
going one ply deeper.

If during any search you happen to run out of time, you'll need to abort the
search and return the search result from the last one of these searches that
completed successfully.

Because we're continually doing deeper and deeper searches, this technique is
called _Iterative Deepening_.

A very rough sketch could look something like this.

```rust
fn iterative_deepening(board: Board, clock: Clock) -> Report {
  let mut depth = 0;
  let mut latest_report;

  // Make sure we finish at least one depth, regardless of time, or we won't
  // have a move to print
  while clock.time_left() {
    depth += 1;
    let report = search(board, depth);

    // If we timed out during the search, don't use the search result
    if !clock.time_left() { break; }

    latest_report = report;
  }

  println!("{report}");

}
```

This might feel wasteful, because we're re-doing the same work over and over
agains on each subsequent search. There's a couple of things to keep in mind,
though:

- The number of nodes you have to search in addition to the previous ply grows
  exponentially. That means that, if you're searching to ply `N`, the work
  you've done in the previous `N-1` ply is usually just a fixed (small)
  fraction of the total work.
- The work you did in previous iterations isn't wasted! Modern chess engines
  store and use a lot of the information that was gathered in previous searches
  to make subsequent searches _much_ faster (for example, through [Move ordering](move-ordering.md),
  filling up the [Transposition table](transposition-table.md), or [narrowing the
  search's `alpha`-`beta` window](aspiration-windows.md)). In practice, doing
  iterative deepening is often a _speedup_!

## How long to search each move

The most common time controls will provide you with a given amount of _time_
(`wtime` and `btime` in UCI) and _increment_. If a player exceeds their allotted
time, they lose the game. After each move, they get a little bit of time added
to their clock (the increment).

Our main goal then, above all else, is to _not_ exceed our time! 

### Bad idea: fixed chunks

What if we take our time at the beginning of the game, and just chunk it up?

```rust
struct Clock {
  available: u64,
  start_of_turn: Instant,
}

impl Clock {
  fn new(initial_time: u64) Self {
    Self {
      available: initial_time / 50
      start_of_turn: std::time::Instant::now(),
    }
  }

  fn start_turn(&mut self) {
    self.start_of_turn = std::time::Instant:: now(),
  }

  fn time_left(&self) -> bool {
    self.start.elapsed().as_millis() < self.available
    }
  }
}
```

Each move, we'll spend `initial_time / 50` time. Surely, we'll have finished the
game after 50 moves, right? We'd better, because this kind of approach leaves
_no_ margin for longer games. After 50 moves, your engine _will_ time out.

### Better idea: fixed fractions

A slightly better idea would be to use up a certain fraction of the remaining
time on every turn.

If I use up 10% of my remaining time on every move, then my time will evolve
roughly like

```
time(n) = 0.9 * time(n-1) = 0.9^n time(0)
```

(This is of course ignoring the increment for now). This is better, right?
Our first attempt, using a fixed time for every move, would have blatantly run
out our time. With this approach, our time decreases, but never goes negative!

Let's change our previous implementation slightly. Instead of having a single
clock instance throughout the game, we'll recreate a new clock on each turn
that's initialized with the remaining time.

```diff
struct Clock {
  available: u64,
  start_of_turn: Instant,
}

impl Clock {
  fn new(remaining: u64) Self {
    Self {
-     available: initial_time / 50
+     available: remaining / 10,
      start_of_turn: std::time::Instant::now(),
    }
  }

  fn time_left(&self) -> bool {
    self.start.elapsed().as_millis() < self.available
    }
  }
}
```

### Overhead

Cool! That means we've found a strategy that allows us to keep playing forever,
right? We're mathematically guaranteed to never run out of time!

Hold up there, Zeno! There's a couple of problems here. For one, don't forget
that, in the real world, communication between processes takes time. Even if we
don't run out our time, there might be a communication overhead that will
also eat into our time! We could get timed out, not because we searched for too
long, but because our remaining time was less than the time needed to get our
response back to the client.

Now, most clients usually factor in some grace period to account for this
communication overhead, so even if the engine's response comes back slightly
late, the client will assume it was because of that communication lag, and let
the game continue. After all, because we're getting an increment on every turn,
we'll end up with positive time again at the start of next turn.

Still, it's good practice to make sure we also leave some leeway here, and don't
just blindly trust that the GUI or match runner is going to be kind to us.

```diff
const OVERHEAD: u64 = 50; // 50ms communication overhead

struct Clock {
  available: u64,
  start_of_turn: Instant,
}

impl Clock {
  fn new(remaining: u64) Self {
    Self {
      available: remaining / 10,
      start_of_turn: std::time::Instant::now(),
    }
  }

  fn time_left(&self) -> bool {
-   self.start.elapsed().as_millis() < self.available
+   self.start.elapsed().as_millis() + OVERHEAD < self.available
    }
  }
}
```

### Clear at least one depth

Then, on to our next problem. What if we don't have enough time to clear even a
single depth? We won't have anything to report, which is arguably _as_ bad as
running out of time. Let's put in a guard that, no matter how dire the time
situation, we always clear _at least_ one depth. If that means we time out, then
I guess that's just how it'll have to be.

```diff
fn iterative_deepening(board: Board, clock: Clock) -> Report {
  let mut depth = 0;
  let mut latest_report;

  // Make sure we finish at least one depth, regardless of time, or we won't
  // have a move to print
-  while clock.time_left() {
+  while clock.time_left() || depth < 1 {
    depth += 1;
    let report = search(board, depth);
    
    // If we timed out during the search, don't use the search result
    if !clock.time_left() { break; }

    latest_report = report;
  }

  println!("{report}");

}
```


### Recap
Alright, that touches on the main points of time management. The discussion so
far was mostly qualitative, and didn't really use realistic values. A common
starting point for your time management might be something like:

```diff
-  fn new(remaining: u64) Self {
+  fn new(remaining: u64, increment: u64) Self {
    Self {
-     available: remaining / 10,
+     available: remaining / 20 + 3 * increment / 4,
     start_of_turn: std::time::Instant::now(),
    }
  }
 ```

## Soft time management

This is a pretty solid start, but there's much more we could be doing here.

For example, it feels a little wasteful if we finish a search iteration, see
that we have a little time left on the clock, and start a new, deeper, search
until we run out of time. If the time left on the clock was little enough, we
could've probably _guessed_ we'd run out of time in the next iteration. We just
wasted a bunch of time on a search iteration we knew we wouldn't be able to
finish!

Similarly, maybe we're in the middle of an iteration, and we could've finished
it if we'd been given _just a little_ extra time. Clearly, our rigid "one time
limit for everyone" could be improved.

Instead, we could introduce two additional time limits: a "soft time limit" and
a "hard time limit", both derived from the time limit we've already established
in the previous section:


```diff
const OVERHEAD: u64 = 50; // 50ms communication overhead

struct Clock {
- available: u64,
+ soft_time: u64,
+ hard_time: u64,
  start_of_turn: Instant,
}

impl Clock {
  fn new(time: u64, increment: u64) Self {
+   let base_time = remaining / 20 + 3 * increment / 4;
+   let soft_time = base_time / 2;
+   let hard_time = 3 * base_time;

    Self {
-     available: remaining / 20 + 3 * increment / 4,
+     soft_time,
+     hard_time,
      start_of_turn: std::time::Instant::now(),
    }
  }

- fn time_left(&self) -> bool {
-    self.start.elapsed().as_millis() + OVERHEAD < self.available
-  }

+ fn should_start_search(&self) -> bool {
+   self.start.elapsed().as_millis() + OVERHEAD < self.soft_time
+ }
+
+ fn should_stop(&self) -> bool {
+   self.start.elapsed().as_millis() + OVERHEAD < self.hard_time
+ }
}
```

Then, at the start of each iteration, we check whether we have enough
_soft time_ to start a new iteration. This soft time is a fraction of our
"base time". If we're too close to the base time, we don't start a new iteration
and just return early. Likewise, during our negamax search, we check
`clock.should_stop()` in each node. If we've run out of `hard_time`, we should
stop the search immediately and abandon the iteration.

## Fancier tricks

This form of soft/hard time management is the meat and bones of most modern
time management implementations. It also accounts for most of the Elo that is to
be gained from good time management. Anything past this point is in
"diminishing returns" territory.

But, now that we have these concepts of hard/soft time, can we take them any
further? There might be other clues to tell us that spending more time to
complete the next search isn't likely to change the result. Or, conversely,
maybe the result has been very volatile between iterations and we'd love to
finish the next iteration to have a more accurate result.

The mechanics of how you implement these next couple of heuristics is very
dependent on the rest of the engine, so there will be less code snippets from
here on out.

### PV stability

If the best move that we've found hasn't changed in the last couple of
iterations, there's a good chance it won't change after the next iteration
either. We could decrease our `soft_time` accordingly. That means we're less
likely to complete the next iteration, but we didn't expect it to change much
anyway.

A rough idea then is:
1. Keep track of how many search iterations the PV move has remained the same.
2. Decrease the `soft_time` by a certain factor accordingly.
3. If the PV changes, revert to the old `soft_time`.

### Score stability

Likewise, if the score has stayed mostly the same between iterations, we expect
not much to change in the next iteration. The score tends to be unreliable in
the first handful of iterations, so we probably don't want to apply this
heuristic until we're searching a certain depth (e.g., once `depth >= 7`).

1. Keep track how many iterations the score has remained within some narrow window.
2. Decrease the `soft_time` by a certain factor accordingly.
3. If the score leaves the window, revert to the old `soft_time`.

### Node-based time management

This one is a bit wild, but weirdly effective. We count how many nodes were
spent investigating the subtree under the current PV move, as a fraction of
the total number of nodes searched. The idea being that if we're spending most
of our time invistigating the PV move, we're less likely to find other
candidates. Similarly, if we're spending most of our time searching _other_
moves than the PV move, that means we're not getting quite as many cutoffs from
the PV move, and are more likely to find a better move elsewhere in the tree.

1. Keep track of how many nodes were spent searching the PV move as a fraction
   of all the nodes searched.
2. Reduce/extend the `soft_time` accordingly.

