//! Quirk switches of the simulation, all defaulting to the original's
//! behaviour (PLAN.md 4.6, D6; CLAUDE.md). Each switch is named after the spec
//! that defines it, and its doc comment cites that spec and says what the
//! original does when it is on. Switches that act only in the importer or the
//! renderer live with those (see the test at the end of this file).
//!
//! Generated from a table; edit the table in the generator, not the fields.

use schemars::JsonSchema;
use serde::Deserialize;

fn on() -> bool {
    true
}

/// The quirk switches. Absent fields take the original's behaviour.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Quirks {
    /// Every floating-point result rounds toward zero (the original's program-wide mode); off: round to nearest, ties to even.
    // spec: S-0205 claim 5
    #[serde(default = "on")]
    pub chop_rounding: bool,
    /// Game speed reaches game state through speed-normalised delays and a chrono flash stepping in frozen cycles.
    // spec: S-0201 claim 11; glossary G3
    #[serde(default = "on")]
    pub speed_normalised_delays: bool,
    /// A building body range's speed normalisation follows two rules: an immediate entry needs the art's normalised mark and excludes construction; a deferred entry and every wrap ignore the mark and cover construction and idle only.
    // spec: S-0108 Edge cases (quirks.building_wrap_normalisation; the simulation's idle body range, S-0215 claim 11)
    #[serde(default = "on")]
    pub building_wrap_normalisation: bool,
    /// A chrono flash restores the order block in force when it started, so a block from game start or an earlier run outlasts it.
    // spec: S-0201 Edge cases
    #[serde(default = "on")]
    pub chrono_flash_keeps_order_block: bool,
    /// A ranged draw whose bounds differ by exactly 2147483647 takes no value and returns the lower bound plus 2147483648, wrapped.
    // spec: S-0202 claim 5
    #[serde(default = "on")]
    pub ranged_draw_half_span: bool,
    /// A latchable event of a repeating tag stays satisfied after its first satisfaction (only timer-position alias clearing undoes it); off: a trigger instance's latches are cleared each time it fires.
    // spec: S-0011 claim 9 and Edge cases
    #[serde(default = "on")]
    pub persistent_event_latch: bool,
    /// Event positions 33 and later share the latch of the position 32 lower, including clearing by aliased timer positions; off: every position has its own latch.
    // spec: S-0011 claim 9 and Edge cases
    #[serde(default = "on")]
    pub event_latch_aliasing: bool,
    /// An all-carriers tag with no carrier never fires; off: a carrier count of 0 counts like 1, so the tag fires once and is deleted.
    // spec: S-0011 claim 10 and Edge cases
    #[serde(default = "on")]
    pub all_carriers_without_carrier: bool,
    /// The entered-by value is read only as a country index, so player-position values never match; off: it is resolved like an action's house value.
    // spec: S-0011 claim 18 and Edge cases
    #[serde(default = "on")]
    pub entered_by_country_only: bool,
    /// A trigger owner written as <none> gives the first country of the country list; off: <none> names no owner country.
    // spec: S-0011 claim 5 and Edge cases
    #[serde(default = "on")]
    pub none_owner_is_first_country: bool,
    /// The type name of the type-exists and type-does-not-exist events is matched case-sensitively, with no no-type markers; off: S-0003 claim 14's case-insensitive matching and markers.
    // spec: S-0011 claim 21 and Edge cases
    #[serde(default = "on")]
    pub exact_trigger_type_names: bool,
    /// A vehicle's killing hit makes the extra destruction-raise group before the common group; off: only the common group, for every vehicle.
    // spec: S-0312 claim 29, S-0011 claim 19
    #[serde(default = "on")]
    pub vehicle_destruction_extra_raises: bool,
    /// The 'magnitude first' remainder keeps the negative sign of −2147483648.
    // spec: S-0202 Edge cases
    #[serde(default = "on")]
    pub random_magnitude_overflow: bool,
    /// Each counting removal during the turn pass costs one later object its turn, and an object that re-enters play takes a second turn.
    // spec: S-0203 claims 10 and 11
    #[serde(default = "on")]
    pub update_order_removal_skip: bool,
    /// Distances use the table square root; off: the exact integer square root.
    // spec: S-0204 claim 5
    #[serde(default = "on")]
    pub table_root_distance: bool,
    /// Directions follow the tabulated arctangent and the short facing scale; off: the exact angle.
    // spec: S-0204 claim 8
    #[serde(default = "on")]
    pub table_direction: bool,
    /// The building reduction applies only to the second object of a distance.
    // spec: S-0204 claim 7
    #[serde(default = "on")]
    pub one_sided_building_distance: bool,
    /// The table sine and cosine of a negative angle can land two steps short.
    // spec: S-0105 F1
    #[serde(default = "on")]
    pub table_trig_negative_bias: bool,
    /// A negative turn-rate field acts through its remainder modulo 256.
    // spec: S-0206 claim 6
    #[serde(default = "on")]
    pub turn_rate_negative_wrap: bool,
    /// A turret still turning at the end of the previous tick is not re-aimed by the target rows.
    // spec: S-0206 claim 9
    #[serde(default = "on")]
    pub turret_reaim_after_turn: bool,
    /// A spinning turret turns by the quantised steps of claim 11.
    // spec: S-0206 claim 11
    #[serde(default = "on")]
    pub turret_spin_quantised: bool,
    /// Infantry bodies turn at the fixed rate 127.
    // spec: S-0206 claim 15
    #[serde(default = "on")]
    pub infantry_fixed_turn_rate: bool,
    /// Setting a facing at once to its current interpolated value during a turn ends the turn at its target.
    // spec: S-0206 claim 5
    #[serde(default = "on")]
    pub set_facing_keeps_target: bool,
    /// A turret-less track-class body turn that passes exactly through its target's direction stops there.
    // spec: S-0206 claim 16
    #[serde(default = "on")]
    pub track_body_turn_stops_at_target: bool,
    /// Only passive vehicles chain two-cell curves.
    // spec: S-0207 claim 8
    #[serde(default = "on")]
    pub drive_curve_chaining_passive_only: bool,
    /// Within the slowdown distance the braking floor raises a slower vehicle to the floor.
    // spec: S-0207 claim 10
    #[serde(default = "on")]
    pub drive_brake_floor_raises: bool,
    /// The step towards the next track point is taken from the current, possibly interpolated, position.
    // spec: S-0207 claim 4
    #[serde(default = "on")]
    pub drive_subpoint_compounding: bool,
    /// A tick finishes at most one track if it began without one, two if it began on one.
    // spec: S-0207 claim 12
    #[serde(default = "on")]
    pub drive_track_end_cap: bool,
    /// A vehicle that comes to rest keeps its leftover progress budget for its next move; off: the budget is cleared at rest.
    // spec: S-0207 claim 3, Edge cases (quirks.drive_leftover_kept_at_rest)
    #[serde(default = "on")]
    pub drive_leftover_kept_at_rest: bool,
    /// A unit standing at the top position is recorded as taking the centre.
    // spec: S-0208 claim 2
    #[serde(default = "on")]
    pub top_position_counts_as_centre: bool,
    /// A prone unit whose type cannot crawl walks half as fast again.
    // spec: S-0208 claim 8
    #[serde(default = "on")]
    pub prone_noncrawler_speedup: bool,
    /// A fast walker can step past its walking target and swing back.
    // spec: S-0208 Edge cases
    #[serde(default = "on")]
    pub walker_overshoot: bool,
    /// A walker of speed 1 heading between east and south stalls.
    // spec: S-0208 claim 14
    #[serde(default = "on")]
    pub slow_walker_stall: bool,
    /// A walker's cell destination on a high-bridge cell aims at deck height (the centre's terrain height plus 414 leptons) whatever layer it arrives on; off: the arrival test uses the centre's height on the walker's arrival layer.
    // spec: S-0208 claim 18 and Edge cases
    #[serde(default = "on")]
    pub walker_bridge_destination_deck: bool,
    /// A teleporting infantry unit's landing entry check takes a cell at terrain level −6 or lower as clear without testing terrain or occupants; off: the ordinary landing entry check applies.
    // spec: S-0208 claim 19 and Edge cases
    #[serde(default = "on")]
    pub teleport_infantry_negative_level_entry: bool,
    /// A teleporting infantry unit's ground standing-position reservation is left behind by deck placement, and a later destination choice clears by its actual position and height; off: the reservation moves to the actual landing surface at the jump.
    // spec: S-0208 claim 19 and Edge cases
    #[serde(default = "on")]
    pub teleport_infantry_ground_reservation: bool,
    /// A teleporting vehicle reserves the ground record of its landing cell even on a high bridge, remembers that point for later clearing and keeps the ground reservation through deck placement, which also marks deck; off: reserve and clear the eventual landing layer only.
    // spec: S-0207 claim 20 a, e and Edge cases; S-0209 claim 1
    #[serde(default = "on")]
    pub teleport_vehicle_ground_reservation: bool,
    /// A teleporting aircraft reserves its landing cell's ground object-present flag, keeps it through deck placement and clears only actual deck occupancy on removal; off: reserve and clear only the eventual landing layer.
    // spec: S-0207 claim 20 a, e and Edge cases; S-0209 claim 1
    #[serde(default = "on")]
    pub teleport_aircraft_ground_reservation: bool,
    /// The hard-coded cow type's idle action can scatter it.
    // spec: S-0107 claim 14; S-0208
    #[serde(default = "on")]
    pub cow_idle: bool,
    /// Infantry facing groups are skewed 1024 facing units counter-clockwise.
    // spec: S-0107 claim 3
    #[serde(default = "on")]
    pub infantry_facing_sector_skew: bool,
    /// A substituted sequence kind is not checked for an empty frame count.
    // spec: S-0107 claim 7
    #[serde(default = "on")]
    pub unchecked_substitute: bool,
    /// The hover substitution needs a hover entry starting at a frame other than 0.
    // spec: S-0107 claim 7
    #[serde(default = "on")]
    pub hover_needs_nonzero_start: bool,
    /// The tread entry is used for no current sequence on water only, not beach.
    // spec: S-0107 claim 2
    #[serde(default = "on")]
    pub tread_default_water_only: bool,
    /// A completed sequence's declared direction is snapped on every tick it stays current.
    // spec: S-0107 claim 9
    #[serde(default = "on")]
    pub repeat_direction_snap: bool,
    /// The fire frame is chosen by stance and weapon, not by the playing kind.
    // spec: S-0107 claim 11
    #[serde(default = "on")]
    pub fire_frame_by_stance: bool,
    /// A prone unit stays in the attack state and can fire again without a new firing sequence.
    // spec: S-0107 claim 11
    #[serde(default = "on")]
    pub prone_attack_state_persists: bool,
    /// A fearless fraidycat still gets its fear set by a known attacker.
    // spec: S-0107 claim 16
    #[serde(default = "on")]
    pub fraidycat_fear_ignores_fearless: bool,
    /// A prone unit asks for get-up only when its current non-empty destination is assigned again; off: every non-empty destination assignment asks.
    // spec: S-0107 claim 15; S-0213 claim 21
    #[serde(default = "on")]
    pub prone_get_up_repeated_destination: bool,
    /// The falling air death has the generic completion follow-up.
    // spec: S-0107 claim 21
    #[serde(default = "on")]
    pub air_death_fall_generic_follow_up: bool,
    /// Only the type's first idle comment sound is used.
    // spec: S-0107 claim 14
    #[serde(default = "on")]
    pub first_comment_only: bool,
    /// At most two sound cues per sequence entry.
    // spec: S-0107 claim 22
    #[serde(default = "on")]
    pub two_sound_cues: bool,
    /// A death request turned down because the die-1 or die-2 entry has no image frames leaves the unit in play with its current sequence; off: the resulting death kind counts as completed on the death tick. Not consulted in M3, which has no death handling (combat, M4).
    // spec: S-0107 claim 17
    #[serde(default = "on")]
    pub refused_death_keeps_unit: bool,
    /// The infantry house clears when no side position stays taken.
    // spec: S-0209 Edge cases
    #[serde(default = "on")]
    pub occupancy_house_side_positions_only: bool,
    /// The infantry house stays that of the last infantry unit to mark, even after it left.
    // spec: S-0209 Edge cases
    #[serde(default = "on")]
    pub occupancy_house_stale: bool,
    /// Stationary friendly needs exactly three allied infantry without movement in progress.
    // spec: S-0209 claim 4
    #[serde(default = "on")]
    pub stationary_friendly_exactly_three: bool,
    /// An overriding occupant hides blockers listed after it.
    // spec: S-0209 claim 4
    #[serde(default = "on")]
    pub occupancy_override_hides_later_blockers: bool,
    /// A crusher reads obstacle outcomes as clear in cells whose overlay has registration index 0.
    // spec: S-0209 claim 5
    #[serde(default = "on")]
    pub crusher_overlay_index0_clear: bool,
    /// Row VR2 reads the first vehicle of the ground layer even when the deck layer counts.
    // spec: S-0209 claim 5
    #[serde(default = "on")]
    pub crusher_entry_ground_layer: bool,
    /// On a deck over uneven ground, objects come from the deck layer and the record from the ground layer.
    // spec: S-0209 Questions for the project lead (proposed)
    #[serde(default = "on")]
    pub cell_layer_split_on_uneven_deck: bool,
    /// A walker that meets an allied wall with no object listed takes the wall as its target.
    // spec: S-0209 Questions for the project lead (proposed)
    #[serde(default = "on")]
    pub walker_targets_allied_wall: bool,
    /// The omni-crusher flag alone makes no vehicle a crusher.
    // spec: S-0210 Edge cases
    #[serde(default = "on")]
    pub omnicrusher_needs_crusher: bool,
    /// The driving vehicle counts itself as a vehicle in the cell it drives into.
    // spec: S-0210 claim 9
    #[serde(default = "on")]
    pub crush_slowdown_counts_self: bool,
    /// The crush slowdown looks at the ground layer whichever layer the vehicle is in.
    // spec: S-0210 claim 9
    #[serde(default = "on")]
    pub crush_slowdown_ground_layer: bool,
    /// A crushed crushable overlay that is not a wall stays in its cell.
    // spec: S-0210 claim 8
    #[serde(default = "on")]
    pub crushable_overlay_stays: bool,
    /// The crush-slowdown state ends only when the vehicle next crushes an overlay.
    // spec: S-0210 claim 9
    #[serde(default = "on")]
    pub crush_slowdown_persists: bool,
    /// Ship and walking-vehicle crush slowdown is set by the overlay type of registration index 0.
    // spec: S-0210 claim 9
    #[serde(default = "on")]
    pub crush_slowdown_first_overlay: bool,
    /// The crushers of player houses never make the crush response: the auto-crush flag and the
    /// player-auto-crush rule have no effect.
    // spec: S-0210 claim 11 and Edge cases (default on: the original); S-0215 claim 39
    #[serde(default = "on")]
    pub auto_crush_computer_only: bool,
    /// The passability category judges every class by the wheel column.
    // spec: S-0211 claim 11
    #[serde(default = "on")]
    pub passability_wheel_column: bool,
    /// Laser-fence sections leave the passability category alone.
    // spec: S-0211 claim 11
    #[serde(default = "on")]
    pub passability_laser_fence_lost: bool,
    /// Behind a cliff, row 1 of claim 5 becomes rock whatever its land type.
    // spec: S-0211 claim 8
    #[serde(default = "on")]
    pub cliff_back_case_sets: bool,
    /// Restricted vehicles may use a deck only above overlays of registration indices 237 and 238.
    // spec: S-0211 claim 14
    #[serde(default = "on")]
    pub restricted_deck_overlay_indices: bool,
    /// An unset role's range still counts tile numbers from its unset start.
    // spec: S-0010 claim 5; S-0406 claim 3 (d) (the naval range, formerly S-0211 claim 13)
    #[serde(default = "on")]
    pub unset_role_range: bool,
    /// A role naming the first missing tileset starts at the tile count.
    // spec: S-0010 claim 4
    #[serde(default = "on")]
    pub role_one_past_end: bool,
    /// A tile of the ramp range or of the smooth-ramp range whose slope shape is 0 becomes the tile just before the ramp range.
    // spec: S-0010 claim 11
    #[serde(default = "on")]
    pub ramp_shape_zero: bool,
    /// The level base below level 0 is one lepton higher.
    // spec: S-0106 claim 3
    #[serde(default = "on")]
    pub negative_level_floor_raised: bool,
    /// A search that reaches the destination after exactly 10000 expansions fails.
    // spec: S-0212 claim 12
    #[serde(default = "on")]
    pub path_search_10000_failure: bool,
    /// A recorded search pair keeps its first cost and is never reopened.
    // spec: S-0212 claim 11
    #[serde(default = "on")]
    pub path_no_reopen: bool,
    /// Cells along cell x join one region across 2 or 3 height levels.
    // spec: S-0212 claim 3
    #[serde(default = "on")]
    pub region_join_height_asymmetry: bool,
    /// Flattening reads the threat value at the new steps' first cell.
    // spec: S-0212 claim 17
    #[serde(default = "on")]
    pub flatten_threat_start_cell: bool,
    /// Some straight runs open a V that flattening replaces.
    // spec: S-0212 claim 17
    #[serde(default = "on")]
    pub flatten_straight_run_v: bool,
    /// From the third leg of the detour pass on, the reference distance is
    /// taken to the step's end cell mirrored through the leg's first cell.
    // spec: S-0212 claim 18
    #[serde(default = "on")]
    pub detour_mirrored_reference: bool,
    /// After a tunnel step, the detour pass starts afresh at the cell north
    /// of the tunnel entrance.
    // spec: S-0212 claim 18
    #[serde(default = "on")]
    pub detour_after_tunnel_position: bool,
    /// Congestion marks follow the parity of each cell's count.
    // spec: S-0212 claim 19
    #[serde(default = "on")]
    pub path_congestion_toggle: bool,
    /// Whether the progress sector is split is decided at the probe cell.
    // spec: S-0212 claim 13a
    #[serde(default = "on")]
    pub corridor_split_probe: bool,
    /// A failed new coarse search after a corridor retry fails the path
    /// computation, with no further cell search.
    // spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback)
    #[serde(default = "on")]
    pub corridor_retry_no_fallback: bool,
    /// Cells of an unusable category belong to the free part.
    // spec: S-0212 claim 13a
    #[serde(default = "on")]
    pub corridor_free_part_unusable_cells: bool,
    /// The free part judges each cell at its own height level, not by the height rule.
    // spec: S-0212 claim 13a and Edge cases (quirks.corridor_free_part_ignores_height)
    #[serde(default = "on")]
    pub corridor_free_part_ignores_height: bool,
    /// Destination substitution judges heights around the ground of a bridge-cell destination.
    // spec: S-0212 claim 24
    #[serde(default = "on")]
    pub substitution_centre_ground_level: bool,
    /// Destination substitution measures to the destination cell at world height 0.
    // spec: S-0212 claim 24
    #[serde(default = "on")]
    pub substitution_distance_flat_destination: bool,
    /// A path recomputed as the last step begins keeps the old step and drops the new path's first.
    // spec: S-0212 claim 20
    #[serde(default = "on")]
    pub path_refresh_keeps_old_step: bool,
    /// The nearby-cell search examines its centre twice.
    // spec: S-0212 claim 25
    #[serde(default = "on")]
    pub nearby_centre_twice: bool,
    /// The close case reverses the occupant's facing by 32767, not 32768.
    // spec: S-0212 claim 27
    #[serde(default = "on")]
    pub opposite_facing_reversal: bool,
    /// A coarse route found at the pending limit still sets the sector's cost.
    // spec: S-0212 claim 13
    #[serde(default = "on")]
    pub coarse_limit_records_cost: bool,
    /// The exclusion of the cells behind a train holds for the search only.
    // spec: S-0212 claim 16
    #[serde(default = "on")]
    pub train_exclusion_search_only: bool,
    /// A stalled unit's re-path may lead through the cells of other stalled
    /// units, at the moving-blocker cost (the original). Off (our design, C7
    /// part 4): a unit stalled for the blockage delay in a deadlock (stalled
    /// units that wait on each other's cells), or waiting on one, first looks
    /// for a way that keeps off the cells of the deadlock's other units, and
    /// the computation's own searches follow when there is none; a step
    /// refused by a unit with a destination starts the blockage timer.
    // spec: S-0212 claims 8 and 22 (the mode-2 moving-blocker cost; the off position is our design, the lead's decision of 2026-10-29)
    #[serde(default = "on")]
    pub path_through_stalled_blockers: bool,
    /// A move whose destination cell fails the gate or the search is dropped
    /// (the original). Off (our design, C7 part 4): the computation first
    /// tries once more towards the nearest cell the unit can stand on in its
    /// own reach region.
    // spec: S-0212 claim 23 (the failed computation; the off position is our design, the lead's decision of 2026-10-29)
    #[serde(default = "on")]
    pub unreachable_move_dropped: bool,
    /// No unit of a deadlock gives way (the original). Off (our design, C7
    /// part 5): in a deadlock of stalled units (`path_through_stalled_blockers`
    /// off) with no way round, the units near it heading against the one
    /// with the lowest object id give way: each goes to the nearest free cell
    /// off the other side's way, waits there until the other side has
    /// arrived or passed it (or for a longest wait) and then resumes its
    /// order. A deadlock with a way round keeps C7 part 4's detour.
    // spec: S-0212 claims 8 and 22 (the mode-2 re-path through moving blockers; the off position is our design, the lead's decision of 2026-10-29)
    #[serde(default = "on")]
    pub deadlock_no_yield: bool,
    /// The rebuild of a house's threat records counts its own and its allies' objects.
    // spec: S-0212 claim 16a and Edge cases (quirks.threat_rebuild_counts_friends)
    #[serde(default = "on")]
    pub threat_rebuild_counts_friends: bool,
    /// A negative threat field takes its amount away from the threat records.
    // spec: S-0212 claim 16a and Edge cases (quirks.negative_threat_field)
    #[serde(default = "on")]
    pub negative_threat_field: bool,
    /// An aircraft's threat and corridor exemption stay where it was placed.
    // spec: S-0212 claims 13 and 16a and Edge cases (quirks.aircraft_threat_stays_put)
    #[serde(default = "on")]
    pub aircraft_threat_stays_put: bool,
    /// Each house's pending list holds at most 128 orders.
    // spec: S-0213 claim 2
    #[serde(default = "on")]
    pub pending_order_limit: bool,
    /// A detonate order is issued in the planter's house's name.
    // spec: S-0213 claim 11
    #[serde(default = "on")]
    pub detonate_in_planter_name: bool,
    /// The last accepted cheer tick carries over between games of a session.
    // spec: S-0213 claim 19
    #[serde(default = "on")]
    pub cheer_memory_spans_session: bool,
    /// An object keeps its route record once its route is empty, so a
    /// route-mode click on a point its route does not hold is refused and it
    /// never joins a point again (the original). Off: an empty route counts
    /// as no route record.
    // spec: S-0213 claim 16b and Edge cases (quirks.spent_route_blocks_join, default: original)
    #[serde(default = "on")]
    pub spent_route_blocks_join: bool,
    /// The house's route-holder count is raised only by a point add to an
    /// empty route and lowered only when a route that still holds a point
    /// ends, so a deletion that empties a route leaves it raised and a join
    /// that begins a route does not raise it (the original). Off: the count
    /// is always the number of the house's route holders.
    // spec: S-0213 claim 16f and Edge cases (quirks.route_holder_count_drift, default: original)
    #[serde(default = "on")]
    pub route_holder_count_drift: bool,
    /// A route-mode press that takes no highlighted point is tested against
    /// the earliest-made point of any house whose click area holds the
    /// cursor, drawn or not, so a point the player cannot see can refuse
    /// the press (the original). Off: only points drawn for the player are
    /// tested, and a point not drawn plays no part in a press.
    // spec: S-0213 claim 16b, Click areas, and Edge cases (quirks.hidden_point_refuses_click, default: original)
    #[serde(default = "on")]
    pub hidden_point_refuses_click: bool,
    /// A route-mode press inside a point's click area is not tested for
    /// kind, so a selected aircraft or building can join a point (the
    /// original). Off: every route-mode battlefield press with objects
    /// selected meets the kind test first.
    // spec: S-0213 claim 16d, When the tests are made, and Edge cases (quirks.click_area_skips_kind_test, default: original)
    #[serde(default = "on")]
    pub click_area_skips_kind_test: bool,
    /// A point delete applied after the route-mode key's release selects
    /// the next point, which the delete key then deletes unseen (the
    /// original). Off: a point delete applied while the key is not held
    /// selects no point.
    // spec: S-0213 claim 16e, The delete key, and Edge cases (quirks.delete_selects_after_release, default: original)
    #[serde(default = "on")]
    pub delete_selects_after_release: bool,
    /// Where several points of the house stand at one place, a press takes
    /// the earliest-made of them, and its orders name the place (the
    /// original). Off: with objects selected, the press takes the
    /// earliest-made one a selected route holds, else the earliest-made,
    /// and its orders name that point itself.
    // spec: S-0213 claim 16b, Two points at one place, and Edge cases (quirks.stacked_point_takes_earliest, default: original)
    #[serde(default = "on")]
    pub stacked_point_takes_earliest: bool,
    /// The copy a press's join gives an object can take its route past 16
    /// points, and its loop can begin at one of the object's own points
    /// and end before the route's last point (the original). Off: such a
    /// join does nothing when the copy would take the route past 16
    /// points, and the copy's loop always begins in the copy.
    // spec: S-0213 claim 16b, Who gets a press's orders, and Edge cases (quirks.press_copy_ignores_route_state, default: original)
    #[serde(default = "on")]
    pub press_copy_ignores_route_state: bool,
    /// No further search follows a spoiled group-destination look: the member gets its fallback cell (the original). Off (our design, C7 part 4): a member whose look gives no cell of its own gets the nearest open cell round the common cell, in the common cell's reach region and given to no other member.
    // spec: S-0213 claim 7 and Edge cases (no further search after a spoiled look; the off position is our design, the lead's decision of 2026-10-29)
    #[serde(default = "on")]
    pub group_spread_no_detour: bool,
    /// Every unit a factory releases with a cell rally point is sent to the
    /// rally cell itself (the original). Off (our design, C7 part 5): each
    /// released unit is sent to a cell of its own round the rally point, the
    /// rally cell while no other unit of the house has it.
    // spec: S-0417 claims 8 and 10 and S-0404 claim 13 (the rally point copied at release; the off position is our design, the lead's decision of 2026-10-29)
    #[serde(default = "on")]
    pub rally_point_shared: bool,
    /// A harvester's ring search accepts an ore cell that another unit of
    /// its house is already heading for (the original). Off (our design, C7
    /// part 5): such a cell is not acceptable, so harvesters sent to one
    /// field each take a cell of their own.
    // spec: S-0402 claim 4 (the ring search's acceptable cell; the off position is our design, the lead's decision of 2026-10-29)
    #[serde(default = "on")]
    pub harvest_cell_shared: bool,
    /// The harvester-truce exception of row I15 never applies.
    // spec: S-0214 Edge cases
    #[serde(default = "on")]
    pub hijack_truce_harvester_check: bool,
    /// A permanently disguised target passes the disguise rule.
    // spec: S-0214 claim 6
    #[serde(default = "on")]
    pub permadisguise_attack_cursor: bool,
    /// An unmodified band box with no live screen candidate falls through to a click at its release pixel; off: it clears the selection.
    // spec: S-0214 claim 19 and Edge cases
    #[serde(default = "on")]
    pub empty_box_release_click: bool,
    /// *Centre group* keeps the rest of the selection, objects outside the
    /// group included, when the first selected object is in the group, and
    /// centres on all of them (the original). Off: it replaces the
    /// selection with the group's members first, as *select group* does.
    // spec: S-0214 claim 33 and Edge cases (quirks.group_center_keeps_extra_selection, default: YR behaviour)
    #[serde(default = "on")]
    pub group_center_keeps_extra_selection: bool,
    /// A control-group number stays with the object through an owner
    /// change (the original). Off: a group belongs to the house that made
    /// it, and an object counts as a member only while that house owns it.
    // spec: S-0214 claim 33 and Edge cases (quirks.group_kept_on_owner_change, default: YR behaviour)
    #[serde(default = "on")]
    pub group_kept_on_owner_change: bool,
    /// Hovering an engineer over a building above its capture threshold retains the preceding cursor; off: it uses the capture cursor of table C.
    // spec: S-0214 claim 21 and Edge cases
    #[serde(default = "on")]
    pub damaging_entry_cursor_retained: bool,
    /// The attack rule's part (e) takes the firing check's answer, so a reason of the moment that ranks above a target test hides it: while the rearm countdown runs, out of ammunition, cloaked, or with an early rearming of rank 4 (phasing in among them) the attack is offered over a target only those tests refuse; off: part (e) asks only whether a reason that gives illegal holds, at any rank, passing over every reason of the moment.
    // spec: S-0214 claim 6 (e) and Edge cases (Attack offered over a reason of the moment)
    #[serde(default = "on")]
    pub moment_reason_attack_cursor: bool,
    /// View-arrow presses and releases made during an order block are discarded; off: they are accepted (unit orders stay blocked).
    // spec: S-0213 claim 23 and Edge cases; S-0201 claim 14
    #[serde(default = "on")]
    pub blocked_arrow_events: bool,
    /// Queued move and attack move are processed as sleep.
    // spec: S-0215 claim 5
    #[serde(default = "on")]
    pub queued_move_sleeps: bool,
    /// An override suspends the queued behaviour when one is queued.
    // spec: S-0215 claim 10
    #[serde(default = "on")]
    pub override_suspends_queued: bool,
    /// A refused override still replaces the target and destination.
    // spec: S-0215 claim 10
    #[serde(default = "on")]
    pub refused_override_retargets: bool,
    /// The deploy area-fire rule singles out one infantry type by name.
    // spec: S-0215 claim 30
    #[serde(default = "on")]
    pub named_deploy_area_fire: bool,
    /// On a map of width 1 the cell pass visits part of the map.
    // spec: S-0009 claim 3
    #[serde(default = "on")]
    pub narrow_map_partial_pass: bool,
    /// The fill uses the base tiles of the theater loaded before.
    // spec: S-0009 claim 6
    #[serde(default = "on")]
    pub stale_fill_tiles: bool,
    /// An observer's drawn position is handed to the next player needing one.
    // spec: S-0009 claim 22
    #[serde(default = "on")]
    pub observer_position_handoff: bool,
    /// The unused-house cleanup keeps the house after a removed one.
    // spec: S-0009 claim 28
    #[serde(default = "on")]
    pub cleanup_skips_next_house: bool,
    /// A driving vehicle's give-up test at a step check ignores contact.
    // spec: S-0207 claim 15, Edge cases
    #[serde(default = "on")]
    pub drive_giveup_ignores_contact: bool,
    /// A blocked driving vehicle can send its scatter request to the same cell layer twice in one tick; off: only the earliest.
    // spec: S-0207 claim 15, Edge cases
    #[serde(default = "on")]
    pub drive_duplicate_scatter_request: bool,
    /// The nearest object of the look at a new path's first cell is measured from the cell's top corner; off: from its centre.
    // spec: S-0207 claim 15, F8, Edge cases
    #[serde(default = "on")]
    pub drive_nearest_from_top_corner: bool,
    /// A step from a bridge deck is judged by the ground level under the deck, so a side step of 1 level lands on ground 3 or 5 levels below; off: such a step onto a cell that is not a bridge cell is refused.
    // spec: S-0212 claim 10, Edge cases
    #[serde(default = "on")]
    pub deck_side_step_ground_level: bool,
    /// A gate door that reverses from closing to opening is at rest open at the gate's next update; off: the reopening takes as long as the closing had run.
    // spec: S-0215 claim 35
    #[serde(default = "on")]
    pub gate_reverse_snaps: bool,
    /// A turreted building's turret snap bound reads the turn-rate field as a signed value modulo 256; off: the bound is 256 times the turn rate S-0206 gives for the field.
    // spec: S-0215 Edge cases and F8
    #[serde(default = "on")]
    pub turret_snap_bound_wraps: bool,
    /// A non-SAM building with an installed upgrade fires on a *can fire* answer only when its first installed upgrade's ordinary weapons pass the gate (a primary, and the same weapon in both slots or a burst count above 1); off: a failed gate permits the ordinary selected-weapon attempt.
    // spec: S-0215 claim 32
    #[serde(default = "on")]
    pub upgrade_attack_weapon_gate: bool,
    /// Start-next and force leave the suspended behaviour and the put-aside target and destination in place, so a late revert can bring them back; off: an effective start-next or force empties them.
    // spec: S-0215 claims 8 to 10 and Edge cases
    #[serde(default = "on")]
    pub stale_suspended_revert: bool,
    /// Disarm bomb gives its attack order only to an actor able to fire; off: it needs only that the actor is movable.
    // spec: S-0214 Edge cases
    #[serde(default = "on")]
    pub disarm_bomb_needs_weapon: bool,
    /// A berserk carrier aircraft can still get tote and its move order; off: it gets no order, like every berserk object.
    // spec: S-0214 claim 15
    #[serde(default = "on")]
    pub berserk_carrier_tote: bool,
    /// A berserk infantry or vehicle representative shows the cursor of its resolved action; off: the ordinary arrow (presentation only, the action and orders are unchanged).
    // spec: S-0214 claim 15, claim 21 and Edge cases
    #[serde(default = "on")]
    pub berserk_action_cursor: bool,
    /// A building with no weapon in its first slot gets attack over a cell whose generic result is attack (row BC3); off: nothing.
    // spec: S-0214 Edge cases
    #[serde(default = "on")]
    pub building_cell_attack_no_weapon: bool,
    /// A pre-placed object whose starting health comes out above its strength less 3 starts at full strength; off: it keeps the computed value.
    // spec: S-0009 claim 37
    #[serde(default = "on")]
    pub near_full_health_rounds_up: bool,
    /// An absent upgrade name field of a map building entry keeps the type the same field named in an earlier entry; off: it means no upgrade.
    // spec: S-0009 claim 40
    #[serde(default = "on")]
    pub upgrade_fields_carry_over: bool,
    /// The placed vehicle numbered i takes the follow value numbered i, not its own entry's; off: each vehicle takes its own entry's value.
    // spec: S-0009 claim 41
    #[serde(default = "on")]
    pub follow_index_by_place: bool,
    /// The start list skips undefined start waypoints and K stops at the first gap; off: position i is waypoint i whenever it is defined.
    // spec: S-0009 claim 17
    #[serde(default = "on")]
    pub start_waypoint_gap: bool,
    /// A random start candidate's y range runs ten cells further than its x range; off: both from 11 to W + H − 10.
    // spec: S-0009 claim 18
    #[serde(default = "on")]
    pub random_start_asymmetric_range: bool,
    /// Start positions are resolved twice, from three start lists, the assignment against the early phase's list 2; off: one list after the terrain is loaded.
    // spec: S-0009 claim 20
    #[serde(default = "on")]
    pub double_start_resolution: bool,
    /// The early phase judges passivity by the base rules' country data only; off: by this game's full rules layering.
    // spec: S-0009 claim 33
    #[serde(default = "on")]
    pub early_phase_stale_country_data: bool,
    /// In skirmish, pre-placed objects owned by a country name or a player-at-position name vanish; off: they go to the first player of that country or to the player at that position.
    // spec: S-0009 claim 27
    #[serde(default = "on")]
    pub skirmish_drops_country_owned_objects: bool,
    /// Crate overlays placed by a map never appear in skirmish or multiplayer games; off: they are kept.
    // spec: S-0009 claim 12
    #[serde(default = "on")]
    pub skirmish_drops_map_crates: bool,
    /// A starting infantry unit stands at its cell's centre position whatever infantry already stand there, with no draw; off: it takes a standing position by S-0208 claim 3, with that claim's draw.
    // spec: S-0009 claim 46
    #[serde(default = "on")]
    pub starting_infantry_share_centre: bool,
    /// A gate's or wall tower's placement and departure recompute the wall bits of fixed cells around its top-left foundation cell only; off: also every wall beside any of its foundation cells.
    // spec: S-0112 claim 7
    #[serde(default = "on")]
    pub gate_tower_fixed_cells: bool,
    /// A building holding more than 1 round rearms in 1 tick after a shot, over every other kind of delay.
    // spec: S-0301 claim 10
    #[serde(default = "on")]
    pub building_ammo_fast_rearm: bool,
    /// Sonic waves and magnetic-beam waves share one effect slot per firer.
    // spec: S-0301 claim 5
    #[serde(default = "on")]
    pub shared_wave_slot: bool,
    /// The minimum of 1 on an undemolishable building applies to every hit no row stops, so healing hits damage it.
    // spec: S-0302 claim 7
    #[serde(default = "on")]
    pub undemolishable_heal_is_damage: bool,
    /// The yellow damage state is reported at half the strength, whatever the yellow ratio.
    // spec: S-0302 claim 12
    #[serde(default = "on")]
    pub yellow_state_at_half: bool,
    /// An airborne infantry unit hit by a warhead of infantry-death kind 9 takes 0, which the defences raise to 1.
    // spec: S-0302 claim 17
    #[serde(default = "on")]
    pub airborne_mutation_hit_raised: bool,
    /// Readiness loss uses the amount after the defences and before the versus value, even when a later row stops the hit.
    // spec: S-0302 claim 10
    #[serde(default = "on")]
    pub readiness_from_defended_damage: bool,
    /// The delayed-kill countdown divides by the spread's whole cells, so a spread below one cell gives 0.
    // spec: S-0302 claim 16
    #[serde(default = "on")]
    pub delay_kill_whole_cell_spread: bool,
    /// Every hit that changes the health of a building in a delayed-kill countdown credits its destruction again.
    // spec: S-0302 claim 16
    #[serde(default = "on")]
    pub delay_kill_credited_again: bool,
    /// The expected damage divides by the armour factor for the firer's kind and the firer's armour multiplier.
    // spec: S-0302 claim 18
    #[serde(default = "on")]
    pub expected_damage_firer_armour: bool,
    /// A healing hit raises the ammunition of a damage-reduces-readiness type with no upper limit.
    // spec: S-0302 claim 10
    #[serde(default = "on")]
    pub readiness_heal_over_max: bool,
    /// A building takes one hit for each scanned foundation cell within the spread radius.
    // spec: S-0303 claim 8
    #[serde(default = "on")]
    pub area_damage_per_foundation_cell: bool,
    /// A scan radius of 11 uses the original's ring-11 list (one cell twice, some cells missing).
    // spec: S-0303 table T1
    #[serde(default = "on")]
    pub spread_ring_eleven: bool,
    /// Only the first-listed object of a visited cell can take a fragment, so an own or allied object listed first blocks the cell; off: the first object in listing order that passes the test.
    // spec: S-0318 claim 3 and Edge cases
    #[serde(default = "on")]
    pub shrapnel_first_listed_only: bool,
    /// Terrain objects, which have no owning house, take fragments; off: objects without an owning house are not candidates.
    // spec: S-0318 claim 3 and Edge cases
    #[serde(default = "on")]
    pub shrapnel_hits_terrain: bool,
    /// A building takes a fragment in every visited cell where it is first-listed; off: each object takes at most one fragment from one burst's scan.
    // spec: S-0318 claim 3 and Edge cases
    #[serde(default = "on")]
    pub shrapnel_building_per_cell: bool,
    /// A debris piece's ground impact that the deck test puts on a high-bridge deck hits the cell's ground layer (the objects under the deck); off: it hits the deck layer of that cell, as a detonation on the deck does.
    // spec: S-0317 claims 7 and 10 and Edge cases
    #[serde(default = "on")]
    pub voxel_debris_deck_impact_layer: bool,
    /// A debris piece's ground impact from a high-bridge cell over water is a water impact even on the deck (no bounce animation, no hit, the piece ends in its next turn); off: an impact the deck test puts at the deck height is treated as on land.
    // spec: S-0317 claims 8 and 11 and Edge cases
    #[serde(default = "on")]
    pub voxel_debris_deck_over_water: bool,
    /// A debris piece's trailer animation is made at about 6.967 times the piece's height (F7); off: at the piece's flight point before the move.
    // spec: S-0317 claim 6, F7 and Edge cases
    #[serde(default = "on")]
    pub voxel_debris_trailer_height: bool,
    /// A debris piece's end deals its area damage and light flash only when its type names an expiry animation; off: also when the damage amount is not 0.
    // spec: S-0317 claim 11 and Edge cases
    #[serde(default = "on")]
    pub voxel_debris_expiry_needs_animation: bool,
    /// A debris piece's contact on a high bridge's deck rebounds off the slope of the ground cell under the deck; off: a contact that the deck test puts at the deck height rebounds by the flat rule.
    // spec: S-0317 claim 17 and Edge cases
    #[serde(default = "on")]
    pub voxel_debris_deck_rebound_slope: bool,
    /// A debris piece's cliff turn on a north-east or south-west step examines the two cells behind the move with their faces swapped, and a north-west step with only the northern neighbour high gives (c, uₓ, u_y); off: those steps examine the cells beside the corner and the north-west case turns across y.
    // spec: S-0317 claim 18, F9 and Edge cases
    #[serde(default = "on")]
    pub voxel_debris_cliff_step_faults: bool,
    /// A sprite type's upper vertical speed key is not read: F1 always uses 3.5 for the upper vertical limit; off: F1 uses the type's upper vertical speed field.
    // spec: S-0319 claim 2, F1, Edge cases
    #[serde(default = "on")]
    pub anim_fixed_upper_vertical_speed: bool,
    /// A bouncing animation's end deals claim 8's area damage and light flash only when its type names an expiry animation; off: also when its damage amount is not 0.
    // spec: S-0319 claim 8, Edge cases
    #[serde(default = "on")]
    pub bouncing_anim_expiry_needs_animation: bool,
    /// An impact landing on a high-bridge deck hits the objects under the deck in that cell; off: a landing at deck height on a high-bridge cell hits the deck layer, every other impact landing the ground layer.
    // spec: S-0319 claim 7, Edge cases
    #[serde(default = "on")]
    pub bouncing_anim_deck_impact_layer: bool,
    /// A resource-spreading animation's radius selects pairs but the cell depends only on the pair's first number, so at most the eight neighbours receive resource and the landing cell never does; off: the pair (i, j) offers the cell (X + i, Y + j), in the same order.
    // spec: S-0319 claim 10, F5, Edge cases
    #[serde(default = "on")]
    pub anim_resource_spread_neighbours_only: bool,
    /// A move that ends 100 leptons or more below the ground height is not put back on the ground, so the landing point and what is made there lie below it; off: every ground landing is put at the ground height and claim 7 counts a height of 0.
    // spec: S-0319 claims 6 and 7, Edge cases
    #[serde(default = "on")]
    pub bouncing_anim_deep_landing: bool,
    /// An impact landing on water makes its bounce animation and hits before the splash; off: one on water and not at deck height makes no bounce animation and no hits.
    // spec: S-0319 claim 7, Edge cases
    #[serde(default = "on")]
    pub bouncing_anim_water_impact_hits: bool,
    /// A deck contact rebounds off the ground cell's slope under the deck, and the north-west cliff turn gives (c e, e u_x, e u_y); off: a deck contact rebounds by the flat rule and the north-west case turns across the second axis.
    // spec: S-0319 claim 7, F6, F7, Edge cases
    #[serde(default = "on")]
    pub bouncing_anim_rebound_faults: bool,
    /// The impact hits go through the cell's listing as it changes; off: through the objects listed when the hits begin.
    // spec: S-0319 claim 7, Edge cases
    #[serde(default = "on")]
    pub bouncing_anim_impact_hits_walk: bool,
    /// A bouncing animation's new resource cell joins neither of S-0401's lists; off: it is offered to its tiberium's growth list.
    // spec: S-0319 claim 10, Edge cases
    #[serde(default = "on")]
    pub anim_resource_no_growth_entry: bool,
    /// Tracked flying objects are considered only by a detonation above the terrain at the centre of its cell.
    // spec: S-0303 claim 6
    #[serde(default = "on")]
    pub ground_detonation_misses_aircraft: bool,
    /// The flying-object search reads only the sectors of F7 and can miss a flying object in range.
    // spec: S-0303 F7
    #[serde(default = "on")]
    pub flying_search_sectors: bool,
    /// A tracked flying object is not left out for being the source object or a listed harvester.
    // spec: S-0303 claim 9
    #[serde(default = "on")]
    pub flying_candidates_unfiltered: bool,
    /// A small detonation that finds a curtain-protected object hits only protected objects.
    // spec: S-0303 claim 15
    #[serde(default = "on")]
    pub curtain_nullifies_small_detonation: bool,
    /// A wall hit of exactly -1 is absolute.
    // spec: S-0303 claim 16
    #[serde(default = "on")]
    pub wall_hit_minus_one_is_absolute: bool,
    /// Arcing shots at a higher impact point are launched at 45 degrees.
    // spec: S-0306 claim 6
    #[serde(default = "on")]
    pub uphill_arc_45: bool,
    /// A straight shot at a building pitches by the building's art height term.
    // spec: S-0306 claim 6
    #[serde(default = "on")]
    pub building_pitch_term: bool,
    /// The steering turn step uses only the residue mod 256 and is a signed 16-bit value.
    // spec: S-0306 claim 13
    #[serde(default = "on")]
    pub missile_turn_wrap: bool,
    /// A shot fired with no current target aims at the world origin.
    // spec: S-0306 claim 3
    #[serde(default = "on")]
    pub aim_origin_without_target: bool,
    /// The shot-speed distance's sum of squares wraps in 32-bit arithmetic.
    // spec: S-0306 claim 1
    #[serde(default = "on")]
    pub shot_distance_wrap: bool,
    /// A start speed of 0 launches at a horizontal speed of 100.
    // spec: S-0306 F1
    #[serde(default = "on")]
    pub zero_speed_launch: bool,
    /// An area-fire or spray shot is launched towards the firer's own target.
    // spec: S-0306 claim 3
    #[serde(default = "on")]
    pub redirected_shot_aim: bool,
    /// A switched-off hovering vehicle that S-0503 counts as moving is led.
    // spec: S-0306 claim 3 b and Edge cases
    #[serde(default = "on")]
    pub switched_off_hover_lead: bool,
    /// A straight or arcing projectile that has a firer is detonated only by the deck crossings of T1's first row.
    // spec: S-0306 T1 and Edge cases
    #[serde(default = "on")]
    pub owned_shell_ground_pass: bool,
    /// An arcing shot at a point exactly below the fire point uses F4's zero-distance rule.
    // spec: S-0306 claim 6
    #[serde(default = "on")]
    pub zero_distance_arc: bool,
    /// Arrival, pass-by, same-cell and in-flight line-of-fire rows use the point stored at placement.
    // spec: S-0306 claim 9
    #[serde(default = "on")]
    pub stored_target_point: bool,
    /// The world origin stands for no target point in both directions.
    // spec: S-0306 Terms
    #[serde(default = "on")]
    pub origin_as_missing_target_point: bool,
    /// A vertical projectile's speed grows by its acceleration without a cap.
    // spec: S-0306 claim 11
    #[serde(default = "on")]
    pub vertical_speed_uncapped: bool,
    /// A warp member that departed from a high-bridge deck counts as airborne for a projectile until it appears.
    // spec: S-0306 claim 17 and Edge cases
    #[serde(default = "on")]
    pub warp_deck_departure_airborne: bool,
    /// A garrisoned building's range replaces the weapon range and its air bonus.
    // spec: S-0307 claim 4
    #[serde(default = "on")]
    pub garrison_range_drops_air_bonus: bool,
    /// A floor-bound target in a high-bridge cell is measured at deck height even below the bridge.
    // spec: S-0307 claim 7
    #[serde(default = "on")]
    pub range_target_on_deck: bool,
    /// Elevation levels add 4 for every cell marked as carrying a bridge deck.
    // spec: S-0307 claim 6
    #[serde(default = "on")]
    pub elevation_deck_cell_level: bool,
    /// A passenger weapon index is used only by a firer with weapons in both slots.
    // spec: S-0308 claim 4
    #[serde(default = "on")]
    pub passenger_weapon_needs_secondary: bool,
    /// A gattling type's anti-air test reads the weapon in slot 1.
    // spec: S-0308 claim 7
    #[serde(default = "on")]
    pub gattling_air_test_slot_one: bool,
    /// The secondary is chosen against an airborne target whenever it is anti-air.
    // spec: S-0308 table P row 19
    #[serde(default = "on")]
    pub secondary_wins_against_air: bool,
    /// An elite occupant without an elite occupying weapon fires its own primary.
    // spec: S-0308 claim 13
    #[serde(default = "on")]
    pub elite_occupant_skips_occupy_weapon: bool,
    /// A search scores only the offered object of each visited cell.
    // spec: S-0309 claim 10
    #[serde(default = "on")]
    pub search_one_object_per_cell: bool,
    /// The map-wide search measures ground-layer distance in leptons.
    // spec: S-0309 F4
    #[serde(default = "on")]
    pub hunt_distance_in_leptons: bool,
    /// A final threat score of exactly 0 rejects the candidate.
    // spec: S-0309 claim 13
    #[serde(default = "on")]
    pub zero_score_rejected: bool,
    /// Passengers of a mind-controlled open-topped transport take the cell's last object only.
    // spec: S-0309 claim 10
    #[serde(default = "on")]
    pub mind_controlled_passengers_cell_choice: bool,
    /// A distributed-fire searcher picks nothing in a map-wide search.
    // spec: S-0309 claim 20
    #[serde(default = "on")]
    pub hunt_ignores_distributed_fire: bool,
    /// A defeated enemy house stays the enemy house until an anger value next changes.
    // spec: S-0309 claim 16
    #[serde(default = "on")]
    pub defeated_enemy_house_kept: bool,
    /// The map-wide search considers only ground-layer objects besides aircraft.
    // spec: S-0309 claim 7
    #[serde(default = "on")]
    pub hunt_ground_layer_only: bool,
    /// A wall candidate wins over a candidate object later in the same ring.
    // spec: S-0309 claim 25
    #[serde(default = "on")]
    pub wall_cell_over_same_ring_object: bool,
    /// A building with a voxel turret must point exactly at its target.
    // spec: S-0310 claim 4
    #[serde(default = "on")]
    pub voxel_turret_exact_aim: bool,
    /// The recipient's actual cost is computed for the destroyed object's house.
    // spec: S-0311 claim 10
    #[serde(default = "on")]
    pub experience_cost_victim_house: bool,
    /// A garrison's firing occupant gains experience whatever its trainable flag.
    // spec: S-0311 claim 8
    #[serde(default = "on")]
    pub occupant_experience_untrained: bool,
    /// A garrison's experience goes to the occupant whose turn to fire it is.
    // spec: S-0311 claim 8
    #[serde(default = "on")]
    pub occupant_experience_next_in_turn: bool,
    /// A kill value of 0 against an actual cost of 0 sets experience to the cap.
    // spec: S-0311 F3
    #[serde(default = "on")]
    pub zero_kill_value_caps: bool,
    /// An exploding aircraft fires its death weapon at death and again at the crash.
    // spec: S-0312 claim 16
    #[serde(default = "on")]
    pub crash_death_weapon_twice: bool,
    /// A destroyed building takes up to two ranged draws whose results are not used.
    // spec: S-0312 claim 19
    #[serde(default = "on")]
    pub building_smudge_discarded_draws: bool,
    /// The death weapon's projectile tests use its unplaced default position.
    // spec: S-0312 claim 6
    #[serde(default = "on")]
    pub death_weapon_unplaced_projectile: bool,
    /// Rubble is the overlay type at a fixed registration index.
    // spec: S-0312 claim 20
    #[serde(default = "on")]
    pub rubble_overlay_by_index: bool,
    /// An exploding building probes for barrels from its top-left cell only.
    // spec: S-0312 claim 23
    #[serde(default = "on")]
    pub barrel_probe_top_left_only: bool,
    /// A smudge footprint cell beyond the map edge is tested as a
    /// placeholder cell that holds nothing and has the theater's first tile
    /// type; off: a footprint beyond the map edge does not fit.
    // spec: S-0312 claim 26, Edge cases
    #[serde(default = "on")]
    pub smudge_footprints_beyond_map_edge: bool,
    /// A destruction smudge's anchor cell with both coordinates 0 receives
    /// no smudge and the smudge choice takes no draw; off: that anchor takes
    /// the ordinary fit, candidate and choice rules, map legality included.
    // spec: S-0312 claim 26, Edge cases
    #[serde(default = "on")]
    pub destruction_smudge_zero_anchor: bool,
    /// A building's turret displacement does not turn with its aim.
    // spec: S-0313 claim 4
    #[serde(default = "on")]
    pub building_fire_point_shift_fixed: bool,
    /// Negative self-heal counts are added without a floor.
    // spec: S-0314 claim 10
    #[serde(default = "on")]
    pub negative_self_heal_counts: bool,
    /// A fraidycat that fires its last round panics even when fearless.
    // spec: S-0315 claim 2
    #[serde(default = "on")]
    pub fraidycat_ammo_panic_ignores_fearless: bool,
    /// The last bale taken from a cell yields nothing.
    // spec: S-0401 claim 3
    #[serde(default = "on")]
    pub last_bale_lost: bool,
    /// Neighbours of an emptied cell join the emptied cell's tiberium's spread list.
    // spec: S-0401 claim 5
    #[serde(default = "on")]
    pub spread_offer_by_emptied_type: bool,
    /// Failed spread attempts count and the cell returns to the front of the list.
    // spec: S-0401 claim 9
    #[serde(default = "on")]
    pub spread_attempts_count_failures: bool,
    /// A cell dropped from the spread list with one free neighbour can rejoin only through a rebuilding.
    // spec: S-0401 claim 9
    #[serde(default = "on")]
    pub spread_mark_kept_on_last_neighbour: bool,
    /// A cell holding a terrain object (a tree) counts as germinable; when a spread step or an
    /// ore mine chooses it, it receives nothing and the spread step's attempt is used. Off: such
    /// a cell is not germinable, so a spread step passes over it and it does not count in *c*.
    // spec: S-0401 claims 10 and 11, Edge cases
    #[serde(default = "on")]
    pub spread_step_lost_on_terrain_object: bool,
    /// The stage a cell needs to spread depends on the tiberium's index.
    // spec: S-0401 claim 17
    #[serde(default = "on")]
    pub spread_stage_from_index: bool,
    /// Harvesting a full cell takes it out of growth until a rebuilding.
    // spec: S-0401 claim 18
    #[serde(default = "on")]
    pub full_cell_no_regrowth: bool,
    /// Resource timers carry over between games of one session.
    // spec: S-0401 claim 19
    #[serde(default = "on")]
    pub resource_timers_carry_over: bool,
    /// The waypoint-thinning action takes every bale from the waypoint's own cell only.
    // spec: S-0401 claim 16
    #[serde(default = "on")]
    pub thinning_hits_waypoint_only: bool,
    /// The squared refinery distance is a signed 32-bit integer that wraps.
    // spec: S-0402 claim 8
    #[serde(default = "on")]
    pub refinery_distance_wraps: bool,
    /// An idle harvester whose step-off search finds no cell gets the
    /// off-map destination cell (0, 0); off: its destination is cleared.
    // spec: S-0402 claim 13 and Edge cases
    #[serde(default = "on")]
    pub harvester_failed_step_off_origin: bool,
    /// A load period of 0 keeps the harvester loading for good after its first bale.
    // spec: S-0402 claim 5
    #[serde(default = "on")]
    pub zero_load_period_stalls: bool,
    /// A harvester whose refinery breaks off contact while it unloads, with nothing queued, stays in unload until an order gives it a behaviour.
    // spec: S-0402 claim 11
    #[serde(default = "on")]
    pub unload_without_contact_stalls: bool,
    /// The slave miner's kick delay wraps when added to the tick number.
    // spec: S-0402 claim 14
    #[serde(default = "on")]
    pub slave_kick_delay_wraps: bool,
    /// A deployed slave miner whose long scan finds no field, or whose site
    /// search finds no site, still makes the F7 comparison with cell (0, 0)
    /// for the missing result; off: either failure leaves it deployed.
    // spec: S-0402 claim 25
    #[serde(default = "on")]
    pub slave_relocation_failed_search_origin: bool,
    /// A slave of negative storage stays in *harvest* on its resource cell
    /// without loading; off: it is treated as a slave of storage 0.
    // spec: S-0402 claim 26
    #[serde(default = "on")]
    pub slave_negative_capacity_stalls: bool,
    /// Every factory building of the kind counts for the several-factories factor, also those that cannot make the item.
    // spec: S-0403 claim 6
    #[serde(default = "on")]
    pub multiple_factory_counts_all: bool,
    /// The step period never exceeds 255 ticks.
    // spec: S-0403 claim 1
    #[serde(default = "on")]
    pub build_step_period_cap: bool,
    /// A resumed line waits a full step period before its next step.
    // spec: S-0403 claim 8
    #[serde(default = "on")]
    pub resume_restarts_step: bool,
    /// A cancel refunds the item's price at the moment of the cancel less the balance owed.
    // spec: S-0404 claim 6
    #[serde(default = "on")]
    pub refund_at_current_price: bool,
    /// A forced start with no switched-on factory starts on a player hold when the house can pay P ÷ 54.
    // spec: S-0404 claim 10
    #[serde(default = "on")]
    pub forced_start_player_hold: bool,
    /// The factory check after a building-factory event covers only the ordinary-building line.
    // spec: S-0404 claim 8; S-0414
    #[serde(default = "on")]
    pub defence_line_partial_factory_check: bool,
    /// A queued type that cannot start is dropped and the line starts nothing more.
    // spec: S-0404 claim 10
    #[serde(default = "on")]
    pub unstartable_queue_head_stalls: bool,
    /// The infantry exit search uses each foundation's fixed exit list, gaps and repeats included.
    // spec: S-0404 claim 13
    #[serde(default = "on")]
    pub foundation_exit_lists: bool,
    /// A blackout of length d counts as 0 output in at most d − 1 house updates.
    // spec: S-0405 claim 6
    #[serde(default = "on")]
    pub blackout_one_short: bool,
    /// One count of active control centres per house serves every control-centre type.
    // spec: S-0405 claim 13
    #[serde(default = "on")]
    pub shared_control_centre_count: bool,
    /// A cell meeting a wall exception passes without the ground conditions of claim 3 (d).
    // spec: S-0406 claim 8
    #[serde(default = "on")]
    pub wall_exception_skips_ground: bool,
    /// A person's placement click takes the shroud result from when the cursor entered the cell.
    // spec: S-0406 claim 6
    #[serde(default = "on")]
    pub placement_shroud_at_cursor: bool,
    /// An upgrade is placed even when the target records nothing.
    // spec: S-0406 claim 12
    #[serde(default = "on")]
    pub upgrade_level_without_room: bool,
    /// An upgrade prerequisite examines only the most recently acquired eligible building.
    // spec: S-0407 claim 8 (c)
    #[serde(default = "on")]
    pub upgrade_prerequisite_last_building: bool,
    /// A building put on the map by a place order does not raise its type's ever-built count.
    // spec: S-0407 claim 10
    #[serde(default = "on")]
    pub placed_buildings_not_ever_built: bool,
    /// An unknown or non-building prerequisite name is dropped when the list is read; off: it stays an unmet entry.
    // spec: S-0407 claim 8 (d) and Edge cases
    #[serde(default = "on")]
    pub prerequisite_names_dropped: bool,
    /// An ownership change gives the gaining house a kill on the result screen; off: only destructions add kills.
    // spec: S-0414 claim 21
    #[serde(default = "on")]
    pub ownership_gain_counts_as_kill: bool,
    /// A tab flash starts with an extra 10-update period; off: the first toggle comes at the next 10-tick boundary.
    // spec: S-0411 claim 11 and Edge cases
    #[serde(default = "on")]
    pub tab_flash_initial_extra_period: bool,
    /// A sidebar icon clips only at its content region's right and bottom; off: at all four edges.
    // spec: S-0411 claim 16 and Edge cases
    #[serde(default = "on")]
    pub sidebar_icon_surface_clip: bool,
    /// Each secret lab picks its entry by position with replacement, so labs can share a type.
    // spec: S-0407 claim 14
    #[serde(default = "on")]
    pub secret_lab_pick_by_position: bool,
    /// Unit entries, departures, owner changes and removals below the limit call for no recheck.
    // spec: S-0407 claim 12
    #[serde(default = "on")]
    pub unit_changes_no_recheck: bool,
    /// The start of a sale calls for no recheck of the offer.
    // spec: S-0407 claim 12
    #[serde(default = "on")]
    pub sale_start_no_recheck: bool,
    /// A type left in the offer after it stopped qualifying can be ordered until the next recheck.
    // spec: S-0407 claim 13
    #[serde(default = "on")]
    pub stale_offer_orderable: bool,
    /// The queue-time limit test counts the item in production twice.
    // spec: S-0407 claim 16
    #[serde(default = "on")]
    pub queue_limit_counts_item_twice: bool,
    /// A dock-bound aircraft type is tested at queue time by the dock test only.
    // spec: S-0407 claim 16
    #[serde(default = "on")]
    pub dock_bound_aircraft_ignores_limit: bool,
    /// The infantry in-production exception of the build limit applies only at exactly L owned.
    // spec: S-0407 claim 9
    #[serde(default = "on")]
    pub infantry_limit_exception_exact: bool,
    /// Country lists cannot tell apart countries whose indices are equal modulo 32.
    // spec: S-0407 F1
    #[serde(default = "on")]
    pub country_index_wraps: bool,
    /// A vehicle of a not-scored type is counted on entering the map and never taken off.
    // spec: S-0414 claim 4; S-0407
    #[serde(default = "on")]
    pub unscored_vehicle_stays_counted: bool,
    /// A nominal survivor is not added to the infantry count but is taken off when it leaves.
    // spec: S-0414 claim 4; S-0407
    #[serde(default = "on")]
    pub nominal_survivor_count: bool,
    /// A wall sale pays nothing.
    // spec: S-0408 claim 12
    #[serde(default = "on")]
    pub wall_sale_no_refund: bool,
    /// A packing building releases survivors according to its guard spot and undeploy type only.
    // spec: S-0415 claim 9; S-0408
    #[serde(default = "on")]
    pub packup_release_by_guard_spot: bool,
    /// A placed building's build-up runs once before *construction* becomes current and again after (off: the place order sets the ready mark at entry, as a deploy does).
    // spec: S-0406 claim 17, F7 and Edge cases
    #[serde(default = "on")]
    pub placed_buildup_twice: bool,
    /// A positive-delay construction range of one stage or fewer never reports completion (off: it reports d_b ticks after the range starts).
    // spec: S-0406 claim 18 and Edge cases
    #[serde(default = "on")]
    pub single_stage_buildup_never_completes: bool,
    /// A gap radius is read reduced into −128 … 127.
    // spec: S-0409 claim 11
    #[serde(default = "on")]
    pub gap_radius_wrap: bool,
    /// The height bonus of sight wraps into −128 … 127.
    // spec: S-0409 F1
    #[serde(default = "on")]
    pub sight_bonus_wrap: bool,
    /// A building's sight, area reveal and gap field are centred on its top-left foundation cell.
    // spec: S-0409 claim 5
    #[serde(default = "on")]
    pub building_sight_top_left: bool,
    /// A sight whose centre is not a legal cell reveals nothing.
    // spec: S-0409 claim 5
    #[serde(default = "on")]
    pub sight_centre_off_map: bool,
    /// The height block tests the displaced cell and the step directions of table T2.
    // spec: S-0409 claim 7
    #[serde(default = "on")]
    pub height_block_cell: bool,
    /// When a hostile gap field stops, its cells stay unexplored until a sight reaches them.
    // spec: S-0409 claim 11
    #[serde(default = "on")]
    pub gap_end_keeps_cover: bool,
    /// The reveal on fire is a plain reveal that the next re-cover sweep undoes.
    // spec: S-0409 claim 16
    #[serde(default = "on")]
    pub fire_reveal_expires: bool,
    /// The whole-map re-cover takes in allied units' sights only at their next refresh.
    // spec: S-0409 claim 15
    #[serde(default = "on")]
    pub recover_skips_allied_units: bool,
    /// The whole-map re-cover and reveal leave pending re-cover marks in place.
    // spec: S-0409 claim 15
    #[serde(default = "on")]
    pub recover_keeps_pending: bool,
    /// The original applies an uncounted sight at a few moments, never removed.
    // spec: S-0409 claim 8
    #[serde(default = "on")]
    pub uncounted_sight: bool,
    /// Every cell that is not legal shares one explored state per house.
    // spec: S-0409 claim 14
    #[serde(default = "on")]
    pub offmap_cells_shared: bool,
    /// Only the first qualifying spy-satellite building is looked at.
    // spec: S-0409 claim 15
    #[serde(default = "on")]
    pub spy_satellite_first_building_decides: bool,
    /// The defeat reveal leaves the defeated house's pending re-cover marks in place.
    // spec: S-0409 claim 15
    #[serde(default = "on")]
    pub defeat_reveal_keeps_pending: bool,
    /// A cell coordinate outside 0 … 511 is read as another cell by gap fields and the shroud point test.
    // spec: S-0409 claim 14; S-0110
    #[serde(default = "on")]
    pub shroud_point_cell_wrap: bool,
    /// An odd negative height count in the shroud point test reads the cell pair one cell toward map north-west (k = (u + 3)/2); off: k = (u + 1)/2 for every odd u. No effect while the renderer's shroud_on_flat_grid is off, which the simulation never models.
    // spec: S-0110 claim 8, Edge cases
    #[serde(default = "on")]
    pub shroud_negative_odd_pair: bool,
    /// Only the first qualifying radar building decides whether the house has radar.
    // spec: S-0410 claim 1; S-0405 claim 12; S-0409
    #[serde(default = "on")]
    pub radar_first_building_decides: bool,
    /// Radar overlay colours depend on the overlay type's position in the overlay list.
    // spec: S-0410 claim 9
    #[serde(default = "on")]
    pub overlay_list_position_rules: bool,
    /// The cell a radar pixel stands for is the one F3 gives, shifted from the point colour's cell.
    // spec: S-0410 claim 6
    #[serde(default = "on")]
    pub radar_pixel_cell_shift: bool,
    /// The automatic place order of a done unit is sent once.
    // spec: S-0411 claim 9
    #[serde(default = "on")]
    pub release_order_once: bool,
    /// A left click on a done aircraft with every dock taken cancels it.
    // spec: S-0411 claim 4
    #[serde(default = "on")]
    pub done_aircraft_click_cancels: bool,
    /// A random country is drawn from registration indices 0 … 9.
    // spec: S-0412 claim 6
    #[serde(default = "on")]
    pub random_country_fixed_range: bool,
    /// The superweapon setting's build bar tests only a building type's first superweapon.
    // spec: S-0412 Edge cases; S-0407 claim 7
    #[serde(default = "on")]
    pub superweapon_setting_tests_first_only: bool,
    /// Only human participants count for the initial crate count.
    // spec: S-0412 claim 10
    #[serde(default = "on")]
    pub crate_count_humans_only: bool,
    /// A short game with the bases setting off is allowed (every house without a counted building fails at tick 1).
    // spec: S-0412 claim 11
    #[serde(default = "on")]
    pub short_game_without_bases: bool,
    /// Only the first three positions of the base-unit list count in the short-game test.
    // spec: S-0414 claim 2
    #[serde(default = "on")]
    pub short_game_first_three_base_units: bool,
    /// A building that counts as a vehicle keeps its house alive only when it is a refinery-list building.
    // spec: S-0414 claim 3
    #[serde(default = "on")]
    pub vehicle_buildings_keep_nobody_alive: bool,
    /// A house defeated during the end delay loses (the person-played house's win turns into a loss and restarts the delay).
    // spec: S-0414 claim 9; S-0412
    #[serde(default = "on")]
    pub defeat_during_end_delay: bool,
    /// Immediate deletions by one destruction hit of a defeat or surrender skip later objects.
    // spec: S-0414 claim 15
    #[serde(default = "on")]
    pub defeat_cancellation_skip: bool,
    /// A deploying vehicle cannot deploy onto its own house's wall.
    // spec: S-0415 claim 3
    #[serde(default = "on")]
    pub deploy_refuses_own_walls: bool,
    /// The vehicle of a failed pack-up remains.
    // spec: S-0415 claim 11
    #[serde(default = "on")]
    pub failed_packup_keeps_vehicle: bool,
    /// A sold building with a pack-up sound also plays it.
    // spec: S-0415 claim 9
    #[serde(default = "on")]
    pub packup_sound_on_sale: bool,
    /// A ground factory whose exit list has fewer than eleven entries takes
    /// the original's special apron pairs; off: the list's last pair, (0, 0)
    /// for an empty list.
    // spec: S-0417 Edge cases
    #[serde(default = "on")]
    pub factory_apron_short_list: bool,
    /// A released vehicle that neither drives nor drives temporarily heads
    /// for the fixed cell (x₀ + 4, y₀ + 1); off: the exit apron.
    // spec: S-0417 Edge cases; S-0417 claim 5
    #[serde(default = "on")]
    pub factory_fixed_hover_exit: bool,
    /// A factory's release queues *unload* and its closing queues *guard*,
    /// each starting only at a per-tick start that finds the ready mark
    /// set; off: queueing either also sets the ready mark.
    // spec: S-0417 claims 3 and 6 and Edge cases
    #[serde(default = "on")]
    pub factory_unload_waits_ready_mark: bool,
    /// A sidebar item tied with existing items on every key goes before
    /// them (the newest first); off: after them.
    // spec: S-0004 claim 11; S-0407 claim 17 and Edge cases
    #[serde(default = "on")]
    pub sidebar_tie_newest_first: bool,
    /// An item with both the counts-as-aircraft and naval flags ranks 1 as
    /// the new item and 2 as an item already in the tab; off: 1 in both.
    // spec: S-0004 claim 11
    #[serde(default = "on")]
    pub sidebar_both_flags_rank: bool,
    /// An explicit initial ammunition above a nonnegative maximum is used as
    /// it stands; off: capped at the maximum.
    // spec: S-0301 claim 10, Formulas F6
    #[serde(default = "on")]
    pub initial_ammo_above_capacity: bool,
    /// A building awaiting removal takes a second round of survivors and
    /// smudge steps when it is removed; off: no second round.
    // spec: S-0312 claim 21, Edge cases
    #[serde(default = "on")]
    pub building_removal_second_round: bool,
    /// A vehicle's approach search tries the eight alternative cells in
    /// the original's order (0, −1), (1, −2), …, (0, 0); off: the eight
    /// neighbours in direction order 0 to 7.
    // spec: S-0307 claim 16, Edge cases
    #[serde(default = "on")]
    pub approach_alternative_walk: bool,
    /// A vehicle's approach alternatives are measured at world height 0;
    /// off: at the terrain height of their centre, plus 416 leptons on a
    /// high bridge, as primaries are.
    // spec: S-0307 claim 19, Edge cases
    #[serde(default = "on")]
    pub approach_alternative_zero_height: bool,
    /// A vehicle's approach alternatives skip the usable-area test; off:
    /// they must pass it as primaries do.
    // spec: S-0307 claim 21, Edge cases
    #[serde(default = "on")]
    pub approach_alternative_outside_area: bool,
    /// A negative elevation bonus cap applies as it is, even at no height
    /// difference; off: the cap is max(0, cap) in the elevation bonus.
    // spec: S-0307 claim 6, Edge cases (quirks.negative_elevation_cap)
    #[serde(default = "on")]
    pub negative_elevation_cap: bool,
    /// Negative weapon range, minimum range and air range bonus fields are
    /// used as they are; off: max(0, value), the unlimited −512 kept.
    // spec: S-0307 claim 1, Edge cases (quirks.negative_range_fields)
    #[serde(default = "on")]
    pub negative_range_fields: bool,
    /// The sound data's game-wide default priority is read and never applied: an entry without its own priority is normal; off: it is the default for entries without their own priority.
    // spec: S-0701 claim 3, Edge cases
    #[serde(default = "on")]
    pub default_sound_priority_ignored: bool,
    /// One sample missing from the audio archive silences the whole sound entry; off: the missing sample is left out of the list and the rest play.
    // spec: S-0701 claim 4, Edge cases
    #[serde(default = "on")]
    pub missing_sample_silences_entry: bool,
    /// Immediate-mode random ordering with no attack sample takes and discards one startup ordering draw; off: no discarded draw.
    // spec: S-0701 claim 14, Edge cases
    #[serde(default = "on")]
    pub sound_order_startup_draw: bool,
    /// A sound entry's volume of 100 % gives level 16383; off: 16384.
    // spec: S-0701 F1, Edge cases
    #[serde(default = "on")]
    pub full_volume_16383: bool,
    /// A sound entry with the attack or decay word whose samples leave no body sample makes no sound, then stays silent for the session; off: its attack and decay samples play.
    // spec: S-0701 Edge cases
    #[serde(default = "on")]
    pub attack_decay_without_body: bool,
    /// A sound with the shroud word at cell (0, 0) is never heard; off: that cell is judged like any other.
    // spec: S-0701 claim 6, Edge cases
    #[serde(default = "on")]
    pub shroud_sound_corner_cell: bool,
    /// The deploy command's game-wide sound is never read, so the command is silent; off: it is read and played.
    // spec: S-0701 claim 19, Edge cases
    #[serde(default = "on")]
    pub deploy_command_silent: bool,
    /// An announcement's volume key has no effect; off: the volume (a fraction, default 1) scales it.
    // spec: S-0701 claim 26, Edge cases
    #[serde(default = "on")]
    pub announcement_volume_ignored: bool,
    /// A vehicle harvester type with capacity 0 runs its cycle without ever carrying a bale (fill ratio 0/0); off: imported capacity 0 counts as 1 bale.
    // spec: S-0402 claim 1, Edge cases
    #[serde(default = "on")]
    pub zero_capacity_harvester_cycles: bool,
    /// A dock-type building without the dock-unload flag still accepts refinery contact, leaving the harvester in enter without unloading; off: such buildings are excluded from both refinery choices and refuse refinery contact.
    // spec: S-0402 claims 8 and 10, Edge cases
    #[serde(default = "on")]
    pub non_unloading_refinery_accepts: bool,
    /// The 9-tick launch spacing is chosen by the missile-spawn flag of the spawner's own type, not of the spawned type; off: the spawned type's flag decides.
    // spec: S-0506 claim 6, Edge cases
    #[serde(default = "on")]
    pub launch_spacing_from_spawner_type: bool,
    /// A set-C missile's rail time uses set B's pause and tilt times; off: set C's own times.
    // spec: S-0506 claim 9, Edge cases
    #[serde(default = "on")]
    pub raised_missile_rail_time_from_set_b: bool,
    /// Missiles on the rail vanish with no detonation when the spawner dies or changes hands; off: they are released like preparing missiles.
    // spec: S-0506 claim 13, Edge cases
    #[serde(default = "on")]
    pub rail_missile_vanishes_with_spawner: bool,
    /// An own-target clear empties the pending target only when the controller target is nonempty, so a pending shot can survive the clear and launch later; off: every explicit own-target clear empties the pending target.
    // spec: S-0506 claim 25, Edge cases
    #[serde(default = "on")]
    pub spawner_pending_target_survives_idle_clear: bool,
    /// A rocket missile's aim point is fixed at the heading point of claim 14.
    // spec: S-0506 claim 14, Edge cases
    #[serde(default = "on")]
    pub rocket_aim_at_heading_point: bool,
    /// A rocket missile destroyed in flight still detonates (claim 20).
    // spec: S-0506 claim 20, Edge cases
    #[serde(default = "on")]
    pub destroyed_rocket_still_detonates: bool,
    /// A rocket missile whose path leaves the legal cells keeps the last legal position and continues its phase, speed and pitch updates; off: it detonates at the last legal position.
    // spec: S-0506 claim 18, Edge cases
    #[serde(default = "on")]
    pub rocket_stalls_at_map_edge: bool,
    /// A parasite's exit-cell search uses the host's locomotion class and movement zone; off: the parasite's own.
    // spec: S-0614 claim 13, Edge cases
    #[serde(default = "on")]
    pub parasite_exit_search_host_mobility: bool,
    /// Whether a parasite may come out at all is judged on the host's cell, not on the chosen cell; off: the chosen cell is tested.
    // spec: S-0614 claim 13, Edge cases
    #[serde(default = "on")]
    pub parasite_exit_tests_host_cell: bool,
    /// On host loss, the fallback neighbour's restricted land passes when the host's cell has a bridge; off: the neighbour's own bridge is tested.
    // spec: S-0614 claim 15, Edge cases
    #[serde(default = "on")]
    pub parasite_host_loss_bridge_test: bool,
    /// A parasite whose carrying projectile is removed without detonating stays off the map, unattached; off: it attempts claim 4's failed-latch return.
    // spec: S-0614 claim 21, Edge cases
    #[serde(default = "on")]
    pub parasite_carrier_loss_strands_unit: bool,
    /// A parasite whose projectile detonates with takeover, bomb planting or electric charging selected stays off the map, unattached and inactive; off: it attempts claim 4's failed-latch return (never attaching).
    // spec: S-0614 claim 24, Edge cases
    #[serde(default = "on")]
    pub parasite_suppressed_latch_strands_unit: bool,
    /// On host loss, a vehicle parasite whose fallback standing-position choice fails survives at world position (128, 128, 0) leptons outside the playable map; off: it is removed silently like infantry, with the same draw count.
    // spec: S-0614 claim 23, Edge cases
    #[serde(default = "on")]
    pub parasite_host_loss_failed_choice_origin: bool,
    /// On host loss, a suppressed naval parasite is removed silently; off: a naval parasite ignores suppression here too.
    // spec: S-0614 claim 15, Edge cases; S-0315 claim 7
    #[serde(default = "on")]
    pub parasite_host_loss_suppresses_naval: bool,
    /// A cull's experience has no rank multiplier, no alliance test and no unscored test, and divides by the parasite's cost for its own house; off: the cull is a destruction credited to the parasite under S-0311 claims 7 to 10.
    // spec: S-0614 claim 11, Edge cases
    #[serde(default = "on")]
    pub cull_experience_path: bool,
    /// With the re-approach flag off, an ordinary non-fighter, non-strafer repeats burst-plus-single-shot cycles without spending its pending round; off: each such cycle ends as one run.
    // spec: S-0501 claims 14 and 15, Edge cases
    #[serde(default = "on")]
    pub aircraft_endless_refire: bool,
    /// An aircraft on a pad with a target is never reloaded past half its ammunition while it keeps the target; off: reload to full regardless of target.
    // spec: S-0501 claim 28, Edge cases
    #[serde(default = "on")]
    pub aircraft_half_reload_with_target: bool,
    /// An aircraft's damage-smoke chance draws from the scenario stream, including updates that show no smoke; off: no damage-smoke draw and no smoke animation.
    // spec: S-0501 claim 37, Edge cases
    #[serde(default = "on")]
    pub aircraft_smoke_scenario_draw: bool,
    /// The pad limit's count includes only types in the pad-aircraft list; off: the asked-for type is counted too.
    // spec: S-0501 claim 30, S-0407 Edge cases
    #[serde(default = "on")]
    pub pad_limit_counts_list_only: bool,
    /// The squared pad distance of F9 wraps in 32 bits, so a very distant pad can look nearer; off: exact squared distances.
    // spec: S-0501 claim 21 and F9, Edge cases
    #[serde(default = "on")]
    pub pad_distance_overflow: bool,
    /// When the own-cell landing search near a close building finds no cell, the landing destination becomes the placeholder at map cell (0, 0), which the aircraft discards at the end of its update and is given again, possibly heading for that map corner; off: the failed search counts as no successful building choice and the other-object fallback follows, with the chosen building counted like any other own-house object, and finally the aircraft's own cell.
    // spec: S-0501 claim 41, F8 and Edge cases
    #[serde(default = "on")]
    pub landing_own_cell_no_result: bool,
    /// A forward step of exactly 1 lepton at a body facing between east and south (facings 16391 to 32766) changes neither coordinate, because each coordinate is truncated and no remainder is kept, so an aircraft slowing toward its destination can hover there with a positive fraction (an ordinary attacker never fires, a move or a return to a pad stops short of landing); off: a forward step of at least 1 lepton that would change neither coordinate moves 1 lepton along the axis of its larger component, in that component's direction, x on equal magnitudes.
    // spec: S-0501 claims 4 and 5, F3, Edge cases
    #[serde(default = "on")]
    pub aircraft_unit_step_stall: bool,
    /// An ordinary attacker reaches firing from approaching only at a current speed fraction of exactly 0, so one hovering with a positive fraction whose forward step truncates to 0 leptons never fires; off: a positive fraction with step length 0 also moves it to firing, keeping its destination, with the same 1-tick delay.
    // spec: S-0501 claim 42, T1, Edge cases
    #[serde(default = "on")]
    pub aircraft_zero_step_attack_stall: bool,
    /// A bomb planted on a pad-bound aircraft that crashes at S-0501 claim 46's fall-back without going off during the fall is removed at impact with no detonation, animation or damage; off: at impact, before the death weapon, the bomb goes off by S-0519 claim 5 at the aircraft's position, with its attacker and attacking house.
    // spec: S-0501 claim 46, Edge cases; S-0519 claim 7
    #[serde(default = "on")]
    pub crash_drops_planted_bomb: bool,
    /// An aircraft in flight that is not tracked and whose cell lies in sector (0, 0) of S-0303's F7, cell (0, 0) included, joins no flying-object list until it moves into another sector, so neither area damage nor the airborne scan finds it there; off: at its first movement update with a positive carried speed fraction it joins the end of sector (0, 0)'s list.
    // spec: S-0501 claim 47 (c), Edge cases
    #[serde(default = "on")]
    pub aircraft_corner_sector_untracked: bool,
    /// A factory release whose docking position lies above the terrain under it leaves the aircraft resting in the air, in neither cell layer and not tracked, and its first movement update descends; released at most 20 leptons up, it reaches height 0 there and, on a move, takes off at the move's first processing, before its second movement update; off: the release puts it at the docking position's horizontal location at the landing height above the terrain there, listed in its cell layer like an aircraft docked by landing, holding the object-present flag in that cell, and it leaves by an ordinary take-off at the order, which steers it toward its destination.
    // spec: S-0501 claim 48 (c) and (e), Edge cases
    #[serde(default = "on")]
    pub aircraft_released_above_pad: bool,
    /// An unarmed pad-bound aircraft with dock types that falls back to rest in flight with no contact makes no pad search: it takes an F8 landing spot and move, and hovers there; off: T2's last in-flight row first looks for a pad and asks it for contact, and when the pad grants it, it takes the pad as destination with enter and docks; otherwise the landing spot and move, with no crash. Only a landable type is affected.
    // spec: S-0501 claim 50, Edge cases
    #[serde(default = "on")]
    pub unarmed_pad_bound_hover: bool,
    /// An aircraft that may leave the map now and lacks the fly-by flag is pushed back at the legal map's boundary, with the side rule and the fallback draw, so with the fly-back flag (or where the usable area's boundary lies on or beyond the legal map's) it is never removed there; off: it is not pushed back, takes no candidate and no draw, and is removed in the update tick in which its position leaves the legal map.
    // spec: S-0501 claims 4, 10 and 51, Edge cases
    #[serde(default = "on")]
    pub aircraft_edge_hold: bool,
    /// A passenger adds its size rounded down to the transport's load; off: the size is kept unrounded.
    // spec: S-0507 claim 1, F1, Edge cases
    #[serde(default = "on")]
    pub transport_load_truncated: bool,
    /// The gap between a neighbour's direction and the rear facing is not taken the short way round; off: the short way.
    // spec: S-0507 F2, Edge cases
    #[serde(default = "on")]
    pub unload_facing_gap_no_wrap: bool,
    /// The second unloading round starts one step clockwise of the rear; off: it tries the rear direction first.
    // spec: S-0507 Edge cases
    #[serde(default = "on")]
    pub unload_second_round_skips_rear: bool,
    /// A failed exit changes the gunner's weapon number to position 1's mode index; off: unchanged.
    // spec: S-0507 Edge cases
    #[serde(default = "on")]
    pub unload_failed_exit_gunner_handover: bool,
    /// An air transport's unload replaces a destination that passes the landing test (S-0501 claim 40's F7) by the replacement search around map waypoint 700, and takes the height-based approach or reselect transition for one that fails it; off: only a failing destination is replaced, and a passing one takes the approach or reselect transition.
    // spec: S-0507 claim 13, Edge cases
    #[serde(default = "on")]
    pub air_unload_reselects_clear_destination: bool,
    /// A braking step is floored at 0 and can cross below a positive goal; off: floored at the goal.
    // spec: S-0503 F2, Edge cases
    #[serde(default = "on")]
    pub hover_brake_undershoot: bool,
    /// A next heading point gains 416 leptons whenever the vehicle is 312 or more above its terrain, bridge or not; off: a high-bridge cell is needed too.
    // spec: S-0503 Edge cases
    #[serde(default = "on")]
    pub hover_elevated_heading_point: bool,
    /// A zero step length while heading exactly at the heading point sets the throttle and fraction to 1 at once; off: no reset.
    // spec: S-0503 Edge cases
    #[serde(default = "on")]
    pub hover_stalled_start_reset: bool,
    /// Clearing the last heading point at an arrival with no destination and no choice keeps its old reservation; off: that reservation is released and the actual position is marked instead.
    // spec: S-0503 claim 15, Edge cases
    #[serde(default = "on")]
    pub hover_idle_arrival_reservation: bool,
    /// Clearing a vehicle-present flag picks the deck by the point's height alone, while setting also needs a high bridge; off: clearing needs the high bridge too.
    // spec: S-0503 claim 13, Edge cases
    #[serde(default = "on")]
    pub hover_flag_clear_height_only: bool,
    /// A landing-first in-place hover whose first deploy step found it above the ground keeps its landing request while it hovers, so it never deploys and *unload* stays current; off: a hover no longer moving treats a standing landing request as met, and its next deploy step deploys it by S-0517 claim 4's height-0 rows.
    // spec: S-0503 claim 19, Edge cases
    #[serde(default = "on")]
    pub hover_landing_request_stands: bool,
    /// A hover that becomes deployed in place while it still holds movement finishes that movement deployed and then stands; off: becoming deployed drops *D* and the stored path, and the hover finishes only the leg to its current heading point.
    // spec: S-0503 claim 19, Edge cases
    #[serde(default = "on")]
    pub hover_deployed_keeps_moving: bool,
    /// The repair cursor offers repair at an enabled first nearby link that has neither endpoint at the traversal cell; off: such links are skipped and the first link with that endpoint decides, none giving no cursor.
    // spec: S-0511 claim 21, Edge cases
    #[serde(default = "on")]
    pub bridge_cursor_unrelated_link: bool,
    /// The last cell along a band is skipped and a one-cell band raises nothing; off: the band is processed to its last cell.
    // spec: S-0511 claim 8, Edge cases
    #[serde(default = "on")]
    pub bridge_event_band_short: bool,
    /// The bridge family comes from the 5 x 5 block while the high-bridge search reaches 3 cells; off: from the cell claim 16's table finds.
    // spec: S-0511 claim 16, Edge cases
    #[serde(default = "on")]
    pub bridge_family_by_block: bool,
    /// A span whose far piece is 30 or more cell steps away is never checked nor repaired; off: no limit.
    // spec: S-0511 claims 7 and 19, Edge cases
    #[serde(default = "on")]
    pub bridge_span_cap: bool,
    /// A span check passes over collapsed piers at both ends, so a pier collapse can join two spans into one checked extent; off: a collapsed pier stays eligible as a start or far piece.
    // spec: S-0511 claim 7, Edge cases
    #[serde(default = "on")]
    pub collapsed_pier_span_search: bool,
    /// A low piece of the other axis along the line is re-evaluated with this line's table; off: not re-evaluated.
    // spec: S-0511 Edge cases
    #[serde(default = "on")]
    pub low_bridge_wrong_axis_recheck: bool,
    /// Re-evaluation can turn a damaged strip back to an intact look; off: never from damaged to intact.
    // spec: S-0511 claim 14, Edge cases
    #[serde(default = "on")]
    pub low_strip_regress: bool,
    /// A pier chipped on one side ignores a loss notice from that side; off: any pier in state 0 to 2 turns to 3.
    // spec: S-0511 claim 6, Edge cases
    #[serde(default = "on")]
    pub pier_loss_notice_skips_chipped: bool,
    /// Repair raises only the pier column after a collapse lowered the pier's whole tile; off: the whole tile is restored.
    // spec: S-0511 Edge cases
    #[serde(default = "on")]
    pub pier_repair_partial_height: bool,
    /// An engineer entering a hut whose bridge needs no repair is used up; off: it is refused entry.
    // spec: S-0511 claims 17 and 18, Edge cases
    #[serde(default = "on")]
    pub repair_consumes_engineer_always: bool,
    /// The repair walk passes over other bridges' cells and can repair a different bridge; off: it stops at the first cell that is not part of a bridge.
    // spec: S-0511 claim 18, Edge cases
    #[serde(default = "on")]
    pub repair_walk_unbounded: bool,
    /// The type's cloak flag is ignored for aircraft and buildings; off: they can cloak by themselves.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub cloak_flag_ground_mobile_only: bool,
    /// With a cloaking speed of 0 cloaking advances a stage per tick and uncloaking never advances; off: fixed.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub cloak_zero_speed_stall: bool,
    /// Healing or a zero-damage hit uncloaks; off: only hits that lower health.
    // spec: S-0516 claim 12, Edge cases
    #[serde(default = "on")]
    pub heal_reveals_cloaked: bool,
    /// An uncloak started while cloaking takes the full duration; off: it starts from the current stage.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub uncloak_from_cloaking_full_duration: bool,
    /// Allied sensor units stop a building cloaking and a sensors ability does not; off: only non-allied sensors, by flag or ability.
    // spec: S-0516 claim 18, Edge cases
    #[serde(default = "on")]
    pub building_sensor_ring_owner_only: bool,
    /// Overlapping cloak fields lose cells as in the original; off: a cell stays while any powered generator of the house covers it.
    // spec: S-0516 claim 20, Edge cases
    #[serde(default = "on")]
    pub cloak_field_overlap_loss: bool,
    /// A growing or shrinking cloak field stops its update after an own building's newly changed centre cell; off: each circle is applied in full.
    // spec: S-0516 claim 20, Edge cases
    #[serde(default = "on")]
    pub cloak_field_building_interrupt: bool,
    /// An accepted manual power-off of a cloak generator makes shrinking pending even at radius 0, a current radius of 0 becoming the type radius first (no coverage is added); off: shrinking is requested only at a positive current radius, as for an ordinary loss of operation.
    // spec: S-0516 claim 20, Edge cases
    #[serde(default = "on")]
    pub cloak_generator_zero_radius_shutdown: bool,
    /// A mobile sensor leaving play removes its disc around its current cell; off: around the cell where it was last added.
    // spec: S-0516 claim 22, Edge cases
    #[serde(default = "on")]
    pub mobile_sensor_removal_centre: bool,
    /// Ordinary flying movement never relocates a mobile sensor's disc, even on landing; off: each horizontal cell-boundary crossing in flight relocates it to the new cell (coverage only, no adjacency uncloak).
    // spec: S-0516 claim 22, Edge cases
    #[serde(default = "on")]
    pub flying_sensor_disc_stationary: bool,
    /// The neighbouring-infantry check runs on 7 ticks of 8; off: only on multiples of 8.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub mirage_adjacent_check_cadence: bool,
    /// A sensor array's removal decrements over its cloak-radius disc; off: over the disc it added.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub sensor_array_removal_radius: bool,
    /// Sensor and disguise-detector coverage is set at first placement only; off: it follows power and capture.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub sensor_building_activation_once: bool,
    /// Building removals have no floor, so a cell's sensor count can go negative; off: removal undoes exactly what was added.
    // spec: S-0516 Edge cases
    #[serde(default = "on")]
    pub sensor_count_negative: bool,
    /// An edge-cell selection without an acceptable candidate (a paradrop's arrival cell, an aircraft's retreat or spy-plane exit destination) uses the local point (1, ⌊width / 2⌋), with coordinates reversed from the north midpoint; off: (⌊width / 2⌋, 1). Implemented for the paradrop arrival cells and the aircraft edge cells of S-0501 claim 34.
    // spec: S-0522 Edge cases
    #[serde(default = "on")]
    pub paradrop_fallback_point: bool,
    /// A falling object uses the parachute fall recurrence while any animation is attached to it; off: only while the parachute animation is attached. The parachute is the only animation our simulation attaches, so both settings behave alike.
    // spec: S-0522 Edge cases
    #[serde(default = "on")]
    pub paratrooper_any_animation_slow_fall: bool,
    /// A tech building's income cycle comes when one tick of its countdown is left, so the period is D − 1 ticks; off: when the full D ticks have passed, so the period is D. Not implemented yet (wave 2).
    // spec: S-0515 claim 5, F1 and Edge cases
    #[serde(default = "on")]
    pub cash_cycle_one_short: bool,
    /// An income cycle that falls while the building is being erased is lost with all later cycles until the next ownership change; off: the building's first turn at or after that tick has the cycle, and cycles continue from there. Not implemented yet (wave 2).
    // spec: S-0515 claim 6 and Edge cases
    #[serde(default = "on")]
    pub cash_cycle_lapse: bool,
    /// A docked vehicle ordered away keeps receiving paid repair steps wherever it stands still; off: a move or attack order ends its contact as for an approaching vehicle, so steps happen only at the depot.
    // spec: S-0314 claim 21 and Edge cases; S-0504 claim 14
    #[serde(default = "on")]
    pub depot_remote_repair: bool,
    /// After its depot is destroyed, a contacted vehicle after its movement arrival in the docking-point cell, docked or not, keeps its behaviour: one in sleep stays asleep and neither fires nor reacts until ordered, one in guard stays in guard; off: destruction gives such contacts the sale's run-away notice and one in sleep switches to guard.
    // spec: S-0314 claim 22 and Edge cases; S-0504 claim 14
    #[serde(default = "on")]
    pub depot_loss_leaves_sleep: bool,
    /// A hovering vehicle whose depot is destroyed after it became settled, or sold with no destination ending its being settled before the depot's removal (one it receives while still above the ground does not), stays settled and switched off and, once down on the ground, never moves again; off: losing contact with a depot ends its being settled, so it is switched on again and rises.
    // spec: S-0314 claim 22 and Edge cases
    #[serde(default = "on")]
    pub depot_loss_strands_hover: bool,
    /// A hovering vehicle docked at a naval yard on water whose switch-off spin ends less than 104 leptons above the water is lost, as at any other spin end over water; off: a spin end that comes while it has contact with a depot or is settled at a depot (by its arrival, a settling processing, or a disabling in contact or in a depot's cell) makes no water test, and a later spin end after its contact has ended and it is not settled at a depot is tested as usual, as is that of a hovering vehicle disabled away from any depot.
    // spec: S-0314 claim 21, Edge cases; S-0503 claim 11
    #[serde(default = "on")]
    pub docked_hover_water_loss: bool,
    /// An engineer entering a building that is neither arrival-friendly nor capturable is used up and changes nothing; off: the entry is refused as in claim 4 and the engineer stays.
    // spec: S-0509 Edge cases (claim 3 row 5)
    #[serde(default = "on")]
    pub engineer_lost_on_uncapturable: bool,
    /// An engineer reaching a damaged arrival-friendly building during its sale repairs it and is used up; off: sale-time entry is refused as in claim 4, regardless of health.
    // spec: S-0509 Edge cases (proposed, pending the lead's sign-off)
    #[serde(default = "on")]
    pub engineer_repairs_during_sale: bool,
    /// A garrison's exit search never tries the corner cell (x₀ − 1, y₀ − 1); off: the corner is tried last.
    // spec: S-0508 Edge cases (claim 10)
    #[serde(default = "on")]
    pub garrison_exit_skips_corner: bool,
    /// A garrison's exit candidate at terrain level −6 or lower qualifies without the later entry tests; off: it gets the ordinary terrain, usable-area and occupancy tests.
    // spec: S-0508 Edge cases (claim 10)
    #[serde(default = "on")]
    pub garrison_negative_level_exit: bool,
    /// An eviction without inside fallback and with no qualifying exit cell removes every occupant; off: the eviction does nothing and the occupants stay inside.
    // spec: S-0508 Edge cases (claim 12)
    #[serde(default = "on")]
    pub garrison_no_exit_removes: bool,
    /// An infiltrated building type that matches several effect rows gives only the first; off: every matching row applies in table order, and E3 requires only tech-lab-list membership.
    // spec: S-0510 claim 3 and Edge cases
    #[serde(default = "on")]
    pub infiltration_single_effect: bool,
    /// A spy's thief gains the computed amount even when the victim pays less or the amount is negative; off: the amount is clamped to between 0 and the victim's balance.
    // spec: S-0510 claim 6 and Edge cases
    #[serde(default = "on")]
    pub spy_theft_unbacked_gain: bool,
    /// A paradrop's west scan starts at local row r − 1, which is row −1 when r = 0; off: it starts at row r modulo 2·height and continues cyclically. The draw is unchanged. Implemented for the paradrop arrival cells and the aircraft edge cells of S-0501 claim 34.
    // spec: S-0522 claim 4 and Edge cases
    #[serde(default = "on")]
    pub paradrop_west_edge_start_row: bool,
    /// A second-stage superweapon order is accepted unchecked; off: it is refused unless the first stage is granted, not suspended and has an unconsumed source selection.
    // spec: S-0512 claim 10; S-0213 table T1 (fire superweapon)
    #[serde(default = "on")]
    pub second_stage_order_unchecked: bool,
    /// The silo's weapon, a teleport-source pick's armed type and the radar's superweapon offers use a kind number as a position in the superweapon type list; off: each uses the type of that kind.
    // spec: S-0513 claim 8 and Edge cases; S-0410 claim 12
    #[serde(default = "on")]
    pub superweapon_kind_as_list_position: bool,
    /// A superweapon teleport's arrival hold uses the member's held delay, which nothing resets after a redirected landing; off: the hold is the teleport delay after a redirected landing and 0 otherwise, for each warp.
    // spec: S-0513 claim 40; S-0214 Terms (being erased, phasing in)
    #[serde(default = "on")]
    pub warp_hold_persists: bool,
    /// A temporal projectile in flight at a superweapon warp's launch, or a fragment, locks a beam on a member; the warp and the erasure both go on, and from its departure the member is no longer being erased while the beam stays attached, up to a later lock; off: a pending warp, from the launch until the departure update, is one more lock refusal.
    // spec: S-0513 claim 39a and Edge cases; S-0514 claims 3, 9, 14 and 17
    #[serde(default = "on")]
    pub warp_departure_keeps_beam: bool,
    /// While it phases in, a vehicle or aircraft with a temporal beam attached has its turns cut short whether or not it is being erased, and an infantry unit only while it is being erased; off: an infantry unit with an attached beam has its turns cut short while it phases in as well.
    // spec: S-0513 claim 39a and Edge cases; S-0514 claim 17; S-0610 claim 5
    #[serde(default = "on")]
    pub phase_in_beam_spares_infantry: bool,
    /// A deployed deploy-to-fire vehicle with the in-place deploy flag queues *unload* again instead of firing, so it cycles between deploying and undeploying and never fires; off: a deployed deploy-to-fire vehicle fires normally, and only a mobile one queues *unload*. Not implemented yet (wave 2).
    // spec: S-0517 claim 13 and Edge cases; S-0310 claim 9 and Edge cases
    #[serde(default = "on")]
    pub deploy_to_fire_repacks: bool,
    /// An assault that clears garrison occupants records the surviving assaulter as destroyed by each removed occupant, which gets the kill credit and experience; off: each removed occupant is the victim, with destruction notifications and loss bookkeeping, and the assaulter gets ordinary kill credit and experience. The assaulter survives in both settings.
    // spec: S-0508 claim 14 and Edge cases; S-0311 claim 9
    #[serde(default = "on")]
    pub assaulter_reversed_kill_credit: bool,
    /// A trainable erasing leader gets F5's direct experience gain on top of ordinary kill credit, which may go elsewhere; off: only ordinary S-0311 credit remains.
    // spec: S-0514 claim 13 and Edge cases; S-0311 claim 9
    #[serde(default = "on")]
    pub erasure_double_experience: bool,
    /// When the last erasing attacker lets go before completion, the building is switched on regardless of its state before the lock; off: release restores its switch state from before the first lock.
    // spec: S-0514 claim 10 and Edge cases; S-0405 claim 8
    #[serde(default = "on")]
    pub erasure_release_switches_on: bool,
    /// Only a leading erasing attacker inside an open-topped transport is checked against the warp distance, not joiners; off: the distance limit applies to every attached open-topped passenger.
    // spec: S-0514 claim 11 and Edge cases
    #[serde(default = "on")]
    pub erasure_distance_leader_only: bool,
    /// A delivered erasing hit clears the target's spawner slots and frees the units it mind-controls even when the new lock is refused; off: a refused lock does not clear spawners or free controlled units (the firer's previous beam still lets go).
    // spec: S-0514 claim 2 and Edge cases
    #[serde(default = "on")]
    pub erasure_refused_hit_side_effects: bool,
    /// An own-cell early cruise keeps the height above the surface r as its target even when the take-off threshold used the smaller bridge-adjusted h, so it can climb again above the deck; off: it keeps h at that transition.
    // spec: S-0502 claim 7 and Edge cases
    #[serde(default = "on")]
    pub jumpjet_bridge_early_cruise_height: bool,
    /// The look-ahead adds the bridge height of the cell ahead to the jumpjet's own cell's clearance, not to the ahead cell's; off: each cell's bridge height is added to that cell's clearance.
    // spec: S-0502 claim 9, F3 and Edge cases
    #[serde(default = "on")]
    pub jumpjet_lookahead_bridge: bool,
    /// A jumpjet that cannot land and finds no cell nearby destroys itself; off: it clears its movement point and commitment and, at a height above the surface of 1 lepton or more, re-enters *hovering* over its current cell, neither descending nor flying on, while at 0 or less it stays put and becomes *grounded*, even from *ascending* (every seized set-down unit hovering in this outcome, by any route, then follows S-0607 claims 9 and 20).
    // spec: S-0502 claim 17 and Edge cases
    #[serde(default = "on")]
    pub jumpjet_no_cell_self_destruct: bool,
    /// A straight-line step uses F5's shifted angle, table trigonometry and separate coordinate truncation, so southward and westward steps drift one lepton across the line of travel; off: the displacement is the speed times the double-precision sine and cosine of the facing, rounded half away from zero.
    // spec: S-0502 claim 10, F5 and Edge cases
    #[serde(default = "on")]
    pub jumpjet_step_truncation: bool,
    /// The cruise bands' remaining-turn test uses the clockwise difference only, so even a small anticlockwise turn takes the band's slower speed; off: it uses the smaller of the clockwise and anticlockwise differences.
    // spec: S-0502 claim 11, F6 and Edge cases
    #[serde(default = "on")]
    pub jumpjet_turn_slowdown_one_sided: bool,
    /// Garrison entry is refused only when the occupant count equals the capacity, so a negative capacity admits occupants indefinitely; off: entry requires an occupant count below the capacity, so a non-positive capacity admits none.
    // spec: S-0508 claim 1 and Edge cases
    #[serde(default = "on")]
    pub garrison_capacity_equality: bool,
    /// Each deploy or undeploy processing requests the type's deploy or undeploy sound, so landing and transitions repeat the request every processing; off: the deploy sound is requested once, at the first step that starts landing, a transition or an immediate deploy, and the undeploy sound once when an undeploy step starts. Not implemented yet (wave 2).
    // spec: S-0517 claim 12 and Edge cases
    #[serde(default = "on")]
    pub deploy_sound_repeats: bool,
    /// With a presentation stage delay of 0 no stages advance, so a deploy with E > 1 or an undeploy with E > 2 never completes; off: any zero-delay transition completes at the next processing. Not implemented yet (wave 2).
    // spec: S-0517 claim 7 and Edge cases; S-0109 Edge cases
    #[serde(default = "on")]
    pub deploy_stall_zero_delay: bool,
    /// A rate above 900 converts to a stage delay of ⌊900 / ρ⌋ = 0, so the animation never advances or ends through playback; off: a positive rate gives max(1, ⌊900 / ρ⌋). Rates of 0 or below give 0 in both settings. Built for the random-rate conversion of an animation's creation draw (S-0109 claim 3) and the playback of infantry-making animations in the simulation (S-0611 claim 8); the S-0517 claim 7 deploy delay is not implemented yet.
    // spec: S-0109 claim 2, F1 and Edge cases
    #[serde(default = "on")]
    pub anim_fast_rate_freezes: bool,
    /// A converted random rate pair is clamped to [min(d₁, max(d₂, 0)), max(d₂, 0)], so an ascending pair collapses to its faster end and takes no draw; off: the converted pair is put in order and the delay is a ranged draw over [min(d₁, d₂), max(d₁, d₂)], with no draw when they are equal. Changes the number of draws.
    // spec: S-0109 claim 3, F1 and Edge cases
    #[serde(default = "on")]
    pub anim_random_rate_collapse: bool,
    /// An unlinked rocking angle is held at its limit only when the angle before the step was
    /// below it, so repeated pushes can carry a body past its limit and over; off: an unlinked
    /// angle that is not past the edge is held at its limit whenever the new angle would pass it.
    /// Read by the client's rocking (the sim keeps no rocking state yet).
    // spec: S-0113 claim 15 and Edge cases (quirks.rocking_limit_escape, default on)
    #[serde(default = "on")]
    pub rocking_limit_escape: bool,
    /// A superweapon building's slots read the owner's record clock whether or not the record is
    /// granted, so a record never granted reads 0 and the grand opening shows the charged
    /// *super-3*; off: a record that is not granted counts as not charged (*super-1* at the
    /// opening, and the charge check waits for the grant).
    // spec: S-0108 claims 11 and 18, Edge cases (quirks.ungranted_super_reads_clock, default on)
    #[serde(default = "on")]
    pub ungranted_super_reads_clock: bool,
    /// A behaviour queued during deploying or undeploying is replaced by *guard* at completion; off: completion starts the pending behaviour, or *guard* if none is pending. Not implemented yet (wave 2).
    // spec: S-0517 claim 8 and Edge cases
    #[serde(default = "on")]
    pub deploy_transition_discards_orders: bool,
    /// A dropped or projectile-less delayed master shot keeps the support count, so the next shot gets the kept bonus on top of new supporters, with no beams for the kept part; off: the count becomes 0 whenever the delayed shot comes, fired or not.
    // spec: S-0518 claim 10 and Edge cases
    #[serde(default = "on")]
    pub prism_count_survives_dropped_shot: bool,
    /// Every rules layer with a general section re-reads the support modifier with the held value as its default and multiplies the result by 100, even when the layer lacks the key; off: a layer without the key leaves the held value unchanged.
    // spec: S-0518 claim 16, F4 and Edge cases
    #[serde(default = "on")]
    pub prism_modifier_reread_scale: bool,
    /// Supporters need not be in operation, so a switched-off, pulsed, unfinished or selling building still supports a working master; off: a building not in operation (S-0405 claim 8) is not eligible.
    // spec: S-0518 claim 2 and Edge cases
    #[serde(default = "on")]
    pub prism_support_ignores_operation: bool,
    /// With a prism delayed-fire field of 0, a recruited supporter stays eligible until its turn, so another master of the same house processed before then also counts it; off: a building with a pending beam is not eligible.
    // spec: S-0518 claim 2 and Edge cases
    #[serde(default = "on")]
    pub prism_zero_delay_double_count: bool,
    /// A building of a non-overpowerable type never removes chargers that stopped aiming at it; off: it removes them as an overpowerable type does.
    // spec: S-0518 claim 12 and Edge cases
    #[serde(default = "on")]
    pub stale_chargers_kept: bool,
    /// A vehicle with an electric-charge secondary has no break-off search and keeps firing charge shots at the building while enemies approach; off: vehicles break off as infantry do.
    // spec: S-0518 claim 15 and Edge cases
    #[serde(default = "on")]
    pub vehicle_charge_no_break_off: bool,
    /// Only the type's explosive-charges flag plants a charge on arrival, and the veterancy explosive-charges ability plants nothing; off: the veterancy ability also plants, as the type flag does.
    // spec: S-0519 claim 13 and Edge cases
    #[serde(default = "on")]
    pub charge_needs_type_flag: bool,
    /// Armour, firepower and speed crates also boost objects off the map whose last recorded position lies inside the crate area; off: only objects on the map are affected, as for cloak and veteran.
    // spec: S-0520 claims 19 and 20 and Edge cases
    #[serde(default = "on")]
    pub crate_bonus_reaches_limbo: bool,
    /// The unit crate's harvester rule tests only the first two entries of the refinery list and of the harvester list; off: every entry of both lists is tested.
    // spec: S-0520 claim 13 and Edge cases
    #[serde(default = "on")]
    pub crate_harvester_list_prefix: bool,
    /// A selected crate cell whose overlay a terrain object prevents still occupies its crate place, with its lifetime draw, until expiry; off: the failed overlay placement takes no place or lifetime draw. Unreachable in our engine: no terrain object blocks an overlay on a cell the placement search accepted, so both settings act alike.
    // spec: S-0520 claim 3 and Edge cases
    #[serde(default = "on")]
    pub crate_phantom_place: bool,
    /// A unit crate that cannot place its unit gives money with the unit power-up's argument as the base b; off: with the money power-up's argument, as a money crate would.
    // spec: S-0520 claim 13 and Edge cases
    #[serde(default = "on")]
    pub crate_unit_fallback_unit_argument: bool,
    /// Adding to a radiation field removes F5's amount r from its cells, which can exceed the field's remaining contribution by one decay step and take radiation other fields gave; off: adding removes exactly the field's own remaining contribution.
    // spec: S-0525 claim 4, F5 and Edge cases
    #[serde(default = "on")]
    pub radiation_add_overdraw: bool,
    /// A radiation field's end takes nothing from its cells, so a contribution its decay steps did not take stays and keeps damaging; off: at its end each cell loses what remains of that field's contribution, not below 0.
    // spec: S-0525 claim 6 and Edge cases
    #[serde(default = "on")]
    pub radiation_residue: bool,
    /// A strike start that places no aircraft (an elite strike size of 0 or less, or a first aircraft that cannot be created) still leaves the strike in progress, so no aircraft can ever end it and later designations only redirect it; off: such a start leaves the strike idle, with its strike target and end tick unchanged.
    // spec: S-0519 claim 17 and Edge cases
    #[serde(default = "on")]
    pub airstrike_empty_strike_stuck: bool,
    /// The recharge delay holds back only a repeat designation of the same object with no change of target in between, so a cancelled or redirected idle strike is ready at once for any object; off: an idle strike that has ended before is ready only when t ≥ e + R, whatever its target.
    // spec: S-0519 claim 18
    #[serde(default = "on")]
    pub airstrike_recharge_same_target_only: bool,
    /// A controller that changes owner keeps its victims, which keep their owner and original owners and stay on its list; off: an owner change releases all victims first, as claim 10 does (`glasnost_sim::mind::owner_changed`).
    // spec: S-0601 claim 11 and Edge cases
    #[serde(default = "on")]
    pub mind_control_owner_change_keeps_victims: bool,
    /// A dying victim holds its controller slot for as long as it still exists as the same object (a sinking ship, a building awaiting delayed destruction); off: a victim leaves its controller's list at its killing hit (`glasnost_sim::mind::killed`).
    // spec: S-0601 claim 12 and Edge cases
    #[serde(default = "on")]
    pub dying_victim_holds_slot: bool,
    /// A controlled vehicle that deploys into a building leaves its controller's list silently and the owner change becomes permanent; off: the new object inherits the control link, the original owner and the list place, and is released as the old object would have been (`glasnost_sim::mind::converted`).
    // spec: S-0601 claim 15 and Edge cases
    #[serde(default = "on")]
    pub deploy_makes_control_permanent: bool,
    /// A spin-down update whose amount d × m is 0 sets the spin value to 0 at once; off: an amount of 0 leaves the value unchanged (`glasnost_sim::gattling::spin_down`).
    // spec: S-0609 claims 3 and 9, F2 and Edge cases
    #[serde(default = "on")]
    pub gattling_zero_spin_down_empties: bool,
    /// A berserk hit whose duration T is 0 or less still makes the target drop its target and queue *hunt*, berserk until its next update; off: such a hit has no effect on the target (`glasnost_sim::psychic::enter`).
    // spec: S-0610 claim 5 and Edge cases
    #[serde(default = "on")]
    pub berserk_zero_duration: bool,
    /// Hostile-intent lines are shown for units the viewer cannot see, including cloaked, disguised and shrouded units; off: a line is shown only for a unit visible to the viewer's house (`glasnost_sim::psychic::intent_line`).
    // spec: S-0610 claim 10 and Edge cases
    #[serde(default = "on")]
    pub psychic_intent_ignores_visibility: bool,
    /// Wind direction 3 pushes a gas particle 1 lepton in x instead of 2, unlike its mirror direction 5; off: Δx = 2 for direction 3.
    // spec: S-0613 claim 12, Constants and Edge cases
    #[serde(default = "on")]
    pub wind_southeast_short: bool,
    /// A kill by a fire particle has no attacking house, though its system may have one; off: the hit's attacking house is the system's owner house, as for gas (`glasnost_sim::particles`).
    // spec: S-0616 claim 14 and Edge cases
    #[serde(default = "on")]
    pub fire_particle_no_house: bool,
    /// A smoke system emits however many particles it holds, whatever its type's particle-cap field; off: while it holds as many particles as the cap, an emission takes its two raw values and grows the timer but makes no particle (`glasnost_sim::particles`).
    // spec: S-0616 claim 9 and Edge cases
    #[serde(default = "on")]
    pub smoke_ignores_particle_cap: bool,
    /// A spark's vertical step counts gravity twice in a tick, its stored vertical velocity once; off: the step uses the stored vertical velocity (`glasnost_sim::particles`).
    // spec: S-0616 claim 16, F11 and Edge cases
    #[serde(default = "on")]
    pub spark_double_gravity: bool,
    /// A smoke particle stopped at a bridge deck takes one more update with its draws and is removed (with its successors) in the next turn; off: it is removed in the pass that stops it, with no further update draws (`glasnost_sim::particles`).
    // spec: S-0616 claim 7 and Edge cases
    #[serde(default = "on")]
    pub smoke_deck_extra_update: bool,
    /// The flame sweep's facing constants are 16383 and 65534; off: 16384 and 65536 (`glasnost_sim::particles::flame_sweep`).
    // spec: S-0616 F5 and Edge cases
    #[serde(default = "on")]
    pub flame_sweep_facing: bool,
    /// When the first other object under a dropped unit is an infantry unit, every object after it in the cell's list is also removed, whatever its kind, the dropped unit included when it is listed after the infantry; off: only infantry units in the cell are removed, and the dropped unit never is.
    // spec: S-0607 claim 16 and Edge cases
    #[serde(default = "on")]
    pub lift_drop_erases_cell_tail: bool,
    /// A dropped unit that ends its fall in *crashed* leaves play after the two impact detonations even when they did not kill it; off: a unit that survives both detonations stays in play, is set down and gets its own locomotor back.
    // spec: S-0607 claims 18 and 20 and Edge cases
    #[serde(default = "on")]
    pub lift_impact_removes_survivor: bool,
    /// The impact credit of a dropped unit's fall is given whether or not one of the two impact detonations has already killed it, so such a unit is credited twice: once to the detonation's source and once more to the dropper (else the dropper house), in the kill and loss tallies, the score and the experience; off: the impact credit is skipped for a unit a detonation has already killed.
    // spec: S-0607 claim 18 and Edge cases
    #[serde(default = "on")]
    pub lift_impact_double_credit: bool,
    /// When a dropped unit's fall ends on a bridge deck, the objects listed in the ground layer under the bridge decide what lies underneath, not those on the deck; off: the layer the unit landed in decides.
    // spec: S-0607 claim 16 and Edge cases
    #[serde(default = "on")]
    pub lift_deck_impact_reads_ground: bool,
    /// A unit seized with the driving, walking, ship or teleporting stand-in locomotor is never restored, and one seized with the drop-pod stand-in gets its own locomotor back but stays seized, so either way it ignores destinations and cannot fire until it leaves play; off: claim 20's restoration applies to every stand-in, with that stand-in's own moving test, and clears the seized state.
    // spec: S-0607 claim 24 and Edge cases
    #[serde(default = "on")]
    pub lift_standin_stays_seized: bool,
    /// A unit seized with the driving, walking or ship stand-in locomotor gives up the vehicle-present mark of its cell at the seize and holds none until its stand-in's own movement starts a step into a neighbouring cell, so one that never moves stands unmarked for good; its listed presence still decides vehicle entry outcomes, but rules that read only the occupancy record, such as the nearby-cell search, see its cell as free; off: it keeps the vehicle-present mark of its own cell at the seize, as a vehicle at rest does (the teleporting stand-in is unchanged).
    // spec: S-0607 claim 23 and Edge cases
    #[serde(default = "on")]
    pub lift_standin_unmarked_cell: bool,
    /// A recycling building tests nothing about itself or its owner when an entrant arrives, so a building being sold, switched off or now owned by a house that does not count the entrant's house as allied still consumes the entrant and pays its owner; off: at consumption the building gives claim 2's docking answer to every entrant, infantry included, without the link-place row, and a refused entrant is not consumed, its destination is cleared and it falls back to rest.
    // spec: S-0603 claim 5 and Edge cases
    #[serde(default = "on")]
    pub grinder_no_arrival_recheck: bool,
    /// An infantry entrant that reaches an absorbing building goes in whatever the passenger count, the switch or its size, so with two or more link places the building can hold more than its capacity; off: at completion the building applies claim 1's acceptance test again, and a refused entrant is treated as a refused vehicle.
    // spec: S-0605 claim 6 and Edge cases
    #[serde(default = "on")]
    pub absorber_contact_no_capacity_recheck: bool,
    /// When the eighth direction of the first round of an absorbing building's exit search passes both cell tests but its neighbour cell carries a bridge deck, the second round never starts and the passenger stays inside this processing; off: the second round starts as after any other failure at the eighth direction.
    // spec: S-0605 claim 12 and Edge cases
    #[serde(default = "on")]
    pub absorber_release_bridge_ends_search: bool,
    /// The second round of an absorbing building's exit search does not try the rear direction; off: the second round tries the rear direction first.
    // spec: S-0605 claim 12 and Edge cases
    #[serde(default = "on")]
    pub absorber_release_second_round_skips_rear: bool,
    /// An actor that stops being in *enter* with its contact with an absorbing building intact keeps the building's link place for good, so every later entrant waits in line for ever; off: in its own turn, after its movement, an actor in contact with an absorbing building that has neither *enter* current nor *enter* queued gives the contact up (the window in which a waiting actor has been granted contact and has *enter* queued but not yet started keeps it).
    // spec: S-0605 claim 17 and Edge cases
    #[serde(default = "on")]
    pub absorber_stale_contact_kept: bool,
    /// A passenger removed while inside an absorbing building does not make the building owner's power due, so the owner's totals keep its contribution until something else makes them due; off: the removal makes the building owner's power due, as a release does, when the type's extra output is above 0.
    // spec: S-0605 claim 15 and Edge cases
    #[serde(default = "on")]
    pub absorber_removal_keeps_stale_power: bool,
    /// A unit of the neutral or special house that took the enter-nearest instruction through the bio-reactor fate keeps it when it goes in and when it is let out, so a released passenger walks straight back in; off: going in clears the instruction.
    // spec: S-0605 claims 13 and 16 and Edge cases
    #[serde(default = "on")]
    pub enter_nearest_survives_absorption: bool,
    /// The clearing and waiting rows of a tank bunker's entry look only at the single chosen object of each foundation cell (F5), so when the entering vehicle is that object a second object in the cell is neither asked to scatter nor waited for; off: every candidate other than the entering vehicle on each foundation cell gets the scatter request, and the waiting row waits until none is left.
    // spec: S-0606 claim 6 and Edge cases (revision 5: renamed from bunker_clear_nearest_only)
    #[serde(default = "on")]
    pub bunker_clear_chosen_only: bool,
    /// A tank bunker's clearing and waiting rows choose each foundation cell's object by its distance from a point 896 leptons from the cell's top corner along each map axis, outside the cell, so the object nearest the cell's bottom corner is chosen; off: the distance is measured from the cell's centre (128 leptons per axis), with the same table root and tie rule.
    // spec: S-0606 claim 6, F5 and Edge cases
    #[serde(default = "on")]
    pub bunker_clear_far_point: bool,
    /// A deploy order to an airborne in-place deploying type without the deploy-to-land flag is silently dropped; off: the order is kept until the vehicle is on the ground, and the next processing takes the ground deploy step of claim 4. Not implemented yet (wave 2).
    // spec: S-0517 claim 5 and Edge cases
    #[serde(default = "on")]
    pub airborne_deploy_order_dropped: bool,
    /// A drain attempt that forms a link while the drainer is still linked to another building does not end the first link, which stays drained (blackout, refused firing, sale refusal, payouts) until the drainer is removed; off: forming a new link first ends the old one with all of claim 14's effects.
    // spec: S-0608 claim 4 and Edge cases
    #[serde(default = "on")]
    pub drain_relink_strands_previous: bool,
    /// Only a drainer that is a vehicle performs the position check, so an infantry or aircraft drainer keeps its link after it moves away until claim 11 or 13 ends it; off: every drainer performs it.
    // spec: S-0608 claim 12 and Edge cases
    #[serde(default = "on")]
    pub drain_position_check_vehicles_only: bool,
    /// In a high-bridge cell of an effect block only the deck layer's objects are reached, elsewhere only the ground layer's; off: both layers of a high-bridge cell are reached, deck-layer effects first.
    // spec: S-0513 claim 4 and Edge cases; S-0611 claim 4 and Edge cases
    #[serde(default = "on")]
    pub superweapon_block_one_layer: bool,
    /// When a reached object leaves its cell immediately during its own effect, later listed objects in that cell are not reached; off: objects listed when the effect begins stay eligible.
    // spec: S-0513 State and ordering and Edge cases; S-0611 claim 4
    #[serde(default = "on")]
    pub superweapon_list_cut_by_removal: bool,
    /// The conversion test's position choice reads the cell's ground record even for a victim on a high-bridge deck; off: a victim on a deck is tested against the deck record.
    // spec: S-0611 claim 6 and Edge cases
    #[serde(default = "on")]
    pub mutation_test_ground_record: bool,
    /// A refused mutation victim's standing position is free from its death tick while it plays die-2, and its removal frees that position again even if a walker took it; off: the position stays taken until removal, which frees only the victim's own mark.
    // spec: S-0611 claim 7 and Edge cases
    #[serde(default = "on")]
    pub refused_mutation_frees_position: bool,
    /// An infantry-making animation marks its position taken and sets the cell's infantry house in every update, and frees the position at its end whatever holds it; off: it holds the position only while no other unit does, leaves the infantry house alone while other infantry stands there and frees only its own mark.
    // spec: S-0611 claim 8 and Edge cases
    #[serde(default = "on")]
    pub infantry_animation_occupancy: bool,
    /// A disc laser's detonation names the firer as attacker but no attacking house; off: the attacking house is the firer's owner, as for a projectile's detonation.
    // spec: S-0615 claim 16 and Edge cases
    #[serde(default = "on")]
    pub disc_laser_no_house: bool,
    /// A disc laser whose weapon has the unlimited range compares its re-check distance with that negative value as it is, so it is cancelled at its first step and never deals damage; off: unlimited range never cancels a disc laser on distance.
    // spec: S-0615 claim 14 and Edge cases
    #[serde(default = "on")]
    pub disc_laser_unlimited_range: bool,
    /// A railgun candidate whose calculated line distance is not a number (exact collinearity among others) qualifies whatever the hit distance; off: the finite Euclidean distance to the beam's line (to its start for a zero-length beam), strictly below the hit distance.
    // spec: S-0306 claim 27 and Edge cases
    #[serde(default = "on")]
    pub railgun_collinear_hits: bool,
    /// An object found by both the flying-object search and a scanned cell of a detonation keeps both entries and can take two hits, the flying entry first; off: it keeps only its flying entry.
    // spec: S-0303 claim 6 and Edge cases
    #[serde(default = "on")]
    pub flying_listed_double_hit: bool,
    /// A jumpjet touching down in the bridge or parachute falling state, or no longer alive or in play, stays a tracked flying object; off: every touchdown ends its registration.
    // spec: S-0502 claim 29 and Edge cases
    #[serde(default = "on")]
    pub jumpjet_falling_touchdown_tracked: bool,
    /// A damaging animation's damage accumulator starts at 1.0, so its first advance already delivers 1 when δ < 1; off: it starts at 0.0. Implemented for the burning walker's expiring advances (S-0615 claim 11), the only animation the simulation plays with damage.
    // spec: S-0109 claim 17, F5 and Edge cases
    #[serde(default = "on")]
    pub anim_damage_starts_at_one: bool,
    /// The remaining-pass count of an animation is taken modulo 256 and 255 means endless; off: an unbounded count, endless for a negative loop count.
    // spec: S-0109 claim 7 and Edge cases
    #[serde(default = "on")]
    pub anim_loop_count_wrap: bool,
    /// A reversed animation starts at the loop end less one, restarts at the loop end and ends its pass at k <= 0; off: it starts and restarts at the loop end less the start field less one and ends its pass at k < 0.
    // spec: S-0109 claim 8 and Edge cases
    #[serde(default = "on")]
    pub anim_reverse_raw_loop_end: bool,
    /// While r >= 2 a ping-pong animation's lower turn compares k with the start field; off: k <= loop start less the start field.
    // spec: S-0109 claim 9 and Edge cases
    #[serde(default = "on")]
    pub anim_pingpong_lower_turn: bool,
    /// An absent loop end takes the sprite end; off: it takes the end field.
    // spec: S-0109 claims 6 and 25 and Edge cases
    #[serde(default = "on")]
    pub anim_loop_end_from_sprite: bool,
    /// A follow-up type starts at k = a'; off: at its own k0 of F3.
    // spec: S-0109 claim 15 and Edge cases
    #[serde(default = "on")]
    pub anim_followup_start_doubled: bool,
    /// The step keeps its sign at a follow-up switch; off: it is set afresh from the reversed test.
    // spec: S-0109 claim 15 and Edge cases
    #[serde(default = "on")]
    pub anim_followup_keeps_step: bool,
    /// A dominator strike's area damage names the current storm owner as attacking house, sampled before the storm's own update; off: the dominating house.
    // spec: S-0612 claim 9 and Edge cases; S-0302 claim 1
    #[serde(default = "on")]
    pub dominator_damage_storm_house: bool,
    /// A dominator strike makes the dominating house's own objects permanently controlled with a ring; off: they are left alone.
    // spec: S-0612 claim 14 and Edge cases; S-0601 claim 13
    #[serde(default = "on")]
    pub dominator_marks_own_objects: bool,
    /// A dominator sweep visits only the ground layer of each swept cell, so an object on a bridge deck is never reached; off: the deck layer of a swept cell is visited after its ground layer, in the same way.
    // spec: S-0612 claim 11 and Edge cases
    #[serde(default = "on")]
    pub dominator_ground_layer_only: bool,
    /// A dominator sweep examines, in each swept cell, only the start object (the nearest to the cell's top corner, S-0207 F8) and the objects listed after it, up to the first listing that is no infantry, vehicle, aircraft or building; off: every listed infantry, vehicle and aircraft of the layer is examined, in listing order.
    // spec: S-0612 claim 12 and Edge cases
    #[serde(default = "on")]
    pub dominator_sweep_from_nearest: bool,
    /// A dominator sweep takes over an examined object that is still in play while dying; off: a dying object is not examined.
    // spec: S-0612 claim 13 and Edge cases
    #[serde(default = "on")]
    pub dominator_takes_dying: bool,
    /// With a zero ambient change rate the current ambient never moves, even for nuke and dominator fades; off: it jumps straight to the target.
    // spec: S-0104 claim 4 and Edge cases (Zero change rate freezes ambient); S-0612 claim 7
    #[serde(default = "on")]
    pub zero_rate_freezes_ambient: bool,
    /// The map's dominator fade rate is stored in thousandths and used as ticks (a map value of 1 means 1000 ticks); off: the map value is a number of ticks, truncated as the nuke rate is.
    // spec: S-0104 claim 4 and Edge cases (Dominator fade rate scale)
    #[serde(default = "on")]
    pub dominator_rate_thousandths: bool,
    /// Removing an upgrade makes the immediate superweapon availability check only when the upgrade names a first superweapon; off: an upgrade naming either superweapon field makes it.
    // spec: S-0512 claim 2 and Edge cases
    #[serde(default = "on")]
    pub superweapon_upgrade_second_removal_waits: bool,
    /// A superweapon granted by buildings while its house has low power starts suspended, whether or not the type is power-dependent, and no other condition suspends it at the grant; off: the grant suspends exactly when the claim 6 test would.
    // spec: S-0512 claim 4 and Edge cases
    #[serde(default = "on")]
    pub superweapon_grant_suspends_by_house_power: bool,
    /// A second-stage launch and a set-charge action start the clock running even on a suspended record; off: a suspended record's clock stays stopped at the new remaining time.
    // spec: S-0512 claims 10 and 14 and Edge cases
    #[serde(default = "on")]
    pub superweapon_restart_ignores_suspension: bool,
    /// A fire order needs a granted, ready record whose clock is running; off: it needs a granted, ready record that is not suspended.
    // spec: S-0512 claim 8 and Edge cases
    #[serde(default = "on")]
    pub superweapon_order_tests_clock: bool,
    /// Setting a superweapon's charge keeps an existing ready state; off: it clears readiness and the new clock completes normally.
    // spec: S-0512 claim 14 and Edge cases
    #[serde(default = "on")]
    pub set_charge_keeps_ready: bool,
    /// Drain-mode orders change the drain state without testing grant, readiness, the clock or suspension, restart the clock even when suspended, and do not consume a one-time record; off: they need a granted record that is not suspended, and a one-time record is consumed on launch.
    // spec: S-0512 claim 19 and Edge cases
    #[serde(default = "on")]
    pub drain_orders_bypass_normal_checks: bool,
    /// A second stage's link names a kind whose number is a position in the superweapon type list; off: the first stage is the first type in the list of that kind.
    // spec: S-0512 claim 11 and Edge cases
    #[serde(default = "on")]
    pub predependent_kind_as_list_position: bool,
    /// A missile superweapon whose house cannot start a launch sequence consumes its charge and launches nothing; off: the order is refused and the record stays ready.
    // spec: S-0512 claim 8 and Edge cases; S-0513 claim 7
    #[serde(default = "on")]
    pub missile_without_launcher_consumes_charge: bool,
    /// A superweapon fire order fires the first type in list order whose aim action is the selected type's; off: it fires the selected type.
    // spec: S-0512 claim 16; S-0513 claim 2 and Edge cases
    #[serde(default = "on")]
    pub superweapon_order_first_type_of_action: bool,
    /// A permanent grant on a suspended record leaves it suspended for good; off: the grant also resumes it.
    // spec: S-0512 claim 13 and Edge cases
    #[serde(default = "on")]
    pub permanent_grant_freezes_suspension: bool,
    /// A one-time grant keeps the suspension of a record lost while suspended; off: it ends any suspension.
    // spec: S-0512 claim 12 and Edge cases
    #[serde(default = "on")]
    pub one_time_grant_keeps_stale_suspension: bool,
    /// Only the first silo type in type-list order that names the superweapon can launch it; off: every such silo type is considered, in type-list order.
    // spec: S-0513 claim 7 and Edge cases
    #[serde(default = "on")]
    pub nuke_silo_first_type_only: bool,
    /// A silo launches at the target last recorded for its current owner; off: at the target recorded when it accepted the launch.
    // spec: S-0513 claim 13 and Edge cases
    #[serde(default = "on")]
    pub silo_target_of_current_owner: bool,
    /// The curtain's hit on an organic vehicle or aircraft does not ignore defences and credits nobody; off: it ignores defences and credits the launching house.
    // spec: S-0513 claim 24 and Edge cases
    #[serde(default = "on")]
    pub curtain_organic_vehicle_defended: bool,
    /// Starting protection on a protected object replaces its timer and kind; off: the longer remaining time is kept, with the kind of the effect that gave it.
    // spec: S-0513 claim 25 and Edge cases
    #[serde(default = "on")]
    pub protection_restart_replaces: bool,
    /// A warp kills an organic object without the teleporter flag even when it is protected; off: a protected organic object is left alone.
    // spec: S-0513 claim 35 and Edge cases
    #[serde(default = "on")]
    pub warp_kill_before_protection: bool,
    /// A blocked warp landing searches with the track locomotion class and a simplified movement zone; off: with the member's own class and zone.
    // spec: S-0513 claim 37 and Edge cases
    #[serde(default = "on")]
    pub warp_redirect_as_tracked: bool,
    /// A naval warp member landing on a road cell sinks; off: a road cell that is water is not counted.
    // spec: S-0513 claim 38 and Edge cases
    #[serde(default = "on")]
    pub warp_naval_road_cell: bool,
    /// The spy plane count is the length of the first side group's paradrop infantry list, and none when that list and its count list differ in length; off: exactly one plane.
    // spec: S-0513 claim 43 and Edge cases
    #[serde(default = "on")]
    pub spy_plane_count_from_paradrop_list: bool,
    /// A starting-building restoration whose forced placement fails removes a displaced unit that cannot be put back at its original position; off: such a unit is kept and placed at a nearby cell instead.
    // spec: S-0524 claim 9 and Edge cases
    #[serde(default = "on")]
    pub restoration_failed_return_removes_unit: bool,
    /// Off (the original): every lightning-strike action makes a cloud, repeated ones at one waypoint included; on: after one cloud made by the action at a waypoint, further actions at that waypoint make none.
    // spec: S-0524 claim 2 and Edge cases (default: off)
    #[serde(default)]
    pub repeated_lightning_strike_suppressed: bool,
    /// Off (the original): the create-crate action ignores the crate setting; on: it has no effect while crates are off. Not consulted yet (crates, S-0520, are not built).
    // spec: S-0524 claim 4 and Edge cases (default: off)
    #[serde(default)]
    pub trigger_crate_respects_setting: bool,
    /// Off (the original): an uncollected trigger crate never expires while crates are off; on: it expires after its assigned lifetime. Not consulted yet (crates, S-0520, are not built).
    // spec: S-0524 claim 4 and Edge cases (default: off)
    #[serde(default)]
    pub trigger_crate_expiry_while_disabled: bool,
}

