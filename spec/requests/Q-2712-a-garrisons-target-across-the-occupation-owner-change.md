---
id: Q-2712
title: A civilian garrison building's search in the entry tick and its target across the owner change
status: open
related: [S-0508, S-0309, S-0509, S-0215, S-0311, S-0203]
answered-by: [S-0508]
---

## Need

S-0309 claim 2 (since round 8) lets an occupiable building that holds occupants search passively.
S-0508 claim 7 changes a civilian garrison building's owner to its first occupant's house "at each
of its own turns", and says that in the tick of the entry the building may still belong to the
civilian house. No spec found places that owner change within the building's turn relative to its
behaviour processing, search and firing. None says whether a holder whose *own* owner changes
keeps its target either. S-0215 claim 10 and S-0309 claims 27 to 29 drop or revert on events of
the target, and claim 25 part k drops allied targets only for computer-controlled owners. The
firing check refuses no target for being the firer's own house's (S-0301 claim 9; a forced attack
on an own object passes).

What the specs do give: while the building is still the civilian house's, its search treats the
occupant's house as any house the civilian house is not allied with. No claim found allies the
civilian or neutral house with a player house (S-0508 claim 1 lets a player's occupiers into a
passive house's building but changes no alliance). Table C's "friendly" row (S-0309) rejects only
allied targets, and its "passive house" row only targets *owned by* a multiplayer-passive house.
So in that turn the occupant's house's objects are candidates. The questions below are about what
comes after.

Seen in the local lane (`crates/app/tests/r8_render_lane.rs`, `note_49_a_veteran_goes_into_a_building`,
on main bf6a19b, an install map, the local house human-controlled):
- A civilian garrisonable building stands 7 cells from the local base, with a few of the local
  house's infantry nearby.
- One local infantry unit enters it.
- In the entry tick, in the building's turn, the building (still the civilian house's, with one
  occupant) takes a local infantry unit as its target by its passive search.
- Later in the same turn S-0508 claim 7 makes the local house its owner.
- The building keeps that target, now a unit of its own house, and fires.
- 81 ticks later three of the local house's infantry die in one hit.
- The building's type is trainable, so by S-0311 claims 7 and 8 the building is the recipient of
  allied kills of value 0. Under F3 and `quirks.zero_kill_value_caps` with an actual cost of 0,
  its experience goes to the cap.
- It so shows an elite insignia (S-0111 claim 18) before any fight, the case of the lead's note 49.

In our engine the owner change runs late in the building's turn (`econ::events::turn` calls
`structures::garrison::turn`), after the behaviour processing and the search. That order was
chosen with no claim to place it.

## Questions

1. In a civilian garrison building's turn, does S-0508 claim 7's owner change come before or after
   the building's behaviour processing, its target search and its firing step? Is the same order
   kept in the turn of the entry tick?
2. When a building changes owner by S-0508 claim 7 (or claim 8, or any building ownership change
   of S-0509 claim 12) while it holds a target, does it keep the target? In particular, does it
   keep a target that the change makes its own house's or allied with it? If it is dropped: at
   the change, at its next turn, or at claim 29's next check tick? Is the timer draw of claim 27
   taken?
3. Does the same hold for a unit whose owner changes other than by mind control (S-0601 gives
   mind control a guard reset)?

## Possible observation

On an install map:
1. Place a civilian garrisonable building near three infantry units of the human player's house.
2. Order one more infantry unit of that house into it. No enemy should be in range.
3. Record per tick:
   - the building's owner and target;
   - its shots, and the health of the nearby units;
   - its experience, by the insignia drawn.
4. Repeat with the nearby units belonging to an allied house, and with an enemy unit in range too.

## Current workaround

None: the engine keeps the order and the target as described, so a building garrisoned next to its
owner's units can fire at them in the entry tick's turn and gain experience from it. No interim is
added. The lane's scene is the only case seen so far.
