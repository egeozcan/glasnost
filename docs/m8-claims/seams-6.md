# M8 seam sweep 6: rows marked partly, untested or seam in intakes 96 to 111

Brief: `.claude/worktrees/M8-SEAMS-6-BRIEF.md`. Method and row form follow [seams-4.md](seams-4.md).
Rows read: every row of `intake96.md` to `intake111.md` marked partly, not tested, untested, by
construction, by inspection, built with no test, not built or seam. Each was first checked for a
later close (a later intake, a spec revision, its `requests:` field, `git log --grep`).

**Counts.** 62 rows read: 15 closed earlier, 7 blocked by an open request, 17 tested (one of them
with a code change, under *Built*), 23 left. A row restated in two intakes counts twice. Intakes
102 and 103 hold only rows restated from intake 96 (claims 31 and 45) and S-0212 claim 26; their
other marker words are the untested-touchdown switch's name.

**Result.** 17 rows tested through the real path, in 19 new tests. One test showed the code
differing from an approved claim: S-0015 claim 1, the string table resolved through the whole
search order (fixed, app and VFS code). No simulation change: no hashed or replayed state, no
format bump (save 112, AI game 77, wire 12), goldens and seed sweeps unaffected. No spec request
filed (Q-2766 to Q-2775 unused). Part 2 (the rocket commit c040e98 split) is below.

## Built

| Item | What changed |
|---|---|
| S-0015 claim 1, the table's source (intake 109, partly) | `strings::read` resolved `ra2md.csf` with `Vfs::resolve`, which searches the whole order (S-0005 claim 9), so a copy only in a later archive such as `ra2md.mix` was read. Claim 1 names the language archives. `Vfs::resolve_language_only` (`crates/vfs/src/game.rs`) keeps the loose-file layer when `mod_layer` is on and then searches only the leading language ranks; `strings::read` uses it. On the local install nothing changes: `r8_render_lane::i109_*` before (9d2aa4c) and after read a table of the same size, give the unit's and the building's tips the same box, line count and length, never the placeholder, and write byte-identical screens. |

## Tested

