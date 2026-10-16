# M6c step 1c claim ledger (spec revisions that landed on main after step 1b)

Part 3 of step 1. Each spec is compared with the revision the code had taken in; the old state is the spec file at the commit named in the table (`git diff <old> HEAD -- spec/<path>`). Every changed claim is classified as *satisfied* (already true, with the test that shows it), *changed* (code changed, with the test) or *pointer* (the claim only points at another spec's claim); *(c)* is editorial (front matter, scenario text, rewording) and *(e)* is not built, with the reason. Per-spec ledgers: [S-0502.md](S-0502.md), [S-0514.md](S-0514.md), [S-0517.md](S-0517.md), [S-0518.md](S-0518.md), [S-0519.md](S-0519.md), [S-0520.md](S-0520.md), [S-0525.md](S-0525.md) (the S-0215 claim 31 row).

| Spec | Revision | Old state | Answers |
|---|---|---|---|
| S-0514 chrono weapons | 5 (from 4) | 8dcc6b8^ | Q-0984 |
| S-0517 deployable units | 7 (from 6) | b4e90bf^ | |
| S-0518 linked weapons | 3 (from 2) | 9c30045^ | Q-0985 |
| S-0519 terror and attack animals | 3 (from 2) | d4632d1^ | Q-0986 |
| S-0520 crates | 3 (from 2) | 292cb87^ | Q-0987 |
| S-0209 occupancy | 41 (from 39) | 81d410a^ | |
| S-0212 pathfinding | 44 (from 43) | f779d89^ | |
| S-0502 jumpjets | 6 (from 5) | 800a588^ | Q-1010, Q-1011, Q-1012 |
| S-0102 draw order | 20 (from 19) | 922ea61^ | |
| S-0213 human orders | 34 (from 33) | 998e595^ | |
| S-0215 behaviours | 51 (from 49) | 5ce7902^ | Q-0983 |

## S-0514 revision 5

| Claim | Class | Result |
|---|---|---|
| Terms (erasing shot, fall-back to rest) | pointer | claim 20 and S-0215 claims 13 to 15. |
| 1 (delivery) | changed | the selection is claim 20's table (`combat::projectile::special_of`); the old rule "unless the warhead's parasite, air-strike or electric-charge flag is set" is gone. Tests `rows_above_erasure_suppress_it_and_ordinary_damage`, `erasure_outranks_the_lower_rows`. |
| 14 (release causes) | changed | the fall-back release moved into the common effects of `process::fall_back`, behind the aircraft early exits and inside the once-per-tick limit. Tests `the_fall_back_release_shares_the_once_per_tick_limit`, `falling_back_to_rest_lets_the_beam_go`, `clearing_the_attackers_target_keeps_the_beam`. |
| 15 (house destruction), table rows "attackers fall back to rest" | satisfied | `temporal::detach_for_destruction`, `complete` and `leaving` call `process::fall_back`; `a_destroy_all_detaches_the_victims_beams`, `removal_detaches_beams`. |
| 20 (one selected special hit) | changed | `special_of` and its use in `detonation_inner`, `explode`, the nuke-maker branch and `parasite::carrier_ended`; the importer reads `MindControl` and `directrocker` (`WarheadType::mind_control`, `direct_rocker`, `chrono_coverage`). Takeover, locomotor replacement and direct rocking are selected only (S-0601 and the locomotor effects are not built): tests `a_selected_row_replaces_ordinary_damage_even_unbuilt`, `direct_rocking_counts_only_against_a_vehicle`. New request Q-1032 (the carried parasite). |
| 21 (no beam) | satisfied | `temporal::deliver` already made no lock and no side effect; the new test `a_firer_without_a_beam_makes_no_lock` shows it. |
| 22 (boarding) | satisfied | `transport::board` hands over only to a gunner; the new test `boarding_a_non_gunner_transport_keeps_the_beam`. |
| Import mapping, scenarios E3, E8, E10, E13 to E15, cross-spec list | (c) | rows `Primary`, `Secondary`, `Gunner`, `OpenTopped` use existing fields. |

## S-0517 revision 7

| Claim | Class | Result |
|---|---|---|
| Scope list (S-0502 claim 24) | (c) | pointer. |
| 5 (landing first) and the transition table row | changed | the deploy order's destination clear reroutes a committed hovering vehicle (`jump::destination_cleared` from `orders::apply_other`); a suppressed unchanged assignment keeps the preceding point and commitment; the landing request no longer creates a commitment (`jump::request_landing`), so it does not wake an idle hover. Tests `claim_5_effective_clear_keeps_the_replacement_point`, `claim_5_a_different_replacement_cruises_before_descent`, `claim_5_a_suppressed_assignment_keeps_the_preceding_point`, `claim_5_no_commitment_the_request_does_not_wake_an_idle_hover`; `claim_5_landing_for_deploy` now gives the hover its commitment as a completed flight does. |
| Scenario (R), (V) text | (c) | |

## S-0518 revision 3

| Claim | Class | Result |
|---|---|---|
| 3 (tie order) | changed | acquisition rank: `World::acquisitions`, `CombatState::acquired`, `linked::acquire` (placement into play, `objects::place`; ownership change, `structures::ownership::change`); `linked::recruit` orders by (distance, rank, object number). Tests `equally_distant_supporters_go_by_placement_order`, `a_captured_building_ranks_after_the_buildings_already_held`, `reacquiring_gives_a_later_rank_and_an_unchanged_owner_gives_none`, `acquisitions_in_one_tick_rank_by_event_order`. Save format 44. |
| 14 (starting to charge) | changed | attack is queued, not started; the idle action is skipped and the processing's own delay table follows (guard rows and area guard's rate plus 1 to 5, divided by 6 when close; `process::process_guard`, `process_area_guard`). Tests `a_charger_keeps_the_guard_delay_after_taking_a_building`, `a_charger_picks_the_first_own_building_by_step_direction`. The area guard delay is coded, not tested. |
| 15 (breaking off) | changed | `SearchContext::BreakOff`: no targeting-timer reset draw (the earlier code took one), the held building neither tested nor cleared, a result clears the mark, nothing found leaves target and mark. Test `the_break_off_search_takes_no_draw_and_keeps_the_target_when_empty`. A result equal to the held building is coded, not tested. |
| Import mapping rows `[Guard]` and `[Area Guard]` Rate, "no data key" | satisfied | `process::rate_delay` reads the behaviour rate already (S-0215 claims 24 and 26). |
| Open questions, scenarios PS4, TC2, TC3 | (c) | |

