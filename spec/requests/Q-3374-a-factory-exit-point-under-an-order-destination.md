---
id: Q-3374
title: What a driving vehicle's readings measure to after a factory exit track that kept an order-level destination
status: open
related: [S-0207, S-0417, S-0212]
answered-by: []
---

## Need

S-0207 claim 29 (r92) says that a driving vehicle's movement readings that need its destination's
point use the locomotor destination point of claim 25, "the point that the last assignment to
install one supplied, until claim 25's table clears it". It names every path search of S-0212
claims 20 to 23, the 512-lepton distance that starts claim 20's recomputation, the braking test of
claim 10 and the give-up test of claim 15.

S-0417 claim 5 puts a released drive vehicle on the exit track towards *E*'s centre. A successful
exit-track instruction "also supplies the separate locomotor destination point at *E*'s horizontal
centre with integer world height 0 leptons". S-0417 claim 16's table lists what later replaces or
clears that point: a rally or accepted scatter destination, an ore destination, a later track
completing at its order-level goal with claim 25's height condition met, a cell-goal cleanup or
stop, and restoration.

Our engine meets a vehicle that already holds an order-level cell destination *D*, assigned while
it stood in contact with its factory, when the exit-track instruction is given. The instruction
installs *E*'s point and keeps *D*. When the exit track ends at *E*, claim 25's completion row asks
*D* for its point, whose cell is not *E*'s, so it does not clear *E*'s point. No new assignment
follows. Read by claim 29, every later search then looks for *E*'s cell, which the vehicle is
already in. It makes no steps, never leaves the apron, and *E*'s point is never cleared. In our AI
games every slave miner of one side does this, and that side's economy never starts.

Neither S-0207 claim 29 nor S-0417 claims 5 and 16 says which point these readings use when the
installed point is the exit track's and the order-level destination is another destination,
assigned before the exit.

## Questions

1. Can a drive vehicle hold an order-level destination *D* when its factory gives the exit-track
   instruction (for example one assigned at its release, S-0417 claim 8 or S-0402)? If not, which
   rule removes or replaces *D* first?
2. If it can, after the exit track ends at *E* with *D* still held and no new assignment, which
   cell do the searches of S-0212 claims 20 to 23 look for, and which point do braking and the
   give-up test measure to: *E*'s point (so the vehicle stays at *E*), or *D*'s point?
3. Does anything clear *E*'s point in that state, apart from the events of S-0417 claim 16?

## Possible observation

A drive harvester or slave miner built at a factory whose house has a refinery and ore in reach,
and no rally point, so that it is given a destination while still in contact with the factory.
Record per tick from its release: its order-level destination, whether it is on the exit track,
its cell, and over the next 300 ticks whether it leaves *E* and the cell its stored path ends in.

## Current workaround

A flag in the driving state marks the installed point as the exit track's. While it is set and an
order-level destination is held, `drive::measured_point` gives the point that destination supplies
at the time of the reading: S47's rule before claim 29. The next assignment that installs a point
clears the flag, and claim 29's reading of the installed point applies from then on.
