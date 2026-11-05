# S-0501 r121 to r132: claim ledger, M8 step A3 intake 102

- **S-0501** r121 to r132 (commit 691cf14, from the r121 that intake 96 built). r132 answers
  Q-2576 (an aircraft given no destination while it lands), Q-2577 (landing where it is over a
  cell outside the usable area), Q-2578 (the flight after a refused touchdown that kept the
  destination) and Q-2579 (when an entry restarted by the pad's answer in the docking stage is next
  processed), which intake 96 filed with interim rules. It rewrites claim 7: claim 3's descent
  brings every landing to height 0 before its landing step, so no touchdown is tested; the drop and
  slide after a refusal that kept the destination; the landing's check of its flight point's cell.
  It rewrites claim 6's *landing where it is* (while landing; outside the usable area), claim 24's
  entry report after a touchdown beside the docking position and the restarted entry, claim 45's
  rest in the edge cell, adds AC4's runs (4) and (5), and Open questions 28 and 29.
- Three quirks are added, all new switches of `quirks.rs` with the original's position as default:
  `aircraft_descent_touchdown_untested` (claim 7), `aircraft_refused_kept_drop` (claim 7, Q-2578)
  and `aircraft_offmap_stop_start` (claim 6, Q-2577). C7 verdicts in
  [c7-play-fixes.md](c7-play-fixes.md): the first two rejected, `aircraft_offmap_stop_start`
  admitted (off). The layer holds nineteen switches.

Requests filed (commit 2b58fbd): Q-2646 (what follows the landing check's refusal, Open question
28), Q-2647 (tracking, listing and movement after an untested touchdown that kept the flight point,
Open question 29), Q-2648 (claim 6's exact-cell trigger while the aircraft slides on the ground),
Q-2649 (the entry report and the service after a touchdown beside the docking position), Q-2650
(an entry restarted by the pad's answer in the approaching stage). Q-2651 to Q-2655 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The untested touchdown** (`air::tick_flight`'s touchdown block,
  `quirks.aircraft_descent_touchdown_untested`, on). No cell test, no refusal and none of an
  accepted touchdown's effects, the forget of the destination in any case included: only the
  touchdown's own forget test (`air::touchdown_forgets`, intake 96's). A touchdown that forgot, or
  that had no flight point to compare, rests where it touched down (`air::touchdown` with
  `forget_dest` false). One that kept its flight point is on the ground by its height, not landing
  and not at rest, with its destination (if any) and its flight point; its tracking ends and it is
  listed in its cell layer, with no object-present flag (`air::untested_kept`). Off: the r120
  tested touchdown, unchanged.
- **The drop and the slide** (`AirState::sliding`, `quirks.aircraft_refused_kept_drop`, on). The
  update after a refusal that kept the destination drops it to height 0 by claim 3's descent (no
  touchdown) as its take-off ends; it is then on the ground by its height with held height 0. While
  it slides it makes claim 4's forward step with claim 5's fraction toward its flight point, no
  climb and no guidance (which acts only above the terrain); at the start of the update after the
  first one that leaves its target fraction at 1 it takes ordinary flight's held height and climbs
  in that update. An untested touchdown that kept its flight point slides the same way. A new
  destination takes it off (claim 8). Off: ordinary flight's held height at the next update, so it
  climbs at once.
- **The landing check** (`air::landing_cell_check`). At the start of each movement update of a
  landing, after a new destination's hand-over: a pad-bound aircraft whose flight point's cell
  holds no building it is in contact with takes off (claim 8's take-off, climbing in that update,
  destination and flight point kept; Q-2646 interim). For any other aircraft it refuses nothing.
- **Given no destination while landing** (`air::given_no_destination`, Q-2576). The same
  selection as in flight (the *attack* rules included); the flight point's cell keeps the landing
  with that flight point and no destination; another cell takes off toward its centre, handed to
  the flight only. `air::tick_flight` now reloads the whole flight state after that call while
  landing, so the take-off is not overwritten.
- **Outside the usable area** (`air::stop_start`, `quirks.aircraft_offmap_stop_start`, on;
  Q-2577). An aircraft that is not visiting and may not leave the map now, over a cell outside the
  usable area, starts the selection from the cell of the local point with one coordinate moved to 1
  or to one less than the local area's size (rows counted by *L*h), by S-0522 F2's inverse. Off:
  from the cell it is over.
- **The restarted entry in the docking stage** (`air::enter`, Q-2579). The pad's answer that
  restarts entry makes entry's beginning in that processing: the stage back to approaching, the
  ready mark set, 1 tick; no report, no docking step, no fall-back. It is tested before the
  docking step, since the second rule's aircraft is in flight. The approaching stage is unchanged
  (Q-2650 interim).
