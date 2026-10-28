# S-0319 r26: claim ledger, M8 step A3 intake 33

This intake takes in one spec revision. It answers Q-1846 and Q-1847, the rest of Q-1762, so the
interims of intake 15 part 1 ([intake15a-S-0319.md](intake15a-S-0319.md)) are retired, and so is
its start-sound seam.

- **S-0319 revision 26** (spec commit 3b1bf3b). It covers r10 to r26 against r9, which intake 15
  part 1 built. The whole diff was taken in, not only the two answers:
  - Terms: the cell read for a position (trunc(*x*/256), trunc(*y*/256); not legal: the off-map
    cell) and the high cell of the cliff turn.
  - Claim 6: the building cell is S-0317 claim 19's rule (the first building listed decides; not
    an inactive laser-fence section or a vehicle-like building; with none listed, the three wall
    overlays). The cliff test's line is *u* < −0.0002 in double precision, *z*₀ the flight
    position's height. The off-map cell is blank.
  - Claim 7: the rebound is F6 (the cell's tilted plane, *ε* = *e* narrowed once), the cliff turn
    F7 (*e* as imported). At a deck contact the ground's slope (a fault). The hits walk the live
    listing (a fault). No skip for an object with no health left. The multi-cell cliff move
    (a departure) and its importer warning on the bound *H*.
  - Claim 10: offers repeat on a cell that received nothing (a terrain object, a refused
    placement, a place past the list); the placement is S-0202 claim 10's, with its conditions and
    draws; the stage draw comes last; places of 255 or more are ordinary; no list entry (a fault);
    the past-list departure and its importer warning.
  - Claim 11: off the legal map; the neighbours; placement anywhere except exactly (0, 0, 0).
  - Claims 2, 3, 5, 8, 9, 12, 13, 14: the (0, 0, 0) exception, the start sound follows the
    animation (claim 5), the placement draws in claim 13; backing and confidence text otherwise.
  - F5 to F7 (F6 and F7 new), Constants (the cliff line, the north-west constant, the 256 warning
    limit), State and ordering (the (0, 0, 0) row).
  - Edge cases: three new switches, `quirks.bouncing_anim_rebound_faults`,
    `quirks.bouncing_anim_impact_hits_walk` and `quirks.anim_resource_no_growth_entry`; the
    departures named.
  - Import mapping (the two warnings), scenarios BA1 to BA13, Changes for other specs (S-0302
    claim 8, S-0312 claim 21, S-0401 claim 10 new), Open questions.

The cross-specs it names were read as they stand: S-0317 claims 7, 10, 17 to 21 and F6, F8, F9
(r57, intake 32), S-0106 C4, S-0202 claim 10, S-0209 claim 13, S-0211 Terms and claims 9 and 15,
S-0214 Terms, S-0306's wall overlays, S-0307 claim 11, S-0401 claims 7, 10 and 11, S-0613 F3,
S-0009 claim 44, S-0205 claim 7, S-0203 claim 6, S-0312 claims 15 and 21. None was revised for
this intake. S-0212 r82 (also on main) is intake 34's, not taken in here.

**Code.**
- `bounce::step` (sim): the building band uses `pieces::building_cell`. The cliff test draws
  the line at the double nearest −0.0002. An undone move turns by `pieces::cliff_turn` with *e*
  as imported; the north-west turn under `quirks.bouncing_anim_rebound_faults`, off across the
  second axis. A contact the cliff test lets stand rebounds by `pieces::rebound` (F6), off the
  new cell's slope shape. At a deck contact that is the ground cell's shape under the switch, the
  identity with it off. `BounceType::elasticity` is a double as imported, `epsilon` its narrowing.
- `bounce::land`: the hits are `pieces::ground_hits` with the walk under
  `quirks.bouncing_anim_impact_hits_walk`. On, the next object is the one listed after the object
  just hit, and the hits end when that object is no longer listed. Off, the list as the hits
  begin, skipping objects out of play. Pieces keep the list as the hits begin (S-0317 leaves the
  walk open for pieces). Each object's radius test and distance are taken at its own hit.
- `bounce::resources` and `offer`: per *i*, the pairs of F5 offer the neighbour until an offer
  places an overlay or takes no draws. An offer that places nothing (a terrain object listed in the
  cell, a placement refused, a place past the list) takes both draws and leaves the cell germinable.
  Eight successive *i* with no draws end the walk, since nothing has changed. The placement is
  `overlays::place_in_play` (new). The vein overlay goes in only when the vein placement test
  passes. An end marker is refused when its end row holds an overlay. Under
  `quirks.anim_resource_no_growth_entry` off, a placed resource cell is offered to its tiberium's
  growth list (`econ::resource::offer_growth`, now `pub(crate)`) before the stage draw. The
  intake 15 bar on a cell for the rest of the landing is gone.
- `anims::spawn`: a bouncing animation whose placement point is exactly (0, 0, 0) takes its
  creation draws and is not put in play.
- Audio: an `Effect::Anim` whose animation is in play with a flight plays its start sound attached
  (anchor `1 << 43` | id), moved to the animation's position each tick while it is in play.
- Importer (`convert::sim::bouncing_fields`): a warning when *H* = max(|*M*|, | |*W*ₓᵧ| − 1 − *M* |)
  is 256 or more, or when only |*M* + |*W*ₓᵧ| − 1| is (named as under
  `quirks.random_magnitude_overflow`, which the importer cannot see); a warning for a
  `TiberiumSpawnType` with fewer than three overlay types after it.
- Three switches in `Quirks` (default on) and the schema. No new saved or hashed state: save
  format 82, AI game format 44, unchanged.
- Test hook `bounce::step_for_test` (doc-hidden): one motion step without its effects, for the
  F6 and F7 velocities, which a sprite animation does not keep past its landing.

**Design.**
- A deck contact for the rebound fault is a crossing from above onto the deck (the contact that
  puts the animation at the deck height). The under-deck landing rebounds off the ground cell as
  any contact does. This is the same reading as S-0317's `voxel_debris_deck_rebound_slope`.
- (0, 0, 0): the start event of an animation that is not placed is still made when it has no loop
  delay (a deferred one never comes), and its number is used up. Nothing keeps a record of it.
- The vein placement's other effects (cell animation, recalculation) follow in `place_overlay`'s
  order after the vein pattern. A refused end marker recalculates nothing.
- The walk ends when the object just hit is no longer listed, whatever took it out (claim 7's
  design rule for infantry and buildings that leave at once is a case of this).
- The start sound of an animation that has left play plays on where it was, as a piece's.

**Seam.** A place of 255 or more within the overlay list places nothing. A cell holds its overlay
in one byte with 255 meaning none (`glasnost_map::Cell`), so it cannot hold that overlay. Claim 10
calls it an ordinary overlay.

| Claim or item | Status | Notes |
|---|---|---|
| 2, 3 (placed unless at (0, 0, 0)) | tested | `claim_11_not_placed_at_the_origin`, `claim_11_meteor_start_off_the_map`. |
| 5 (the start sound follows the animation) | tested | Audio `bouncing_animation_start_sound_follows_it` (moved off the screen with it and back; after it leaves play the sound stays). Seam of intake 15 retired. |
| 6 (building cell kinds) | tested | `claim_6_kinds_of_building_cell` (a building counts; a laser-fence section, a vehicle-like building do not; the three wall overlays alone count, another overlay does not; a fence listed first hides a wall). First-listed of two buildings: the shared `pieces::building_cell`, tested in `claims_s0317::claim_19_cells_that_hold_a_building`. `claim_6_building_cell` (the 150 band) still passes. Interim Q-1846 retired. |
| 6 (cliff line −0.0002, *z*₀ the flight height) | tested | `claim_6_cliff_line_and_flight_height` (*u* = 0 undone above *z*₀ + 1; −0.0001 not below the line, −0.001 below; a rising animation measured from its flight height); `claim_6_cliff_undone_first_turn` still passes. |
| 6, 11 (cell read; the off-map cell; neighbours) | by construction | The shared helpers of S-0317 claim 20 (`pieces::ground`, `level`, `slope`, `water`, `building_cell`), tested there; `claim_11_meteor_start_off_the_map` keeps its turns off the map. |
| 7 (F6 rebound; deck fault) | tested | `claim_7_f6_rebound_on_a_slope` (shapes 0, 1, 6, 14, 18 against F6 worked in the test; flat shapes the flat rule; *ε* = 30 % narrowed; a deck over shape 1 with the switch on the slope, off flat). |
| 7 (F7 cliff turn; north-west fault) | tested | `claim_7_f7_cliff_turns` (straight steps across *y* and *x*; d 7 with (*x* − 1, *y*) high across *x*; with (*x*, *y* − 1) high the north-west turn (*c*, *u*ₓ, *u*ᵧ) on, across *y* off; *e* = 30 % as imported). `claim_7_measure_after_a_cliff_turn` (an impact landing after an undone descending move, both settings). Multi-cell move: departure, `bouncing_importer_warnings`. |
| 7 (the hits' listing walk) | tested | `claim_7_impact_hits_walk_switch` (a vehicle listed before a building, killed by its hit: on, the building takes no hit; off, it does). Objects entering or taken out before they are reached: by construction (the walk reads the live list). |
| 7 (no skip at no health) | by construction | `ground_hit` skips only objects out of play; unchanged rule. |
| 10 (terrain object; repeated offers) | tested | `claim_10_terrain_object_neighbour` (the tree's cell takes both draws at each of its three pairs, no overlay). |
| 10 (past the list) | tested | `claim_10_place_past_the_overlay_list` rewritten (each pair repeats while the place lies past the list, stops when one places). Interim Q-1847 retired. |
| 10 (the placement's conditions: vein, end marker) | by construction | `overlays::place_in_play`; no test builds a 127-entry overlay list. |
| 10 (places of 255 or more) | seam | Above. |
| 10, 13 (no list entry; the switch) | tested | `claim_10_growth_entry_switch` (on: R(0, 3), R(0, 2), no entry; off: the raw offer draw between them and one growth-list entry). |
| 10, 11 (resource lists unchanged otherwise; F5 order) | tested | `claim_10_resources_neighbours`, `claim_10_resources_switch_and_none`, `claim_10_large_radius_neighbours` still pass. |
| 11 (placement anywhere; not at (0, 0, 0)) | tested | As claims 2, 3. |
| 13 (the placement draws) | by construction | `place_in_play` takes S-0202 claim 10's draws; the resource tests check the stream. |
| 1, 4, 8, 9, 12, 14 (text) | tested | No rule change; the intake 15 tests still pass. |
| F5 | tested | As claim 10. |
| F6, F7 | tested | As claim 7. |
| Constants (the cliff line, *c*, 256) | tested | Cited where used; checked by the tests above. |
| State and ordering (the (0, 0, 0) row) | tested | `claim_11_not_placed_at_the_origin`. |
| Edge cases (three new switches) | tested | Each both ways: `claim_7_f6_rebound_on_a_slope` and `claim_7_f7_cliff_turns` (rebound faults), `claim_7_impact_hits_walk_switch`, `claim_10_growth_entry_switch`. `every_spec_quirk_has_a_switch` lists no S-0319 quirk (it fails at this base only on four S-0212 r82 quirks, which A3 intake 34 takes in). |
| Edge cases (departures) | tested | The multi-cell move and the past-list place (above), the far off-map reading (shared helpers). |
| Import mapping (the two warnings) | tested | `bouncing_importer_warnings` (|*M*| ≥ 256; the edge term; the overflow term alone; none for the default; the spawn overlay type near the end and not). |
| Scenarios BA1 to BA13 | not ours | Oracle scenarios. |
| Changes for other specs (S-0302 claim 8, S-0312 claim 21, S-0401 claim 10 and the rest) | named, left | Text for other specs. Behaviour built here: the hits reach no-health objects and any listed building; a tree's cell takes no overlay from this offer. |
| Open questions (r26) | design | Killing hits that leave the listing: claim 7's design rule. (0, 0, 0): claim 11's design rule. |

**Goldens, arena, sweeps.** No golden moved (`cargo xtask gate` ok, native and wasm32): the gate
games make no bouncing animations. M6c, M7a and M7b seed sweeps 1 to 16, reference and with
`GLASNOST_PLAY_FIXES=1`: all exit 0, output identical before and after apart from timings. Arena:
M-0001 31 of 96 pairings failing, M-0002 13 of 24, M-0003 2 of 35, before and after; the arena
output folders are identical. Runs in `local-lane/m8/a3-33/before` and `after` (git-ignored).

**Local lane.** New scene `bouncing-anims` in `crates/app/tests/m7b_lane.rs`: six enemy armed
vehicles destroyed, the bouncing animations followed tick by tick. On the install's first map: 3
animations in flight from +1 (the view centres on the first), rising from about 680 to about 1070
leptons, falling, and leaving play between +46 and +52. Screens `bouncing-anims-001` to `-045` and
`-end` in `local-lane/m8/a3-33/lane/`.

**Requests.** None; Q-1970 to Q-1974 unused.
