---
id: Q-1032
title: What becomes of a carried parasite when a higher-ranked special hit suppresses its attachment
status: open
related: [S-0514, S-0614]
answered-by: []
---

## Need

S-0514 revision 5 claim 20 ranks the special detonation effects: takeover, bomb planting and electric
charging all outrank parasite attachment (row 4), and "failed higher effects still suppress lower
ones". Its cross-spec list says S-0614 claims 1 and 3 must be qualified: "Takeover, planting and
charging suppress attachment". S-0614 describes the parasite leaving the map when its carrier is
fired and either latching on or failing to latch when the carrier detonates (claims 3, 4, 21). Neither
spec says what happens to the parasite when the detonation selects a higher row, so no attachment is
made, and the parasite was still carried: it is neither attached nor told to fail.

## Questions

1. When a parasite-flagged warhead also carries a takeover, bomb-planting or charging flag and the
   carrier detonates, does the carried parasite fail to latch (S-0614 claim 4: it returns to the map as
   after a failed latch), stay stranded off the map (S-0614 claim 21), or something else?
2. Does the answer depend on the target being a legal host?

## Possible observation

A named test warhead with the parasite and the charging flags (S-0514 E13) fired by a parasite unit at a
building and at a vehicle: record where the parasite ends and its state afterwards.

## Current workaround

Interim rule: `parasite::carrier_ended` treats the suppressed attachment as a failure to latch
(S-0614 claim 4), so the parasite returns to the map as after a refused latch. Noted in
`docs/development.md` (section *Simulation (M6c: chrono weapons)*).
