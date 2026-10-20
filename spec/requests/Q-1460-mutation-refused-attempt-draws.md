---
id: Q-1460
title: The random draws of a refused creation attempt at the end of an infantry-making animation
status: answered
related: [S-0611, S-0202, S-0109, S-0208]
answered-by: [S-0611]
---

## Need
S-0611 claim 11 says that when an infantry-making animation cannot place its unit, no unit
appears, the animation steps back one frame and tries again, and each attempt takes its own
position-choice draw. It also says the original builds a unit on each refused attempt that is
never placed, while our engine builds none (documented departure). S-0202 claim 10 gives every
newly created object one raw creation draw. The spec does not say whether that creation draw is
taken on a refused attempt, nor where it falls against the position-choice draw. Because both
come from the scenario stream, the answer decides every later draw of the game after a refused
attempt. The code is `attempt` and `discard` in `crates/deterministic/sim/src/mutation.rs`.

## Questions
1. On a refused creation attempt (for example a vehicle standing in the animation's cell), does
   the scenario stream advance by the creation draw of the never-placed unit as well as by the
   position-choice draw? Is the same true of a successful attempt?
2. In which order are the two draws taken: the creation draw first, then the position choice?
3. Does anything else that creating a unit draws or numbers (object numbers, group numbers)
   advance on a refused attempt, so that the next object created afterwards gets a different
   number than if the attempt had not happened?

## Possible observation
MU9 of S-0611 with the scenario stream recorded: hold a vehicle in a conversion victim's cell for
several animation frames on a centre-class point, count the draws per retry, and compare the
object number of the next unit built by a factory with a run without the vehicle.

## Current workaround
Each attempt, refused or not, takes the creation raw draw and then the position-choice draw.
A refused attempt then gives the unit and its object numbers back, so later objects number as if
it had never been made. Marked Q-1460 in the code.
