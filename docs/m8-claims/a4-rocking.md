# M8 A4 rocking: claim ledger

This step builds body rocking, which seam sweep 2 found missing ([seams-2.md](seams-2.md)) and
which [intake26-S-0504.md](intake26-S-0504.md) named as a seam. Specs as read: S-0303 r25,
S-0614 r12, S-0601 r16, S-0210 r22, S-0106 r12, S-0504 r26.

**Split.** The simulation keeps no rocking state. It reports what rocks to the drawing through
`World.effects`, which is not saved or hashed: `Effect::Rock` (S-0303 claim 20),
`Effect::DroneRock` (S-0614 claim 9), `Effect::GrapplePush` (S-0614 claim 10) and
`Effect::OverloadRock` (S-0601 claim 16, F4). The drone, squid and overload draws were already
taken; they are now reported rather than dropped. The view keeps the angles
(`mapview::rocking::Rocking`) and moves them once per game tick. S-0303 claim 20 believes rocking
does not reach game state, and S-0504 claim 16 allows the angles to be presentation state.

**Interim rules (Q-2081, open).** No approved spec says how a strength becomes rates, how a rocked
body returns to level, how the weight scales it, or the sideways limit (S-0504 Open questions).
Ours, marked `design: Q-2081` in `rocking.rs`:
- a push sets both rates to 0.05 rad/tick × strength ÷ weight, split by cos and sin of the push's
  facing relative to the body's;
- each tick an angle adds its rate, stops at its limit with the rate level, and a damped spring
  (period 18 ticks, ζ 0.2) pulls the rate back;
- only vehicles rock (an aircraft is drawn by its flight attitude), weighted by the type's
  imported weight (1.0 when absent or not positive);
- the sideways limit is π/4;
- a centre-cell object with no source is pushed from *P*;
- the drone's +*s* side is the host's right;
- a squid's push is the roll, added to the rest.

| claim | status | test names or reason |
|---|---|---|
| S-0303 claim 20, a rocking warhead whose strength min(*D* × 0.01, 4.0) exceeds 0.3 (*D* ≥ 31) | tested | `claim_20_rocks_from_31_damage_with_the_flag` (30 no, 31 yes, a warhead without the flag no); `the_strength_of_a_damage_amount` (app: the cap from 400). |
| S-0303 claim 20, the 7 × 7 cells around the centre cell in the detonation's cell layer | tested | `claim_20_reaches_the_seven_by_seven_cells` (3 cells out yes, 4 no). The deck-layer (bridge) case is not covered by a test. |
| S-0303 claim 20, the centre cell from the source object, the rest from *P*; other cells only when *S* > 0 | tested | `claim_20_pushes_the_centre_from_the_source_and_the_rest_from_p` (no source: from *P*, design Q-2081 question 5), `claim_20_without_spread_rocks_only_the_centre_cell`. |
| S-0303 claim 20, the rates set, scaled by the type's weight | partly | `a_blast_lifts_the_side_it_comes_from` (ahead, behind, left and right), `a_heavier_hull_rocks_less` (absent = 1.0, 4 rocks less, 0 = 1.0). The rule is the Q-2081 interim. |
| S-0303 claim 20, no draw and no game state | tested | `claim_20_rocking_takes_no_draw_and_changes_no_state` (random state and state hash equal with and without the flag). |
| S-0303 claim 15, a nullified detonation has no rocking | tested | `claim_20_a_nullified_detonation_does_not_rock`. |
| S-0614 claim 9 and F2, a drone delivery to a host that is not infantry rocks it with strength 1.5 to the side its draw chose | tested | `claim_9_each_delivery_reports_its_rocking_side`, `claim_9_an_infantry_host_is_not_rocked` (sim); `a_drone_rocks_its_host_sideways` (app; the + side is design Q-2081 question 6). |
| S-0614 claim 10, a squid's push tilts sin(*k* × ±π/10) × π/4 at frame *k* (10 frames × 4 ticks, 0 at the strike) while the host has its parasite | tested | `claim_10_each_push_reports_its_way` (sim); `a_squid_rocks_its_host_through_each_push` (app; both ways, every frame, ended once no parasite holds the host). The axis and the way's sense are design Q-2081 question 7. |
| S-0601 claim 16 and F4, a damaging pulse at level *i* > 0 on a surviving controller sets its sideways rate (±0.015 at level 1, ±0.03 from level 2, negative below 50) | tested | `claim_16_each_damaging_pulse_reports_its_rocking_draw` (sim: the draw reported, no further draw); `an_overload_pulse_sets_the_sideways_rate` (app: the four cases, exact single values, unscaled, the pitch rate kept). |
| S-0210 claim 12, forward limit π/10 while crush-slowed, π/4 otherwise; at the limit the angle stays and the rate becomes level; the sideways limit unchanged | tested | `the_angles_stop_at_their_limits`, `a_crush_slowed_vehicle_pitches_at_most_a_tenth_of_pi`. The sideways value π/4 is design Q-2081 question 4. |
| S-0210 Terms, the angle built up by a rate per tick | tested | `the_angles_do_not_depend_on_the_frame_rate` (redraws between ticks move nothing). |
| S-0106 claim 9 and Terms (a pitch or roll of 0.005 or more rocks; drawn with pitch and roll) | tested | `a_rocking_vehicle_is_drawn_rocking`, `drawn_angles_are_held_to_the_rocking_step` (drawn angles truncated to multiples of 0.005, never −0; a rock draws fewer poses than ticks). The tilt of *p* in the transform is not built (slope tilt *T*, seam). |
| S-0105 claim 26 and S-0106 claim 13, a rocking body drawn fresh | tested | `a_rocking_vehicle_has_images_of_its_own_and_keeps_its_shadow` (own images, settled when its locomotor draws no rocking, level again after). Design: a rocking body that does not sink keeps its shadow. |
| S-0504 claim 15, *Starting value* (the lean starts from the pitch left over from rocking) | tested | `a_sinking_vessel_leans_from_its_rocked_pitch_and_rolls_freely`: the view seeds its lean from the rocked pitch and steps it by `naval::pitch_step` in the simulation's direction. |
| S-0504 claim 15, while sinking the forward rate is not added and the pitch does not return to level; the roll is free | tested | The same test: a blast from the side during descent rolls the hull, the pitch rate stays 0 and the lean keeps its steps. |
| S-0504 claim 16 and F5, drawn transform for the driving, ship, hovering and jumpjet locomotors | tested | `a_rocking_vehicle_is_drawn_rocking` (walking: kept, not drawn). Hovering and jumpjet take the driving order (S-0504 Open questions; design). |
| S-0504 claim 16, *Passengers* (fire point on the leaned body) | seam | S-0313 claim 5's fire point is built from the heading alone, so neither the lean nor the roll moves it. Not built. |
| S-0504 Open questions (a rocked body's return to level) | request | Q-2081 question 2; interim above. |
| S-0514 claim 20, `directrocker` | no change | Selection only; no behaviour spec. |

**Seams named here.** S-0313 claim 5's passenger fire point does not take the body transform
(S-0504 claim 16); slope tilt *T* is not in F5 (S-0106 claim 9); direct rocking (S-0514 claim 20)
is selection only; the deck-layer case of S-0303 claim 20 has no test.

**Requests.** Q-2081 (open): strength to rates, the return to level, weight, the sideways limit,
a centre cell without a source, the drone's side, the squid's push, and what reads rocking.
Q-2082 to Q-2085 unused.

**Simulation.** Effects only. No state, hash, draw, save format or AI format change; goldens
unchanged; seed sweeps m6c, m7a and m7b 1 to 16 identical before and after.
