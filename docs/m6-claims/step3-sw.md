# M6b step 3 (superweapons, drops, firing, tick order and presentation): spec revisions since 6a1cc2d

Classification of every changed claim in `git diff 6a1cc2d HEAD` of the specs below. Buckets:
**(a)** changes behaviour the engine implements (implemented, with a test); **(a, already done)** the
committed code already did what the new text says (a test was added where none existed); **(b)** a
seam or a pointer into a spec that is not built (named); **(c)** editorial, no behavioural change;
**(e)** not built, with the reason.

Every file's front matter (revision, approval date, `requests`) changed. That is (c) and is not
repeated. The per-spec ledgers ([S-0512](S-0512.md), [S-0513](S-0513.md), [S-0522](S-0522.md),
[S-0301](../m4-claims/S-0301.md), [S-0308](../m4-claims/S-0308.md)) carry the test names; the code
sections are in `docs/development.md` ("Simulation (M6b: superweapons)", "Simulation (M6b:
parachutes and drops)", "Simulation (M6b: depot cycle)").

| Spec | Revisions |
|---|---|
| S-0512 Superweapon charge | 7 to 8 |
| S-0513 Superweapon effects | 10 to 11 |
| S-0522 Parachutes and drops | 4 to 5 |
| S-0301 Firing cycle | 16 to 17 |
| S-0308 Weapon choice | 11 to 12 |
| S-0215 Behaviours | 44 to 45 |
| S-0212 Pathfinding | 41 to 42 |
| S-0203 Tick order | 28 to 30 |
| S-0207 Ground vehicle movement | 28 to 29 |
| S-0102 Draw order | 17 to 18 |
| S-0103 Palettes and house colour | 12 to 13 |
| S-0105 Voxel rendering | 17 to 18 |
| S-0106 Voxel tilt on slopes | 8 to 9 |
| S-0109 Animation playback | 25 to 26 |

Requests. **Answered by these revisions** (the interim rows are retired or narrowed): Q-0738 (S-0513
claim 48), Q-0741 (claims 33 and 49), Q-0830 (S-0207 claim 26), Q-0799 (S-0105 claim 40). The
request files for Q-0738, Q-0741, Q-0760 to Q-0765 and Q-0799 already read `answered`; Q-0760 to
Q-0765 belong to S-0524 (not in this step). **Still open:** Q-0800 (the position of a multi-cell
building in the storm strike witness test; S-0513 claim 18 did not change). **Filed:** Q-0850 (the
landing point of a warp member whose destination cell is off the map or whose nearby search fails,
S-0513 claim 49's unmeasured rows) and Q-0851 (what a warp member does with the rest of its turn in
the stages S-0513 claim 48 marks unmeasured).

Changes of saved state or of the rules schema made in this step (the lead regenerates the goldens
and the schema file and bumps the save format): **none to the saved state's layout and none to the
rules schema.** Behaviour changes that move the state hash of a game that meets them: an infantry
unit falling by parachute is no longer ready to change (`process::ready`), so a paratrooper's
queued rest starts after landing; a destination naming a unit is no longer removed by cell equality
(`drive::goal_cleanup`); a warp survivor with an off-map destination cell now joins the warp
(`warp::examine`). The goldens of games with paradrops, units following units or warps near the
map edge may therefore move.

---

## S-0512 Superweapon charge (rev 7 to 8)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 8 (busy-silo acceptance) | (a, already done) | A chosen selling silo refuses the launch; a busy silo records the target and plays the sounds, spends readiness, and starts no new sequence before 8 ticks after the accepted launch, a fresh one at or after; S-0513 claim 47 owns the boundary. | `nuke::accepts`, `nuke::silo_launch`, `ObjSw::silo_busy_until`. New test `s0512_claim8_a_busy_silo_spends_readiness_without_another_missile_until_eight_ticks_later` (through the fire order and the record, in `claims_s0513_nuke.rs`); the claim 47 tests exist (`claim47_busy_silo_interval_is_eight_ticks`). |
| SW21 text in the verification section | (c) | Scenario. | none |

