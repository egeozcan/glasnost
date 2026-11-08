# M8 A3 intake 118: S-0501 r167 and r172

Brief: `.claude/worktrees/M8-A3-INTAKE118-BRIEF.md`, scope moved by the coordinator to S-0501 r172
(401710c, after r167 59cb081). Answered requests: **Q-2802** (S-0501 r167, claim 29) and **Q-2821**
(S-0501 r172, claim 18's *Missile types*). **Q-2822** (S-0209, the same destination given again)
stays open; it is not in this revision. New requests: **Q-2866** and **Q-2867** (r167 claim 29, both
with interims built) and **Q-2868** (r172 claim 20, the kamikaze's removal after its shot, blocked).
r167 and r172 add no quirk name; the four aircraft switches the brief named are checked again, with
C7 verdicts in [c7-play-fixes.md](c7-play-fixes.md); none is admitted to the play-fixes layer.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0501 r167 claim 29, *A pad found below cruise height*: a *guard* or *sticky* processing below cruise height with less than full ammunition and no contact runs claim 21's search, keeps its destination as it was, queues *enter* and asks the pad found to be its destination, so claim 24's contact rule asks for contact | built, tested | `air::guard` (below-cruise path), `air::load_request`; `claims_s0501_r150.rs` `ac33_c_with_a_pad_it_gets_the_pad_and_its_contact`, `claim_29_the_pad_search_is_made_on_the_ground_too` (cites moved from the Q-2802 interim to r167) |
| S-0501 r167 claim 29: a settled aircraft (stopped, with no destination, or pad-bound over its own cell, read with the destination from before the request) whose pad accepts it starts *enter* at once; *sticky* ends | built, tested | `air::load_request` helpers `settled` and `start_enter_at_once`; `claims_s0501_r167.rs` `claim_29_a_landed_aircraft_that_finds_a_pad_starts_enter_at_once` (*sticky* and *guard*) |
| S-0501 r167 claim 29: one in flight with a destination keeps *sticky* with *enter* queued and the pad's contact | built, tested | `claim_29_one_in_flight_with_a_destination_keeps_sticky_with_enter_queued` |
| S-0501 r167 claim 29, the same for *guard*: a guard aircraft now takes the contact at that processing (before r167 our engine sent it to the pad without the contact) | built, tested | `air::guard`; `claim_29_a_landed_aircraft_that_finds_a_pad_starts_enter_at_once` (the *guard* half); changes the quick lane, see Lanes |
| S-0501 r167 claim 29, *A contact held at cruise height*: T2's guard row needs *enter* current; the in-flight pad row runs claim 21's search, gives no destination for a moment, asks the pad found for contact by the row's own request and gives it as destination; *enter* stays queued | built, tested | `air::fall_back_rows`; `claims_s0501_r150.rs` `claim_29_a_processing_at_cruise_height_chooses_1_tick`, `claims_s0501_r167.rs` `ac33_c_…` |
| S-0501 r167 claim 29: the moment with no destination outside claim 29's case, and when the search finds no pad | blocked (Q-2867), interim built and tested | `// design: Q-2867` in `air::fall_back_rows` (the moment only in claim 29's case and only when a pad is found); the same tests |
| S-0501 r167 claim 29 and claim 21: the pad last heading for, recorded for a contact granted in *sticky* | blocked (Q-2866), interim built and tested | `// design: Q-2866` in `air::request_contact` (every granted contact records the pad); `ac33_c_it_lands_at_the_pad_and_rests_in_sticky_with_enter_queued` |
| S-0501 r167 claim 29, *On the pad*, and AC33 c: it lands at the pad and rests there in *sticky* with *enter* queued; the pad starts no servicing for it | built, tested | `claims_s0501_r167.rs` `ac33_c_it_lands_at_the_pad_and_rests_in_sticky_with_enter_queued` (`quirks.aircraft_descent_touchdown_untested` both ways) |
| S-0501 r167 AC33 d: with a slowdown distance of 0 it never lands | built (already), tested | `ac33_d_with_a_slowdown_distance_of_0_it_never_lands` |
| S-0501 r167 AC33 e: a slow aircraft hovers short of the zone or lands in its cell | built (already), tested | `ac33_e_a_slow_aircraft_hovers_short_of_the_zone_or_lands_in_its_cell` (raw speed 3; `quirks.aircraft_unit_step_stall` both ways as a consistency check only, the stall's own tests are in `claims_s0501.rs`) |
| S-0501 r167 AC33 f: over a docking position outside the pad's foundation it never touches down | built (already), tested | `ac33_f_over_a_docking_position_outside_the_foundation_it_never_touches_down` |
| S-0501 r167 claim 6 and claim 48 (b)'s third case: a held height of 0 with a target fraction of 1, in an update begun neither landing nor taking off, becomes the held height of ordinary flight | built, tested | `air::tick_flight`; `claims_s0501_r167.rs` `claim_29_a_full_speed_type_that_is_not_landable_keeps_cruise_height` (a fly-by type keeps cruise height; one not held at full speed descends to 0 with no landing, claim 50); `claims_s0501_r132.rs` `claim_7_an_untested_touchdown_that_keeps_the_destination_slides_then_climbs` |
| S-0501 r167 claim 6: being given its flight point again changes nothing in the speed fraction | aligned | `air::guard` cruise block cites; no code change |
| S-0501 r167 claim 29: the settled question in claim 24's contact rule outside claim 29 (`air::pad_accepted` asks no settled question elsewhere) | left | the general rule is not restated by r167; only claim 29's path asks it here |
| S-0501 r172 claim 18, *Missile types* (Q-2821): the missile-spawn flag alone decides; a flagged type that is not a rocket type takes the ground rows in flight, whatever its dock types or pad-bound field | built (interim as answered), tested | `air::fall_back_rows` (the `// design: Q-2821` line replaced by the r172 cite); `claims_s0501.rs` `claim_18_r154_a_missile_type_takes_the_ground_rows_in_flight`; `claims_s0501_r167.rs` `ac35_a_flagged_type_that_is_not_a_rocket_type_takes_the_ground_rows_in_flight` (with the unflagged control) |
| S-0501 r172 claims 17, 34 and 45: a visiting aircraft of a missile type takes the *Missile types* rows (forgets the edge cell, *retreat* chooses it again with its draws) | built (same flag), not tested on its own | `air::fall_back_rows` (the visiting ground row); `unmeasured` in the spec, no scenario sets it up |
| S-0501 r172 claim 35: a flagged slot type's run end falls back by the *Missile types* rows | built (same flag), tested through AC35 | `air::fall_back_rows`; `ac35_a_…` |
| S-0501 r172 claim 20: a released flagged object is a kamikaze removed after its first ordinary shot | blocked (Q-2868) | no code; S-0522 claim 11, S-0506 Terms, claims 9 and 12 checked, none gives the removal's moment or effects |
| S-0501 r172 claim 29 and T4: one returning with ammunition left and a controller target becomes attacking; the *guard* processing below cruise height can find it a pad | built (by claim 29's path), not tested on its own | `air::guard`; spec rates it `unmeasured`, medium |
| Intake 114 claim 52, claims 26 and 28, `quirks.aircraft_pad_sleep_ends_retaliation` | checked, as built | `claims_s0501.rs` (`claim_52_*`, `claim_26_r154_*`), `claims_s0215_air_retaliate.rs` pass unchanged |
| Intake 114 row 22 (Q-2821 interim) | replaced | see the r172 claim 18 row; `intake114.md` row notes it |

## Quirks

| Quirk | Default | Tests on and off | C7 |
|---|---|---|---|
| `aircraft_descent_touchdown_untested` | on | `claims_s0501_r132.rs` `claim_7_…` (on), `claims_s0501.rs` `claim_7_refused_touchdown` (off, through `tested_touchdowns`); `claims_s0501_r167.rs` `ac33_c_…` (both) | rejected, as before |
| `aircraft_unit_step_stall` | on | `claims_s0501.rs` `claim_6_unit_step_hover_stops_a_move` (on), `claim_5_unit_step_stall_off_advances_and_fires` (off) | near miss, not admitted |
| `landing_own_cell_no_result` | on | `claims_s0501.rs` `claim_41_own_cell_no_result_gives_the_placeholder` (on), `claim_41_own_cell_no_result_quirk_off_falls_back` (off) | first verdict: not admitted at this intake, flagged to the lead as a candidate |
| `unarmed_pad_bound_hover` | on | `claims_s0501_r93.rs` `claim_50_an_unarmed_pad_bound_aircraft_hovers_near_its_pad` (on), `quirk_unarmed_pad_bound_hover_off_docks_at_a_free_pad_and_hovers_at_a_taken_one` and `quirk_unarmed_pad_bound_hover_changes_nothing_for_a_type_that_is_not_landable` (off) | rejected, as at intake 76 |

## Formats

Save format 118 → 119, AI game format 84 → 85, one past intake 116's (main bdb91f1): claim 29's
contact for *guard* changes the hashed state (before the rebase, the sim-gate skirmish golden first
differed after tick 1419). Goldens regenerated on the tree rebased onto bdb91f1.

## Lanes

Before: 1c88f74 (code-identical to origin/main 7006a96, this branch's base before the rebase).
After: this branch's code commit before the rebase (0891fb9, on 7006a96), now b536a7f on main
bdb91f1 (intake 116). The sweeps, arena and quick lane were not run again on bdb91f1: a fair pair
there needs a new base binary as well, and the rebased goldens move in the same five files with
the same line counts as before the rebase (combat, scripted, duel-m7a and duel-m7b unchanged), so
the change does not meet intake 116's in the gate runs. `glasnost-ai --include-ignored` was run
again on the final tree.

- **Sweeps, seeds 1 to 16.** M7a and M7b identical. M6c changes: seed 11 no longer ends early
  (9473 → 12000 ticks, smallest counter 0 → 4), seeds 9 and 13 rise from 2 to 4; seed 1 ends
  early at 9605 as before, so the smallest minimum stays 0.
- **Arena** (`arena --record all --threads 2`): identical apart from timings (M-0001 12 of 96,
  M-0002 2 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0020 1 of 25, M-0021 11 of 50 failing, as before).
- **`glasnost-ai --include-ignored`:** 264 passed, 0 failed, as before.
- **AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, all side pairs, 36 games): 5 games change,
  all on map 0, and no winner changes: seed 7 sides 0 and 2 (39143 → 40346), seed 8 sides 0 and 1
  (74219 → 51339) and sides 0 and 2 (55288 → 63388), seed 9 sides 0 and 1 (38084 → 44160), seed 10
  sides 0 and 2 (55641 → 51085). With no winner change there is no trace to make.
