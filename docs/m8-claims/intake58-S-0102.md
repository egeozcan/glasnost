# Intake 58: S-0102 revision 32 (walk counter, claim 22a; answers Q-2176)

Last built revision: 24 (round 6 and earlier ledgers). The r24 to r32 diff touches the walk-counter Term, claim 22 (frame table and Walk counter bullet), new claim 22a, the Import mapping rows (`WalkRate`, `IdleRate`, `HoverAttack`, `DeployToLand`), scenario D48 and Open questions. Nothing else changed. No new quirk switch (the spec adds none; its two departures are design rules with no switch). The counter is not simulation state (claim 22a *Saved games*), so no format change and no simulation behaviour change; the only simulation-crate edit is a read-only wrapper, `occupancy::locomotor_moving_now`.

Code: `crates/app/src/mapview/walk.rs`, `live.rs` (rates), `build.rs` (`sprite_vehicle_frame`).

| Claim | Status | Where / reason |
|---|---|---|
| Term *walk counter* (per vehicle, 0 at creation, voxel and sprite) | Tested | `Counters::advance`; `a_vehicles_walk_counter_runs_while_it_moves` |
| 22 frame table: walking row tests *movement in progress*, not moving now | Tested | `Walk::in_progress`, `a_sprite_vehicle_walks_through_its_row_while_it_has_movement_in_progress` (waiting vehicle holds the walking row) |
| 22a advance (a): walk-rate multiple, moving now | Tested | `moving_steps_on_multiples_of_the_walk_rate`, `moving_ignores_the_idle_rate` |
| 22a (a) hover-attack alternative (target, not deployed, `HoverAttack`) | Tested at the rule; wiring partly | `a_hover_attack_vehicle_holding_a_target_steps_at_the_walk_rate`; `situation()` reads target, S-0517 deployed, flag. No live scenario with a hover-attack type |
| 22a (b) idle rate while not moving now | Tested | `standing_steps_only_with_an_idle_rate` |
| 22a (c) deploy-to-land above surface, every tick | Tested at the rule | `a_deploy_to_land_vehicle_in_the_air_steps_every_tick`; height from `jump::height_above` |
| 22a Moving now per locomotor (S-0207 c23, S-0504 c2, S-0503, S-0502 c20) | Partly | `occupancy::locomotor_moving_now` dispatches hover, jumpjet, drive/ship; walking, teleporting and tunnelling vehicles fall back to S-0107 is-moving-now (design, marked). Live test covers the driving case only |
| 22a No advance: off map (transport, carrier lift), falling, sinking, erased, phasing in, seized, warp member | Tested for off map; others wired | `held()`; the live test checks off-map. Others read sim accessors, untested |
| 22a No advance: melee-linked without keep-acting, travelling through a tunnel | Seam | the simulation keeps no answer the view can read; marked `// seam:` in `held()` |
| 22a Rates: absolute value, walk rate 0 used as 1 | Tested | `rates_use_their_absolute_value_and_a_zero_walk_rate_is_one` |
| 22a Leaving and entering play (value kept through a transport) | Tested | counters of held vehicles are kept, not dropped (before: dropped when off map) |
| 22a Saved games / reading time / at most 1 per tick | Tested by construction | view state, restarted at 0 on load or join; advanced once per game tick from the state the tick left |
| Import mapping `HoverAttack`, `DeployToLand` | Tested | `Rates` defaults; read in `live.rs` |
| D48 | Not run | oracle scenario, not a client test; D48 (a) and (d) are mirrored by the rule tests above |

Earlier rows updated: `a4-play-notes.md` note 39 (Q-2176 answered, interim retired); `development.md` Simulation section sentence on the walk counter. Q-2176 answered by S-0102 r32, built. No requests filed (Q-2201 to Q-2205 unused).
