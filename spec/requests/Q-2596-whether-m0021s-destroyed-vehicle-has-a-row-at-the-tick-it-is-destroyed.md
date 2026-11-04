---
id: Q-2596
title: Whether M-0021's destroyed vehicle still has a per-tick row at the tick it is destroyed
status: open
related: [S-0203, S-0312, S-0302]
answered-by: []
---

## Need

M-0021 (revised for Q-2536 and Q-2546), Result, *Reading points*, gives for every vehicle
destroyed by a hit at *t* (part B): "Last per-tick row of a vehicle destroyed by a hit at *t*: the
row stamped *t*, with no health left." The per-tick rows list every vehicle's position and health,
read after tick *t* has run.

S-0203 claim 6 (a) says a vehicle whose damage destroys it leaves play at the moment that damage
is applied, at the end of its death effects, and takes no further turn, "not even later in the
same turn pass" (S-0312 claim 15 says the same for every destroyed vehicle outside its sinking and
crash exceptions, which do not apply to a ground tank at ground level on a land cell). So in our
runs a vehicle destroyed by a hit at *t* is no longer one of the game's objects when the rows
stamped *t* are read: it has no row stamped *t*, and its last row is the one stamped *t* − 1, with
health left; this holds for every vehicle destroyed in our part B runs (the harness reports the
record's row as failing for all of them). The harness's rows come from the objects the world
still holds, and it does not make a row at *t* from the hit's health after, since that would copy
the record's answer.

## Questions

1. In the record's runs, at the end of tick *t*, after a hit at *t* destroyed a test tank, is that
   tank still one of the game's objects with no health left, and does it leave in tick *t* + 1
   (or later)? Or is the row stamped *t* the recorder's last read of a tank that left play at the
   hit?
2. Does the answer hold for both kinds of destroyed tank: a G tank destroyed in an F vehicle's
   turn (its own turn is still ahead in tick *t*), and an F tank destroyed in a G vehicle's turn
   (its own turn has already been taken in tick *t*)? Does either take a turn after the hit (a
   movement step, a firing step) in tick *t*?
3. If a destroyed tank is still an object at the end of tick *t*, which spec states it, and is
   S-0203 claim 6 (a) ("leaves play at the moment that damage is applied") then wrong for these
   tanks?

## Possible observation

The record's own rows answer question 1 for any part B run: whether the destroyed vehicle appears
in the row stamped *t* and not in the row stamped *t* + 1. Question 2 needs, for the same
vehicles, whether a shot is recorded for a destroyed vehicle at the tick of its destruction after
the hit.

## Current workaround

None in the simulation: S-0203 claim 6 (a) and S-0312 claim 15 are built as written. The harness
shows the row as failing with our last row and changes no rule.
