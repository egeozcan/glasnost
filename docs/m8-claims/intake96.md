# S-0501 r120 and r121: claim ledger, M8 step A3 intake 96

- **S-0501** r111 to r120 (commits ecc509e and 21712e0, from the r110 that intake 92 built and the
  r111 that intake 94 built), then r121 (a59d1e7). r120 answers Q-2506 (the docking stage in
  contact that finds the aircraft in flight and not landing), Q-2507 (the landing point of an
  aircraft that lands where it is) and Q-2508 (the order of a pad's run-away notices), which intake
  92 filed with interim rules. It rewrites claim 6's *landing where it is*, claim 7's touchdown
  (the forget before the cell test, the hold above a refused cell), claim 8's exception, claim 24's
  entry report after a refused touchdown in contact and its run-away notices, claim 45's
  computer-controlled area guard at the edge cell, and adds AC4's same-cell runs (1) to (3) and
  AC31 part h (4) and (5). r121 adds T2's row for a visiting aircraft on the ground, without
  passengers, in a team, and the matching exceptions in claims 17, 18 and 45.
- No quirk is added or changed: `quirks::tests::every_spec_quirk_has_a_switch` passes with no new
  switch, and c7-play-fixes.md gains nothing.

Requests filed: Q-2576 (an aircraft given no destination while it lands, Open question 28),
Q-2577 (landing where it is over a cell outside the usable area), Q-2578 (the flight after a
refused touchdown that kept the destination), Q-2579 (when an entry restarted by the pad's answer
in the docking stage is next processed). Q-2580 to Q-2585 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **Landing where it is** (`air::given_no_destination`, `AirState::own_point`). An aircraft in
  flight, moving and not landing that is given no destination gets a new flight point with no
  destination object: the centre of the cell it is over when F7 and the reservation test accept
  it, else of claim 40's replacement around it, with that search's draws. With *attack* current
  and no target the point is an F8 landing spot; with *attack* current or queued and a target the
  flight point is kept (claim 42). Moving means holding a flight point (pitch is not modelled,
  claim 38). It runs at the destination assignment (`air::destination_taken`), at the stop order
  (`orders.rs`), and at the next movement update after any other clear (`air::tick_flight`). The
  flight flies to that point (`air::flight_target`), its landing trigger starts the landing in
  that cell, and the descent move, which needs a destination, does not steer it. An aircraft with
  no flight point and no destination starts no landing (`air::landing_trigger`); intake 92's
  "no destination in flight lands" branch is gone.
- **Claim 8's exception** (`air::ignores`, `air::give_point`, `AirState::ignored`). The point it
  is landing at is its flight point, never its position (intake 92's interim is gone). The cell
  compared is that of the new destination's heading point. The test is made only when the
  destination is handed over; the ignored destination stays the aircraft's destination, so the
  descent move draws toward its cell centre, and the flight keeps its flight point while that
  destination stands.
- **Claim 7's touchdown** (`air::touchdown_forgets`, `AirState::stranded`). Before the cell test
  the touchdown forgets the destination and the flight point when the aircraft is in the flight
  point's cell, or wherever it is when that cell holds its contact (for an aircraft with no
  contact, a cell with no building). A refused touchdown rises 10 leptons and starts a take-off
  that ends in the next movement update and keeps the landing's held height (no longer the cruise
  height). After a forget nothing moves the aircraft: it holds 10 leptons above the refused cell,
  not landing, until it gets a destination.
