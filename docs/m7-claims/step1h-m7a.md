# M7a step 1h claim ledger (S-0616 revision 4, S-0601 revision 16, S-0613 revision 8, S-0614 revision 11, S-0215 revision 60, S-0312 revision 31, S-0314 revision 24, S-0405 revision 17, S-0202 revision 31, S-0011 revision 25, S-0214 revision 40, S-0516 revision 10, glossary)

This is part 8 of M7a step 1. It covers the approved revisions that reached main with the step 1g merge 850767f and were not yet taken in (`git diff a9da344 850767f -- spec/`). Every changed claim gets one class, as in [step1g-m7a.md](step1g-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

| Spec | Revision | Answers |
|---|---|---|
| S-0616 particle systems (smoke, fire, spark, railgun) | 4 (new) | Q-1153, Q-1243 |
| S-0601 mind control | 16 (from 13) | none |
| S-0613 gas clouds | 8 (from 6) | none |
| S-0614 parasites | 11 (from 9) | none |
| S-0215 behaviours | 60 (from 59) | Q-1154 |
| S-0312 death and destruction | 31 (from 30) | none |
| S-0314 healing and repair | 24 (from 23) | none |
| S-0405 power | 17 (from 16) | none |
| S-0202 random streams | 31 (from 30) | none |
| S-0011 map triggers | 25 (from 24) | none |
| S-0214 order targeting | 40 (from 39) | none |
| S-0516 cloak and stealth | 10 (from 9) | none |
| `spec/glossary.md` | 51 (from 45) | new terms *capacity (transport)*, *particle speed*, *dead mark*, *electric-bolt shot*, *marked (particle system)*, *owner object*, *target point*: (c) |
| `spec/README.md` | | index rows: (c) |

**S-0616 is built where built specs need it.** The four kinds' turns, updates, moves, successors, emission, bursts and lines are built in `glasnost_sim::particles` for the creators that already exist: the warhead's system (S-0303 claim 20), the damage smoke (S-0302 claim 13), the overload sparks (S-0601 claim 16) and the drone delivery sparks (S-0614 claim 9). Every other creator of S-0616 claim 1 is a named seam. The full ledger is [S-0616.md](S-0616.md).

**Retired interims.** Q-1153 (S-0616 missing) and Q-1243 (the particles of the other kinds are not kept) are answered by S-0616. Q-1154 (S-0215 against S-0605 claim 16 on the enter-nearest instruction) is answered by S-0215 revision 60. Their interims are gone from the code (`gas.rs`, `damage.rs`, `process.rs`, `structures/absorb.rs`), the tests, [S-0613.md](S-0613.md), [S-0605.md](S-0605.md) and [development.md](../development.md). The lead's "M7a step ledger" table in `development.md` is unchanged.

**New requests** (block Q-1155 to Q-1159): Q-1155 (successors of a particle whose kind differs from its system's kind), Q-1156 (what triggers the depot walk of S-0314 claim 23 in our engine, where a queued depot vehicle never has *enter* current). Q-1157 to Q-1159 are unused. Q-1152 stays open: S-0314 revision 24 also leaves the nearby-location search to a lead decision.

**Quirk switches.** Five new ones, all default on (the original): `fire_particle_no_house`, `smoke_ignores_particle_cap`, `spark_double_gravity`, `smoke_deck_extra_update`, each tested both ways, and `flame_sweep_facing`, a seam (the effect-weapon emitter). Its default is tested, the importer and loader read it in both positions, and it changes nothing until the seam is filled. All five load from rules in both positions (`particle_quirks_load_in_both_positions`). `glasnost_rules` `every_spec_quirk_has_a_switch` passes. The schema is regenerated.

**Formats.** Save format 51 to 52 (the particles of the other kinds are now kept, with the speed, direction, velocity, step, emission timer, spark counter, target point, owner object and the own stage-advance value; an object's damage smoke is a system; the depot's `walked` mark). The AI game format is unchanged (21 after merging main with S-0603): the AI makes no new decision. The gate figures and the M7a seed are in the gas section of [development.md](../development.md).

## S-0616 revision 4 (new)

See [S-0616.md](S-0616.md) for the claim-by-claim ledger. In summary:

| Claim | Class | Notes |
|---|---|---|
| 1, rows of the warhead, damage smoke (position, healing mark), overload and drone delivery | changed | `gas::warhead_system`, `particles::damage_smoke`, `particles::spark_system`. `claim_1_warhead_systems_keep_their_first_particle`, `damage_effect_draw_and_removal`, `claim_16_pulse_timing_and_draws`, `claim_9_delivery_spark_system`. |
| 1, the other rows | seam | `effect_weapon_emitter` (fire, spark and railgun emitters, let-go events, S-0301 claim 5's gate, F5 to F7), `barrel_particle_system`, `destruction_smoke`, `refinery_smoke`, `natural_smoke`, `electric_bolt_sparks`, `damage_sparks`, `deactivated_sparks`, `voxel_animation_system`, the trigger actions (S-0011 codes 88 and 89). |
| 2, 3 | changed | `claim_3_particle_kind_chooses_the_update`; Q-1155 interim for successors across kinds. |
| 4 | changed | `objects::remove` calls `particles::owner_removed`. `claim_4_and_5_owner_object`. The target object is part of `effect_weapon_emitter`. |
| 5 to 9, F2 to F4 (smoke) | changed | `claim_4_and_5_owner_object`, `claim_8_smoke_successors`, `claim_9_timer_and_cut_off`, `smoke_ignores_particle_cap_both_positions`, `smoke_deck_extra_update_both_positions`. No object is inside a tunnel, so claim 9's tunnel test always passes. |
| 10, 12 to 14, F8, F9 (fire) | changed | `claim_12_fire_burns_out`, `fire_particle_no_house_both_positions`. |
| 11, F5 to F7 | seam `effect_weapon_emitter` | `flame_sweep_facing_changes_nothing_yet`. |
| 15, 16, F10, F11 (spark) | changed | `spark_burst_and_double_gravity_both_positions`, `claim_15_spark_without_held_type`. |
| 17, 18, F12 (railgun) | changed | `claim_17_railgun_line`. |
| 19, F13 | changed | `particles::own_advance`. `claim_19_normalised_stage_advance`. |
| 20 | changed | `particle_system_kind_warnings`, `claim_15_spark_without_held_type`. |
| 21 | satisfied | Step 1g (S-0613 revision 6). |
| 22 | changed | As claims 4, 9, 10, 15 and 17, with S-0613 claim 17's departure. |
| Edge cases (five quirks) | changed / seam | As Quirk switches above. |
| Constants, State and ordering, Import mapping | changed | `particle_system_kind_fields`, `default_spark_system_absent_warns`; save format 52. |
| Scenarios, Open questions | (c) | |

## S-0601 revisions 14 to 16

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 8: the bio-reactor fate names S-0605 claim 16 | (c) | Built at step 1g. |
| 16: with the spark type key present, five spark systems and their placement draws; absent, neither | changed | `mind::overload_turn` (spark systems through `particles::spark_system`; with `default_spark_given` off no placement draws). `claim_16_pulse_timing_and_draws`, `claim_16_no_spark_type_no_spark_draws`. |
| Import mapping (`DefaultSparkSystem`: no default, importer warns; the same key for S-0616 and S-0614) | changed | `convert::sim_combat::gas_checks`. `default_spark_system_absent_warns`. |
| Scenario setup text | (c) | |

## S-0613 revisions 7 and 8

| Claim | Class | Notes |
|---|---|---|
| Header, Summary | (c) | |
| 1: the shared system's position is S-0616's engine constant, read only by S-0616's trigger action | changed (constant) / seam (action) | `gas::create_shared` places it at (2688, 2688, 0), S-0616 Constants. The trigger action is a seam (S-0011 code 89). |
| 4: the first particle of the other kinds is kept (Q-1243 answered) | changed | `claim_4_warhead_system_and_first_particle`, `claim_1_warhead_systems_keep_their_first_particle`. |
| 7: the successor rule is the gas kind's | changed | `gas.rs` (only a dead gas particle splits in the gas turn's second pass). Q-1155 for a dead particle of another kind in a gas turn. `claim_7_successor`. |
| 17: owner removal and the trigger action mark a system | changed (owner removal) / seam (action) | `claim_4_and_5_owner_object` (every kind, gas included). |
| Edge cases (`wind_southeast_short` is gas-only) | satisfied | Smoke uses its own table (`particles::SMOKE_WIND`, `smoke_wind_direction_three_is_full`). |
| Scenarios, backing | (c) | |

## S-0614 revisions 10 and 11

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 9: the delivery makes one system of the game-wide spark type, no draw | changed | `parasite.rs` calls `particles::spark_system`. `claim_9_delivery_spark_system`. |
| Import mapping, Open questions | (c) | |

## S-0215 revision 60

| Claim | Class | Notes |
|---|---|---|
| Terms (enter-nearest instruction: the bio-reactor fate gives the absorbing-building instruction in our engine) | changed | Q-1154 interim retired; citations in `process.rs` and `structures/absorb.rs` now point at S-0215 claims 24 and 26 row 3. |
| 24: the instruction's repeat; already bound for *enter* means in contact or waiting in line | changed | `absorb::instruction_turn` (waiting in line counts as bound). `claim_16_waiting_in_line_is_already_bound` (scenario B21 (d)). |
| 26, row 3 and the rate table | satisfied | `absorb::instruction_turn` from *area guard*, *D*<sub>r</sub>, no draw (step 1g tests). |
| Scenario B21 (d), backing | (c) | |

## S-0312 revision 31

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 24: an infantry passenger's desired point and its one draw over 0 to 3 | satisfied | The placement already in use (`absorb::place_released`, S-0208 table O). `claim_14_destruction_draws_as_the_sale`. |
| Import mapping (`Cyborg` also the damage-sparks flag) | seam `damage_sparks` | |
| Scenarios | (c) | |

## S-0314 revision 24

| Claim | Class | Notes |
|---|---|---|
| Header, Terms (*locomotion class*) | (c) | |
| 23: a queued depot vehicle with no contact walks to a free cell near the depot, the queued entry kept | changed | `depot::queued_walk` with the nearby search of Q-1152's interim (`absorb::approach`). Q-1156 interim: in our engine the queued vehicle never has *enter* current, so it walks once per queued entry in its first queued-entry turn whose retry finds no place (option A). `claim_23_queued_walk`. |
| State table row, scenarios, Open questions | (c) | |

## S-0405 revision 17

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| Due events: removal of a passenger of another house inside an absorber does not make power due | satisfied | `quirks.absorber_removal_keeps_stale_power` (S-0605 claim 15). `claim_15_removal_inside_both_positions`. |
| Open questions | (c) | |

## S-0202 revision 31

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 15: the creation draws apply to every creator of a particle | satisfied | Every creator goes through `gas::add_particle`, which takes the table's draws (S-0616 tests compare the stream). |
| Scenario R14 A, backing | (c) | |

## S-0011 revision 25

| Claim | Class | Notes |
|---|---|---|
| Header, Terms | (c) | |
| Actions 88 and 89: outside I4's subset, parsed by I2 | satisfied / seam | Parsed as any action; their behaviour is a seam (S-0616 claim 1 trigger actions). |

## S-0214 revision 40

| Claim | Class | Notes |
|---|---|---|
| Header, backing | (c) | |
| Rows x and q: a building's passenger count above 0, whatever its kind | changed | `targeting::passengers` reads the count of any building type. `s0214_i13_any_building_count`, `claim_1_i13_capacity_zero_and_negative`. |
| Row I13: for a building, a count equal to 0 | changed | `targeting::building_passengers`. Same tests. The non-absorbing building case is weakly shown: the later rows give select as well. |
| Import mapping (`Passengers` for building types) | satisfied | Already imported (`absorber_fields`). |

## S-0516 revision 10

| Claim | Class | Notes |
|---|---|---|
| Header, Terms (*natural smoke system*) | (c) | |
| Cloaking state changes: the natural smoke system removed at full cloak, a new one at full uncloak | seam `natural_smoke` | Building cloaking and natural smoke are not built. |
| Scenario 18 | (c) | |

## Merge of main (S-0603 grinder)

main (cfde12f) was merged into this branch before the report, at the coordinator's request. It brings S-0603 (AI game format 21, the HULK fixture, the grinder counters) and a later batch of spec revisions (S-0616 revision 5 and most other specs one revision on). That batch is not taken in here; it is for the next intake. The save format stays 52. The goldens were regenerated after the merge, and the gate figures are in the particle-systems section of [development.md](../development.md).
