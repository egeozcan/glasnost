---
id: Q-0340
title: Which production clicks request the building announcement and which the training one
status: answered
related: [S-0701, S-0411]
answered-by: [S-0701]
---

## Need

S-0701 claim 28 lists *building* and *training* as separate production-feedback announcements, raised by
"the matching production-feedback click of S-0411 claim 12". Revision 3's Import mapping gives
each one its own fixed entry (`EVA_Building` and `EVA_Training`). Neither spec says which click
requests which. S-0411 claim 12 lists the clicks that start, queue, resume, hold or cancel an
item, but it does not split them into building and training. `Audio::cue` in
`glasnost_audio::events` gets `UiCue::BuildStarted` and `UiCue::BuildResumed` from the client and
has to pick one entry.

## Questions

1. Does a start or queue click on an infantry, vehicle or aircraft item request the training
   entry, and one on a building or defence item the building entry?
2. Does a resume click request the same entry as a start click on that item?
3. Does a naval item, or any other production kind, follow the unit rule or the building rule?
4. Does a queue click that adds a second item of a type already being produced request the
   announcement again?

## Possible observation

Use S-0701 scenario G4's announcement test data, with distinct samples for the two entries. Start,
queue and resume one item from each sidebar tab. Record which entry each click requests.

## Current workaround

Every start, queue and resume click requests the building entry, whatever the item's kind. The
training entry is never requested. Documented in docs/development.md, "Audio (M5b)".
