# S-0319 revision 9 claim ledger, M8 step A3 intake 15 part 1

S-0319 revision 9 (bouncing animations; answers Q-1762 in part; spec commits f4044da and
907ebf6). It fills A5's "bouncing motion" seam in `glasnost_sim::anims`: a sprite animation whose
type is a bouncing type at its creation now flies, lands and ends there. The six S-0319 switches
(commit 6333832, all default on) are consulted: `quirks.anim_fixed_upper_vertical_speed`,
`quirks.bouncing_anim_expiry_needs_animation`, `quirks.bouncing_anim_deck_impact_layer`,
`quirks.anim_resource_spread_neighbours_only`, `quirks.bouncing_anim_deep_landing` and
`quirks.bouncing_anim_water_impact_hits`. The existing switches are wired, not copied:
`quirks.chop_rounding` (S-0205), `quirks.random_magnitude_overflow` and
`quirks.ranged_draw_half_span` (S-0202), `quirks.table_root_distance` (S-0204). S-0317's own
switches are left alone: each spec's switch covers only its own objects.

The lead's ruling for this step: a zero span takes the remainder 0 with an importer warning
(claim 14), and a trailer separation of 0 or absent acts as 1 with an importer warning
(claim 12). Both match revision 9's text.

**Simulation.** `glasnost_sim::bounce` (new).
- **Type fields.** `BounceType`, built once per animation type in `AnimRules::bounce`
  (`rules.rs`), holds the Import mapping's defaults (M 15.0, Z_min 3.5, Z_max 3.5 or the type's
  MaxZVel under the switch off, e the single 0.8 widened), the spans W_xy and W_z of F1, the
  named types, the warhead resolved after the warhead list, and the spawn overlay's index
  (matched case-insensitively).
- **Creation.** `load::creation_draws` now returns the three raw values of the bouncing-type row
  (S-0202 claim 10) with the stage-delay draw. `anims::spawn` hands them to `bounce::create`,
  which forms the first velocity (F1 for a falling animation, F2 for a meteor) and places the
  animation: a falling one at the creation point with its flight position 10 leptons above, a
  meteor at its start position trunc(c − T·v). The start event uses the placed position.
- **State.** `Anim::flight: Option<Flight>` (single-precision flight position and velocity),
  saved and hashed. Save format 77 → 78, AI game format 39 → 40.
- **Turn.** `anims::turn`: after the owner check, the motion step (`bounce::step`) when the
  animation has a flight and a position. A landing calls `bounce::land`, then the end and the
  removal from the update order, and the turn returns (no stage update, no trailer, no
  follow-up switch). Otherwise the trailer when due, then the stage update of S-0109. Claims 4
  and 7 to 10 read the type the animation has now; a follow-up name with no type gives the
  absent-key defaults.
- **Step.** `bounce::step`: gravity, the move, then claim 6's tests in order (deck, under-deck,
  ground with the 100-lepton snap limit, the 150-lepton building band), the cliff undo, the
  rebound and the elasticity, S-0317 F6's stop measure, a meteor's 1.4 added back, and the
  position in play truncated.
- **Landing.** `bounce::land`: an impact's bounce animation at the position before the move and
  its hits (`pieces::ground_hits`, now shared with `pieces::land_impact`, no behaviour change),
  then claim 8's rows: dry or at deck height the expiry animation, area damage, spawns (F4) and
  new resource cells (F5); on water the wake and the first splash, or a meteor's last splash.
- **Trailer.** `bounce::trailer_due` for every animation whose type names one, bouncing or not,
  on ticks divisible by |s| (0 acting as 1), made by `combat::effects::trailer` at the position
  in play.

**Importer.** `convert::sim::bouncing_fields` reads the 15 keys of the Import mapping (the four
speeds and the elasticity kept as decimal text; DamageRadius, TrailerSeperation, SpawnCount and
TiberiumSpreadRadius as integers; IsTiberium; the six names canonicalised, `none` and `<none>`
naming no type). Coverage rows in `sim_combat.rs`. Three warnings: a zero span (claim 14), a
trailer with no separation or 0 (claim 12), SpawnCount 2147483647 (claim 9). The importer
cannot see the simulation's `quirks.anim_fixed_upper_vertical_speed`, so it checks the vertical
span under both settings and says which one gives 0. That is our choice, not the spec's text.
The splash-list warning now names S-0319 claim 8 as well.

