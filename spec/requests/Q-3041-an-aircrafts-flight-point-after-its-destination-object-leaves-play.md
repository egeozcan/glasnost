---
id: Q-3041
title: Between the removal of an aircraft's destination object and its next processing, what flight point does the aircraft fly to?
status: open
related: [S-0501]
answered-by: []
---

## Need

An aircraft's destination can name an object: the target T1 gives a strafer in *choosing*, the
building of *enter*, the unit of a *move* onto a unit. When that object leaves play (killed and
removed, sold, boarding a transport), the destination is not cleared at that moment; the aircraft
keeps it until its behaviour is next processed. S-0501 claim 32 says an aircraft that loses its
target "keeps flying in its current T1 stage until its next *attack* processing", and claim 24
says a destination that is gone "leaves the aircraft none" for *enter*, but neither says what
flight point (claim 5) the movement updates in between fly to.

GitHub issue #160 found that our code resolved the gone object's position to the map origin and
turned the aircraft toward the (0, 0) corner for up to the length of the stage's delay. That is
wrong under any reading; this request asks which of the sound readings is right.

## Questions

1. In the movement updates between the moment an aircraft's destination object leaves play and
   the next processing of its behaviour, does the aircraft
   (a) keep flying to the last flight point that destination gave it (the object's heading point
   at the last movement update before it left play), or
   (b) have no flight point, as an aircraft whose destination was cleared (no guidance, claim 42,
   and no slowing toward a point, claim 5), or
   (c) something else?
2. Is the answer the same for *attack* (a strafer's T1 destination), *enter* and *move*?
3. Does the answer depend on how the object leaves play (destroyed and removed after its death
   sequence, removed at once, boarding a transport or a building)?

## Possible observation

Two strafers ordered onto one vehicle; the first kills it during the second's standard delay in
*choosing*. Watch the second strafer's facing and track over the ticks until it next turns: under
(a) it keeps steering toward where the vehicle was heading, under (b) it holds the heading it had.

## Current workaround

Reading (b): a destination that names an object no longer in play gives the aircraft no flight
point at its movement updates (`air::flight_target`), as the *enter* processing already treats such
a destination as none. The destination itself is left as it is until the next processing. The code
and the test `a_strafer_whose_destination_object_is_removed_does_not_steer_to_the_origin` cite this
request.
