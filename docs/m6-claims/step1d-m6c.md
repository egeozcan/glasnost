# M6c step 1d claim ledger (spec revisions that landed on main after step 1c)

Part 4 of step 1. Each spec is compared with the revision the code had taken in; the old state is the spec file at the commit named in the table (`git diff <old> HEAD -- spec/<path>`; for specs whose revision the earlier ledgers do not record, the commit that introduced the previous revision). Every changed claim is classified as *satisfied* (already true, with the test that shows it), *changed* (code changed, with the test) or *pointer* (the claim only points at another spec's claim); *(c)* is editorial (front matter, scenario text, rewording) and *(e)* is not built, with the reason. Per-spec ledgers: [S-0518.md](S-0518.md), [S-0519.md](S-0519.md), [S-0520.md](S-0520.md), [S-0524.md](S-0524.md), [S-0525.md](S-0525.md), [S-0614.md](S-0614.md).

| Spec | Revision | Old state | Answers |
|---|---|---|---|
| S-0518 linked weapons | 4 (from 3) | 9c30045 | |
| S-0519 terror and attack animals | 4 (from 3) | d4632d1 | |
| S-0525 radiation fields | 2 (from 1) | 14106de | |
| S-0212 pathfinding | 45 (from 44) | f779d89 | |
| S-0213 human orders | 35 (from 34) | 998e595 | |
| S-0215 behaviours | 52 (from 51) | 805943a | |
| S-0301 firing cycle | 20 (from 18; 19 landed before step 1b and is part of the diff) | b85eb7b | |
| S-0309 target acquisition | 19 (from 18) | 479ce63 | |
| S-0404 production queue and payment | 18 (from 17) | 3e488b5 | |
| S-0107 infantry sprite sequences | 15 (from 14) | 69a114b | |
| S-0112 overlay and terrain object drawing | 13 (from 12) | 0e3ce31 | |
| S-0006 rules layering | 18 (from 17) | e38fcc9 | Q-1034 (reversed by revision 20, not taken in) |
| S-0203 tick order | 33 (from 32) | 8d51d85 | |
| S-0414 victory and defeat | 14 (from 13) | 1c85680 | |
| S-0524 multiplayer trigger completion | 9 (from 8) | 268a3b6 | |
| S-0614 parasites | 8 (from 7) | 3663115 | |
| S-0601 mind control (M7, not built) | 5 (from 4) | 3106b34 | |

## S-0518 revision 4

| Claim | Class | Result |
|---|---|---|
| 11 (charge shot) and scenario | satisfied | S-0514 claim 20 owns the selection: `combat::projectile::special_of` ranks mind control, bomb planting, then electric charging above parasite attachment and erasure; a selected charge row replaces ordinary damage and every lower row whether or not it registers (`charge_hit` registers only an infantry firer on a building). New test `charging_registers_and_suppresses_the_lower_rows_without_falling_through` (`claims_s0514_r5.rs`): infantry on a building registers with no erasure and no damage, a vehicle target registers nothing and nothing falls through, for parasite, erasure, locomotor and designation as the lower row. |

## S-0519 revision 4

| Claim | Class | Result |
|---|---|---|
| 2 (planting selected by S-0514 claim 20), T24 a, b | satisfied | `special_of` (takeover suppresses planting, planting suppresses every lower row). New tests `takeover_suppresses_planting`, `planting_outranks_charging_and_the_lower_rows`, `a_vehicle_designator_with_planting_does_neither` (`claims_s0519_r4.rs`). |
| 7 (disarming selected; direct rocking an earlier row only for a vehicle; disguise and nuke-making later rows), T24 d to f | satisfied | New tests `earlier_rows_suppress_disarming`, `direct_rocking_suppresses_disarming_only_against_a_vehicle`, `disarming_outranks_disguise_and_nuke_making`. |
| 16 (designation selected; takeover, planting, charging, parasite, erasure and locomotor suppress it; selected designation suppresses direct rocking, disarming, disguise and nuke-making), T24 c | satisfied | New tests `higher_rows_suppress_designation` (with a control that calls the strike) and `designation_suppresses_the_lower_rows`. |
| State tables (planted bomb, strike) | pointer | the same selection. |
| Scenario T24, Backing lines | (c) | |

## S-0525 revision 2

| Claim | Class | Result |
|---|---|---|
| 1 (field created on the detonation's tick; the own-cell shot's tick is the deciding processing's) and 4 (top-up effective on that tick), RF4 part e | satisfied | S-0215 revision 51 (Q-0983) made the shot in the deciding processing: `the_own_cell_shot_is_made_in_the_deciding_processing`. The top-up case and the absence of a second shot from the deployed-fire sequence were untested: new test `the_own_cell_top_up_is_effective_in_the_deciding_tick` (`claims_s0525_deploy.rs`). |

## S-0212 revision 45

| Claim | Class | Result |
|---|---|---|
| 25 (unit-crate fallback: a separate direct placement attempt at the crate cell accepting only a clear entry, not ring-zero qualification; the search's cell gets the same direct test once; refusal ends it) and the T-table line | satisfied | S-0520 revision 3 claim 13 (Q-0987): `an_empty_crate_cell_takes_the_unit_directly`, `a_stationary_friend_refuses_the_direct_placement` (`claims_s0520_r3.rs`). |

## S-0213 revision 35

| Claim | Class | Result |
|---|---|---|
| 3 (air-strike cancellation at every applied target change, A to B to A, identical target, no applied change) and the scenario paragraph | satisfied | S-0519 revision 3 claim 19: `an_order_changing_the_target_cancels_in_its_own_tick`, `a_change_and_back_still_cancels`, `the_same_target_does_not_cancel` (`claims_s0519_r3.rs`). |

## S-0215 revision 52

| Claim | Class | Result |
|---|---|---|
| 12 (strike designators' targets: every actual change cancels immediately; same target does not; away and back still cancels) | satisfied | the same three tests. |
| 13 (release of an attached temporal beam as a common fall-back effect, once per tick, skipped by the aircraft early exits; not by stop-order clearing alone) | satisfied | S-0514 revision 5 claim 14: `the_fall_back_release_shares_the_once_per_tick_limit`, `falling_back_to_rest_lets_the_beam_go`, `clearing_the_attackers_target_keeps_the_beam` (`claims_s0514_r5.rs`, `claims_s0514*.rs`). |
| 23 (general rules table: the charging break-off search replaces the charge-target check; mark-gated, any nonempty result accepted without another check, even the held target; vehicles do not run it; no halving) | satisfied | `process::process_attack` calls `linked::break_off`; tests `the_break_off_search_takes_no_draw_and_keeps_the_target_when_empty`, new `the_break_off_ignores_passive_acquisition_and_the_opportunity_gate`, `the_no_auto_fire_restriction_still_limits_the_break_off`, `a_result_equal_to_the_held_building_clears_the_mark` (`claims_s0518_r3.rs`). |
| 28 (S-0519 claim 13 owns charge planting at arrival; not a sabotage-processing effect) | pointer | `bomb::charge_arrival`; `claims_s0519_r3.rs`. |

## S-0301 revision 20 (and 19)

| Claim | Class | Result |
|---|---|---|
| Terms, 1, 2, 4, 19, F1 table row and F18 (charge arrival replaces the countdown without a shot, burst counter unchanged, delay table) | satisfied | S-0519 revision 3 claim 13: `combat::fire::arrival_delay`, tests `a_fresh_walker_takes_the_short_arrival_delay`, `a_spent_burst_takes_the_full_arrival_delay_without_berserk_halving`, `no_second_weapon_gives_one_tick`, `a_sonic_second_weapon_keeps_its_bare_rate`; new `arrival_replaces_a_running_countdown` (a countdown already running is replaced, burst and ammunition unchanged). The bare-rate row for a spark, fire or railgun weapon with a live effect stays a seam (no live effects). |
| 7 and F1 (occupant count: the original counts retained dead places, our engine uses the count after the same-update removal; with n = 0 the building is not garrisoned for F1) | satisfied | S-0508 claim 23 (Q-0970): `garrison::count` is the post-removal count; `claim_23_the_delay_uses_the_count_after_removal` (`claims_s0508_fire.rs`). |

## S-0309 revision 19

| Claim | Class | Result |
|---|---|---|
| 1 and 4 (the charging search: the held target neither prevents nor is cleared; any nonempty result, even the same object, becomes the target and clears the mark; empty leaves both; claims 2 and 3 do not limit it; claim 5's no-auto-fire restriction still applies; no timer gate or reset draw) and A17 part c | satisfied | `linked::break_off` through `acquire::search` with `SearchContext::BreakOff` (no gates of claims 2 and 3, `classes` applies no-auto-fire); the three new tests named above. |
| A-series run 3 text | (c) | |

## S-0404 revision 18

| Claim | Class | Result |
|---|---|---|
| 7 (acquisition order: placement or ownership change; capture ranks last; reacquiring ranks last; unchanged owner no reacquisition; within a tick by event order) and its use in claims 13, 14 and F1, scenario P7 acquisition-order runs | satisfied | `econ::events::entered_map` and `owner_changed` keep `HouseEcon::acquired` in event order (`structures::ownership::change` returns early for an unchanged owner); new test `claim_7_acquisition_order_of_the_releasing_building` (`claims_s0404.rs`). Claims 13 and 14 and F1 read the same list. |

## S-0107 revision 15

| Claim | Class | Result |
|---|---|---|
| 11 (the radiation-immune deployer's own-cell shot fires in the deciding processing, clears its target there, and the deployed-fire sequence presents an already-made shot) and M8 part e | satisfied | S-0215 revision 51: `the_own_cell_shot_is_made_in_the_deciding_processing`; no second shot: `the_own_cell_top_up_is_effective_in_the_deciding_tick`. The viewer's sequence playback is unchanged. |

## S-0112 revision 13

| Claim | Class | Result |
|---|---|---|
| 1 row 6, 8 precedence 1, Edge cases and quirks (`quirks.crate_data_value_ignored`, default on) | changed | new field `OverlayQuirks::crate_data_value_ignored` (default on, listed in `named()`) with `crate_body_frame` and `crate_shadow_frame`; `mapview::build` draws a crate's body and shadow through them. Test `crate_frames_follow_the_switch` (`render::overlay`); `every_rendering_spec_quirk_has_a_switch` passes. The viewer builds with the defaults, so the switch is not reachable from a rules file yet (as with the other overlay switches). |
| Terms, 10, Constants table (the resource stage-picture count 12, equal to F, for every image number except -1; S-0520 claim 24 owns the use) | satisfied | the picture count of a crate's stage increase is the run's flat length, 12 for every run (`econ::resource::crate_stage`); `the_stage_cap_is_one_below_the_picture_count` (`claims_s0520_r3.rs`). |
| Companion test text, import mapping row | (c) | |

## S-0006 revision 18

| Claim | Class | Result |
|---|---|---|
| 5 (the power-up rows act on the held table per layer; an absent section changes nothing; a present section resets an omitted entry; a present short entry changes only the tokens it supplies; a later layer omitting the section does not undo an intermediate reset) | satisfied | S-0520 revision 3 claim 28: the layering tests in `convert::sim_crate_tests` (`the_power_up_table`, the CR19 test on layers, `no_section_no_table`). |
| 5, game-wide rows (switch-off startup values: the reset values, independent of S-0520's original start table) and open question 11 | satisfied | answers Q-1034: with the switch off, an entry that no layer mentions has weight 0 and no animation. The code (`crate_import`) and its test already did this; the comments now cite S-0006 revision 18. The Q-1034 row stays in the interim table because S-0006 revision 20 (on main, newer than this step) reverses the answer to S-0520's built-in start table; it is retired when that revision is taken in. |
| Measurement scenarios (CR19 pointer), list of unmeasured keys | (c) | |

## S-0203 revision 33

| Claim | Class | Result |
|---|---|---|
| 7 (immediate target-change effects: a strike cancels when the change applies, even after the designator's turn; a target lost in a later object's turn cancels in that tick; an order in row 8 cancels in row 8) | satisfied | `strike::sweep` after each applied order and each object's turn (S-0519 revision 3 claim 19): `an_order_changing_the_target_cancels_in_its_own_tick`, `a_change_and_back_still_cancels`. The target lost in a later object's turn is the sweep after that turn (coded, not separately tested, as in the S-0519 ledger). |

## S-0414 revision 14

| Claim | Class | Result |
|---|---|---|
| 5 (permanent control: hit when the current owner is defeated, not hit when only the former owner is; no pre-domination owner is remembered) | satisfied | `econ::victory::destroy_all` judges `owner == house`; there is no controlling unit to remember (S-0601 is not built). New test `claim_5_permanent_control_follows_the_current_owner` (`claims_s0414.rs`). The controlling-unit rows stay a seam. |
| Preamble pointer to S-0601 claim 14 | pointer | |

## S-0524 revision 9

| Claim | Class | Result |
|---|---|---|
| 4 and the I1 row for code 108 (a number slot of exactly 20 stores 255; every other input stores its reduction modulo 256, even when that is 20; stored 0 to 18 fixed, 19 to 255 random) and scenario C4 | satisfied | `crates::place_trigger`. The old test only compared the value clamped to 19; new test `trigger_crates_store_the_exact_reduction_of_the_number_slot` asserts the stored overlay data value for inputs 19, 20, 21, 255, 256, 276, -1, 5 and 18 (`claims_s0520.rs`). |

## S-0614 revision 8

| Claim | Class | Result |
|---|---|---|
| 1, 3, 4, state table (attachment and the failed latch happen only when attachment is the special hit selected by S-0514 claim 20; takeover, planting and charging suppress it even when they cannot succeed; selected attachment suppresses erasure even when latching fails; the firing-time departure and the latch block are unchanged) and P14 | satisfied | `parasite::carrier_ended` reads `special_of`. New tests `p14_a_higher_row_suppresses_attachment` (departure, latch block, no attachment, no damage) and `p14_selected_attachment_outranks_erasure` (`claims_s0614.rs`); `selected_attachment_suppresses_erasure` (`claims_s0514_r5.rs`). |
| What becomes of the carried parasite when a higher row suppresses attachment | open | the revision does not say (claim 4 is stated "with attachment selected" and P14 expects only "no fallback attachment"): Q-1032 stays open with its interim rule (the parasite fails to latch and returns). |

## S-0601 revision 5 (M7, not built)

Not implemented. Compared against the claims of specs we built: claim 14's query (permanent control supplies the current owner) is the S-0414 claim 5 row above; claim 8 gains a measurable backing (MC12) and claim 18 a measured ring height (MC13), both for the M7 effect; claim 5, the takeover's place in the special-hit table, is unchanged, so S-0514 claim 20's pointer (`Special::Takeover`, a seam that selects and suppresses the lower rows without an effect) stands.

## Results of the runs

See the M6c ledger row 1 in [docs/development.md](../development.md).
