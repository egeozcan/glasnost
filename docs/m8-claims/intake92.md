# S-0501 r110: claim ledger, M8 step A3 intake 92

- **S-0501** r103 to r110 (commit 4b34f37, from the r103 that intake 86 built). It answers
  Q-2436 (an aircraft in *enter*, in contact, whose destination is not its pad; the pad the
  pad-bound rule records for a cell destination; an entry report after a touchdown outside
  *enter*), Q-2437 (a lost pad and the wait, the move on to the docking stage without a contact,
  the docking stage's fall-back, a sale in the docking stage) and Q-2438 (the ready mark when
  *enter* is left by force), which intake 86 filed with interim rules. It changes claims 8, 24,
  26, 29, 31 and 46, adds AC31 part h, AC4's same-cell part and AC29 part f (4) to (6), and adds
  two quirks (`quirks.aircraft_grounded_enter_stall`, `quirks.aircraft_ready_mark_outlives_enter`),
  judged in [c7-play-fixes.md](c7-play-fixes.md) (both admitted, off).

Requests filed: Q-2506 (a processing of the docking stage in contact that finds the aircraft in
flight and not landing), Q-2507 (which position is the landing point of an aircraft that lands
where it is, for claim 8's exception), Q-2508 (the order of a pad's run-away notices). Q-2509 to
Q-2515 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The second rule and the load request** (`air::enter`, `air::load_request`,
  `air::destination`). After the pad-bound rule, at every processing of *enter*, an aircraft in
  flight with no destination (a gone pad counts as none) asks its contact to be loaded. A pad that
  is switched on confirms; a switched-off one refuses, which ends the contact; without a contact
  nobody confirms. Unless confirmed the aircraft falls back to rest at once, whatever is queued,
  1 tick, the ready mark left as it is. The only other request is a pad-bound aircraft's in flight
  in the approaching stage. A confirmed request for an aircraft that has settled (its locomotor
  stopped, no destination, or pad-bound with its own cell as destination) and that the pad
  accepts gives the pad as destination and queues *enter* with an immediate start, so a queued
  behaviour is lost and entry starts again.