| # | Intake | Spec | Claim or row | State before | Test and result |
|---|---|---|---|---|---|
| 1 | 98 | S-0301 r73 | claim 9, the other T1 rows (drained, selling, under construction) above the held temporal rearming | partly | `seams6_the_other_t1_rows_rank_above_the_held_temporal_rearming` (`claims_s0301_r73.rs`): each answers *illegal*. Code agrees. |
| 2 | 99 (53) | S-0215 r118, S-0107 r40 | a change of target by the target's leaving play, at once | partly | `seams6_the_target_leaving_play_ends_the_state_at_once` (`claims_s0601.rs`). Code agrees. |
| 3 | 99 (55) | S-0215 r118, S-0107 r40 | every change asks, not forced, for *deployed*, *prone* or *stand* | partly | `seams6_the_release_request_is_not_forced` (`claims_s0601.rs`), all three stances. Code agrees. |
| 4 | 99 (56) | S-0215 r118, S-0309 claim 23 | the attack-move discard and put-back are no change | partly | `seams6_the_attack_move_discard_and_put_back_are_no_change` (`claims_s0215_r118.rs`), through `acquire::per_tick`: the state and *fire standing* stay. Code agrees. |
| 5 | 100 | S-0302 | claim 12, a berserk hit has none of the reactions (a disguise kept) | partly | `seams6_a_berserk_hit_keeps_the_disguise` (`claims_s0302_r41.rs`), with an ordinary hit as control (disguise lost, lockout set). Code agrees. |
| 6 | 107 | S-0203 r53 | 13 (f), three sets of effects for a vehicle removed earlier in the tick | partly | `o18_a_and_g_a_vehicle_removed_earlier_in_the_tick_has_three_sets_of_effects` (`claims_s0203_r53.rs`): two leftover hits on the removed vehicle and one each on the others with the switch on, none with it off. The crate count allows S-0520 claim 3 to refuse a third crate whose nearby cell holds an overlay. Code agrees. |
| 7 | 107 | S-0107 r40, S-0601 | claim 11, guard reset, takeover, release, reverts, transport clears | partly | `seams6_a_takeover_ends_the_attack_state_and_asks`, `seams6_an_open_topped_transports_clear_and_hand_down_end_the_state` (`claims_s0601.rs`); `seams6_a_cloak_drop_or_revert_ends_an_infantry_holders_attack_state` (`claims_s0516_machine.rs`), drop and revert. Code agrees. |
| 8 | 108 | S-0501 | claim 7, the aiming request above the upper threshold | partly | `seams6_a_survivor_above_the_upper_threshold_makes_the_aiming_request` (`claims_s0501_r146.rs`): only that band turns the turret target to the body's. Code agrees. |
| 9 | 109 | S-0015 | claim 1, which file | partly | `seams6_the_table_is_resolved_among_the_language_archives_only` (`strings.rs`): a copy in a language archive reads; a copy only in `ra2md.mix` gives an error, though `resolve` finds it. **Code differed: fixed** (*Built*). |
| 10 | 110 | S-0208 | claim 6, a tethered walker drops its path unless the link is an unloading kind | partly | `seams6_a_tethered_walker_drops_its_path_unless_the_link_is_an_unloading_kind` (`claims_s0208_r52.rs`): with an unloading building the order breaks the contact and keeps the path; otherwise the contact stays and the path goes. Code agrees. |
| 11 | 110 | S-0309 | claim 30, an aircraft holder | partly | `an_aircraft_holder_drops_its_target_at_an_owner_change` (`claims_s0309_r120.rs`). Code agrees. |
| 12 | 110 | S-0309, S-0601 | claim 30, the simple-deployer exception in *unload* | partly | `a_trigger_owner_change_of_a_simple_deployer_in_unload_keeps_unload` (`claims_s0309_r120.rs`). Code agrees. |
| 13 | 110 | S-0506 r34 | claim 14, a pending target that goes inside | partly | `a_pending_target_that_goes_inside_is_cleared_with_no_reset` (`claims_s0506_r34.rs`): cleared, the controller keeps its target. Code agrees. |
| 14 | 111 | S-0601 | claim 9, the request for *stand*, *prone* or *deployed* at the clear | partly | `seams6_a_release_asks_for_stand_prone_or_deployed_at_the_clear` (`claims_s0601.rs`). Code agrees. |
| 15 | 111 | S-0601, S-0309 | claim 9, claim 30's listed causes through the same function | partly | `a_trigger_owner_change_of_a_firing_infantry_unit_starts_guard_at_once` (`claims_s0309_r120.rs`). Code agrees. |
| 16 | 111 | S-0111 | claim 20, a join or a loop close | partly | `seams6_route_mode_a_loop_close_does_not_block` (`client_name_tip_tests.rs`). Code agrees. |
| 17 | 111 | S-0111 | claim 20, a press refused only at its release (route state, holder limit) blocks | partly | `seams6_route_mode_a_press_refused_only_at_its_release_blocks` (`client_name_tip_tests.rs`): a mixed-state press and one at the holder limit block. Code agrees. |

## Closed earlier (no work)

| Intake | Row | Closed by |
|---|---|---|
| 96 | claims 6, 7, 8 and 24 (4 rows) | a later intake's tests or spec revision |
| 97 | Q-2598 | answered and built |
| 99 | the target leaving play (held drop); the own processing or per-tick search route (2 rows) | intake 107's tests |
| 101 | Q-2636; the M-0021 fail (2 rows) | answered; the row's later intake |
| 104 | Q-2676, Q-2677 (2 rows) | answered and built |
| 105 | Q-2691 to Q-2693 (3 rows) | answered and built |
| 106 | S-0208 claim 6, *enter* | intake 110 (r52) |

## Blocked by an open request

Q-2596, Q-2597, Q-2649, Q-2716 (13 (g) raises), Q-2726 (*sticky*), Q-2746 (a garrison order under
*enter*), Q-2749 (the release cell after the target left play): 7 rows, left as the brief says.
Q-2716, Q-2746 and Q-2749 were answered and built at intake 113 ([intake113.md](intake113.md)).

## Left (no test written)