- **The docking stage in contact, in flight and not landing** (`air::enter`). Its processing
  reports entry at once (`air::report_entered`), 1 tick, the aircraft left ready; it is no longer
  sent back to the approaching stage (intake 92's interim is gone). When the pad's answer to the
  second rule's request restarted entry (a behaviour was queued), the processing ends with 1 tick
  and no report (Q-2579, interim).
- **The run-away notices** (`air::sale_started`): the contact-place order is now the spec's, cited
  as such.
- **r121**: T2's on-the-ground visiting row in a team never applies, since our engine has no teams
  (claim 18); the citation in `air::fall_back_rows` says so. No code change.

`AirState::own_point`, `ignored` and `stranded` are hashed state: save format 98 → 99, AI game
format 61 → 62.

## Goldens

Regenerated on the rebased tree (`GLASNOST_UPDATE_GOLDEN=1`, sim-gate `--include-ignored`):
`duel` (from line 1180), `duel-diagonal` (1181), `duel-naval` (1181), `skirmish` (1163) and
`duel-m6c` (16) move: the new hashed fields and the r120 flight. `combat`, `duel-m7a`, `duel-m7b`
and `scripted` do not. `cargo xtask gate` (native and wasm32) matches all nine.

## Claims

### S-0501 r120 and r121

| claim | status |
|---|---|
| 6, landing where it is: a new flight point, its own cell when F7 and the reservation test accept it, the landing starts in that cell, the descent does not steer, the touchdown within claim 24's bound | Tested: `claims_s0501_r120::ac4_run_1_landing_where_it_is_starts_in_the_cell_and_does_not_steer` (AC4's same-cell run (1)) |
| 6, the cell refused: claim 40's replacement search with its draws | Tested: `claim_6_a_refused_cell_takes_the_replacement_search` |
| 6, *attack* current with no target: an F8 landing spot; *attack* with a target keeps the flight point | Partly: built; the target case is claim 42's existing tests, the F8 case is not run |
| 6, an aircraft that is not moving (no flight point) given no destination: nothing changes | Partly: built; the hold with no flight point and no destination is tested (`claim_7_a_refused_touchdown_in_the_flight_points_cell_forgets_and_holds`, `claims_s0501::claim_45_…`), a clear given during the hold is not |
| 6, given no destination while landing | Seam (Q-2576, Open question 28): the landing goes on to its flight point, interim |
| 6, over a cell outside the usable area | Seam (Q-2577): the selection starts from the cell it is over, interim |
| 8, the exception at hand-over: a destination other than the flight point whose heading point is in the flight point's cell is ignored, kept as destination, the flight point and the landing kept | Tested: `claim_8_a_destination_object_whose_heading_point_is_in_the_flight_points_cell_is_ignored` (a vehicle standing in the flight point's cell as destination; fails with the exception switched off by hand) |
| 8, the flight point's cell, not the position; a destination in the cell it is now over makes it take off; the descent drawn toward the destination's cell | Tested: `ac4_runs_2_and_3_the_point_compared_is_the_flight_points_cell` (AC4's runs (2) and (3), with the flight point set by hand in the next cell; run (2)'s destination is the flight point itself, so it shows the outcome, not the exception), `claims_s0501_r110::ac4_same_cell_…`. Partly: AC4's drift out of the flight point's cell is not produced, the flight point is set by hand |
| 7, the forget in the flight point's cell; the 10-lepton rise; the take-off ending at the next update with the landing's held height; the hold until a destination | Tested: `claim_7_a_refused_touchdown_in_the_flight_points_cell_forgets_and_holds` |
| 7, the forget outside the flight point's cell: no contact and no building there, forgotten; a building that is not the contact, kept; the contact's cell, forgotten | Tested: `claim_7_the_forget_outside_the_flight_points_cell` (the three cases) |
| 7, the flight after a refusal that kept the destination | Seam (Q-2578): ordinary flight toward that destination with ordinary flight's held height, interim |
| 24, a refused touchdown at the docking position, forgotten; the report in flight at the next processing, 1 tick; *guard* current, service at once | Tested: `ac31h4_a_refused_touchdown_at_the_docking_position_reports_entry_in_flight` (AC31 part h (4)) |
| 24, the same with a behaviour queued: entry starts again | Partly: built (the pad's answer); the processing's delay and the next processing are Q-2579's interim |
| 24, the run-away notices in contact-place order; no spot depends on another | Tested: `ac31h5_the_run_away_spots_do_not_depend_on_each_other` (AC31 part h (5), runs 5a and 5b) |
| 29, an unarmed aircraft at cruise height outside a team: its own cell and *move*, 45 ticks, the ready mark left as it is, it lands | Tested: `claim_29_an_unarmed_aircraft_at_cruise_height_lands_in_its_own_cell` (both mark positions; intake 92's seam) |
| 31, *move* for a type that carries vehicles | Seam: r120 still names no processing for it (not built) |
| 45, the computer-controlled area guard: the edge cell kept until the touchdown there, which forgets it; the refusal and the hold above the edge cell | Tested: `claims_s0501::claim_45_computer_area_guard_at_flight_height`. Seam: what *area guard* does after (Open question 18) |
| 46, AC29 part f (5): a pad-bound aircraft on its pad, its ammunition rising, when sold crashes | Tested: `claims_s0501::ac29f5_pad_bound_on_its_pad_with_ammunition_rising_when_sold_crashes` (intake 92's seam) |
| T2 (r121), on the ground, visiting, no passengers, in a team | No change: our engine has no teams (claim 18) |

## Lanes

Before: the pre-rebase base c09e56d (binaries built there); after: this intake rebased on main
ddda868, which since gained only tests, docs, the M-0021 record and the r121 text (seam sweeps 5a
and 5b); same machine, release.

| lane | before | after |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest counter 0 | smallest counter 0; minima and medians move (the duel has aircraft; seeds 5, 9 and 12 end early instead of 5 and 16) |
| M7a sweep, seeds 1 to 16 | smallest counter 0 | identical minima and medians |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | identical minima and medians |
| arena (`--record all`) | 13 of 155 pairings fail; M-0021 22 of 24 rows | identical rows and recorded files: the same 13 (ROBO–HTK, TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT, CAOS–E1, DISK–FV, INIT–E2, MIND–E1, CAOS–APOC, CAOS–TTNK, CAOS–E2, YURI–SHK), none changed; M-0021 22 of 24, the same two rows (also 22 of 24 for the base binary on the revised record) |
| `glasnost-ai --include-ignored` (33 binaries) | 248 passed | 248 passed |
| AI quick lane, seeds 7 to 10 × sides 0,1 / 0,2 / 1,2 × maps 0, 2, 8 | 36 of 36 games over with a winner | 36 of 36 over with a winner |

The quick lane: 32 of the 36 games change course, sides 1,2 included (unlike intake 92), since the
r120 rules reach every aircraft. Four are unchanged. House 0 won 8, 8 and 4 of 12 against the
other side (pairs 0,1 / 0,2 / 1,2) before, and 9, 7 and 3 after. The median game length per seed
was 42364, 41586, 41299 and 40980 ticks before, and 48248, 31330, 44134 and 43335 after. Aircraft
at pads: the lane counts no pad service, crash or hold, so it shows little. Distinct aircraft went
from 733 to 623 over 1 570 738 and 1 502 864 game ticks (46.7 and 41.5 per 100 000 ticks). In the
end-of-game army lines, *enter* with an object destination went from 46 to 53, and *enter* with no
destination from 3 to 1.
