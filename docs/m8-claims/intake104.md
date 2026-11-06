# S-0501 r132 to r134 and S-0302 r41 to r43: claim ledger, M8 step A3 intake 104

- **S-0501** r132 to r134 (commit 45ad367, from the r132 that intake 102 built). r134 answers
  Q-2646 (what follows the landing check's refusal), Q-2647 (tracking, listing and movement after
  an untested touchdown that kept the flight point), Q-2648 (claim 6's exact-cell trigger while the
  aircraft slides), Q-2649 (the entry report and the service after a touchdown beside the docking
  position) and Q-2650 (an entry restarted by the pad's answer in the approaching stage), which
  intake 102 filed with interim rules. It moves claim 7's landing check to the update's landing
  step, after the forward step, claim 3's descent and claim 6's triggers; it gives the refusal a
  take-off, a nearby search for a free cell and, where none is found, a demolition hit; every
  touchdown sets both speed fractions to 0; it states claim 47's listing, object-present flag and
  tracking for a sliding aircraft, claim 6 for an aircraft on the ground that is moving, claim 24's
  restart in either stage and claim 26's release before the skip. It adds T5's row for the
  refusal's search and the `C4Warhead` row of the Import mapping. No quirk is added; the scope of
  three switches moves (C7 entries in [c7-play-fixes.md](c7-play-fixes.md)).
- **S-0302** r41 to r43 (commit 53963ee). r43 answers Q-2626 as intake 100's interim had it: a
  berserk hit plays no damage sound and makes no announcement other than the harvester one (claim
  10). No behaviour changes; the cites name r43.

Requests filed (commit abd582a): Q-2676 (an aircraft that survives the hit of a refused landing
check, Open question 28) and Q-2677 (Open question 30: *guard* with a target during the slide, and
where the aircraft rests). Q-2678 to Q-2685 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The landing step** (`air::landing_step`). `air::tick_flight`'s order is now: sector update,
  crash, start, the given-no-destination clear, destination and holding mark, the released return,
  the drop, held height, take-off requests, forward step, altitude, write-back, speed fraction and
  claim 6's triggers, then the landing step, then the released climb and guidance. The landing step
  re-files a landing that started in the update (S-0102 claim 4), makes claim 7's check of the
  flight point's cell and, at height 0, the touchdown. A landing that the update's exact-cell
  trigger starts at height 0 touches down in that update (Q-2648), and the update whose descent
  reaches height 0 can still be refused (Q-2646).
- **The refusal** (`air::check_refused`, Q-2646). Take-off toward cruise height, then S-0212 claim
  25's nearby search around the cell the aircraft is over, with T5's row (class track, zone fly).
  A found cell's centre is the new flight point and the destination is kept. Above height 0 the
  take-off ends in that update (`air::end_refusal_take_off`), so the aircraft climbs from the next.
  At height 0 the touchdown follows in that update, untested whatever the two claim 7 switches
  say: its forget test against the new point rests it (`air::touchdown` without the destination's
  forget), or it keeps the point and stays on the ground by its height with ordinary flight's held
  height and no slide (`air::refusal_touchdown_kept`). No cell found: a hit of the aircraft's
  current health with the demolition warhead (`damage::apply`, defences ignored, no passenger
  escape, no attacker, no house, distance 0); its flight point is cleared and no touchdown follows.
  A hit that ignores defences is S-0302 F3's d₀, so it always destroys the aircraft in our engine;
  what a survivor does next is Q-2676's interim (design cite in `air::end_refusal_take_off`).
- **Speed fractions at touchdown** (claim 7). Every touchdown sets the current and the target
  fraction to 0: the accepted, the untested ones (resting or kept) and the refused touchdown's
  rise.
- **The touchdown's forget** (`air::forget_point`). The flight point is cleared before the
  destination, so that claim 6's given-no-destination rule, which the cleared destination calls,
  finds the aircraft holding none and changes nothing (r120's forget while the landing is under
  way).
- **The slide's listing and flag** (`air::kept_mark`, claim 47 (b), Q-2647). The touchdown of an
  aircraft that keeps its flight point ends its tracking and lists it in its cell layer with the
  cell's object-present flag. While it slides at height 0, each movement update moves the listing
  (at the front) and the flag to the cell it ends the update in; the first update whose climb
  leaves its height positive ends both. `air::touchdown` clears the old flag before it sets the
  new. The r132 interim that rested a sliding aircraft whose flight point had gone is removed.
- **Tracking again** (`air::sector_update`, claim 47 (c), Q-2647). A sliding aircraft is tracked
  again by the rule for any untracked aircraft not at rest, at its first movement update with a
  positive carried fraction, so `quirks.aircraft_corner_sector_untracked` covers it too. The
  touchdown's fraction of 0 means no sector update in its next update.
- **Given no destination on the ground** (`air::given_no_destination`, claim 6, Q-2647). An
  aircraft on the ground by its height that is moving, as one sliding after an untested touchdown
  that kept its flight point, gets the new flight point like one in flight and takes off toward it
  (claim 8, re-filed by S-0102 claim 4). Only an aircraft at rest is left alone.
- **The restarted entry** (`air::restarted`, claim 24, Q-2650). The pad's answer that restarts
  entry makes entry's beginning within that processing in the approaching stage as in the docking
  stage: 1 tick, the approaching stage, the ready mark; its next processing is the approaching
  stage's. Q-2649: the entry report on the ground requests no facing, as r132's interim did.
- **The release before the skip** (`air::pad_turn`, claim 26). The release of a ready aircraft at
  full health and full ammunition comes before the skip of one flying in or moving; a skipped one
  does not count as settled.
- **S-0302 r43**: cite changes in `damage::berserk_reactions`, `effects::Effect::Hit`,
  `effects::Sound::HarvesterAttacked` and `glasnost-audio`'s event mapping.
- **Import mapping**: `glasnost_convert`'s `C4Warhead` row coverage adds S-0501.

## Claims

All S-0501 tests are in `crates/deterministic/sim/tests/claims_s0501_r134.rs` unless named
otherwise.

| Claim (r134) | Status | Tests and notes |
|---|---|---|
| 7: the check at the landing step; the touchdown in the refusing update (Q-2646, Q-2648) | tested | `claim_7_the_touchdown_in_a_refusing_update_is_untested_and_climbs_with_no_slide` (the four positions of the two claim 7 switches), `claim_7_a_refusing_update_over_a_free_cell_rests_there` |
| 7: the refusal in flight, flying on to the free cell | tested | `claim_7_a_refusal_in_flight_flies_on_to_the_free_cell` |
| 7: no free cell, the demolition hit | tested | `claim_7_with_no_free_cell_it_takes_the_demolition_hit` (destroyed at verses 1 and 0.5: defences ignored) |
| 7: what a survivor of that hit does (Open question 28) | seam (interim Q-2676) | unreachable with a hit that ignores defences in our engine |
| 7: every touchdown sets both fractions to 0 | tested | `claim_47_a_sliding_aircraft_is_listed_and_flagged_until_it_climbs`, `claim_7_the_touchdown_in_a_refusing_update_is_untested_and_climbs_with_no_slide` |
| 47 (b): the slide's listing and flag (Q-2647) | tested | `claim_47_a_sliding_aircraft_is_listed_and_flagged_until_it_climbs` |
| 47 (c): tracking again, the corner switch (Q-2647) | tested | `claim_47_the_corner_switch_applies_to_a_sliding_aircraft` (both positions; the corner case on a 10 × 10 map, whose every cell lies in sector (0, 0)), `claims_s0501_tracking::claim_47_corner_sector_quirk_both_ways` |
| 6: on the ground and moving, given no destination (Q-2647) | tested | `claim_6_a_sliding_aircraft_given_no_destination_takes_off` |
| 24: the restart in the approaching stage (Q-2650) | tested | `claim_24_an_entry_restarted_in_the_approaching_stage_makes_its_beginning_at_once`; `claims_s0501_r110::claim_24_*` now expects the beginning within that processing |
| 24: the restart in the docking stage | tested | `claims_s0501_r132::claim_24_an_entry_restarted_in_the_docking_stage_makes_its_beginning_at_once`: the same tick's movement update, with the pad given back, lands again in the occupied docking cell and is refused again (Q-2646's order), so the tick ends with no destination |
| 24: the report on the ground (Q-2649) | tested | `claims_s0501_r132::ac31h4_an_occupant_at_the_docking_position`, `ac31h1_after_an_untested_touchdown_beside_the_docking_position_it_comes_to_rest` |
| 26: the release before the skip | tested | `claim_26_the_release_comes_before_the_skip` (full and not full) |
| *guard* with a target during the slide (Open question 30) | seam (interim Q-2677) | as the code does, no cite |
| S-0302 r43 claim 10: no damage sound, the harvester announcement only (Q-2626) | tested | `claims_s0302_r43::claim_10_a_berserk_hit_reports_no_hit_event`, `claim_10_an_ordinary_hit_reports_its_hit_event`; audio: `a_berserk_hit_plays_no_damage_sound`, `a_berserk_hit_on_another_unit_is_not_announced`, `a_berserk_hit_on_a_local_harvester_is_announced` |

Earlier tests whose expectations moved: `claims_s0501_r110` (claim 24's approaching-stage
restart) and `claims_s0501_r132` (the docking-stage restart above; the landing check's refusal,
which now ends its take-off in the update and flies to a free cell; the untested touchdown that
kept its flight point, which now holds its cell's flag with both fractions at 0).
`claims_s0501_r120`'s expectations are unchanged: its touchdown test failed on an early draft
whose forget took the aircraft off, which the order of `air::forget_point` now prevents.

## Goldens

LANES-GOLDENS

## Lanes

LANES-RESULTS
