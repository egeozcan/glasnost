---
id: Q-1712
title: What an empty BFRT group does in the arena
status: answered
related: [S-0309, S-0215, S-0507, S-0210, S-0607, S-0610]
answered-by: [M-0011, M-0015]
---

## Need
In the oracle arena (M-0001), BFRT groups are placed empty (no passengers), hunting, with no
orders. Two of the ten pairings whose spread falls furthest short of the record's
(`docs/m8-claims/a1-arena-spread.md`) are against them:

- TELE against BFRT. Stock: most games end at the frame limit, but both groups have lost part of
  their army, by amounts that vary from seed to seed. Ours: all twelve games end at the frame limit
  with nothing lost on either side. In a watched game (side A, seed 0) our BFRT group takes no
  target and fires no shot in 6080 frames; the TELE group fires about 3000 shots at BFRT vehicles
  without any effect.
- CAOS against BFRT. Stock: the BFRT side wins more than half the games and the rest end at the
  frame limit, with shares that vary. Ours: all twelve games end at the frame limit with the same
  shares: the CAOS group untouched, part of the BFRT group lost.

So in the stock games the empty BFRT group harms the enemy, and in ours it does not. No draw we
leave out explains that: the seed reaches these games (from the first contact the games differ
from seed to seed), yet our BFRT group never acts.

Code: `crates/deterministic/sim/src/combat/acquire.rs` (target searches),
`crates/deterministic/sim/src/crush.rs` (crushing), `crates/deterministic/sim/src/lift.rs` (the
magnetic lift).

## Questions
1. Does an empty transport whose type has no weapon of its own, in the hunt behaviour, look for
   enemies and move towards them? If it does, what does it do when it reaches them (for example
   drive over them)?
2. In TELE against BFRT, how do the BFRT vehicles lose army value in the stock games: by the
   TELE shots themselves, by being lifted and dropped, or otherwise? How does the TELE group lose
   army value?
3. In CAOS against BFRT, how does the BFRT group destroy CAOS vehicles?

## Possible observation
The arena's TELE against BFRT and CAOS against BFRT games, watched: record for each BFRT vehicle
its movement, target and every hit it deals or takes, and for each TELE vehicle what its shots
do to their target.

## Current workaround
The built rules: our BFRT group in the hunt behaviour takes no target. The pairings are reported
as failing for this reason.
