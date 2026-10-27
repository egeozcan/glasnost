---
id: Q-1940
title: Does a person's move order end the pad contact of a pad-bound aircraft released at a docking position outside its pad's foundation?
status: open
related: [S-0501, S-0213, S-0417]
answered-by: []
---

## Need

S-0501 revision 59, claim 48 (c), *Pad contact*, says that a released aircraft's contact with its
pad ends at the order when it takes a destination other than the pad. That rule reads the cell of
its horizontal position, so "a docking position outside the pad's foundation leaves the contact
held after the order". *At height 0* and Open question 26 build on this: "a pad-bound type whose
docking position lies outside the pad's foundation still holds its contact".

Two other approved specs seem to give a different result for a person's move order:
- S-0417 claim 11: when an aircraft factory releases an aircraft onto its pad, a type that must
  return to a dock refuses the tether request. So a pad-bound aircraft is never tethered by the
  release.
- S-0213 table T3, part (b), which applies to every behaviour order: "If the actor is not
  tethered, it breaks off any contact."

Taken together, a person's move order ends the contact of any pad-bound released aircraft,
wherever its docking position lies. On that reading, claim 48 (c)'s held contact outside the
foundation can arise only when the destination comes without a person's order (for example, from
a computer-controlled house's own behaviours), or for a type that need not return to a dock. The
engine has to decide which rule governs a person's order. This decides whether that aircraft falls
back to rest with no contact at height 0, or falls back holding the contact and is given *enter*.

## Questions

1. A pad-bound aircraft is released onto a pad whose docking position lies outside the pad's
   foundation. It is then given a person's move order to a cell other than the pad. Does it still
   hold its contact with the pad after the order?
2. If it does, which rule keeps it, given S-0213 table T3 (b) and S-0417 claim 11?
3. If it does not, does claim 48 (c)'s "still holds its contact" apply only to a destination that
   comes without a person's order, or only to a type that need not return to a dock? Should
   Open question 26 say so?

## Possible observation

A copy of AC30 part k (7)'s second pad whose docking displacement puts the docking position in a
cell next to the foundation, at 10 leptons above level ground, with the pad-bound type. After the
release, give a person's move order to a cell 512 leptons straight ahead along the released
facing. Record the aircraft's pad contact (for example, by whether the pad still counts the place
as taken by it) at the order and per tick. Also record what it does in the halving zone of that
cell: it comes to rest there in *guard*, or it is given *enter* and returns to the pad.

## Current workaround

S-0213 table T3 (b) and S-0417 claim 11 as written, which is our design until this is answered.
A person's move order ends the contact of an untethered pad-bound aircraft, so a low release from
outside the foundation falls back to rest at height 0 in its destination cell's halving zone, as
from a docking position inside it (*guard*, no pad search, no *enter*). Where the contact is held,
because the destination came without a person's order, our design for Open question 26 applies.
After the fall-back, *enter* is claim 24's, which takes it back to the pad. With a destination cell
on that pad, it lands and docks there.
Tests: `claim_48_c_person_order_ends_a_pad_bound_contact_outside_the_foundation`,
`open_question_26_outside_the_foundation_falls_back_then_enters` and
`open_question_26_destination_on_its_pad_lands_and_docks` in
`crates/deterministic/sim/tests/claims_s0501_released.rs`.
