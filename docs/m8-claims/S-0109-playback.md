# S-0109 claim ledger (animation playback in the simulation)

Revision 32 of the spec, built at M8 step A5. The drawing's side (claims 19 to 21, 24, 26, 27 and
the client's use of the clock) stays in [docs/m6-claims/S-0109-presentation.md](../m6-claims/S-0109-presentation.md);
this ledger is the simulation's playback. Tests (`crates/deterministic/sim/tests/`):
`claims_s0109_playback.rs` (first update and lifetime, update-order entry, the trailer's loop delay,
the start and middle events, loop-delay draws, damage over time with the quirk both ways, the
follow-up's accumulator restart and reverse request, the repeat factor, attachment and owner
removal, the removal skip with `quirks.update_order_removal_skip` both ways, a save round trip),
`claims_s0522.rs` (`claim13_parachute_attached_until_landing`,
`claim13_a_short_parachute_ends_before_landing`). The clock itself (`glasnost_sim::anim_clock`) keeps
the unit tests of `glasnost_app::mapview::anims` (claims 1 to 12, 15, 16) from M6b.

Code: `glasnost_sim::anims` (`AnimWorld` in `World::anims`, `spawn`, `turn`, `remove`, the
playback environment `AnimEnv` with the damage, middle and start events), `anim_clock` (`Creation`:
initial loop delay, repeat factor, reverse request; `Env::delay_ended`), `tick::object_turn` (an
animation's turn), `load::anim_created` (every creation not played by a specialised model goes
through `anims::spawn`; tile animations of the first pass are dropped before the second, S-0010),
`combat::effects` (`Effect::Anim::id`, `Effect::ShotAnim::id`, `trailer`), `combat::shotanim`
(attached to a firer that is not a building), `combat::projectile` (the trailer), `drain` (the
drain animation attached to the drainer, removed at the link's end), `superweapons::warp` (the
source marker, removed at a new source pick and at launch), `paradrop` and `fall` (the parachute,
S-0522 claim 13). State: `World::anims` (save format 69). Client: `app::mapview::live`
(`LiveAnim::sim`, `follow_sim_anims`).

