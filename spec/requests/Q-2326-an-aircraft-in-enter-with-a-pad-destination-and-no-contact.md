---
id: Q-2326
title: What a processing of enter does for an aircraft with a pad destination and no pad contact
status: open
related: [S-0501, S-0215]
answered-by: []
---

## Need

S-0501 r93 claim 24 says that each processing of *enter* asks only the pad the aircraft is in
contact with to be loaded, "asks no pad that it is not in contact with and makes no contact
itself", and that "the unlinked retry and fallback paths also return 1 tick". Claim 18's search
after the row (r93, Q-2251) now gives an armed aircraft with its ammunition spent the pad as
destination and *enter* without any contact, so *enter* can now be current with a pad
destination and no contact in ordinary play, not only after a lost reservation. No claim says
what that processing does then. The "unlinked retry" is named but not described.

Our engine (`air::enter`) treats a processing of *enter* with no contact as claim 24's lost
reservation: it clears the destination and falls back to rest (claim 18), returning 1 tick.
For an aircraft in flight, T2's in-flight pad row then asks a pad for contact by its own request,
so it reaches its pad after one more fall-back; on the ground it takes T2's ground row and the
search after the row again. Whether the original does the same, or keeps the destination and
waits (a retry), decides when the aircraft takes off and docks after a fall-back with its
ammunition spent.

## Questions

1. An aircraft has *enter* current, a pad of its dock types as destination, and no pad
   contact, and the pad has a free contact place. At its next processing of *enter*, does its
   destination stay the pad or is it cleared? Does it fall back to rest in that processing?
2. If it does not fall back: in which later processing, or on which event, is a contact made
   (its own retry request to the pad, its arrival over the pad, or none at all)? Does it ever
   land on the pad without a contact?
3. The same with the pad's every contact place taken: does anything differ?
4. Does the answer differ on the ground (just accepted, its take-off under way) and in flight?

## Possible observation

S-0501 scenario AC31's harness: a landed aircraft of a test copy of a stock armed, landable,
not pad-bound type with its ammunition spent and no contact, in *guard* on open ground, its
house owning one helipad of its dock type with a free place 10 or more cells away. Make it fall
back to rest once (for example by a capture behaviour on open ground, S-0215 claim 28 row 2)
and record, per tick for 600 ticks, its current and queued behaviour, destination, pad contact,
height and the pad's contact places. Run again with the pad's place taken by another aircraft.

## Current workaround

`air::enter` keeps its earlier rule: no contact means a cleared destination and a fall-back to
rest, with the 1-tick delay of claim 24's unlinked and fallback paths. Cited to claim 24; the
intake 76 ledger (docs/m8-claims/intake76.md) lists it as a seam.
