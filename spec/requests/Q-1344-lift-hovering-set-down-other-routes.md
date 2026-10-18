---
id: Q-1344
title: Which seized set-down units count as hovering under the no-cell switch-off rule
status: open
related: [S-0607, S-0502]
answered-by: []
---

## Need
S-0607 claim 20 (r8) keeps one kind of unit from being restored: "a set-down unit hovering under claim 9's switch-off rule for no cell". That unit repeats the stop's search at each fall-back to rest instead. S-0502's Edge cases (r16) state the rule more widely: "For a seized set-down unit, S-0607 claims 9 and 20 apply: while hovering it is not restored and retries the search at each fall-back to rest."

The two wordings give different results when a seized set-down unit starts hovering, with the switch off, by some other route than claim 9. Example: a set-down unit that is still moving (claim 13, or claim 9 when a cell was found) flies to its point. Its landing there fails (S-0502 claim 15), and the reroute's search (S-0502 claim 17) then finds no cell. With `quirks.jumpjet_no_cell_self_destruct` off, it hovers over its current cell with no movement point and no commitment. It is set down and seized, and its jumpjet movement is not moving.
- Under claim 20's three conditions as written, it is restored at its next fall-back to rest while in the air.
- Under S-0502's wording, it is not restored and retries the search.

A second, smaller gap is the boundary between "in the air" and "grounded" for a jumpjet that is not seized. S-0502's Edge cases say "In the air it re-enters hovering ...; a grounded jumpjet stays grounded instead". That reads as the flight state. S-0607 claim 9 splits a seized unit by its height above the surface (1 lepton or more against 0 or less). A jumpjet whose take-off update has just made it *ascending* is still at height 0 until its next update. Which rule applies if claim 17's stop reaches it in that tick with no cell found?

The gaps are in `lift::restore_at_rest` and `jump::stop_with` (crates/deterministic/sim/src/lift.rs and jump.rs).

## Questions
1. Does claim 20's hovering rule (no restoration, the search repeated at each fall-back to rest) apply to every seized set-down unit that hovers with no movement point and no commitment after S-0502 claim 17's switch-off outcome, or only to one that claim 9 set down?
2. If only to claim 9's unit, what happens to the others at a fall-back to rest while they hover in the air?
3. With the switch off and no cell, is a jumpjet that is *ascending* at height 0 or less "in the air" (it hovers) or "grounded" (it stays where it is)? If it is grounded, does its flight state become *grounded*?

## Possible observation
None in the original: the switch-off alternative is a design rule (S-0502 Edge cases; S-0607 claims 9 and 20).

## Current workaround
1. S-0502's wider wording is followed, with the switch off only. Every seized set-down unit that hovers with no commitment and has no lifter is not restored at a fall-back to rest. It repeats the stop's search instead. With a cell found, it re-orders itself to that cell's centre and flies there. With none, it keeps hovering.
2. In `jump::stop_with`, the flight state decides: only a *grounded* jumpjet stays grounded, and any other state re-enters *hovering*. A seized unit set down by claim 9 at height 0 or less is grounded where it stands either way (`lift::set_down_released`).
