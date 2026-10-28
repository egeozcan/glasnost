---
id: Q-1980
title: Does S-0417 claim 17's request refuse a walking unit that holds a destination with no path?
status: open
related: [S-0417, S-0208, S-0212]
answered-by: []
---

## Need

S-0417 revision 64, claim 17, sends a forced, move-anywhere scatter request with no threat point,
in each of its turns, to an infantry unit in *guard* or *area guard* on a cell that holds a
building that counts. The unit judges it by S-0208 claim 11: with no movement in progress the
request is effectively forced and accepted, with its draw and a new destination (S-0208 claim 13).

Claim 17 leaves one case open in so many words: "Whether its locomotor reports movement in
progress while it holds a destination with no path, so that S-0208 claim 11's third case refuses
each later request as during the walk-off (below), is not established." It names this for a
walking unit whose path failed at the reachability gate (S-0212 claim 6). The same state comes up
in a second way: a walking unit given a destination while a fresh search is waiting for the retry
delay (S-0212 claims 20 and 21). It holds the destination and has no path until the delay is
over.

The engine must choose. In our engine the choice matters. Our M7a synthetic duel (seed 1, with
our play-fixes layer) has an enslaved unit that ends up standing on a cell of its own house's
absorbing building, in *guard*. That is a separate fault of ours, which we will follow up
separately. Its search back to its master fails in each retry cycle, and the failure clears its
destination (S-0212 claim 23). With "no movement in progress" for a held destination, claim 17's
request was accepted in every turn: about 4 300 draws over 4 300 ticks, a new destination each
time. Each new destination fell inside the retry delay, so it was never searched, and the unit
never moved.

## Questions

1. A walking infantry unit holds a destination for which its path computation failed at the
   reachability gate. It rests in *guard* on a cell that holds a building that counts. In its next
   turn, is claim 17's request refused, with no draw and the destination kept, or accepted?
2. The same question for a walking infantry unit that holds a destination with no path yet
   because a fresh search waits for the retry delay.
3. If either case is accepted, does each later turn's request in that state take its draw again?

## Possible observation

A unit on a building cell in *guard*, given a destination it cannot reach (an enclosed spot)
through a request of claim 17. Log in each tick whether the unit gets a new destination and
whether the random draw count moves.

## Current workaround

Our design, marked `// design:` in `scatter::answer` (`crates/deterministic/sim/src/scatter.rs`)
and tested by `claim_17_destination_without_a_path_is_kept`: for claim 17's request only, a walking
infantry unit that holds a destination counts as having movement in progress. S-0208 claim 11's
third case then refuses each later request with no draw, and the unit keeps its destination. The
fraidycat exception still applies. This is the same outcome claim 17 gives, during the walk-off,
for a destination from the fall-back to the neighbours whose *move* is not yet current. Every
other caller of S-0208 claim 11 is unchanged.
