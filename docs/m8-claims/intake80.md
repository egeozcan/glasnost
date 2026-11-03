# S-0501 r99: claim ledger, M8 step A3 intake 80

- **S-0501** r93 to r99 (commit f3a13c9, from 87ef8af, which intake 76 built). It answers
  Q-2326 (what a processing of *enter* does for an aircraft with a pad destination and no
  contact), which intake 76 filed with an interim rule (clear the destination and fall back).
  It changes claims 18, 24 and 49's notes, Open question 19, AC31 (new part f) and a note for
  S-0404 claim 15. No quirk is added or changed: `quirks.aircraft_edge_hold` and
  `quirks.unarmed_pad_bound_hover` read as at r93, so `c7-play-fixes.md` gets no new entry.

Requests filed: Q-2366 (a waiting aircraft that lands over its destination pad without a
contact: F7 refuses the pad's cell, and the pad-bound rule that "comes first"). Q-2367 to
Q-2375 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **Claim 24's processing without a contact** (`air::enter`, `air::without_contact`). Every
  processing of *enter* after its first that finds the aircraft without a contact returns 1 tick.
  With no behaviour queued it forgets the destination and falls back to rest (claim 18), as
  before. With a behaviour queued it keeps the pad as destination and *enter*, asks no pad and
  waits; the queued behaviour becomes current at the next of claim 31's start opportunities.
  A destination that is no longer a pad of the aircraft's dock types (gone, being sold, or
  cleared at a touchdown, claim 40) ends the wait: the aircraft falls back whatever is queued.
- **A refusal and a sale** (`air::enter`). A pad that refuses the loading request of an aircraft
  in contact ends the contact. A pad being sold is no longer the aircraft's destination: it is
  cleared and the aircraft falls back, with or without a behaviour queued (claims 24, 32 and
  46). Any other refusal leaves the pad as destination and the processing without a contact
  follows in the same processing (Open question 19 keeps the refusal itself open; our engine
  meets it at the request, for instance at a pad switched off).
- **The docking move without a contact** (`air::docking_step`). A waiting aircraft landing over
  a cell of its destination pad is moved, at each processing, toward the docking position of a
  processing without a contact: with one docking place that place's position, otherwise the
  foundation centre with no displacement (`pad_position_at` with an index outside the count,
  S-0404 claim 15). An aircraft in contact still moves toward its contact place's position.

No state is added, so the save format (93) and the AI game format (56) are unchanged.

## Claims

### S-0501 r99

| claim | status |
|---|---|
| 18, queueing *enter* while it is current with nothing queued changes nothing and starts nothing; *enter* goes on from its stage | Tested: `claims_s0501_r99::claim_24_ac31f_the_second_processing_falls_back_and_entry_is_not_restarted` (the start tick is unchanged and the next processing is 3 ticks after the approach's first). Built already (`Behaviours::queue`, S-0215 claim 7) |
| 18, the search after the ground row asks the pad for no contact (AC31 part f's first expectation) | No change (intake 76); checked again at the start of each r99 test (`without_contact`) |
| 24, the first processing of *enter* only starts entry and makes no contact | Tested: `claim_24_ac31f_…` |
| 24, a later processing without a contact, nothing queued: the destination forgotten, the fall-back, 1 tick; the fall-back's pad search gives the pad back and the contact; the next processing, 1 tick later, in the approaching stage; no processing at *t*₁ + 3 or *t*₁ + 4; it then docks | Tested: `claim_24_ac31f_…` (AC31 part f, in flight at the second processing). The case on the ground at that processing is not reached here: the search after the ground row starts the take-off at once (claim 8), as the spec says of its own set-up |
| 24, the pad's every contact place taken at that processing: not found again; claim 18's outcome | Tested: `claim_24_with_the_pad_full_the_fall_back_takes_claim_18s_outcome` (the take-off has begun, so T2's ground row: the destination forgotten, *guard*; not pad-bound) |
| 24, with a behaviour queued: the destination and *enter* kept, no pad asked, 1 tick, with or without a free place | Tested: `claim_24_with_a_behaviour_queued_it_keeps_the_pad_and_waits` (the behaviour queued before the processing, S-0215 claim 12; both with the place free and taken) |
| 24, the wait lasts until the queued behaviour becomes current | Partly: in our engine a ready aircraft starts its queued behaviour at the end of the same update (claim 31's third point), so the wait lasts one processing; an aircraft in *enter* is not ready only with claim 31's processing-cleared ready mark, which our engine does not model |
| 24, a refusal that ends the contact while the pad stays the destination leaves it to the processing without a contact | Tested: `claim_24_a_refusal_ends_the_contact_and_the_processing_without_one_follows` (a pad switched off, with and without a behaviour queued). Partly as to the moment: the processing without a contact follows in the processing that meets the refusal (Open question 19 keeps the refusal open) |
| 24, the pad stops being the destination while it approaches (a sale): the fall-back, whatever is queued | Tested: `claim_24_a_sale_during_the_approach_falls_back_with_a_behaviour_queued`; the pad-bound crash after a sale is still `claims_s0501::pad_bound_pad_sold_during_the_approach_crashes` |
| 24, a waiting aircraft landing over its pad is moved toward the docking position without a contact: one docking place, its position; several, the foundation centre | Tested: `claim_24_a_waiting_aircraft_is_moved_toward_its_docking_position_without_a_contact` (one and two places; a contact ended by a switched-off pad during the docking stage while the aircraft stands over place 0's position: with one place it is not moved, with two it steps toward the centre) |
| 24, at a touchdown without a contact it falls back even with a behaviour queued; so does an aircraft whose contact ends during the docking stage | Seam, Q-2366: F7 refuses a cell of the pad to an aircraft that holds no contact (the found object is neither the lander nor its contact), so on the pad the touchdown never comes and the aircraft takes off again (claim 7). A touchdown elsewhere clears the destination (claim 40), and the next processing falls back whatever is queued |
| 24, the pad-bound rule on the ground away from the destination pad "comes first" | Seam, Q-2366: what it comes before is not clear. Our engine applies the processing without a contact; the kept destination gives the take-off (claim 8) |
| 24, an aircraft never reports that it has entered without a contact | No change: `air::dock` and the touchdown's docking need a contact |
| 24, the delays (1 tick for every processing that finds no contact, whether it waits or falls back) | Tested in the tests above (`beh.last`) |
| 49, in the two end states without a contact, *enter*'s later processings are claim 24's | No change: they reach the same `air::enter` |
| Open question 19, what *enter* does after a refusal with a behaviour queued | Tested with claim 24's refusal above |
| AC31 part f | Tested: `claim_24_ac31f_…` |
| Changes for other specs, S-0404 claim 15 (the docking position without a contact) | Built through `pad_position_at`'s rule for an index outside the count |
