---
id: Q-2171
title: Approach candidates on a high-bridge deck over water, and the reach test of an actor on such a deck
status: open
related: [S-0307, S-0212, S-0215]
answered-by: [S-0307]
---

## Need

S-0307 claim 19 lets high-bridge cells qualify as approach positions. Their clear-cell test
reads the deck occupancy, and the actor's reach region comes from its actual cell and its
ground/deck layer, but "the candidate region is the ground-layer region". RG18 run c checks this
with the actor's region equal to the candidate's ground region, and then with the two different.

On a bridge over water, the ground under every deck cell is water, which is in no reach region
of a ground zone (S-0212 claim 4). Read literally, a ground vehicle never qualifies a deck cell
over water. An ordinary attacker whose target stands on such a bridge, and whose radii around the
target reach only deck cells, finds no position. It then drops the target (claim 17), even when
it stands on the same deck itself.

The lead's pathing report of 2026-10-31 (M8 A4 round 6, note 37, `path-46317-1.gpath`) shows
this case. A tank stood on the deck of an enabled high bridge running along cell y 89/90, over
water. Its own deck read gave the banks' reach region. An enemy vehicle stood on the same deck
5 cells west, at cell (91, 90). Every primary in range was a deck cell, at (93, 90) and (92, 89).
Each was enterable and empty, but its ground was in no reach region, so the tank dropped the
target. The code follows the claim as written: `combat::range::approach_search`, the `clear`
closure.

## Questions

1. Over water, is the region test of a high-bridge candidate made with its ground (no reach
   region, so always refused for a ground unit) or with its deck read through the bridge link
   (S-0212 claim 4)?
2. If it uses the ground, does a ground vehicle on the same deck, ordered to attack a unit
   standing on that bridge out of range, drop the target at once? (This assumes no ground cell
   near the bridge passes the range test.)
3. Does the answer differ for infantry, or for an attacker standing on the bank rather than on the deck?
4. A related read: S-0215 claim 37 (a) runs the per-tick approach step for an actor that "cannot
   reach its target's cell (the target's cell is not in the actor's reach region ...)". For an
   actor standing on a deck over water, is that region the deck's (read through the link, as
   S-0307 claim 19 now reads it) or its ground's? Our engine reads the ground's (S-0212 claim 4,
   "every other read"). So such an actor counts as unable to reach any cell and takes the step on
   every tick, and in a behaviour other than *attack*, with may-approach *no*, it drops its
   target by claim 37 (d)'s table. In the same report, after the fix named under Current workaround, a vehicle on the deck at
   (95, 88) received an attack order. On the next tick its behaviour was still *move*, and it
   dropped the target in this way.

## Possible observation

Use RG18's recorder on a named map with an enabled high bridge of at least 8 deck cells over
water. Stand an unarmed enemy vehicle in the middle of the deck. Give it a range so that every
primary of F4 that passes the range test is a deck cell. Order a ground vehicle on the same deck,
out of range, to attack it. Record whether it keeps the target and its first destination. Repeat
with the attacker on a bank.

## Current workaround

None. The code follows claim 19 as written: the candidate's ground region, so the deck cells over
water are refused and the attacker drops the target. A wrong first reading of the attacker's own
region was fixed in the same round (the actor's region now comes from its deck, as claim 19 says).
Questions 1 to 3 are about the candidate side; question 4 about S-0215 claim 37 (a)'s gate.
