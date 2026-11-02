# S-0411 r49, S-0102 r34, S-0109 r42, S-0405 r51 and S-0404 r22: claim ledger, M8 step A3 intake 78

The last built revision of each spec is the one at 5f2b926 (batch brief of 2026-11-02); each delta
was read in full (`git diff 5f2b926 main -- <file>`):

- **S-0411** r28 to r49 (d87d8d6): answers **Q-2266** (the sidebar button images' transparent
  index and display colour). New in claim 19: *Shell images* and *Pixels*, their backing and
  confidence, the Formulas entry *a*ₘ, the `SIDEBAR.PAL` row and a new shell-images row of the
  Import mapping, S15's map overrides and its part (c).
- **S-0102** r32 to r34: claim 3's warp-member row (also in the *State and ordering* table) and
  claim 34's shrouded-cell exception for slot animations.
- **S-0109** r41 to r42: claim 29's creator row for aircraft damage smoke.
- **S-0405** r47 to r51: claim 20's sentences on shots at a switched-off hovering vehicle, and
  the matching Open question.
- **S-0404** r19 to r22. Our ledger names r19 as the last built (docs/m6-claims/step1f-m6c.md).
  r20 and r21 (f57504e, ce5f09d) change only the measurement setup's wording. r22 (810735d)
  restates the aircraft factory row's rally move and claim 15's reach.

Requests filed: **Q-2346** (when a warp member enters the ground render layer at the launch and
when it leaves it after the hold). Q-2347 to Q-2355 are unused.

**Client only.** The code changes are in `crates/app` (`game::sidebar_art`,
`mapview::layers`). The simulation files touched are two sim test files, and only their citation
comments change (`claims_s0501_r65.rs`, `claims_s0501_released.rs`). No simulation code, rules,
saved or hashed state changes, so the save format (93) and the AI game format (56) stay the same,
no golden moves and no lane is due. No quirk switch is added or changed: no delta names one.

Status words: **tested** (built and checked by the named test), **partly** (the part named is
untested or not built), **seam** (built by another intake's spec; the reason is given), **no
change** (the revision restates what is built).

## S-0411 r49

| claim part | status |
|---|---|
| 19 *Pixels*: every index 1 to 255 shows its converted sidebar-palette colour packed by S-0103 claim 9's 16-bit step (Q-2266, question 3) | Tested: `game::sidebar_art::palette_rgba` now packs each converted entry into 5-6-5 (`palette::hicolor_565`) and widens it by repeating the top bits (`compose::expand565`, the low-bit fill our display path uses for every hi-colour pixel). `sidebar_art::tests::claim_19_pixels_every_index_shows_its_packed_colour` (a test palette in the manner of S15 part c: odd red and blue bytes, no converted value from 128 to 159, never shown at full precision); `the_sidebar_palette_converts_each_byte_by_four` and `the_button_images_are_read_with_the_frames_claim_19_uses` now expect the packed values. The Q-2266 interim (8-bit precision) is retired |
| 19 *Pixels*: index 0 transparent, what lies beneath shows through, entry 0 plays no part (questions 1 and 2) | No change: the 2D painter discards index 0 for every sprite (`render/src/batch2d.wgsl`, sprite mode), so entry 0's colour is never read. The interim of intake 69 is now the rule. Not unit-tested (a shader path) |
| 19 *Pixels*: no index 1 to 255 drawn as a shadow or translucent | No change: the sprite mode returns the palette row's colour, opaque (`palette_rgba` sets alpha 255) |
| 19 *Pixels*: no change by house colour, map lighting, ion storm, map-tint or base-ambient trigger actions, or any other global effect; a button's state changes only its frame | No change, by construction: the sidebar palette is its own painter row, built once from the file alone, with no house-colour, shade or tint table. Lighting, tints and the flashes act on the battlefield image (`MAP_IMAGE`), which the sidebar is drawn over, not through. `hud::image` draws the frame claim 19's table gives with no styling (intake 69) |
| 19 *Shell images* and the Import mapping row `SIDE1.SHP`, `SIDE2.SHP`, `SIDE3.SHP`, `ADDON.SHP` | No change: our engine need not draw them (claim 19), and our shell is our layout's design (*Layout*). They are not read |
| 19 backing, confidence, Formulas (*a*ₘ), S15 map overrides and part (c) | No change: measurement text, nothing for the engine to build |

## S-0102 r34

| claim part | status |
|---|---|
| 3 and *State and ordering*: a superweapon warp member, any kind, is in the ground layer from its warp's launch until at least the end of its arrival hold, whatever its height; the row comes first | Tested: `mapview::layers::evaluate_at` asks `superweapons::warp::in_ground_layer` (the member state the simulation keeps from the launch until the hold ends) before every other row. `layers::tests::claim3_a_warp_member_is_in_the_ground_layer_whatever_its_height` (a jumpjet unit at its cruise height, an aircraft above 0 and a rocket-moved vehicle, at every on-map stage; the ordinary rows again after the hold) |
| 3: in the ground layer *from the launch*, although claim 4 names no moment at the launch | Partly (interim, **Q-2346** question 1): `Layers::evaluate_mobiles` puts a member that is still in an upper list into the ground layer after each tick. `claim4_a_warp_member_leaves_its_upper_list_at_the_launch`, both positions of `quirks.layer_change_on_events_only` |
| 3: when it leaves the ground layer after the hold "is not established" | Partly (interim, **Q-2346** question 2): the ordinary rows apply again at the next of claim 4's moments. With the switch on the member stays in the ground layer until a moment re-files it; with it off, the sweep moves it in the tick the hold ends. Same test |
| 34: the shrouded-cell row excepts a building's slot animations; an opaque animation over a marked building's cell takes no mask while the cell holding its own position is shrouded; its brightness unchanged; the supplying building's centre cell plays no part | No change: built from S-0513 claim 29b (intake 71). `mapview::protection::Masks::slot_animation`, tested by `protection::tests::claim29b_slot_animation_masks_follow_their_own_cell` (also the marked-only case). Comments now name S-0102 r34 |
| 34: whether a protected and marked building's body keeps the laser-target mask under a shrouded centre cell is open (S-0519 claim 26) | No change: open in S-0519's Open questions. Intake 3's interim stands (the laser-target mask stays; only the force-shield mask is withheld) |

## S-0109 r42

| claim part | status |
|---|---|
| 29: aircraft damage smoke's creator row (during the aircraft's own turn, right after its smoke chance draw, at its position then, after its own movement; no initial loop delay, repeat factor 1, no reverse request, no attachment; its creation draws are claim 3's delay draw and claim 13's start-event draws) | No change: built for S-0501 claim 37 (Q-2237) in `air::smoke`. Tested by `claims_s0501_r65::claim_37_a_living_aircrafts_smoke_is_the_fixed_type_where_its_movement_left_it` and `claim_37_the_smoke_creation_draws_follow_u`, whose citations now name S-0109 claim 29 |

