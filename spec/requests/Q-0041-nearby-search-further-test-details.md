---
id: Q-0041
title: The further test of the nearby-cell search, in the no-move rows of table T5 and on bridge structure
status: answered
related: [S-0212, S-0010]
answered-by: [S-0010, S-0212]
---

## Need
`glasnost_sim::path::nearby` runs the nearby-cell search of S-0212 claim
25, with the further test that S-0212 r15 names as the burrowing test of
S-0010 claim 17. Two parts of it are not stated far enough to implement.

## Questions
1. **No-move rows of table T5.** For a no-move order onto a cell or onto an
   object by a unit that is in the air, travels underground, is of a
   jumpjet infantry type or is an aircraft, the search asks for the further
   test "for a unit of movement zone *subterranean*, under a further
   condition of the unit's state that is not established". Which state of
   the unit is that (for example: it is underground at the moment of the
   click, it is burrowing, or it is on the surface)? Is the destination
   tested first, as in claim 24 (the test applies to the candidates only
   when the destination passes it)?
2. **Bridge structure in the burrowing test.** S-0212 claim 25 says the
   original "also refuses some other cells of a bridge's structure, by a
   mark it keeps on them that was not identified further (S-0010 owns the
   full rule)", and S-0010 claim 17 says "among other conditions". Which
   cells besides bridge cells (cells carrying a high bridge's deck) fail
   the test: the bridge ends, the cells of a low bridge, the cells under a
   bridge's ramps, or cells marked by some other overlay? Are there other
   conditions besides being flat, not a bridge cell, and holding no
   building and no terrain object on the ground layer?

## Possible observation
1. A unit of movement zone *subterranean* given a no-move order onto an
   occupied cell next to a sloped or built-on cell, once while it travels
   underground and once while it stands on the surface; record the cell it
   heads for.
2. S-0010 scenario G3 with the tunnelling unit ordered to come up beside a
   high bridge's end, beside a low bridge and on a ramp cell.

## Current workaround
1. The no-move rows never ask for the further test (marked "Q-0041" in
   `path.rs`).
2. `path::burrow_ok` applies only the stated conditions; the tileset's
   allow-burrowing flag reads as its absent-key default (allowed), since
   the tile catalogue does not carry it yet. Both are in the provisional
   table of `docs/development.md`.
