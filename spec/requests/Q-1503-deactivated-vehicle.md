---
id: Q-1503
title: What a vehicle disabled by the loss of its control centre cannot do
status: answered
related: [S-0405, S-0616, S-0301, S-0215]
answered-by: [S-0405]
---

## Need
S-0405 claim 13 disables a vehicle whose type has the powered-unit flag when it enters a cell
while its house has no active control centre: it "is inactive until enabled". No approved spec
says what an inactive vehicle cannot do. Can it fire, take a behaviour, be given orders, move
or turn? Our engine sets the state but nothing reads it
(`crates/deterministic/sim/src/econ/power.rs`, `unit_disabled`).

The oracle arena shows the effect. Houses own no buildings, so no control centre exists. In
every stock game of M-0001 and M-0003 with ROBO, the opponent kept all or nearly all of its
army (against LTNK, MIND, YTNK, DISK, APOC, TTNK, V3, HTNK and HTK). ROBO games end `none` unless the opponent destroys the ROBO group. In our
engine the ROBO group fights normally. For example, ROBO against HTK in M-0003: stock column
wins 12 of 12; ours, row wins 12 of 12. LTNK against ROBO in M-0001:
stock row wins 5 of 12; ours 12 of 12.

## Questions
1. What can a disabled (deactivated) vehicle not do? For each, can it fire, acquire targets,
   turn its turret or body, move, process its current behaviour, or be ordered?
2. Does a vehicle created on a cell, or placed there by the map or a harness, count as
   entering that cell for claim 13's cell-entry rule? Or is it disabled only at its first move
   into another cell?
3. Can a disabled vehicle still be targeted and damaged in the ordinary way?

## Possible observation
A house with no control centre and one powered-unit vehicle, placed by the map with an enemy in
its weapon range. Record whether it fires before and after it is ordered to move one cell.

## Current workaround
The disabled state is set as S-0405 claim 13 says and read by nothing else (the existing seam),
so the vehicle acts normally. The arena's ROBO pairings are reported as failing for this reason.