| Rows | Intake | Spec | Claim or row | Why left |
|---|---|---|---|---|
| 2 | 96, 102 | S-0501 | claim 31, *move* for a vehicle-carrying type | not built: r120 and r132 name no processing for it |
| 1 | 102 | S-0501 | claim 45 after the hold, with the untested-touchdown switch off | Open question 18 |
| 1 | 108 | S-0501 | claim 24, the slide before or in the report's tick | no run of part h (6) reaches the docking position's cell first; no set-up found |
| 3 | 99 | S-0215, S-0107 | the airstrike rows (the reset, the repeated secondary fire, scenario M8 b) | no airstrike sequence is built |
| 2 | 99, 100 | S-0215, S-0208 | the scatter from damage, requests A and B | not built in our hit reactions |
| 1 | 100 | S-0302 | claim 10, a team member has left its team | no house of our engine has teams |
| 1 | 103 | S-0212 | claim 26, follower cars | not built |
| 2 | 97 | M-0017 oracle | the shot reading-point rows and G's per-tick distance, *by construction* | oracle rows that follow from our update order; no game-path test adds to them |
| 1 | 101 | M-0021 oracle | placed infantry at Procedure 5's position, *partly by construction* | as above |
| 1 | 107 | S-0312 | claim 11's wake and splash for a bomb-marked vehicle | not built |
| 2 | 105 | S-0203 | 13 (a), projectiles and animations kept; 13 (c), debris, death weapon and animations repeated | no such object reaches the leftover path in a synthetic set-up; the crate part is tested |
| 1 | 109 | S-0111, S-0015 | the sidebar's names through the table | the sidebar shows no names yet |
| 1 | 110 | S-0208 | the `econ::assign_destination` seam | an engine seam, no claim behind it |
| 1 | 98 | S-0301 | T1's type rows | a pulse cannon or an occupiable building never locks its beam, so no test can hold one |
| 1 | 111 | S-0309 | claim 30, capture and occupation | a building's readiness is its ready mark; no game state reaches the row |
| 1 | 111 | S-0111 | claim 20's Open question (a block kept after an ordinary click released over a point) | no change: not built, our block always ends at the release, as the claim states |

## Part 2: the rocket commit c040e98 split

c040e98 (on e5ac2f4) holds four pieces. Each was built alone on e5ac2f4 (not committed) and both
of intake 110's flipped games were played with `GLASNOST_AI_SIDES=1,2 GLASNOST_AI_QUICK=1` on the
install, with a per-tick fingerprint of every object's position, health, owner and map state and
the houses' credits (a local diagnostic, not committed).

- **A**: claim 14's aim fixed in the launch pass (`fix_at_launch`), the old counter timer kept
  (an update in the flight's start tick does not count, as the old first update did not).
- **B**: claim 24's timer on the tick number, the aim fixed at the first update as before.
- **C**: D plus `take_inside` calling `spawn::target_leaves_play`.
- **D**: the `spawn::removing` refactor alone (`settle_waiting`, `target_gone`).

| Build | Seed 7 map 0 | Seed 10 map 8 |
|---|---|---|
| e5ac2f4 | 39 541 ticks, house 0 | 46 880, house 1 |
| c040e98 | 55 227, house 1; first difference tick 22 167 | 53 646, house 0; first difference tick 24 625 |
| A | identical to e5ac2f4 tick for tick | identical to e5ac2f4 |
| B | 77 457, house 1; first difference 22 167, as c040e98 | 53 646, house 0; first difference 24 625, as c040e98 |
| C | identical to e5ac2f4 | identical to e5ac2f4 |
| D | identical to e5ac2f4 | identical to e5ac2f4 |

**B flips both games.** The refactor (D) changes nothing, so there was no bug to fix there, and
neither does going inside as leaving play (C) nor the aim at the launch (A) in these two games.

**The first difference follows from S-0506 r34.** Seed 7 map 0: house 0's V3 rocket (object 4277)
fires at house 1's infantry 5614 and starts its tilt at tick 22 102. It makes no update in tick
22 132 (its last turn stays 22 131), a missed turn whose cause (a removal before it in the update
order, S-0203 claim 10, is the likely one) was not identified; claim 9's "delays run on" covers any
missed turn. On e5ac2f4 the counter steps
once per update, so the tilt's 60 counts end at the update of tick 22 162 instead of 22 161, and
ascent starts a tick later. With the timer on the tick number (B) the lost turn does not shift
the tilt, and the rocket is one tick ahead from then on; its position first differs at tick
22 167, when ascent's speed first moves it. Seed 10 map 8: rocket 6134 loses two turns in its
tilt (ticks 24 598 and 24 604); on e5ac2f4 its ascent starts two ticks late, first seen at tick
24 625. Claim 14 (r34) says exactly this: "Claim 24's timer runs on the tick number (S-0203
claim 9), so a lost turn within a phase does not shift its ticks", and S-0203 claim 10: "The
skipped object acts normally from the next tick on, with its delays run on (claim 9)". So the
flips are r34's lost-turn rule reaching two games through a few ticks of rocket timing; nothing
was tuned.