- **The report on the ground, not at rest** (`air::enter`'s docking stage, Q-2649 interim). A
  processing that finds the aircraft on the ground by its height and not at rest (sliding, or after
  an untested touchdown that kept the destination with the drop switch off) reports entry as the
  report in flight does (`air::report_entered`), without docking or turning it; claim 26's service
  skips it until it rests.
- **A sliding aircraft whose flight point is gone** comes to rest where it is, with its
  object-present flag (Q-2647 interim).

`AirState::sliding` is hashed state: save format 101 → 102, AI game format 64 → 65.

## Claims

All tests are in `crates/deterministic/sim/tests/claims_s0501_r132.rs` unless named otherwise.

| Claim (r132) | Status | Tests and notes |
|---|---|---|
| 7: the untested touchdown (forget test only; rest, or on the ground with the flight point kept) | tested | `claim_7_an_untested_touchdown_in_an_occupied_cell_rests_there`, `claim_7_an_untested_touchdown_that_keeps_the_destination_slides_then_climbs` (both drop positions); off: `claim_7_off_a_tested_touchdown_on_open_ground_forgets_the_destination_in_any_case`. The r120 refusal tests (`claims_s0501_r120`, `claims_s0501_r103`, `claims_s0501::claim_7_refused_touchdown`, `ac27_a_refused_touchdown_clears_the_mark`) now set the switch off. |
| 7: the drop and slide after a refusal that kept the destination (Q-2578) | tested | `claim_7_a_refusal_that_kept_the_destination_drops_back_to_the_ground` (both positions) |
| 7: the landing's check of its flight point's cell | tested | `claim_7_the_landing_check_refuses_a_pad_bound_flight_point_off_its_pad`; what follows the refusal is Q-2646's interim |
| 6: given no destination while landing (Q-2576), AC4 runs (4) and (5) | tested | `ac4_run_4_given_no_destination_over_its_flight_points_cell_the_landing_goes_on`, `ac4_run_5_given_no_destination_over_another_cell_it_takes_off_and_lands_there`, `claim_6_given_no_destination_while_landing_in_attack_with_no_target_takes_an_f8_spot` |
| 6: outside the usable area (Q-2577) | tested | `claim_6_given_no_destination_outside_the_usable_area_starts_from_the_local_edge` (both positions) |
| 24: the restarted entry in the docking stage (Q-2579) | tested | `claim_24_an_entry_restarted_in_the_docking_stage_makes_its_beginning_at_once` |
| 24: the report after a touchdown beside the docking position, AC31 part h (4) | tested (interim Q-2649) | `ac31h4_an_occupant_at_the_docking_position`, `ac31h1_after_an_untested_touchdown_beside_the_docking_position_it_comes_to_rest` (both drop positions come to rest) |
| 24: an entry restarted in the approaching stage | no change (interim Q-2650) | unchanged code path |
| 45: the rest in the edge cell | tested | `claims_s0501::claim_45_computer_area_guard_at_flight_height`, both positions of the untested switch (on: at rest in the edge cell, outside the usable area; off: r120's hold) |

Intake 96's seams:

| Seam | Status | Tests and notes |
|---|---|---|
| Claim 6's F8 case in flight (attack current, no target) | tested | `claim_6_given_no_destination_in_flight_in_attack_with_no_target_takes_an_f8_spot` |
| AC4's drift out of the flight point's cell | tested | `ac4_a_touchdown_that_drifts_out_of_the_flight_points_cell_rests_there` (set-up by hand: the landing under way at full speed fraction at the flight point cell's edge along its heading; a stop's landing here starts once the aircraft has slowed to 0, so at the test speeds it touched down in its own cell) |
| A clear given during claim 7's hold | tested | `claim_6_a_stop_during_claim_7s_hold_changes_nothing` (switch off, the only position with the hold) |
| Claim 31's *move* for a vehicle-carrying type | seam | r132 does not change it |
| Claim 45 after the hold | tested at the default; seam off | at the default the hold no longer arises (it rests in the edge cell); with the untested switch off what follows the hold stays Open question 18 |

## Goldens

All five sim-gate goldens were regenerated (`duel`, `duel-diagonal`, `duel-m6c`, `duel-naval`,
`skirmish`): each agrees with the old one over its first part (for `duel`, its first 1176 hashes)
and differs from then on; the formats moved too (save 102, AI game 65).

## Lanes

Before: main at 5061dee. After: this branch. Never tuned toward a lane.

- Sweeps M6c, M7a and M7b, seeds 1 to 16: identical line for line, minimums and medians included
  (smallest minimum 0, 0; smallest drain minimum 3).
- Arena, every record (`--record all --threads 2`): identical. The same 13 pairings fail (ROBO–HTK,
  TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT, CAOS–E1, DISK–FV, INIT–E2, MIND–E1, CAOS–APOC,
  CAOS–TTNK, CAOS–E2, YURI–SHK), with the same counts; M-0021 fails the same 3 of 41 rows.
- `glasnost-ai` with `--include-ignored`: 248 passed, 0 failed, before and after.
- AI quick lane, seeds 7 to 10, maps 0, 2 and 8: 9 of 36 games change length, every one with the
  same winner (seed 7: three games; seed 8: one; seed 9: three, each longer, 74593 → 97554,
  41919 → 51175, 50846 → 64100 ticks; seed 10: two). The lane counts aircraft built but has no
  counter for aircraft at pads; the distinct aircraft counts move with the games' divergence and
  no report line shows aircraft held at or around a pad.
