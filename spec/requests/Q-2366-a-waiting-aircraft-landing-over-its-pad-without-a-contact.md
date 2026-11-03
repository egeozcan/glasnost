---
id: Q-2366
title: A waiting aircraft that lands over its destination pad without a contact
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r99 claim 24 (Q-2326) says that a processing of *enter* after its first that finds the
aircraft without a contact, with a behaviour queued, keeps the pad as destination and waits. It
then says that a waiting aircraft that lands over its destination pad (claims 6 and 7) is moved
while landing toward a docking position, and that "at touchdown it falls back to rest (claim 18)
even with a behaviour queued: a touchdown without a contact ends the wait". It adds: "The rule
below for a pad-bound aircraft on the ground away from its destination pad comes first."

Two points do not fit together in our engine (`air::enter`, `air::touchdown`, F7 in
`air::landing_cell_ok`):

- F7 accepts a landing cell whose found object is "the lander itself or its contact". A cell of
  the destination pad's foundation holds the pad building in its ground layer, and a waiting
  aircraft holds no contact, so F7 as written refuses that cell, and claim 7 then makes the
  aircraft rise by 10 leptons and take off again. A touchdown without a contact on the pad
  would then never come; it comes only where the docking move has carried the aircraft into a
  cell outside the foundation that F7 accepts.
- It is not clear what the pad-bound rule "comes first" before: the touchdown fall-back only,
  or the whole processing without a contact. Claim 6 already keeps a pad-bound aircraft with
  no contact from starting to land, so the case seems to be one on the ground for another
  reason.

## Questions

1. A non-pad-bound aircraft in *enter*, its pad as destination, no contact, a behaviour
   queued that has not yet become current, starts to land over a cell of the pad's foundation.
   When it reaches the ground render layer, is the cell accepted (a touchdown, then the
   fall-back to rest), or refused (it rises by 10 leptons and takes off again)? If refused,
   does it start to land over the pad again, and what then ends the wait?
2. For the same aircraft, is it moved toward its docking position already at the first
   processing of *enter* that finds it landing, or, as for an aircraft in contact, only from the
   processing after the one that finds it landing?
3. A pad-bound aircraft on the ground, not on its destination pad, in *enter* with that pad as
   destination and no contact: at a processing of *enter* after its first, does it take off
   toward the pad and then wait (with a behaviour queued) or fall back (with none), or does the
   take-off replace the processing without a contact altogether? What delay does that
   processing return?
4. At a touchdown without a contact, is the fall-back to rest made in the movement update of
   the touchdown, or at the next processing of *enter*?

## Possible observation

S-0501 AC31's harness and part f's set-up, with a test copy of a helipad type whose single
docking displacement is zero, and a test action that queues a behaviour for *A* without an
immediate start in each tick before its processing while it approaches *P* without a contact.
Record per tick *A*'s height, position, render layer, behaviour (current and queued),
destination and contact.

## Current workaround

F7 is applied as written, so on a cell of the pad the touchdown is refused and the aircraft
takes off again. A touchdown elsewhere clears the destination (claim 40), and the next
processing of *enter*, finding no contact and no pad destination, falls back to rest whatever
is queued. The waiting aircraft is moved toward the docking position of a processing without a
contact at every processing that finds it landing over a cell of its destination pad. A
pad-bound aircraft on the ground away from its pad takes the processing without a contact like
any other aircraft; its take-off toward the pad comes from its kept destination (claim 8). The
intake 80 ledger (docs/m8-claims/intake80.md) lists these as seams. In our engine a queued
behaviour of a ready aircraft becomes current at the end of the same update (claim 31's third
point), so the wait lasts one processing unless the aircraft is not ready.
