---
id: Q-1343
title: With the no-cell quirk off, does a no-destination set-down still descend after it hovers again
status: answered
related: [S-0607, S-0502]
answered-by: [S-0607]
---

## Need
S-0607 claim 9 (r5) sets a gripped unit given no destination down where it is: "it then descends and lands under claim 17's clear-cell rule". Claim 13's jumpjet stop runs first. If its search finds no cell, S-0502 claim 17's self-destruction applies unchanged, with its quirk switch. With `quirks.jumpjet_no_cell_self_destruct` off, S-0502's Edge cases say the unit "re-enters *hovering* over its current cell".

The specs do not say which comes after:
- claim 9's set-down descent and landing, onto a cell the stop's search has just found unusable for its own movement; or
- the hover re-entry, so that the unit stays hovering, set down and seized, until something else moves it.

With the switch on, the unit is destroyed and the question does not arise. The gap is in `lift::set_down_released` in crates/deterministic/sim/src/lift.rs.

## Questions
1. With the switch off and no cell found, does claim 9's set-down unit still descend and land where it is, or does it stay hovering over its current cell?
2. If it stays hovering, what later restores it or moves it (claim 20 needs a fall-back to rest after touchdown)?

## Possible observation
None in the original: the switch-off alternative is a design rule (S-0502 Edge cases).

## Current workaround
The hover re-entry clears the unit's movement point and commitment. Claim 9's set-down descent then follows over its current cell, and the unit lands where claim 9's descent takes it.