| claim | status | test names or reason |
|---|---|---|
| 1 | tested | `claims5_16_first_update_and_lifetime` (frames 0 to 3 through `anims::shown`); the clock's stage rules in the M6b ledger. |
| 2 | tested | Every fixture's rate 900 gives *d* = 1 (F1); `anim_delay` keeps its M6b tests. |
| 3 | partly | The creation draw is the scenario stream's (`load::creation_draws`, S-0202 claim 10 tests) and its value reaches the clock (`Spawned::delay`). A follow-up type's random rate draws from the scenario stream in the animation's turn (`AnimEnv::draw`), coded, not tested here. |
| 4 | partly | `mutation::playback` normalises the delay under `quirks.speed_normalised_delays` (S-0611 tests); no test of an ordinary normalised animation at two speeds. |
| 5 | tested | `claims5_16_first_update_and_lifetime` (no advance in tick *c*, first advance in *c* + 1), `claims11_13_29_trailer_loop_delay_defers_the_start_event` (no count-down at the first update). |
| 6 | tested | `claims11_28_loop_delay_draw_from_the_scenario_stream`, `claim7_repeat_factor_multiplies_the_passes` (each pass ends on its *E*-th advance). |
| 7 | tested | `claim7_repeat_factor_multiplies_the_passes` (*L* = 1, *m* = 3: three passes); endless through `claim13_parachute_attached_until_landing` (*L* = −1). |
| 8 | tested | `claim8_15_reverse_request_is_kept_at_a_follow_up` (a creator's reverse request: *k*₀ = λₑ − 1, then down). |
| 9 | partly | Ping-pong is the shared clock's (M6b tests); no simulated fixture. |
| 10 | partly | Shadow ends are the shared clock's (M6b tests); no simulated fixture. |
| 11 | tested | `claims11_13_29_trailer_loop_delay_defers_the_start_event` (a creator's delay: start event on update *D* + 1, no advance, the clock not restarted), `claims11_28_loop_delay_draw_from_the_scenario_stream` (between passes: one ranged draw, *D* updates without advance). |
| 12 | tested | Every fixture starts at frame 0 (`claims5_16_first_update_and_lifetime`). |
| 13 | tested | `claim13_start_event_at_creation_with_mu_zero`, `claim14_middle_event_on_the_advance_to_mu` (μ ≠ 0: no draw at creation); the start event after a loop delay in `claims11_13_29_trailer_loop_delay_defers_the_start_event`; after a follow-up switch coded (`AnimEnv::switched`). |
| 14 | tested | `claim14_middle_event_on_the_advance_to_mu` (one ranged draw over 0 … 2147483646 on the advance to μ, none after). Bouncing types make no middle event on an advance (coded); their motion and landing are S-0319's (`bounce`, built in A3 intake 15 part 1, ledger [intake15a-S-0319.md](intake15a-S-0319.md)). The order against the same advance's damage: interim rule of Q-1716. |
| 15 | tested | `claim15_follow_up_restarts_the_accumulator_at_zero` (the type switch, A = 0.0), `claim8_15_reverse_request_is_kept_at_a_follow_up`. |
| 16 | tested | `claims5_16_first_update_and_lifetime` (lifetime *E* × *d*, removed in its own turn, out of the update order). Infantry creation at the end stays with S-0611's model (`mutation`). |
| 17 | tested | `claim17_damage_over_time_pattern` (δ = 0.375: 1, 0, 1, 0, 0, 1, 0, 1), `claim17_damage_accumulator_quirk_off`, `claim15_follow_up_restarts_the_accumulator_at_zero`. The area detonation allows resource reduction with no attacker or house (`area::detonate`, coded). The factor 5 for an animation attached to a terrain object is coded (no creator attaches one yet). The warhead: `FlameDamage2` for every type, interim rule of Q-1717. A trigger-played animation: seam (below). |
| 18 | partly | Follows from claims 4, 11, 14, 15 and 17 as built; no test at two game speeds. |
| 19, 20, 21, 24, 26, 27 | not in the simulation | Drawing rules (M6b ledger). |
| 22 | tested | `claim22_owner_removed_deletes_at_the_next_update` (owner position plus displacement; deleted at the next update with nothing else). Detachment: no creator detaches. The render layer is the drawing's. |
| 23 | seam | Held animations are building slot animations, which the simulation does not play (section *Map viewer (M2)*: drawn at their first frame). |
| 25 | tested | The importer's art lookup gives μ and the frame count (M6b and S-0202 intake tests); the simulation reads `AnimRules::mu` and `frames`. |
| 28 | tested | `claims11_28_loop_delay_draw_from_the_scenario_stream` (the draw in the animation's own turn, from `World::rng`), `state_survives_a_save_round_trip`; visibility plays no part (the simulation has no view). |
| 29 | tested | `claim29_appended_to_the_update_order`, `claims11_13_29_trailer_loop_delay_defers_the_start_event` (the trailer's loop delay of 1), `claim29_removal_in_own_turn_costs_the_next_animation_a_turn` (S-0203 claim 10 with the quirk both ways). Shot animations are attached to a firer that is not a building (`shotanim`), deploy presentations carry the reverse request (`effects::anim_reversible`); coded. |

S-0312, claim 18 (survivors of a destroyed building):

| claim | status | test names or reason |
|---|---|---|
| S-0312 18 | seam | Not this step. `combat::death::survivors` and `second_round` keep *C* = 0: F4 and F5 need a house's side as an index (0, 1 or 2), and `SimRules` carries country side names only for the tech tree (`econ::tech`), with no mapping from a house to F4's side. The refund (`seams::building_refund`) and game-over test (`seams::game_over`) exist. Building it also changes vehicle crews (`crew_type` stops always giving the technician), so it needs its own arena run. |

Seams of this step:
- **Specialised models keep their own playback.** Infantry-making animations (S-0611, `mutation`), virus-death clouds (S-0613, `gas`), burning walkers (S-0615, `walker`), domination, depot slot animations and storm clouds (S-0513, `superweapons::storm`, played by the storm model with `load::anim_created(…, true)`) are not entered twice.
- **Bouncing motion.** Built in A3 intake 15 part 1 (S-0319, `bounce`; ledger [intake15a-S-0319.md](intake15a-S-0319.md)): a bouncing type now flies, lands and ends there. Its stages still play without damage or advance middle events (claim 14, claim 17).
- **Trigger-played animations.** Map triggers that play animations (S-0011 claim 36) are not built, so no animation is trigger-played.
- **A warp source marker's end on leaving aiming** is a sidebar action the simulation does not see; the marker ends at a new pick or at launch.
- **Rocket trailers** (`rocket.rs`) are made as ordinary animations without the trailer's loop delay: a rocket is not a projectile of S-0306.
- **Resource glints** have no position in any spec, so they deliver no damage and take only their particle draws at a middle event (S-0202 claim 14).
- **Quirk switches of the clock.** The simulation plays with `AnimQuirks::default()` (every switch on), as `mutation` did; the switches are not read from the rules' quirks.
- **The drawing.** Drain and warp-marker pictures in the client are still its own cosmetic animations; tile animations and glints are not drawn by the client.
