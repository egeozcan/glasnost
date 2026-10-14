---
id: Q-0270
title: How a slave miner gets its slaves, and where and when it deploys and moves on
status: answered
related: [S-0402, S-0215, S-0415, S-0404]
answered-by: [S-0402]
---

## Need
S-0402 claim 14 specifies the cycle of a slave miner's slaves once they exist: the wake-up rule,
the slaves' search, loading, delivery, reload and the replacement of a lost slave inside the
miner. It does not say how the slaves first come into being, and its Open questions leave the
miner's own deploying and moving on unspecified. Our engine therefore never creates a slave
(crates/deterministic/sim/src/econ/harvest.rs). A side whose only resource gatherer is a slave
miner has no income in a skirmish. On the real-install lane, the third side's games run on
starting credits and refunds alone.

## Questions
1. When a slave miner is created (released from a factory, given as a starting unit, created by
   the map or by a refinery's free unit), does it get its slaves at once? How many: the type's
   slave number field? Do they start inside the miner or on the map, and on which cells?
2. Do the slaves of a newly created miner count as built or as created objects for the tech tree
   and the end-screen statistics (S-0407, S-0414 claim 20)? Do they cost anything?
3. Who owns the slaves, and what becomes of them when the miner is destroyed, sold, captured or
   deployed? Do they stay, die, or change owner?
4. Where and when does a slave miner deploy by itself, if it ever does (for a computer player,
   or for a person's miner given a harvest order)? When does a deployed miner undeploy and move
   on, for example when the fields near it are exhausted? What do its long scan and scan
   correction fields decide?
5. A person orders a slave miner to harvest (S-0213): what does the order do to the miner, and
   to its slaves?

## Possible observation
S-0402 scenario H7's setup with a recorder of object creations around the miner's release, and
a run where the fields near a deployed miner are emptied.

## Current workaround
No slave is ever created, so a slave group never wakes and the miner gathers nothing. Our own AI
treats such a miner as no harvester. A house that has no income for a long while sells its
buildings for army units (crates/deterministic/ai), so games against that side still end.
