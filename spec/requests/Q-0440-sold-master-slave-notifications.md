---
id: Q-0440
title: Notifications and cheer for the slaves of a sold slave master
status: open
related: [S-0408, S-0312, S-0402]
answered-by: []
---

## Need

S-0312 claims 30 and 31 now say that hidden slaves of a destroyed or
removed master are destroyed with claim 30's common destruction
notifications (a loss for their house, the destruction raises), and that an
on-map slave given to the civilian-side house has *cheer* requested after
its reset. S-0312 claim 31 leaves the true-sale fate to S-0408 claim 10a.
Claim 10a says hidden slaves are "removed without being placed" and on-map
slaves get *guard*, but says nothing about destruction notifications or
*cheer*. The code is `econ::slaves::end_group` (`End::Sold`), called from
`econ::sell::finish`.

## Questions

1. When a slave-master building is truly sold, does each hidden slave count
   as destroyed: a loss for its house in the end-of-game statistics, and the
   destruction notifications of S-0312 claim 29 (for example a destroyed
   trigger event on the slave)? Or does it simply vanish?
2. When an on-map slave of a sold master goes to the civilian-side house,
   does it play its cheer sequence (S-0107 claim 21), as a slave of a
   destroyed or removed master does?

## Possible observation

A purpose-made skirmish map with a civilian-side house and a deployed slave
miner whose slaves are partly inside and partly out. Sell it. Compare the
loss count in the end-of-game statistics with a control game without the
sale, and watch the on-map slaves' animation in the tick they change owner.

## Current workaround

A true sale: hidden slaves are removed with no loss and no destruction
notifications; on-map slaves given to the civilian-side house are reset and
queued to *guard* but no *cheer* is requested. Listed in
docs/development.md (interim decisions).
