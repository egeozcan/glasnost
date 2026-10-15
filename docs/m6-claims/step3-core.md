# M6b step 3 (core): spec revisions since 6a1cc2d

Classification of the changed claims of the specs below. Buckets: **(a)** changes behaviour the
engine implements (implemented, with a test); **(a, already done)** the code already did what the
new text says (a test is added where none existed); **(b)** a pointer into a spec or locomotor that
is not built (a seam); **(c)** editorial. Front-matter revision and date changes are (c) and are not
repeated.

Requests answered and retired: Q-0760, Q-0761, Q-0762, Q-0763, Q-0764, Q-0765 (S-0524) and Q-0799
(S-0105 claim 40). Requests filed: **Q-0840** (ready mark of a restored building) and **Q-0841**
(an aircraft "marked on a bridge deck"). Q-0738 and Q-0741 belong to the superweapon area and were
not touched; Q-0751 (drop shot) was retired on main by the superweapon work.

Saved state: no new field. The restored building's construction state and the re-placed units use
existing fields. Rules schema: no change. The save format number and the goldens are not touched.

## S-0524 Multiplayer trigger completion (rev 4 to 7) and S-0011 (rev 20 to 21)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 5 (undefined marker) | (a) | Still five damages, centred at (128, 128, 0), no direct bridge attempts. Retires Q-0760's no-effect. | `mp_triggers::number_waypoint`, `marker_point`, `demolition_blast`; `claim5_undefined_marker_acts_at_the_stand_in_point`, `claim5_defined_marker_uses_its_own_point`, `claim5_invalid_number_has_no_effect` |
| 6 (undefined marker) | (a) | The flash is requested at (128, 128, 0) under the same gates. | `light_flash`; `claim6_undefined_marker_flashes_at_the_stand_in_point` |
| 8 (shrink) | (a, already done) | Nothing is removed, moved, stopped or switched off; power, defeat counts and attack orders unchanged. Retires Q-0761. | `resize_local_area` (comment); `claim8_shrink_leaves_excluded_objects_alone`, `claim8_unit_outside_is_still_a_target` |
| 9 (record, order, covering cell) | (a, already done) | Top-left cells, creation order, any foundation cell of the right type and house heals. Retires Q-0763. | `record_starting_buildings`, `holder`; `claim9_restoration_follows_creation_order`, `claim9_each_action_restores_only_its_house`, `claim9_covering_cell_heals_a_shifted_building` |
| 13 (replacement state, displacement) | (a) | Fresh object under ordinary placement (`begin_construction`), switch on; ground-layer foundation occupants (a landed aircraft too; not deck or airborne) lifted in foundation-cell order, re-placed in reverse by the 1 x 1 nearby search (class, zone normal, no reach, deck off, no height or overlay test, bridges allowed, old cell as reference), at the cell centre surface point, infantry not packed, facing 0, ordinary fall-back; no result removes the unit; a failed building returns occupants at their points facing 0. Retires Q-0764. | `mp_triggers::replace`, `occupants`, `place_near`, `put_back`; `claim13_replacement_is_a_fresh_building_under_construction`, `claim13_displaced_occupants_are_placed_by_the_nearby_search`, `claim13_no_nearby_cell_removes_the_unit`, `claim13_failed_placement_returns_occupants_facing_zero`. Ready-mark ambiguity: **Q-0840**. Same-cell lifting ties are the listing order (unmeasured). |
| 10 (ownership change raises nothing; no order of carry-over) | (a, already done) | `capture_seam` makes no ownership raise. Retires Q-0765. | `claim10_ownership_change_raises_no_tag` (with and without carry-over), `claim10_capture_*` |
| 11 (vehicle-like building) | (b) seam, tested | New `vehicle_like_entry_seam` for S-0509 claim 17 (one entry raise, unit destroyed raise on the shared tag). | `claim11_vehicle_like_building_entry` |
| 12 (recycling, retained objects) | (b) seam, tested | `consuming_entry_seam` removes passengers silently (S-0603 claim 10); recycling itself is not built. | `claim12_consumed_entrants_passengers_get_no_raise` |
| 14 (boarding and service table) | (a) / (b) | Infantry boarding a transport raises once on arrival before the acceptance test (a). Vehicle boarding, vehicle docking, refinery docking and pad landing raise nothing (a, already done, tested in the owning files). Hospital, training, absorber and garrison entries are seams (`retaining_entry_seam`). Retires Q-0762. | `transport::arrival`; tests below under S-0507, S-0314, S-0402, S-0501 |
| Terms, Summary, I1a, Edge cases, measurement text | (c) | Restate the above. | none |
| S-0011 I4 row 109 | (b) | The curtain action belongs to S-0513 claim 22 (implemented on main; this branch merges it). | `action_coverage` |

