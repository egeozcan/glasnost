# S-0501 r103: claim ledger, M8 step A3 intake 86

- **S-0501** r99 to r103 (commit 57755b7, from the r99 that intake 80 built). It answers
  Q-2366 (a waiting aircraft that lands over its destination pad without a contact, and the
  pad-bound rule that "comes first"), which intake 80 filed with an interim rule. It changes
  claims 7, 24 and 47, the F7 note, the Backing and Confidence of claims 7 and 24, and AC31
  (new part g). No quirk is added or changed, so `c7-play-fixes.md` gets no new entry.

Requests filed (commit 538a793): Q-2436 (an aircraft in *enter*, in contact, whose destination
is not its pad; the pad the pad-bound rule records for a cell destination; an entry report after
a touchdown outside *enter*), Q-2437 (a lost pad and the wait, the move on to the docking stage
without a contact, the docking stage's fall-back, a sale in the docking stage), Q-2438 (the ready
mark when *enter* is left by force after the move on to the docking stage). Q-2439 to Q-2445
unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The pad-bound rule before every other** (`air::bound_away`, first in `air::enter`). At every
  processing of *enter*, its first included, a pad-bound aircraft on the ground by its height
  whose destination is not the building in its own cell leaves its contact, records a building
  destination as the pad it was last heading for (`heading_pad`, which claim 21's search asks
  first), queues *move* with an immediate start (S-0215 claim 7; claim 34's ignored requests
  aside) and returns 1 tick. *Move*'s first processing then runs claim 40's selection: F7 refuses
  the pad's cell, so a replacement cell is sought.
- **The heading point of a pad without a contact** (`air::heading_point_for`, used by the flight
  point and by claim 40's reservation test). For an aircraft without a contact, a building of one
  of its dock types has the heading point `pad_position_at(w, k, NO_PLACE)`: with one docking place
  its docking position, height included; with several or none the foundation centre; a pad that is
  neither a helipad nor a unit repair building gives its centre (S-0404 claim 15). Any other
  building keeps S-0506 claim 14's centre (its docking-point cases stay a seam in `rocket.rs`). A point more than 120 leptons above the
  terrain makes claim 6's holding mark, so a *move* kept on the pad stays above it.
- **The wait** (`air::without_contact`). With a behaviour queued the aircraft waits whatever its
  destination (a cell given by the guard command too); with nothing queued it forgets the
  destination and falls back. A lost pad (gone, being sold, not of its dock types) still ends the
  wait with a fall-back (Q-2437, interim). The wait no longer moves the aircraft: a processing
  without a contact that finds it landing over its destination pad moves it on to the docking
  stage and does not move it.
- **The docking stage without a contact** (`air::docking_without_contact`). Reached when the
  contact ends after the aircraft reached the docking stage (at a refusal of that stage's request,
  or before a processing). Each processing returns 1 tick and tests neither contact nor a queued
  behaviour: while landing, a docking step toward the docking position without a contact; once no
  longer landing, a fall-back to rest, whatever is queued.
- **The touchdown** (`air::touchdown`). An accepted touchdown forgets the destination and reports
  nothing: the docking at the touchdown of an aircraft in contact outside *enter* is gone (claim
  7: no entry report at a touchdown; Q-2436 asks what such an aircraft does).
- **The ready mark** (`air::ready`, `air::ready_mark_cleared`). The processing that moves an
  aircraft in contact on to the docking stage leaves it not ready to change until its next
  processing. The mark is derived from that processing's record (*enter* current and started
  where `enter_of` says, `enter_docking` set, a last delay of 3, which only that processing
  returns with the docking stage set), so no state is added.

No state is added, so the save format (94) and the AI game format (57) are unchanged.

## Claims

### S-0501 r103

