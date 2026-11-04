---
id: Q-2506
title: What a processing of enter's docking stage does for an aircraft in contact that is in flight and not landing
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 r110 claim 24 gives three things for a processing of *enter*'s docking stage in contact:
while the aircraft is landing it moves it toward its contact place's docking position; once the
aircraft is on the ground it reports that it has entered; and the processing returns 1 tick and
sets the ready mark (claim 31). It gives no outcome for a processing in contact in the docking
stage that finds the aircraft in flight and not landing.

The engine reaches that state when the touchdown of an aircraft in contact is refused (claim 7:
it rises by 10 leptons and starts to take off again). F7 accepts the contact's own cells, so this
happens where the docking position lies outside the pad's foundation (claim 24, *Not pad-bound*)
and an occupant F7 refuses stands in the cell where the aircraft touches down. It can also happen
when the second rule's load request (claim 24) restarts entry for an aircraft that is in flight,
which leaves the docking stage behind, but that case starts entry again and is covered.

Before r110 the engine did not report entry away from the foundation, so it sent such an aircraft
back to the approaching stage. r110 removes the position test, so that path is gone and the
engine must decide.

## Questions

1. An aircraft in *enter*, in contact, in the docking stage, whose touchdown is refused, is in
   flight and not landing at its next processing of *enter*. Does that processing keep it in the
   docking stage, take it back to the approaching stage, or do something else? What delay does it
   return, 1 tick or 3?
2. If it stays in the docking stage, does that processing move it toward the docking position as
   a landing one is moved, or leave its position to its flight?
3. When it starts to land again, which processing moves it on (or back on) to the docking stage,
   and does that processing clear its ready mark again (claim 31)?

## Possible observation

AC31 part h (1)'s map and pad, with a vehicle of another house parked, not moving, on the cell
that holds the docking position. Record each tick *A*'s height, whether its landing is under way,
its position, its current and queued behaviours, and the tick of each of its processings, from the
tick its landing first starts until it reports or leaves *enter*.

## Current workaround

The engine sends the aircraft back to the approaching stage, as it did before r110: that
processing sets the ready mark (it is a processing of the docking stage), marks the approaching
stage and returns 3 ticks. The approaching stage moves it on again when it next finds it landing.
