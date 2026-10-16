---
id: Q-0973
title: Order of occupant removal in an assault and which occupant shows its animation where
status: open
related: [S-0508, S-0313, S-0311]
answered-by: []
---

## Need

S-0508 claim 14 removes every occupant during a passing assault attempt, each with the assaulter
weapon's occupant animation "at a muzzle point described by S-0313 Terms", and says that "the
removal order and the correspondence between an occupant and an animation position are not
specified here". The reversed kill credit (a notification and bookkeeping pass per occupant) also
depends on the order. Assaulter types are not used by stock data, so only synthetic tests and
user data meet this.

## Questions

1. In what order are the occupants removed and credited (list order, reverse list order)?
2. Which muzzle point (by number) carries the animation of the occupant at list position *i*?
3. Does the muzzle index of claim 18 take part (does the animation of the first occupant stand at
   the point of the current index)?

## Possible observation

G10 with differently owned or differently typed occupants and an assaulter weapon whose animation
differs by position.

## Current workaround

Interim rule: occupants are removed and credited from the last to the first, as claim 11's put-out
order goes; the occupant at list position *i* shows its animation at muzzle point *i* (the 0-based
position in the building's list, wrapping modulo the number of muzzle points); the muzzle index
plays no part.