- **The approaching stage in contact** tests neither the destination nor the position: landing,
  or a pad-bound aircraft on the ground and not landing, moves it on to the docking stage and
  clears the ready mark (3 ticks); anything else returns 3 ticks and changes nothing. The pad
  give-back (intake 86's interim for Q-2436) is gone. One that is not pad-bound on the ground
  stays, under `quirks.aircraft_grounded_enter_stall` on; off, it is moved on as a pad-bound one.
- **The docking stage in contact** tests no position either: the landing processings make the
  docking step toward the docking position, the first on the ground reports entry
  (`air::pad_turn` no longer needs the pad's cell). A processing that finds it in flight and not
  landing (after a refused touchdown off the foundation) goes back to the approaching stage
  (Q-2506, interim).
- **The entry report** (`air::report_entered`): the pad has the aircraft queue *sleep*, and the
  aircraft queues *guard*, which replaces it (S-0215 claim 7); no force.
- **The run-away notice** (`air::sale_started`, called from `econ::sell` next to
  `depot::sale_started`): to every aircraft in contact with a pad that starts selling, in
  contact-place order (Q-2508, interim): the contact ends, an F8 landing spot becomes the
  destination, *move* is queued, then in *enter* *guard* is queued (replacing it), in *sleep*
  *guard* with an immediate start. The wait and the docking stage without a contact no longer
  test the destination: they move toward a cell destination's centre and fall back with the
  destination kept (`air::docking_without_contact`, `air::without_contact`, `air::docking_step`).
- **Claim 8's exception** (`air::ignores`, in `air::give_point` and `air::tick_flight`): a landing
  aircraft given a destination in the cell of the point it is landing at ignores it. For an
  aircraft that lands where it is the point is its position now (Q-2507, interim).
- **Claim 29 at cruise height**: outside a team (our engine has none) an armed aircraft falls back
  to rest whatever its destination, setting the ready mark; an unarmed one takes its own cell and
  *move*.
- **T2's row of an aircraft already flying to its *enter* destination** needs only that
  destination (`air::fall_back_rows`).
- **The pad-bound rule** (`air::bound_away`) records a destination that is not a building as no
  pad, losing the earlier record.
- **The kept ready mark** (`AirState::mark_cleared`, `air::set_ready_mark`, `air::ready`): set at
  the start of entry, at every processing of *enter* without a contact the two rules let through,
  at every docking-stage processing and at claim 29's cruise-height fall-back; cleared only by the
  move on to the docking stage. Under `quirks.aircraft_ready_mark_outlives_enter` off,
  `air::ready_mark_after_enter` sets it whenever *enter* is not current (at the start of the
  aircraft's behaviour turn and before claim 31's third point), and `air::ready` reads it only in
  *enter*. Intake 86's derived `ready_mark_cleared` is gone.

`AirState::mark_cleared` is hashed state: save format 96 → 97, AI game format 59 → 60.

The two switches are new fields of `quirks.rs` with the spec's default (on), so
`quirks::tests::every_spec_quirk_has_a_switch`, which failed on main once r110 named them, passes;
`data/schema/ruleset.schema.json` gains their two entries.

## Goldens

Regenerated (`GLASNOST_UPDATE_GOLDEN=1`, sim-gate `--include-ignored`): `duel`, `duel-diagonal`,
`duel-m6c`, `duel-naval` and `skirmish` move from the first tick an aircraft is in the world (the
new hashed field, and the reworked *enter*); `combat`, `duel-m7a`, `duel-m7b` and `scripted`, with
no aircraft, do not.

## Intake 89's warp checks under r110

The r103 → r110 delta does not touch claim 1, claim 18 or any "cannot land" text (no hunk of
`git diff 57755b7 4b34f37` falls in either claim; the phrase does not occur in the delta).
`AirState::kept_filed` (claim 1, set in `air::placed`) and `warp::in_air` (claim 18's in-the-air
answer, through `combat::weapon::in_air`) hold unchanged.

## Claims

### S-0501 r110

| claim | status |
|---|---|
| 8, a landing aircraft given a destination in the cell of the point it is landing at ignores it; a destination two cells away makes it take off | Tested: `claims_s0501_r110::ac4_same_cell_a_landing_aircraft_ignores_a_destination_in_its_cell` (AC4's same-cell part and its control). Partly: the point of an aircraft that lands where it is is read as its position now (Q-2507) |
| 24, the second rule: in flight with no destination, the request; refused or unanswered, a fall-back at once whatever is queued, 1 tick | Tested: `claim_24_the_second_rule_in_flight_without_a_destination_falls_back`, `claim_24_the_load_request_confirmed_and_refused` |
| 24, the load request: confirmed by a switched-on pad; settled and accepted: the pad as destination, *enter* with an immediate start, the queued behaviour lost; refused: the contact ends | Tested: `claim_24_the_load_request_confirmed_and_refused` |
| 24, requests only in flight with no destination and for a pad-bound aircraft in flight in the approaching stage | Tested: `claim_24_no_load_request_with_a_destination_unless_pad_bound`, `claims_s0501_r99::claim_24_a_refusal_ends_the_contact_and_the_processing_without_one_follows` (now pad-bound in flight; with nothing queued it crashes, claim 46) |
| 24, the approaching stage tests neither destination nor position; a pad-bound aircraft on the ground moves on; one that is not stays (switch) | Tested: `claim_24_grounded_in_contact_stall_switch_both_positions` (both positions, and the pad-bound case) |
| 24, the docking stage tests no position: touchdown and report off the foundation, service there | Tested: `ac31h1_docking_off_the_foundation_reports_entry_and_is_serviced` (AC31 part h (1)) |
| 24, the docking stage in contact, in flight and not landing | Seam (Q-2506): back to the approaching stage, interim |
| 24, the entry report: *sleep* queued, *guard* queued after it; *guard* current at the end of that tick | Tested: `ac31h1_…`, `claims_s0501_r103::ac31g4_…` |
| 24, the run-away notice: contact ended, landing spot, *move* then *guard* in *enter*, *guard* at once in *sleep*, *move* stays queued in *guard* | Tested: `ac31h2_the_sale_while_docking_ends_the_contact_and_the_landing` (AC31 part h (2)), `claims_s0501_r99::claim_24_a_sale_during_the_approach_sends_the_run_away_notice`, `claims_s0501::ac29f4_…`, `ac29f6_…`. Partly: the order of several notices (Q-2508) |
| 24, the pad-bound aircraft over a docking position off the foundation: no landing, *enter* with the pad and contact, never serviced | Tested: `ac31h3_a_pad_bound_aircraft_holds_above_a_docking_position_off_the_pad` (AC31 part h (3)) |
| 24, the wait and the docking stage without a contact test no destination; the fall-back keeps the destination | Tested: `claims_s0501_r103::claim_24_after_an_accepted_touchdown_…`, `claim_24_after_a_refused_touchdown_…`, `claim_24_a_processing_without_a_contact_that_finds_it_landing_…` (the contact now ended by the test, since no request is made there) |
| 24, the pad-bound rule records a cell destination as no pad | Tested: `claim_24_the_pad_bound_rule_records_a_cell_destination_as_no_pad` |
| 26, servicing starts in the report's tick or the next; the first service step gives no round | Tested: `ac31h1_…` |
| 29, at cruise height outside a team: armed, a fall-back whatever the destination, the mark set; unarmed, its own cell and *move* | Tested (armed): `claim_29_guard_at_cruise_height_outside_a_team`. Seam: team members (our engine has no teams); the unarmed branch is built, not tested |
| 31, the kept ready mark: set and cleared where the claim lists; a forced change keeps it (switch) | Tested: `claim_31_the_ready_mark_after_a_forced_change_switch_both_positions` (both positions), `claims_s0501_r103::claim_31_…`. Seam: *move* for a type that carries vehicles (not built) |
| 46, a sale reaching an aircraft in contact: no crash on the ground; the take-off; the crash in flight, with or without ammunition | Tested: `claims_s0501::pad_bound_pad_sold_during_the_approach_crashes`, `ac29f4_pad_bound_standing_on_its_pad_when_sold_takes_off_and_crashes`, `ac29f6_pad_bound_asleep_on_its_pad_with_no_ammunition_when_sold_crashes` (AC29 part f (4) and (6)). Partly: part f (5), the half-full type, is not run |
| T2, the row of an aircraft already flying to its *enter* destination needs only that destination | Tested: `t2_the_enter_row_keeps_the_destination_and_gives_guard` |
| Edge cases, the two quirks | Tested in both positions (above); C7 verdicts in c7-play-fixes.md |

## Lanes

Before: main 445d0f1; after: this intake on it (main had not moved); same machine, release.

| lane | before | after |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest counter 0 | identical minima and medians |
| M7a sweep, seeds 1 to 16 | smallest counter 0 | identical minima and medians |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | identical minima and medians |
| arena (`--record all --threads 4`) | 13 of 155 pairings fail | identical rows and recorded files; the same 13 (ROBO–HTK, TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT, CAOS–E1, DISK–FV, INIT–E2, MIND–E1, CAOS–APOC, CAOS–TTNK, CAOS–E2, YURI–SHK), none changed |
| `glasnost-ai --include-ignored` (32 binaries) | 248 passed | 248 passed |
| AI quick lane, seeds 7 to 10 × sides 0,1 / 0,2 / 1,2 × maps 0, 2, 8 | 36 of 36 games over with a winner | 36 of 36 over with a winner |

As at intake 86, the 12 games of sides 1 and 2 are unchanged, and 20 of the 24 games with side 0
change course. Side 0 won 17 of its 24 games before and 16 after (against side 1: 10 of 12, then 8;
against side 2: 7, then 8). Median game length per seed: 42691, 37441, 37704 and 46224 ticks
before; 41299, 41586, 42364 and 42614 after.
