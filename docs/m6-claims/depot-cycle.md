# Depot cycle claim ledger (M6b)

S-0314 claims 6 and 16 to 24 (the repair depot), S-0504 claims 7, 8, 12 and 14 (repair at
naval yards), S-0215 claim 29 (the ordinary depot protocol) and S-0214 table V value *n*.

Tests: `crates/deterministic/sim/tests/claims_s0314_depot.rs` (land depots and a land vehicle
factory with the repair flag), `claim_14_*` in `crates/deterministic/sim/tests/claims_s0504.rs`
(a ship at a naval yard), `depot_*` in `crates/audio/src/events_tests.rs` (announcements),
`depot_with_several_places_warns` in `crates/convert/src/sim_naval_tests.rs` (importer) and
`sends_damaged_vehicles_to_a_depot` in `crates/deterministic/ai/tests/ai.rs` (our AI).
Code: `crates/deterministic/sim/src/depot.rs`, with hooks in `targeting` (value *n*), `orders`
(claims 20 and 23), `process` (enter and repair processing), `combat::behave` (early guard
start), `tick::object_turn` (queued-entry retry), `path::heading_point` (docking point),
`econ::tethered`, `econ::sell` (sale), `combat::death` (destruction), `infantry::destination_assigned`
(locomotor switched on), `econ::release` (place positions; factory-unload contact kept apart
from depot contact) and `crates/audio/src/events.rs`.

