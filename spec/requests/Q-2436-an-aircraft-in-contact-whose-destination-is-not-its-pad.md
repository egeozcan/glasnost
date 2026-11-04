---
id: Q-2436
title: An aircraft in enter, in contact, whose destination is not its pad
status: answered
related: [S-0501]
answered-by: [S-0501]
---

## Need

S-0501 r103 (Q-2366) removed claim 24's sentence "A pad-bound aircraft that is on the ground but
not on its destination pad leaves its current contact and takes off toward the pad" and put a
rule before every other rule of the claim: a pad-bound aircraft on the ground by its height with a
destination other than the building in its own cell leaves its contact, records that destination
as the pad it was last heading for and queues *move* with an immediate start. Three cases are
left that the new text does not reach, and claim 7 r103 changes a fourth:

- Our engine's processing of *enter* in contact during the approaching stage gave the pad back as
  destination whenever the aircraft's destination was not that pad (`air::enter`). That was built
  from the removed sentence, for every aircraft, not only pad-bound ones. Claim 24 now says only
  that the pad answers each processing's request "by confirming the contact and the approach".
  An aircraft in contact can reach that processing with no destination: claim 7 r103 makes an
  accepted touchdown forget the destination, and an aircraft in contact whose flight point (the
  docking position's horizontal location, claim 40) lies outside the pad's foundation touches down
  there, outside the foundation. The new pad-bound rule does not apply to it either: it needs a
  destination.
- The pad-bound rule records "that destination as the pad it was last heading for". The
  destination can be a cell: the guard command (S-0213 T2) gives the aircraft its own cell as
  destination and queues *area guard* without an immediate start, which leaves *enter* current.
- Claim 7 r103: "Neither an accepted nor a refused touchdown makes a fall-back to rest or an entry
  report itself." Our engine reported entry at an accepted touchdown on the pad in contact when the
  aircraft was not in *enter* (S-0501 Open question 26's held contact, which arises without a
  behaviour order). It no longer does; the aircraft then rests on the pad in contact in its
  current behaviour.

## Questions

1. An aircraft (pad-bound or not) in *enter*, in contact with its pad, in the approaching stage,
   whose destination is not that pad (none after an accepted touchdown outside the pad's
   foundation, or a cell), not landing: does the processing of *enter* give it the pad as
   destination again, so that it takes off toward the pad, or does it leave the destination as it
   is and only return its 3 ticks? If the latter, what ends that *enter* for an aircraft on the
   ground outside the foundation with no destination?
2. A pad-bound aircraft on the ground by its height, in *enter*, whose destination is a cell
   (or any object that is not a building): what does the rule before every other record as the
   pad it was last heading for: nothing (the earlier record kept), the cell's building if any,
   or something else?
3. An aircraft in contact with a pad, not in *enter* (for example in *guard*, holding the contact
   by Open question 26's case), that touches down on a cell of that pad's foundation: does it
   ever report that it has entered, and if so at which processing of which behaviour?

## Possible observation

S-0501 AC31's harness with a test copy of a helipad type whose single docking displacement puts
the docking position two cells outside the foundation, and a non-pad-bound armed aircraft with its
ammunition spent sent to it by the search after T2's row. Record per tick the aircraft's height,
position, behaviour (current and queued), destination and contact, through its touchdown and the
next three processings of *enter*.

## Current workaround

Question 1: the approaching-stage processing in contact gives the pad back as destination when the
destination is not that pad, as before r103 (`air::enter`, `design:` comment). Question 2: the
record of the pad last heading for is changed only for a building destination (`air::bound_away`).
Question 3: no entry report at the touchdown (claim 7 r103 as written); an aircraft in *enter*
reports at its next processing in the docking stage.