## Rocket missed turns

Why the missiles of Part 2 make no update in those tilt ticks. Traced on main c2e65bb with
`GLASNOST_AI_SIDES=1,2 GLASNOST_AI_QUICK=1` on the install and a local diagnostic (not
committed) that logs, for each turn of the pass, its counting removals with their place in the
update order and the code that made them, and the objects they cost a turn. Both games end as
Part 2's row B gives (seed 7 map 0: 55 227 ticks, house 1; seed 10 map 8: 53 646 ticks, house 0),
and the object numbers and ticks are those of e5ac2f4: the games have not moved. (The tilt
timers below are the missiles' recorded start ticks: Part 2 names 22 102 for 4277's tilt, main
records 22 101.)

| Game | Missile, tick | Object directly before it | Removal | Claim |
|---|---|---|---|---|
| seed 7 map 0 | 4277 (tilt timer from 22 101), 22 132 | animation 5616, an ordinary sprite animation with no follow-up type, next in creation after the missile's target, infantry 5614, and infantry 5615, all in play before the launch | it ends its last pass in its own turn and leaves play (S-0109 claim 16): the current object leaving, one counting removal | S-0203 claim 10, cause "an animation that ends" |
| seed 10 map 8 | 6134 (tilt timer from 24 559), 24 598 | animation 7471, the same kind | as above | S-0203 claim 10, the same cause |
| seed 10 map 8 | 6134, 24 604 | animation 7464 (directly before it once 7471 had gone), the same kind | as above | S-0203 claim 10, the same cause |

So an approved claim makes each missile skip its turn: S-0203 claim 10, a counting removal of the
current object, an ordinary animation that ends directly before the missile in the update order.
The missile entered play at its launch, after the animation, so it stands directly behind it
(claim 3). No skip rule of the turn pass itself is involved, and nothing is a bug. Its effect on
the flight is S-0506 claim 14 (r34) with claim 24's timer on the tick number (S-0203 claim 9):
the lost tilt turn makes no update, and the next update's pitch and the tilt's end are as
without the loss. None of the three falls on its tilt's last tick, where the ascent would come a
tick later.

Over both whole games the diagnostic logged 455 lost missile turns, in every phase (pause 73,
tilt 57, raise 164, ascent 42, cruise 39, descent 80). Their 489 counting removals came from: an ending
animation (230), a death removal (153), a particle system leaving play (85), a projectile's
detonation (12), a slave going inside its master (5), and one each of boarding a transport, a
parasite going inside its projectile, a lifter's removal and a domination animation's end. None
came from a removal between turns (`strike::sweep`). Only the three tilt cases above were checked
against their claims; the others are listed as counts.

Tests: `an_animation_ending_before_the_missile_costs_it_a_tilt_turn_and_shifts_nothing`,
`a_tilt_turn_lost_at_its_end_starts_the_ascent_a_tick_later` and
`with_the_removal_skip_off_the_missile_keeps_its_tilt_turn` (`claims_s0506_r34.rs`). They run
the tick pipeline alone: an ordinary animation made before the launch stands directly before the
missile and ends in its tilt, against a twin with no animation. Only tests change: no format
bump, no golden change, and no lanes run. No request was filed.
