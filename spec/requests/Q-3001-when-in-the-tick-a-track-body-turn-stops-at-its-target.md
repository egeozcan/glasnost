---
id: Q-3001
title: At which point of the tick is S-0206 claim 16's test made, and against which direction of the target?
status: open
related: [S-0206, S-0203, S-0204]
---

## Need

S-0206 claim 16 (`quirks.track_body_turn_stops_at_target`, default on) stops the body turn of a
turret-less vehicle of the track class, with a target, no movement destination and no locomotor
movement, "if its interpolated value equals that direction exactly in some tick: in that tick the
body's current facing becomes its new body target facing". The interpolated value is fixed for a
tick number (claim 1), but the direction of the target (S-0204 claim 8) changes whenever the
target or the vehicle moves, and both can move during the tick (S-0203). The claim does not say at
which point of the tick the comparison is made, so two implementations can stop at different
ticks, or one stops and the other does not. GitHub issue #327 found the claim unbuilt; we have not
built it rather than guess.

## Questions

1. At which point of the tick is the body's value compared with the target's direction: in the
   vehicle's own turn (before or after its locomotor and its firing check), or at another step?
2. Which positions give the target's direction: both as they stand at that point, or positions
   kept from the start of the tick?
3. Is the test made in every tick of the turn, or only in ticks in which some rule (the firing
   check, the behaviour processing) looks at the body?

## Possible observation

Not needed if the point is known from the original's tick order. Otherwise: S-0206's own
scenario for claim 16 cannot arrange the coincidence reliably (its Backing says so), so the answer
may have to come from the order of the rules rather than a run.

## Current workaround

None: claim 16 is not built, so the body turn goes on to its own target facing in both positions
of the switch (the switch's off position). The switch stays in the schema, with its doc comment
saying it is not consulted until this is answered.