| claim | status |
|---|---|
| 7, an accepted touchdown forgets the destination in the same movement update, the contact kept | Tested: `claims_s0501_r103::ac31g4_the_touchdown_on_the_pad_forgets_the_destination_and_keeps_the_contact` (AC31 part g (4)). Built already (`touchdown` cleared it) |
| 7, neither an accepted nor a refused touchdown makes a fall-back or an entry report itself; in *enter* the next processing makes the docking stage's step | Tested: `ac31g4_…` (no report at the touchdown, sleep with guard queued at the next processing), `claim_24_after_a_refused_touchdown_…` (no fall-back at the refusal). Built: the docking at a touchdown outside *enter* is removed |
| 24, the pad-bound rule before every other: on the ground by its height, a destination other than the building in its own cell; leaves the contact, records the pad, queues *move* with an immediate start replacing the queued behaviour, 1 tick, nothing else | Tested: `claim_24_a_pad_bound_aircraft_on_the_ground_away_from_its_pad_queues_move` (at the first processing; guard queued is replaced), `claim_24_the_pad_bound_rule_needs_the_ground_and_a_destination_elsewhere` (no destination, in flight: the entry delay). The set-up is forced (the spec says no ordinary input reaches it) |
| 24, then *move*'s claim 40 selection: F7 refuses the pad's cell; a replacement cell; the take-off; the fall-back over it; the fall-back's search asks the recorded pad first | Tested: `claim_24_a_pad_bound_aircraft_…` (a cell outside the foundation, no landing off the pad, *enter* again in contact with the pad, then docked) |
| 24, every radius fails: the pad kept; the heading point without a contact (one place: its docking position with height; several or none: the foundation centre); above 120 leptons the holding mark, no stopped report, no fall-back | Tested: `claim_24_the_heading_point_of_a_pad_without_a_contact` (one and two places; in contact, the docking position's horizontal location), `claim_24_a_pad_kept_as_move_destination_without_a_contact_holds_above_a_high_heading_point` (a height of 200 holds above the point with *move* current; 0 falls back and asks the recorded pad). The failed search is emulated by putting the pad back as destination after *move*'s first processing |
| 24, with a behaviour queued the wait lasts one processing: the queued behaviour current at the end of that tick; the same with a cell destination from the order | Tested: `claim_24_ac31g1_the_guard_commands_area_guard_is_current_after_one_processing` (AC31 part g (1), through the guard command's order), `claims_s0501_r99::claim_24_with_a_behaviour_queued_it_keeps_the_pad_and_waits`. Replaces intake 80's "partly" row |
| 24, a processing without a contact that finds the aircraft landing moves it on to the docking stage without moving it, then waits or falls back | Tested: `claim_24_a_processing_without_a_contact_that_finds_it_landing_moves_it_on_to_the_docking_stage`. Partly: the move on needs a cell of the destination pad's foundation (Q-2437, interim) |
| 24, the docking stage without a contact: while landing a docking step toward the docking position without a contact | Tested: `claims_s0501_r99::claim_24_in_the_docking_stage_without_a_contact_it_is_moved_toward_its_docking_position` (renamed from r99's waiting-aircraft test, which r103 replaces; one and two places), `claim_24_after_a_refused_touchdown_…` |
| 24, the docking stage without a contact: the first processing no longer landing falls back, whatever is queued, after an accepted or a refused touchdown | Tested: `claim_24_after_an_accepted_touchdown_the_docking_stage_without_a_contact_falls_back` (area guard queued), `claim_24_after_a_refused_touchdown_the_docking_stage_without_a_contact_falls_back`. Seam (Q-2437): the destination is not forgotten first |
| 24, a refusal goes to the processing without a contact, or in the docking stage to that stage's rules | Tested: `claims_s0501_r99::claim_24_a_refusal_ends_the_contact_and_the_processing_without_one_follows`, `claim_24_after_a_refused_touchdown_…` |
| 24, the move on to the docking stage leaves the aircraft not ready until its next processing; each docking-stage processing leaves it ready | Tested: `claim_31_the_move_on_to_the_docking_stage_clears_the_ready_mark_until_the_next_processing`, `claim_31_a_docking_stage_processing_leaves_the_aircraft_ready`. Seam (Q-2438): a behaviour forced in between counts the aircraft ready (the mark is derived, not kept) |
| 24, the removed sentence (a pad-bound aircraft on the ground away from its pad takes off toward it) | Built as the rule above. Seam (Q-2436): the approaching stage in contact still gives the pad back as destination when the destination is not the pad (interim) |
| 31, not ready when its processing has cleared its ready mark | Tested with claim 24's ready marks above; the *move* of a type that carries vehicles is not built (not in this delta) |
| 47, a refused touchdown makes no overlap: not listed in its cell layer, still tracked | Tested: `claim_24_after_a_refused_touchdown_…` (not in the ground list of its cell, `flying::tracked`). No change: the refusal lists nothing |
| F7, a lander in the air is never found as itself; every foundation cell refuses a lander without a contact, accepts its contact | Tested: `ac31g2_a_move_onto_a_pads_cell_is_replaced_and_the_control_keeps_its_cell` (`air::landing_test` on every cell of the pad, without and with the contact). No change: a lander in flight is not in the ground list |
| AC31 part g (1) to (4) | Tested: `claim_24_ac31g1_…`, `ac31g2_…` (2) and its control (3), `ac31g4_…` |

## Goldens

The duel and naval duel goldens move (the duel from about tick 12030 and ends sooner; the naval
duel from about tick 12000 and runs longer); the other seven are unchanged. Attributed by
reverting each change in turn: with every change reverted the old goldens match; with only the
rework of the processing without a contact on (the wait with a behaviour queued whatever the
destination, the move on to the docking stage while landing, the docking stage's rules without a
contact), the regenerated goldens are byte for byte those of the full change. The pad-bound rule,
the heading point without a contact, the touchdown's lost docking and the ready mark move no
golden on their own. The straight duel still exercises everything it must (transport, launch).

## Lanes

Before: main 53e846c with this intake's requests; after: that base with this intake's code commit
("sim: aircraft enter without a contact, …"); same machine, release. The lanes ran before the
rebase onto 7f29fee (intake 85, S-0214, which changes sim and AI code but no golden); on the
rebased tree the checks and the gate were run again (the goldens hold), the lanes were not.
Intake 87 moves every golden, the arena and the quick lane, so after this intake is rebased onto
it the two goldens are regenerated there and the arena and the quick lane are run again.

| lane | before | after |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest counter 0 | identical output |
| M7a sweep, seeds 1 to 16 | smallest counter 0 | identical output |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | identical output |
| arena (`--record all`) | 11 of 155 pairings fail | identical output and recorded files; the same 11 (LTNK–E1, TELE–APOC, TELE–V3, CAOS–APOC, CAOS–TTNK, CAOS–E1, CAOS–E2, DISK–FV, INIT–E2, ROBO–HTK, MIND–E1), none changed |
| `glasnost-ai --include-ignored` | 246 passed | 246 passed |
| AI quick lane, seeds 7 to 10 × sides 0,1 / 0,2 / 1,2 × maps 0, 2, 8 | 36 of 36 games over with a winner | 36 of 36 over with a winner |

In the quick lane every game of sides 1 and 2 is unchanged; the games with side 0 (whose aircraft
use pads) change course. Side 0 won 15 of its 24 games before and 10 after (against side 1: 10 of
12, then 6; against side 2: 5 of 12, then 4); game lengths range 25734 to 91418 ticks before and
14557 to 91418 after. At the games' ends 39 army objects were in *enter* before, 33 after, and
side 0 made 359 aircraft before, 345 after: no sign of aircraft held in a wait.
