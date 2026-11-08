# S-0208 r52, S-0309 r120, S-0508 r13, S-0506 r34: claim ledger, M8 step A3 intake 110

- **S-0208** r46 to r52 (r52 at 1317d74), answering Q-2701 and Q-2702. Claim 6 gains *A stored
  path kept under enter*: a walker given a new destination, by an order or any other rule, keeps
  its stored path when at that moment its current or queued behaviour is *enter* and it is in
  contact with nothing; whether it has a walking target plays no part. An ordinary behaviour order
  breaks off the contact (a tether excepted), queues its behaviour and then assigns its destination
  (S-0213 T3), so an enter order to a walker under *move*, and any order to a walker under *enter*,
  keep the path; a scatter breaks off no contact. A kept path is followed (the walking target in its
  next cell, no search while steps remain); a destination of nothing leaves no path to follow. A
  standing walker with no walking target that keeps its path at a scatter has the extra locomotion
  too. *Requests sent from an extra locomotion* answers Q-2702: in the original the chain of extra
  locomotions can nest; in our engine (a departure, no switch) no unit is given an extra locomotion
  while a locomotion of its own, its update's or an extra one, is under way. New quirk
  `enter_keeps_stored_path` (default on, the original). Scenario W17 parts f to i are new.
- **S-0309** r116 to r120 (r120 at 366af11) and **S-0508** r9 to r13 (r13 at 0e3d6e5), answering
  Q-2712. S-0309 claim 30 is new: a holder that changes owner to another house, whatever the cause,
  drops its target at that moment, inside the change, whatever the target; no targeting-timer draw;
  for the causes listed (S-0508 claims 7 and 8, S-0509 claim 8, S-0011 claim 24, S-0601 claims 6 and
  9) the change also clears the destination and requests *guard* as S-0601 claim 9 gives for a
  release. S-0508 claim 7 places the occupation's owner change after the building's behaviour
  processing, its per-tick search and all of its firing in that turn, the animation-delayed shot
  included, and says our engine follows this. **Our code was last built against S-0508 r8, not r9**
  (the S-0508 ledgers name r8; nothing in code cites r9). The r8 to r9 change (9cfdd81) is
  scenario wording only, so the delta read here is r8 to r13, of which only r9 to r13 changes
  claims. S-0309 was last built at r116.
- **S-0506** r20 to r34 (r34 at 7d84ba5), answering Q-2711. Claim 14: for a destination whose
  in-the-air answer is false at the launch, the aim point is fixed at the launch, in the launch
  pass of the placement tick, and does not change whatever the destination does; a destination that
  leaves play later (death sequence, removal, going inside) changes neither the aim nor the phases;
  claim 24's timer runs on the tick number, so a lost turn does not shift a phase's ticks, and a
  phase's end is acted on at the first update at or after its timer runs out. Going inside is
  leaving play for claim 11's reset. Claims 7, 8, 12 and 24 are reworded to match.
- The one new switch: C7 in [c7-play-fixes.md](c7-play-fixes.md), *Intake 110* (not admitted). The
  layer file is unchanged.

Requests filed (my block Q-2746 to Q-2755): Q-2746 (a garrison order and the path kept under
*enter*), Q-2747 (owner changes outside claim 30's list: the vehicle-like takeover and the contacts
that follow a building), Q-2748 (a rocket launched at a target in the air), Q-2749 (the release
cell when the controller target has left play), Q-2750 (the extra locomotion of a standing walker
whose stored path a scatter drops). Q-2751 to Q-2755 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The keep rule** (`walk::keeps_stored_path`: the switch on, a walker with the walk state,
  current or queued *enter*, no contact). Read in `orders` at the assignment of an order's
  destination (after the contact is broken and the behaviour queued), where a walker that keeps
  its path is not cleared, and in `scatter::answer` before the request (a scatter breaks off no
  contact), where the path is kept instead of dropped. A destination of nothing clears the path in
  `walk::standing` (8be0462). The kept path is followed by the existing walk: at a locomotion with
  no walking target the walker takes the path's next cell.
