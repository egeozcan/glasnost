# Spec revisions after M7b: S-0306 r16 and r17, S-0007 r14 and r15, S-0301 r28 (projectiles and the target line)

This step takes in the drawing half of the projectile spec and the target line. S-0306 revisions 16 and 17 answer Q-1570 (how a projectile in flight is drawn), S-0301 revision 28 answers Q-1580 (the target line), and S-0007 revisions 14 and 15 give a projectile's art. Launch, flight and detonation are unchanged since revision 15, which was taken in at 61f0d3e. Classes are as in [step1b-m7b.md](step1b-m7b.md): *changed*, *satisfied*, *seam*, *(c)*.

| Spec | Revision | Answers | Outcome |
|---|---|---|---|
| S-0306 projectiles | 16 (bacb019) and 17 (e589b1a), from 15 | Q-1570 | changed (drawing only) |
| S-0007 art lookup | 14 (13ef391) and 15 (62d9d1b) | none | changed (claims 26 and 27, projectile art); claim 21 is intake 14B's |
| S-0301 firing cycle | 28 (0da403e), from 27 | Q-1580 | changed (claim 24) |
| S-0102 draw order | 24 (claim 13's projectile rows) | none | changed (the rows are used for the first time) |

**Format.** Nothing that is saved or hashed changes. The target line travels as a presentation-only `Effect::TargetLine`, which is neither saved nor hashed, like the other effects. The importer quirk `projectile_image_afresh` lives in the art quirks, not in the ruleset. The save format (67), the AI game format (29) and the goldens do not move. Default play does not change: the simulation takes no new draw and changes no state.

**Requests.** Q-1580 and Q-1570 are answered and their interim rows are retired. Q-1600 (filed in this intake, commit 6c1c878) asks where a turreted voxel vehicle's shot animation sits against its drawn barrel. Interim: none needed, the code follows the specs as they stand.

## S-0306 revisions 16 and 17

| Claim | Class | Notes |
|---|---|---|
| Terms: the drawing fields of a projectile's image (rotating, animation low, high and rate, anim palette, flat, voxel), the general palette | changed | `glasnost_convert::art::ProjectileLook`, filled by `art::projectile` from the image's art section (`held`: fields held across type reads, skipped while the image name is empty). The general palette is the fixed palette file of S-0111, untinted (`build::Builder::general_row`). Test: `projectile_drawing_fields` (`crates/convert/src/art.rs`). |
| 28, what is drawn and where: the draw point with the height lift, a sprite centred on it or a voxel body placed as a part, never both; surface layer when flat, air otherwise; nothing for an invisible one, one waiting for its detonation animation, a sprite without a file or a voxel without art; a frame number past the frame count draws nothing (no shadow either) | changed | `live::Live::projectile_items` and `build::Painter::projectiles` (`live_projectile`, `voxel_projectile`). The old 7 × 7 diamond is drawn only by the debug layer, and only when no body drew. `live::Live::sync_layers` places projectiles in the surface or air layer. Tests: `sprite_body_and_shadow`, `nothing_drawn_without_a_frame_or_art`, `live_projectiles_are_drawn_in_the_air_layer` (`crates/app/src/mapview/projectile_tests.rs`). |
| 29, the sprite image frame: animated (counter *a* of F15, whatever the rotating flag says), rotating ((28 − ψ) mod 32), else 0; chosen afresh each drawn frame | changed | `mapview::projectiles::sprite_frame`. Tests: `sprite_frames`, `animation_counter` (`crates/app/src/mapview/projectiles.rs`). |
| 30, the model body: body model only, motion-file frame 0, the orientation Rz · Ry from ψ and *e*, the F16 shade table, the colour definition's 53-shade palette at brightness 1000, depth (non-flat: none; flat: strict flat reader with bias −65 and no lift), no shadow | changed | `glasnost_render::voxel::{projectile_body, projectile_shade_table, draw_model_shaded}`, `ImageKey::Projectile`; `build::Builder::voxel_projectile` (Shades → the named definition's 53-shade row, Single → the first definition's single-shade row). Tests: `projectile_body_orientation`, `projectile_shades_are_diffuse_only` (`crates/render/src/voxel.rs`), `voxel_body` (`projectile_tests.rs`), `projectile_colour_walk` (`art.rs`). |
| 31, the sprite palette: anim palette, else the firer's house palette (the local player's when there is no firer), else the general palette; brightness 1000; never translucent | changed | `live::Live::projectile_items` picks `ProjectilePalette::{Anim, House, General}`. Test: `sprite_palettes`. |
| 32, the shadow: sprite with the shadow flag, η above 0 (above the deck over a high bridge's cell at or above 416), the body's image frame drawn as a shadow *L*(η) rows below, just before the body | changed | `mapview::projectiles::shadow`, `build::Painter::projectiles`. Test: `sprite_body_and_shadow`. |
| 33, the light-glow image of a projectile's translucent sprite | seam | The client draws no light-glow images yet (`cover.rs`); when the glow ends is an Open question of S-0306. No request. |
| F14 drawn direction and pitch | changed | `glasnost_sim::combat::projectile::drawn_facings` (integer, no floats). Test: `f14_drawn_facings` (`claims_s0306.rs`). |
| F15 animation counter | changed | `mapview::projectiles::counter` (cycles skipped for long flights). Test: `animation_counter` (scenario P42 part c). |
| F16 shade of a voxel projectile | changed | `voxel::projectile_shade_table`. Test: `projectile_shades_are_diffuse_only`. |
| Constants, Edge cases (animation fields outside 0 to 255 kept modulo 256 with a warning; the original's fault on absent voxel art: we draw nothing), Import mapping rows (Shadow default yes, FirersPalette default no, the colour definition key) | changed | `art::projectile`. Test: `projectile_drawing_fields`. |
| Claim 3, the aim point (used by the target line's end) | satisfied | `combat::projectile::led_aim_point` factors out the existing lead. Test: `claim3_led_aim_point_of_a_still_target`. |
| P42 scenario text, backing, confidence, Open question (glow) | (c) | |

## S-0007 revisions 14 and 15

| Claim | Class | Notes |
|---|---|---|
| 26, the projectile image name taken afresh at every type read (cut to 24 bytes; empty without a key; no default of the type name; the importer warns when a later read drops the key) | changed | `art::projectile`, `ArtQuirks::projectile_image_afresh` (on by default; off keeps the earlier name, still warned), listed in `EXTERNAL_QUIRKS`. `glasnost_convert::sim::images` no longer gives a projectile the art-image override. Test: `projectile_image_is_taken_afresh`. |
| 27, the projectile art: sprite by theater flag, new-theater substitution or I ‖ .shp with the generic fallback; voxel parts of claim 16 for a non-vehicle (only the body is drawn); the arctic model only at a read where claim 10 appends the A; never the A for a sprite; invisible ones not looked up | changed | `art::projectile`; `sim_combat` takes the voxel flag from the look. Test: `projectile_voxel_and_arctic_art`. |
| 23 (revision 15), a projectile's translucent sprite is S-0306 claim 33's glow | seam | As S-0306 claim 33 above. |
| 21 (revision 15), shadow files | not this intake | Intake 14B. |

## S-0301 revision 28

| Claim | Class | Notes |
|---|---|---|
| 24 a, every shot that reaches the shot-animation choice starts the target line of a vehicle whose type has the flag; the other exclusions of claim 23 start none; the owner test at the shot | changed | `combat::fire` pushes `Effect::TargetLine { firer, house }` after `shotanim::create`. Tests: `claim_24_a_every_choosing_shot_starts_the_target_line`, `claim_24_a_excluded_shots_start_no_target_line` (`claims_s0301_shotanim.rs`). |
| 24 b to h, one timer per vehicle, 15 ticks, restarted by a new shot; drawn in the vehicle's own drawing step while it has a target; from the turret pivot to the led aim point of the current target, recomputed each frame; projected with the height lift; palette index 8 of the general palette in row 26, dotted one pixel per step along the longer axis with the phase swapping each tick, 3 × 3 squares at both ends with the end point at the bottom right | changed | `live::Live::take_effects` (timer, the viewer test), `live::Live::draw_target_lines`, `build::Painter::target_line`, `build::dotted_line`, `combat::position::turret_pivot`. Our choices where the claim leaves it open: which phase a tick draws (the tick's parity), the rounding of the walk (nearest), and with no viewer (the map viewer) every house's lines show. Tests: `dotted_lines`, `target_line_follows_its_timer`. |
| Claim 23's sentence on the target line | (c) | Moved into claim 24. The `TargetLaser` field docs now cite claim 24. |

## S-0102 revision 24 (claim 13's projectile rows)

| Row | Class | Notes |
|---|---|---|
| projectile sprite: −30 − *L*(z), flat profile | changed | `build::PROJECTILE_BIAS`. |
| projectile shadow: −10 − *L*(ground or deck) | changed | `build::PROJECTILE_SHADOW_BIAS`. |
| non-flat voxel projectile: no depth test; flat voxel projectile: −65, no lift | changed | `build::FLAT_VOXEL_PROJECTILE_BIAS`, `DepthTest::Ignore`. |
