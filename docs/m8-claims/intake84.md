# S-0513 r91, S-0514 r16, S-0306 r65 and S-0209 r45: claim ledger, M8 step A3 intake 84

S-0513 r89 to r91 answers **Q-2376** (filed by [intake 81](intake81.md)) in claim 39a, *The
firer*: a building firer, with and without the SAM flag, and an infantry or vehicle firer beyond
its weapon's range of the old position, each whose temporal beam holds a warp member that waits
to land, with the matching Changes for S-0309 claim 27. S-0514 r14 to r16 answers **Q-2377** in
claim 18, *Order against the erasure update*: within one turn of an object being erased, an
infantry unit sparkles before the turn's erasure update and every other kind after it, so a turn
whose update removes the object still sparkles; S-0513 r91 carries this into claim 39a for a
warp member. Both replace intake 81's interim rules. S-0306 r64 to r65 (commit 7e9f0ff, claim 17)
says how long a warp member stays in the ground render layer after its arrival hold, and S-0209
r44 to r45 (commit e7f04c9, claim 13) that units with drop-pod movement and
tunnelling units while digging are in no cell layer, so a warp never takes them. Two seams that [intake 83](intake83.md) (S-0102 r41) left in S-0513
claims 34 and 36 for a jumpjet member are checked here too, and the drawing's warp row is kept
apart from the sim's new kept ground layer. Requests filed: **Q-2416**,
**Q-2417**, **Q-2418** and **Q-2419**.

## Built

- **The sparkle's order against the erasure update** (S-0514 claim 18). `temporal::turn_sparkle`
  is replaced by a pair around the turn's erasure update: `temporal::sparkle_before_update`
  makes an infantry unit's sparkle at once and, for any other kind, records whether it is due
  (being erased or phasing in), with its kind and position; `temporal::sparkle_after_update`
  then makes a vehicle's, aircraft's or building's sparkle from its state after the update, or,
  when the update removed it, from the recorded state and last position. So a removing turn
  sparkles in every kind, after the removal and the warp-away animation for every kind but
  infantry, and a releasing update (claim 11) leaves a vehicle, aircraft or building that is not
  phasing in with no sparkle. `tick.rs` calls the pair around `temporal::member_update` for a
  warp member (both before the warp step, S-0513 claim 39a) and around `temporal::turn` for any
  other object.
- **A building firer** (S-0513 claim 39a, Q-2376). `held::firing_target` now keeps a beam-held
  departed member for a building firer too, so `behave::building_process`'s firing check sees
  the member and answers *rearming* (`check::target_exists`, intake 81), and the building takes
  S-0215 claim 32's *rearming* row: no shot, delay 2, a rotation towards the old position, no
  *guard*. A building with the SAM flag (`behave::sam_attack`) already reads the target's old
  position and height, which the departure keeps (`warp::lift_off` leaves `pos` as it is), so it
  drops a member at height 0 and starts *guard*, keeps an aircraft member above 0, and keeps any
  member under the power row. Re-cited, no code change there.
- **A firer beyond range** (S-0513 claim 39a, Q-2376). Intake 81's interim rule was already the
  answer: the firing check answers *rearming* whatever the range, and the approach step reads the
  old position. Re-cited, no code change.
- **The ground render layer after the hold** (S-0306 claim 17, r65). `warp::in_ground_layer`
  now also holds for a former member that is an aircraft whose type cannot land, from the end of
  its hold while it flies, until a crash: `warp::keep_ground_filing` sets the new
  `air::AirState::warp_filed` when the hold ends, and `combat::effects::layer_refile` clears it.
  Such a type is re-filed by no landing or take-off (S-0102 claim 4), so only a crash fall
  clears it. Every other former member stands at no height after its hold, so its own rows
  (`temporal::ground_layer`) give the ground layer until its movement raises it, as r65 says.
  The readers are r65's: S-0306 claim 20 (c) and S-0514 claim 1. `warp_filed` is hashed: save
  format 94, AI game format 57.
- **No cell layer for drop-pod and digging units** (S-0209 claim 13, r45). Our engine builds
  neither movement (`rules::Locomotor::None`), and the drop-pod stand-in of S-0607 ends at once,
  so every such unit stands listed in its cell and the warp may take it. Holds trivially;
  `warp::examine` cites it.

## Claims

