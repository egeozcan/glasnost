---
id: Q-1850
title: The lead's play report against S-0417 claims 10 and 14: infantry released with no rally point never block the factory's exit
status: answered
related: [S-0417, S-0404, S-0215, Q-1757]
answered-by: [S-0417]
---

## Need

This follows Q-1757 and settles its premise. The project lead reports from play of the original
game (M8 step A4, round 3, 2026-10-24) that units from an infantry factory with no rally point
staying at the exit and blocking later units "is 100% wrong": in the original, later infantry
units still come out.

The approved specs, as written, give the blocking. S-0417 claim 10 gives an infantry unit released
with no rally point (or one in its own cell) no destination; it rests in *guard* on a foundation
cell and keeps factory contact. With one contact place, S-0417 claim 14 makes every later infantry
release from that factory *busy*, and S-0404 claim 7 then cancels the item with a refund. Claim 10
has low confidence and says it is "not a prediction of a permanent wait"; Open question 3 leaves
open what moves the unit off the foundation. Our engine follows the claims
(`econ::release::infantry`, crates/deterministic/sim/src/econ/release.rs), so in our game the
first unit stays at the exit and later units of that factory never appear. The specs and the
lead's report cannot both be right.

## Questions

Q-1757's questions 1 to 5 stand. In addition, given the lead's report:

1. With no rally point, does a released infantry unit leave the factory's foundation cells on its
   own, or does the factory's contact with it end without it moving (so that the next release is
   not *busy*)? Which of the two is it, and on which tick after the release?
2. If it leaves on its own: to which cell (relative to the factory and its exit), by what rule is
   the cell chosen when the first choice is occupied, and does each later unit go to the same
   place or a different one?
3. Five infantry units of one house are queued at one factory with no rally point, nothing else
   near the factory. Where does each unit stand once all five are out, and do all five appear
   without delay beyond their build time?

## Possible observation

S-0417 scenarios U10 part b and U13 part a, with five units queued in a row at one infantry
factory and no rally point: record each unit's release tick, the cells it occupies for 300 ticks,
its behaviour and the factory's contact, and the sidebar state of each item.

## Current workaround

The claims as written (Q-1757's workaround): no destination and contact kept, so later releases are
busy and their items are cancelled with a refund. No invented walk-out, scatter or contact timeout.
The lead has asked for this to be settled first among the open A4 requests.
