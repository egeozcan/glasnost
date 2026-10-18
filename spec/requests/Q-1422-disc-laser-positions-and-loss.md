---
id: Q-1422
title: Which points a disc laser's range re-check measures, and whether a killed target counts as lost
status: answered
related: [S-0615, S-0204]
answered-by: [S-0615]
---

## Need
S-0615 claim 14 measures "from the firer's position to the target's position", and cancels the
disc "as soon as its firer or its target leaves play: goes off the map, is removed, or otherwise
stops being a valid object to point at". The engine needs the exact points and the exact loss
test. Code: `crates/deterministic/sim/src/disc.rs`, `recheck_distance` and `in_play`.

## Questions
1. Which point of the firer is measured: its position (centre at its world height), or another
   point such as its fire point? Which point of the target: its position, its centre, or the
   target point where shots aimed at it detonate (which claim 16 uses for the final beam)?
2. For a target that is a cell (a ground shot), is the point the cell centre at ground height?
3. Does a target that has been killed in the same tick but is not yet removed (health 0, still
   listed) cancel the disc? Does an infantry unit or vehicle that has entered a transport, a
   building or a bunker count as gone?
4. Is the loss test made in every update, or only at steps (every second update)? A firer that
   leaves the map and comes back between two steps would tell them apart.

## Possible observation
S-0615 scenario DL2 with targets whose centre and target point differ in height (a building, an
infantry unit on a slope), and DL3 with a target that enters a transport in an even update.

## Current workaround
The firer's reference point and the target's target point. An object is lost when it does not
exist, is off the map, is inside a building or bunker, or is killed (health 0); this is tested in
every update, so a loss between steps cancels the disc too.
