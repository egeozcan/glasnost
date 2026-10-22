# S-0313 revision 29 claim ledger, M8 step A3 intake 4

S-0313 revision 10 to 29 (spec commit 79395e1, answering Q-1600; fcea575 names S-0313 as its
answer): the title and Summary (where a turreted vehicle's shot animation is drawn), four new
Terms (the object height lift Λ, the scale ρ, the draw point, the unshifted draw point), the new
claims 11, 11a, 11b, 11c, 11d and 11e, the new scenario P10 and seven new Open questions. Claims
1 to 10, F1 to F4, the Constants, the Edge cases, the quirk and the Import mapping are unchanged:
no new quirk switch, no importer change.

**Simulation.** The fire point is game state, but no rule for it changes: claims 11 to 11e say
that no correction moves it, and claim 1 has no barrel term. `git diff --stat main --
crates/deterministic` is empty, so the save format (70), the AI game format (31) and the
sim-gate goldens do not move, and the oracle arena and the seed sweeps were not re-run.

**Code.** Client only. The painter already drew a vehicle's body, turret and barrel from
`project_lifted` of its position, with no locomotor shift (`Builder::mobile`,
`Builder::vehicle_voxels`), and a shot animation centred on `project_lifted` of its own position
moved down by its vertical draw shift (`Builder::anim_items`), so claim 11a's draw point holds as
built. One change: claim 11 asks that the turret be drawn with the same displacement δ that F2
uses for the fire point, and the importer holds `TurretOffset` across type reads while the
painter read only the final image's art section. The painter now takes the simulation's imported
δ when the typed rules name the type (`Builder::sim_turret_offset`), and the art read otherwise
(the map viewer without typed rules).

Tests: `crates/app/src/mapview/shot_anim_tests.rs` (our engine's side of P10, on invented art and
the simulation's synthetic combat ruleset; every expected position exact).

| claim | status | test names or reason |
|---|---|---|
| Front matter, title, Summary | no change | Text. The Summary's "the fire point is game state" is claim 1's, unchanged. |
| Terms: object height lift Λ | not implemented (original only) | Λ is the original's lift, a hypothesis of claim 11b; our lift is S-0101 claim 3's (`projection::height_lift`). |
| Terms: ρ, draw point | satisfied | ρ is S-0105 F3's (`voxel::pixels_per_lepton`); the draw point is S-0105 claim 25's, with no locomotor shift in our engine. |
| Terms: unshifted draw point | tested | Our draw point is the unshifted draw point, `project_lifted` of the position with S-0101 claim 3's lift: `run` in every P10 test checks the body, turret and barrel rectangles against independent draws placed there. |
| 11, scope (turret flag, voxel turret, driving locomotor, flat ground, no tilt; dropping weapons outside) | no change | Scope text. |
| 11, the animation starts at the fire point and is attached there (S-0301 claim 22) | tested | Unchanged rule, tested at M7b: `claim_1_a_shot_starts_at_the_fire_point` (`claims_s0313.rs`), the claim 22 tests of `claims_s0301_shotanim.rs`, `live_tests::shot_animation_follows_its_firer`. The P10 tests draw the animation at `fire_point`. |
| 11, drawn as S-0109 claim 26: centred (S-0102 claim 19) on the projection of its position with our lift, moved down by v; a tiled image by S-0109 claim 27 | tested | `p10_marker_at_the_projected_fire_point` (odd canvas 21 × 15, v = −7), `tests::slot_animation_draw_shift_and_shadow`, `tests::slot_animation_tiled_copies`. |
| 11, no facing-dependent correction moves the fire point or the animation (without a barrel model in both engines; with one, our engine's rule) | tested | `p10_marker_at_the_projected_fire_point` (32 turret steps, both counters, three muzzle heights, two levels), `p10_every_body_facing` (H40, 32 body facings at both levels), `p10_barrel_model_moves_nothing`. |
| 11, our driving locomotor adds no draw shift (our design) | tested | No locomotor of ours shifts the draw point (`Builder::mobile`); `run` checks the parts at the unshifted draw point in every P10 test. |
| 11, the turret drawn from the draw point by S-0105 claims 4 and 8 with F2's δ and 32-step turret and body facings | tested (changed) | δ: `turret_drawn_with_the_fire_points_displacement` (the art read and the simulation's value differ; with the typed rules the simulation's is drawn). Facings: `turret_drawn_with_the_fire_points_facings` (S-0105's 32-step facing equals S-0204 F4's with n = 32 for every raw facing), and both read the facings at the same tick. |
| 11, the original's turret drawing and its driving locomotor's shift | not implemented (original only) | Open in the spec; our engine needs no answer. |
| 11, backing and confidence | no change | Ratings only. |
| 11a, the animation's draw point counted from the unshifted draw point: the fire point's projection minus the vehicle's, each truncated on its own with its own lift, then v and the centring; not the displacement's projection alone | tested | `p10_marker_at_the_projected_fire_point` (exact at every step; at least one sampled step separates the two projections, so the test would fail a displacement-only rule), `p10_moves_between_muzzle_heights`. |
| 11a, Q-1600 question 2 partly settled; a constant one-pixel shift open | no change | About the original. Our engine adds no further displacement (S-0109 claim 26). |
| 11a, backing and confidence (moving vehicle, other levels) | no change | Ratings only. Our rule has no movement or height term; the P10 tests keep the vehicle still at levels 0 and 6. |
| 11b, the original's lift as the hypothesis Λ = S-0409 F2's lift | not implemented (original only) | A hypothesis about the original; our engine keeps S-0101 claim 3's lift (claim 11e). Nothing to build. |
| 11c, the distance between the drawn turret and the animation is set by the data; our engine corrects no mismatch | tested | No code adds a correction: the turret is placed as the parts check of `run` expects and the marker exactly where claim 11a puts it. The figures Q-1600 reports for our engine (about 5 pixels further along, about 2 lower) stand; the spec neither confirms nor explains them. |
| 11c, the original's distance | not implemented (original only) | Open in the spec. |
| 11d, a separate barrel model: our barrel by S-0105 claim 10 (departure); the fire point and our animation unchanged by the barrel | tested | `p10_barrel_model_moves_nothing` (fire points and marker positions equal with and without a barrel model, every step and counter, both levels); the barrel's own placement is S-0105's, unchanged. |
| 11d, the original's barrel pixels and any barrel correction | not implemented (original only) | Open in the spec. |
| 11e, our lift against the original's: check values *L*(664) − *L*(624) = 6, *L*(729) − *L*(624) = 15, *L*(40) = 6 | tested (our half) | `p10_moves_between_muzzle_heights` (our lift's values, and the moves 6, 15 and 9 at both levels); the comparison with the original rests on claim 11b and needs no code. |
| P10 | not implemented (measurable) | Oracle scenario; no M-record. Its engine run is `shot_anim_tests.rs`: the marker exact at every turret step and counter (`p10_marker_at_the_projected_fire_point`), every body facing in H40 (`p10_every_body_facing`), the muzzle variant and the mirrored odd counter with the turret's pixels unchanged (`p10_muzzle_variant_and_mirroring`), the moves between muzzle heights and the H0 residual difference of 0 between levels (`p10_moves_between_muzzle_heights`), the body drawn from the unshifted draw point (`run`), the barrel repeat (`p10_barrel_model_moves_nothing`). The test reads the painter's items instead of screen captures, so the marker is one centre pixel rather than a cross whose arms reach past the hull. |
| Open questions (seven new: the original's locomotor shift, its lift, its barrel and barrel correction, the turret-to-animation distance, the constant one-pixel shift, its turret correction, other locomotors) | no change | All about the original; the code needs no answer. Our engine adds no draw shift for any locomotor. |

Requests: Q-1600 answered by S-0313 revision 29; it had no interim (the code followed the specs).
Its notes are retired in `docs/development.md` (the visual review's shot-animation rows) and in
[../m7-claims/step1j-m7b-projectiles.md](../m7-claims/step1j-m7b-projectiles.md). None filed.

Seams: none new. The lane trace switches of `crates/app/tests/m7a_lane.rs`
(`GLASNOST_TRACE_SHOTS`, `GLASNOST_TRACE_ANIMS`, `GLASNOST_TRACE_BOX`) stay as diagnostics.