impl Default for Quirks {
    fn default() -> Self {
        Self {
            chop_rounding: true,
            speed_normalised_delays: true,
            building_wrap_normalisation: true,
            chrono_flash_keeps_order_block: true,
            ranged_draw_half_span: true,
            persistent_event_latch: true,
            event_latch_aliasing: true,
            all_carriers_without_carrier: true,
            entered_by_country_only: true,
            none_owner_is_first_country: true,
            exact_trigger_type_names: true,
            vehicle_destruction_extra_raises: true,
            random_magnitude_overflow: true,
            update_order_removal_skip: true,
            table_root_distance: true,
            table_direction: true,
            one_sided_building_distance: true,
            table_trig_negative_bias: true,
            turn_rate_negative_wrap: true,
            turret_reaim_after_turn: true,
            turret_spin_quantised: true,
            infantry_fixed_turn_rate: true,
            set_facing_keeps_target: true,
            track_body_turn_stops_at_target: true,
            drive_curve_chaining_passive_only: true,
            drive_brake_floor_raises: true,
            drive_subpoint_compounding: true,
            drive_track_end_cap: true,
            drive_leftover_kept_at_rest: true,
            top_position_counts_as_centre: true,
            prone_noncrawler_speedup: true,
            walker_overshoot: true,
            slow_walker_stall: true,
            walker_bridge_destination_deck: true,
            teleport_infantry_negative_level_entry: true,
            teleport_infantry_ground_reservation: true,
            teleport_vehicle_ground_reservation: true,
            teleport_aircraft_ground_reservation: true,
            cow_idle: true,
            infantry_facing_sector_skew: true,
            unchecked_substitute: true,
            hover_needs_nonzero_start: true,
            tread_default_water_only: true,
            repeat_direction_snap: true,
            fire_frame_by_stance: true,
            prone_attack_state_persists: true,
            fraidycat_fear_ignores_fearless: true,
            prone_get_up_repeated_destination: true,
            air_death_fall_generic_follow_up: true,
            first_comment_only: true,
            two_sound_cues: true,
            refused_death_keeps_unit: true,
            occupancy_house_side_positions_only: true,
            occupancy_house_stale: true,
            stationary_friendly_exactly_three: true,
            occupancy_override_hides_later_blockers: true,
            crusher_overlay_index0_clear: true,
            crusher_entry_ground_layer: true,
            cell_layer_split_on_uneven_deck: true,
            walker_targets_allied_wall: true,
            omnicrusher_needs_crusher: true,
            crush_slowdown_counts_self: true,
            crush_slowdown_ground_layer: true,
            crushable_overlay_stays: true,
            crush_slowdown_persists: true,
            crush_slowdown_first_overlay: true,
            auto_crush_computer_only: true,
            passability_wheel_column: true,
            passability_laser_fence_lost: true,
            cliff_back_case_sets: true,
            restricted_deck_overlay_indices: true,
            unset_role_range: true,
            role_one_past_end: true,
            ramp_shape_zero: true,
            negative_level_floor_raised: true,
            path_search_10000_failure: true,
            path_no_reopen: true,
            region_join_height_asymmetry: true,
            flatten_threat_start_cell: true,
            flatten_straight_run_v: true,
            detour_mirrored_reference: true,
            detour_after_tunnel_position: true,
            path_congestion_toggle: true,
            corridor_split_probe: true,
            corridor_retry_no_fallback: true,
            corridor_free_part_unusable_cells: true,
            corridor_free_part_ignores_height: true,
            substitution_centre_ground_level: true,
            substitution_distance_flat_destination: true,
            path_refresh_keeps_old_step: true,
            nearby_centre_twice: true,
            opposite_facing_reversal: true,
            coarse_limit_records_cost: true,
            train_exclusion_search_only: true,
            path_through_stalled_blockers: true,
            unreachable_move_dropped: true,
            deadlock_no_yield: true,
            threat_rebuild_counts_friends: true,
            negative_threat_field: true,
            aircraft_threat_stays_put: true,
            pending_order_limit: true,
            detonate_in_planter_name: true,
            cheer_memory_spans_session: true,
            spent_route_blocks_join: true,
            route_holder_count_drift: true,
            hidden_point_refuses_click: true,
            click_area_skips_kind_test: true,
            delete_selects_after_release: true,
            stacked_point_takes_earliest: true,
            press_copy_ignores_route_state: true,
            group_spread_no_detour: true,
            rally_point_shared: true,
            harvest_cell_shared: true,
            hijack_truce_harvester_check: true,
            permadisguise_attack_cursor: true,
            empty_box_release_click: true,
            group_center_keeps_extra_selection: true,
            group_kept_on_owner_change: true,
            damaging_entry_cursor_retained: true,
            // spec: S-0214 claim 6 (e) and Edge cases (default on = original)
            moment_reason_attack_cursor: true,
            blocked_arrow_events: true,
            queued_move_sleeps: true,
            override_suspends_queued: true,
            refused_override_retargets: true,
            named_deploy_area_fire: true,
            narrow_map_partial_pass: true,
            stale_fill_tiles: true,
            observer_position_handoff: true,
            cleanup_skips_next_house: true,
            drive_giveup_ignores_contact: true,
            drive_duplicate_scatter_request: true,
            drive_nearest_from_top_corner: true,
            deck_side_step_ground_level: true,
            gate_reverse_snaps: true,
            turret_snap_bound_wraps: true,
            upgrade_attack_weapon_gate: true,
            stale_suspended_revert: true,
            disarm_bomb_needs_weapon: true,
            berserk_carrier_tote: true,
            berserk_action_cursor: true,
            building_cell_attack_no_weapon: true,
            near_full_health_rounds_up: true,
            upgrade_fields_carry_over: true,
            follow_index_by_place: true,
            start_waypoint_gap: true,
            random_start_asymmetric_range: true,
            double_start_resolution: true,
            early_phase_stale_country_data: true,
            skirmish_drops_country_owned_objects: true,
            skirmish_drops_map_crates: true,
            starting_infantry_share_centre: true,
            gate_tower_fixed_cells: true,
            building_ammo_fast_rearm: true,
            shared_wave_slot: true,
            undemolishable_heal_is_damage: true,
            yellow_state_at_half: true,
            airborne_mutation_hit_raised: true,
            readiness_from_defended_damage: true,
            delay_kill_whole_cell_spread: true,
            delay_kill_credited_again: true,
            expected_damage_firer_armour: true,
            readiness_heal_over_max: true,
            area_damage_per_foundation_cell: true,
            spread_ring_eleven: true,
            shrapnel_first_listed_only: true,
            shrapnel_hits_terrain: true,
            shrapnel_building_per_cell: true,
            voxel_debris_deck_impact_layer: true,
            voxel_debris_deck_over_water: true,
            voxel_debris_trailer_height: true,
            voxel_debris_expiry_needs_animation: true,
            voxel_debris_deck_rebound_slope: true,
            voxel_debris_cliff_step_faults: true,
            anim_fixed_upper_vertical_speed: true,
            bouncing_anim_expiry_needs_animation: true,
            bouncing_anim_deck_impact_layer: true,
            anim_resource_spread_neighbours_only: true,
            bouncing_anim_deep_landing: true,
            bouncing_anim_water_impact_hits: true,
            bouncing_anim_rebound_faults: true,
            bouncing_anim_impact_hits_walk: true,
            anim_resource_no_growth_entry: true,
            ground_detonation_misses_aircraft: true,
            flying_search_sectors: true,
            flying_candidates_unfiltered: true,
            curtain_nullifies_small_detonation: true,
            wall_hit_minus_one_is_absolute: true,
            uphill_arc_45: true,
            building_pitch_term: true,
            missile_turn_wrap: true,
            aim_origin_without_target: true,
            shot_distance_wrap: true,
            zero_speed_launch: true,
            redirected_shot_aim: true,
            switched_off_hover_lead: true,
            owned_shell_ground_pass: true,
            zero_distance_arc: true,
            stored_target_point: true,
            origin_as_missing_target_point: true,
            vertical_speed_uncapped: true,
            warp_deck_departure_airborne: true,
            garrison_range_drops_air_bonus: true,
            range_target_on_deck: true,
            elevation_deck_cell_level: true,
            passenger_weapon_needs_secondary: true,
            gattling_air_test_slot_one: true,
            secondary_wins_against_air: true,
            elite_occupant_skips_occupy_weapon: true,
            search_one_object_per_cell: true,
            hunt_distance_in_leptons: true,
            zero_score_rejected: true,
            mind_controlled_passengers_cell_choice: true,
            hunt_ignores_distributed_fire: true,
            defeated_enemy_house_kept: true,
            hunt_ground_layer_only: true,
            wall_cell_over_same_ring_object: true,
            voxel_turret_exact_aim: true,
            experience_cost_victim_house: true,
            occupant_experience_untrained: true,
            occupant_experience_next_in_turn: true,
            zero_kill_value_caps: true,
            crash_death_weapon_twice: true,
            building_smudge_discarded_draws: true,
            death_weapon_unplaced_projectile: true,
            rubble_overlay_by_index: true,
            barrel_probe_top_left_only: true,
            smudge_footprints_beyond_map_edge: true,
            destruction_smudge_zero_anchor: true,
            building_fire_point_shift_fixed: true,
            negative_self_heal_counts: true,
            fraidycat_ammo_panic_ignores_fearless: true,
            last_bale_lost: true,
            spread_offer_by_emptied_type: true,
            spread_attempts_count_failures: true,
            spread_mark_kept_on_last_neighbour: true,
            spread_step_lost_on_terrain_object: true,
            spread_stage_from_index: true,
            full_cell_no_regrowth: true,
            resource_timers_carry_over: true,
            thinning_hits_waypoint_only: true,
            refinery_distance_wraps: true,
            harvester_failed_step_off_origin: true,
            zero_load_period_stalls: true,
            unload_without_contact_stalls: true,
            slave_kick_delay_wraps: true,
            slave_relocation_failed_search_origin: true,
            slave_negative_capacity_stalls: true,
            multiple_factory_counts_all: true,
            build_step_period_cap: true,
            resume_restarts_step: true,
            refund_at_current_price: true,
            forced_start_player_hold: true,
            defence_line_partial_factory_check: true,
            unstartable_queue_head_stalls: true,
            foundation_exit_lists: true,
            blackout_one_short: true,
            shared_control_centre_count: true,
            wall_exception_skips_ground: true,
            placement_shroud_at_cursor: true,
            upgrade_level_without_room: true,
            upgrade_prerequisite_last_building: true,
            placed_buildings_not_ever_built: true,
            prerequisite_names_dropped: true,
            ownership_gain_counts_as_kill: true,
            tab_flash_initial_extra_period: true,
            sidebar_icon_surface_clip: true,
            secret_lab_pick_by_position: true,
            unit_changes_no_recheck: true,
            sale_start_no_recheck: true,
            stale_offer_orderable: true,
            queue_limit_counts_item_twice: true,
            dock_bound_aircraft_ignores_limit: true,
            infantry_limit_exception_exact: true,
            country_index_wraps: true,
            unscored_vehicle_stays_counted: true,
            nominal_survivor_count: true,
            wall_sale_no_refund: true,
            packup_release_by_guard_spot: true,
            placed_buildup_twice: true,
            single_stage_buildup_never_completes: true,
            gap_radius_wrap: true,
            sight_bonus_wrap: true,
            building_sight_top_left: true,
            sight_centre_off_map: true,
            height_block_cell: true,
            gap_end_keeps_cover: true,
            fire_reveal_expires: true,
            recover_skips_allied_units: true,
            recover_keeps_pending: true,
            uncounted_sight: true,
            offmap_cells_shared: true,
            spy_satellite_first_building_decides: true,
            defeat_reveal_keeps_pending: true,
            shroud_point_cell_wrap: true,
            shroud_negative_odd_pair: true,
            radar_first_building_decides: true,
            overlay_list_position_rules: true,
            radar_pixel_cell_shift: true,
            release_order_once: true,
            done_aircraft_click_cancels: true,
            random_country_fixed_range: true,
            superweapon_setting_tests_first_only: true,
            crate_count_humans_only: true,
            short_game_without_bases: true,
            short_game_first_three_base_units: true,
            vehicle_buildings_keep_nobody_alive: true,
            defeat_during_end_delay: true,
            defeat_cancellation_skip: true,
            deploy_refuses_own_walls: true,
            failed_packup_keeps_vehicle: true,
            packup_sound_on_sale: true,
            factory_apron_short_list: true,
            factory_fixed_hover_exit: true,
            factory_unload_waits_ready_mark: true,
            sidebar_tie_newest_first: true,
            sidebar_both_flags_rank: true,
            initial_ammo_above_capacity: true,
            building_removal_second_round: true,
            approach_alternative_walk: true,
            approach_alternative_zero_height: true,
            approach_alternative_outside_area: true,
            negative_elevation_cap: true,
            negative_range_fields: true,
            default_sound_priority_ignored: true,
            missing_sample_silences_entry: true,
            sound_order_startup_draw: true,
            full_volume_16383: true,
            attack_decay_without_body: true,
            shroud_sound_corner_cell: true,
            deploy_command_silent: true,
            announcement_volume_ignored: true,
            zero_capacity_harvester_cycles: true,
            non_unloading_refinery_accepts: true,
            launch_spacing_from_spawner_type: true,
            raised_missile_rail_time_from_set_b: true,
            rail_missile_vanishes_with_spawner: true,
            spawner_pending_target_survives_idle_clear: true,
            rocket_aim_at_heading_point: true,
            destroyed_rocket_still_detonates: true,
            rocket_stalls_at_map_edge: true,
            parasite_exit_search_host_mobility: true,
            parasite_exit_tests_host_cell: true,
            parasite_host_loss_bridge_test: true,
            parasite_carrier_loss_strands_unit: true,
            parasite_suppressed_latch_strands_unit: true,
            parasite_host_loss_failed_choice_origin: true,
            parasite_host_loss_suppresses_naval: true,
            cull_experience_path: true,
            aircraft_endless_refire: true,
            aircraft_half_reload_with_target: true,
            aircraft_smoke_scenario_draw: true,
            pad_limit_counts_list_only: true,
            pad_distance_overflow: true,
            landing_own_cell_no_result: true,
            aircraft_unit_step_stall: true,
            aircraft_zero_step_attack_stall: true,
            crash_drops_planted_bomb: true,
            aircraft_corner_sector_untracked: true,
            aircraft_released_above_pad: true,
            unarmed_pad_bound_hover: true,
            aircraft_edge_hold: true,
            transport_load_truncated: true,
            unload_facing_gap_no_wrap: true,
            unload_second_round_skips_rear: true,
            unload_failed_exit_gunner_handover: true,
            air_unload_reselects_clear_destination: true,
            hover_brake_undershoot: true,
            hover_elevated_heading_point: true,
            hover_stalled_start_reset: true,
            hover_idle_arrival_reservation: true,
            hover_flag_clear_height_only: true,
            hover_landing_request_stands: true,
            hover_deployed_keeps_moving: true,
            bridge_cursor_unrelated_link: true,
            bridge_event_band_short: true,
            bridge_family_by_block: true,
            bridge_span_cap: true,
            collapsed_pier_span_search: true,
            low_bridge_wrong_axis_recheck: true,
            low_strip_regress: true,
            pier_loss_notice_skips_chipped: true,
            pier_repair_partial_height: true,
            repair_consumes_engineer_always: true,
            repair_walk_unbounded: true,
            cloak_flag_ground_mobile_only: true,
            cloak_zero_speed_stall: true,
            heal_reveals_cloaked: true,
            uncloak_from_cloaking_full_duration: true,
            building_sensor_ring_owner_only: true,
            cloak_field_overlap_loss: true,
            cloak_field_building_interrupt: true,
            cloak_generator_zero_radius_shutdown: true,
            mobile_sensor_removal_centre: true,
            flying_sensor_disc_stationary: true,
            mirage_adjacent_check_cadence: true,
            sensor_array_removal_radius: true,
            sensor_building_activation_once: true,
            sensor_count_negative: true,
            paradrop_fallback_point: true,
            paratrooper_any_animation_slow_fall: true,
            cash_cycle_one_short: true,
            cash_cycle_lapse: true,
            depot_remote_repair: true,
            depot_loss_leaves_sleep: true,
            depot_loss_strands_hover: true,
            docked_hover_water_loss: true,
            engineer_lost_on_uncapturable: true,
            engineer_repairs_during_sale: true,
            garrison_exit_skips_corner: true,
            garrison_negative_level_exit: true,
            garrison_no_exit_removes: true,
            infiltration_single_effect: true,
            spy_theft_unbacked_gain: true,
            paradrop_west_edge_start_row: true,
            second_stage_order_unchecked: true,
            superweapon_kind_as_list_position: true,
            warp_hold_persists: true,
            // spec: S-0513 claim 39a and Edge cases (default on = original)
            warp_departure_keeps_beam: true,
            // spec: S-0513 claim 39a and Edge cases (default on = original)
            phase_in_beam_spares_infantry: true,
            deploy_to_fire_repacks: true,
            assaulter_reversed_kill_credit: true,
            erasure_double_experience: true,
            erasure_release_switches_on: true,
            erasure_distance_leader_only: true,
            erasure_refused_hit_side_effects: true,
            jumpjet_bridge_early_cruise_height: true,
            jumpjet_lookahead_bridge: true,
            jumpjet_no_cell_self_destruct: true,
            jumpjet_step_truncation: true,
            jumpjet_turn_slowdown_one_sided: true,
            garrison_capacity_equality: true,
            deploy_sound_repeats: true,
            deploy_stall_zero_delay: true,
            anim_fast_rate_freezes: true,
            anim_random_rate_collapse: true,
            rocking_limit_escape: true,
            ungranted_super_reads_clock: true,
            deploy_transition_discards_orders: true,
            prism_count_survives_dropped_shot: true,
            prism_modifier_reread_scale: true,
            prism_support_ignores_operation: true,
            prism_zero_delay_double_count: true,
            stale_chargers_kept: true,
            vehicle_charge_no_break_off: true,
            charge_needs_type_flag: true,
            crate_bonus_reaches_limbo: true,
            crate_harvester_list_prefix: true,
            crate_phantom_place: true,
            crate_unit_fallback_unit_argument: true,
            radiation_add_overdraw: true,
            radiation_residue: true,
            airstrike_empty_strike_stuck: true,
            airstrike_recharge_same_target_only: true,
            mind_control_owner_change_keeps_victims: true,
            dying_victim_holds_slot: true,
            deploy_makes_control_permanent: true,
            gattling_zero_spin_down_empties: true,
            berserk_zero_duration: true,
            psychic_intent_ignores_visibility: true,
            wind_southeast_short: true,
            fire_particle_no_house: true,
            smoke_ignores_particle_cap: true,
            spark_double_gravity: true,
            smoke_deck_extra_update: true,
            flame_sweep_facing: true,
            lift_drop_erases_cell_tail: true,
            lift_impact_removes_survivor: true,
            lift_impact_double_credit: true,
            lift_deck_impact_reads_ground: true,
            lift_standin_stays_seized: true,
            lift_standin_unmarked_cell: true,
            grinder_no_arrival_recheck: true,
            absorber_contact_no_capacity_recheck: true,
            absorber_release_bridge_ends_search: true,
            absorber_release_second_round_skips_rear: true,
            absorber_stale_contact_kept: true,
            absorber_removal_keeps_stale_power: true,
            enter_nearest_survives_absorption: true,
            bunker_clear_chosen_only: true,
            bunker_clear_far_point: true,
            airborne_deploy_order_dropped: true,
            drain_relink_strands_previous: true,
            drain_position_check_vehicles_only: true,
            superweapon_block_one_layer: true,
            superweapon_list_cut_by_removal: true,
            mutation_test_ground_record: true,
            refused_mutation_frees_position: true,
            infantry_animation_occupancy: true,
            disc_laser_no_house: true,
            disc_laser_unlimited_range: true,
            railgun_collinear_hits: true,
            flying_listed_double_hit: true,
            jumpjet_falling_touchdown_tracked: true,
            anim_damage_starts_at_one: true,
            anim_loop_count_wrap: true,
            anim_reverse_raw_loop_end: true,
            anim_pingpong_lower_turn: true,
            anim_loop_end_from_sprite: true,
            anim_followup_start_doubled: true,
            anim_followup_keeps_step: true,
            dominator_damage_storm_house: true,
            dominator_marks_own_objects: true,
            dominator_ground_layer_only: true,
            dominator_sweep_from_nearest: true,
            dominator_takes_dying: true,
            zero_rate_freezes_ambient: true,
            dominator_rate_thousandths: true,
            superweapon_upgrade_second_removal_waits: true,
            superweapon_grant_suspends_by_house_power: true,
            superweapon_restart_ignores_suspension: true,
            superweapon_order_tests_clock: true,
            set_charge_keeps_ready: true,
            drain_orders_bypass_normal_checks: true,
            predependent_kind_as_list_position: true,
            missile_without_launcher_consumes_charge: true,
            superweapon_order_first_type_of_action: true,
            permanent_grant_freezes_suspension: true,
            one_time_grant_keeps_stale_suspension: true,
            nuke_silo_first_type_only: true,
            silo_target_of_current_owner: true,
            curtain_organic_vehicle_defended: true,
            protection_restart_replaces: true,
            warp_kill_before_protection: true,
            warp_redirect_as_tracked: true,
            warp_naval_road_cell: true,
            spy_plane_count_from_paradrop_list: true,
            restoration_failed_return_removes_unit: true,
            // spec: S-0524 Edge cases (default: off, the original)
            repeated_lightning_strike_suppressed: false,
            // spec: S-0524 Edge cases (default: off, the original)
            trigger_crate_respects_setting: false,
            // spec: S-0524 Edge cases (default: off, the original)
            trigger_crate_expiry_while_disabled: false,
        }
    }
}

