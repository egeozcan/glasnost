---
id: Q-0900
title: Which destination does a seeking harvester test, once its last track has ended in the field cell
status: open
related: [S-0402, S-0207, S-0215]
answered-by: []
---

## Need

S-0402 claim 3 says a harvester in the seeking phase that is not full, has no remembered cell
and "has a destination" remains seeking and runs no ring search; it starts *loading* only when
the search finds its own cell and it has no destination. S-0207 claim 26 (final track) says the
qualifying track completion clears the locomotor destination point in the completion tick but
leaves the order-level destination in place until the next locomotion update that begins
without a track, which removes it.

A harvester that drives to a field cell therefore ends its last track in that cell with the
order-level destination (the field cell) still present for one more locomotion update. If the
seeking processing is due in that window, "has a destination" decides whether it starts loading
at once or waits one more processing interval (the standard delay of *harvest*, S-0215 claim 6).
The code follows the literal reading (the order-level destination counts) and the engine
currently waits for the next processing: in a scripted skirmish a harvester's track ended in its
field cell at one tick, the order-level destination was removed at the next, and the harvester's
next decision came about ten ticks after the decision it made before claim 26 was implemented.
Such shifts moved the income timing of whole games (build orders moved by tens of ticks, and
with them the winner of the first clash). Neither spec says which reading is meant.

## Questions

1. When the seeking phase decides between "remains seeking" and "starts loading", is the
   destination it tests the order-level destination, the locomotor destination point, or either?
2. For a harvester whose last track has just ended in the field cell it was sent to (so the
   order-level destination is that same cell and still present), does it start loading in the
   first processing after arrival, or only after the following one?
3. How many ticks pass between the tick of the track's completion and the first load-step-bearing
   processing, as a distribution over several arrivals (compare S-0402 F1, which counts from the
   start of loading)?

## Possible observation

Send a harvester to a field cell from several distances and record, per arrival, the tick of the
track completion, the tick at which its phase becomes *loading*, and the tick of the first bale
(S-0402 H1 and H2 record the later ticks; the earlier two are the new observation).

## Current workaround

`econ::harvest::seeking` tests the order-level destination (`Object::dest`), as claim 3's wording
reads. Noted in `docs/development.md` (Own AI section, M6c step 2a) with this request's ID. No
interim value is invented.