## S-0519 revision 3

| Claim | Class | Result |
|---|---|---|
| 13 (rearm) | changed | `combat::fire::arrival_delay` replaces `second_weapon_delay`: the counter *k* not *k* + 1 and never advanced, 1 tick without a second weapon, a sonic weapon its rate, *k* below the burst count R(3, 5), otherwise the full delay with R(0, 2) and no berserk halving. Tests in `claims_s0519_r3.rs`; `the_planter_rearms_with_the_second_weapon_delay` adjusted. The bare-rate row for spark, fire and railgun weapons needs a live effect, which is not built (seam). |
| 19 (cancel moment) | changed | `strike::sweep` after each applied order and each object's turn; target changes counted in `CombatState::target_epoch` so A to B to A cancels. Tests `an_order_changing_the_target_cancels_in_its_own_tick`, `a_change_and_back_still_cancels`, `the_same_target_does_not_cancel`. |
| 22 (no bomb warhead) | satisfied | `bomb::detonate` ends the bomb without hits; the hut demolishes. Test `a_bomb_with_no_warhead_ends_without_a_blast` (hut: coded, not tested). |
| 23 (no demolition warhead) | changed | the self-hit and the death were already as the claim says (`a_suicide_without_a_demolition_warhead_still_dies_and_its_death_weapon_fires`); new: the importer warning (`a_suicide_weapon_without_a_demolition_warhead_warns`). |
| Edge cases, Import mapping rows, scenarios T13, T19, T22, T23, Open questions | (c) | the new rows use existing fields. |

## S-0520 revision 3

| Claim | Class | Result |
|---|---|---|
| Terms (crate, stage-picture count), 8, Import mapping (trigger number slot), CR20 | satisfied | `crates::RANDOM` (255) and the trigger reduction were as stated; tests `trigger_crates_store_exact_contents`, `random_crates_store_255`. |
| 13 (unit placement) | changed | `crates::unit`: direct placement with the clear entry outcome (`crates::clear_for`), then the search in the *normal* zone with one more direct test of its result, then the money fallback. Tests `an_empty_crate_cell_takes_the_unit_directly`, `a_stationary_friend_refuses_the_direct_placement`. |
| 24 (stage-picture count) | satisfied | `econ::resource::crate_stage` already used the flat run's length (12) as the count and left higher stages alone; test `the_stage_cap_is_one_below_the_picture_count`. |
| 28 and the `[Powerups]` row (layered table) | changed | the layering records a reset event for each omitted key of a layer holding a surviving `[Powerups]` section (`convert::layers`, `quirks.override_resets_keys`), the importer applies it (`convert::sim::econ::crate_import`). Tests `the_power_up_table_is_layered`, `the_reset_animation_lookup_finds_no_type_named_none`. New request Q-1033. |

## S-0209 revision 41

| Claim | Class | Result |
|---|---|---|
| 1 and 13 (jumpjet listing predicate) | changed | pointer to S-0502 claim 4: `jump::placement_qualifies` (*r* < 208 or *r*′ = 0). Tests `claim_4_flat_land_predicate`, `claim_4_bridge_predicate_uses_two_heights`, `claim_4_crashing_is_not_falling`. |
| S-0102 alignment item | (c) | |

## S-0212 revision 44