- **The extra locomotion of a walker that keeps its path.** `walk::standing_free(w, id, keeps)`:
  no walking target, and no stored path or a kept one, read before the request. A walker whose
  path the scatter drops gets none (Q-2750 interim).
- **The departure** (`World::locomotion_under_way`, renamed from `extra_locomotion`; `#[serde(skip)]`,
  empty between calls). `walk::tick` pushes the unit for its update locomotion and
  `walk::extra_locomotion` for an extra one; an extra locomotion is not given to a unit on the
  stack. Q-2702's `// design:` interim is retired.
- **Claim 30's drop** (`econ::events::owner_changed`, at its top when the old house differs from
  the new): `spawn::clear_own_target` (claim 27's consequences: a spawner's conditional pending
  clear, the emitter let-go) and the burst counter reset (S-0301 claim 4). No draw; the targeting
  timer is not touched; no revert of an override. Every owner change in the simulation passes
  through this hook, the mobile ones included.
- **Claim 30's destination and *guard*** (`mind::release_orders`, now `pub(crate)`):
  `ownership::Cause::clears_orders` (Capture, Trigger, Occupation) calls it after the change in
  `ownership::change_by`; the trigger action's mobile branch (`triggers.rs`, action 14) calls it
  after its change. The mind-control release already called it; the takeover's
  `mind::guard_reset` (S-0215 claim 9) covers it. A building being sold ignores the request
  (S-0215 claim 7, inside `release_orders`'s guard request).
- **S-0508 claim 7's order** holds already: in `tick::object_turn` a building's
  `process::tail` (behaviour processing with the per-tick search) and
  `combat::behave::delayed_shot_turn` run before `econ::events::turn`, which calls
  `structures::garrison::turn`. Citations added at both places; no code moved.
- **The aim fixed at the launch** (`spawn::launch`): for a missile slot, when the controller target
  is a cell or an object whose in-the-air answer (`combat::weapon::in_air`) is false,
  `rocket::fix_at_launch` fixes the aim from `rocket::heading_point` and starts the flight
  (`rocket::start_flight`: pause, or tilt when the pause is 0; pitch Pᵢ; speed 0; `started` the
  launch tick). The destination and *move* are still given (claim 7).
- **The timer on the tick number** (`RocketState::started` replaces `elapsed`): the pause, tilt and
  raise read elapsed = tick − started; a phase's end sets `started` to that update's tick.
- **Going inside is leaving play for the controllers** (`spawn::target_leaves_play`, called from
  `objects::take_inside`; `spawn::removing` now shares `settle_waiting` and `target_gone` with it):
  waiting clear requests settled, a pending target that goes inside cleared, and a controller whose
  target goes inside reset when no pending target waits.

## Claims

Tests in `crates/deterministic/sim/tests/claims_s0208_r52.rs` (S-0208),
`claims_s0309_r120.rs` (S-0309, S-0508) and `claims_s0506_r34.rs` (S-0506), unless named
otherwise. Synthetic data only.

| Claim | Status | Tests or reason |
| --- | --- | --- |
| S-0208 claim 6, an enter order to a walker under *move* keeps its path (W17 f) | tested | `w17_f_enter_order_during_a_step_keeps_the_old_line`, `w17_f_switch_off_drops_the_path` |
| S-0208 claim 6, any order to a walker under *enter* keeps its path (W17 g) | tested | `w17_g_move_order_to_a_walker_under_enter_keeps_its_path` (both switch positions) |
| S-0208 claim 6, a walker between steps keeps it too (W17 h) | tested | `w17_h_enter_order_between_steps_keeps_the_path` (both positions) |
| S-0208 claim 6, the condition: current or queued *enter*, no contact | tested | `the_keep_condition` |
| S-0208 claim 6, a tethered walker drops its path; the unloading-kind link broken by the order | partly | follows from the contact part of the condition, read after the order breaks the contact (`orders`); no test times an order within a tether |
| S-0208 claim 6, a scatter breaks off no contact: in contact, the path is dropped | tested | `a_scatter_drops_the_path_in_contact_or_with_the_switch_off` |
| S-0208 claim 6, a kept path is followed in the extra locomotion | tested | `a_kept_path_is_followed_in_the_extra_locomotion` |
| S-0208 claim 6, a destination of nothing leaves no path | tested | `no_destination_no_path_to_follow` |
| S-0208 claim 6, requests from an extra locomotion fall in the scatter tick (W17 i) | tested | `w17_i_requests_from_the_extra_locomotion_fall_in_the_scatter_tick` (an invented close-enough distance, so the runs the scenario keeps) |
| S-0208 claim 6, the departure: no extra locomotion while one of the unit's own is under way | tested | `no_extra_locomotion_while_one_of_its_own_is_under_way`; `claims_s0208_r46.rs` `q2702_extra_locomotions_in_a_crowd_end`, `q2702_no_extra_locomotion_inside_its_own` (recited) |
| S-0208 Edge cases, `enter_keeps_stored_path` off | tested | `w17_f_switch_off_drops_the_path`, `w17_g_…`, `w17_h_…`, `the_keep_condition`, `a_scatter_drops_…` |
| Q-2750 interim, a standing walker whose path a scatter drops | tested | `a_scatter_drops_the_path_in_contact_or_with_the_switch_off` (no walking target until its next locomotion) |
| S-0208 claim 6, a garrison order under *enter* (Q-2746) | seam; tested since intake 113 | which orders count as an *enter* for a building to garrison is not given; built as any *enter*. Answered by S-0208 r53: a garrison order gives *capture*, which is no *enter* (`claims_s0208_r53.rs`, [intake113.md](intake113.md)) |
| S-0309 claim 30, the drop inside the change, no draw, the timer counts on (A19 a, c) | tested | `a_captured_building_drops_its_target_with_no_draw` |
| S-0309 claim 30, whatever the target; no change, no drop | tested | `the_drop_is_whatever_the_target_and_none_without_a_change` |
| S-0309 claim 30, the listed causes: destination and *guard* | tested | `listed_causes_request_guard_the_takeover_drops_only`, `a_trigger_owner_change_drops_target_and_destination` (A19 b) |
| S-0309 claim 30, a mind-control change (A19 d) | tested | `a_mind_control_owner_change_drops_the_target_inside_the_change` |
| S-0309 claim 30, any other owner change: drop only | tested | `any_other_owner_change_drops_the_target_only`; the vehicle-like takeover and the contacts that follow a building are Q-2747 interims |
| S-0309 claim 30, the pending animation-delayed shot | tested | `a_pending_delayed_shot_ends_with_the_drop` (pending across the change, no shot at the delay's end) |
| S-0309 claim 30, an aircraft holder | partly | the drop is in the common hook, which every owner change of an aircraft passes through; no aircraft test |
| S-0309 claim 30, the simple-deployer exception in *unload* | partly | inside `mind::release_orders`, as S-0601 claim 9 built it (intake of S-0601); not retested here |
| S-0508 claim 7, the change after the per-tick search and the firing (G20) | tested | `an_occupied_civilian_building_drops_its_target_at_the_change` (play note 49's shape: no shot at its own infantry over 300 ticks) |
| S-0508 claim 7, the delayed shot after the per-tick search | no change | `tick::object_turn` already runs it after `process::tail`; cited |
| S-0506 claim 14, the aim and flight fixed at the launch | tested | `the_aim_and_the_flight_are_fixed_at_the_launch`; `claims_s0506.rs` (unchanged expectations) |
| S-0506 claim 14, a destination that leaves play after the launch (R25) | tested | `a_destination_removed_in_the_launch_tick_does_not_stop_the_flight` (the missile makes no update before the removal), `a_destination_that_goes_inside_resets_the_controller_and_the_missile_flies_on` |
| S-0506 claim 11 through claim 14, no reset with a pending target waiting | tested | `no_reset_when_a_pending_target_waits` |
| S-0506 claims 14 and 24, the timer on the tick number | tested | `the_flight_timer_runs_on_the_tick_number`, `uninterrupted_boundaries_from_the_launch_tick` |
| S-0506 claim 14, a destination in the air at the launch (Q-2748) | tested; replaced at intake 113 | `q2748_a_flight_not_fixed_at_the_launch_starts_at_the_first_update` (interim: r20's rule). S-0506 r37 gives no destination and no flight until a destination not in the air; the test is gone, see `claims_s0506_r37.rs` and [intake113.md](intake113.md) |
| S-0506 claim 12, the release cell when the controller target has left play (Q-2749) | seam; tested since intake 113 | interim: the cell under the object's position while it exists; not tested on its own. Answered by S-0506 r37: the target is cleared before the reset, so the F8 cell (`claims_s0506_r37.rs`) |
| S-0506 claim 14, a pending target that goes inside | partly | cleared as at a loss notification (claim 25) in `spawn::target_leaves_play`; our reading of "leaves play" for the pending target, not tested on its own |

Older tests changed by these revisions, each with the claim it now cites:

- `claims_s0208_r46.rs`: `a_walker_with_a_walking_target_gets_no_extra_locomotion` cites r52
  (the r46 clause on a walker holding a stored path is gone); `extra_locomotion_only_for_the_walk_locomotor`
  and the two `q2702_*` tests use `World::locomotion_under_way` and cite r52's departure instead of
  the Q-2702 interim.
- `claims_s0417_r64.rs` `claim_17_one_draw_then_refusals`: the same rename and citation.
- `claims_s0506.rs` `s0312_claim_16_interception_keeps_timer_and_position`: compares `started`
  instead of `elapsed`.

### Cause mapping (S-0309 claim 30)

| Cause | Spec | Our path | Target dropped | Destination and *guard* |
| --- | --- | --- | --- | --- |
| Occupation of a civilian garrison building | S-0508 claim 7 | `garrison::turn` → `ownership::change` (`Cause::Occupation`) | yes, in `events::owner_changed` | yes, `release_orders` |
| Return of an abandoned building | S-0508 claim 8 | the same, `Cause::Occupation` | yes | yes |
| Engineer capture | S-0509 claim 8 | `engineer` → `ownership::change_by` (`Cause::Capture`) | yes | yes |
| Trigger change-owner, a building | S-0011 claim 24 | `triggers` action 14 → `ownership::change` (`Cause::Trigger`) | yes | yes |
| Trigger change-owner, a unit | S-0011 claim 24 | `triggers` action 14, mobile branch → `events::owner_changed` | yes | yes, `release_orders` after it |
| Mind-control takeover | S-0601 claim 6 | `mind::take` → `ownership::change_by` (`Cause::MindControl`) or `events::owner_changed` | yes, inside the change | yes, by the takeover's guard reset (S-0215 claim 9) |
| Mind-control release | S-0601 claim 9 | `mind` release → the same | yes | yes, `release_orders` (as before) |
| Vehicle-like takeover | S-0509 claim 17 | `engineer` → `ownership::change_by` (`Cause::Takeover`) | yes | no: Q-2747 interim |
| Contacts that change owner with a building | S-0509 claim 14 | `ownership::contacts` → `events::owner_changed` | yes | no: Q-2747 interim |
| A slave transferred at its master's death | S-0312 claims 30 and 31, S-0408 claim 10a | `slaves` → `events::owner_changed` | yes (already cleared by `free_to_guard`) | its own rule (already cleared, *guard* queued) |

## Formats and goldens

- Save format **111** and AI game format **76**, one past main 841f16d (intake 107 took 109 and
  74, intake 108 took 110 and 75). The rocket's flight timer is now a start tick
  (`RocketState::started` in place of `elapsed`), the extra-locomotion stack is renamed and stays
  unsaved, and saved games replay differently from the first owner change, kept path or launch.
- **`M6C_SEED` 1 to 2.** The gate's full run of duel-m6c asserts that every counter is above zero.
  With S-0309 claim 30, seed 1's first engineer capture of a tech building (tick 48) requests
  *guard* for the building (`Cause::Capture` → `mind::release_orders`), and the game diverges from
  there: house 1 wins at tick 9605 and no radiation exit is ordered. Bisected by commit: with
  S-0208 alone seed 1 still plays all 12 000 ticks (smallest counter 3); with claim 30 added it
  ends at 9605. Flipping `enter_keeps_stored_path` does not change it. Seed 2 plays all its ticks
  with smallest counter 4 both before (main 01c5688) and after; it is the next seed, as at M7a's
  move to seed 2. `docs/development.md` names the move.
- Goldens regenerated on 049bb49 (seven of nine move: duel, duel-diagonal, duel-naval, duel-m6c,
  duel-m7a, duel-m7b, skirmish; combat and scripted do not). 049bb49 to 841f16d changes only
  `spec/` and docs, so the code tree is the one regenerated. `cargo xtask gate` passes native and
  wasm32.

## Lanes

Bases: **before** is main 01c5688 (`bin/before`), unless named. **After** is this branch on
049bb49 or 841f16d (the same code) unless named. The arena was also played on main e0695d3 and on
this branch on e0695d3, which gives the cleanest before/after pair for the arena.

- **Sweeps, seeds 1 to 16** (smallest minimum; before → after):
  - M6c reference 0 → 0 (seeds at 0: 3, 4 → 1, 11); play fixes 0 → 0 (seed 3 → 11).
  - M7a reference 1 → 1; play fixes 2 → 2.
  - M7b smallest drain minimum 3 → 3 in both positions. Reference: drain orders min 3 median 5 →
    min 3 median 4; credits drained min 1560 median 2080 → min 1520 median 2080. Play fixes:
    orders min 3 median 4 → the same; credits min 1480 median 2000 → min 1480 median 1960.
- **AI suite** (`cargo test --release -p glasnost-ai -- --include-ignored`): 263 of 263 before
  and after, `skirmish::plays_to_the_last_side` included.
- **Arena** (`--record all --threads 2`), failing rows on 01c5688 → e0695d3 → this branch on
  e0695d3: M-0001 11 → 11 → **10** of 96; M-0002 2 → 2 → 2 of 24; M-0003 1 → 1 → 1 of 35; M-0019
  0 → 0 → 0 of 6; M-0020 1 → 1 → 1 of 25 (YURI against SHK); M-0021 8 → 8 → 8 of 50. Against
  e0695d3, intake 110 changes 12 of our 1152 M-0001 games and 21 of 288 M-0002 games, none of
  M-0003's. The changed M-0001 games are MIND against E1 and E2 (6, mind-control owner changes:
  claim 30's drop inside the takeover), TELE and CAOS against V3 (5, the V3's missiles: S-0506's
  aim fixed at the launch) and one CAOS against E2. CAOS against V3 now passes the win test
  (ours 1/0/11 → 2/1/9 of 12, p 0.0094 → 0.035). The changed M-0002 games are infantry-only rows
  (INIT, BRUTE and YURI against E1, E2, GGI and SHK). Mind control explains YURI's; INIT's,
  BRUTE's and the rest have no owner change and no missile, which leaves S-0208's crowd
  locomotion (the kept path and no extra locomotion inside its own). M-0002's verdicts are
  unchanged: INIT against E2 changes three winners but still fails its side outcome.
- **Quick lane** (`GLASNOST_AI_QUICK=1 ai-sides 120000 <seed> 0 2 8`, seeds 7 to 10, 36 games),
  played on 01c5688, on main 841f16d and on this branch: every game ends with a winner on all
  three. Against 01c5688, 3 of 36 winners change (seed 7 map 0 sides 1 and 2, seed 9 map 0 sides
  0 and 2, seed 10 map 8 sides 1 and 2). That pair includes intakes 107 and 108, so the pair to
  read is main 841f16d against this branch: 4 of 36 winners change, all between sides 1 and 2
  (seed 7 maps 0 and 2, seed 10 maps 0 and 8), and 33 of 36 games change length. Each changed
  winner was replayed on this branch's own commits (b79ce47, S-0208 only; e5ac2f4, adding S-0309
  claim 30), which places it:
  - **Seed 10 map 0** (house 0 → house 1): S-0309 claim 30. With S-0208 alone the game is main's
    to the tick (67 544 ticks, house 0). The two commits part at an owner change: house 1
    occupies a civilian building between ticks 24 350 and 24 375. Without the drop (b79ce47) the
    building comes to house 1 still holding its target, house 1's own neighbouring civilian
    building, and by tick 24 400 it is firing at it, the neighbour losing health shot after shot.
    With the drop (e5ac2f4) it holds no target at 24 375, stays under *guard*, and the neighbour
    stays at full health. Play note 49's shape; it follows from claim 30 and S-0508 claim 7.
  - **Seed 7 map 2** (house 0 → house 1): S-0309 claim 30. S-0208 alone changes the length
    (56 706 → 71 645 ticks) but not the winner; claim 30 flips it (52 349, house 1). Not traced
    to the first difference.
  - **Seed 7 map 0** (house 0 → house 1) and **seed 10 map 8** (house 1 → house 0): placed on the
    rocket commit c040e98 (S-0506 r34); the mechanism is not established. With S-0208 and claim
    30 both games keep main's winner (seed 7 map 0: S-0208 alone moves its length 38 959 →
    39 541; seed 10 map 8 ends at 46 880, house 1), and the rocket commit flips both. That commit
    holds claim 14's aim fixed in the launch pass, the timer on the tick number, going inside as
    leaving play, and a refactor of `spawn::removing`, and the replay does not tell them apart.
    In seed 7 map 0 the first difference the 50-tick dumps show is at tick 22 550 (an ore truck
    takes another ore cell), some 400 ticks after a V3 launched a rocket at an enemy infantry that
    moved and died before the rocket arrived; the rocket's cells match in both runs, so the link
    is not seen.

## M-0001, M-0020 and M-0021

- **M-0001**: every row keeps its verdict but CAOS against V3, which now passes (above). The
  other ten failing rows are main's.
- **M-0020**: every row keeps its verdict. 24 of 25 checked rows hold, and part 2's YURI against
  SHK, arm S, still fails (ours 11/1/0 against the record's 12/0/0, as on main). What moves is the
  report's diagnostics, not a verdict: the drones' end (tick 351 → 373), a few end ticks and
  first captures in the mind-control arms. Mind control's owner changes now drop the target inside
  the change (claim 30).
- **M-0021**: unchanged: 8 of 50 rows fail, the same rows, and the oracle report is identical to
  main's. These answers do not touch it.
- **Play note 49** (a veteran goes into a civilian building; lane
  `r8_render_lane::note_49_a_veteran_goes_into_a_building` on this branch, local only): once the
  occupant is inside (tick 415 of the lane), the building has **no experience of its own** and
  draws no badge. Round 9 found it at the cap by then, from its own house's infantry killed by a
  target kept across the occupation. Claim 30 now drops that target at the change. Later in the
  lane the occupants kill the three enemies it sends, and the building then has experience
  (status items 27 → 28), from enemy kills. That is S-0311's rule for kills, which these answers
  leave alone. Unchanged and not from this intake: one of the two elite occupiers drops its order
  within 50 ticks and stays outside, badge showing. Round 8 saw this too and passed it on with
  note 53.

## Seams

- **`econ::assign_destination` drops no path** for any walker (pre-existing): the destinations it
  gives (a contact told to leave, S-0509 claim 14, and the economy's other moves) neither drop nor
  keep a path by r52's rule. Only the order and scatter assignments read it.
- **Claim 30's drop is a plain clear.** An override or a suspended target with a revert is not
  reverted at the change (no claim gives a revert there); S-0215 claim 10's revert is not invoked.
- **Q-2746** (a garrison order under *enter*), **Q-2747** (owner changes outside claim 30's list),
  **Q-2748** (a rocket at a target in the air), **Q-2749** (the release cell) and **Q-2750** (a
  standing walker whose path a scatter drops) are `// design:` interims.
