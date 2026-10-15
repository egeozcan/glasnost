# Teleport movement claim ledger (M6b)

S-0207 claims 20 to 22 and F9 (the teleporting locomotor's own move) and S-0208 claims 19 and 20
(a teleporting infantry unit's landing point and placement clearing).

Most of the locomotor was built before M6b (`glasnost_sim::teleport`). M6b closes what play
needed: the warp animations reach the view (`combat::effects::anim`), the departure and arrival
sounds are events and play (`Sound::ChronoOut`, `Sound::ChronoIn`), and a vehicle's landing has
the arrival effects of reaching a track target (arrival report and cell-centre crushing before the
entered-by raise). A click on a cell with a teleporting unit selected gives a move, and the unit
jumps on its next movement update.

Tests: `crates/deterministic/sim/tests/claims_s0207_teleport.rs`, `claims_s0208_teleport.rs`,
`teleport_*` in `claims_arrival_raise.rs`, `claim_12_superweapon_locomotor_and_jump` in
`claims_s0614.rs`, `enclosed_cell_and_teleporter` in `targeting.rs` and `chrono_sounds` in
`crates/audio/src/events_tests.rs`. Code: `crates/deterministic/sim/src/teleport.rs`, with hooks
in `tick` (movement update), `orders` (destination taken), `objects::remove` (claim 20 k),
`econ::release` (temporary drive, claim 21) and `crates/audio/src/events.rs`.

| claim | status | test names or reason |
|---|---|---|
| S-0207 claim 20 (a) (taking a destination, the mark moved at once, dropped while phasing in or held) | tested | `claim_20_jump_timing_and_effects`, `claim_20_orders_while_phasing_in_are_lost`, `claim_20_held_object_drops_the_destination` (new), `claim_20_click_move_teleports_in_play` (new: the play path). The electromagnetic pulse and *being erased* have no state yet (S-0511, S-0514 out of scope): seam in `take_destination`. The S-0517 claim 9 deploy restriction waits for S-0517. |
| S-0207 claim 20 (b) (the landing point of a non-infantry object) | tested | `claim_20_occupied_destination`, `claim_20_landing_pick_at_fixation`, `claim_20_high_bridge_entry_is_asked_at_deck_level`, `claim_20_aircraft_moves_its_object_present_mark`. The superweapon after-effect hold is a seam (superweapons, S-06xx). |
| S-0207 claim 20 (c) (infantry: S-0208 claims 19 and 20) | tested | see the S-0208 rows |
| S-0207 claim 20 (d) (when it jumps) | tested | `claim_20_jump_timing_and_effects`, `claim_20_own_cell_destination`, `claim_20_click_move_teleports_in_play` |
| S-0207 claim 20 (e) (what the jump does) | tested | `claim_20_targets_drop_the_jumper`, `claim_20_jump_reverts_overridden_attackers`, `claim_20_click_move_teleports_in_play` (new: warp, departure sound, arrival sound, warp, in that order and at those points), `claim_20_crusher_landing_crushes` (new: a vehicle's arrival effects), `claim_20_vehicle_landing_on_a_high_bridge`, `claim_20_aircraft_*`, `claim_12_superweapon_locomotor_and_jump` (parasite exit), audio `chrono_sounds` (new). Partly: the projectile retarget of a shot in flight (S-0306 claim 17) is built but has no jump test; crate collection at *L* is a seam (crates, S-0520); the airstrike target-retention exception is a seam (S-0519). |
| S-0207 claim 20 (f) (phasing in) and F9 | tested | `claim_20_jump_timing_and_effects`, `claim_20_harvester_does_not_phase_in`, `claim_20_draw_at_the_end_of_phasing_in`, `claim_20_end_of_phasing_in_draws_with_the_search_gates`, `f9_phasing_in_delay` |
| S-0207 claim 20 (g) (successive jumps) | tested | `claim_20_harvester_does_not_phase_in` |
| S-0207 claim 20 (h) (a new destination before the jump) | tested | `claim_20_landing_pick_at_fixation`, `claim_20_aircraft_next_jump_from_the_deck` |
| S-0207 claim 20 (i) (random draws) | tested | `claim_20_draw_at_the_end_of_phasing_in`, `claim_19_one_draw_for_a_centre_destination`; the warp animations take their creation draws through `combat::effects::anim` |
| S-0207 claim 20 (j) (other queries) | tested | `claim_20j_body_turns_at_the_turn_rate` (new: the same facings as a driving twin), `claim_20_crusher_landing_crushes` (new: crushing unchanged) |
| S-0207 claim 20 (k) (removal with a pending or remembered landing point) | tested | `claim_20k_vehicle_removal_rows`, `claim_20_aircraft_removed_before_the_jump`, `claim_20k_switch_off_clears_the_pending_reservation` |
| S-0207 Edge cases (`quirks.teleport_vehicle_ground_reservation`, `quirks.teleport_aircraft_ground_reservation`) | tested | `claim_20_vehicle_ground_reservation_switch_off`, `claim_20_aircraft_ground_reservation_switch_off` |
| S-0207 claim 21 (after temporary driving) | tested | `temporary_drive_then_own_locomotor_back`, `claim_21_restoration_retains_the_teleporting_state`, `claim_21_previous_deck_landing` |
| S-0207 claim 22 (arrival timing) | tested | `teleport_landing_raises_in_the_jump_tick`, `teleport_passes_no_intermediate_cell`, `teleport_onto_a_bridge_deck_raises`; the vehicle's arrival report and crushing before the raise: `claim_20_crusher_landing_crushes` (new) |
| S-0208 claim 19 (an infantry unit's landing point) | tested | `claim_19_cell_destination_lands_on_a_side_position`, `claim_19_full_cell_refuses_without_a_fallback`, `claim_19_vehicle_cell_refuses`, `claim_19_negative_level_entry`, `claim_19_negative_level_entry_switch_off`, `claim_19_one_draw_for_a_centre_destination`, `claim_19_enter_destination_ignores_occupants`, `claim_19_object_destination_searches_nearby`, `claim_19_refused_replacement_restores_the_present_entry`, `claim_19_low_departure_onto_a_high_bridge`, `claim_19_later_choice_clears_by_actual_position`, `claim_19_refused_choice_after_the_jump_marks_the_deck`, `claim_19_ground_reservation_switch_off` |
| S-0208 claim 20 (placement-time clearing) | tested | `claim_20_placement_clears_the_destination_cell`, `claim_20_ordinary_move_does_not_clear` |
