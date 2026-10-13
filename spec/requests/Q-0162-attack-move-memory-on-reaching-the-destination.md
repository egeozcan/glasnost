---
id: Q-0162
title: What happens to an attack-move memory when its destination is reached
status: open
related: [S-0309, S-0215, S-0213]
answered-by: []
---

## Need
S-0213 revision 8, claim 6, gives a mobile actor that is eligible for
attack-move *move* plus an attack-move memory holding the destination.
The memory is removed only by another behaviour order (S-0213 claim 6)
or by a resumption with neither a destination nor a target (S-0309
claim 23 (b)). S-0215 revision 14, claims 14 and 15, row 1, says a
fall-back changes nothing for an actor with the memory. S-0309 claim 23
(b) resumes the attack move only when the actor is engaged or in
*guard*, and says how such an actor comes to be in *guard* "is
S-0215's and S-0213's". Neither spec says so.

Implemented as written, an attack-mover that reaches its destination
without finding a target stays in *move* forever. It has no destination
and does not move. Claim 22 processes it every tick and each time falls
back with a delay of 1, and row 1 makes that fall-back change nothing.
Only its attack-move searches (S-0309 claim 23 (a)) can move it on.
S-0309's Open questions list the same gap: "what removes the attack-move
memory when the destination is reached".

The synthetic skirmish of `glasnost_ai` shows the effect. One house's
attack wave reaches its destinations just outside the weapon range of
the other house's last units and never moves again.

## Questions
1. When an actor with an attack-move memory holding a destination
   reaches that destination (claim 22's "no destination, not moving,
   nothing queued"), is the memory removed, kept, or changed?
2. Does such an actor stay in *move*, or does something put it into
   *guard* (so that S-0309 claim 23 (b) resumes the attack move), or
   into another behaviour (*area guard*, as row 12 of claim 15 would
   give without the memory)?
3. If it stays in *move* with the memory, does its processing keep
   taking a delay of 1 each tick?
4. The same questions for an actor whose memory holds only a target,
   once that target is gone.

## Possible observation
S-0309 scenario A6, part g: an attack-move order on an empty cell with
no enemy within reach, then read the actor's current behaviour, its
memory and its processing delay for a few hundred ticks after arrival
with the read-only recorder. Then place an enemy just outside its weapon
range and see whether it ever moves to engage.

## Current workaround
Nothing removes the memory on arrival. The actor stays in *move* with
the memory and no destination, as the specs read literally. The
scripted skirmish player of `glasnost_ai` treats such an actor (*move*,
no destination, no target, not moving) as idle and gives it new orders.
That is an AI policy, not simulation behaviour.
