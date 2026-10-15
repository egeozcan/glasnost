# Walls and gates claim ledger (M6b)

S-0112 claim 7 (wall connection bits, the tower's departure hits, map wall buildings), S-0210
claims 8, 13 and 14 (crushing walls, wall data, wall owners), S-0215 claim 35 (the gate's door),
S-0108 claims 2, 4 and 5 (gate frames), S-0406 claims 8 to 11 and S-0408 claim 12 where they
cover walls, and S-0417 (no wall or gate claims).

Tests: `crates/deterministic/sim/tests/claims_s0112.rs`, `claims_s0210.rs`, `claims_s0406.rs`,
`claims_s0408.rs`, `s0215_gate_*` in `claims_orders.rs`, `crush_sounds` in
`crates/audio/src/events_tests.rs`, `moving_gate_frames` in `crates/app/src/mapview/unload.rs`,
`damaged_gate_body_frame` in `crates/app/src/mapview/tests.rs`, and
`moving_gate_draws_its_door_frame` and `overlays_changed_in_play_are_redrawn` in
`crates/app/src/mapview/live_tests.rs`.
Code: `crates/deterministic/sim/src/walls.rs` (bits), `combat::area::wall_hit` and `wall_success`
(S-0303 F4), `crush.rs`, `objects::remove` (tower departure), `objects::place_map_objects` (map
wall buildings), `econ::placement` (claims 8 to 11), `econ::sell` (wall sale), `process::gate_open`
and `behaviour::Door` (claim 35), `crates/app/src/mapview/build.rs` (`building_items`,
`live_overlays`), `crates/app/src/mapview/live.rs` (`redraw`), `crates/app/src/mapview/unload.rs`
(`gate_frame`) and `crates/audio/src/events.rs`.

| claim | status | test names or reason |
|---|---|---|
| S-0112 claim 7 (connection bits and when they are recomputed) | tested | `claim_7_same_type_bits`, `claim_7_gate_and_tower`, `claim_7_fixed_cells_switch` (`quirks.gate_tower_fixed_cells`) |
| S-0112 claim 7 (a destroyed tower's four hits of 200) | tested | `claim_7_destroyed_tower_hits_its_walls` (N, E, S, W of the top-left cell, any wall type, stage 0 only, bits recomputed first), `claim_7_tower_removed_alive_deals_no_hits` (a sold tower is the open question: no hits, `objects::remove` checks health 0) |
| S-0112 claim 7 (initial state: a map wall building is placed as a wall built in play) | tested | `claim_7_map_wall_building_becomes_its_overlay` (stage 0, no building remains, bits recomputed). The owner is then S-0210 claim 14 (a)'s (Q-0684 answered). |
| S-0210 claim 4 (crush sounds) | tested | `crushing_plays_the_crush_sounds` (sim events), audio `crush_sounds` (the crushed type's and the overlay's sounds at the cell) |
| S-0210 claim 8 (crushing overlays and walls) | tested | `a_wall_is_crushed_only_by_a_crusher_all_crusher`, `a_wall_at_stage_d_minus_two_hits_its_undamaged_neighbours` (the neighbour hits are now dealt through S-0303 F4: stages checked), `a_weak_wall_hit_always_succeeds_without_a_draw`, `other_stages_remove_the_wall_without_hits`, `a_crushable_overlay_stays_and_ends_the_slowdown`; the view redraws the cell (`overlays_changed_in_play_are_redrawn`) |
| S-0210 claim 13 (D and S, defaults 1 and 1) | tested | `wall_defaults_are_one_stage_and_strength_one`, `the_imported_crush_rules` |
| S-0210 claim 14 (wall owners) | tested | `map_walls_take_the_owner_of_the_nearest_qualifying_building`, `a_wall_far_from_every_building_still_has_an_owner`, `equally_near_buildings_the_first_created_wins`, `a_crushed_wall_loses_its_owner`. A wall from a map wall building is then re-owned by (a), even to no owner (Q-0684 answered); a dropped human-owned entry places no wall. |
| S-0215 claim 35 (the door and the *open* behaviour) | tested | `s0215_gate_starts_closed_and_opens_on_demand`, `s0215_gate_open_and_close_cycle`, `s0215_gate_door_rests_before_the_processing`, `s0215_gate_stays_open_while_occupied`, `s0215_gate_reverse_snaps` (d2 off: the reopening runs over *T*<sub>d</sub> from progress *T*<sub>d</sub> − *a*), `s0215_open_without_a_gate`, `s0215_guard_stray_and_gate_times` (built before M6b) |
| S-0108 claim 2 (a stationary gate: 0, damaged G + 1; the occupiable rule first) | tested | `damaged_gate_body_frame`, `moving_gate_draws_its_door_frame` (at rest) |
| S-0108 claim 4 (moving gate: p, damaged p + G + 1; no clamp, bib, unloading or under-door image) | tested | `moving_gate_frames` (p, the damaged offset, at rest open G − 1, S-0215 claim 35 d3's zero-time motion), `moving_gate_draws_its_door_frame` (drawn frame, no bib, beyond the claim 3 clamp, at rest open). The door position *p* is S-0215 claim 35 d1's (Q-0685 answered); the special image is drawn only while *open* is current (`Live` passes the door only then). The map's gates are drawn live so that their door shows. |
| S-0108 claim 4 (build-up and selling run backwards) | seam | not drawn: the live view draws no build-up or selling frames for any building yet (placed buildings appear complete). The reversal goes in `building_items` when build-up drawing is added. |
| S-0108 claim 5 (a gate's build-up has G + 1 frames) | tested | `claim_15_gate_range` in `claims_s0406.rs` (timing) |
| S-0406 claim 8 (wall exceptions to the cell test) | tested | `claim_8_wall_exceptions`, `quirk_wall_exception_skips_ground_off` |
| S-0406 claim 9 (what placement removes) | tested | `claim_9_tower_removes_wall`, `claim_9_wall_without_builder_stays`, `claim_9_gate_row` (each foundation cell's own wall and own laser-fence building; no refund or sound) |
| S-0406 claim 10 (a wall type becomes its overlay) | tested | `claim_10_wall_becomes_overlay`, `claim_10_unsellable_builder_keeps_damaged_wall`; the view redraws the new wall |
| S-0406 claim 11 (wall fill) | tested | `claim_11_wall_fill`, `claim_11_fill_stops_at_failing_cell` |
| S-0408 claim 12 (selling a wall) | tested | `claim_12_wall_sale`, `claim_12_unsellable_builder`, `claim_12_non_player_owner`, `quirk_wall_sale_no_refund_off` |
| S-0417 | none | no claim of S-0417 covers walls or gates |
| Body tilt of a gate (presentation) | seam | the renderer draws no building tilt |
