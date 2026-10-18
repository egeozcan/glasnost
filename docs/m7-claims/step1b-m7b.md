# M7b step 1b claim ledger (S-0616 revision 8 and its companions)

This is M7b step 1b (intake 1). It covers the approved revisions that reached main between a10d1c0 and 78261d8: S-0616 revision 8, S-0306 revision 14, S-0312 revision 33, S-0301 revision 24, S-0314 revision 30, S-0402 revision 32, S-0405 revision 19, S-0203 revision 38 and S-0302 revision 30, the index and the answered requests Q-1400 to Q-1404. Every changed claim gets one class, as in [step1r-m7a.md](step1r-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered and no quirk switch is new (`every_spec_quirk_has_a_switch` passes). Claim 20's spark-turns fix is a lead decision with no switch. The new tests are in `crates/deterministic/sim/tests/claims_s0616_emitters.rs` (now 29 tests, 10 new), `claims_s0402.rs` (1), `claims_s0402_slaves.rs` (1) and `claims_s0525_hits.rs` (1), with unit tests in `combat::railgun` and the importer tests `game_wide_combat_values` (extended) and `spark_turns_of_zero_or_less_warn` (new). `claims_s0614.rs` `claim_9_delivery_spark_system` now counts the system made, since the FIZZ system of its test rules (no spark-turns key) leaves in its first turn under claim 20.

| Spec | Revision | Answers |
|---|---|---|
| S-0616 particle systems | 8 (from 7) | Q-1400, Q-1401, Q-1402, Q-1403, Q-1404 |
| S-0306 projectiles | 14 (from 13) | none (new claim 27, the railgun shot) |
| S-0312 death and destruction | 33 (from 32) | Q-1400 (through S-0616) |
| S-0301 firing cycle | 24 (from 23) | Q-1403 question 2 (through S-0616) |
| S-0314 healing and repair | 30 (from 29) | none (the healing marks of damage smoke) |
| S-0402 harvest cycle | 32 (from 31) | Q-1401 (through S-0616) |
| S-0405 power | 19 (from 18) | Q-1404 (the deactivated state) |
| S-0203 tick order | 38 (from 37) | none |
| S-0302 damage resolution | 30 (from 29) | none |
| `spec/README.md` | | index rows revised: (c) |
| `spec/requests/` Q-1400 to Q-1404 | | `status: answered`: (c) |

**Retired interims** (all five, from M7b step 3.1):
- Q-1400 (destruction smoke). Smoke-kind entries only: kept. "Attached" was any of the four emitter places held; now only an earlier destruction smoke of the same building counts, so nothing stops it (changed). The owner object is the building, as before; a building removed at once marks it before its first turn, an exploding or sold one lets it emit until its removal (as before, now tested).
- Q-1401 (refinery smoke). Once per payment row: kept, now before the payment, plus one event for the empty-load row that starts *finishing* (changed). Slave delivery: one event per delivery after its payments, only when one paid a positive amount (changed: before, one event per delivery whatever was paid).
- Q-1402 (railgun beam end). The projectile's impact point is gone: the target point is F14's end point, computed at the shot (changed, `combat::railgun`).
- Q-1403 (creator details). 1: the barrel system's owner house is none (changed). 2: every flag makes its own emitter in the order fire, spark, railgun (changed). 3: the flame sweep uses the owner's current burst counter (changed, it was 0). 4: only *now yellow* multiplies the natural smoke's *T*, and it compounds (changed: *now red* counted before, and the factor was applied once). 5: kept.
- Q-1404 (sparks in the turn). Damage sparks moved before the radiation hit, after behaviour processing, overload and self-healing; deactivated sparks moved to the very start of the turn (changed, `tick::object_turn`).

**New request:** Q-1405 (F14's last sample under round-toward-zero, the railgun line distance and degenerate beams), with its interim in `combat::railgun` and `docs/development.md`. Block Q-1406 to Q-1419 is unused.

**Fixes on main checked against the revisions:** b2add06 (infantry firing before the completion follow-up, S-0107) agrees: S-0616 claim 1 puts the damage sparks before the separate infantry firing, and `tick::object_turn` still calls `damage_sparks` before the radiation hit and the infantry firing. fcc5627 (letter case of weapon projectile and warhead names, S-0004) touches no claim of these revisions.

## S-0616 revision 8

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, requests), scope sentences | (c) | |
| Terms: *deactivated vehicle* (only S-0405 claim 13's disabling; a pulse, paralysis or no driver are other states) | satisfied | `deactivated_sparks` reads only `power::unit_disabled`. Test: `claim_1_deactivated_sparks_first_in_the_turn`. |
| Terms: *resource-carrying slave*; the list of claims that mark systems | (c) | |
| 1, exploding overlay row: owner house none | changed | `emitters::barrel_particle_system` passes no owner house. Test: `claim_1_barrel_particle_system`. |
| 1, refinery smoke: smoke events (each payment row before its payment, the empty-load row; one per slave delivery after its payments, only when one paid a positive amount; a vehicle master makes none; a building need not be a refinery) | changed | `econ::harvest` (payment rows and the empty-load row), `econ::slaves` (`paid` flag). Tests: `claim_11_refinery_smoke_events` (3, 2 and 1 events for two, one and no resources, each before its payment), `claim_14_refinery_smoke_per_delivery`. |
| 1, destruction smoke (smoke kind only; attached means an earlier destruction smoke only; no owner house; marked at removal: at once for a building leaving at once, until removal for an exploding or sold one) | changed | `emitters::destruction_smoke`. Tests: `claim_1_destruction_smoke` (damage smoke and natural smoke no longer stop it; one entry draws R(0, 0)), `claim_1_destruction_smoke_until_removal`. |
| 1, natural smoke: only *now yellow* multiplies *T* by 1.5; *now red* does not; it compounds | changed | `emitters::natural_smoke_yellow` from `combat::damage`. Tests: `claim_1_natural_smoke` (6.0 then 9.0), `claim_1_natural_smoke_now_yellow_hits_only`. |
| 1, effect weapons: the flags are independent, one emitter per flag with an empty place, order fire, spark, railgun | changed | `emitters::effect_weapon_emitter`. Test: `claim_1_several_flags_make_several_emitters`. |
| 1, effect weapons: railgun target point is F14's end point, fixed at the shot; the railgun shot's own effect computed with it when the place is empty | changed | `combat::railgun::shot`. Tests: `claim_1_spark_and_railgun_emitters`, the S-0306 claim 27 tests below. |
| 1, damage smoke: the healing marks (own self-healing; house-wide healing of a non-organic vehicle; strictly above yellow or below −10 leptons) | changed | `combat::heal::smoke_heal_mark`. Test: `claim_1_self_healing_marks_damage_smoke` (vehicle, infantry, organic vehicle, house-wide and own healing, the below-surface test; the place stays held). |
| 1, damage sparks: place in the turn; none in a tunnel or off the map | changed (place, off the map); seam (tunnel) | `tick::object_turn` calls `damage_sparks` after `process::turn` and before `radiation::hit_turn` and the separate infantry firing; `damage_sparks` returns at once when the object is not on the map. Test: `damage_sparks_come_before_the_radiation_hit` in `claims_s0525_hits.rs` (fails with the old order). Tunnel travel is not built (S-0215 claim 2's tunnel turn has no code yet), so the tunnel case waits for it. |
| 1, deactivated sparks: place in the turn (first, also in turns that end early); *z* unchanged; truncating division | changed (place); satisfied (the rest) | called first in the living branch of `tick::object_turn`, before every early return of that branch (warp, overload kill, sinking, fall, erasure). An electromagnetic pulse, paralysis and tunnel travel make no early return in our engine yet (not built); when they are built, their returns must come after this call. Test: `claim_1_deactivated_sparks_first_in_the_turn`; the cadence and draws: `claim_1_deactivated_vehicle_sparks`. |
| 1, backing and confidence; scenarios P20 to P23 | (c) | their checks are the tests named in this table. |
| 7, 8 (successor translucency wording), 9 to 19 backings | (c) | |
| 11, F5: the sweep uses the owner's current burst counter | changed | `particles` fire sweep. Test: `claim_11_sweep_uses_the_current_burst`. |
| 15, 20: a spark-turns field of 0 or less (including the absent key) is marked in the first turn, no burst and no draw; the importer warns | changed | `particles::spark_turn`; convert warning. Tests: `claim_20_spark_turns_zero_frees_the_place`, `spark_turns_of_zero_or_less_warn`. |
| 22 and the state tables (healing marks, the spark-turns mark, the no-kind row) | changed / satisfied | as claims 1 and 20; the no-kind row is S-0613 `claim_17_kinds_and_no_kind`. |
| F14: the railgun end point | changed | `combat::railgun` (`step`, `fraction`, `sample`, `walk`). Unit tests `step_is_the_single_nearest_one_49th`, `ends_are_exact`; `s0306_claim_27_terrain_stop`. *t*₄₉ pinned to 1.0: interim Q-1405. |
| Constants (cadence division truncated; railgun samples and step; the two depth limits) | changed | cited at their uses. |
| Import mapping (`SparkSpawnFrames` warning; destruction and weapon-flag keys owned elsewhere) | changed / (c) | the warning in `crates/convert/src/sim.rs`. |
| Open questions (railgun shot's own effect owned by no spec) | (c) | S-0306 revision 14 claim 27 owns it now; S-0616 claim 1 and its Open questions still say no approved spec does (cross-spec lag for the lead, not a contradiction of behaviour). |

## S-0306 revision 14

| Claim | Class | Notes |
|---|---|---|
| Front matter, scope | (c) | |
| 27: objects hit (newly entered cells, the deck layer by height, the kinds, the firer excluded, no alliance test, buildings at distance 0, the target always and last when not met, each once) | changed | `combat::railgun::shot`. Test: `s0306_claim_27_railgun_hits_along_the_beam`. |
| 27: line distance in floating point with the table root, strictly below the hit distance | changed | `railgun::line_distance`, interim Q-1405 for the exact form and degenerate beams. |
| 27: damage (one ordinary hit of the raw weapon damage, no bonuses, not an area detonation) | changed | as above. |
| 27: terrain (a stop cancels every hit; one draw R(0, 99) only for a destroyable-cliff stop or cliff-cell target; the collapse when strictly below the chance; draws before the emitter) | changed (draws); seam (the collapse) | Tests: `s0306_claim_27_terrain_stop`, `s0306_claim_27_cliff_cell_target`. The collapse itself is the seam `railgun::cliff_collapse` (no approved spec gives what a collapse does). |
| Import mapping (`IsRailgun`, `Damage`, `railgundamageradius` 128, `CollapseChance` 100) | changed | `rules::General::railgun_damage_radius`, `collapse_chance`; `game_wide_combat_values`. |
| P41, Open questions | (c) | |

## S-0312 revision 33

| Claim | Class | Notes |
|---|---|---|
| 19 (f): smoke kind only, the attachment condition, no draw with none eligible, owner object the building, no owner house, the marks at removal | changed | as S-0616 claim 1's destruction smoke row. |
| 19 confidence, backing boundaries, D14 f, Open questions | (c) | |

## S-0301 revision 24

| Claim | Class | Notes |
|---|---|---|
| 5: any held effect of any flag of the asked weapon (rearming) or of its other weapon (impossible); rank 4 wins; one emitter per flag; the next shot waits for every place | changed | `emitters::own_effect_alive`, `other_effect_alive` (`places_of`, `any_held`). Tests: `claim_1_several_flags_make_several_emitters`, `s0301_claim_5_live_effect_gate`. Sonic effects: seam (not modelled, S-0607's note). |
| 5 backing; F4 multi-flag part | (c) | |

## S-0314 revision 30

| Claim | Class | Notes |
|---|---|---|
| 9: after the own self-healing gain, the damage-smoke mark test; the system stays in its place | changed | `heal::smoke_heal_mark`. Test: `claim_1_self_healing_marks_damage_smoke`. |
| 10: after house-wide healing, only a non-organic vehicle takes the test | changed | as claim 9. |
| Backings, H7 b | (c) | |

## S-0402 revision 32

| Claim | Class | Notes |
|---|---|---|
| 11: refinery smoke events (payment rows and the empty-load row, before the payment) | changed | Test: `claim_11_refinery_smoke_events`. |
| 14: one event on the master per delivery, after its payments, only when one paid something; an undeployed master makes none | changed | Test: `claim_14_refinery_smoke_per_delivery`. |
| Backings and confidence | (c) | |

## S-0405 revision 19

| Claim | Class | Notes |
|---|---|---|
| 13: this disabling is the only way into the deactivated state | satisfied | Test: `claim_1_deactivated_sparks_first_in_the_turn` (new). |
| Backing and confidence | (c) | |

## S-0203 revision 38 and S-0302 revision 30

| Claim | Class | Notes |
|---|---|---|
| S-0203: S-0616 claim 1 owns the place of both spark steps in the turn | (c) | the order is S-0616's row above. |
| S-0302 claim 12: the hit-driven marks; S-0616 owns the healing marks | (c) | |

## Format and gate

- The save format goes from 59 to 60: a particle system no longer carries the natural smoke's `yellowed` flag (the factor now compounds), and the rules gain `railgun_damage_radius` and `collapse_chance`. The AI game format is unchanged.
- Goldens: `duel-m6c` and `duel-m7a` were regenerated; their hashes differ from tick 0 because their worlds hold particle systems whose saved form lost the flag. The other six goldens did not change.
- Seed sweep over seeds 1 to 16: the same counters as at steps 1o to 1r and 3.1, to the last digit (minimum / median / maximum: berserk orders 14/24/34, units made berserk 37/45/63, sensor orders 2/2/2, intent ticks 2 275/4 367/5 500, reveal orders and reveals fired 10/11/12, slave-miner buildings 3/4/13, slave ticks 1 980/6 505/8 640, gattling stage rises 0/14/28, top-stage ticks 0/2 870/4 108, takeover orders 2/24/33, takeovers 8/18/24, gas clouds 0/152/301, gas hits 0/611/1 502, absorber entry orders 9/18/46, infantry absorbed 4/9/15, grinder entry orders 4/14/19, units ground 4/12/16, bunker build orders 4/4/4, bunker entry orders 2/7/11, vehicles bunkered 2/5/6, bunker exits 2/5/6, lifter build orders 5/5/7, lift orders 5/11/23, vehicles seized 4/12/25, seized units dropped 1/9/20), with the same early ends (seeds 3, 6, 7, 9, 11, 12, 13 and 16) and shortfalls. Default play shows no change: every counter and every early-end tick is as before. Seed 2 still plays all 12 000 ticks with every counter at least 3 except the sensor orders' known 2, so `M7A_SEED` stays 2. After the regeneration, `cargo test --release -p glasnost-sim-gate -- --include-ignored` passes without the update flag.
