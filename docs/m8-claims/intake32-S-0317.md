# S-0317 r57: claim ledger, M8 step A3 intake 32

This intake takes in one spec revision, which answers three requests, so the three interims of C5
part 2 ([c5-S-0317.md](c5-S-0317.md)) are retired: Q-1834 (the rebound on a slope, the turn off
a cliff face, which cells count as holding a building), Q-1835 (how a piece is drawn) and Q-1836
(placement and the cells off the map).

- **S-0317 revision 57** (spec commit f6e5ac1; the requests answered in ec5eb2a, 009f627 and
  fee676d). It covers r35 to r57 against r34, which C5 part 2 built. The whole diff was taken in,
  not only the three answers:
  - Claim 10: the cliff test's line is *u* < −0.0002 (double precision), not *u* < 0, and *z*₀
    is the flight position's height (10 leptons above the position in a falling piece's first
    turn). The rebound on a slope is claim 17's, the cliff turn claim 18's, the building cells
    claim 19's; *e* is narrowed to single precision before a rebound scales.
  - Claims 17 to 21 (new): F8's rebound in the cell's tilted plane, F9's cliff turn (straight,
    diagonal and the two diagonal faults), the cells that hold a building, the cell off the legal
    map, no placement at exactly (0, 0, 0).
  - Claims 22 to 28 (new): the drawn piece. F10's orientation from the net count of flying turns,
    reversed at each contact; one section (VoxelIndex); S-0306 F16's light without the view; the
    palette, brightness and translucency; the draw point and depth; F11's shadow; visibility.
  - Claim 2: a meteor piece whose counter wraps is placed at F2's start, wherever it lies, and
    flies as any other (claims 20 and 21); no longer open.
  - Claim 7: the hits do not skip an object with no health left (now the spec's rule, as S-0319
    claim 7 states it for sprite animations).
  - Claim 15: the spin turns the model in each flying turn, reversing at each contact (claim 22).
  - Claims 1, 3, 4, 11, 12: backing and confidence text, the palette as the house's drawn effect
    (claim 25), claim 12's off-map neighbours by claim 20. No rule change beyond the above.
  - Constants: the high cell 2, the cliff speed line −0.0002, the north-west factor
    2.3771235942840576, the off-map cell, the origin, the default section, the shade of normals
    253 to 255, brightness 1000, the two depth biases, the shadow's deck lift and *x* shift.
  - Edge cases: two new switches, `quirks.voxel_debris_deck_rebound_slope` and
    `quirks.voxel_debris_cliff_step_faults`; S-0105's `voxel_light_first_section_mode` and
    `voxel_mode0_misread` named for claim 24.
  - Import mapping: `VoxelIndex` and `Translucent` (new rows); `Elasticity` kept as a double, 0.8
    in single precision when absent.
  - Scenarios V1 to V15 (V14 and V15 new) and Changes for other specs: text for the oracle and
    other specs.

The cross-specs it names were read as they stand: S-0105 F10 and claims 18 and 35, S-0306 F16
and its wall overlays, S-0102 F8, S-0211 Terms and claims 9 and 15, S-0214 Terms, S-0209 claim 13,
S-0205 claim 7, S-0009 claim 44, S-0613 claim 10, S-0616 claims 4 and 5. None was revised for this
intake. S-0319 r26 (spec commit 3b1bf3b, also on main) was not taken in: it is the next intake.

**Code.**
- `pieces::fly` (sim): claim 10's cliff test with the −0.0002 line and the flight position's
  height. A contact the cliff test lets stand rebounds by F8 (`rebound`): the flat cell's
  reflection, or for a sloped cell the reflection in the tilted plane of the cell's slope shape
  (`slope`), *ε* = *e* narrowed once. On a deck the ground's slope is used under
  `quirks.voxel_debris_deck_rebound_slope` (default on); off, the deck is flat. An undone move
  turns by F9 (`cliff_turn`), scaled by *e* as imported. With
  `quirks.voxel_debris_cliff_step_faults` (default on) the north-east and south-west steps examine
  the far-side cells and swap their faces, and the north-west case with only the northern cell
  high gives (*c*, *u*ₓ, *u*ᵧ); off, the corner cells and a turn across *y*.
- `pieces::building_cell` (claim 19): the first building listed in the ground layer decides; a
  laser-fence section or a vehicle-like building does not count; with no building, the three wall
  overlays count.
- Claim 20: off the legal map the cell is level 0, ground 0, slope shape 0, clear, and the piece
  is never removed for being there. Claim 21: no piece is placed at exactly (0, 0, 0).
- `Piece::turns` (wrapping `i32`) and `Piece::reversed` (claim 22) replace the creation tick;
  each flying turn adds ±1 after its contact flips the sense. Both are saved and hashed: save
  format 82, AI game format 44.
