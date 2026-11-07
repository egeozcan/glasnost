# S-0501 r134 to r146: claim ledger, M8 step A3 intake 108

- **S-0501** r134 to r146 (commit 072bf02, from the r134 that intake 104 built). r146 answers
  Q-2676 (what an aircraft does when it survives the hit of a landing check's refusal with no free
  cell, Open question 28) and Q-2677 (*guard* with a target during the slide after an entry report,
  and where that aircraft comes to rest, Open question 30), which intake 104 filed as seams. It
  gives the survivor the map's origin point as flight point (claim 7), moves claim 33's *move* for
  an aircraft in *guard* to the movement update, at any height (claims 29 and 33), and traces the
  slide after the report to its end (claim 24, *Not pad-bound*; AC31 part h (6)). No quirk is
  added or renamed; the texts of `quirks.aircraft_edge_hold` and
  `quirks.aircraft_grounded_enter_stall` gain the survivor and the slid aircraft (C7 entries in
  [c7-play-fixes.md](c7-play-fixes.md)). Claim 32 and the Terms are unchanged, so A4 round 9's
  rocket-missile finding stands.

Requests filed (commit 4865616): Q-2726 (whether claim 33's *move* at the movement update also
comes for an aircraft in *sticky*, which S-0215 claim 5 processes as *guard*). Q-2727 to Q-2735
unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The origin point** (`AirState::origin`, `air::check_refused`, claim 7, Q-2676). Where no free
  cell is found the hit comes as before; the flight point then becomes the origin point (0, 0, 0),
  held as a flag with the destination the aircraft kept recorded in `ignored`, and the flight stays
  unfinished. The hit update's take-off step makes claim 8's requests toward the origin point
  (`air::end_refusal_take_off` with `ORIGIN`): the aiming request above the upper threshold, the
  body toward the origin point and full speed in the middle band, none lower. A destroyed aircraft
  holds the flag too and crashes as before.
- **The survivor's movement updates** (`air::flight_target`, `air::tick_flight`). While it holds
  the origin point and its destination is the one it kept (or none), `flight_target` gives no
  flight point, so claim 5's target calculation, the fixed-full-speed case, guidance and claim 6's
  triggers do not act; the forward step, the climb to the held height and the step of the current
  fraction go on. It stops in the air where its slowing ends, or flies on along the line toward the
  origin point at full speed. No movement update hands the kept destination back.
- **Its end** (`air::destination_taken`, `air::give_point`, `air::given_no_destination`). Any
  destination given (an order, or a processing such as the pad's answer that gives the pad back)
  clears the record of the kept one, so the next movement update hands it over and `give_point`
  ends the origin point. Being given no destination is claim 6's rule for a moving aircraft (a new
  point of its own); the clear of a kept destination that did not go through the assignment is met
  at the movement update as for any flight point.
- **Off the legal map** (claim 7, Open question 28). No new code: claims 4 and 10 act as for any
  aircraft. A survivor that may not leave the map now is moved on outside and never removed; every
  rule above goes on acting there (positions outside the map index no cell and panic nowhere: the
  long-run test runs 6000 updates and round-trips the save there).
- **Claim 33 at the movement update** (`air::guard_move`, Q-2677). After claim 31's first start
  point in each movement update, a living aircraft that is not pad-bound and not crashing, with
  *guard* current, a destination and nothing queued, gets *move* queued, on the ground included;
  claim 31's third point then makes it current. The branch in `air::guard`'s processing is removed
  (claim 29 r146: "at any height", at the movement update). *Sticky* gets no such *move*:
  Q-2726's interim (`// design:` cite in `air::guard_move`).
- **The slide after the report** (claim 24, *Not pad-bound*). No further code: the report forces
  *guard*, the same tick's movement update queues *move*, *move*'s first processing keeps the pad
  (claim 40) and takes it off, it lands again at the docking position and rests in *enter*, the
  stay of `quirks.aircraft_grounded_enter_stall` (off: it reports again and is serviced). The
  release in the report's tick (claim 26) forces *guard* and empties the queue, T2's ground row
  forgets the pad, and the next movement update gives it claim 6's own point and a take-off.
- **Format**: save format 110 and AI game format 75 (the new hashed field), one past main
  e0695d3's 109 and 74.

## Claims

All tests are in `crates/deterministic/sim/tests/claims_s0501_r146.rs` unless named otherwise. A
survivor is reached through S-0302 claim 8's stage 2 poison row (a poison demolition warhead and a
poison-immune type), which S-0302 claim 9 lets stop a hit that ignores defences.

