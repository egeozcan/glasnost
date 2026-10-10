---
id: Q-0045
title: Start lists and starting units, details the specs leave open
status: open
related: [S-0009, S-0208, S-0209, S-0215]
answered-by: []
---

## Need
`glasnost_sim::starts` builds S-0009's start lists (claims 17 and 18) and
assigns positions (claims 19, 20 and 33), and `glasnost_sim::starting`
gives each house its starting units (claims 45 and 46). A few details that
decide which cells accept a unit, which draws are taken and what the
non-default quirk switches do are not stated.

## Questions
1. **Acceptance test of the placement search (S-0009 claim 46).** A cell
   accepts a starting unit when it "holds no infantry, vehicle, aircraft or
   building, or the unit is an infantry unit and the cell's occupant nearest
   to its centre is an infantry unit".
   a. On a bridge cell (a cell with a bridge deck, S-0211 claim 15), are the
      objects on the ground, on the deck, or on both counted?
   b. For "the occupant nearest to its centre": which objects are
      occupants (do terrain objects count, and does a building count in
      every cell of its foundation)? Is the distance measured in the plane
      or in 3-D, from each object's position or its reference point
      (S-0204 claim 4)? Which occupant wins a tie?
2. **`quirks.starting_infantry_share_centre` off (S-0009 Edge cases).** The
   unit takes the standing position S-0208 claim 3 gives for class centre,
   "with that claim's draw", and a cell with no free position does not
   accept it. Is that draw taken each time the search tests a cell for the
   unit, or only once, on the cell that accepts it?
3. **`quirks.start_waypoint_gap` off (S-0009 Edge cases).** "Position i is
   waypoint i whenever waypoint i is defined, and only undefined waypoint
   markers become random." Is N still max(P, K) with K the count of
   consecutive start waypoint markers from 0, or does K change (for
   example to one more than the highest defined start waypoint marker)?
4. **`quirks.random_start_asymmetric_range` off (S-0009 Edge cases).** Both
   coordinates are drawn from 11 … W + H − 10. Which of the two ranged
   draws gives x and which gives y?
5. **Skill level of the neutral and special houses (glossary *house skill
   level*; S-0209 claim 11; S-0215 claim 15).** In a skirmish the original
   gives every computer-controlled house the rules' maximum skill level
   when the houses are set up. Do the neutral and special houses get that
   maximum too, or do they keep level 0?
6. **The unused-house cleanup (S-0009 claim 28) in our engine.** It runs
   only in a game with no computer players. Do AI houses (glossary *AI
   house*) count as computer players for that condition?

## Possible observation
Questions 1, 2 and 5 by S-0009's scenario M26 (variants C and D) with a
pre-placed infantry unit and a vehicle on a bridge cell next to the base
cell, and S-0209's scenario C7 with a fraidycat-free unit of the neutral
house. Questions 3, 4 and 6 are decisions for our engine's switches.

## Current workaround
1. The ground layer's list is read on every cell, bridge cells included;
   occupants are infantry, vehicles, aircraft and buildings; the nearest is
   the one with the smallest squared plane distance from its position to
   the cell's centre, the first listed on ties.
2. The draw is taken each time a cell passes the other conditions.
3. N is kept as with the switch on.
4. The first draw gives y and the second x, as with the switch on.
5. The neutral and special houses keep skill level 0.
6. The cleanup is not implemented (a note in the world's notes).