| Claim | Class | Result |
|---|---|---|
| T5 row "stopped-movement reroute" and claim 25 | changed | the row is `jump::Use::Stop` for every effective clear (stop, deploy, behaviour); searches keep the unit's own occupancy (`jump::order_point` no longer lifts its mark). Tests `claim_23_vehicle_own_mark_excludes_its_own_cell`, `claim_24_effective_clear_then_suppressed_assignment`. |
| Scenario text | (c) | |

## S-0502 revision 6

| Claim | Class | Result |
|---|---|---|
| Terms (destination-refresh state), 3 | pointer | not modelled (no activation is stated): Q-1031. |
| 4 | changed | the two-height predicate (tests listed under S-0209 above); `claim_15_touchdown_on_a_deck` now expects no listing a few leptons above the deck over a high bridge. |
| 7 | satisfied | the early cruise occurs when the accepted point is in the ascent cell: `claim_7_own_cell_early_cruise` and `claim_7_bridge_early_cruise_height` now use an entering vehicle (`go_here_entering`). |
| 16, 23 | changed | own occupancy is not lifted; the own-cell filter of jumpjet-type infantry (`jump::own_cell_filtered`). Tests `claim_23_infantry_own_cell_order_is_ignored`, `claim_23_the_filter_reads_the_type_flag`, `claim_23_vehicle_own_mark_excludes_its_own_cell`. |
| 24 | changed | effective clears reroute, unchanged vehicle assignments are suppressed, the deploy clear reroutes, a landing request creates no commitment. Tests above and `claim_24_clear_without_a_commitment_does_nothing`. Open: Q-1030, Q-1031. |
| Import mapping, scenarios J2, J3, J16, J18, J22 | (c) | |

## S-0102 revision 20

| Claim | Class | Result |
|---|---|---|
| 3, the layer table, D12 (b) | (e) | the render layer of units is not modelled (the map view draws mobile objects in the ground layer, `quirks.layer_change_on_events_only`); the predicate S-0102 shares with S-0502 claim 4 is `jump::placement_qualifies`. |

## S-0213 revision 34

| Claim | Class | Result |
|---|---|---|
| 8 (stop order) | changed | a stop order reroutes a committed jumpjet unless the vehicle assignment is unchanged (`jump::stop_ordered`). Test `claim_24_stop_in_each_flight_state` and `claim_24_clear_without_a_commitment_does_nothing`. |
| 10 (deploy order) | changed | `orders::apply_other` calls `jump::destination_cleared`. Tests of `claims_s0517_r7.rs`. |
| 21 and table T3 notes (jumpjet filters) | changed | the own-cell filter and the suppression; the get-up of a repeated destination skips an assignment the filter ignores (`infantry::destination_assigned`: coded, not tested). |

## S-0215 revision 51

| Claim | Class | Result |
|---|---|---|
| 13 and the destination-clear paragraph | changed | pointer to S-0502 claim 24; the temporal release is one of the common effects (S-0514 claim 14). |
| 31 (own-cell shot, answers Q-0983) | satisfied | `process::immune_deployed` already fired in the deciding processing, dropped the target after the shot and returned the frame count; new test `the_own_cell_shot_is_made_in_the_deciding_processing` (the field exists in the tick of the shot). |
| Import mapping rows (no data key, `DeployedFire`), scenario B19 (e) | (c) | |

## Requests

Retired interim rules: Q-0983 (S-0215 claim 31), Q-0984 (S-0514 claims 14, 20 to 22), Q-0985 (S-0518 claims 3, 14, 15), Q-0986 (S-0519 claims 13, 19, 22, 23), Q-0987 (S-0520 claims 8, 13, 24, 28), Q-1010, Q-1011 and Q-1012 (S-0502 claims 4, 23, 24). New requests with interim rules: Q-1030 (the unchanged destination assignment of jumpjet-type infantry), Q-1031 (the destination-refresh state and the balloon-hover substitution), Q-1032 (the carried parasite when a higher row suppresses attachment), Q-1033 (an animation type named NONE and S-0003 claim 14).

## Results of the runs

- Save format 44 (`World::acquisitions`, `CombatState::acquired` and `target_epoch`, `Strike::watched_epoch`); AI game format 12.
- Sim-gate goldens (`GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored`): regenerated. The six scenario files changed because the hashed state gained fields; no behaviour change was needed to explain it, and the AI sweeps below did not move.
- AI whole games, seeds 1 to 32 (`GLASNOST_SWEEP=1-32`, [docs/development.md](../development.md) M6c row 2a): the naval sweep passed 29 of 32 before (seeds 5, 7 and 17 send no ship across) and 29 of 32 after (the same seeds); the superweapon sweep passed 29 of 32 before and 29 of 32 after (seeds 17, 18 and 20, at the same ticks). The fixed-seed tests `builds_a_navy_on_a_water_map` and `builds_and_fires_a_superweapon` pass.