## S-0513 Superweapon effects (rev 10 to 11)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 34 (second stage before any source pick) | (a, already done) | Cell (0, 0) is the source, the ordinary warp is attempted, a granted first stage restarts, an ungranted one is untouched. | `launch::second_stage`, `launch::effect` (step 2, Q-0737). Tests `claim10_second_stage_before_any_pick_warps_from_cell_zero_and_restarts_the_first_stage`, `claim10_second_stage_before_any_pick_is_refused_with_the_quirk_off`. |
| 39 (arrival sensor update) | (a, already done) | A surviving member's disc transfer and adjacency reveal come at the arrival-effects update, after the arrival sound and the arrival fate and before the landing animation and the hold; the disc stays at the old cell through departure, retries and appearance; a member killed by the fate test gets none. | `warp::lift_off` (disc kept), `warp::arrive` (`cloak::machine::cell_movement_completed`). Test `claim_22_warp_member_disc_transfers_at_the_arrival_effects` (`claims_s0516_warp.rs`). |
| 43 (resolved paradrop list) | (a, already done) | The plane count is the length of the compact resolved infantry list, compared with the independently parsed count list; unknown names and a vehicle's name count, empty items and no-type names are omitted. | The importer's creating references (`Refs(Kind::Infantry)`), `planes::spy_plane_count`. New test `spy_plane_count_lists_are_the_resolved_ones` (`crates/convert/src/sim_econ_tests.rs`); `paradrop_payload_names_create_infantry_types` exists. |
| 48 (confirmed rows) | (a, already done) for the leaving exclusion; (a) for the hold ordering test | A member still leaving takes no other part of its turn and drops target and destination; a member beginning its turn in the arrival hold has the hold-end test first, then the rest of its turn; no ordering after the protection tint is implied. | `warp::turn`, `warp::join`, `tick::object_turn`. New tests `claim48_a_member_still_leaving_takes_no_other_part_of_its_turn`, `claim48_the_hold_end_test_precedes_the_rest_of_the_turn`; `claim48_a_warp_member_drops_its_target_and_destination` exists. |
| 48 (unmeasured rows of the table) | (b) | Pending, departure, retry, placement and arrival-effects turns, and the clearing of the path and queued destinations at launch, are unmeasured. | Interim: the warp update is the whole turn there and launch clears the path. **Q-0851** filed; the row is reworded in `docs/development.md`. Q-0738's row is retired. |
| 49 (source marker) | (a, already done) | The marker takes its creation draws whichever house is local. | `warp::pick_source`. Test `claim33_the_marker_takes_its_creation_draws_whichever_house_picks`. |
| 49 (off-map destination cell) | (a) | Claim 34's parasite release and claim 35's organic death still occur; an otherwise eligible survivor joins the warp (no early rejection). Its landing point and fate are unmeasured. | `warp::examine` (was: left in place). The landing point is the blank stand-in cell (0, 0) (interim, **Q-0850**, a note is added to the world's notes). Test `claim49_an_off_map_destination_is_no_early_refusal`. |
| 49 (failed nearby search) | (a) | The failed result replaces the landing point without a success test and does not keep the old point; the unsuccessful check stays in the retry stage; a retry sets the held delay to the teleport delay, also when the check then passes. The exact point and fate are unmeasured. | `warp::redirect` (was: kept the old point). The replacement is the stand-in cell (0, 0) with the position within the cell of claim 36 (interim, **Q-0850**). Test `claim49_a_failed_search_replaces_the_landing_point_and_the_retry_sets_the_delay`. |
| 18 (strike witness, multi-cell building) | (b) | Unchanged in this revision. | **Q-0800** stays open; its interim row stays. |
| Import mapping rows `Type`, `Action`, `AllyParaDropInf`/`Num`, the new "no data key" row | (a, already done) / (c) | Words are S-0512 claims 23 and 24; no key supplies the claim 48 and 49 answers. | `glasnost_rules::sim_econ::SUPERWEAPON_KIND_WORDS`, `SUPERWEAPON_ACTION_WORDS`. |
| SW12 control, SW23, SW24, open questions | (c) | Scenarios and wording. | none |

## S-0522 Parachutes and drops (rev 4 to 5)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 9 (release sensor coverage) | (a, already done) | A release adds any mobile sensor coverage at once at the release cell, without an adjacency reveal; descent does not relocate it. | `objects::place` through `econ::events::entered_map` (`cloak::machine::entered_map`). New test `claim9_12_release_adds_the_sensor_disc_at_the_release_cell_and_landing_keeps_it`; `claim_22_entering_play_adds_the_disc_without_an_adjacency_test` exists. |
| 12 (landing) | (a, already done) | Landing is S-0516 claim 22's cell-arrival update, also in the same cell, before a remaining parachute ends. | `fall::turn`. The same new test; `claim_22_fall_landing_is_a_cell_arrival_update` exists. |
| Backing text of claims 9 and 12 | (c) | Scenario backing. | none |

## S-0301 Firing cycle (rev 16 to 17)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Summary, Terms (drop attempt) | (c) | Definition of the S-0522 claim 11 attempt. | none |
| 1 | (a, already done) | A drop attempt leaves the countdown unchanged on refusal, expires it on success, and sets no ordinary delay. | `combat::fire::shot` (the drop branch). New test `s0301_claims_1_4_18_a_drop_attempt_takes_no_ordinary_delay_burst_step_or_draw`; `claim11_a_shot_drop_replaces_the_ordinary_shot_effects` exists. |
| 4 | (a, already done) | While a first passenger exists an attempt never advances the burst counter or draws an ordinary delay; the exception ends with the cargo, and later attempts of the same burst are ordinary shots. | `shot_drops` is tested at every attempt. The same new test (a burst of 3). |
| 10 | (a, already done) | A successful drop spends one round at once, a refused one none; the run's pending round (S-0501 claim 15) is separate and also made after a refusal. | `paradrop::attempt`; `combat::fire::shot` returns true. New test `s0301_claims_10_15_a_drop_spends_one_round_at_once_and_is_not_a_void_shot`; `claim11_a_refused_shot_drop_changes_nothing_but_still_counts_as_a_shot` exists. |
| 15 | (a, already done) | A drop is not a void shot; neither outcome produces a report, a muzzle animation, a spray draw or a projectile. | The same test (no `Effect::Shot` or `Effect::Anim`, no projectile). |
| 18 | (a, already done) | A drop attempt takes no ordinary-shot draws, whether successful or refused. | The claims 1, 4, 18 test (the random state equals the twin's attempt alone). |
| State table row, backing text, F17 text | (c) | Restate the above; F17 is a scenario. | none |

## S-0308 Weapon choice (rev 11 to 12)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Terms (installed upgrade) | (c) | Positions 1 to 3, not installation history. | none |
| 12 provider order | (a, already done) | Positions 1 through the count (at most 3), increasing, empty positions and types without the slot skipped; positions above the count ignored; count 0 none; ordinary slots only; then occupants, then the building's own slot. | `combat::weapon::installed_upgrades` (step 2, S-0406 revision 9). `s0308_claim_12_upgrades_supply_weapons`, `s0406_claim_12_count_zero_retains_positions_without_effect`, `s0406_claim_12_count_above_three_has_three_positions`. |
| 12 gate | (a, already done) | The gate applies only with a nonzero count and an occupied position 1 and tests that position alone; an empty position 1 bypasses it. | `combat::behave::upgrade_gate`. `s0215_claim_32_upgrade_gate_predicate`, `s0406_claim_12_level_skip_bypasses_the_gate`. |
| 12 excessive count | (a, already done) | At most the three valid positions are used; the importer warns about a capacity above 3. | `add_upgrade`; the warning in `convert::sim_econ` (`Upgrades`). `s0406_claim_12_count_above_three_has_three_positions`. |
| 16 registration order | (a, already done) | Numbered references register normal before elite at each number, lower numbers first (W11's recorder extension is a measurement). | `convert::refs`. `numbered_weapons_above_eighteen_register` (`refs_tests.rs`). |
| Interactions paragraph, scenario part h, W11 text | (c) | Wording and scenarios. | none |

## S-0215 Behaviours (rev 44 to 45)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 4 (the *none* case) | (a) | A due processing with the current behaviour none chooses exactly 450 ticks, without a delay draw, changing neither the current nor the queued slot; measurable with a fresh paratrooper. | `process::process` (the no-rule arm, `NO_RULE_DELAY`). New test `s0215_claims_4_11_a_fresh_paratrooper_with_no_current_behaviour_waits_450_ticks`. |
| 11 (infantry falling) | (a) | A falling infantry unit is not ready to change, so the queued rest placement left does not start in the air; falling move clicks are refused, not deferred; claim 11's fall-back for an infantry unit with neither behaviour is unchanged. | `process::ready` (new `fall.falling` test for infantry). The same test, which also checks the rest starts after landing, before the 450 ticks are over; `claim17_released_infantry_rests_with_guard_queued_and_takes_no_move_while_falling` exists. |
| B44, B36 text | (c) | Scenarios. | none |

## S-0212 Pathfinding (rev 41 to 42)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 20 to 22 (no-track cleanup versus search failure) | (a, already done) | The cleanup is a locomotion rule needing no search, alignment or due processing; mere cell equality does not clear a building destination; final-track completion clears only the locomotor point; the next no-track update clears the cell goal. | `drive::goal_cleanup`, `drive::completion_clears_goal` (step 2). `claim_26_*` in `claims_s0207_goal.rs`. |
| 25 table row (starting-building restoration displacement), paragraph | (b) | The S-0524 claim 13 restoration (reference-distance choice within the collected cells, facing 0, terrain-surface centre, stand-in attempt at (0, 0) when no result, removal on failed placement) belongs to the S-0524 pass, not this one. | `mp_triggers::replace` keeps its Q-0764 interim (substitution search from the old cell). Not touched here. |
| Backing text, P29 | (c) | Scenario. | none |

## S-0203 Tick order (rev 28 to 30)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 4 (within-turn superweapon warp) | (a, already done) for the leaving exclusion and the hold ordering; (b) for the unmeasured rows | The text restates S-0513 claim 48 (no claim of a universal ordering after the protection tint, no assumption of one warp update per tick or of a whole-turn warp). | `tick::object_turn` comment and `warp::turn`. The S-0513 claim 48 tests above; the unmeasured rows are Q-0851. |
| Verification text | (c) | Scenario pointer. | none |

## S-0207 Ground vehicle movement (rev 28 to 29)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 26 (destination naming a unit; Q-0830) | (a) | A destination naming a building, vehicle, infantry unit or aircraft, moving or stationary, is not a cell destination: cell equality alone removes neither destination nor point; the guard row removes it only when the retained locomotor point equals the world point in all three coordinates, a unit goal included, and compares the retained point rather than the named unit's position. | `drive::goal_cleanup` (`named`, was a building test; the Q-0830 interim is gone). New tests `claim_26_a_destination_naming_a_unit_is_kept_by_cell_equality`, `claim_26_guard_row_for_a_destination_naming_a_unit_compares_the_retained_point` (a vehicle and an infantry unit; an aircraft takes the same `Target::Object` branch, and no aircraft is driven by this locomotor). |
| Claim 25 text, constants table row, D31 | (c) | Wording and scenario. | none |
| S-0507 claims 5 and 6 (boarding exchange, named by the added text) | (b) | Arrival-time boarding is the transport spec's. | `transport::arrival`, already built. |

## S-0102 Draw order (rev 17 to 18)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 18 (voxel shadow point, aircraft) | (a) | An aircraft's shadow draw point is S-0105 claim 40's; the shadow is a depth reader with the body's bias and the upright profile and is drawn before its own body, which is never darkened by it. Vehicle shadow points stay unresolved (Open questions). | `mapview::build::aircraft_shadow_point` and `vehicle_voxels`. `flying_aircraft_shadow_lies_on_the_ground` (now also checks the reader test, the shared bias and profile and the order). D23 (e) is a local-lane observation. |
| Open questions | (c) | Vehicle shadow point remains open. | none |

## S-0103 Palettes and house colour (rev 12 to 13)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 8 (row 3, parachute) | (a, already done) | The parachute is drawn with its object when a type is held; a genuinely absent type draws nothing; an unlisted name creates a type. | The app attaches the parachute only on the simulation's `SwEvent::Parachute` (`Live::attach_parachute`), which `paradrop::attempt` raises only for a held type: `claim13_absent_type_uses_the_parachute_recurrence_once_in_the_release_tick`, `claim13_an_unlisted_name_is_a_type_not_an_absent_one`; the app test `parachute_follows_its_paratrooper`. |
| Import mapping (`Parachute`, `BombParachute`) | (a, already done) | Creating animation references; `none` and `<none>` clear; omitted or empty layered values retain; initially none; the absence needs no warning. | `convert::refs`, `convert::sim_econ`. New test `parachute_reference_layers_and_absence`. `BombParachute` is a creating reference (types are created) with no simulation field: projectile parachutes are not built (the claim's own "unmeasured" and Open questions). |
| Scenario P8 pointer, Open questions | (c) | Scenario and wording. | none |

## S-0105 Voxel rendering (rev 17 to 18)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 40 (aircraft shadow anchor; Q-0799) | (a) | The anchor is the projection of the horizontal position at the terrain height there (slope included, no clamp to the body, also at heights 0 and below the surface), identical for every aircraft locomotor; a qualifying high bridge raises it by exactly 60 pixels (the cell and its neighbour towards the bridge axis are both high-bridge cells, and *a* ≥ 416 off the deck or *a* ≥ 0 on it). | `mapview::build::aircraft_shadow_point`, `terrain_height`; `Mobile::on_deck`, `Mobile::deck_bridge`, `live::deck_bridge`. New tests `claim40_the_anchor_is_the_terrain_projection`, `claim40_a_qualifying_bridge_raises_the_anchor_by_60_pixels` (`live_tests.rs`); `flying_aircraft_shadow_lies_on_the_ground` is updated to claim 40 (heights 300 and 2500 give one anchor). The Q-0799 interim and its scene note for aircraft are removed; the note stays for vehicles. A static map view has no deck mark: its aircraft use the terrain. MS-30 is a local-lane observation. |
| Terms (shadow draw point, aircraft shadow height *a*) | (a) | *a* = *z* − *H* − *b*. | as above. |
| Import mapping row `Locomotor` | (c) | The original's non-flying displacement is corrected without a switch; our anchor does not read the locomotor. | none |
| Open questions | (c) | Vehicle shadow vertical placement stays open. | none |

## S-0106 Voxel tilt on slopes (rev 8 to 9)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Open questions | (c) | Wording: the airborne vehicle shadow stays open, aircraft are S-0105 claim 40's. | none |

## S-0109 Animation playback (rev 25 to 26)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 25 and Edge cases (no type versus a type with no sprite) | (a, already done) | A defined type with no sprite follows claim 25 and the missing-sprite rule; an animation with no type at all has no ordinary playback, while its attachment effects are S-0522 claim 13's; the importer warns about invalid references but not about an absent parachute type. | `paradrop::attempt` (`FallState::chute_once`, `quirks.paratrooper_any_animation_slow_fall`); `parachute_reference_layers_and_absence`; the S-0522 claim 13 tests. |
| Backing text | (c) | Inherits S-0522 claim 13's backing. | none |

---

Not built, with the reason: the pixel observations D23 (e), MS-30, P8 and P11 (local lane, need an
install); the S-0524 claim 13 restoration displacement of S-0212's table (the S-0524 pass); the
unmeasured warp rows (Q-0850, Q-0851); the multi-cell building position in the strike witness test
(Q-0800).

Test environment note: `crates/deterministic/ai/tests/ai_sw.rs` `sends_the_spy_plane_at_the_enemy`
was failing before this step's changes took effect on the lead's merge: the test waited a fixed 400
ticks after placing the giving building, and the house's own brain fires a ready record at once,
so the order was already pending in the world when the record was inspected. It now waits for the
record to be ready (`ready_until`). No S-0513 revision 11 effect was involved.
