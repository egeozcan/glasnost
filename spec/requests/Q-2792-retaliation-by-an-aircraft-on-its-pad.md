---
id: Q-2792
title: Retaliation by an aircraft that stands on its pad
status: answered
related: [S-0501, S-0215, S-0309]
answered-by: [S-0501]
---

## Need

S-0215 claim 18 and S-0309 claim 18 give the retaliation conditions for every actor, with no
exception for aircraft, and S-0215 claim 10 gives the override to *attack* that retaliation makes.
S-0501 already assumes an aircraft can be overridden (claim 32, table T2's first row) and that an
aircraft may retaliate (scenario AC31 part e). Our engine now builds aircraft retaliation through
those claims.

One case does not fit. S-0501 claim 28 says: "An aircraft of a human-controlled house gets a target
on the pad only from an order." In our engine every player house, human or AI, follows the rules
for a human-controlled owner (S-0215 Terms, *Houses in our engine*). Under S-0215 claim 18 an
aircraft of such a house in *guard* may retaliate, and an aircraft that has just reported its entry
on a pad is in *guard* (S-0501 table T3, first row) until the next service step tells it to queue
*sleep*. With the player return-fire setting on, *sleep* is no bar either when its retaliate
property is set. So the two claims disagree for a hit on an aircraft on its pad: claim 18 gives it
the attacker as its target, and claim 28 says it gets none.

For an owner that is not human-controlled (the neutral and special houses in our engine), claim 28
says nothing, but T3 does not say what an aircraft on a pad does with *attack* current through an
override: whether it takes off at once (the take-off row of T3 needs *guard* with a target and at
least half its ammunition), whether its pad contact (reservation) ends, and whether the service
steps go on for it.

The gap is in `acquire::retaliate` (`crates/deterministic/sim/src/combat/acquire.rs`).

## Questions

1. An armed aircraft of the human-controlled house stands on its helipad in *guard*, just after it
   has reported its entry, with no target, full ammunition and its retaliate flag set. An enemy
   vehicle within its weapon's reach fires at it and the aircraft survives. Does the aircraft take
   the attacker as its target (an override to *attack*, *guard* suspended)? Does the answer change
   when it is in the docked *sleep*, with the player return-fire setting on and *sleep* given the
   retaliate property?
2. If it does retaliate: does it take off at once, or only as T3's take-off row gives (half its
   ammunition)? Does its pad contact end at the override, at the take-off, or not at all? Is it
   still serviced while it stands on the pad with *attack* current?
3. The same for an aircraft of a computer-controlled house (in our engine, the neutral or special
   house), for which claim 28 does not speak.
4. Does any of this differ for an aircraft that holds a pad contact but is still flying in
   (*enter*), or for one that is on the ground on its pad's cell without a contact?

## Possible observation

A test map with one helipad of the human-controlled house and one armed, landable aircraft of a
test copy type docked on it (S-0501 scenario AC31's setup), and an enemy test vehicle of the
computer-controlled house placed within the aircraft's weapon range that is ordered to fire at it,
with a strength the aircraft cannot lose in the run. Record each tick the aircraft's current,
queued and suspended behaviours, its target, destination, height, pad contact and ammunition,
from the first hit until 300 ticks later. Repeat with the docked *sleep* (wait for the first
service step), with a test file turning player return fire on and giving *sleep* the retaliate
property. Repeat with the aircraft owned by the computer-controlled house.

## Current workaround

An aircraft on a pad is held back as before this build: `acquire::retaliate` answers *may retaliate
but does not* (`Retaliation::Held`) for an aircraft that is on the ground (S-0501 Terms) and holds a
contact with a building of one of its type's dock types (S-0501 Terms, *pad* and *pad
reservation*). So it gets no target, and with the player-scatter setting on, or the scatter
ability, and neither target nor destination, it is sent case 1 of S-0215 claim 19, as before.
Every other aircraft that may retaliate now does, through S-0215 claims 10 and 18.
