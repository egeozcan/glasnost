# M7a step 1g claim ledger (S-0605 revision 2, S-0613 revision 6, S-0301 revision 21, S-0302 revision 28, S-0601 revision 13, S-0614, S-0004 revision 18, S-0202 revision 30)

This is part 7 of M7a step 1. Approved revisions that landed on main after 58ba06c (`git diff 58ba06c 5b0120d -- spec/`). Every changed claim gets one class, as in [step1f-m7a.md](step1f-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason (a claim for a part not built yet);
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

| Spec | Revision | Answers |
|---|---|---|
| S-0605 infantry absorber | 2 (from 1) | Q-1260 to Q-1267 |
| S-0613 gas clouds | 6 (from 5) | none |
| S-0301 firing cycle | 21 (from 20) | none |
| S-0302 damage resolution | 28 (from 27) | none |
| S-0601 mind control | 13 (from 12) | none |
| S-0614 parasites | 9 (date and wording only) | none |
| S-0004 type registration order | 18 (from 17) | none |
| S-0202 random streams | 30 (from 29) | none |
| `spec/README.md` | | index rows moved and bumped: (c) |

**S-0616 is not in `spec/`.** S-0613, S-0301, S-0302, S-0601, S-0614, S-0004 and S-0202 now point at S-0616 (particle systems of the other kinds) for the smoke, fire, spark and railgun kinds, the owner-removal mark, the empty-named type's origin and the non-gas draws. No S-0616 claim is cited in code. The rows that need S-0616 behaviour are *not built (S-0616 absent)*, filed as Q-1153, with the current behaviour as the interim. Non-gas systems stay inert apart from their turns and lifetime (Q-1243).

**Retired interims.** Q-1260 to Q-1267 are answered by S-0605 revision 2 (the lead already set them to `answered`). Each interim is gone from the code, the tests, [S-0605.md](S-0605.md), [S-0601.md](S-0601.md) and the absorber section of [development.md](../development.md), and the answer is built (rows below). The "M7a step ledger" table row 3.5 in `development.md` still names them; that table is the lead's.

**New requests** (block Q-1152 to Q-1159): Q-1152 (the nearby-location search of claim 4's approach), Q-1153 (S-0616 referenced but absent), Q-1154 (S-0215 says no rule of our engine gives an enter-nearest instruction; S-0605 claim 16 gives one, and the two differ on "already bound"). Q-1155 to Q-1159 unused.

**Quirk switches.** Three new ones, all default on (the original), wired and tested both ways: `absorber_stale_contact_kept` (claim 17), `absorber_removal_keeps_stale_power` (claim 15), `enter_nearest_survives_absorption` (claims 13 and 16). The schema is regenerated.

**Formats.** Save format 50 to 51 (the new hashed field `StructState::enter_nearest`, and changed absorber behaviour). AI game format 19 to 20: the AI absorber module sends a stalled entrant again (a unit of its own that holds an absorber's contact without *enter* current or queued, claim 17, gets the *enter* click on that building again, once per cell it stands on; `glasnost_ai::absorber::stalled`, test `a_stalled_entrant_is_sent_again`). Without it the gate sweep's infantry-absorbed minimum fell to 2 (seeds 11, 13 and 14). Goldens regenerated.

## S-0605 revision 2

Claims 1 to 15 keep their numbers; 16 and 17 are new. No claim number was re-mapped.

| Claim | Class | Notes |
|---|---|---|
| Header, Summary, Terms (*absorber list*, *enter-nearest instruction*), backing and confidence lines | (c) | The absorber list is the house's holding order, `house.econ.acquired`, filtered to absorbing buildings (pushed on entering the map, moved to the end on an owner change). |
| 1: rows x and q test capacity above 0, row I13 tests capacity equal to 0 (answers Q-1265) | changed | `absorb::carries_no_passengers`, read by `targeting` row I13. `claim_1_i13_capacity_zero_and_negative`. |
| 2 | satisfied | `claim_2_counts_passengers_not_sizes`. |
| 3: the three entry points (order, fate, retry); a building holding any contact puts the actor in line; contact *not granted* (alliance missing either way, health 0, no place): no wait, the building kept as destination; a refused loading gives the contact up, destination kept (answers Q-1261, Q-1266) | changed | `absorb::seek_contact`. `claim_3_4_waiting_in_line`, `claim_3_switched_off_refuses_loading`, `claim_3_5_6_not_granted`. |
| 4: the approach phase (a free location near the centre cell, *move* started at once, waiting kept); the retry queues *enter* with an immediate start; a refusal queues the none behaviour (no start) and clears the destination; nothing while phasing in (answers Q-1261) | changed | `absorb::approach`, `absorb::waiting_turn`. The search's settings and tie-breaks are open in the spec: Q-1152 interim (S-0212 claim 25 from the centre cell with S-0507 claim 10's vehicle-exit settings). `claim_4_approach_and_retry`, `claim_4_refused_after_waiting`, `claim_4_wait_ends`. |
| 5: a refusal at an *enter* processing gives up the contact and falls back, destination and walk kept; without contact and not waiting the processing falls back at once (answers Q-1266) | changed | `absorb::process_enter`. `claim_5_switched_off_while_approaching`, `claim_3_5_6_not_granted`. |
| 6: entry needs contact; the tag raise is unconditional | changed | `absorb::arrival` (no contact: nothing after the raise). `claim_3_5_6_not_granted` (raised, not inside), `claim_6_entry_raise`, `claim_6_no_recheck_overfills`, `claim_6_recheck_off_turns_away`. |
| 7 to 11 | satisfied | Unchanged behaviour; tests as in [S-0605.md](S-0605.md). |
| 12: the deck wording at the eighth direction (answers Q-1262) | satisfied | The revision's wording equals the old interim. `claim_12_deck_at_the_eighth_direction_both_positions`. |
| 13: a voluntary release clears the enter-nearest instruction for a human-controlled owner only | changed | `absorb::let_out`. `claim_16_back_in_after_release_both_positions` (neutral owner: kept). |
| 14: destruction places an infantry passenger by the sale's centre placement, one draw per open cell (answers Q-1264) | changed | `absorb::place_released` (one path for destruction and sale). `claim_14_destruction_draws_as_the_sale`, `claim_14_destruction_releases_front_first`. |
| 15: removal inside makes no power due (answers Q-1263) | changed | `absorb::removing`, `quirks.absorber_removal_keeps_stale_power`. `claim_15_removal_inside_both_positions`. The same-house case is open in the spec; the quirk's default makes no power due either way. |
| 16 (new): the bio-reactor fate: candidates in absorber-list order, claim 1's test, ρ₃ to the reference point, ties to the later building in the list, the instruction, none found clears it; the repeat in *guard* and *area guard* (answers Q-1260) | changed | `absorb::bio_reactor_fate`, `absorb::instruction_turn` (from `process_guard` and `process_area_guard`, *D*<sub>r</sub>, no draw), `StructState::enter_nearest`, `quirks.enter_nearest_survives_absorption`. Q-1154 interim for the S-0215 conflict. `bio_reactor_fate_sends_the_released_unit_in`, `claim_16_tie_goes_to_the_later_building`, `claim_16_none_found_and_held_contact`, `claim_16_back_in_after_release_both_positions`, `a_released_neutral_unit_takes_the_bio_reactor_fate`. |
| 17 (new): what ends a contact; leaving *enter* does not (answers Q-1267) | changed | `quirks.absorber_stale_contact_kept` (on: `contact_outlived` does not run). `claim_17_stale_contact_both_positions`. The *enter* processing takes no approach step in our engine (`approach::step` runs only from *attack*, *hunt* and *area guard*). Which rule takes the holder out of *enter* in the gate duel was not traced; the AI now re-sends such a unit (Formats above). |
| Formulas, Constants, State and ordering | (c) | No new number. |
| Edge cases (the three new quirk switches) | changed | As claims 15, 16 and 17. |
| Import mapping | satisfied | No new key. |
| Measurement scenarios (A1b, A2c, A13 (f), A14 and the rest) | (c) | |
| Open questions (the nearby search's tie-breaks; the same-house power case) | (c) | Q-1152 for the first. |

## S-0613 revision 6

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| Terms, *Particle system* and *Empty-named type* (new): a section read without `HoldsWhat` names the empty-named type; a never-read system type has none | changed | Importer `particle_system` (`crates/convert/src/sim.rs`): with the key absent from a read section, `holds_what` is the empty-named type's key, which `layers.rs` already registers by reference (S-0004 claim 13). `absent_holds_what_in_a_read_section_names_the_empty_type`; `particle_system_fields_and_shared_gas_system` (`<none>` and the never-read shared system: none). |
| 4: such a system gets a first particle, of the empty-named type; non-gas emission is S-0616's | changed (first particle) / not built (S-0616 absent) | A no-kind or gas system makes the first particle (`gas::warhead_system`); the smoke, fire, spark and railgun kinds take its creation draws and do not keep it (Q-1243). |
| 17: owner removal marks a system to end | not built (S-0616 absent) | No system has an owner object in our engine. The gas-kind case is `unmeasured` in the spec. Q-1153. |
| State table row (owner object removed) | not built (S-0616 absent) | As claim 17. |
| Edge cases (non-gas particles in the shared system: their kind's update rules, the lifetime count-down) | not built (S-0616 absent) | No data path puts one there; Q-1243. |
| Import mapping (`HoldsWhat` default) | changed | As Terms. |
| Scenarios G3 and G13 (b), Open questions, Questions for the project lead | (c) | |

## S-0301 revision 21

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| Terms, *Effect weapon* (a particle emitter is alive until let go; the fire emitter's let-go events) | not built (S-0616 absent) | Live effects are not built (`combat::fire::arrival_delay` docs); the effect weapon takes its bare rate. Q-1153. |
| 5: an effect weapon fires again at the first attempt at least *R*′ after the shot and after the effect ended | not built (S-0616 absent) | As before: bare rate, no draw (`combat::fire`). |
| 18: creating an effect weapon's particle system is not a draw | satisfied | No shot code takes a particle-system draw (`combat::fire`, `combat::projectile`). |
| Scenarios F4, F14, backing and confidence | (c) | |

## S-0302 revision 28

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 10 (berserk hits: no damage effect or marking of one) | (c) | Wording follows claim 13. |
| 13: a hit leaving the ratio above the yellow ratio *marks* the damage effect, kept until its system leaves play | not built (S-0616 absent) | `combat::damage::damage_effect_reaction` still lets the effect go at once (Q-1153 interim cited there). |
| Backing, confidence, scenario D27 | (c) | |

## S-0601 revision 13

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 16: the ten placement draws come before the rocking draw; creating a spark system takes no draw | satisfied | `mind::overload_turn` takes the placement draws only (the systems are a seam). Existing overload tests in `claims_s0601.rs`. |
| Scenario text | (c) | |

## S-0614 (revision 9, re-approved)

| Claim | Class | Notes |
|---|---|---|
| `approved-on` | (c) | |
| 9: creating the spark effect takes no draw | satisfied | `parasite.rs` takes no draw for it (the effect is a seam). |
| Open questions | (c) | |

## S-0004 revision 18

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 13: a particle-system section read without the emitted-particle key registers the empty-named particle type by reference, at the end of the list | satisfied | `layers.rs` (`How::EmptyParticle`). `absent_holds_what_in_a_read_section_names_the_empty_type` shows the type at the end of the particle list. |

## S-0202 revision 30

| Claim | Class | Notes |
|---|---|---|
| Header | (c) | |
| 15 and Open questions: S-0616 owns the non-gas update, successor, emission and burst draws | pointer | S-0616 (absent; Q-1153). |
