---
id: Q-1152
title: The nearby-location search of an absorber's waiting-in-line approach
status: answered
related: [S-0605, S-0212, S-0507]
answered-by: [S-0605]
---

## Need

S-0605 revision 2, claim 4 (*Approach*), says that an actor waiting in line for an absorbing
building, at its first *enter* processing after the wait began, "chooses a free location near the
building's centre cell, of the kind the actor's own movement class can stand on (the nearby-location
search of claim 12; its exact tie-breaks are open)". Claim 12 itself uses S-0507 claim 10's exit
search, which is a ring search around the building's top-left cell, not a search from the centre
cell. Our engine must pick one search with full settings to place the waiting actor. The code is
`approach` in `crates/deterministic/sim/src/structures/absorb.rs`.

## Questions

1. Which search does the approach use: S-0212 claim 25's nearby-cell search from the building's
   centre cell, S-0507 claim 10's exit ring, or another one?
2. With which settings: movement zone, a reach limit, bridge decks and heights, overlay, and whether
   the cells of the building itself and cells reserved by other actors count as free?
3. When several cells are equally near, which one wins?
4. What happens when no location is found: does the actor stay where it is with *enter* current
   (so that the approach is tried again at its next processing), or does something else happen?

## Possible observation

S-0605 scenario A2c already records where a waiting actor walks. Repeating it with the building
surrounded on all but one side, then on all but two equally near sides, with the waiting actor
starting on each side in turn, would settle questions 1 to 3. Blocking every free cell around the
building would settle question 4.

## Current workaround

Interim rule, cited as "Q-1152 interim" in `absorb::approach` and noted in `docs/development.md`
(Simulation, M7a: Infantry absorber): S-0212 claim 25's nearby-cell search from the cell of the
building's reference point (the centre of its foundation, S-0204 claim 4), with the settings of
S-0507 claim 10's vehicle exit (normal zone, no reach limit, no deck or height test, no bridge or
overlay exclusion) and the actor's own movement class. When nothing is found, the actor's
destination and behaviours are left as they are and it keeps waiting.
