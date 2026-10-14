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
    /// Cells of an unusable category belong to the free part.
    // spec: S-0212 claim 13a
    #[serde(default = "on")]
    pub corridor_free_part_unusable_cells: bool,
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
    /// Hovering an engineer over a building above its capture threshold retains the preceding cursor; off: it uses the capture cursor of table C.
    // spec: S-0214 claim 21 and Edge cases
    #[serde(default = "on")]
    pub damaging_entry_cursor_retained: bool,
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
            corridor_free_part_unusable_cells: true,
            substitution_centre_ground_level: true,
            substitution_distance_flat_destination: true,
            path_refresh_keeps_old_step: true,
            nearby_centre_twice: true,
            opposite_facing_reversal: true,
            coarse_limit_records_cost: true,
            train_exclusion_search_only: true,
            pending_order_limit: true,
            detonate_in_planter_name: true,
            cheer_memory_spans_session: true,
            hijack_truce_harvester_check: true,
            permadisguise_attack_cursor: true,
            empty_box_release_click: true,
            damaging_entry_cursor_retained: true,
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
            zero_distance_arc: true,
            stored_target_point: true,
            origin_as_missing_target_point: true,
            vertical_speed_uncapped: true,
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
            building_fire_point_shift_fixed: true,
            negative_self_heal_counts: true,
            fraidycat_ammo_panic_ignores_fearless: true,
            last_bale_lost: true,
            spread_offer_by_emptied_type: true,
            spread_attempts_count_failures: true,
            spread_mark_kept_on_last_neighbour: true,
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
                "corridor_free_part_unusable_cells",
                self.corridor_free_part_unusable_cells,
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
            ("pending_order_limit", self.pending_order_limit),
            ("detonate_in_planter_name", self.detonate_in_planter_name),
            (
                "cheer_memory_spans_session",
                self.cheer_memory_spans_session,
            ),
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
                "damaging_entry_cursor_retained",
                self.damaging_entry_cursor_retained,
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
            ("zero_distance_arc", self.zero_distance_arc),
            ("stored_target_point", self.stored_target_point),
            (
                "origin_as_missing_target_point",
                self.origin_as_missing_target_point,
            ),
            ("vertical_speed_uncapped", self.vertical_speed_uncapped),
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
// spec: S-0215 claim 21, S-0104, S-0107 claim 22, S-0009 Edge cases, S-0006 Edge cases, S-0010 Edge cases, S-0308 claim 1, S-0309 Edge cases, S-0003 claim 13 (the owning specs of these switches)
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
        "variant_origin_mismatch",
        "importer (convert::theater::TheaterQuirks), S-0010 claim 13",
    ),
    (
        "variant_cell_missing",
        "importer (convert::theater::TheaterQuirks), S-0010 claim 24",
    ),
    (
        "anti_air_rate_layer_reset",
        "importer (convert::layers::LayerQuirks), S-0215 claim 21",
    ),
    (
        "red_normalisation_divisor",
        "renderer (render::light), S-0104",
    ),
    ("sequence_value_length_limit", "importer, S-0107 claim 22"),
    ("case_sensitive_direction", "importer, S-0107 claim 22"),
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
                if name.starts_with("S-02") || name.starts_with("S-03") || name.starts_with("S-04")
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
            ("triggers", &["S-0011"][..]),
            ("rules-import", &["S-0006"][..]),
            ("rendering", &["S-0107"][..]),
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
        assert!(
            q.named().iter().all(|(_, v)| *v),
            "every default is the original"
        );
    }
}
