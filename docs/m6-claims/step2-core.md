# M6b step 2 (core): spec revisions since dcf30dc

Classification of the changed claims in `git diff dcf30dc HEAD` of five specs. Buckets:
**(a)** changes behaviour the engine implements (implemented, with a test); **(a, already done)** the
committed code already did what the new text says (a test is added where none existed); **(b)** a
pointer into a locomotor, spec or milestone that is not built (a seam); **(c)** editorial, no
behavioural change.

Every file's front matter (revision, requests) changed; that is (c) and is not repeated. The
answered requests are Q-0795, Q-0796, Q-0797 and Q-0798. Q-0790 to Q-0794 were already fully
retired (no marker in `crates/` or in the interim tables). Request filed: **Q-0830**.

Saved state: `cloak::Generator::was_on` (`serde(default)`) is new. Rules schema: the new quirk
`quirks.cloak_generator_zero_radius_shutdown` (default on). Neither the save format number nor the
schema and the sim-gate goldens are regenerated here.

## S-0207 Ground vehicle movement (rev 27 to 28)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Terms (*Moving destination*) | (c) | Restates S-0314 claim 17's combined condition. | `drive::goal_present`, already as claim 25 (`claim_17_driving_movement_report`). |
| 10 (table intro wording) | (c) | "The earliest matching row, clamped"; same rule. | none |
| 25 (rows "Other track completion", closing paragraph) | (a, already done) | Standing in the goal cell without completing a track leaves the point to claim 26. | `drive::completion_clears_goal`, `claim_25_*` in `claims_s0207.rs`. |
| 26 first row (cell destination at rest) | (a) | Retires the Q-0797 interim. The first locomotion update without an active track removes a cell goal in the vehicle's cell (no search, tolerance, alignment, or behaviour condition), the point with it; with approach destinations the goal is abandoned and the vehicle falls back (S-0215 claim 13). | `drive::goal_cleanup` (called from `drive::tick` at the start of a tick without a track). `claim_26_same_cell_order_is_cleaned_up_by_the_first_update`, `claim_26_approach_destinations_fall_back` in `claims_s0207_goal.rs`. |
| 26 second row (building destination) | (a) | A building goal is not removed by cell equality. | `goal_cleanup` (`building` test). `claim_17_same_cell_enter_supplies_the_point` and `s0215_claim_29_entry_keeps_the_building_destination` in `claims_s0314_depot.rs`. |
| 26 third row (guard, exact point) | (a) | At rest in *guard*, world point equal to the point in x, y and z removes the goal, a building one too. | `goal_cleanup` (`guard_goal`). `claim_26_guard_row_removes_the_building_goal_at_the_exact_point`, `claim_26_guard_row_needs_exact_equality` (horizontal and height differences in the same cell). |
| 26 Final track | (a) | Completion clears only the locomotor point; the cell goal goes at the next tick's update. `arrive_or_rest` no longer removes the destination. | `claim_26_final_track_leaves_the_cell_goal_for_the_next_update`. |
| 26 Same-cell order from rest | (a) | Removed in the first update without a track, possibly the assignment tick; a same-cell *enter* to a depot keeps both destinations. | `claim_26_same_cell_order_is_cleaned_up_by_the_first_update`; `claim_17_same_cell_enter_supplies_the_point` (offset 400, point not equal to the world point). |
| 26, a destination naming a unit | (b) | The claim names only cells and buildings. | Interim: a unit destination is a cell destination (as before). **Q-0830**; row in `docs/development.md` (M6b depot cycle). |
| 22 chain-point arrival (named by S-0516 claim 22's table) | (b) | No curve chaining for ordinary vehicles; a passive vehicle's chain-point update is not built. | Seam (S-0207 claim 8 says ordinary vehicles never chain). |
| Constants table, D30, Cross-spec changes | (c) | Scenario and import row; no imported field. | none |

Test edits: `claims_s0207.rs` `start` (a path that returns to the start orders a far cell, since a
same-cell order is cleaned up at once); `claim_25_no_departure_cell_keeps_contact` sees the retained
depot destination.

## S-0301 Firing cycle (rev 15 to 16)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 9 (empty secondary attempt) | (a, already done) | The attempt does nothing: no shot, burst, rearm, ammunition or draw, no firing-check answer. | `combat::fire::shot` returns before any change when the slot has no weapon; `s0301_claim_9_empty_secondary_attempt_does_nothing` (new: state and draws equal to the primary alone). |
| 9 (priority over animation delay; resolved-slot condition) | (a) | A passing gate's pair is immediate, also for an animation-delayed building. | `combat::behave::building_process` (CanFire arm); `s0301_claim_14_passing_gate_beats_animation_delay`. |
| 14 (upgrade gate priority) | (a) | Passing gate: pair at once, no delayed shot. Failed gate (quirk on): neither. Quirk off or bypassed gate: ordinary path with the delay. | same arm; `s0301_claim_14_failed_gate_and_bypass_with_animation_delay`. Retires the Q-0798 interim of the old branch. |
| State table rows, import table row (`IsAnimDelayedFire`, `Primary`, `Secondary`…), F17 | (c) | Restate the above; F17 is a scenario. | none |

## S-0215 Behaviours (rev 43 to 44)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Preface (Q-0797, Q-0798 answers), import mapping paragraph and rows | (c) | Pointers. | none |
| 22 Vehicle goal cleanup | (a, already done) | *Move* processing removes no destination for cell equality; the cleanup is the locomotion's (S-0207 claim 26) and does not reset the timer; the next due processing with no destination falls back with 1 tick. | `process.rs` unchanged. `s0215_claim_22_cleanup_is_the_locomotions_and_leaves_the_timer` (new). |
| 29 Goal-cell equality | (a, already done) | An accepted entry keeps the building destination at the docking cell, through due processings and the arrival exchange (queued *sleep* is no removal). | `depot::process_enter`, `depot::movement_arrival` do not touch it; `s0215_claim_29_entry_keeps_the_building_destination` (new) and the changed `claim_17_same_cell_enter_supplies_the_point`. |
| 32 Upgrade weapon gate | (a) | Applies only with a nonzero upgrade count and an occupied position 1, tests that type only (never the first occupied higher position or the latest installation). | `combat::weapon::gate_upgrade`, `combat::behave::upgrade_gate`; `s0406_claim_12_level_skip_bypasses_the_gate`, `s0215_claim_32_upgrade_gate_predicate`. |
| 32 Animation-delay priority; quirk-off wording | (a) | See S-0301 claim 14. | see S-0301 |
| B36, B43, scenario edits, cross-spec changes | (c) | Scenarios. | none |

## S-0406 Building placement (rev 8 to 9)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 12 (sale text) | (a, already done) | S-0408 claim 5 sells the position named by the count. | `econ::sell`; existing S-0408 tests. |
| 12 Upgrade precedence: no compaction, replacement at *c*′ | (a, already done) | A successful installation replaces the type in position *c*′. | `econ::placement::add_upgrade`; `s0406_claim_12_count_zero_retains_positions_without_effect` (new: replacement). |
| 12 weapon provider order | (a) | Positions 1 through *c* (at most 3), increasing, empties skipped; count 0 none; count above 3 uses 1 to 3. | `combat::weapon::installed_upgrades`; `s0308_claim_12_upgrades_supply_weapons`, `s0406_claim_12_count_zero_retains_positions_without_effect`, `s0406_claim_12_count_above_three_has_three_positions`, `s0406_claim_12_level_skip_bypasses_the_gate`. |
| 12 attack gate | (a) | Position 1 only, nonzero count. | see S-0215 claim 32. |
| 12 count above 3 (bad-data departure, wall tower) | (a, already done) | The existing departure; the search is bounded to three positions. | `add_upgrade` (positions 1 to 3 only). |
| Import table rows, PL16, Changes for other specs | (c) | The pending S-0308 text change is noted in `docs/m4-claims/S-0308.md`. | none |

`turreted` and the power sums keep reading every recorded position (S-0215 Terms and S-0405 claims 1
and 2 did not change).

## S-0516 Cloak and stealth (rev 7 to 8)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Terms (*Cell-arrival update*) | (c) | Adds warp and fall landing; placement alone is distinct. | see claim 22. |
| 13 (all rows of claim 22 use the test; flight and rocket have none) | (a) | Adjacency test at each source of claim 22. | `claim_13_teleport_arrival_reveals_a_cloaked_jumper`, `claim_22_warp_member_disc_transfers_at_the_arrival_effects`, `claim_22_fall_landing_is_a_cell_arrival_update`. |
| 20 Growth, Pending changes (table) | (a) | A new generator is idle at radius 0; edge-triggered on the recorded operation state; no automatic recovery; pending growth has no power gate. Retires the Q-0796 interim. | `cloak::field::turn`, `Generator::was_on`, `placed`; `claims_s0516_phases.rs`: `claim_20_new_generator_is_idle_at_radius_zero`, `claim_20_partial_shrink_reverses_on_restored_operation`, `claim_20_rows_without_a_phase_change`, `claim_20_radius_zero_loss_of_operation_changes_no_phase`, `claim_20_no_automatic_recovery`. |
| 20 Shrinking (phase-ending tests) | (a, already done) | Growth ends on the update after reaching the type radius, shrinking the update after 0. | `field::turn`; `claim_20_growth_shrink_ownership`. |
| 20 Manual power-off | (a) | Accepted power-off makes shrinking pending even at radius 0; a radius of 0 becomes the type radius, no coverage added. New `quirks.cloak_generator_zero_radius_shutdown`. | `field::switched_off` from `econ::power::switch`; `claim_20_manual_power_off_at_radius_zero` (quirk on and off). |
| 20 Immediate collapse | (a) | The restart test runs also at type radius 0 (reach 16 square cells); capture grows only if in operation. | `field::restart_neighbours`, `captured`; `claim_20_immediate_collapse_at_type_radius_zero`, `claim_20_capture_of_an_unpowered_generator`. |
| 22 table: driving or ship (chain-point arrival) | (b) | Passive chaining is not built (see S-0207). | seam |
| 22 table: teleporting locomotor | (a) | In the jump tick, after position and layer placement and a vehicle's reporting and crushing, before the entered-by raise. | `teleport::jump`; `claim_22_teleport_disc_moves_in_the_jump_tick`. |
| 22 table: jumpjet touchdown, underground, mechanical walking, drop-pod descent | (b) | Unbuilt locomotors (S-0502 and others; `Locomotor::None`). | seam; `claim_22_unbuilt_locomotors_supply_no_update_from_the_cloak_turn` (nothing moves the disc). `arrival_source` and the cloak-turn interim are removed. |
| 22 table: rocket flight | (a) | No update; the disc stays; with the quirk off a crossing moves it (no adjacency). | `rocket::tick` calls `flight_cell_crossed`; `s0516_claim_22_rocket_flight_moves_no_sensor_disc` in `claims_s0506.rs`. |
| 22 Entering play (unloading, put-down, parachute release) | (a, already done) | The disc is added by `entered_map`, with no adjacency test. | `cloak::machine::entered_map`; `claim_22_entering_play_adds_the_disc_without_an_adjacency_test`. A vehicle-lifting carrier's put-down is not built. |
| 22 Teleport arrival consequences | (a) | Sensor effects after arrival reporting and crushing, before the tag raise and crate pickup (crates are S-0520's seam). | `teleport::jump` (call placed before `objects::arrival_raise`). |
| 22 Superweapon teleport | (a) | The disc stays at the old cell through departure, retries and appearance; transfer and adjacency at the arrival-effects update, survivors only. | `superweapons::warp::lift_off` (disc kept), `warp::arrive`; `claim_22_warp_member_disc_transfers_at_the_arrival_effects`. |
| 22 Drops and unloads: completed fall landing | (a) | A cell-arrival update after falling ends, before the parachute detaches. | `fall::turn`; `claim_22_fall_landing_is_a_cell_arrival_update`. |
| Quirks (`flying_sensor_disc_stationary` extended to rocket and airborne jumpjet flight) | (a) / (b) | Rocket done; an airborne jumpjet is unbuilt. | see above |
| CS20, CS21, import table | (c) | Scenarios; no imported key. | none |