impl Quirks {
    /// Every switch by name, in field order.
    pub fn named(&self) -> Vec<(&'static str, bool)> {
        vec![
            ("chop_rounding", self.chop_rounding),
            ("speed_normalised_delays", self.speed_normalised_delays),
            (
                "building_wrap_normalisation",
                self.building_wrap_normalisation,
            ),
            (
                "chrono_flash_keeps_order_block",
                self.chrono_flash_keeps_order_block,
            ),
            ("ranged_draw_half_span", self.ranged_draw_half_span),
            ("persistent_event_latch", self.persistent_event_latch),
            ("event_latch_aliasing", self.event_latch_aliasing),
            (
                "all_carriers_without_carrier",
                self.all_carriers_without_carrier,
            ),
            ("entered_by_country_only", self.entered_by_country_only),
            (
                "none_owner_is_first_country",
                self.none_owner_is_first_country,
            ),
            ("exact_trigger_type_names", self.exact_trigger_type_names),
            (
                "vehicle_destruction_extra_raises",
                self.vehicle_destruction_extra_raises,
            ),
            ("random_magnitude_overflow", self.random_magnitude_overflow),
            ("update_order_removal_skip", self.update_order_removal_skip),
            ("table_root_distance", self.table_root_distance),
            ("table_direction", self.table_direction),
            (
                "one_sided_building_distance",
                self.one_sided_building_distance,
            ),
            ("table_trig_negative_bias", self.table_trig_negative_bias),
            ("turn_rate_negative_wrap", self.turn_rate_negative_wrap),
            ("turret_reaim_after_turn", self.turret_reaim_after_turn),
            ("turret_spin_quantised", self.turret_spin_quantised),
            ("infantry_fixed_turn_rate", self.infantry_fixed_turn_rate),
            ("set_facing_keeps_target", self.set_facing_keeps_target),
            (
                "track_body_turn_stops_at_target",
                self.track_body_turn_stops_at_target,
            ),
            (
                "drive_curve_chaining_passive_only",
                self.drive_curve_chaining_passive_only,
            ),
            ("drive_brake_floor_raises", self.drive_brake_floor_raises),
            (
                "drive_subpoint_compounding",
                self.drive_subpoint_compounding,
            ),
            ("drive_track_end_cap", self.drive_track_end_cap),
            (
                "drive_leftover_kept_at_rest",
                self.drive_leftover_kept_at_rest,
            ),
            (
                "top_position_counts_as_centre",
                self.top_position_counts_as_centre,
            ),
            ("prone_noncrawler_speedup", self.prone_noncrawler_speedup),
            ("walker_overshoot", self.walker_overshoot),
            ("slow_walker_stall", self.slow_walker_stall),
            (
                "walker_bridge_destination_deck",
                self.walker_bridge_destination_deck,
            ),
            (
                "teleport_infantry_negative_level_entry",
                self.teleport_infantry_negative_level_entry,
            ),
            (
                "teleport_infantry_ground_reservation",
                self.teleport_infantry_ground_reservation,
            ),
            (
                "teleport_vehicle_ground_reservation",
                self.teleport_vehicle_ground_reservation,
            ),
            (
                "teleport_aircraft_ground_reservation",
                self.teleport_aircraft_ground_reservation,
            ),
            ("cow_idle", self.cow_idle),
            (
                "infantry_facing_sector_skew",
                self.infantry_facing_sector_skew,
            ),
            ("unchecked_substitute", self.unchecked_substitute),
            ("hover_needs_nonzero_start", self.hover_needs_nonzero_start),
            ("tread_default_water_only", self.tread_default_water_only),
            ("repeat_direction_snap", self.repeat_direction_snap),
            ("fire_frame_by_stance", self.fire_frame_by_stance),
            (
                "prone_attack_state_persists",
                self.prone_attack_state_persists,
            ),
            (
                "fraidycat_fear_ignores_fearless",
                self.fraidycat_fear_ignores_fearless,
            ),
            (
                "prone_get_up_repeated_destination",
                self.prone_get_up_repeated_destination,
            ),
            (
                "air_death_fall_generic_follow_up",
                self.air_death_fall_generic_follow_up,
            ),
            ("first_comment_only", self.first_comment_only),
            ("two_sound_cues", self.two_sound_cues),
            ("refused_death_keeps_unit", self.refused_death_keeps_unit),
            (
                "occupancy_house_side_positions_only",
                self.occupancy_house_side_positions_only,
            ),
            ("occupancy_house_stale", self.occupancy_house_stale),
            (
                "stationary_friendly_exactly_three",
                self.stationary_friendly_exactly_three,
            ),
            (
                "occupancy_override_hides_later_blockers",
                self.occupancy_override_hides_later_blockers,
            ),
            (
                "crusher_overlay_index0_clear",
                self.crusher_overlay_index0_clear,
            ),
            (
                "crusher_entry_ground_layer",
                self.crusher_entry_ground_layer,
            ),
            (
                "cell_layer_split_on_uneven_deck",
                self.cell_layer_split_on_uneven_deck,
            ),
            (
                "walker_targets_allied_wall",
                self.walker_targets_allied_wall,
            ),
            ("omnicrusher_needs_crusher", self.omnicrusher_needs_crusher),
            (
                "crush_slowdown_counts_self",
                self.crush_slowdown_counts_self,
            ),
            (
                "crush_slowdown_ground_layer",
                self.crush_slowdown_ground_layer,
            ),
            ("crushable_overlay_stays", self.crushable_overlay_stays),
            ("crush_slowdown_persists", self.crush_slowdown_persists),
            (
                "crush_slowdown_first_overlay",
                self.crush_slowdown_first_overlay,
            ),
            ("auto_crush_computer_only", self.auto_crush_computer_only),
            ("passability_wheel_column", self.passability_wheel_column),
            (
                "passability_laser_fence_lost",
                self.passability_laser_fence_lost,
            ),
            ("cliff_back_case_sets", self.cliff_back_case_sets),
            (
                "restricted_deck_overlay_indices",
                self.restricted_deck_overlay_indices,
            ),
            ("unset_role_range", self.unset_role_range),
            ("role_one_past_end", self.role_one_past_end),
            ("ramp_shape_zero", self.ramp_shape_zero),
            (
                "negative_level_floor_raised",
                self.negative_level_floor_raised,
            ),
            ("path_search_10000_failure", self.path_search_10000_failure),
            ("path_no_reopen", self.path_no_reopen),
            (
                "region_join_height_asymmetry",
                self.region_join_height_asymmetry,
            ),
            ("flatten_threat_start_cell", self.flatten_threat_start_cell),
            ("flatten_straight_run_v", self.flatten_straight_run_v),
            ("detour_mirrored_reference", self.detour_mirrored_reference),
            (
                "detour_after_tunnel_position",
                self.detour_after_tunnel_position,
            ),
            ("path_congestion_toggle", self.path_congestion_toggle),
            ("corridor_split_probe", self.corridor_split_probe),
            (
                "corridor_retry_no_fallback",
                self.corridor_retry_no_fallback,
            ),
            (
                "corridor_free_part_unusable_cells",
                self.corridor_free_part_unusable_cells,
            ),
            (
                "corridor_free_part_ignores_height",
                self.corridor_free_part_ignores_height,
            ),
            (
                "substitution_centre_ground_level",
                self.substitution_centre_ground_level,
            ),
            (
                "substitution_distance_flat_destination",
                self.substitution_distance_flat_destination,
            ),
            (
                "path_refresh_keeps_old_step",
                self.path_refresh_keeps_old_step,
            ),
            ("nearby_centre_twice", self.nearby_centre_twice),
            ("opposite_facing_reversal", self.opposite_facing_reversal),
            ("coarse_limit_records_cost", self.coarse_limit_records_cost),
            (
                "train_exclusion_search_only",
                self.train_exclusion_search_only,
            ),
            (
                "path_through_stalled_blockers",
                self.path_through_stalled_blockers,
            ),
            ("unreachable_move_dropped", self.unreachable_move_dropped),
            ("deadlock_no_yield", self.deadlock_no_yield),
            (
                "threat_rebuild_counts_friends",
                self.threat_rebuild_counts_friends,
            ),
            ("negative_threat_field", self.negative_threat_field),
            ("aircraft_threat_stays_put", self.aircraft_threat_stays_put),
            ("pending_order_limit", self.pending_order_limit),
            ("detonate_in_planter_name", self.detonate_in_planter_name),
            (
                "cheer_memory_spans_session",
                self.cheer_memory_spans_session,
            ),
            ("spent_route_blocks_join", self.spent_route_blocks_join),
            ("route_holder_count_drift", self.route_holder_count_drift),
            (
                "hidden_point_refuses_click",
                self.hidden_point_refuses_click,
            ),
            (
                "click_area_skips_kind_test",
                self.click_area_skips_kind_test,
            ),
            (
                "delete_selects_after_release",
                self.delete_selects_after_release,
            ),
            (
                "stacked_point_takes_earliest",
                self.stacked_point_takes_earliest,
            ),
            (
                "press_copy_ignores_route_state",
                self.press_copy_ignores_route_state,
            ),
            ("group_spread_no_detour", self.group_spread_no_detour),
            ("rally_point_shared", self.rally_point_shared),
            ("harvest_cell_shared", self.harvest_cell_shared),
            (
                "hijack_truce_harvester_check",
                self.hijack_truce_harvester_check,
            ),
            (
                "permadisguise_attack_cursor",
                self.permadisguise_attack_cursor,
            ),
            ("empty_box_release_click", self.empty_box_release_click),
            (
                "group_center_keeps_extra_selection",
                self.group_center_keeps_extra_selection,
            ),
            (
                "group_kept_on_owner_change",
                self.group_kept_on_owner_change,
            ),
            (
                "damaging_entry_cursor_retained",
                self.damaging_entry_cursor_retained,
            ),
            (
                "moment_reason_attack_cursor",
                self.moment_reason_attack_cursor,
            ),
            ("blocked_arrow_events", self.blocked_arrow_events),
            ("queued_move_sleeps", self.queued_move_sleeps),
            ("override_suspends_queued", self.override_suspends_queued),
            (
                "refused_override_retargets",
                self.refused_override_retargets,
            ),
            ("named_deploy_area_fire", self.named_deploy_area_fire),
            ("narrow_map_partial_pass", self.narrow_map_partial_pass),
            ("stale_fill_tiles", self.stale_fill_tiles),
            ("observer_position_handoff", self.observer_position_handoff),
            ("cleanup_skips_next_house", self.cleanup_skips_next_house),
            (
                "drive_giveup_ignores_contact",
                self.drive_giveup_ignores_contact,
            ),
            (
                "drive_duplicate_scatter_request",
                self.drive_duplicate_scatter_request,
            ),
            (
                "drive_nearest_from_top_corner",
                self.drive_nearest_from_top_corner,
            ),
            (
                "deck_side_step_ground_level",
                self.deck_side_step_ground_level,
            ),
            ("gate_reverse_snaps", self.gate_reverse_snaps),
            ("turret_snap_bound_wraps", self.turret_snap_bound_wraps),
            (
                "upgrade_attack_weapon_gate",
                self.upgrade_attack_weapon_gate,
            ),
            ("stale_suspended_revert", self.stale_suspended_revert),
            ("disarm_bomb_needs_weapon", self.disarm_bomb_needs_weapon),
            ("berserk_carrier_tote", self.berserk_carrier_tote),
            ("berserk_action_cursor", self.berserk_action_cursor),
            (
                "building_cell_attack_no_weapon",
                self.building_cell_attack_no_weapon,
            ),
            (
                "near_full_health_rounds_up",
                self.near_full_health_rounds_up,
            ),
            ("upgrade_fields_carry_over", self.upgrade_fields_carry_over),
            ("follow_index_by_place", self.follow_index_by_place),
            ("start_waypoint_gap", self.start_waypoint_gap),
            (
                "random_start_asymmetric_range",
                self.random_start_asymmetric_range,
            ),
            ("double_start_resolution", self.double_start_resolution),
            (
                "early_phase_stale_country_data",
                self.early_phase_stale_country_data,
            ),
            (
                "skirmish_drops_country_owned_objects",
                self.skirmish_drops_country_owned_objects,
            ),
            ("skirmish_drops_map_crates", self.skirmish_drops_map_crates),
            (
                "starting_infantry_share_centre",
                self.starting_infantry_share_centre,
            ),
            ("gate_tower_fixed_cells", self.gate_tower_fixed_cells),
            ("building_ammo_fast_rearm", self.building_ammo_fast_rearm),
            ("shared_wave_slot", self.shared_wave_slot),
            (
                "undemolishable_heal_is_damage",
                self.undemolishable_heal_is_damage,
            ),
            ("yellow_state_at_half", self.yellow_state_at_half),
            (
                "airborne_mutation_hit_raised",
                self.airborne_mutation_hit_raised,
            ),
            (
                "readiness_from_defended_damage",
                self.readiness_from_defended_damage,
            ),
            (
                "delay_kill_whole_cell_spread",
                self.delay_kill_whole_cell_spread,
            ),
            ("delay_kill_credited_again", self.delay_kill_credited_again),
            (
                "expected_damage_firer_armour",
                self.expected_damage_firer_armour,
            ),
            ("readiness_heal_over_max", self.readiness_heal_over_max),
            (
                "area_damage_per_foundation_cell",
                self.area_damage_per_foundation_cell,
            ),
            ("spread_ring_eleven", self.spread_ring_eleven),
            (
                "shrapnel_first_listed_only",
                self.shrapnel_first_listed_only,
            ),
            ("shrapnel_hits_terrain", self.shrapnel_hits_terrain),
            (
                "shrapnel_building_per_cell",
                self.shrapnel_building_per_cell,
            ),
            (
                "voxel_debris_deck_impact_layer",
                self.voxel_debris_deck_impact_layer,
            ),
            (
                "voxel_debris_deck_over_water",
                self.voxel_debris_deck_over_water,
            ),
            (
                "voxel_debris_trailer_height",
                self.voxel_debris_trailer_height,
            ),
            (
                "voxel_debris_expiry_needs_animation",
                self.voxel_debris_expiry_needs_animation,
            ),
            (
                "voxel_debris_deck_rebound_slope",
                self.voxel_debris_deck_rebound_slope,
            ),
            (
                "voxel_debris_cliff_step_faults",
                self.voxel_debris_cliff_step_faults,
            ),
            (
                "anim_fixed_upper_vertical_speed",
                self.anim_fixed_upper_vertical_speed,
            ),
            (
                "bouncing_anim_expiry_needs_animation",
                self.bouncing_anim_expiry_needs_animation,
            ),
            (
                "bouncing_anim_deck_impact_layer",
                self.bouncing_anim_deck_impact_layer,
            ),
            (
                "anim_resource_spread_neighbours_only",
                self.anim_resource_spread_neighbours_only,
            ),
            (
                "bouncing_anim_deep_landing",
                self.bouncing_anim_deep_landing,
            ),
            (
                "bouncing_anim_water_impact_hits",
                self.bouncing_anim_water_impact_hits,
            ),
            (
                "bouncing_anim_rebound_faults",
                self.bouncing_anim_rebound_faults,
            ),
            (
                "bouncing_anim_impact_hits_walk",
                self.bouncing_anim_impact_hits_walk,
            ),
            (
                "anim_resource_no_growth_entry",
                self.anim_resource_no_growth_entry,
            ),
            (
                "ground_detonation_misses_aircraft",
                self.ground_detonation_misses_aircraft,
            ),
            ("flying_search_sectors", self.flying_search_sectors),
            (
                "flying_candidates_unfiltered",
                self.flying_candidates_unfiltered,
            ),
            (
                "curtain_nullifies_small_detonation",
                self.curtain_nullifies_small_detonation,
            ),
            (
                "wall_hit_minus_one_is_absolute",
                self.wall_hit_minus_one_is_absolute,
            ),
            ("uphill_arc_45", self.uphill_arc_45),
            ("building_pitch_term", self.building_pitch_term),
            ("missile_turn_wrap", self.missile_turn_wrap),
            ("aim_origin_without_target", self.aim_origin_without_target),
            ("shot_distance_wrap", self.shot_distance_wrap),
            ("zero_speed_launch", self.zero_speed_launch),
            ("redirected_shot_aim", self.redirected_shot_aim),
            ("switched_off_hover_lead", self.switched_off_hover_lead),
            ("owned_shell_ground_pass", self.owned_shell_ground_pass),
            ("zero_distance_arc", self.zero_distance_arc),
            ("stored_target_point", self.stored_target_point),
            (
                "origin_as_missing_target_point",
                self.origin_as_missing_target_point,
            ),
            ("vertical_speed_uncapped", self.vertical_speed_uncapped),
            (
                "warp_deck_departure_airborne",
                self.warp_deck_departure_airborne,
            ),
            (
                "garrison_range_drops_air_bonus",
                self.garrison_range_drops_air_bonus,
            ),
            ("range_target_on_deck", self.range_target_on_deck),
            ("elevation_deck_cell_level", self.elevation_deck_cell_level),
            (
                "passenger_weapon_needs_secondary",
                self.passenger_weapon_needs_secondary,
            ),
            (
                "gattling_air_test_slot_one",
                self.gattling_air_test_slot_one,
            ),
            (
                "secondary_wins_against_air",
                self.secondary_wins_against_air,
            ),
            (
                "elite_occupant_skips_occupy_weapon",
                self.elite_occupant_skips_occupy_weapon,
            ),
            (
                "search_one_object_per_cell",
                self.search_one_object_per_cell,
            ),
            ("hunt_distance_in_leptons", self.hunt_distance_in_leptons),
            ("zero_score_rejected", self.zero_score_rejected),
            (
                "mind_controlled_passengers_cell_choice",
                self.mind_controlled_passengers_cell_choice,
            ),
            (
                "hunt_ignores_distributed_fire",
                self.hunt_ignores_distributed_fire,
            ),
            ("defeated_enemy_house_kept", self.defeated_enemy_house_kept),
            ("hunt_ground_layer_only", self.hunt_ground_layer_only),
            (
                "wall_cell_over_same_ring_object",
                self.wall_cell_over_same_ring_object,
            ),
            ("voxel_turret_exact_aim", self.voxel_turret_exact_aim),
            (
                "experience_cost_victim_house",
                self.experience_cost_victim_house,
            ),
            (
                "occupant_experience_untrained",
                self.occupant_experience_untrained,
            ),
            (
                "occupant_experience_next_in_turn",
                self.occupant_experience_next_in_turn,
            ),
            ("zero_kill_value_caps", self.zero_kill_value_caps),
            ("crash_death_weapon_twice", self.crash_death_weapon_twice),
            (
                "building_smudge_discarded_draws",
                self.building_smudge_discarded_draws,
            ),
            (
                "death_weapon_unplaced_projectile",
                self.death_weapon_unplaced_projectile,
            ),
            ("rubble_overlay_by_index", self.rubble_overlay_by_index),
            (
                "barrel_probe_top_left_only",
                self.barrel_probe_top_left_only,
            ),
            (
                "smudge_footprints_beyond_map_edge",
                self.smudge_footprints_beyond_map_edge,
            ),
            (
                "destruction_smudge_zero_anchor",
                self.destruction_smudge_zero_anchor,
            ),
            (
                "building_fire_point_shift_fixed",
                self.building_fire_point_shift_fixed,
            ),
            ("negative_self_heal_counts", self.negative_self_heal_counts),
            (
                "fraidycat_ammo_panic_ignores_fearless",
                self.fraidycat_ammo_panic_ignores_fearless,
            ),
            ("last_bale_lost", self.last_bale_lost),
            (
                "spread_offer_by_emptied_type",
                self.spread_offer_by_emptied_type,
            ),
            (
                "spread_attempts_count_failures",
                self.spread_attempts_count_failures,
            ),
            (
                "spread_mark_kept_on_last_neighbour",
                self.spread_mark_kept_on_last_neighbour,
            ),
            (
                "spread_step_lost_on_terrain_object",
                self.spread_step_lost_on_terrain_object,
            ),
            ("spread_stage_from_index", self.spread_stage_from_index),
            ("full_cell_no_regrowth", self.full_cell_no_regrowth),
            (
                "resource_timers_carry_over",
                self.resource_timers_carry_over,
            ),
            (
                "thinning_hits_waypoint_only",
                self.thinning_hits_waypoint_only,
            ),
            ("refinery_distance_wraps", self.refinery_distance_wraps),
            (
                "harvester_failed_step_off_origin",
                self.harvester_failed_step_off_origin,
            ),
            ("zero_load_period_stalls", self.zero_load_period_stalls),
            (
                "unload_without_contact_stalls",
                self.unload_without_contact_stalls,
            ),
            ("slave_kick_delay_wraps", self.slave_kick_delay_wraps),
            (
                "slave_relocation_failed_search_origin",
                self.slave_relocation_failed_search_origin,
            ),
            (
                "slave_negative_capacity_stalls",
                self.slave_negative_capacity_stalls,
            ),
            (
                "multiple_factory_counts_all",
                self.multiple_factory_counts_all,
            ),
            ("build_step_period_cap", self.build_step_period_cap),
            ("resume_restarts_step", self.resume_restarts_step),
            ("refund_at_current_price", self.refund_at_current_price),
            ("forced_start_player_hold", self.forced_start_player_hold),
            (
                "defence_line_partial_factory_check",
                self.defence_line_partial_factory_check,
            ),
            (
                "unstartable_queue_head_stalls",
                self.unstartable_queue_head_stalls,
            ),
            ("foundation_exit_lists", self.foundation_exit_lists),
            ("blackout_one_short", self.blackout_one_short),
            (
                "shared_control_centre_count",
                self.shared_control_centre_count,
            ),
            (
                "wall_exception_skips_ground",
                self.wall_exception_skips_ground,
            ),
            (
                "placement_shroud_at_cursor",
                self.placement_shroud_at_cursor,
            ),
            (
                "upgrade_level_without_room",
                self.upgrade_level_without_room,
            ),
            (
                "upgrade_prerequisite_last_building",
                self.upgrade_prerequisite_last_building,
            ),
            (
                "placed_buildings_not_ever_built",
                self.placed_buildings_not_ever_built,
            ),
            (
                "prerequisite_names_dropped",
                self.prerequisite_names_dropped,
            ),
            (
                "ownership_gain_counts_as_kill",
                self.ownership_gain_counts_as_kill,
            ),
            (
                "tab_flash_initial_extra_period",
                self.tab_flash_initial_extra_period,
            ),
            ("sidebar_icon_surface_clip", self.sidebar_icon_surface_clip),
            (
                "secret_lab_pick_by_position",
                self.secret_lab_pick_by_position,
            ),
            ("unit_changes_no_recheck", self.unit_changes_no_recheck),
            ("sale_start_no_recheck", self.sale_start_no_recheck),
            ("stale_offer_orderable", self.stale_offer_orderable),
            (
                "queue_limit_counts_item_twice",
                self.queue_limit_counts_item_twice,
            ),
            (
                "dock_bound_aircraft_ignores_limit",
                self.dock_bound_aircraft_ignores_limit,
            ),
            (
                "infantry_limit_exception_exact",
                self.infantry_limit_exception_exact,
            ),
            ("country_index_wraps", self.country_index_wraps),
            (
                "unscored_vehicle_stays_counted",
                self.unscored_vehicle_stays_counted,
            ),
            ("nominal_survivor_count", self.nominal_survivor_count),
            ("wall_sale_no_refund", self.wall_sale_no_refund),
            (
                "packup_release_by_guard_spot",
                self.packup_release_by_guard_spot,
            ),
            ("placed_buildup_twice", self.placed_buildup_twice),
            (
                "single_stage_buildup_never_completes",
                self.single_stage_buildup_never_completes,
            ),
            ("gap_radius_wrap", self.gap_radius_wrap),
            ("sight_bonus_wrap", self.sight_bonus_wrap),
            ("building_sight_top_left", self.building_sight_top_left),
            ("sight_centre_off_map", self.sight_centre_off_map),
            ("height_block_cell", self.height_block_cell),
            ("gap_end_keeps_cover", self.gap_end_keeps_cover),
            ("fire_reveal_expires", self.fire_reveal_expires),
            (
                "recover_skips_allied_units",
                self.recover_skips_allied_units,
            ),
            ("recover_keeps_pending", self.recover_keeps_pending),
            ("uncounted_sight", self.uncounted_sight),
            ("offmap_cells_shared", self.offmap_cells_shared),
            (
                "spy_satellite_first_building_decides",
                self.spy_satellite_first_building_decides,
            ),
            (
                "defeat_reveal_keeps_pending",
                self.defeat_reveal_keeps_pending,
            ),
            ("shroud_point_cell_wrap", self.shroud_point_cell_wrap),
            ("shroud_negative_odd_pair", self.shroud_negative_odd_pair),
            (
                "radar_first_building_decides",
                self.radar_first_building_decides,
            ),
            (
                "overlay_list_position_rules",
                self.overlay_list_position_rules,
            ),
            ("radar_pixel_cell_shift", self.radar_pixel_cell_shift),
            ("release_order_once", self.release_order_once),
            (
                "done_aircraft_click_cancels",
                self.done_aircraft_click_cancels,
            ),
            (
                "random_country_fixed_range",
                self.random_country_fixed_range,
            ),
            (
                "superweapon_setting_tests_first_only",
                self.superweapon_setting_tests_first_only,
            ),
            ("crate_count_humans_only", self.crate_count_humans_only),
            ("short_game_without_bases", self.short_game_without_bases),
            (
                "short_game_first_three_base_units",
                self.short_game_first_three_base_units,
            ),
            (
                "vehicle_buildings_keep_nobody_alive",
                self.vehicle_buildings_keep_nobody_alive,
            ),
            ("defeat_during_end_delay", self.defeat_during_end_delay),
            ("defeat_cancellation_skip", self.defeat_cancellation_skip),
            ("deploy_refuses_own_walls", self.deploy_refuses_own_walls),
            (
                "failed_packup_keeps_vehicle",
                self.failed_packup_keeps_vehicle,
            ),
            ("packup_sound_on_sale", self.packup_sound_on_sale),
            ("factory_apron_short_list", self.factory_apron_short_list),
            ("factory_fixed_hover_exit", self.factory_fixed_hover_exit),
            (
                "factory_unload_waits_ready_mark",
                self.factory_unload_waits_ready_mark,
            ),
            ("sidebar_tie_newest_first", self.sidebar_tie_newest_first),
            ("sidebar_both_flags_rank", self.sidebar_both_flags_rank),
            (
                "initial_ammo_above_capacity",
                self.initial_ammo_above_capacity,
            ),
            (
                "building_removal_second_round",
                self.building_removal_second_round,
            ),
            ("approach_alternative_walk", self.approach_alternative_walk),
            (
                "approach_alternative_zero_height",
                self.approach_alternative_zero_height,
            ),
            (
                "approach_alternative_outside_area",
                self.approach_alternative_outside_area,
            ),
            ("negative_elevation_cap", self.negative_elevation_cap),
            ("negative_range_fields", self.negative_range_fields),
            (
                "default_sound_priority_ignored",
                self.default_sound_priority_ignored,
            ),
            (
                "missing_sample_silences_entry",
                self.missing_sample_silences_entry,
            ),
            ("sound_order_startup_draw", self.sound_order_startup_draw),
            ("full_volume_16383", self.full_volume_16383),
            ("attack_decay_without_body", self.attack_decay_without_body),
            ("shroud_sound_corner_cell", self.shroud_sound_corner_cell),
            ("deploy_command_silent", self.deploy_command_silent),
            (
                "announcement_volume_ignored",
                self.announcement_volume_ignored,
            ),
            (
                "zero_capacity_harvester_cycles",
                self.zero_capacity_harvester_cycles,
            ),
            (
                "non_unloading_refinery_accepts",
                self.non_unloading_refinery_accepts,
            ),
            (
                "launch_spacing_from_spawner_type",
                self.launch_spacing_from_spawner_type,
            ),
            (
                "raised_missile_rail_time_from_set_b",
                self.raised_missile_rail_time_from_set_b,
            ),
            (
                "rail_missile_vanishes_with_spawner",
                self.rail_missile_vanishes_with_spawner,
            ),
            (
                "spawner_pending_target_survives_idle_clear",
                self.spawner_pending_target_survives_idle_clear,
            ),
            (
                "rocket_aim_at_heading_point",
                self.rocket_aim_at_heading_point,
            ),
            (
                "destroyed_rocket_still_detonates",
                self.destroyed_rocket_still_detonates,
            ),
            ("rocket_stalls_at_map_edge", self.rocket_stalls_at_map_edge),
            (
                "parasite_exit_search_host_mobility",
                self.parasite_exit_search_host_mobility,
            ),
            (
                "parasite_exit_tests_host_cell",
                self.parasite_exit_tests_host_cell,
            ),
            (
                "parasite_host_loss_bridge_test",
                self.parasite_host_loss_bridge_test,
            ),
            (
                "parasite_carrier_loss_strands_unit",
                self.parasite_carrier_loss_strands_unit,
            ),
            (
                "parasite_suppressed_latch_strands_unit",
                self.parasite_suppressed_latch_strands_unit,
            ),
            (
                "parasite_host_loss_failed_choice_origin",
                self.parasite_host_loss_failed_choice_origin,
            ),
            (
                "parasite_host_loss_suppresses_naval",
                self.parasite_host_loss_suppresses_naval,
            ),
            ("cull_experience_path", self.cull_experience_path),
            ("aircraft_endless_refire", self.aircraft_endless_refire),
            (
                "aircraft_half_reload_with_target",
                self.aircraft_half_reload_with_target,
            ),
            (
                "aircraft_smoke_scenario_draw",
                self.aircraft_smoke_scenario_draw,
            ),
            (
                "pad_limit_counts_list_only",
                self.pad_limit_counts_list_only,
            ),
            ("pad_distance_overflow", self.pad_distance_overflow),
            (
                "landing_own_cell_no_result",
                self.landing_own_cell_no_result,
            ),
            ("aircraft_unit_step_stall", self.aircraft_unit_step_stall),
            (
                "aircraft_zero_step_attack_stall",
                self.aircraft_zero_step_attack_stall,
            ),
            ("crash_drops_planted_bomb", self.crash_drops_planted_bomb),
            (
                "aircraft_corner_sector_untracked",
                self.aircraft_corner_sector_untracked,
            ),
            (
                "aircraft_released_above_pad",
                self.aircraft_released_above_pad,
            ),
            ("unarmed_pad_bound_hover", self.unarmed_pad_bound_hover),
            ("aircraft_edge_hold", self.aircraft_edge_hold),
            ("transport_load_truncated", self.transport_load_truncated),
            ("unload_facing_gap_no_wrap", self.unload_facing_gap_no_wrap),
            (
                "unload_second_round_skips_rear",
                self.unload_second_round_skips_rear,
            ),
            (
                "unload_failed_exit_gunner_handover",
                self.unload_failed_exit_gunner_handover,
            ),
            (
                "air_unload_reselects_clear_destination",
                self.air_unload_reselects_clear_destination,
            ),
            ("hover_brake_undershoot", self.hover_brake_undershoot),
            (
                "hover_elevated_heading_point",
                self.hover_elevated_heading_point,
            ),
            ("hover_stalled_start_reset", self.hover_stalled_start_reset),
            (
                "hover_idle_arrival_reservation",
                self.hover_idle_arrival_reservation,
            ),
            (
                "hover_flag_clear_height_only",
                self.hover_flag_clear_height_only,
            ),
            (
                "hover_landing_request_stands",
                self.hover_landing_request_stands,
            ),
            (
                "hover_deployed_keeps_moving",
                self.hover_deployed_keeps_moving,
            ),
            (
                "bridge_cursor_unrelated_link",
                self.bridge_cursor_unrelated_link,
            ),
            ("bridge_event_band_short", self.bridge_event_band_short),
            ("bridge_family_by_block", self.bridge_family_by_block),
            ("bridge_span_cap", self.bridge_span_cap),
            (
                "collapsed_pier_span_search",
                self.collapsed_pier_span_search,
            ),
            (
                "low_bridge_wrong_axis_recheck",
                self.low_bridge_wrong_axis_recheck,
            ),
            ("low_strip_regress", self.low_strip_regress),
            (
                "pier_loss_notice_skips_chipped",
                self.pier_loss_notice_skips_chipped,
            ),
            (
                "pier_repair_partial_height",
                self.pier_repair_partial_height,
            ),
            (
                "repair_consumes_engineer_always",
                self.repair_consumes_engineer_always,
            ),
            ("repair_walk_unbounded", self.repair_walk_unbounded),
            (
                "cloak_flag_ground_mobile_only",
                self.cloak_flag_ground_mobile_only,
            ),
            ("cloak_zero_speed_stall", self.cloak_zero_speed_stall),
            ("heal_reveals_cloaked", self.heal_reveals_cloaked),
            (
                "uncloak_from_cloaking_full_duration",
                self.uncloak_from_cloaking_full_duration,
            ),
            (
                "building_sensor_ring_owner_only",
                self.building_sensor_ring_owner_only,
            ),
            ("cloak_field_overlap_loss", self.cloak_field_overlap_loss),
            (
                "cloak_field_building_interrupt",
                self.cloak_field_building_interrupt,
            ),
            (
                "cloak_generator_zero_radius_shutdown",
                self.cloak_generator_zero_radius_shutdown,
            ),
            (
                "mobile_sensor_removal_centre",
                self.mobile_sensor_removal_centre,
            ),
            (
                "flying_sensor_disc_stationary",
                self.flying_sensor_disc_stationary,
            ),
            (
                "mirage_adjacent_check_cadence",
                self.mirage_adjacent_check_cadence,
            ),
            (
                "sensor_array_removal_radius",
                self.sensor_array_removal_radius,
            ),
            (
                "sensor_building_activation_once",
                self.sensor_building_activation_once,
            ),
            ("sensor_count_negative", self.sensor_count_negative),
            ("paradrop_fallback_point", self.paradrop_fallback_point),
            (
                "paratrooper_any_animation_slow_fall",
                self.paratrooper_any_animation_slow_fall,
            ),
            ("cash_cycle_one_short", self.cash_cycle_one_short),
            ("cash_cycle_lapse", self.cash_cycle_lapse),
            ("depot_remote_repair", self.depot_remote_repair),
            ("depot_loss_leaves_sleep", self.depot_loss_leaves_sleep),
            ("depot_loss_strands_hover", self.depot_loss_strands_hover),
            ("docked_hover_water_loss", self.docked_hover_water_loss),
            (
                "engineer_lost_on_uncapturable",
                self.engineer_lost_on_uncapturable,
            ),
            (
                "engineer_repairs_during_sale",
                self.engineer_repairs_during_sale,
            ),
            (
                "garrison_exit_skips_corner",
                self.garrison_exit_skips_corner,
            ),
            (
                "garrison_negative_level_exit",
                self.garrison_negative_level_exit,
            ),
            ("garrison_no_exit_removes", self.garrison_no_exit_removes),
            (
                "infiltration_single_effect",
                self.infiltration_single_effect,
            ),
            ("spy_theft_unbacked_gain", self.spy_theft_unbacked_gain),
            (
                "paradrop_west_edge_start_row",
                self.paradrop_west_edge_start_row,
            ),
            (
                "second_stage_order_unchecked",
                self.second_stage_order_unchecked,
            ),
            (
                "superweapon_kind_as_list_position",
                self.superweapon_kind_as_list_position,
            ),
            ("warp_hold_persists", self.warp_hold_persists),
            ("warp_departure_keeps_beam", self.warp_departure_keeps_beam),
            (
                "phase_in_beam_spares_infantry",
                self.phase_in_beam_spares_infantry,
            ),
            ("deploy_to_fire_repacks", self.deploy_to_fire_repacks),
            (
                "assaulter_reversed_kill_credit",
                self.assaulter_reversed_kill_credit,
            ),
            ("erasure_double_experience", self.erasure_double_experience),
            (
                "erasure_release_switches_on",
                self.erasure_release_switches_on,
            ),
            (
                "erasure_distance_leader_only",
                self.erasure_distance_leader_only,
            ),
            (
                "erasure_refused_hit_side_effects",
                self.erasure_refused_hit_side_effects,
            ),
            (
                "jumpjet_bridge_early_cruise_height",
                self.jumpjet_bridge_early_cruise_height,
            ),
            ("jumpjet_lookahead_bridge", self.jumpjet_lookahead_bridge),
            (
                "jumpjet_no_cell_self_destruct",
                self.jumpjet_no_cell_self_destruct,
            ),
            ("jumpjet_step_truncation", self.jumpjet_step_truncation),
            (
                "jumpjet_turn_slowdown_one_sided",
                self.jumpjet_turn_slowdown_one_sided,
            ),
            (
                "garrison_capacity_equality",
                self.garrison_capacity_equality,
            ),
            ("deploy_sound_repeats", self.deploy_sound_repeats),
            ("deploy_stall_zero_delay", self.deploy_stall_zero_delay),
            ("anim_fast_rate_freezes", self.anim_fast_rate_freezes),
            ("anim_random_rate_collapse", self.anim_random_rate_collapse),
            ("rocking_limit_escape", self.rocking_limit_escape),
            (
                "ungranted_super_reads_clock",
                self.ungranted_super_reads_clock,
            ),
            (
                "deploy_transition_discards_orders",
                self.deploy_transition_discards_orders,
            ),
            (
                "prism_count_survives_dropped_shot",
                self.prism_count_survives_dropped_shot,
            ),
            (
                "prism_modifier_reread_scale",
                self.prism_modifier_reread_scale,
            ),
            (
                "prism_support_ignores_operation",
                self.prism_support_ignores_operation,
            ),
            (
                "prism_zero_delay_double_count",
                self.prism_zero_delay_double_count,
            ),
            ("stale_chargers_kept", self.stale_chargers_kept),
            (
                "vehicle_charge_no_break_off",
                self.vehicle_charge_no_break_off,
            ),
            ("charge_needs_type_flag", self.charge_needs_type_flag),
            ("crate_bonus_reaches_limbo", self.crate_bonus_reaches_limbo),
            (
                "crate_harvester_list_prefix",
                self.crate_harvester_list_prefix,
            ),
            ("crate_phantom_place", self.crate_phantom_place),
            (
                "crate_unit_fallback_unit_argument",
                self.crate_unit_fallback_unit_argument,
            ),
            ("radiation_add_overdraw", self.radiation_add_overdraw),
            ("radiation_residue", self.radiation_residue),
            (
                "airstrike_empty_strike_stuck",
                self.airstrike_empty_strike_stuck,
            ),
            (
                "airstrike_recharge_same_target_only",
                self.airstrike_recharge_same_target_only,
            ),
            (
                "mind_control_owner_change_keeps_victims",
                self.mind_control_owner_change_keeps_victims,
            ),
            ("dying_victim_holds_slot", self.dying_victim_holds_slot),
            (
                "deploy_makes_control_permanent",
                self.deploy_makes_control_permanent,
            ),
            (
                "gattling_zero_spin_down_empties",
                self.gattling_zero_spin_down_empties,
            ),
            ("berserk_zero_duration", self.berserk_zero_duration),
            (
                "psychic_intent_ignores_visibility",
                self.psychic_intent_ignores_visibility,
            ),
            ("wind_southeast_short", self.wind_southeast_short),
            ("fire_particle_no_house", self.fire_particle_no_house),
            (
                "smoke_ignores_particle_cap",
                self.smoke_ignores_particle_cap,
            ),
            ("spark_double_gravity", self.spark_double_gravity),
            ("smoke_deck_extra_update", self.smoke_deck_extra_update),
            ("flame_sweep_facing", self.flame_sweep_facing),
            (
                "lift_drop_erases_cell_tail",
                self.lift_drop_erases_cell_tail,
            ),
            (
                "lift_impact_removes_survivor",
                self.lift_impact_removes_survivor,
            ),
            ("lift_impact_double_credit", self.lift_impact_double_credit),
            (
                "lift_deck_impact_reads_ground",
                self.lift_deck_impact_reads_ground,
            ),
            ("lift_standin_stays_seized", self.lift_standin_stays_seized),
            (
                "lift_standin_unmarked_cell",
                self.lift_standin_unmarked_cell,
            ),
            (
                "grinder_no_arrival_recheck",
                self.grinder_no_arrival_recheck,
            ),
            (
                "absorber_contact_no_capacity_recheck",
                self.absorber_contact_no_capacity_recheck,
            ),
            (
                "absorber_release_bridge_ends_search",
                self.absorber_release_bridge_ends_search,
            ),
            (
                "absorber_release_second_round_skips_rear",
                self.absorber_release_second_round_skips_rear,
            ),
            (
                "absorber_stale_contact_kept",
                self.absorber_stale_contact_kept,
            ),
            (
                "absorber_removal_keeps_stale_power",
                self.absorber_removal_keeps_stale_power,
            ),
            (
                "enter_nearest_survives_absorption",
                self.enter_nearest_survives_absorption,
            ),
            ("bunker_clear_chosen_only", self.bunker_clear_chosen_only),
            ("bunker_clear_far_point", self.bunker_clear_far_point),
            (
                "airborne_deploy_order_dropped",
                self.airborne_deploy_order_dropped,
            ),
            (
                "drain_relink_strands_previous",
                self.drain_relink_strands_previous,
            ),
            (
                "drain_position_check_vehicles_only",
                self.drain_position_check_vehicles_only,
            ),
            (
                "superweapon_block_one_layer",
                self.superweapon_block_one_layer,
            ),
            (
                "superweapon_list_cut_by_removal",
                self.superweapon_list_cut_by_removal,
            ),
            (
                "mutation_test_ground_record",
                self.mutation_test_ground_record,
            ),
            (
                "refused_mutation_frees_position",
                self.refused_mutation_frees_position,
            ),
            (
                "infantry_animation_occupancy",
                self.infantry_animation_occupancy,
            ),
            ("disc_laser_no_house", self.disc_laser_no_house),
            (
                "disc_laser_unlimited_range",
                self.disc_laser_unlimited_range,
            ),
            ("railgun_collinear_hits", self.railgun_collinear_hits),
            ("flying_listed_double_hit", self.flying_listed_double_hit),
            (
                "jumpjet_falling_touchdown_tracked",
                self.jumpjet_falling_touchdown_tracked,
            ),
            ("anim_damage_starts_at_one", self.anim_damage_starts_at_one),
            ("anim_loop_count_wrap", self.anim_loop_count_wrap),
            ("anim_reverse_raw_loop_end", self.anim_reverse_raw_loop_end),
            ("anim_pingpong_lower_turn", self.anim_pingpong_lower_turn),
            ("anim_loop_end_from_sprite", self.anim_loop_end_from_sprite),
            (
                "anim_followup_start_doubled",
                self.anim_followup_start_doubled,
            ),
            ("anim_followup_keeps_step", self.anim_followup_keeps_step),
            (
                "dominator_damage_storm_house",
                self.dominator_damage_storm_house,
            ),
            (
                "dominator_marks_own_objects",
                self.dominator_marks_own_objects,
            ),
            (
                "dominator_ground_layer_only",
                self.dominator_ground_layer_only,
            ),
            (
                "dominator_sweep_from_nearest",
                self.dominator_sweep_from_nearest,
            ),
            ("dominator_takes_dying", self.dominator_takes_dying),
            ("zero_rate_freezes_ambient", self.zero_rate_freezes_ambient),
            (
                "dominator_rate_thousandths",
                self.dominator_rate_thousandths,
            ),
            (
                "superweapon_upgrade_second_removal_waits",
                self.superweapon_upgrade_second_removal_waits,
            ),
            (
                "superweapon_grant_suspends_by_house_power",
                self.superweapon_grant_suspends_by_house_power,
            ),
            (
                "superweapon_restart_ignores_suspension",
                self.superweapon_restart_ignores_suspension,
            ),
            (
                "superweapon_order_tests_clock",
                self.superweapon_order_tests_clock,
            ),
            ("set_charge_keeps_ready", self.set_charge_keeps_ready),
            (
                "drain_orders_bypass_normal_checks",
                self.drain_orders_bypass_normal_checks,
            ),
            (
                "predependent_kind_as_list_position",
                self.predependent_kind_as_list_position,
            ),
            (
                "missile_without_launcher_consumes_charge",
                self.missile_without_launcher_consumes_charge,
            ),
            (
                "superweapon_order_first_type_of_action",
                self.superweapon_order_first_type_of_action,
            ),
            (
                "permanent_grant_freezes_suspension",
                self.permanent_grant_freezes_suspension,
            ),
            (
                "one_time_grant_keeps_stale_suspension",
                self.one_time_grant_keeps_stale_suspension,
            ),
            ("nuke_silo_first_type_only", self.nuke_silo_first_type_only),
            (
                "silo_target_of_current_owner",
                self.silo_target_of_current_owner,
            ),
            (
                "curtain_organic_vehicle_defended",
                self.curtain_organic_vehicle_defended,
            ),
            (
                "protection_restart_replaces",
                self.protection_restart_replaces,
            ),
            (
                "warp_kill_before_protection",
                self.warp_kill_before_protection,
            ),
            ("warp_redirect_as_tracked", self.warp_redirect_as_tracked),
            ("warp_naval_road_cell", self.warp_naval_road_cell),
            (
                "spy_plane_count_from_paradrop_list",
                self.spy_plane_count_from_paradrop_list,
            ),
            (
                "restoration_failed_return_removes_unit",
                self.restoration_failed_return_removes_unit,
            ),
            (
                "repeated_lightning_strike_suppressed",
                self.repeated_lightning_strike_suppressed,
            ),
            (
                "trigger_crate_respects_setting",
                self.trigger_crate_respects_setting,
            ),
            (
                "trigger_crate_expiry_while_disabled",
                self.trigger_crate_expiry_while_disabled,
            ),
        ]
    }

