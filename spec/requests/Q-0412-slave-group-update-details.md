---
id: Q-0412
title: Details of the slave group update and the slaves' own harvest
status: answered
related: [S-0402, S-0203, S-0208, S-0209]
answered-by: [S-0203]
---

## Need

S-0402 claims 14, 19 and 21 fix what a slave group does but leave the
following unstated. Each one changes the tick on which a slave appears,
loads or pays, so the state hash depends on it. The code is `econ::slaves`
(`group_turn`, `regenerate`, `emerge`, `process_slave`, `slave_full`,
`slave_accepts`, `building_ring`).

## Questions

1. Within the master's turn, does the group update come before or after the
   master's own other turn work (a building's power and repair steps, a
   vehicle's movement)? Within one group update, what is the order of:
   replacing lost slaves, reloading slaves inside, taking back returning
   slaves, the master's own decisions (field seeking, travelling, deploying,
   relocation) and letting ready slaves out? For example: can a replacement
   created in an update leave in the same update?
2. When more than one slave has been lost and their delays have run out at
   the same update, are all replaced in that update? In which order are the
   new slaves created (the order matters for the object-creation draws)?
3. Which way does a slave face when it comes out of its master?
4. A slave type with a storage of 0 or less: does a slave ever load? Does it
   go straight back to its master after it reaches a cell?
5. Is the interval between a slave's harvest processings the same while it
   searches for a cell and while it walks back as while it loads?
6. When a slave searches for a cell, which cells does it refuse besides those
   without resource: cells another slave or unit stands on, cells it cannot
   enter as infantry (for example an occupied infantry position), cells in
   another movement zone?
7. A deployed slave miner checking for a field near itself with its short
   scan: is the scan centred on its top-left cell, its centre cell or its
   unloading cell? Does a cell need anything besides being in the usable area
   with land type tiberium (for example, being free or reachable)?

## Possible observation

A purpose-made map with one slave miner of a human player on a field. Record
every slave's creation tick, facing, cell and load each tick, and the house's
credits, through a field exhaustion, the loss of two slaves in the same tick
and their replacement. Repeat with a slave type of storage 0.

## Current workaround

1. The group update runs in the master's turn after its power and repair
   steps and before its shroud update, and in the order replace, reload, take
   back, decide, let out; a replacement is ready and may leave in the update
   that creates it.
2. Every slot whose delay has run out is filled in that update, in slot order
   (initial creation order).
3. The master's body facing.
4. Counted as full at once: it walks back with nothing.
5. The slave type's harvest interval *h* in every phase.
6. The cell must give a clear infantry entry outcome (S-0209 claim 4) and lie
   in the slave's movement zone.
7. Centred on the top-left cell; usable area and land type tiberium only.

Listed in docs/development.md (slave miners).
