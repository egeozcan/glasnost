---
id: Q-1870
title: A harvester that leaves a refinery's docking cell gives up its way back to the field after 5 corridor searches and stays on the dock; is that the game's behaviour?
status: answered
related: [S-0212, S-0402, S-0209, S-0207, Q-1771]
answered-by: [S-0212]
---

## Need

The lead's second batch of pathing reports (M8 step A4, `docs/m8-claims/a4-play-notes.md`,
"Pathing report 2 (2026-10-25)") shows the same give-up as Q-1771, for a harvester rather than a
unit at a factory's exit. The specs as approved produce it with no departure, and the lead saw it as
"pathing problems, especially the ore miners". This request adds the case to Q-1771 and asks what
the harvester does after the give-up, which Q-1771 does not cover.

The shape: the person's refinery on raised ground, its docking cell on its east side, the ore field
about 17 cells west, other buildings of the base between them. A harvester unloads and is sent back
to the field (S-0402). The coarse search sees through buildings (S-0212 claim 1), so the first
corridor runs west through the refinery's own cells. The level-2 retry then reads its sector as split
(claim 13a) and closes the connections west, the later corridors run north into the other buildings,
and the 5th corridor cell search fails (claim 13's cap). By claim 23 row 2 and claim 27a the
destination is cleared and the harvester queues *guard* on the docking cell, where nothing moves it
(S-0402 claim 3a). A second, loaded harvester then drives round the occupied dock (S-0402 claims 10
and 10a; S-0212 claims 24 and 25) until the first is moved off by hand. In our build a 7th search
would find a 21- to 23-step path, and with both `corridor_retry_no_fallback` and
`corridor_split_probe` off (one alone changes nothing) the first computation finds the way.

## Questions

1. Q-1771's questions, for this case too: does the game's computation give up here, or find the way
   (a higher cap, a fallback search without a corridor, or another corridor)?
2. If the harvest computation does fail: does a harvester that has just left the dock rest on the
   docking cell, or does something move it off the dock (the refinery, the harvester's own mission,
   a retry after some ticks, a scatter)? How long before it tries the field again?
3. A second harvester arriving loaded at a dock that a resting harvester occupies: does it wait (where),
   circle as the specs give now, or make the resting one move?

## Possible observation

A skirmish base where the refinery stands between its docking cell and the field, with other
buildings beyond it on the field side. Let one harvester unload and watch whether it leaves for the
field. If it stays, send a second harvester to unload and watch both for a minute.

## Current workaround

None needed in code: the code follows the approved claims (S-0212 claims 1, 13, 13a, 23, 27a; S-0402
claims 3a, 10, 10a). Tests `claims_s0212_cap.rs` (`claim_13_cap_for_a_vehicle_on_a_docking_cell` and
two neighbours) pin the case on a synthetic layout.
