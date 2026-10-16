---
id: Q-0990
title: Animation of an assault's removed occupant at list position 10 or later
status: answered
related: [S-0508, S-0313]
answered-by: [S-0508]
---

## Need

S-0508 claim 14 gives the assaulter weapon's occupant animation of the occupant at list position
*i* the building's muzzle point *i*, for the supported point numbers 0 through 9, with no wrapping
by the number of configured points. For an occupant at position 10 or later the claim says there is
no supported animation point and no specified safe original result, and S-0508 (Questions for the
project lead) asks for a safety policy beyond point 9. A building with an occupant capacity above
10 can only come from user data (stock data has none), but our engine must do something
deterministic in `garrison::assault`.

## Questions

1. For a removed occupant at list position 10 or later, should the engine show its occupant
   animation at some point (which one: point 0, point 9, the building's base point), or none?

## Possible observation

None in the original (the result is unsafe there); a design decision.

## Current workaround

Interim rule: no occupant animation is created for a list position of 10 or later; the occupant
is still credited and removed in the reverse list order. Positions 0 to 9 follow claim 14.
