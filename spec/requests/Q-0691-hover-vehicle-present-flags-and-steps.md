---
id: Q-0691
title: When a hovering vehicle sets and clears vehicle-present flags, and when it is between steps
status: answered
related: [S-0503, S-0209, S-0212]
answered-by: [S-0503]
---

## Need

S-0503 claim 6 says that a hovering vehicle's occupancy is moved from cell to cell as its
position crosses a cell edge. S-0209 claim 1 gives the moments at which a driving vehicle sets
and clears its vehicle-present flags (the flag of the cell a step enters is set when the step
begins; the flag of the cell it leaves is cleared when it first advances). Its Open questions
say that when a hovering vehicle sets and clears these flags is not established. S-0209 Terms
also reads a hovering vehicle as *between steps* "at the end of each step" with medium
confidence, without saying when a hover's step ends.

Separately, S-0212 claim 8 sends the heading cell of a unit along its body facing when "its
current speed is not zero". For a hovering vehicle the spec does not say which value is its
current speed: the hover speed φ, the step length *d* of the last tick, or *s* × φ.

These decide the entry outcomes other vehicles see near a moving hover (S-0209 claims 4 and 5),
the mode-0 blocker chain of the path search, and the code in
`crates/deterministic/sim/src/hover.rs`, `occupancy.rs` and `path.rs`.

## Questions

1. When a hovering vehicle chooses a heading point in a new cell (claim 5, outcome *clear*), is
   that cell's vehicle-present flag set at once, as for a driving vehicle's step?
2. When is the flag of the cell it leaves cleared: when its position crosses the cell edge, when
   it arrives at the next heading point, or at another moment? Are cells it only swings through
   at a turn (claim 7) flagged?
3. Is a hovering vehicle *between steps* from the arrival at its heading point until it chooses
   the next one, or only when it has no heading point?
4. For S-0212 claim 8, is a hovering vehicle's current speed non-zero when φ > 0, or when the
   last tick's *d* > 0?

## Possible observation

Use S-0503 scenario H3 with a second vehicle (a walker) ordered into the cell the hover is
heading for, and into the cell it is leaving, on chosen ticks of the hover's step. Record the
walker's entry outcome on each tick. A walker refused by a flag shows when the flag is set and
when it is cleared.

## Current workaround

As for a driving vehicle: the flag of the cell of a newly chosen heading point is set at once,
and the flag of a cell is cleared when the hover's position leaves that cell, unless it is the
cell of the current heading point. Cells it only swings through are not flagged. A hovering
vehicle is between steps while it has no heading point. Its current speed for S-0212 claim 8 is
non-zero when φ > 0. Each choice is marked in the code with this request's ID.
