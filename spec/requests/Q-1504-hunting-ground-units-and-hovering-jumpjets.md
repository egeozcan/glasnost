---
id: Q-1504
title: How hunting ground units find a hovering jumpjet group
status: answered
related: [S-0309, S-0215, S-0502, S-0615, S-0102]
answered-by: [S-0309, S-0502]
---

## Need
S-0309 claim 1 gives a hunting unit only the map-wide search (row "the processing of hunt").
S-0309 claim 7 says that search's ground-layer pass takes only objects in the ground render
layer, "so an object that is neither an aircraft nor in the ground layer, such as a hovering unit
with jumpjet movement (air or top layer, S-0102 claim 3), is never found by a map-wide search,
whatever the searcher's weapons can hit".

Our engine now follows that text (`crates/deterministic/sim/src/combat/acquire.rs`, map-wide
pass 1 skips objects outside the ground layer, `temporal::ground_layer`). In the oracle arena,
M-0001 DISK (a jumpjet vehicle type) against FV (all units hunting, no orders) then goes the
other way: stock, the column wins 12 of 12 (by frame 608 to 912); ours, the row wins 11 of 12.
In ours the FV fire about 160 shots before the first sample (while the DISK group is still on
the ground or in the ground layer), then almost none: once the DISK hover, no FV search finds
them. The FV are then destroyed one by one without firing back (their shot count stays the
same from frame 912 to the end of the watched game, side A seed 0). Before this change
(when the map-wide search also took hovering jumpjets) ours matched the stock outcome.

So in the original something else lets a hunting ground unit attack a hovering jumpjet. Candidates
we cannot choose between from the specs: the hovering jumpjets come down into the ground layer
while they attack (S-0502, S-0615 claim 16 for the disc weapon); hunting units retaliate when
attacked by them (S-0309 claim 18; is the disc weapon's damage attributed to the attacker?); or
hunting also runs a range search with the airborne scan of claim 9.

## Questions
1. Does a unit in the hunt behaviour, armed against air, attack an enemy jumpjet that hovers in
   range and never lands? Through which search or event does it get that target?
2. Does a jumpjet vehicle with a disc weapon (S-0615 claim 16) leave the air or top layer while it
   attacks, or at other times during a fight?
3. Does a ground unit damaged by such a jumpjet's disc weapon retaliate against it?

## Possible observation
One enemy jumpjet vehicle hovering (given a move order to a cell and then no order) within range
of an anti-air vehicle in the hunt behaviour, with the jumpjet unarmed; then armed and attacking
the anti-air vehicle. Record whether and when the anti-air vehicle fires, and the jumpjet's height
over time.

## Current workaround
The literal text of S-0309 claim 7: a map-wide search never finds a hovering jumpjet. The M-0001
DISK pairings against hunting anti-air groups are reported as failing for this reason.
