# S-0309 r124, S-0501 r150 and S-0207 r85: claim ledger, M8 step A3 intake 112

- **S-0309** r120 to r124 (commit ef8315e, from the r120 that intake 110 built). r124 answers
  Q-2747. New claim 31 gives the orders reset of every owner change that claim 30 lists, in one
  order: the destination is cleared, the guard spot forgotten (a factory's rally point, a released
  vehicle's saved rally point; not by a vehicle in the middle of deploying or undeploying in place),
  a harvester's remembered cell forgotten, then *guard* is requested with its readiness-dependent
  start (a simple deployer in *unload* excepted), *rescue* current or queued is replaced by *guard*
  at once, and an object on the map that is not in contact with a vehicle factory falls back to
  rest (a building requests its idle body and queues *guard*). The override slot and the
  attack-move memory survive the change. Two quirks: `owner_change_keeps_override` and
  `owner_change_keeps_attack_move`, both on.
- **S-0501** r146 to r150 (commit 7de0c03, from the r146 that intake 108 built). r150 answers
  Q-2726: *sticky* is processed as *guard* (claim 29) with every queued behaviour waiting, the pad
  search is made whatever the height, a processing at cruise height chooses 1 tick, and claim 8's
  hand-over test covers the destinations that claims 18 and 29 give. AC33 parts a to c. No quirk.
- **S-0207** r83 to r85 (commit b4fe474, from r83, the revision the scatter-from-damage step
  built). r85 answers Q-2781: case b and case g read the vehicle's current locomotor, a gripped
  jumpjet scatters around the point it is heading for, and in the threat branch the destination is
  taken only when the locomotor accepts one, after the *move* request. No quirk.

Requests: Q-2801 (seven open cases of claim 31) and Q-2802 (three open cases of *sticky*). Block
Q-2801 to Q-2810; Q-2803 to Q-2810 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

### S-0309 r124 (claim 31, Edge cases)

- `structures::ownership::reset_orders` (new) makes claim 31's reset, in the claim's order:
  - with a quirk off, its clear first (the suspended slot and put-aside pair, or the attack-move
    memory and engaged mark);
  - the change-of-target test (`infantry::watch_test`) that S-0601 claim 9 gives;
  - the destination clear, only when there is one;
  - the guard spot, kept by `econ::inplace::in_transition`;
  - the harvester's remembered cell (`econ::harvest::forget_remembered`, new);
  - the readiness read and the *guard* request with its immediate start (`queue_and_start`), not
    for a simple deployer in *unload* (`mind::deploying_in_place`, now crate-visible);
  - the *rescue* rule (`beh.force(Guard)`);
  - the fall-back: none off the map, none in contact with a vehicle factory, none for a vehicle
    factory still in *unload*; a mobile object `process::fall_back`, a building
    `econ::body::enter_idle` (unless selling, being built up, or already in its idle range) and
    *guard* queued.
- Every listed cause calls it: `change_by` after the holder's `owner_changed` (every cause, the
  former `Cause::clears_orders` gone), `contacts` for a contact that follows the building,
  the trigger action's mobile branch, and the mind-control take and release
  (`mind::release_orders` gone). Claim 30's target drop is already in
  `econ::events::owner_changed`.
- Psychic domination makes no reset (`design: Q-2801`, question 7).

### S-0501 r150 (claims 8, 29, 33; AC33)

- `air::guard`: the claim 29 pad search no longer needs the aircraft in the air; a processing at
  cruise height returns 1 tick; *sticky* takes *guard*'s outcomes with the queued behaviour kept.
- Claim 8: `air::set_dest` hands the destination to the flight at once
  (`econ::assign_destination`), so claims 18 and 29's destinations are covered whatever the
  behaviour.
- Q-2802's three interims carry `design: Q-2802` lines.

### S-0207 r85 (claim 16)

- `scatter::vehicle_request`: case b tests the current locomotor (`lift::current_locomotor` is
  Teleport); case g tests `hover::powered` when the current locomotor is Hover on a hover type, and
  disabled otherwise.
- The threat branch queues *move* first and then takes the destination only when
  `lift::refuses_destination` lets it, with the depot and refresh marks inside that test.
- `vehicle_centre_cell`: a gripped jumpjet unit uses the cell it is heading for
  (`jump::heading_cell`, new).

## Claims