| Claim part | Status | Where, and why |
|---|---|---|
| S-0514 r16 claim 18: a turn whose erasure update removes the object sparkles in a sparkle tick, every kind, placed from its last position | tested | `claims_s0514_r16::e1_a_turn_whose_erasure_update_removes_the_target_sparkles_in_every_kind` (an infantry unit, a vehicle, a landed aircraft and a building with no damage-fire positions) |
| 18: the draw order, infantry's sparkle before the update's draws, the others' after them | tested | `e2_the_sparkles_draw_comes_before_the_updates_for_infantry_and_after_them_for_other_kinds` (ranged creation draws on the sparkle and the warp-away animation) |
| 18: a release in the update leaves a vehicle, aircraft or building with no sparkle, an infantry unit with one | tested | `e3_a_release_in_the_update_leaves_no_sparkle_but_at_an_infantry_target` (an open-topped transport's leader beyond the warp distance) |
| 18: the state after the update decides for a vehicle (a joiner takes over) | tested | `a_joiner_taking_over_in_the_update_keeps_the_vehicles_sparkle` |
| 18: a building with damage-fire positions (E11 part e3's second building) | seam | damage-fire positions are not imported; such a building sparkles at its own position |
| S-0513 r91 claim 39a Sparkles: a warp member removed by its erasure update in a sparkle tick sparkles in its kind's order | tested | `claims_s0513_r89::r91_claim39a_a_member_removed_by_its_erasure_update_in_a_sparkle_tick_sparkles_in_its_kinds_order` (an infantry and a vehicle member in the leaving stage) |
| 39a Sparkles: an aircraft member's removing turn | partly | the same path as a vehicle member; no aircraft member test |
| 39a Sparkles: the r89 rules under the new order | tested | intake 81's l6, l6c and draw-order tests, unchanged and passing |
| 39a The firer: a building firer without the SAM flag keeps the member, *rearming* row, no shot, facing unchanged, processings every other tick, beam attached after the appearance (SW12 part l7) | tested | `r91_claim39a_l7_a_building_firer_keeps_a_member_that_waits_to_land_taking_the_rearming_row` |
| 39a The firer: a building's outranking *illegal* or *impossible* drops the member and starts *guard* | partly | S-0215 claim 32's row for those answers, as before the departure; no building test with an outranking answer |
| 39a The firer: a SAM-flag building drops a member at world height 0 and starts *guard* | tested | `r91_claim39a_a_sam_building_firer_drops_a_ground_member_and_starts_guard` |
| 39a The firer: a SAM-flag building keeps any member under the power row | request | **Q-2418** question 2. Its own rows keep the member (`r91_claim39a_a_sam_building_firer_short_of_power_keeps_the_member_in_its_own_rows`), but the building is then not in operation, so its firing check answers S-0405 T1's *impossible*, which outranks *rearming*, and S-0309 claim 29's check drops the member at the next check tick, as for a target on the map. Interim: that drop |
| 39a The firer: a SAM-flag building keeps an aircraft member above world height 0 | partly | `sam_attack` reads the kept old position and height; no test puts an aircraft member above 0 (a landed aircraft on a raised cell or a deck) |
| 39a The firer: an infantry or vehicle firer beyond range keeps the member, *rearming* whatever the range, approaches the old position, lets the beam go at its first new cell | tested | `r91_claim39a_a_vehicle_firer_beyond_range_keeps_the_member_approaches_its_old_position_and_lets_go_at_a_new_cell` |
| 39a The firer: a deployed infantry firer of a human owner beyond range takes no approach step, neither moves nor turns | tested | `r91_claim39a_a_deployed_infantry_firer_beyond_range_takes_no_approach_step_and_stays_put` |
| 39a The firer: that deployed firer keeps the member and its beam | request | **Q-2418** question 1. S-0215 claim 23's deployed-firing rule (S-0517 claim 16) drops an out-of-range target in the first *attack* processing, as for a target on the map. Interim: that drop (the same test) |
| 39a The firer: a firer of type speed 0 beyond range keeps the member and its beam while it does not move | tested | `r91_claim39a_a_zero_speed_firer_beyond_range_keeps_the_member_and_its_beam_while_it_does_not_move` |
| 39a The firer: an aircraft firer | seam | "the aircraft specs'"; no aircraft spec says it yet. Kept at turn start (`held::turn_start_drop`), no target in its firing step (`held::firing_target`), our own rule as before |
| 39a The firer: a member protected or cloaked at its departure | request | **Q-2416**; interim as intake 81 (the cloak state kept off the map, protection running by its tick window, so the outranking answer drops the member at the check) |
| 39a A lock while the member waits to land: what it does to the warp's own steps | seam | still an open question of S-0306 claim 17 and S-0513 claim 39a; built as before |
| S-0309 claim 27 (r91 Changes): a building firer and a firer beyond range | tested | as *The firer* above; S-0309 itself has not yet taken r91's Changes, so the code cites S-0513 r91 and the Changes |
| S-0306 r65 claim 17: a member is in the ground render layer from the launch until its own movement applies again after its hold | tested | `claims_s0306_r62::r65_claim17_a_ground_member_is_in_the_ground_layer_by_its_own_rows_after_the_hold`; r62's member tests for the span to the hold's end |
| 17: an aircraft member whose type can land leaves the layer when its movement raises it | tested | `r65_claim17_an_aircraft_member_whose_type_can_land_leaves_the_ground_layer_when_raised` (threshold 128, not erasable) |
| 17: an aircraft member whose type cannot land keeps the ground layer while it flies | tested | `r65_claim17_an_aircraft_member_whose_type_cannot_land_keeps_the_ground_layer_in_flight` (threshold 42, erasable) |
| 17: until a crash | tested | `r65_claim17_a_crash_ends_the_kept_ground_layer` |
| 17: the other readers of the ground render layer (S-0309 claims 7 and 9, S-0607 claim 3) for such an aircraft | request | **Q-2417**; interim: they read its height as before |
| 17: a member moving from one bridge deck to another: when it leaves the layer | seam | not established (S-0306 Open questions); the height test after the hold, as before |
| 17: the render-list filing itself and S-0102 claim 4's re-file moment | client, built | S-0102 r41, intake 83. After the rebase onto intake 83, the drawing's warp row (`layers::evaluate_at`, `listed` and the sweep's skip) reads the new `warp::in_warp_row` (the member state alone) instead of `warp::in_ground_layer`. With the new kept layer, `in_ground_layer` would have stopped the sweep of `quirks.layer_change_on_events_only` off from moving a flying non-landable aircraft member after its hold, which S-0102 Edge cases say it does once the warp row has ended. With the switch on, the drawing keeps such a member in the ground list by claim 4's own rule (no take-off re-file for its type), as S-0306 r65 says. Test: `layers::tests::claim4_a_non_landable_aircraft_member_keeps_the_ground_list_while_it_flies` now sets the kept layer as a real warp's hold end does (both switch positions) |
| 17 and `quirks.warp_deck_departure_airborne` | unchanged | r65 changes none of its text; its tests (r62 run j, run i) pass |
| S-0513 claim 34 against S-0102 r41 (intake 83's seam): a unit with jumpjet movement listed in a source cell while above its surface (*r* < 208, S-0209 claim 13) | request | **Q-2419**: r41 names it a possible member, but claim 34 leaves "a jumpjet in flight" alone without saying when a jumpjet is in flight. Interim: `warp::in_air` unchanged, any height above the surface or deck is in the air (`claims_s0513_warp::intake84_claim34_a_listed_jumpjet_above_its_surface_is_left_alone_q2419_interim`); a grounded jumpjet joins (`intake84_claim34_a_grounded_jumpjet_listed_in_the_block_joins_the_warp`) |
| S-0513 claim 36 with S-0102 r41 (intake 83's seam): a jumpjet member landing on a high-bridge deck from the ground has its own on-deck state set | tested | `warp::place` sets the deck layer and `objects::place` calls `jump::placed`, which sets `on_deck` from the layer: `claims_s0513_warp::intake84_claim36_a_jumpjet_member_landing_on_a_deck_from_the_ground_stands_on_it_on_deck` (deck layer, on-deck state, height 0 above the deck, listed on the deck) |
| S-0209 r45 claim 13: drop-pod and digging tunnelling units are in no cell layer, so a warp never takes them | trivially | neither movement is built (`rules::Locomotor::None`, the drop-pod stand-in ends at once); `warp::examine` cites it |
| S-0209 r45: drop-pod and tunnelling movement | seam | when either is built, its no-cell listing comes with it |

## Quirks

No new switch: S-0514 r16 and S-0513 r91 copy the orders with no switch, and S-0306 r65 adds
none. `quirks.warp_departure_keeps_beam` and `quirks.phase_in_beam_spares_infantry` are
unchanged in name, default and meaning. With the first off the lock at the warp's launch is
refused, so no firer holds a departed member and r91's *The firer* rows do not arise; the second
changes no sparkle (it is about cut-short turns). Their C7 verdicts stand
([c7-play-fixes.md](c7-play-fixes.md)).

## Lanes

Main against the branch. The base binaries and tests were built from a worktree at 0ff0bea,
whose code is the same as e2239c1's: the commits between them change only docs and spec files. All
lane outputs are under `local-lane/m8/i84/` (git-ignored).

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1 to 16 | | identical, line for line |
| M7a sweep, seeds 1 to 16 | | identical, line for line |
| M7b sweep, seeds 1 to 16 | | identical, line for line |
| Arena, `arena --record all --threads 8` | 11 of 155 pairings fail | byte-identical CSVs and oracle reports: the same 11 (LTNK–E1, TELE–APOC, TELE–V3, CAOS–APOC, CAOS–TTNK, CAOS–E1, CAOS–E2, DISK–FV, INIT–E2, ROBO–HTK, MIND–E1), none changed |
| `glasnost-ai --include-ignored` | 246 passed, 0 failed | 246 passed, 0 failed |
| AI quick lane, seeds 7 to 10, maps 0, 2 and 8, sides 0–1, 0–2, 1–2 (36 games) | every game ends | the same tick counts and winners in all 36; output identical apart from timings |
| `glasnost-sim --include-ignored` (release) | not run; the brief's known failure is `pathing_c7p4::diag` | 5195 passed, 1 failed: `pathing_c7p4::diag` only |

The goldens move (the new hashed `warp_filed` byte), but no lane output does. The likely
reason, which I did not trace, is that no aircraft whose type cannot land is warped while
in flight in these games.
