---
id: Q-0223
title: Which updates of a driving vehicle are its cell-arrival updates for the clear report
status: open
related: [S-0417, S-0207, S-0409]
answered-by: []
---

## Need

S-0417 claim 7 sends a tethered vehicle's clear report "on a cell-arrival update", and claim 8
adds a fallback scatter at a tethered vehicle's "arrival update". Scenario U8 expects the
report at the exit apron E, at the end of the exit track. S-0409 Terms define *arrival in a
cell* as the tick a walking or driving unit arrives in a new cell "as the movement specs use the
word"; S-0207's scenarios count arrival at a cell as reaching its centre. The exit track of
S-0417 table T1 crosses about three cells from its first point to E, and its points do not pass
through the centres of the cells before E. An ordinary track can also cross cell boundaries
before its target.

It is not stated whether the clear report is tested when a vehicle crosses into a new cell,
only when it reaches a track target (a cell centre), or at another moment, and whether
reaching a track target in the middle of a longer move counts.

## Questions

1. For a tethered driving vehicle, which events are cell-arrival updates: crossing into a new
   cell, reaching a track target, or both?
2. On the exit track of T1, can the report happen before E (for example as the vehicle enters
   the cell west of E with no destination)?
3. Does a temporarily driving (teleporting or tunnelling) vehicle on the exit track arrive the
   same way?

## Current workaround

A driving vehicle's cell-arrival update is reaching a track target (the end of each track, at a
cell centre), including the exit track's target E and track ends in the middle of a move. A
walking unit's is the arrival tick of S-0208 claim 6. Interim row in `docs/development.md`
(Q-0223).