    /// The rounding of floating-point results (S-0205 claim 5).
    pub fn rounding(&self) -> glasnost_num::Rounding {
        if self.chop_rounding {
            glasnost_num::Rounding::TowardZero
        } else {
            glasnost_num::Rounding::Nearest
        }
    }
}

/// Quirk switches named in the specs that are owned by the importer or the
/// renderer rather than the simulation, with their owner.
// spec: S-0215 claim 21, S-0104, S-0107 claim 22, S-0009 Edge cases, S-0006 Edge cases, S-0010 Edge cases, S-0308 claim 1, S-0309 Edge cases, S-0003 claim 13, S-0202 claim 17 (the owning specs of these switches)
pub const EXTERNAL_QUIRKS: &[(&str, &str)] = &[
    (
        "override_resets_keys",
        "importer (convert::layers::LayerQuirks), S-0006 claim 5",
    ),
    (
        "persist_global_overrides",
        "importer (convert::layers::LayerQuirks), S-0006 claim 10",
    ),
    (
        "colour_redefinition_ignored",
        "importer (convert::layers::LayerQuirks), S-0103 claim 5 and S-0006 claim 2",
    ),
    (
        "persist_tiberium_overrides",
        "importer (convert::layers::LayerQuirks), S-0006 claim 10",
    ),
    (
        "mission_rules_before_reset",
        "importer (convert::layers::LayerQuirks), S-0006 claim 14",
    ),
    (
        "mode_number_first_wins",
        "importer (convert::layers::LayerQuirks), S-0006 claim 13",
    ),
    (
        "bad_language_rules_skips_mode",
        "importer (convert::layers::LayerQuirks), S-0006 claim 12",
    ),
    (
        "barrel_recoil_layer_reset",
        "importer (convert::layers::LayerQuirks), S-0105 claim 27",
    ),
    (
        "reread_truncates_light_values",
        "importer (convert::layers::LayerQuirks), S-0104 claim 12",
    ),
    (
        "anti_air_rate_layer_reset",
        "importer (convert::layers::LayerQuirks), S-0215 claim 21",
    ),
    (
        "prism_modifier_reread_scale",
        "importer (convert::layers::LayerQuirks), S-0518 claim 16",
    ),
    (
        "red_normalisation_divisor",
        "renderer (render::light), S-0104",
    ),
    (
        "layer_change_on_events_only",
        "renderer (mapview::quirks), S-0102 claim 4; S-0501 claim 48 (a) names it for an aircraft released above its pad (S-0501 r52)",
    ),
    (
        "enable_unfreezes_unpowered",
        "renderer (mapview::quirks), S-0108 claims 13 and 26; S-0405 claim 10 names it for a switched-on building's slots (S-0405 r36)",
    ),
    (
        "shroud_on_flat_grid",
        "renderer (render::shroud::Quirks), S-0110 claim 8; S-0410 claim 6 names it for the radar's shroud and gap tests (S-0410 r15, not yet followed by the radar)",
    ),
    ("sequence_value_length_limit", "importer, S-0107 claim 22"),
    ("case_sensitive_direction", "importer, S-0107 claim 22"),
    ("projectile_image_afresh", "importer, S-0007 claim 26"),
    (
        "absent_rect_is_zero",
        "importer (convert::map::MapQuirks), S-0009 claim 30",
    ),
    (
        "cell_index_alias",
        "importer (convert::map::MapQuirks), S-0009 claims 7 and 31",
    ),
    (
        "header_pairs_as_start_marks",
        "importer (convert::map::MapQuirks), S-0009 claims 25 and 29",
    ),
    (
        "lobby_counts_zero_waypoint",
        "importer (convert::map::MapQuirks), S-0009 claim 23",
    ),
    (
        "no_overlay_below_format_2",
        "importer (convert::map::MapQuirks), S-0009 claim 11",
    ),
    (
        "object_empty_fields_collapse",
        "importer (convert::map::MapQuirks), S-0009 claim 34",
    ),
    (
        "preview_markers_from_header",
        "importer (convert::map::MapQuirks), S-0009 claim 25",
    ),
    (
        "clear_weapons_any_turret_count",
        "importer (convert::sim::SimQuirks), S-0308 claim 1",
    ),
    (
        "coefficient_zero_replaced",
        "importer (convert::sim::SimQuirks), S-0309 Edge cases",
    ),
    (
        "theater_five_forced_unset_roles",
        "importer (convert::theater::TheaterQuirks), S-0010 claim 4",
    ),
    (
        "map_pack_line_limit",
        "importer (convert::map::MapQuirks), S-0012 claim 1",
    ),
    (
        "map_base64_groups",
        "importer (convert::map::MapQuirks), S-0012 claim 2",
    ),
    (
        "partial_terrain_values",
        "importer (convert::map::MapQuirks), S-0012 claim 9 and S-0009 claim 7",
    ),
    (
        "tree_strength_fixed_at_first_section",
        "importer (convert::sim::SimQuirks), S-0102 claim 23 and S-0006 claim 5",
    ),
    (
        "percent_list_integer_prefix",
        "importer (ini::Quirks), S-0003 claim 13",
    ),
    (
        "list_items_untrimmed",
        "importer (ini::Quirks), S-0003 claim 12",
    ),
    (
        "colour_list_item_triples",
        "importer (convert::sim::SimQuirks), S-0202 claim 17 and Edge cases",
    ),
    (
        "voxel_light_first_section_mode",
        "renderer (render::voxel::VoxelQuirks), S-0105 claim 18; S-0317 claim 24 names it for debris pieces (S-0317 r57)",
    ),
    (
        "voxel_mode0_misread",
        "renderer (render::voxel::VoxelQuirks), S-0105 claim 35; S-0317 claim 24 names it for debris pieces (S-0317 r57)",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `quirks.name` in the simulation specs (S-0201 to S-0215), in
    /// S-0006, S-0009, S-0010 and in S-0107 is a field here or listed as
    /// owned elsewhere.
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn every_spec_quirk_has_a_switch() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../spec");
        let mut files = Vec::new();
        for dir in [
            "simulation",
            "movement",
            "infantry",
            "ui",
            "combat",
            "economy",
            "production",
            "structures",
            "visibility",
        ] {
            for e in std::fs::read_dir(format!("{root}/{dir}")).expect("spec dir") {
                let p = e.expect("entry").path();
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_owned();
                if name.starts_with("S-02")
                    || name.starts_with("S-03")
                    || name.starts_with("S-04")
                    || name.starts_with("S-05")
                    || name.starts_with("S-06")
                {
                    files.push(p);
                }
            }
        }
        assert!(files.len() >= 15, "found {} M3 specs", files.len());
        // S-0009's and S-0010's switches: the loader's and the recalculation's are fields here,
        // the importer's are listed below; likewise S-0006's (importer) and S-0107's (simulation,
        // apart from two importer switches).
        for (dir, prefixes) in [
            ("formats", &["S-0009", "S-0010", "S-0012"][..]),
            ("triggers", &["S-0011", "S-0524"][..]),
            ("rules-import", &["S-0006"][..]),
            ("rendering", &["S-0107"][..]),
            // The M7b superweapons built: S-0612 (S-0611 names no switch of its own here).
            ("superweapons", &["S-0512", "S-0513", "S-0612"][..]),
        ] {
            for e in std::fs::read_dir(format!("{root}/{dir}")).expect("spec dir") {
                let p = e.expect("entry").path();
                if p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| prefixes.iter().any(|x| n.starts_with(x)))
                {
                    files.push(p);
                }
            }
        }
        // Switches a spec names only to withdraw them.
        const WITHDRAWN: &[&str] = &[
            "alliance_message_slot_colour", // spec: S-0413 Edge cases (fixed with no switch; the old switch must not survive)
            "variant_origin_mismatch",      // spec: S-0010 claim 13 (corrected with no switch)
            "variant_cell_missing",         // spec: S-0010 claim 24 (corrected with no switch)
        ];
        let q = Quirks::default();
        let have: std::collections::BTreeSet<&str> =
            q.named().into_iter().map(|(n, _)| n).collect();
        let mut missing = Vec::new();
        for f in files {
            let text = std::fs::read_to_string(&f).expect("read spec");
            // A switch a spec only proposes to the project lead is not approved yet.
            let text = text
                .split("## Questions for the project lead")
                .next()
                .unwrap_or("");
            let mut rest = text;
            while let Some(i) = rest.find("quirks.") {
                rest = &rest[i + 7..];
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                    .collect();
                if name.is_empty() {
                    continue;
                }
                if !have.contains(name.as_str())
                    && !EXTERNAL_QUIRKS.iter().any(|(n, _)| *n == name)
                    && !WITHDRAWN.contains(&name.as_str())
                {
                    missing.push(format!("{} in {}", name, f.display()));
                }
            }
        }
        assert!(missing.is_empty(), "quirks without a switch: {missing:#?}");
        // Switches whose spec gives the original as the off position.
        const DEFAULT_OFF: &[&str] = &[
            "repeated_lightning_strike_suppressed", // spec: S-0524 Edge cases (default: off)
            "trigger_crate_respects_setting",       // spec: S-0524 Edge cases (default: off)
            "trigger_crate_expiry_while_disabled",  // spec: S-0524 Edge cases (default: off)
        ];
        assert!(
            q.named().iter().all(|(n, v)| *v != DEFAULT_OFF.contains(n)),
            "every default is the original"
        );
    }
}