- `PieceType::elasticity` is a double as imported; `epsilon` its single-precision narrowing (F8).
- Importer (`sim_pieces.rs`): `VoxelIndex` and `Translucent` read; warnings for `MaxXYVel` of 256
  or more (claim 18's departure) and for a `VoxelIndex` outside the type's own model (claim 23).
  `TypeArt::voxel_sections` carries the model's section count.
- Render: `voxel::piece_rotation` (F10, Rodrigues about the normalised axis by spin × net
  turns), `voxel::draw_piece_shadow` (F11 on the shared footprint core), `Scene::replace_image`
  and `MapRenderer::sync` re-uploading rewritten slots. `ImageKey::Piece` removed.
- App: `mapview::build::live_piece` draws claims 22 to 28: the VoxelIndex section, the first
  section's normal mode under `voxel_light_first_section_mode`, the projectile shade table of
  F16's diffuse term, the shadow first (Reader, flat profile, bias −4 − *L*(*h*ₛ), at *L*(*f*), or
  *L*(*f* + 417) over a high bridge when *z* ≥ *f* + 416), then the body (bias −2 − *L*(*z*)), the
  house's palette, the start-up mine table for no house (resource flag or not), brightness from the
  cell's light (1000 off the legal map), `Translucent` at 50 %. `mapview::live` fills the ground,
  bridge and on-map fields; `live::piece_pose` removed.
- `mapview::piece_pool` (new, design): each piece holds one body and one shadow image slot,
  rewritten in place for each new pose and freed when the piece leaves play.

**Design (Decisions of 2026-10-25).**
- Claim 18's multi-cell move (a departure, the spec's design): the step direction from the signs
  of the two cell changes; the importer warns for *M* ≥ 256.
- Claim 21: our engine keeps no record of a piece that is not placed. Nothing reads it again; its
  system keeps it as an owner that never moves and is never marked (S-0616 claims 4 and 5), and
  its start sound stays at the creation point.
- Claim 19's laser-fence sections: section states are not modelled (S-0211), so every section
  counts as inactive and none is a building contact, as for S-0613 claim 10's gas obstacles.
  An active section would count; seam.
- F10 in the renderer is formed in `f64` and applied in `f32`: presentation only, not in the
  simulation. Matching captured frames bit for bit is not claimed.
- Image slots: a slot is a square half as large again as the first drawing's larger side, plus 2;
  a drawing that does not fit moves to the smallest free slot it fits, or a new one.