**Requests and interim rules.**
- **Q-1846** (S-0319's open contact and map-edge items). Every ground-layer building counts for
  the building band and no overlay does; the cliff turn reverses each horizontal component along
  which the cell changed; a sloped cell rebounds as a flat one; the stop measure is
  `pieces::stopped`; an animation is placed and turns anywhere, off the map with ground 0,
  level 0, not water, no bridge; the impact hits are the pieces' hits. These reuse S-0317's
  Q-1834 and Q-1836 interims through the shared helpers.
- **Q-1847** (resource cells). A new resource cell enters neither of S-0401's lists and takes no
  growth-list draw. A place past the overlay list (or at or past the no-overlay index) places
  nothing; both draws are taken, and the cell takes no further offer in that landing.

**Seams.**
- Claim 5's start sound: it plays where the animation was created and does not follow the
  flight (presentation only; `Effect::Anim` carries no animation id for the audio crate to
  follow).
- The warhead's light flash at the end (claim 8): drawing only, not drawn, as for S-0317.

**Tests.** `crates/deterministic/sim/tests/claims_s0319.rs` (28 tests, synthetic types on a
synthetic map), `bounce::tests` (2), `convert` `sim::tests::bouncing_animation_fields` and
`bouncing_span_names_the_upper_vertical_switch`.

| claim | status | test names or reason |
|---|---|---|
| 1 (which animations move) | tested | `claim_1_11_who_moves` (a type with neither flag never moves, even after switching to a flagged type), `claim_3_meteor_creation` (both flags: a meteor) |
| 2 (falling creation) | tested | `claim_2_falling_creation` (r₁ vertical, r₂ second horizontal, r₃ first horizontal, three ranged draws, placed at the creation point, flight 10 above), `claim_2_upper_vertical_limit_switch` (`quirks.anim_fixed_upper_vertical_speed` both ways) |
| 3 (meteor creation) | tested | `claim_3_meteor_creation` (q₁ y, q₂ x, v_z = Z_min, the sign swap, T = 70 − \|q₃ rem 20\|, placed at the start). Meteor start off the map: placed (Q-1846) |
| 4 (motion step) | tested | `claim_4_motion_step` (F3, a meteor's 1.4 added back), `claim_4_turns_in_the_tick` (the step in the turns the update order gives) |
| 5 (the stage keeps playing) | partly | `claim_4_motion_step` checks the stage advances in flight; S-0109's bouncing exceptions are the existing S-0109 tests. The start sound does not follow the flight (seam) |
| 6 (what counts as landing) | tested | `claim_6_7_8_ground_impact`, `claim_6_high_bridge_deck`, `claim_6_deep_landing_switch` (both ways), `claim_6_building_cell` (the 150 band), `claim_6_cliff_undone_first_turn`. Which buildings count and the cliff turn: Q-1846 |
| 7 (impact or stopping landing) | tested | `claim_6_7_8_ground_impact`, `claim_7_hit_radius_and_cell`, `claim_7_stopping_landing_at_the_ground`, `claim_7_stop_without_contact`, `claim_7_deck_impact_layer_switch` (both ways), `claim_7_8_water` (`quirks.bouncing_anim_water_impact_hits` both ways). Sloped rebound and stop form: Q-1846 |
| 8 (the end at a landing) | partly | `claim_6_7_8_ground_impact` (expiry, area damage), `claim_8_expiry_needs_animation_switch` (both ways), `claim_7_8_water` (wake, first splash 3 higher), `claim_8_meteor_on_water_and_empty_list` (last splash; empty list none), `claim_6_high_bridge_deck` (at deck height the dry row). Partly: the light flash is not drawn (seam) |
| 9 (spawned animations) | tested | `claim_9_spawns` (F4, both draws first, then each creation), `claim_9_no_spawn_count_no_draws`; importer warning for 2147483647 in `bouncing_animation_fields` |
| 10 (new resource) | tested | `claim_10_resources_neighbours` (radius 1: directions 7, 0, 1; overlay R(0, 3), placement, stage R(0, 2)), `claim_10_resources_switch_and_none` (switch off: the 3 × 3 square; no spawn type no draws), `claim_10_large_radius_neighbours`, `claim_10_place_past_the_overlay_list` (Q-1847). The resource lists: Q-1847 |
| 11 (ends without a landing, follow-ups) | tested | `claim_11_pass_end_without_landing`, `claim_1_11_who_moves` (a follow-up with neither flag keeps the flight); the meteor test reads the current type in `claim_8_meteor_on_water_and_empty_list` |
| 12 (trailer) | tested | `claim_12_trailer` (ticks divisible by s, after the step and before the stage update, non-bouncing too, 0 as 1, none in a landing turn), `bounce::tests::trailer_cadence`; importer warning in `bouncing_animation_fields` |
| 13 (random draws) | tested | The order is checked against a cloned stream in `claim_2_falling_creation`, `claim_3_meteor_creation`, `claim_9_spawns`, `claim_10_resources_neighbours`, `claim_10_place_past_the_overlay_list`. The growth-list draw: Q-1847 |
| 14 (zero spans) | tested | `claim_14_zero_spans` (remainder 0), `bounce::tests::magnitude_remainders` (0 span, −1 with −2147483648); importer warnings in `bouncing_animation_fields` and `bouncing_span_names_the_upper_vertical_switch` |
| F1 | tested | `claim_2_falling_creation`, `claim_2_upper_vertical_limit_switch`, `claim_14_zero_spans` |
| F2 | tested | `claim_3_meteor_creation` |
| F3 | tested | `claim_4_motion_step` |
| F4 | tested | `claim_9_spawns` |
| F5 | tested | the four `claim_10_*` tests |
| Constants | tested | each constant is cited at its use in `bounce.rs` and checked by the claim tests above (gravity, the 10 lift, 3.5, T, the draw bounds, 2.5, 416, 20, 100, 150, 2 levels, 3, R(0, 3), R(0, 2)) |
| State and ordering | tested | `claim_4_turns_in_the_tick`, `claim_12_trailer`, `flight_round_trip` (the state hash survives a save mid-flight) |
| Edge cases (six switches) | tested | each switch both ways: `claim_2_upper_vertical_limit_switch`, `claim_8_expiry_needs_animation_switch`, `claim_7_deck_impact_layer_switch`, `claim_10_resources_switch_and_none`, `claim_6_deep_landing_switch`, `claim_7_8_water` |
| Edge cases (departures) | tested | zero spans (`claim_14_zero_spans`), separation 0 (`claim_12_trailer`), empty splash list (`claim_8_meteor_on_water_and_empty_list`) |
| Import mapping | tested | `bouncing_animation_fields` (all 15 keys, the names canonicalised, `<none>` none); `Bouncer`, `IsMeteor`, `Damage` were already read (S-0109, S-0615); `Wake` and `SplashList` are S-0312's and S-0303's |
| Scenarios BA1 to BA12 | by construction | Oracle scenarios; the claim tests above play each one's synthetic shape. No measurement record yet |
| Changes for other specs: S-0109 Terms (bouncing type) | named, left | A text change to S-0109, for the rest of intake 15. The behaviour (a follow-up with neither flag keeps moving) is built here, claim 11 |
| Changes for other specs: S-0109 (the trailer applies to every animation) | named, left | A text change to S-0109, for the rest of intake 15. The behaviour is built here, claim 12 |
| Changes for other specs: S-0312 claim 4 (sprite debris move and end by S-0319) | built | `combat::death::debris` makes sprite debris (the debris animations, or the metallic-debris list) through `effects::anim` at the raised position, so they take the creation of claims 2 and 3 and now fly and end (lane below) |
| Changes for other specs: S-0317 claim 16 (zero span) | no change | A proposal for S-0317's next revision; S-0317's departure is kept until then |
| Changes for other specs: S-0302 claim 1 (sources table) | built | The landing hits and the expiry damage have no attacker and no house (`pieces::ground_hits`, `area::detonate` with `source: None`, `house: None`) |
| Changes for other specs: S-0303 claim 23 (a third producer) | built | The expiry damage is `area::detonate` with resources affected and none of the projectile-only effects; its light flash is the seam above |
| Changes for other specs: S-0414 claim 21 (the tally reading) | no change | The reading S-0317 proposed is the one our tally already uses (no house a kill, the victim's house a loss) |

**Lane.** See the end of this file.

## Lane (by-eye check for the lead)

The `debris-left` scene of `crates/app/tests/m7b_lane.rs` (play note 13) on the real install, run
on main 707b61f (before) and on this step rebased onto main b236238 (after); output under `local-lane/m8/a3-15a/`
(`debris-left-before`, `debris-left-after`). It kills 14 enemy objects, 6 of them vehicles, and
counts what stays.

| ticks after the kills | bouncing animations before | after |
|---|---|---|
| +50 | 10 | 4 |
| +100 | 10 | 0 |
| +300 | 10 | 0 |
| +1000 | 10 | 0 |

Corpses, voxel debris pieces, walkers and particle systems end as before. The extra short-lived
"other" animations after the step at +50 to +150 are those the landings make (claims 7 to 9).
The screenshots (`debris-left-300.png`, `debris-left-1000.png`) still do not
frame the kill spot, as in note 13's run, so the lead's by-eye check is a game where sprite
debris should now fall, land and disappear.
