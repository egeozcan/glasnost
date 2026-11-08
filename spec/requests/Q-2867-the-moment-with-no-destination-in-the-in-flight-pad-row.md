---
id: Q-2867
title: The moment with no destination in the in-flight pad row
status: open
related: [S-0501, S-0215]
answered-by: []
---

## Need

S-0501 r167 claim 29 (*A contact held at cruise height*) says of claim 18's in-flight pad row: "In
that row claim 21's search runs, and the aircraft is given no destination for a moment, which gives
its flight claim 6's new flight point around the cell it is over, with claim 40's direction draws
when that cell is refused (Q-2507). It then asks the pad found for contact by the in-flight pad
row's own request (claim 18) ... and, when that pad grants it, gets it as destination." A
processing that comes while the aircraft is landing at the pad can have its landing undone by that
moment.

Claim 18 and table T2, which give the in-flight pad row for every fall-back, do not mention the
moment: the row reads "a pad is found and grants contact | destination: the pad; *enter*". Claim 29
states it only for an aircraft in *sticky* that holds a pad's contact with *enter* queued. Our
engine builds the row in `air::fall_back_rows` (`crates/deterministic/sim/src/air.rs`). Applying
the moment at every use of the row would add claim 40's draws, and the landing undo, to every
fall-back in flight of an armed aircraft with dock types; applying it only where claim 29 states it
leaves the row as claim 18 gives it elsewhere.

## Questions

1. Is an armed aircraft with dock types that falls back in flight and reaches the in-flight pad
   row given no destination for a moment between the search and its contact request whatever its
   current behaviour (*guard*, *area guard*, *attack* with no ammunition, *move*, *enter* without a
   contact, *sticky* without a contact)?
2. If not, which states does it apply in?
3. When the row's pad refuses the contact (so that the row gives F8's landing spot and *move*, or
   the crash of a pad-bound aircraft), is the moment still taken before that?

## Possible observation

The moment leaves only claim 40's direction draws and, for a landing aircraft, the landing's undo.
An aircraft in *guard* at its cruise height that falls back while landing at a pad cell outside its
flight point's cell (AC33 part c's set-up with the aircraft in *guard*, and a second aircraft headed
for the cell it is landing in) would show the undo: it takes off toward the cell claim 40 selects.

## Current workaround

Our engine gives the moment only where claim 29 states it: in `air::fall_back_rows`, the in-flight
pad row of an aircraft with *sticky* current that holds a contact with *enter* queued (`// design:
Q-2867`). Every other use of the row asks the found pad for contact without the moment, as before
r167.