| Claim (r146) | Status | Tests and notes |
|---|---|---|
| 7: the survivor's flight point is the origin point, its flight unfinished (Q-2676) | tested | `claim_7_a_survivor_holds_the_origin_point_in_each_band`; `claims_s0501_r134::claim_7_with_no_free_cell_it_takes_the_demolition_hit` (the destroyed aircraft) |
| 7: the hit update's requests by band (aim, body and full speed, none) | partly | `claim_7_a_survivor_holds_the_origin_point_in_each_band`: each band reached; the target fraction (1 in the middle band, 0 otherwise) and the flight toward the origin point are judged; the aiming request above the upper threshold is not observed (no facing is read) |
| 7: no target calculation, no guidance; forward step, climb and fraction step go on; it stops, or flies on at full speed | tested | `claim_7_a_survivor_holds_the_origin_point_in_each_band` (1500 updates per band: held height reached, stopped near the hit or flying on, no landing, no second check or hit) |
| 7: no movement update hands the kept destination back | tested | the same test (the pad kept as destination throughout) |
| 7: it lasts until a destination is given; a stop order is claim 6's | tested | `claim_7_an_order_ends_the_origin_point` (a move order hands the cell over at the next movement update; a stop order gives a point of its own) |
| 7: one with no destination in contact gets the pad back and flies back | tested | `claim_7_a_survivor_with_no_destination_gets_its_pad_back` (the pad's service step gives the pad, the next update hands it over, it flies back; the pad-bound rule then refuses each landing start off the foundation, so no second hit comes in this set-up) |
| 7: off the legal map, claims 4 and 10 as for any aircraft; one that may not leave is never removed; our rules go on there (Open question 28) | tested | `claim_7_a_survivor_flies_on_off_the_legal_map` (both positions of `quirks.aircraft_edge_hold`: the pad-bound survivor flies on outside, never removed, with either; a visiting survivor with the fly-back flag is held at the edge with the switch on and removed on leaving the legal map with it off; state hash survives a save and load outside the map) |
| 7: the hit still destroys an aircraft with no stage 2 row | tested | `claim_7_without_the_stage_2_row_the_hit_destroys_it` |
| 24: the pad-bound survivor in *enter* stays approaching and is never serviced | tested | `claim_24_a_pad_bound_survivor_in_enter_is_never_serviced` (stopped and flying on, 1200 updates) |
| 33: *move* at the movement update, at any height, on the ground included (Q-2677) | tested | `claim_33_guard_with_a_destination_gets_move_on_the_ground` (and none for a pad-bound aircraft, another behaviour, or no destination), `claim_33_in_flight_and_not_when_crashing` |
| 33: *sticky* | seam (interim Q-2726) | *guard* only, as the claim names it |
| 29: "at any height"; no *move* from the processing | tested | through the claim 33 tests; `claims_s0501::lost_target_no_dock_types_heads_for_the_edge_and_lands_near_it` now sees *move* current at the end of the tick *guard* becomes current |
| 24: the slide after the report: *move* in the report's tick, take-off at *move*'s first processing, a new landing, rest in *enter* (AC31 part h (6)) | tested | `ac31h6_after_a_touchdown_beside_the_docking_position_it_takes_off_and_lands_again`, both positions of `quirks.aircraft_grounded_enter_stall` (on: the stay, never serviced, never *sleep*; off: a second report and a round). In our engine every approach to this pad touches down in the docking position's cell, so the run moves the aircraft by hand at its landing start into each of five neighbouring cells, as part h (6) keeps only runs that touch down in one |
| 24: the release in the report's tick: *guard* current, nothing queued, the pad forgotten, claim 6's take-off, no *move* | tested | `claim_24_a_release_in_the_report_tick_leaves_guard` (set up at the report tick by hand: the pad's service step called directly) |
| 24: the slide into the flight point's cell before, or in, the report's tick | partly | no run of part h (6)'s five reaches the docking position's cell before *move*'s first processing (none is void); the rules involved (claim 7's exact-cell trigger on the ground, the forget, the report with no destination, the fall-back into *enter*) are tested in intakes 102 and 104 |
| 24: the report after a touchdown in the docking position's cell, and the other aircraft | no change | the other branches of the report are r134's |
| Edge cases: `aircraft_edge_hold` and `aircraft_grounded_enter_stall` texts | tested | see the rows above; C7 re-judged |

Earlier tests whose expectations moved: `claims_s0501::lost_target_no_dock_types_heads_for_the_edge_and_lands_near_it`
(claim 33's *move* now comes in the update in which *guard* becomes current) and
`claims_s0501_r134::claim_7_with_no_free_cell_it_takes_the_demolition_hit` (cite to r146; the
origin point asserted).

## Goldens

Regenerated on main e0695d3's goldens, through the state hash only: duel, duel-diagonal,
duel-m6c, duel-naval and skirmish. duel-m7a and duel-m7b do not move. The same five moved on each
earlier base (07d549c and 01c5688). Save format 110, AI game format 75 (main: 109 and 74).

## Lanes

The branch is now rebased onto main e0695d3 (intake 107). The lane figures below were taken on
earlier bases, and each says which: main 01c5688 (code equal to 926f78d's) with the branch rebased
onto it, or main 07d549c with the branch on it. They were not run again on e0695d3. The final
checks were run on e0695d3 with the branch on it: the workspace tests 7737 passed and 1 failed
(`quirks::tests::every_spec_quirk_has_a_switch`, from S-0208 r52 on main, intake 110's), 128
ignored; the simulation tests with `--include-ignored` 5438 passed and 1 failed
(`pathing_c7p4::diag`, as expected); the AI tests with `--include-ignored` 263 passed and 0
failed; clippy and fmt clean; `cargo xtask check` with the 128 stock-guard problems main has in
the lead's records; `cargo xtask gate` ok.

- **The M6c, M7a and M7b seed sweeps**, seeds 1 to 16, with and without `GLASNOST_PLAY_FIXES`: the
  same output line for line. Smallest counters: M6c 0 (median 4) both ways; M7a 2 with the layer
  and 1 without (median 2); M7b, no drain counter below 3 on any seed. The same held against main
  07d549c before round 9b.
- **The arena** (`--record all --threads 2`, main 07d549c against the branch on 07d549c):
  byte-identical reports and CSVs. M-0001 has 11 of 96 pairings failing, M-0002 2 of 24, M-0003 1
  of 35, M-0019 0 of 6, M-0020 1 of 25 (YURI against SHK, arm S) and M-0021 8 of 50. Round 9b
  (01c5688) changed only AI code, and the arena and oracle modules do not use the AI, so it was
  not run on 01c5688. It was not run on e0695d3 either: intake 107 changed the simulation, so
  main's merge lanes cover that base.
- **The AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games). Every game ends, before and
  after. 32 games are identical apart from timings; 4 games changed:

  | Seed | Map | Sides | Main | Branch |
  |---|---|---|---|---|
  | 7 | 0 | 0 and 1 | 62 196 ticks, house 0 | 52 253 ticks, house 0 |
  | 8 | 0 | 0 and 2 | 45 358 ticks, house 1 | 38 619 ticks, house 1 |
  | 9 | 0 | 0 and 1 | 33 361 ticks, house 0 | 33 982 ticks, house 0 |
  | 9 | 0 | 0 and 2 | 51 113 ticks, house 0 | 70 236 ticks, house 1 (the winner changed) |

  - **Trace of the changed winner.** The object dumps are equal up to tick 29 600 and differ at
    29 700. The first object that differs is house 0's ASW aircraft 11814, a destroyer's slot
    aircraft.
  - In tick 29 682 it leaves *attack* for *guard* with a destination cell, (35,43). On main it
    stays in *guard* for three ticks and gets *move* toward its destroyer at 29 685. On the branch,
    claim 33's *move* (r146, at the movement update) is current in tick 29 682 itself. Claim 35
    puts a slot aircraft that is not visiting under T2's rows, which for a human-controlled house
    give *guard* with claim 33's *move* (the AI tool's houses are player-owned to the simulation, as
    the trace's *guard* shows); which row applies is existing code, not this intake's. Its
    processing gives the destination (43,51), and at 29 685 it gets the same *move* toward its
    destroyer.
  - From there its flight is a cell and a tick apart, and the long game drifts. This is the change
    r146 asks for, not a defect.
  - Against main 07d549c (before round 9b), 2 of the 36 games changed and no winner changed.
- **`cargo test --release -p glasnost-ai -- --include-ignored`**: 263 passed and 0 failed on main
  01c5688, and the same on the branch. `skirmish::plays_to_the_last_side` passes on both, so the
  change does not move it. Against main 07d549c the result was 228 passed and 1 failed on both
  sides, that test failing the same way on both.
