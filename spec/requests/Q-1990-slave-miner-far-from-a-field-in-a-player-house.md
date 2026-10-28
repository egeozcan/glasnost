---
id: Q-1990
title: Does a slave miner of a computer-played player house wake its slaves far from a field?
status: answered
related: [S-0402, S-0215]
answered-by: [S-0402]
---

## Need

S-0402 r76 claim 14 wakes a slave miner's slaves at a processing of *guard* or *area guard* only
when the kick-delay test on the behaviour's start tick passes, the group is at rest, and one of
three alternatives holds: the owner is computer-controlled; the miner stands on tiberium land; or
the kick-delay test on the group's last-rest tick passes and the short-scan search finds a cell.
The claim adds that "the original's computer-controlled condition does not apply to player houses
in our engine". Claim 22 also says the vehicle with no destination queues *guard* and records rest
"without a field or site search", and that the long-scan field seek starts only after waking.

So in our engine a slave miner vehicle of any player house that is released or left with no
destination, away from tiberium land and with no field within the short scan, stays in *guard*
for the whole game. The other paths do work: ordered to harvest a field cell (claim 22) it finds a
site, travels and deploys, and near a field it wakes once the kick delay has passed.

In play (round 5 notes 23 and 24) this reads as "slave miners do not work" for both the person
and the computer. Our computer players own player houses, so they never take the
computer-controlled alternative. If the original's computer players do, then by claims 14 and 22
a computer player's idle slave miner far from a field wakes, seeks a field with the long scan and
moves there. Ours sits until its brain gives it an order.

In our engine `econ::harvest::slave_wake_rule_holds` takes the alternative from `House.human`,
which is true for every player house, the computer-played ones included.

## Questions

1. In the game, a person's slave miner vehicle with no destination, standing away from tiberium
   land with no field cell within its short scan range: does it ever wake its slaves or move
   without an order? (Expected from claim 14: no.)
2. In the game, the same miner owned by a computer player in a skirmish: does it wake, seek a
   field with the long scan and move there (the computer-controlled alternative)? After how long:
   the kick delay from its *guard*'s start tick, or something else?
3. Should our engine apply the computer-controlled alternative to a player house that a computer
   brain plays (skirmish computer opponents), and not to a person's house? If not, which
   alternative should a computer-played house use?

## Possible observation

H7 variant with the slave miner of the person and of a computer opponent each placed with no
destination in a clear area more than the short scan range from every field and within the long
scan of one. Record the slaves' first emergence tick and the miner's first move tick for each,
with the kick delay set short.

## Current workaround

None in the simulation: the code follows claim 14 as written, so the slave miner does not wake
for any player house. The computer brain still deploys and orders its own miners. The lead may
decide an interim design for play (see the M8 A3 intake 36 ledger, docs/m8-claims/intake36-S-0402.md).
