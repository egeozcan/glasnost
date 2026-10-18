---
id: Q-1340
title: Does the teleporting stand-in's jump, or a driving stand-in's arrival, end the grip
status: open
related: [S-0607, S-0207]
answered-by: []
---

## Need
S-0607 claim 23 (r4) moves a unit seized with the teleporting stand-in under S-0207 claim 20, with the lifter as its object destination, and says the grip ends by claim 12's no-destination row "when the stand-in's own rules remove the unit's order-level destination". S-0207 claim 20 removes the order-level destination when the jump is made. So under the two specs together the grip ends as soon as the victim lands next to the lifter. Scenario L13 part b, though, lists "a grip ended by arrival alone" as a falsifier for the driving and the teleporting warheads. The same tension can reach the driving stand-in: the seized unit is in *guard* (claim 5), and S-0207 claim 26's guard row removes the destination when the vehicle's world point equals its locomotor destination point (the lifter's point). That is rare, because a mobile lifter stands on its own point, but it may be reachable at a building's docking point. The engine must pick one. The gap is in `lift::enforce` together with `teleport::tick` (crates/deterministic/sim/src/).

## Questions
1. With the teleporting warhead and a lifter that stands still, does the grip end on the tick the victim lands near the lifter?
2. If not, what does end it, and does the victim make further jumps meanwhile?
3. With the driving warhead and a building lifter that gives a docking point, does the grip end when the victim stops exactly on that point?

## Possible observation
Scenario L13 part b with the teleporting warhead, and with the driving warhead against a repair-depot lifter: record the tick of the jump, the victim's order-level destination and the lifter's target on that tick and the next few.

## Current workaround
The specs are followed as written: S-0207 claim 20's jump removes the destination, so S-0607 claim 12's no-destination row ends the grip on the tick of the jump and the unit is set down where it landed (still seized, claim 24). A driving stand-in that meets S-0207 claim 26's guard row likewise loses its destination, and the grip ends.
