---
id: Q-0311
title: Where a teleporting infantry unit's ground-record reservation goes after it jumps onto a bridge deck
status: open
related: [S-0208, S-0207, S-0209]
answered-by: []
---

## Need

S-0208 revision 19 claim 19 has a ground-height teleporting infantry unit choose its standing
position on a high-bridge destination cell with the ground record and reserve it there (the
chosen height is below the 416-lepton deck mark). At the jump S-0207 claim 20 (e) lists it on
the deck with deck height, and claim 19 says its later occupancy uses its deck status. The spec
does not say what becomes of the standing-position entry in the ground record, or whether the
deck record gets an entry for that position at the jump. Our engine must keep one consistent
entry so that later moves clear the right record. The code is in `teleport::jump`
(crates/deterministic/sim/src/teleport.rs).

## Questions

1. After such a jump, is the standing position still taken in the ground record of the cell?
2. Is the same position taken in the deck record from the jump on?
3. When the unit later leaves the cell, which record's entry is cleared?

## Possible observation

A teleporting infantry type standing on the ground, ordered onto a high-bridge deck cell, with
the read-only recorder. Record the ground and deck records' standing positions of that cell
before and after the jump, then order a walking infantry unit under the bridge into that cell
and see which positions it may take.

## Current workaround

At the jump the reservation moves from the ground record to the deck record, at the same
standing position. Interim row in `docs/development.md`.
