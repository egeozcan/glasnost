---
id: Q-0310
title: What happens to a teleporting aircraft's ground-record reservation when it lands on a bridge deck
status: open
related: [S-0207, S-0209]
answered-by: []
---

## Need

S-0207 revision 13 claim 20 (a) has an aircraft with the teleporting locomotor reserve its
landing point by setting the object-present flag of the ground record of the landing cell.
Claim 20 (e) then lists it in the deck layer at H + 416 when the cell is a high-bridge cell.
The spec's Open questions leave unsettled whether the ground-record reservation persists after
the deck placement. Our engine must decide which record or records hold the aircraft's flag
after the jump, and which ones are cleared when it later moves or leaves play. The code is in
`teleport::jump` and `teleport::aircraft_mark` (crates/deterministic/sim/src/teleport.rs), and
in `objects::remove`.

## Questions

1. After a teleporting aircraft jumps onto a high-bridge cell and stands on the deck, is the
   object-present flag of that cell's ground record still set by it? Is the deck record's flag
   set?
2. When it then jumps away, or is destroyed, which of the two records has its flag cleared?
3. While it stands on the deck, does a ground unit below the bridge treat the cell as holding
   an object (S-0209's entry outcomes)?

## Possible observation

S-0207 scenario D22's run 11 type (an aircraft type with the teleporting locomotor), ordered
onto a high-bridge deck cell with the read-only recorder. Record both occupancy records of the
cell on the jump tick and the ticks after, then order it away and record them again. Drive a
ground vehicle under the bridge through that cell while the aircraft stands on the deck.

## Current workaround

At the jump, the aircraft's flag moves from the ground record to the deck record of the landing
cell, so it is held only in the record of the layer it is listed in. Interim row in
`docs/development.md`.
