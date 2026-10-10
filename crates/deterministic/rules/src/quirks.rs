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
    /// A chrono flash restores the order block in force when it started, so a block from game start or an earlier run outlasts it.
    // spec: S-0201 Edge cases
    #[serde(default = "on")]
    pub chrono_flash_keeps_order_block: bool,
    /// A ranged draw whose bounds differ by exactly 2147483647 takes no value and returns the lower bound plus 2147483648, wrapped.
    // spec: S-0202 claim 5
    #[serde(default = "on")]
    pub ranged_draw_half_span: bool,
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
    // spec: S-0010 claim 5; S-0211 claim 13
    #[serde(default = "on")]
    pub unset_role_range: bool,
    /// A role naming the first missing tileset starts at the tile count.
    // spec: S-0010 claim 4
    #[serde(default = "on")]
    pub role_one_past_end: bool,
    /// A ramp-range tile of shape 0 becomes the tile just before the ramp range.
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
    /// A gate door that reverses from closing to opening is at rest open at the gate's next update; off: the reopening takes as long as the closing had run.
    // spec: S-0215 claim 35
    #[serde(default = "on")]
    pub gate_reverse_snaps: bool,
    /// Disarm bomb gives its attack order only to an actor able to fire; off: it needs only that the actor is movable.
    // spec: S-0214 Edge cases
    #[serde(default = "on")]
    pub disarm_bomb_needs_weapon: bool,
    /// A berserk carrier aircraft can still get tote and its move order; off: it gets no order, like every berserk object.
    // spec: S-0214 claim 15
    #[serde(default = "on")]
    pub berserk_carrier_tote: bool,
    /// A building with no weapon in its first slot gets attack over a cell whose generic result is attack (row BC3); off: nothing.
    // spec: S-0214 Edge cases
    #[serde(default = "on")]
    pub building_cell_attack_no_weapon: bool,
}

impl Default for Quirks {
    fn default() -> Self {
        Self {
            chop_rounding: true,
            speed_normalised_delays: true,
            chrono_flash_keeps_order_block: true,
            ranged_draw_half_span: true,
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
            top_position_counts_as_centre: true,
            prone_noncrawler_speedup: true,
            walker_overshoot: true,
            slow_walker_stall: true,
            cow_idle: true,
            infantry_facing_sector_skew: true,
            unchecked_substitute: true,
            hover_needs_nonzero_start: true,
            tread_default_water_only: true,
            repeat_direction_snap: true,
            fire_frame_by_stance: true,
            prone_attack_state_persists: true,
            fraidycat_fear_ignores_fearless: true,
            air_death_fall_generic_follow_up: true,
            first_comment_only: true,
            two_sound_cues: true,
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
            queued_move_sleeps: true,
            override_suspends_queued: true,
            refused_override_retargets: true,
            named_deploy_area_fire: true,
            narrow_map_partial_pass: true,
            stale_fill_tiles: true,
            observer_position_handoff: true,
            cleanup_skips_next_house: true,
            gate_reverse_snaps: true,
            disarm_bomb_needs_weapon: true,
            berserk_carrier_tote: true,
            building_cell_attack_no_weapon: true,
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
                "chrono_flash_keeps_order_block",
                self.chrono_flash_keeps_order_block,
            ),
            ("ranged_draw_half_span", self.ranged_draw_half_span),
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
                "top_position_counts_as_centre",
                self.top_position_counts_as_centre,
            ),
            ("prone_noncrawler_speedup", self.prone_noncrawler_speedup),
            ("walker_overshoot", self.walker_overshoot),
            ("slow_walker_stall", self.slow_walker_stall),
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
                "air_death_fall_generic_follow_up",
                self.air_death_fall_generic_follow_up,
            ),
            ("first_comment_only", self.first_comment_only),
            ("two_sound_cues", self.two_sound_cues),
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
            ("gate_reverse_snaps", self.gate_reverse_snaps),
            ("disarm_bomb_needs_weapon", self.disarm_bomb_needs_weapon),
            ("berserk_carrier_tote", self.berserk_carrier_tote),
            (
                "building_cell_attack_no_weapon",
                self.building_cell_attack_no_weapon,
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
// spec: S-0215 claim 21, S-0104, S-0107 claim 22 (the owning specs of these switches)
pub const EXTERNAL_QUIRKS: &[(&str, &str)] = &[
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
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `quirks.name` in the simulation specs (S-0201 to S-0215) and in
    /// the specs whose sim-side rules this crate's users run is a field here
    /// or listed as owned elsewhere.
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn every_spec_quirk_has_a_switch() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../spec");
        let mut files = Vec::new();
        for dir in ["simulation", "movement", "infantry", "ui"] {
            for e in std::fs::read_dir(format!("{root}/{dir}")).expect("spec dir") {
                let p = e.expect("entry").path();
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_owned();
                if name.starts_with("S-02") {
                    files.push(p);
                }
            }
        }
        assert!(files.len() >= 15, "found {} M3 specs", files.len());
        let q = Quirks::default();
        let have: std::collections::BTreeSet<&str> =
            q.named().into_iter().map(|(n, _)| n).collect();
        let mut missing = Vec::new();
        for f in files {
            let text = std::fs::read_to_string(&f).expect("read spec");
            let mut rest = text.as_str();
            while let Some(i) = rest.find("quirks.") {
                rest = &rest[i + 7..];
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                    .collect();
                if name.is_empty() {
                    continue;
                }
                if !have.contains(name.as_str()) && !EXTERNAL_QUIRKS.iter().any(|(n, _)| *n == name)
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