| Claim or item | Status | Notes |
|---|---|---|
| 1 (hits without a house; a detonation's pieces; the palette as the house's drawn effect) | tested | Unchanged rules: `claim_1_hits_carry_no_attacker_and_no_house`. The palette: `a_piece_draws_its_section_in_its_colours`. |
| 2 (a wrapped meteor counter: placed at F2's start, anywhere) | tested | `claim_2_wrapped_meteor_start` (counters −2147483648 + *j* for *j* = 0 and 10: the piece is in play at F2's start converted by S-0205 claim 7, and flies on or ends in its first turn as claim 6's count-down says); off-map flight by claim 20 (`claim_20_off_the_legal_map`). Interim Q-1836 retired. |
| 3, 4, 11, 12 (backing and confidence text) | tested | No rule change; claim 12's off-map neighbours take claim 20's cell (`claim_20_off_the_legal_map`). |
| 7 (no skip for an object with no health left) | by construction | Already the code's reading (C5 part 2's decision); now the spec's rule. No scenario here holds such an object in the cell. |
| 10 (cliff line −0.0002, *z*₀ the flight position's height, *e* narrowed for a rebound) | tested | `claim_10_cliff_line_and_flight_height` (a *u* of 0 is undone when lifted more than 1 above *z*₀ + *u*; just below −0.0002 the descending condition; a falling piece's first turn measures from the flight position, 10 higher); `claim_10_cliff_undone`, `claim_10_high_bridge_deck`, `claim_10_deep_move_and_building_cell` still pass. |
| 15 (spin turns the model in each flying turn, reversed at each contact) | tested | `claim_22_net_turns_and_reversal`, `piece_rotation_turns_about_the_axis`. Interim Q-1835 retired. |
| 17 (rebound on a sloped cell, F8; the deck switch) | tested | `claim_17_f8_rebound_on_a_slope` (each slope shape against F8 worked in the test's own single-precision steps; a flat cell gives *ε*(*v*ₓ, *v*ᵧ, −*v*_z); on a deck with the switch on the ground's slope, off flat). `elasticity_as_imported_and_narrowed`. Interim Q-1834 retired. |
| 18 (turn off a cliff face, F9; the faults switch) | tested | `claim_18_f9_cliff_turns` (each straight step; each diagonal case of F9's table with the switch on and off; the north-west constant; *e* = 0.3 as imported, not narrowed). The multi-cell move is a departure (design above): `piece_fast_horizontal_limit_warns`. |
| 19 (cells that count as holding a building) | tested | `claim_19_cells_that_hold_a_building` (a building counts; a laser-fence section and a vehicle-like building do not; each of the three wall overlays alone counts, another overlay does not; a fence listed first hides a wall overlay; of two buildings the first listed decides). By construction: owner, construction state and gates are not read; only the ground layer is listed. Laser-fence sections: design above. |
| 20 (off the legal map) | tested | `claim_20_off_the_legal_map` (never removed; level 0, ground 0, clear; lands and rebounds at height 0). Drawing: brightness 1000 off the map (`live_piece`, cited). |
| 21 (placement at creation; not at the origin) | tested | `claim_21_no_placement_at_the_origin` (no piece in play, no draw beyond its creation's, its system and sound still made). |
| 22 (pose and spin; F10) | tested | `claim_22_net_turns_and_reversal` (+1 per flying turn, the sense reversed by each contact; wraps), `piece_rotation_turns_about_the_axis` (identity at creation, right-handed, axis fixed, a zero axis without spin), `a_spinning_piece_keeps_its_image_slots`. |
| 23 (one section, VoxelIndex; outside the model nothing) | tested | `a_piece_draws_its_section_in_its_colours`; importer `piece_section_outside_the_model_warns`. |
| 24 (light: F16 without the view, first section's mode, the two S-0105 switches) | partly | Drawn with the projectile shade table and the first section's normal mode under the switch (cited in `live_piece`). Partly: the test art has no VPL, so bodies draw blank in tests and no test reads the shades. |
| 25 (palette, brightness, translucency) | tested | `a_piece_draws_its_section_in_its_colours` (the house's palette; the start-up mine table without a house, resource flag or not; `Translucent` at 50 %). The cell's light as brightness: cited, not tested. |
| 26 (draw point and depth) | tested | `a_piece_draws_its_shadow_and_model` (Reader, flat profile, −2 − *L*(*z*), around the lifted point). |
| 27 (shadow; F11) | tested | `a_piece_draws_its_shadow_and_model` (shadow first, bias −4 − *L*(*f*)), `a_piece_over_a_deck_shadows_the_deck` (*f* + 416 and higher: *L*(*f* + 417), depth from *f* + 416; below, the ground). |
| 28 (visibility: every drawn frame) | tested | `live_pieces_are_drawn_in_the_air_layer_until_they_end`, `a_spinning_piece_keeps_its_image_slots`; GPU `sync_rewrites_replaced_images` (a rewritten slot reaches the screen). |
| F8 to F11 | tested | As claims 17, 18, 22 and 27. |
| Constants (new rows) | tested | Each cited where used; checked by the tests above. |
| Edge cases and quirks (two new switches; S-0105's two for claim 24) | tested | Both new switches both ways in `claim_17_f8_rebound_on_a_slope` and `claim_18_f9_cliff_turns`; `every_spec_quirk_has_a_switch` lists no S-0317 quirk; `every_rendering_spec_quirk_has_a_switch` passes. |
| Import mapping (`VoxelIndex`, `Translucent`, `Elasticity` as a double) | tested | `piece_keys_are_covered`, `elasticity_as_imported_and_narrowed`, the two new warnings. |
| Measurement scenarios V1 to V15, Changes for other specs | not ours | Oracle scenarios and other specs' text. |
| Open questions (r57) | design | The zero spin axis draws without spin (`piece_rotation` gives none); claim 12's fall-backs, the multi-cell cliff move, the section outside the model and the system's place in the update order are our design, as stated. |

**Goldens, arena, sweeps.** No golden moved (`cargo xtask gate` ok, native and wasm32): the gate
games make no debris pieces. M6c, M7a and M7b seed sweeps 1 to 16, reference and with
`GLASNOST_PLAY_FIXES=1`: all exit 0, output identical before and after apart from timings. Arena:
M-0001 31 of 96 pairings failing, M-0002 13 of 24, M-0003 2 of 35, before and after; the arena
output folders are identical. Runs in `local-lane/m8/a3-32/before` and `after` (git-ignored).

**Local lane.** New scene `debris-pieces` in `crates/app/tests/m7b_lane.rs`: six enemy armed
vehicles destroyed, their pieces followed tick by tick. On the install's first map: 3 pieces, up to
about 1000 leptons high, first contact at +43 ticks (the view then centres on that piece), 12
contacts in all, the net turn count falling after each reversal, all pieces ended by +151.
Screens `debris-pieces-002` to `-051` and `-end` in `local-lane/m8/a3-32/lane/`.

**Requests.** None; Q-1965 to Q-1969 unused.