## S-0405 r51

| claim part | status |
|---|---|
| 20: shots at a switched-off hovering vehicle that S-0503 counts as moving are led by S-0306 claims 3 b and 6 a | Seam (intake 79, S-0306 r64): the lead is S-0306's. Our lead (`combat::projectile::led`) already reads the hovering vehicle's moving state through `quirks.switched_off_hover_lead`, an S-0306 switch |
| 20: a led arcing shot can be void, and a void launch leaves the rearm countdown unchanged, so the shot is attempted again | Seam (intake 79, S-0306 claim 6 a and S-0301 claim 15) |
| 20: a straight or arcing projectile whose firer is still in play passes below the ground and flies on, except a flak-scattering one with a target (S-0306 T1 and T2) | Seam (intake 79, S-0306 T1 and T2) |
| 20: the lead reads the moving state, not the disabled state, so no targeting rule reads the disabled state | No change: `led` reads no disabled state |
| 20 backing and confidence; Open questions (M-0005's stall) | No change: measurement text |

## S-0404 r20 to r22

| claim part | status |
|---|---|
| r20 and r21: the measurement setup's wording (engine add-ons, test data) | No change: measurement text |
| 13, aircraft factory row: the rally move is the release's own (the rally point as destination and *move* queued), not a behaviour order of S-0213 claim 5 | No change: built for S-0501 claim 48 (d) (`econ::release::aircraft`). Tested by `claims_s0501_released::claim_48_rally_point_at_the_release`, whose citation now names r22 |
| 15: the height component is applied only at a factory release; docking by landing moves the aircraft horizontally and rests at the landing height (S-0501 claim 24) | No change: built (`air.rs`, pad guidance and the landing's dock). Tested by `claims_s0404::claim_15_pad_position`, `claims_s0501_released::claim_48_release_rests_above_its_pad` and `claim_24_docking_at_a_raised_pad_rests_at_the_landing_height` |
| 13 and 15 backing and confidence | No change |