## S-0507 Transports (rev 6 to 7)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 6 (notifications) | (a) | One entry raise for infantry at the arrival, before the final test (a refused boarding already notified); none for vehicles; none on the order; none on unloading; no passenger destruction raise. | `transport::arrival`; `claim_6_infantry_boarding_raises_the_transport_tag_once` (vehicle and landed air transport, unload and re-board), `claim_6_vehicle_boarding_raises_nothing`, `claim_6_refused_boarding_has_already_notified` in `claims_s0507.rs` |
| 13 table ("fails the landing test" for F7) | (c) | Wording. | none |

## S-0314 Healing and repair (rev 20 to 21)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 7 (hospital entry raises once, retained) | (b) | Hospitals are not built; `retaining_entry_seam`. | `claim12_retaining_entry` |
| 9 and 10 (leaving-stage warp turn skips healing) | (a, already done) | The Q-0738 interim (owned by the superweapon topic) makes a warp update the whole turn, so no healing runs. | `claims_9_and_10_leaving_stage_skips_healing` in `claims_s0314_s0315.rs` |
| 17 (docking raises no tag) | (a, already done) | | `claim_17_service_docking_raises_no_tag` in `claims_s0314_depot.rs` |
| 17 and 18 (cell-goal cleanup, building destination, exact-point guard) | (a, already done) | `drive::goal_cleanup` (step 2). | `claims_s0207_goal.rs`, `claims_s0314_depot.rs` |

## S-0402 Harvest cycle (rev 23 to 24)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 10 (tag notifications) | (a, already done) | Refinery docking raises neither tag. | `claim_10_refinery_docking_raises_no_tag` in `claims_s0402.rs` |

## S-0312 Death and destruction (rev 27 to 28)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 29 (latch-off alternatives) | (a, already done) | Latch off with the extra group: two firings; common group only: one. | `triggers::raise` latch clearing; `s0312_claim_29_latch_off_alternatives` in `tests/triggers.rs` |

## S-0504 Naval units (rev 16 to 17)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 1 (order-level cleanup) | (a, already done) | Ships use `drive::goal_cleanup`: same-cell move removed at the first update, final completion clears the point and the goal goes next tick; building goal and guard row as for vehicles. | `claim_1_ship_goal_cleanup` in `claims_s0504.rs` |

## S-0501 Aircraft (rev 13 to 14)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 12 to 15, 20, T1, AC12 (drop attempts in a run) | (a, already done) | `combat::fire::shot` handles a carrying aircraft's attempt as S-0522 claim 11 (superweapon work on main): no ordinary shot effects, refusal counts as a made shot for the pending round. | `claim11_*` in `claims_s0522.rs`; `claim_15_one_round_per_run` |
| 24 (pad raises nothing) | (a, already done) | | `claim_24_pad_docking_raises_no_tag` in `claims_s0501.rs` |

## S-0511 Bridges (rev 6 to 7)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| 10 (absent parachute release tick) | (b) | Pointer to S-0522 claim 13 (superweapon topic). The release-tick parachute recurrence differs from the no-parachute one only for a stored rate below the parachute floor. | none here |
| 11 (negative locomotion kinds) | (a, already done) | `blow::heads_to` counts only walk, hover, drive and ship. | `claim_11_other_locomotion_kinds_never_head_to_the_cell` |
| 17, 22, binding table, scenario wording | (c) | Backing and constants wording. | none |

## S-0105 Rendering of voxel models (rev 17 to 18, via Q-0799) and S-0102 (rev 18)

| Claim | Bucket | Change | Code, tests |
|---|---|---|---|
| S-0105 40 (aircraft shadow anchor) | (a) | Terrain height at the exact position, 60 pixels higher over a qualifying high bridge, whatever the flight height; every aircraft locomotor. Retires the Q-0799 interim. | `Mobile::shadow`, `live::shadow_anchor`, `build::vehicle_voxels`; `flying_aircraft_shadow_lies_on_the_ground`, `aircraft_shadow_anchor_bridge_qualification`. **Q-0841**. |
| S-0102 18 | (c) | Pointer to S-0105 claim 40. | none |

## Seams only (not built)

- **S-0502** (rev 3 to 4): the touchdown sensor update (S-0516 claims 13 and 22) and the corrected F1 speed text belong to the jumpjet locomotor (M6c). (b)
- **S-0509** (rev 4 to 5): no ownership-change raise on capture; `capture_seam` and `vehicle_like_entry_seam` hold the order for the capture when it is built. (b)
