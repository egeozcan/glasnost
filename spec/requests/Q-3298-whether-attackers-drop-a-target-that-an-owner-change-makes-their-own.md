---
id: Q-3298
title: Whether attackers drop a target that an owner change makes their own or allied
status: open
related: [S-0309, S-0301, S-0601, S-0509, S-0612, S-0215]
answered-by: []
---

## Need

Issue 453. The lead reports that in the original, units attacking an enemy unit stop when a
mind controller of their own side takes that unit over. In our engine they keep attacking it,
because that is what the claims give:

- S-0309 claim 30 drops a target only when the *holder* changes owner. When the *target*
  changes owner, as at a mind-control takeover (S-0601 claim 6), no claim drops it.
- S-0309 claim 29's periodic check drops a held target only when the firing check answers
  *illegal* or *impossible*. No reason of S-0301 claim 9 refuses a target because of its
  relation to the holder's owner, so a target of the holder's own house or of an ally is kept.
- S-0309 claim 25 part k drops an allied held target only for a holder of a
  computer-controlled owner, which in our engine is the neutral and special houses. The claim
  says: "A holder of a human-controlled owner has no such drop: in our engine, the holders of
  every player house keep an allied target here." In our engine AI houses are player houses
  too.
- S-0309 claim 31 (its *Override in effect* and *Attack-move memory* parts) says that a target
  which an owner change has made one of the holder's own house or of an ally "is held like any
  other target", and that *attack* approaches it and fires on it.
- S-0601 claim 6 lists the consumers of the change of owner at a takeover. None of them drops
  the taken object from the objects that hold it as their target.

The engine must decide whether the holders of player houses drop such a target, and if so when.

## Questions

1. A unit of a human-controlled house holds an enemy unit *T* as its target, given by an attack
   order. A mind controller of the same house takes *T* over. Does the unit drop *T*? If so, is
   it dropped at the takeover itself, at the unit's next turn, or at S-0309 claim 29's next
   check tick?
2. Is the answer the same when the target was found by the unit's own search, by retaliation or
   by a revert (S-0215 claim 10), rather than given by an order?
3. Is the answer the same when *T* comes to belong to an ally of the holder's house rather than to
   its own house? And for other changes of the target's owner: an engineer capture of a
   building (S-0509 claim 8), a psychic domination (S-0612 claim 14), a mind-control release
   that returns *T* to the holder's house, or a trigger's change-owner action?
4. Does force-fire change the answer? That is, is an attack order given with force-fire on a
   unit that later becomes own-house or allied still kept and carried out?
5. If the target is dropped, does the drop take a targeting-timer draw (as S-0309 claim 27's
   drops do) or none (as claim 29's and claim 30's do)? Does the holder's behaviour change, for
   example from *attack* to its rest behaviour, at the drop or only at its next processing?
6. Does a computer-controlled holder (claim 25 part k) behave the same as before, or does the
   answer change that rule too?

## Possible observation

Two player houses, Alpha and Beta, at war. A Beta tank is ordered to attack an Alpha vehicle *T*
about 3 cells away. A Beta mind controller then takes *T* over. Record, each tick from the
takeover until 40 ticks after it, the tank's target and current behaviour, and *T*'s health.
Repeat with the tank's target found by its own search (no order), with force-fire on the order,
with *T* taken by a mind controller of a house allied with Beta, and with an Alpha building taken
by a Beta engineer while a Beta tank is attacking it.

## Current workaround

Today's behaviour, as the claims give it above: the holder of a player house keeps the target,
goes on approaching it and firing on it, and S-0309 claim 29's check does not drop it. Guard test:
`claims_s0601::s0309_issue_453_a_player_attacker_keeps_a_target_its_house_took_over`. Claim 25
part k's drop for the neutral and special houses is unchanged (guarded by `claims_s0309_r94`).
