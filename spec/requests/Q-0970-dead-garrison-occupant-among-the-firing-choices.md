---
id: Q-0970
title: What a garrisoned building does when its firing index names a dead occupant
status: answered
related: [S-0508, S-0308, S-0313, S-0311, S-0414]
answered-by: [S-0508]
---

## Need

S-0508 claim 19 says that the defeat of an occupant's own house kills that house's occupants one by
one while the building belongs to a surviving house, that such a death retains the numeric
firing index (no reset, no reduction modulo the new count) and does not compact the occupants,
and that "the original can consequently retain a dead occupant among its firing choices". S-0508
Open questions and Questions for the project lead, S-0308 claim 13, S-0313 claim 6 and S-0311
claim 8 all leave the result of firing, muzzle selection and experience through that dead choice
unspecified and ask for a project-lead decision ("No safe continuation is invented here").
Our engine removes the dead occupant object from the game, so the building's occupant list holds
an identifier that names no object, and `structures::garrison` has to decide something when the
index lands on it (`combat::weapon::slot_weapon`, `combat::fire`, `combat::veterancy`).

## Questions

1. Does the dead occupant still count in the building's occupant count *n* (for the capacity test
   of claim 1, for the abandonment test of claim 8, for the shot-delay quotient of S-0301 claim 7
   and for the index modulus of S-0313 claim 6)?
2. When the index names a dead occupant, which weapon does the building fire (none, the building's
   own, the next living occupant's), which muzzle point does it use, and does the index advance?
3. Who receives the experience of a kill made through that choice (S-0311 claim 8)?
4. Is the dead occupant put out by a later eviction, and is anything placed for it?

## Possible observation

S-0508 scenario G15 with a third house's occupant added, then watch the building's shots after the
defeat.

## Current workaround

Interim rule, `docs/development.md` section *Simulation (M6c: garrisons)*: a dead occupant keeps its
place in the list and counts in *n* (so the index and the modulus are unchanged); a shot whose
index names a dead occupant uses the next living occupant's weapon in list order (the index
moves to it first), and a building whose occupants are all dead has no weapon from them; the
experience of such a kill is given to nobody (the building is never a recipient); an eviction
skips dead places.
