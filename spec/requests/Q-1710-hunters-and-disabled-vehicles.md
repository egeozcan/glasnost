---
id: Q-1710
title: Which disabled vehicles a hunting group leaves alone
status: answered
related: [S-0405, S-0309, S-0215]
answered-by: [M-0005, S-0405]
---

## Need
The arena houses own no buildings, so a powered-unit vehicle is disabled when it completes its
first cell arrival (S-0405 claim 13, built). In the oracle arena the robot-tank group (type ROBO)
therefore never fires, in the stock games as in ours. What differs is what the enemy group does
with it:

- Stock: M-0001 LTNK against ROBO and M-0003 ROBO against HTNK and ROBO against APOC end at
  the frame limit in most games with part of the ROBO group still in play, and the part left
  varies from seed to seed. The enemy group loses nothing.
- Ours: in every game the hunting enemy group destroys the whole ROBO group within the first
  two samples. The share left is the same in all twelve games of each pairing.

These three pairings are among the ten whose spread falls furthest short of the record's
(`docs/m8-claims/a1-arena-spread.md`). The seed reaches these games (the ROBO group's facings and
the draw counts differ from seed to seed), so no draw we leave out explains why some disabled
vehicles survive. Something in the stock game stops the hunters short of the whole group.

Code: `crates/deterministic/sim/src/combat/acquire.rs` (the map-wide search of a hunting unit),
`crates/deterministic/sim/src/econ/power.rs` (`arrival`, the disabling).

## Questions
1. Does a unit in the hunt behaviour pick a disabled enemy vehicle as its target the way it picks
   any other enemy? If not always, what decides it (the vehicle's state, its distance, whether it
   has moved, whether it was attacked before)?
2. Once a hunting unit has destroyed its target and only disabled enemy vehicles are left, does it
   go on to the next one, stop, or take another behaviour?
3. Are all the vehicles of a placed powered-unit group disabled in these games, or only those that
   have moved into another cell? Does a vehicle that is still enabled behave differently as a
   target?

## Possible observation
A house with no buildings and a group of eight powered-unit vehicles placed in two columns, all
hunting, against an enemy group of four armed vehicles, all hunting, over several seeds. Record for
each powered-unit vehicle whether it moved, when it was disabled, whether it was attacked, and
whether it was in play at the end; and for each enemy vehicle its target over time.

## Current workaround
A hunting unit's map-wide search treats a disabled vehicle as any other target, as S-0309 reads.
The pairings are reported as failing for this reason.
