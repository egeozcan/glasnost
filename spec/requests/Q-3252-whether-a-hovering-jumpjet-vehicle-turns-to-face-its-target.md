---
id: Q-3252
title: Whether a hovering jumpjet vehicle with an omni-fire weapon turns to face its target
status: open
related: [S-0502, S-0310, S-0206, S-0501]
answered-by: []
---

## Need

A hovering **vehicle** on the jumpjet path (no turret, not balloon-hover) whose weapon has the
omni-fire flag attacks a ground target. As the specs read now, its body never turns towards the
target:

- S-0310 claim 2: an omni-fire weapon is never *not facing*, so the firing check answers that it
  may fire whatever the aim error.
- S-0206 claim 10 (b) and (c): a turretless vehicle's body is re-aimed only on a facing error, and
  an omni-fire weapon never gives one.
- S-0502 claim 10: a jumpjet's body facing follows its movement heading. The one rule that snaps
  the heading to the target applies only to a hovering **infantry** jumpjet with the jumpjet-turn
  flag (S-0310 claim 8).
- S-0501 claim 19's aircraft rule does not apply, since the object is a vehicle, not an aircraft.

The engine does this as written. The project lead's pathing report (issue #447, an informal
observation, not a measurement record) shows such a vehicle hovering and firing at a tank every
rearm delay, with its body held about 11 500 facing units (more than five times the 2048 aiming
tolerance) away from the direction to the target for the whole window. The lead expects it to face
what it shoots. If the game turns it, a rule is missing from S-0502 claim 10 or S-0206 claims 9
and 10, and the drawn body facing and any facing-dependent fire point would change.

## Questions

1. A hovering jumpjet vehicle (not balloon-hover, no turret) attacks a ground target with an
   omni-fire weapon while it is not moving. Does its body turn towards the target? If so, at which
   rate (the jumpjet turn rate of S-0502, the body turn rate of S-0206, or another), and does it
   fire before the turn ends?
2. Is the answer the same when the weapon does **not** have the omni-fire flag (where S-0310's
   facing test applies and S-0206 claim 10 (b) re-aims the body on a facing error)?
3. Does the answer depend on the jumpjet-turn flag of S-0310 claim 8, which today only an
   infantry type uses?
4. While the vehicle moves to keep its target in range, does its body follow the movement heading
   (S-0502 claim 10) or the target?
5. Does a balloon-hover type (S-0310 Terms) follow the same answer?

## Possible observation

A synthetic or stock setup with one hovering jumpjet vehicle whose weapon has the omni-fire flag,
ordered to attack a stationary ground target placed about 90 degrees to one side of its facing:
record the drawn body facing and each shot's tick from the order until several shots have been
fired. Repeat with the omni-fire flag off, and with the vehicle given a move order between shots.

## Current workaround

None: the engine keeps S-0310 claim 2, S-0206 claim 10 and S-0502 claim 10 as written, so the
body does not turn. A revision that adds a turn needs a facing rule for the jumpjet vehicle path
and claim tests on a synthetic ruleset.
