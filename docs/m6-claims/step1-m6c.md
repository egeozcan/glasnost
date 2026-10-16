# M6c step 1 claim ledger (spec revisions that landed during the session)

S-0508 revision 7 has its own ledger ([S-0508.md](S-0508.md)). This file covers S-0105 revision 20 (answers Q-0950) and S-0007 revision 13 (answers Q-0951).

## S-0105 revision 20

| claim | status | tests and code |
|---|---|---|
| 3 (heading sources) | tested | A flying-locomotor aircraft's body and shadow take the aiming facing, a rocket and every other body the body facing: `build::vehicle_voxels` (`heading`), test `aircraft_attitude_is_drawn` (the body facing plays no part for a flying body; the aiming facing is the heading). |
| 26 (cache keys) | tested | `ImageKey::RocketBody` (heading, frame, pitch class) and `ImageKey::FlyingBody` (heading, frame, pitch presence, bank state), no ramp state: `rocket_body_keys`, `flying_body_keys` (render), `aircraft_attitude_is_drawn` (reuse inside a class and heading, none across them). |
| 30 (shadow transform and key) | tested | Rocket shadow *V · Rz(θ_body)*, flying shadow *V · Rz(θ_flight)*, never the pitch or bank, key = heading bucket: `rocket_shadow_is_not_pitched`, `flying_pose`, `aircraft_attitude_is_drawn`. |
| 41 (rocket pitch) | tested | `rocket_body_pose` / `rocket_body_local` (*V · Rz(θ_body) · Ry(−p)*, the single pitch negated, F1 table trigonometry): `rocket_pitch_raises_the_nose_and_is_not_quantised`; reuse classes `rocket_pitch_class` (level, initial with the unnarrowed double product, final with the narrowed one, fresh; coincident values favour initial): `rocket_pitch_classes`; the pitch is read from the game by `sim::rocket::drawing` (`drawing_reads_the_current_pitch`). Local lane: `local-lane/m6c/attitude/v3-rocket-in-flight*.png`. |
| 42 (flying pitch and bank) | tested | `flight_attitude` (strict height > 0 and speed fraction > `PitchSpeed` in double precision; bank discrete by the turn direction; zero configured angle: no pitch or bank state in the key) and `flying_body_local` (*Rz · Ry(α) · Rx(β)*, angles narrowed to single toward zero): `flight_attitude_tests`, `flying_pose`; the game inputs come from `live::attitude`. |
| Import mapping | tested | `PitchAngle`, `RollAngle` (degrees × binary64 π/180 toward zero, −1 keeps the default 20 and 30 degrees), `PitchSpeed` (0.25): `build::rule_angle`, `rule_real`; the keys are `render` rows of the coverage table; `aircraft_attitude_is_drawn` uses `PitchAngle=40`, `RollAngle=-1`, `PitchSpeed=0.5`. The missile pitch fractions are S-0506's (`RocketSet`). |
| Cross-spec (S-0506 claims 15, 17) | noted | The pitch evolution is unchanged; only its appearance is added. |
| MS-31, MS-32 reuse observations | not built | They need a recorder of reuse against fresh draws; our cache cannot tell the two apart. |

Known departures and limits:

- **Fresh rocket pitches are retained.** Claim 41 says a pitch outside the three classes is drawn fresh and not retained. A fresh draw is a function of the table step of its rotation alone, so `RocketPitch::Fresh(σ)` keeps the image under that step: the pixels are those of a fresh draw; only the scene's image memory is bounded (the scene keeps every image added). Not observable in pixels.
- **Crashing flying bodies** are drawn level (heading only): S-0312's crash attitude rules are not drawn. A map-only scene draws aircraft level with the body facing.

## S-0007 revision 13

| claim | status | tests and code |
|---|---|---|
| 34 (own art section for animation playback and drawing) | tested | `build::anim_own_section` is the section every `slots::read_anim` caller passes (`slot_animations`, `live_anim`, `anim_type`); the conflicting-value, omitted-value and own-shadow/displacement cases: app test `animation_properties_come_from_the_own_art_section`; the importer's animation record read the own section already: `convert` test `animation_properties_are_the_own_sections`. `ShouldUseCellDrawer` defaults to yes (`animation_defaults`). Local lane: `local-lane/m6c/{all-owners,skirmish-view}/map-DESERT-24.png` show one tesla coil and clean reactor clusters. |
| 33 (non-voxel projectile sprite wording) | not built | `convert::art` has no projectile sprite lookup (the claim's trailer fields are read in `convert::sim_combat`). |
| 2, 9 (references to claim 34) | no change | The two theater flags stay claim 2's special table (`convert::art`, animations). |
| A25 | not run | The measurement scenario is the lead's. |

Playback fields of the table that the engine does not consume yet (`Next`, `Damage`, `MakeInfantry`, the sounds) are not read from the art by any code, so the own-section rule has nothing to apply to there.