| Claim | Status | Tests and notes |
|---|---|---|
| S-0309 31: every listed cause requests *guard* (capture, trigger, occupation, take-over) | tested | `claims_s0309_r124::every_cause_and_the_takeover_request_guard`; `claims_s0309_r120::listed_causes_and_the_takeover_request_guard` now expects guard for all four |
| 31: a factory forgets its rally point | tested | `a_captured_factory_forgets_its_rally_point` |
| 31: a building requests its idle body and queues *guard* | tested | `a_captured_building_requests_its_idle_body` |
| 31: a building being sold keeps selling | tested | `a_building_being_sold_keeps_selling` (with Q-2801's interim 2) |
| 31: destination, guard spot and remembered cell forgotten | tested | `a_trigger_change_forgets_destination_guard_spot_and_remembered_cell` |
| 31: an infantry unit's fall-back does not return to its guard spot | tested | `an_infantry_unit_does_not_return_to_its_guard_spot` |
| 31: *rescue* current is replaced by *guard* | tested | `rescue_current_is_replaced_by_guard` |
| 31: *wait* with *rescue* queued | tested | `wait_with_rescue_queued_keeps_both` (Q-2801's interim 6) |
| 31 and Edge cases: an engaged vehicle resumes its attack move; quirk off forgets it | tested | `an_engaged_vehicle_resumes_its_attack_move`, `quirk_owner_change_keeps_attack_move_off_forgets_the_memory`, `a_refresh_used_on_the_tick_keeps_the_memory` |
| 31 and Edge cases: the override stays in effect; with `stale_suspended_revert` off a started guard ends it; quirk off empties the slot | tested | `a_vehicle_keeps_its_override_in_effect`, `with_stale_revert_off_a_started_guard_ends_the_override`, `quirk_owner_change_keeps_override_off_empties_the_slot` |
| 31: an aircraft gets its override back at the fall-back (S-0501 claim 18, T2 row 1); quirk off, no revert | tested | `an_aircraft_gets_its_override_back_at_the_fall_back`, `quirk_owner_change_keeps_override_off_no_revert_for_an_aircraft` |
| 31: a contact that follows the building is reset; one in contact with a vehicle factory gets no fall-back | tested | `a_contact_that_follows_the_building_is_reset` |
| 31: a vehicle deploying in place keeps its guard spot | partly | `econ::inplace::in_transition`; no test of its own |
| 31: a vehicle factory still in *unload* gets no fall-back | partly | by construction; no test of its own |
| 31: an object off the map gets no fall-back | partly | by construction; no test of its own |
| 31 Open questions 1 to 4, and three build cases | seam | Q-2801, interims 1 to 7 |
| S-0501 r150 29: a processing at cruise height chooses 1 tick | tested | `claims_s0501_r150::claim_29_a_processing_at_cruise_height_chooses_1_tick` |
| 29: the pad search is made on the ground too | tested | `claim_29_the_pad_search_is_made_on_the_ground_too` |
| AC33 (a): an unarmed aircraft in *sticky* lands short of its order | tested | `ac33_a_an_unarmed_aircraft_in_sticky_lands_short_of_its_order` |
| AC33 (b): a pad-bound aircraft in *sticky* with no pad crashes | tested | `ac33_b_a_pad_bound_aircraft_in_sticky_with_no_pad_crashes` |
| AC33 (c): with a pad it gets the pad and its contact | tested | `ac33_c_with_a_pad_it_gets_the_pad_and_its_contact`, also checking Q-2802's interim 2 on the following processings |
| 33: *sticky* gets no *move* at a movement update | tested | `claim_33_sticky_gets_no_move_at_a_movement_update` |
| 8: the hand-over covers claims 18 and 29's destinations | partly | by construction (`set_dest` hands over at once) |
| Open questions 32 and 33 | seam | Q-2802, interims 1 to 3 |
| S-0207 r85 16: a gripped unit scatters around its heading point | tested | `claims_s0207_r85::claim_16_a_gripped_unit_scatters_around_its_heading_point` |
| 16 e: a gripped unit with its destination is refused | tested | `claim_16_case_e_refuses_a_gripped_unit_with_its_destination` |
| 16: a gripped unit without a destination queues *move* only | tested | `claim_16_a_gripped_unit_without_a_destination_queues_move_only`, `claim_16_a_gripped_unit_without_a_destination_ignores_a_plain_request` |
| 16 b: the current locomotor of a seized teleporter | tested | `claim_16_case_b_reads_the_current_locomotor_of_a_seized_teleporter` |
| 16 g: the current locomotor of a seized hover | tested | `claim_16_case_g_reads_the_current_locomotor_of_a_seized_hover` |

C7 verdicts for the two quirks: [c7-play-fixes.md](c7-play-fixes.md), *A3 intake 112*. Both are
rejected (not in the layer).

## Records

Arena `--record all --threads 2`, main 7a6baff (before) against this branch on 7a6baff (after):
every output is byte-identical (M-0001 12 of 96 failing, M-0002 2 of 24, M-0003 2 of 35, M-0019 0
of 6, M-0021 11 of 50). None of the three specs reaches a row that moves.

M-0024 and M-0025 (S-0215 claim 19 case 2 for hunting vehicles of computer-controlled houses) get
local-lane tests: `arena::m0024_rows_hold_in_our_engine` and `arena::m0025_rows_hold_in_our_engine`
(`glasnost_tools::oracle_m0024` and `oracle_m0025`), each judging the record's Procedure on our
engine. The reports go to the git-ignored `local-lane/m7b/arena/oracle-M-0024.txt` and
`oracle-M-0025.txt`. Counts and install values stay local. Before: main 7a6baff's engine with only
the two new oracles added. After: this branch. Without `GLASNOST_GAME_DIR` both tests return
early, as the other arena tests do.

| Record | Row | Before | After |
|---|---|---|---|
| M-0024 | Part f: judged hits classed neighbouring destination or target: all | holds | holds |
| M-0024 | Part f: judged hits classed far destination or free: 0 | holds | holds |
| M-0024 | Part f: every judged hit naming no attacker classed neighbouring destination | fails | **fails the same way** (counts local) |
| M-0024 | Part fc: judged hits classed neighbouring or far destination: 0 | holds | holds |
| M-0024 | Part fc: judged hits classed free or target: all | holds | holds |
| M-0024 | Parts f and fc: judged hits whose position changed from t − 2 to t − 1: 0 | holds | holds |
| M-0024 | Parts f and fc: un-hit free vehicles classed neighbouring destination: 0 | holds | holds |
| M-0025 | All six rows | hold | **hold** (report byte-identical) |

The M-0024 row fails because, in our engine, a part f hit naming no attacker leaves its vehicle
holding a target and no destination (classed *target*). The counts and ticks are local, in
`oracle-M-0024.txt`. S-0215 B6 (f1) expects, for a hit naming
no attacker as for one by an enemy, "a target, or a destination and no target". It says that no
single judged hit falsifies the claim, because the recorder does not see what happens within the
tick before the hit lands. So the row is not read as a fault of our engine, and nothing is built
for it. Its rows "all neighbouring destination or target" and "far destination or free: 0" hold.
The test still fails on that row, and is local only.

**Q-2782.** Both records agree with reading (a): the claims as built, where case 2 reaches a
computer-controlled house's vehicle in *hunt*. Reading (b) is ruled out: in both records, judged
hits on free hunting vehicles leave a neighbouring destination. M-0025 also answers question 3:
hits by the vehicle's own house, an allied house, or an enemy it cannot harm send case 2. M-0024
answers the no-attacker half of question 3. Q-2782 was already answered by S-0215 r137 and
S-0208 r58 with reading (a); these records confirm it and change nothing in the code.

## Goldens

Regenerated with `GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate --
--include-ignored` on main 7a6baff with this branch; every test passes. Six goldens move:
`duel.txt` (from line 1424), `duel-diagonal.txt` (1597), `duel-m7a.txt` (12), `duel-m7b.txt`
(570, two lines), `duel-naval.txt` (1498) and `skirmish.txt` (1419). `combat.txt`,
`duel-m6c.txt` and `scripted.txt` do not move. Save format 114 to 115, AI game format 79 to 80;
wire format unchanged (12). `data/schema/ruleset.schema.json` gains the two quirks (regenerated
with `GLASNOST_UPDATE_SCHEMA=1`).

## Lanes

Before: binaries built on main 7a6baff. After: this branch's code on 7a6baff. The sweeps, the
arena, the quick lane, the AI suite and the M-0024 and M-0025 runs use those two bases. The branch
was then rebased onto main ef069bf, which adds spec text only (S-0411 r57, S-0111 r57, S-0501
r154, S-0312 r66); no code moved, so the lane figures stand. On the rebased tree:
`cargo test --workspace --no-fail-fast` 7857 pass and 1 fails, `every_spec_quirk_has_a_switch`.
That test fails on main ef069bf too, on quirks this intake does not build:
`removed_carrier_keeps_tag` (intake 113's), `aircraft_pad_sleep_ends_retaliation` (S-0501 r154)
and `bomb_mark_water_death_on_deck` (S-0312 r66). `cargo xtask check`: deps, determinism, cite
and canary ok, stock guard 137 problems, all in measurement records this branch does not touch.
Clippy (`--all-targets -D warnings`), fmt, the wasm32 build of glasnost-app and `cargo xtask gate`
are clean (gate run before the rebase, which moved no code). Local lane; install values are not
quoted.

- **Sweeps, seeds 1 to 16.** M6c (reference and play fixes) and M7b reference are identical
  before and after, apart from timings. The others change counters but not their minimums:
  - M7a reference: seeds 12, 14 and 16 change; smallest minimum 1 before and after.
  - M7a play fixes: seeds 2, 13, 14 and 15 change; minimum 2 before and after.
  - M7b play fixes: seed 7's drain counters change (one more and one fewer order in two of
    them), and its berserk minimum goes from 22 to 19; drain minimum 3 before and after.
- **Arena, `--record all --threads 2`.** Every output file is byte-identical before and after:
  M-0001 12 of 96 failing, M-0002 2 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0021 11 of 50.
- **`cargo test --release -p glasnost-ai -- --include-ignored`** (`--no-fail-fast`). Before, on
  7a6baff: 33 binaries, 263 pass and 0 fail. After, at first: 262 pass and 1 fails,
  `skirmish::plays_to_the_last_side` ("the game is over within 40000 ticks"). The synthetic game
  now ends at tick 43618 with house 1 the winner, against 26793 with house 0 on main. Without the
  1-tick return it ends at 26793 again, so S-0501 r150 claim 29 moves it. It is not a stall: at
  the end house 1's jets still fly sorties between the pad and changing targets, and its
  production waits for money. Our engine has no teams, so the "outside a team" outcome reaches
  every aircraft (r110's design, unchanged). The test's tick budget (`LIMIT`, a test bound, not a
  rule) goes from 40000 to 60000, with the reason in its comment; the seed is unchanged. After
  that: 33 binaries, 263 pass and 0 fail, `plays_to_the_last_side` included.
- **AI quick lane, seeds 7 to 10, maps 0, 2 and 8** (`GLASNOST_AI_QUICK=1`).
  - 23 of 36 games change; all 36 end before and after. House 0 wins 22 before and 20 after.
  - Four winners change:
    - seed 7, map 0, sides 0 and 1: house 0 at 45962 before, house 1 at 104796 after;
    - seed 7, map 2, sides 0 and 2: house 1 at 52036 before, house 0 at 101669 after;
    - seed 7, map 8, sides 0 and 2: house 0 at 46370 before, house 1 at 46927 after;
    - seed 9, map 8, sides 0 and 1: house 0 at 30735 before, house 1 at 96615 after.
  - Traced with two variant binaries of the after code: one without both r150 changes to
    `air::guard` (the ground pad search and the 1-tick return), one without the 1-tick return
    only.
    - Seed 7 map 0, seed 7 map 8 and seed 9 map 8 return exactly to main's result on both
      variants, so S-0501 r150 claim 29's 1-tick delay at cruise height changes them.
    - Seed 7 map 2 ends at 89838 with house 1 winning on both variants: the winner is main's
      again, but not the end (52036 on main). So the 1-tick delay flips its winner, and another
      change of this intake (S-0309 claim 31 or S-0207 r85; not the ground pad search, since both
      variants agree) also moves the game.
    - First differences seen: seed 7 map 8 between ticks 23540 and 23560 (a disc's health, one
      hit missing); seed 9 map 8 between 14375 and 14450 (a new vehicle released from its
      factory's contact three ticks earlier).
  - Nothing is tuned toward the lane.

## Seams and open points

- Q-2801 (claim 31's four Open questions, a building being built up, *wait* with *rescue* queued,
  domination) and Q-2802 (*sticky* at a pad, at cruise with a pad's contact, with no slowdown
  distance), each with its interims in the code.
- Q-2792 (aircraft retaliation) is not answered by r150; its interim stays. The 64c600f build was
  checked against S-0501 claim 28, T1, T2 row 1 and T3, and S-0309 claims 18 and 25 h: no change.
  Claim 31 now gives an aircraft its override back at an owner change.
- `removed_carrier_keeps_tag` is intake 113's and is not touched here.
- M-0024's row "every judged hit naming no attacker classed neighbouring destination" is marked
  exact in the record. It fails in our engine before and after this intake: a hit naming no
  attacker is classed *target*. S-0215 B6 (f1) allows that, so nothing is built, but the local
  test `arena::m0024_rows_hold_in_our_engine` keeps failing on that row. Whether the test should
  assert it is left to the lead.