| claim | status | test names or reason |
|---|---|---|
| S-0314 claim 6 (repair depot phases) | tested | `end_to_end_click_repair_and_release`, `claim_6_phases_and_timing` (D + K_u − 1, then K_u), `claim_6_done_at_settling_waits_one_step`, `claim_6_full_health_vehicle_is_sent_away_unpaid`, `claim_6_cannot_pay_and_low_funds`, `claim_6_sent_to_the_rally_point`, `claim_6_neutral_vehicle_returns_to_its_guard_spot`; audio `depot_repair_complete_announcement` (F7 kind *unit repaired*, 2 cells, 400 ticks, suppression), `depot_low_funds_announcement`. Partly: "its locomotor has finished settling" is not modelled (our driving locomotor has no settling state; a still vehicle counts as settled). The exit cell without a rally point is the Q-0682 interim. |
| S-0314 claim 16 (permission) | tested | `claim_16_permission` (allied owner, switched off, standing on the depot, selling, no damage or money gate), `claim_16_naval_match`. The build-up state is read as construction (`econ::placement::seen_behaviour`). |
| S-0314 claim 17 (contact before repair) | tested | `claim_17_contact_places`, `claim_17_two_places`, `claim_17_early_guard_start` (63 and 64 leptons), `end_to_end_click_repair_and_release` (facing 16384, repair and sleep queued). The tether moment is the Q-0680 interim. |
| S-0314 claim 18 (docking point) | tested | `claim_18_docking_point` (sole displacement, per place, vehicle-factory precedence, cell centre), `end_to_end_click_repair_and_release` (the vehicle ends on the docking cell), `s0504_claim_14_yard_repair`. A docking point off a cell centre ends at the cell's centre (Q-0680 interim). Foundation blocking stays S-0209 claim 5's (VP12); the tests use a depot without impassable rows. |
| S-0314 claim 19 (healing while sinking) | tested elsewhere | `claim_10_sinking_vessel_self_heals` in `claims_s0504.rs` (built in M6a). |
| S-0314 claim 20 (a new order) | tested | `claim_20_approaching_vehicle` (another order releases; repeated enter ignored), `claim_20_docked_vehicle_keeps_contact`, `claim_20_same_depot_enter_while_docked` (damaged: queued, then refused while standing on it; full health: lost at once). A stop order releases an approaching vehicle; a docked one ignores stop (S-0213 claim 8). |
| S-0314 claim 21 (moving away while docked) | partly | `claim_21_remote_repair` (working steps far from the depot), `claim_22_locomotor_switched_on_by_a_destination`. The settling hold (moving within 200 leptons: next tick) is built but not tested: no input moves a docked vehicle in the 3 ticks between waiting and settling. Moving at 200 leptons or more is the Q-0681 interim. The depot's release destination replaces the player's. |
| S-0314 claim 22 (sale, destruction, power) | tested | `claim_22_sale`, `claim_22_destruction_approaching`, `claim_22_docked_vehicle_left_asleep`, `claim_22_destroyed_vehicle_frees_its_place`, `claim_22_switched_off_keeps_servicing`, `claim_22_locomotor_switched_on_by_a_destination`, `claim_22_sale_leaves_the_locomotor_off`. Partly: the scatter request to a vehicle left without a destination is sent but not tested; a hovering vehicle's stranding is the seam `depot::locomotor_off` for S-0503 (the hovering locomotor is not built). Removal other than destruction leaves an approaching vehicle to its next *enter* processing, which is refused. |
| S-0314 claim 23 (queued entry) | tested | `claim_23_queued_entry` (event order, damaged waits with no destination and enters by itself, full health does not queue), `claim_23_later_order_clears_queued_entry`, `claim_17_contact_places` (a later click on a held depot is select). The retry's place in the tick is the Q-0683 interim. |
| S-0314 claim 24 (no automatic return) | tested | `claim_24_no_automatic_return`; our AI uses ordinary enter orders (`sends_damaged_vehicles_to_a_depot`). |
| S-0314 Edge cases, `quirks.depot_remote_repair` | tested | `claim_21_remote_repair` (on), `quirk_depot_remote_repair_off` |
| S-0314 Edge cases, `quirks.depot_loss_leaves_sleep` | tested | `claim_22_docked_vehicle_left_asleep` (on), `quirk_depot_loss_leaves_sleep_off` |
| S-0314 Edge cases, `quirks.depot_loss_strands_hover` | tested | `claim_22_sale_leaves_the_locomotor_off` (on), `quirk_depot_loss_strands_hover_off`; end to end with a hovering vehicle (S-0503): `sold_depot_strands_a_hover_end_to_end` (on: after the sold depot has left, a move order does not move it) and `sold_depot_frees_a_hover_with_the_quirk_off` (off: it moves). The test depot is 1 × 1: a hover heads for an object's position (S-0503 Terms), which on our 3 × 3 depot is not the docking cell, so it would dock there without being serviced (noted in the S-0507 report). |
| S-0314 Edge cases, several contact places | tested | `claim_17_two_places`, `claim_18_docking_point`; the lowest occupied place is serviced (`depot::serviced`), a change restarts at waiting; importer warning `depot_with_several_places_warns`. |
| S-0314 Import mapping (`UnitRepair`, `NumberOfDocks`, `DockingOffset`*i*, `WeaponsFactory`) | tested | already imported; `depot_with_several_places_warns`, `naval_fields`. The vehicle-factory precedence reads our factory kind *vehicle*. |
| S-0504 claim 7 (naval match) | tested | `claim_7_naval_match`, `claim_16_naval_match` (now asked by the depot's permission) |
| S-0504 claim 8 (waiting gate) | tested | `claim_8_waiting_gate`; in play `claim_14_ship_repaired_at_yard` (200 leptons for ships). |
| S-0504 claim 12 (yard docking point) | tested | `claim_12_yard_docking_point`, `claim_14_ship_repaired_at_yard` |
| S-0504 claim 14 (yard interruptions and queued entry) | tested | `claim_14_ship_repaired_at_yard`, `claim_14_docked_ship_keeps_contact_under_a_move`; the shared rules are S-0314 claims 20 to 23's, tested on land vehicles above. |
| S-0215 claim 29 (enter at repair buildings) | tested | `end_to_end_click_repair_and_release` (contact before arrival, docking point, facing, repair and sleep queued without an immediate start), `s0215_claim_29_refusal_while_approaching`, `claim_17_early_guard_start`. |
| S-0214 table V value *n* | partly | rows n1 (`claim_16_permission`), n3 (`end_to_end_click_repair_and_release`), n4 (`claim_17_contact_places`). Row n2 (recycling building) is not built: no building of ours recycles. Only repair depots give a docking answer. |
| S-0314 claim 7 (hospital) | not built | outside this part; no building heals infantry by entry yet. |
