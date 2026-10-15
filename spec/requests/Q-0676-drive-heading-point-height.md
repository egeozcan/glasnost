---
id: Q-0676
title: The stored height of a driving or ship vehicle's heading point
status: answered
related: [S-0511, S-0207, S-0504]
answered-by: [S-0207, S-0511]
---

## Need

S-0511 claim 11 lets a drive or ship vehicle count as heading to a lost deck cell by "that same
heading-point test": the heading point's cell is the blown-up cell and its *stored world height*
differs from the deck point's by at most 104 leptons. It adds a second test with the end cell of
the current track and the unit's own height. For walking and hovering the heading point's height
is defined (S-0208 claim 18, S-0503 Terms). For the driving and ship locomotors S-0207 Terms
defines the heading point as the target of the track the vehicle is on, but gives it no height.
`bridges::blow::heads_to` needs that height. Claim 9's clearing of a dropped vehicle's
reservation "at that point's stored height" (`bridges::blow::move_marks`) reads it too.

## Questions

1. What height does a driving or ship vehicle's heading point carry when its track begins: the
   vehicle's height at that moment, the terrain height at the track target, the deck height when
   the target cell is a high-bridge cell, or something else?
2. When a curve is chained and the heading point moves one cell on with the track target
   (S-0207 Terms), does the height change with it?
3. Can a drive vehicle on the ground under a high bridge, whose track ends in a deck cell under
   the deck, pass the claim 11 heading-point test (with a stored height near the deck point) when
   its own height fails the track test?

## Possible observation

S-0511 scenario B6 with a tank driving along the ground under a high bridge towards a cell below
a section, and a second tank driving on the deck towards the same cell, at the moment the section
is lost: which tank receives the hit.

## Current workaround

The heading-point test of a drive or ship vehicle uses the vehicle's own current height, so it
coincides with the track-end test. A dropped drive vehicle's reservations are cleared in the
deck layer, the layer they were made in. The choice is marked in the code with this request's ID.
