# S-0215 r132, S-0208 r46, M-0023: claim ledger, M8 step A3 intake 106

- **S-0215** r118 to r132 (r132 at 8b1743c), answering Q-2656. Claim 22 gains the *End-of-update
  check (infantry)*: at the end of each update of an infantry unit, after its late start, a unit
  whose current behaviour is *move* (or none, with *move* queued) that is not moving and has no
  destination falls back to rest (claims 13 and 14), whatever is queued and whether or not *move*
  was processed. What it queues is current at the next tick's first start if the unit is ready
  there. An update that ends before its late start has no check. It catches arrivals, gate
  failures, the stop order and a teleporting infantry unit's jump tick. Claim 6 (the rate after the
  route row's *wait*, B15 g), claim 12 (a stopped infantry unit falls back at the end of the update
  after which it has stopped) and claim 13 (the route row's *wait* with an immediate start, S-0213
  claim 16c) are rewritten to match. New scenario B49 (parts a to d); B6 part e's run C and B15
  part c rewritten. Open questions on the check's other cases (a unit in *move* holding a
  destination; vehicles) and on team members. Changes entries for S-0212 claim 23, S-0417 claim 17,
  S-0207 claim 20 (e) and S-0309 claim 23 (d).
- **S-0208** r34 to r46 (r46 at 8ce9bf6), answering Q-2657. Claim 6 gains *Destinations given
  during a unit's update* (a destination given before the update's locomotion is searched for in
  that update, one given after it in the next tick; a walker standing with no walking target and no
  stored path that accepts a scatter whose destination comes from the nearby-cell search gets one
  more locomotion at once, in which it chooses its walking target without moving, with claim 3's
  draw; a neighbour choice gets none) and *A new destination while a step is under way* (the walker
  keeps its walking target, finishes the step and drops its stored path, except under *enter*,
  where it keeps the path). Claim 15's request B finds request A's walking target. Scenario W17 (a
  to e) is new; W7 and W9 change. Changes entries for S-0212 claim 20, S-0209 Terms and S-0417.
- **M-0023** (bd5d144 to 811c89f): Procedure step 5 also takes the arrival tick; the controls'
  destination still reads the pen's centre at the end of the arrival tick, is cleared in the next
  tick with *move* current and *guard* queued, and *guard* is current in the tick after; the
  baseline's walk is a single step and its destination is cleared in its arrival tick. These are
  recorded outside the verdict; the 13 Result rows are unchanged.
- No quirk is added or renamed in either spec (S-0208 r46 only rewraps a line of claim 15 that
  says the damage scatter's miss has no switch), so no new switch. C7 in
  [c7-play-fixes.md](c7-play-fixes.md), *Intake 106*: the six own pathing improvements fix the same
  things. The layer file is unchanged.

Requests filed: Q-2701 (which walkers under *enter* keep their stored path at a new destination;
not built), Q-2702 (an extra locomotion inside another; a `// design:` interim). Q-2703 to Q-2710
unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The end-of-update check** (`process::end_of_update_check`, called from `tick::object_turn` at
  the end of an infantry unit's turn, after `scatter::resting_turn` and before
  `lift::end_of_turn`). It reads the behaviour claim 1 sees (`Behaviours::seen`), the destination
  and the same moving test as the *move* processing (`process::moving`), and calls
  `process::fall_back`, so claim 13's common effects (at most once per tick, the attack-move
  refresh), the route row's *wait* and claim 14's rows apply as at any fall-back. The turns that
  end before the late start (lost, dead, erased, a warp's leaving stage and its earlier update
  boundaries) return before it, so they have no check. A host killed by its parasite (S-0614
  claim 18) returns after its late start but before the check; it is dead, and the check is for
  living units only, so it makes no difference. Infantry
  only: the Open question on vehicles is left as the spec leaves it, and a unit in *move* holding a
  destination is left alone (Open questions). Its place among the end-of-turn items is a
  `// design:` note: after S-0417 claim 17's request, whose destination it then sees.
