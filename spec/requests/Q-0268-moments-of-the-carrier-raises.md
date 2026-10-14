---
id: Q-0268
title: The moments of the entered-by and destroyed-by-anything raises
status: answered
related: [S-0011, S-0207, S-0208, S-0209, S-0312]
answered-by: [S-0312, S-0207]
---

## Need
S-0011 claims 18 and 19 say when a carrier raises its tag (a unit arriving in a tagged cell, an
object that carries a tag being destroyed) and leave the moment to the movement specs and the
combat specs ("a pointer only"). None of S-0207, S-0208, S-0209 or S-0312 names that moment, so our
engine raises neither. Stock skirmish maps can carry cell tags and tagged objects, so their
triggers stay silent until these moments are specified.

## Questions
1. Arrival: at which point of a ground vehicle's, an infantry unit's and a teleporting unit's
   movement in a tick does its arrival in a tagged cell raise the cell's tag (when it enters the
   cell's centre, when its occupation moves to the cell, or when it stops there)? Does a unit that
   only passes through raise it?
2. Destruction: where among the death steps of S-0312's table (debris, passengers and death
   weapon, the kind's death sequence, removal) does the destroyed-by-anything raise come, and so
   which of those steps' draws and effects come before the trigger's actions? The claim says the
   raises with codes outside the subset that an attacker causes come before it.
3. Does an infantry unit in its death sequence still carry its tag (claim 24's present-object rule
   excludes it), and is it raised again when it is finally removed?

## Possible observation
A tagged vehicle with a one-time destroyed-by-anything trigger whose action is an explosion at a
waypoint, killed by a weapon whose warhead leaves debris: record the order of the debris and the
explosion with the random-state recorder. For arrival: a cell tag with a one-time entered-by trigger
that sets a variable tested by a second, polled trigger; record the tick of the second firing
against the unit's cell positions.

## Current workaround
Neither raise is made (crates/deterministic/sim/src/triggers.rs, module docs; `triggers::raise` is
ready for them). Marked in docs/development.md, section *Map triggers (M5b)*.
