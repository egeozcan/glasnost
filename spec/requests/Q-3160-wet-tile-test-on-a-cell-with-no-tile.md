---
id: Q-3160
title: Whether a warp landing cell that holds no tile passes the wet-tile test
status: answered
related: [S-0513, S-0010, S-0211, S-0009]
answered-by: [S-0513]
---

## Need
S-0513 claim 38 (revision 124) sinks a member with no credit when the landing cell's tile is a wet
tile (Terms: the shore, water and four waterfall roles' ranges of S-0010 table R) and the member is
not infantry, and kills it with no credit otherwise. The road switch `quirks.warp_naval_road_cell`
off also reads the wet tile. S-0513's open questions leave the case of a landing cell that holds no
tile (S-0211 claim 5) unsettled. That decides whether a member that claim 38 sends to this branch on
such a cell sinks or is killed, and whether such a cell, given the road land type, is a road cell
with the switch off (issue #364).

## Questions
1. Does a landing cell that holds no tile (the no-tile marker of S-0009) pass the wet-tile test of
   claim 38, in both branches that read it: the no-credit sink and the road switch off?
2. If it is read as a tile number, is the no-tile marker compared against the role ranges as it is
   stored (so it is wet only when a role's range reaches it), or is it never wet?

## Possible observation
An SW13-style warp of a non-infantry member onto a cell with no tile that it cannot enter. Record
whether it sinks or is killed, and do the same on a cell with no tile that an overlay gives the
road land type, with a naval member.

## Current workaround
`warp::wet_tile` (`crates/deterministic/sim/src/superweapons/warp.rs`) compares the cell's stored
tile with each role's range. The no-tile marker (65535) falls in no range unless a role starts within
range of it, so in practice such a cell is not wet: the member is killed with no credit, and with
the switch off a road cell with no tile still counts as a road cell. A world with no tile catalogue
(synthetic test maps only) has no wet tiles either. In such a world a member in this branch on water
land is now killed with no credit, where before revision 124 it sank. That is our engine's
bookkeeping, not a case the game meets.
