---
id: Q-0651
title: How an ordinary attacking aircraft closes the last leptons to its firing spot
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 revision 7, table T1: an attacking aircraft in the *approaching* state turns toward its
target once it is less than 512 leptons from its destination (the firing spot of claim 39), and
only when it is less than 16 leptons away is the destination cleared and the run moves to
*firing*. Claim 5 and F4 control its speed on the way in.

For an ordinary attacker (not a strafer or fighter, no fly-by flag, so the fixed-full-speed
case does not apply), the specified rules as we read them do not bring it within 16 leptons:

- Claim 5's replacement target for the next tick is 0 below 128 leptons.
- With target 0 and the current fraction 0, F4's creep restart sets the fraction to 0.05 when
  the distance is above 0, but the 0.1 adjustment toward target 0 then moves it back to 0 in the
  same tick.
- Below 86 leptons the fraction is also halved at once with target 0.

Built literally, the aircraft slows to fraction 0 and hovers some distance short of the spot
(73 leptons away in one synthetic test; the distance depends on speed and turn rate). It never
reaches 16 leptons, never fires, and claim 40's *stopped* (no unfinished flight) never holds
for it either. Also, once it turns toward the target below 512 leptons, its heading no longer
points at the spot, so motion at that heading misses it. In a sweep over speed and turn rate,
only a few combinations reached 16 leptons, and inconsistently.

Built literally, the stall also shows in our AI skirmish tests, which use a synthetic plane
type: no plane ever fires (`flies_aircraft_from_pads`), and AI-against-AI games no longer end
within the test limit (`plays_to_the_last_side`, `plays_to_a_winner`). All three passed
before revision 7 was built, and pass again with the interim rule below.

`air::behaviour_turn` (attack) and `air::tick` (F4) must decide this.

## Questions

1. In the game, does an ordinary attacking aircraft (no fly-by, not a strafer or fighter)
   reliably reach its firing spot and fire? If so, does it slow to a stop over the spot, or
   does it keep a small minimum speed until it arrives?
2. Does the creep fraction (0.05) move the aircraft in the same tick before the 0.1 adjustment
   brings it to 0, or does it have no effect on that tick's motion?
3. When the aircraft turns toward its target below 512 leptons, does its direction of travel
   turn with it (so it flies toward the target, not the spot), or does only its aiming facing
   turn while it keeps flying toward the spot?
4. If an ordinary attacker hovers at fraction 0 more than 16 leptons from its spot, does it
   ever fire from there (for example once the target is in weapon range and in front of it),
   or does it wait indefinitely?

## Possible observation

A single armed aircraft of an ordinary type, given an attack order on a stationary enemy
vehicle in open ground, watched frame by frame: its distance to the point it fires from, its
speed in the last half-second before the first shot, and its facing compared with its direction
of travel in that time. Repeat with two or three different speeds and turn rates.

## Current workaround

An interim rule in `glasnost_sim::air`, marked `interim: Q-0651`, keeps the close approach an
ordinary attacker had before revision 7 (when its firing spot was the target itself), and
everything else follows revision 7:

- For T1's row "another aircraft whose locomotor has stopped: firing", it also counts as
  stopped when it hovers at current fraction 0 at its held height, or stands on the ground, and
  is not taking off.
- Below 512 leptons it keeps flying toward its firing spot, and turns toward the target only
  once hovering.
- While it hovers and the firing check answers *not facing*, it stays approaching and turns in
  place toward the target.

`q0651_ordinary_attacker_reaches_its_spot_and_fires` in
`crates/deterministic/sim/tests/claims_s0501.rs` checks this. Recorded in docs/development.md.
