---
id: Q-1714
title: How the M-0001 YTNK against SREF fight unfolds
status: open
related: [S-0609, S-0518, S-0301, S-0309, S-0307]
answered-by: [S-0318, M-0021, S-0301]
---

## Need
M-0001 YTNK against SREF is the second of the ten pairings whose spread falls furthest short of
the record's (`docs/m8-claims/a1-arena-spread.md`). The gap is in the mean as much as in the
spread:

- Stock: the YTNK side wins about two games in three, both sides lose most of their army, and the
  shares left vary from seed to seed.
- Ours: the YTNK side wins all twelve games by the first sample (frame 304) without losing a
  vehicle. In a watched game (side A, seed 0) the YTNK group fires about 110 shots by then and the
  SREF group about 16. Every YTNK of the front column has taken the same damage, those of the second
  column none.

The seed reaches these games (they differ from seed to seed from about tick 58, when the groups
first fire), but a fight that one side wins this clearly leaves nothing for the draws to vary.
Something in how the two groups meet differs from the stock game, and no draw we leave out
explains it.

Code: `crates/deterministic/sim/src/combat/fire.rs` (firing cycle),
`crates/deterministic/sim/src/gattling.rs` (firing stages), `crates/deterministic/sim/src/linked.rs`
(linked weapons).

## Questions
1. In this pairing, which group fires first, and at what distance between the groups?
2. Over the first 300 frames, how many shots does each SREF vehicle fire, and how many YTNK vehicles
   does one SREF shot damage?
3. If either type has firing stages that rise while it fires (S-0609), when does that group reach
   its top stage, compared with the first vehicle lost on either side?
4. Do the SREF vehicles fire at the nearest YTNK, or spread their shots over the group?

## Possible observation
The arena's YTNK against SREF game, watched for the first 600 frames: per vehicle, its position,
target, shots fired, damage taken and firing stage over time.

## Current workaround
The built rules. The pairing is reported as failing for this reason.
