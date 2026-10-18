---
id: Q-1320
title: Which point a gripped unit flies towards
status: answered
related: [S-0607, S-0502]
answered-by: [S-0607]
---

## Need
S-0607 claim 7 says a gripped unit's destination is its lifter, so it cruises "towards the lifter's current position". S-0502 flies a jumpjet to a movement point. The engine must pick the point for a moving lifter, every update.

## Questions
1. Is the gripped unit's movement point the lifter's exact current position (to the lepton), refreshed every update while the lifter moves?
2. Or is it the centre of the lifter's cell, or a point fixed when the grip begins?
3. Does claim 8's arrival test (under 20 leptons) measure against the same point?

## Possible observation
Grip a unit with a lifter that is driving, and record the unit's horizontal path and the drop point against the lifter's position at the drop tick.

## Current workaround
`jump::pull` sets the movement point to the lifter's exact position every update, and arrival is measured to that point.