- **The extra locomotion** (`walk::extra_locomotion`, from `scatter::answer`). `walk::tick`'s
  standing branch is now `walk::standing`, the locomotion of a walker on a standing position with
  no walking target (search, then ask and choose). `scatter::answer` reads `walk::standing_free`
  (no walking target, no stored path) before the request drops the path, and after an accepted
  request whose destination came from the nearby-cell search calls `walk::extra_locomotion`, for
  both the move-order and the resting (S-0417 claim 17) assignments. It runs `walk::standing`
  for a unit that moves under its own walk locomotor now (`walk::walks_now`: on the map, not
  riding, not inside, not falling, not seized); a teleporting unit keeps S-0417 claim 17's jump.
  The arrival-tick rule of the walker's own locomotion is not part of it, as claim 6 states the
  choice without it. A neighbour choice (claim 12, claim 13's fall-back) has none.
- **Q-2702 interim** (`World::extra_locomotion`, a stack of the walkers whose extra locomotion is
  running; `#[serde(skip)]`, empty between calls, so neither saved nor hashed): a unit gets no extra
  locomotion while its own is already running further up the call chain. (Intake 110 renames it
  `World::locomotion_under_way`, which also holds the unit's update locomotion, as S-0208 r52's
  departure gives.)
- **Comments.** `tick.rs`'s note on S-0417 claim 17's request cites S-0208 r46 claim 6 for its
  place after the update's behaviour processing and locomotion; only its place after the late start
  stays `// design:`. No `// design:` interim of intakes 99 or 103 covered Q-2656 or Q-2657 in code
  (intake 103 built none; it pinned four M-0023 rows and one test assertion to them).
- **The M-0023 check** (`m0023_rows_hold_in_our_engine`) now requires every Result row to hold; the
  list of four rows pinned to Q-2656 and Q-2657 is gone. `oracle_m0023::run` also prints the five
  readings that the record keeps outside the verdict (see below), which the check does not judge.

## Claims

Tests in `crates/deterministic/sim/tests/claims_s0215_r132.rs` (S-0215) and
`claims_s0208_r46.rs` (S-0208), unless named otherwise. Synthetic data only.

| Claim | Status | Tests or reason |
| --- | --- | --- |
| S-0215 claim 22, end-of-update check, gate failure (B49 a) | tested | `b49_a_gate_failure_queues_the_rest_at_the_end_of_the_update`, `b49_a_the_move_processing_of_that_update_chooses_the_standard_delay`; `claims_s0417_r79.rs` `u10_g_run_2_gate_failure_then_the_walk_without_a_wait` (*guard* queued at the end of F, G = F + 1) |
| S-0215 claim 22, arrival (B49 b, B6 e run C) | tested | `b49_b_arrival_queues_the_rest_at_the_end_of_the_arrival_update`, `b6_e_run_c_infantry_queues_guard_at_the_end_of_the_arrival_update` |
| S-0215 claim 22, stop order (B49 c, claim 12) | tested | `b49_c_stop_order_queues_the_rest_at_the_end_of_the_step` |
| S-0215 claim 22, teleporting infantry's jump tick (B49 d, S-0207 claim 20 e) | tested | `b49_d_teleporting_infantry_queues_the_rest_in_its_jump_tick` |
| S-0215 claim 22, whatever is queued; *move* queued with nothing current | tested | `claim_22_check_with_move_queued_and_nothing_current` |
| S-0215 claim 22, no check with a destination or another behaviour; infantry only | tested | `claim_22_no_check_with_a_destination_or_another_behaviour`, `claim_22_vehicles_have_no_check` (both Open questions, our engine's reading) |
| S-0215 claim 13, route row's *wait* at the arrival update (B15 c) | tested | `b15_c_route_holder_waits_at_the_end_of_its_arrival_update` |
| S-0215 claim 14 row 6, zombie and paralysed *move* (B6 e runs Z and P) | tested | `b6_e_runs_z_and_p_infantry_queues_nothing` |
| S-0215 claim 6 (rate after the route row's *wait*, B15 g), claim 12 wording | no change | the rate rule was built by intake 99; claim 12's new wording is what the check does (B49 c) |
| S-0215 claim 22, a team member under a team script's move step | seam | Open question in the spec; no team script move step builds *move* for infantry here |
| S-0208 claim 6, destination before the update's locomotion (W17 a) | tested | `w17_a_order_at_the_start_of_a_tick_steps_in_the_next` |
| S-0208 claim 6, extra locomotion at a search scatter (W9 draws) | tested | `extra_locomotion_chooses_the_walking_target_at_once`, `extra_locomotion_before_the_own_locomotion_steps_that_tick`, `neighbour_choice_has_no_extra_locomotion`, `extra_locomotion_only_for_the_walk_locomotor`; `claims_s0417_r64.rs` `claim_17_one_draw_then_refusals` (the factory's request: one scatter draw, then the position choice's draw exactly when the walker's point is of centre class) |
| S-0208 claim 6, no extra locomotion with a walking target (Open question, our engine) | tested | `a_walker_with_a_walking_target_gets_no_extra_locomotion` |
| S-0208 claim 6, a new destination while a step is under way (W17 c to e) | tested | `w17_c_order_into_another_reach_region_during_a_step`, `w17_d_order_further_along_the_line_during_a_step`, `w17_e_order_to_the_cell_being_stepped_into` |
| S-0208 claim 6, *enter* keeps its stored path | seam | Q-2701: which walkers count as under *enter*, and where that path leads, are not given; not built. Answered by S-0208 r52 and built in intake 110 ([intake110.md](intake110.md)) |
| S-0208 claim 6, the factory request's tick (S-0417 claim 17, M-0023 baseline) | tested | `claims_s0417_r79.rs` (U10), `m0023_rows_hold_in_our_engine` (local lane) |
| S-0208 claim 15, request B finds request A's walking target (W7 j) | partly | follows from the extra locomotion (`extra_locomotion_chooses_the_walking_target_at_once`); no two-request test of the damage scatter (see Seams) |
| Q-2702 interim, an extra locomotion inside another | tested | `q2702_extra_locomotions_in_a_crowd_end`, `q2702_no_extra_locomotion_inside_its_own`. Retired in intake 110: S-0208 r52 makes it a departure (no extra locomotion while a locomotion of the unit's own is under way, `World::locomotion_under_way`) |

Older tests changed by these revisions, each with the claim it now cites:

- `claims_s0208.rs` `claim_24_scatter_is_no_escape`: the stranded walker's search destination
  is cleared at once by its extra locomotion's gate failure (S-0208 claim 6 r46, S-0212 claim 23);
  it still never leaves its cell.
- `claims_s0209.rs` `all_accepted`: a friend's answer counts when it has a destination, or when its
  extra locomotion's failed search cleared it first (traced before the answer).
- `claims_s0312_survivors.rs`: the survivor and sale replays take each survivor's extra locomotion
  draws right after its scatter draw. The count per survivor is read from the movement trace (new
  event `ExtraLocomotion`, diagnostics only); the S-0312 and S-0408 draws around them are still
  replayed exactly.
- `drive.rs` `duplicate_request_in_one_tick`: the helper also clears the walking target and path
  the walker's extra locomotion chose, so a second request reaches it again.
- `claims_s0402_slaves.rs` `s0203_claim_12_delivery_removal_counts_before_master`: slaves already
  taken inside have left the update order and are no longer compared; the test now also checks
  that every recalled slave gets back inside.

**M-0023** (with `GLASNOST_M0023_LOCAL`, local lane): 9 of the 13 Result rows held before; all
13 hold after. The four that now hold are the ones intake 103 pinned to Q-2656 and Q-2657: *at the
end of F: move current, guard queued, no destination*, *G − F*, *first step after G* and the
controls' row (*at t + 2 the pen's centre and move*). The five readings that the record keeps
outside the verdict all agree with ours (the pen's centre kept through the arrival tick; cleared in
the next tick with *move* current and *guard* queued; *guard* current in the tick after; the
baseline's single step; its destination cleared in its arrival tick).

## Formats and goldens

- Save format 107 to **108** (`save::FORMAT_VERSION`) and AI game format 71 to **72**
  (`game::GAME_FORMAT_VERSION`): the new state is not stored (`World::extra_locomotion` is
  `#[serde(skip)]` and empty between calls), but saves of 107 replay to other states, since units
  fall back to rest at the end of an update and scatters choose a walking target at once. Wire
  format 12 is unchanged.
- Goldens regenerated on the rebased tree (`GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p
  glasnost-sim-gate -- --include-ignored`). Eight move: `duel.txt`, `duel-diagonal.txt`,
  `duel-naval.txt`, `duel-m6c.txt`, `duel-m7a.txt`, `duel-m7b.txt`, `scripted.txt` and
  `skirmish.txt`. `combat.txt` holds.

## Lanes

Before: binaries built on bf6a19b (local `main` then). After: this branch's code on bf6a19b plus
the two requests; the branch was then rebased onto origin/main 0625dec, which adds only spec
commits since bf6a19b (`cargo xtask check` re-run there). Local `main` has since merged A4 round 9
(ffbf869), which this branch does not include, so no lane here has seen it. Local lane; install
values are not quoted.

- **Sweeps, seeds 1 to 16.**
  - M6c reference: smallest minimum 2 → **0**. Seeds 3, 4 and 14 now end with a result (house 1
    wins at 8356, 8640 and 11740 ticks); before, every seed ran the full 12000 with none. The zeros
    (support beams, radiation exits) are in those three shortened games.
  - M6c play fixes: 3 → **0**. Seeds 3 and 5 now end (house 1 wins at 6183, house 0 at 8838);
    the zeros are in seed 3's shortened game.
  - M7a reference 1 → 1; M7a play fixes 1 → 2; no early endings in either.
  - M7b reference and play fixes: drain minimum 3 → 3; no results.
- **The five M6c games that now end (traced, unit and building counts every 500 ticks).** In each,
  the house that wins was the one ahead at tick 12000 before, except play-fixes seed 3, which was
  mixed (more infantry for house 0, more buildings for house 1). The games part in the first fight
  (ticks 500 to 2000). From then on the losing house's base is broken early: at tick 4000 it has 4
  or 5 buildings, where before it had 11. Before, the samples show a few infantry units left in
  *move* with no destination (up to 5 for a house at a sample). After, there are none beyond one
  passing at a sample. That is what claim 22's check ends. It is consistent with the change, but
  the counts do not single out which of the two rules parts each game.
- **Arena, `--record all --threads 2`.**
  - M-0001: 12 → **11** of 96 failing; LTNK–E1 now passes.
  - Unchanged: M-0002 2 of 24, M-0003 1 of 35, M-0019 0 of 6, M-0020 1 of 25 (still YURI–SHK,
    arm S) and M-0021 8 of 50.
  - Six of our M-0001 games changed. Only one changed winner: LTNK–E1, row on side A, seed 2, which
    went from the row to the column. The other five changed only their end tick.
  - That game's first difference is at tick 106. An LTNK's detonation lands 6 and 5 leptons off
    because an E1 at cell (138, 85) stands at another point, one tick after a neighbour's
    destination is cleared there. From tick 133 the riflemen choose other destinations, and the
    game ends at the same tick with the column alive.
  - The arena trace does not print the extra locomotion, so it does not show which of the two
    rules moves that rifleman.
- **`cargo test --release -p glasnost-ai -- --include-ignored`** (run with `--no-fail-fast`).
  - Before: 33 binaries, all pass.
  - After: 32 pass and **one fails**, `skirmish::plays_to_the_last_side`: the game is not over
    within 40000 ticks.
  - End state: house 1 has only its LAB and one harvester resting in *guard*. House 0 has 3
    WALKER infantry resting in *guard* at home, 4 aircraft and its base. Both houses have 0
    credits.
  - No wave leaves, because of the scripted AI's guard rule (`scripted.rs`): the 2 units nearest
    home stay as a guard (`GUARD_MIN`). The third waits for a wave of at least 6, or at least 3
    after the wait.
  - None of the three walkers is held in *move* without a destination. The harvesters are
    vehicles, so claim 22's check (infantry only) does not reach them, and neither does the extra
    locomotion (walk locomotor only). So the stall is not in the rules built here.
  - It is an AI end-game, A4 round 9's area; nothing in the AI is changed here.
  - A4 round 9's merge on local main (ffbf869) changes `scripted::idle` only for a unit in *move*
    toward an object that is gone. That does not cover units resting in *guard* at home, so the
    merge is not expected to fix this case by itself.
- **AI quick lane, seeds 7 to 10, maps 0, 2 and 8** (`GLASNOST_AI_QUICK=1`).
  - All 36 games end before and after; the longest after is 104143 ticks.
  - 17 of 36 games change winner: 4 at seed 7, 4 at seed 8, 5 at seed 9 and 4 at seed 10.
  - House 0 wins 26 before and 25 after.
  - Whole AI games part early and widely after any change to unit movement, so these were not
    traced one by one.

## Seams

- **Damage scatter, S-0208 claim 15.** Requests A and B are still a seam in `combat::damage` (no
  hit sends them; intake 100). r46's new text needs nothing more from them once they are built:
  request A's search destination gives the walker its extra locomotion, the walking target it
  chooses is the scatter reference point (Terms, `scatter::answer`'s centre), so request B's
  centre is that target's cell, and claim 3's draw falls between the two requests' draws. The two-
  request test of W7 part j waits for the seam.
- **Under *enter* (Q-2701).** Claim 6 says a walker whose current or queued behaviour is *enter*
  keeps its stored path at a new destination, but not which walkers that covers or where the path
  leads. Not built: every walker drops its path, as before. (Answered by S-0208 r52; built in
  intake 110.)
- **Extra locomotion inside another (Q-2702).** A `// design:` interim (see Built). (Answered by
  S-0208 r52 as a departure of our engine's; the interim is retired in intake 110.)
- **The end-of-update check's other cases (S-0215 Open questions).** A unit in *move* holding a
  destination, and vehicles, get no check; team members under a team script's move step are not
  reachable here.
- **The check's place among the end-of-turn items.** After S-0417 claim 17's request and before
  S-0607 claim 20's restoration test: `// design: Q-2656`. S-0203 r53, brought in by the rebase,
  does not touch the end of an object's update, the late start or the resting turn, so the note
  stands.
