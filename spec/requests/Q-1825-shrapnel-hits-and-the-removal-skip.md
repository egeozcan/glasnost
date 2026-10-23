---
id: Q-1825
title: Do a burst's fragments all hit in the tick of the shot when the shot's own removal costs a turn
status: open
related: [S-0318, S-0203, S-0306]
answered-by: []
---

## Need

S-0318 scenario SH1 part a expects the shot's target and the first 3 surrounding vehicles to be
hurt "in the same tick" and is falsified by "damage in a later tick". The text after claim 5's
backing rates "all of these hits fall in the tick of the burst" as `measurable` (SH1 part a, SH4
part a). It also says that each fragment detonates in the tick of the burst unless an earlier
removal costs it its first turn, and that no part sets up such a removal.

Our engine follows the claims as written, and with them SH1 part a has such removals:

- The fragments enter play in the bursting projectile's own turn (S-0318 claim 5: placed at once
  in the update order).
- The bursting projectile then leaves play in that turn. S-0203 claim 10 counts that removal
  (its first cause), so the object that directly follows it misses its turn. That object is the
  first fragment.
- The second fragment detonates and leaves play in its own turn, which costs the third fragment
  its turn.

So, with S-0203's removal-skip switch on (the default), the three fragments hit over three ticks:
the second in the tick of the shot, the first in the next tick, the third in the tick after that.
With the switch off, all three hit in the tick of the shot, as SH1 part a expects. The test
`claims_s0318::sh1a_removals_cost_fragments_their_first_turn` pins the first case and
`sh1a_three_fragments_in_the_tick_of_the_shot` the second.

The code does not wait on this. It follows S-0318 claim 5 and S-0203 claim 10. But one of these
does not match what the game does: SH1 part a's expected outcome, or the claim 5 placement (where
the fragments enter the update order), or S-0203 claim 10 for this case.

## Questions

1. In SH1 part a, in the running game, are the target and all three hurt surrounding vehicles
   hurt in the tick of the shot? Or are some of the surrounding vehicles hurt one or two ticks
   later?
2. If all are hurt in the tick of the shot: where in the update order does a fragment enter
   play, relative to the projectile that bursts? Or does the bursting projectile's removal in its
   own turn cost no turn in this case?
3. If some are hurt later: which of the three, and in which tick? (This would confirm the order
   given above and need only a correction to SH1 part a and to the backing of the timing
   sentence.)

## Possible observation

SH1 part a as written, with S-0203's recorder of turns, or simply health per tick for the three
fragment victims over the tick of the shot and the next 2 ticks.

## Current workaround

None needed. The code follows S-0318 claim 5 and S-0203 claim 10 as written; the default play
gives the three-tick spread above.
