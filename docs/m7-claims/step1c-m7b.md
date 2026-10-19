# M7b step 1c claim ledger (the revisions after intake 1)

This is M7b step 1c. It covers the approved revisions that reached main after step 1b's intake, up to eb1cc74: S-0615 revision 3, S-0109 revision 31, S-0205 revision 10, S-0107 revision 17, S-0616 revisions 9 and 10, S-0306 revision 15, S-0613 revision 11, S-0303 revision 18, S-0514 revision 9, S-0502 revision 18, S-0309 revision 21, S-0405 revision 20, the measurement notes f81fc29 and 0fa06d8, and the answered requests Q-1405, Q-1420 to Q-1427 and Q-1500 to Q-1504. Every changed claim gets one class, as in [step1b-m7b.md](step1b-m7b.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

**New quirk switches** (`crates/deterministic/rules/src/quirks.rs`, schema regenerated; `every_spec_quirk_has_a_switch` passes): `disc_laser_unlimited_range` (S-0615 claim 14), `railgun_collinear_hits` (S-0306 claim 27), `jumpjet_falling_touchdown_tracked` (S-0502 claim 29). Each is tested in both positions.

**Format.** Save format 64 (from 62): S-0405 revision 20 drops the building switches' `last_cell`, since disabling now follows the cell-arrival event (63 on this branch); S-0611, merged from main at the same time, had also gone to 63 with other state, so the merge takes 64. AI game format 24 unchanged. All nine goldens regenerated. `M7A_SEED` moved from 2 to 1: Alpha now wins seed 2 at tick 11 331; seed 1 is the lowest seed that plays all 12 000 ticks with every counter at least 3 but the sensor orders' known 2 (sweep of seeds 1 to 16 in `crates/sim-gate/src/lib.rs`).

| Spec | Revision | Answers |
|---|---|---|
| S-0615 special-hit effects | 3 (from 2) | Q-1420, Q-1421, Q-1422, Q-1423, Q-1424, Q-1426, Q-1427 |
| S-0109 animation playback | 31 (from 30) | Q-1425 |
| S-0205 number domains | 10 (from 9) | none (a quirk-table row for S-0616 F14) |
| S-0107 infantry sprite sequences | 17 (from 16) | none (Q-1421's ordering, through S-0615) |
| S-0616 particle systems | 9 and 10 (from 8) | Q-1405 (revision 10) |
| S-0306 projectiles | 15 (from 14) | Q-1405 |
| S-0613 gas clouds | 11 (from 10) | none (cross-reference to S-0615 claim 1) |
| S-0303 area damage | 18 (from 17) | Q-1420 (through S-0615) |
| S-0514 chrono weapons | 9 (from 8) | Q-1420 (through S-0615) |
| S-0502 jumpjet movement | 18 (from 17) | Q-1500, Q-1501 |
| S-0309 target acquisition | 21 (from 20) | Q-1502, Q-1504 |
| S-0405 power | 20 (from 19) | Q-1503 |
| M-0001, M-0002 (f81fc29) | | the cliff cells large groups reach: listed only |
| M-0001 to M-0003 (0fa06d8) | | the arena houses were computer players: `crates/tools/src/oracle.rs` already plays both houses as computer houses (`oracle::Ground::computer`, true for the records, with the hard computer row of S-0412 claim 4), so it agrees; nothing changed |
| `spec/README.md`, `spec/requests/` | | index rows and `status: answered`: (c) |

**Retired interims** (in code comments, `docs/development.md` and the ledgers):
- Q-1405 (railgun). *t*₄₉ is no longer pinned to 1.0: it follows the rounding mode (changed). The line distance is S-0306 claim 27's mixed-precision formula (changed). A distance that is not a number qualifies under the new quirk (changed).
- Q-1420 (disc detonation, area damage only). Kept as built; now cited to S-0615 claim 16, S-0303 claim 23 and S-0514 claim 20 (satisfied).
- Q-1421 (the virus effect and the parachutist's explosion). The virus effect first was kept; a not-human ground death with kind 8 now makes its die-1 request, then exactly one virus effect (changed).
- Q-1422 (disc positions and loss). Positions, not fire or target points; a cell target at terrain height without the deck (raised by 416 for the final point); a loss is permanent from the moment it happens (changed).
- Q-1423 (negative death kind as kind 0). Kept; the importer now warns (changed, import).
- Q-1424 (walker fall start). The descent now runs in the update that starts the fall, from rate 0 (changed).
- Q-1425 (walker damage and resources). Kept as built (resources allowed, no attacker, no house); a new test (satisfied).
- Q-1426 (unlimited disc range). The interim became the default of `quirks.disc_laser_unlimited_range`; off never cancels on distance (changed).
- Q-1427 (walker with no frame count). N = 0, removed at the first removal test; the importer warns (changed).
- Q-1500, Q-1501 (jumpjet registration and in the air). Replaced by S-0502 claims 29 to 31 (changed).
- Q-1502 (search regions). Two no-region cells compare as one region; the deck read; the no-link exception (changed).
- Q-1503 (disabled vehicles). S-0405 claims 13 and 20 (changed).
- Q-1504 (hunting units and hovering jumpjets). Already as built (satisfied).

**New requests:** Q-1505 (disabling a vehicle docked on a repair depot; interim: passed over when our depot exchange tethers it, so a pad depot yes and an ordinary depot no). Q-1506 (the firing-check answer of a disabled vehicle) was written and then withdrawn before merging, because S-0301 revision 26 in the same batch answers it (see the second batch below). Q-1506 to Q-1519 are unused.

## S-0615 revision 3

| Claim | Class | Notes |
|---|---|---|
| Front matter, backing, scenarios DK1 to DK3 and DL1 to DL7 | (c) | |
| 1, order with the extra virus effect (Q-1421) | changed | `combat::death` (the parachutist's virus effect before its kind 3 or 5 animation; a not-human ground death with kind 8 makes die-1, then one virus effect). Tests: `a_not_human_virus_death_makes_one_virus_effect` and `a_gas_cloud_kills_a_parachutist_with_one_cloud_and_an_explosion` (`pairs_m7b.rs`, now asserting the order PLAGUE before BOOM). |
| 3, negative kinds (Q-1423) | changed (import) / satisfied | `death_kind_effect` treats any kind outside the table as none; the importer warns about a negative `InfDeath`. Test: `kinds_without_animation` (DK-1). |
| 10, first descent and starting rate (Q-1424) | changed | `walker` runs the descent in the update that starts the fall, from rate 0. Test: `lost_deck_fall_and_splash` (revision 3 assertions). |
| 11, unknown frame count and the removal test (Q-1427) | changed | `walker::removal_test`, also at the end of the update in which a first-update choice of none starts expiry; N = 0 for a type without a known frame count; importer warning. Tests: `walker_boxed_in_expires_at_once`, `expiry_lifetime_and_damage`. |
| 14, points measured, the loss rule, unlimited and other negative ranges (Q-1422, Q-1426) | changed | `disc` (positions; a cell target at terrain height; the vertical difference 0 for an airborne firer; a permanent loss; `quirks.disc_laser_unlimited_range`). Tests: `recheck_distance_forms`, `unlimited_range_both_positions`, `other_negative_ranges_cancel`, `loss_between_steps_is_permanent`, `cell_target_over_a_bridge_deck`. |
| 16, area damage only; a cell target's final point (Q-1420, Q-1422) | satisfied / changed | the detonation applies S-0303 claims 1 to 20 only (as built); the final point of a cell target is raised by 416 over a deck (changed). Test: `cell_target_over_a_bridge_deck`. |
| Import mapping (warnings for a negative death kind and an unknown walker frame count) | changed | `crates/convert/src/sim.rs`, `sim_combat.rs`. |
| Row 3 of S-0107 claim 17 (a type's own death-animation list) | seam | `death_anims_row` in `combat::death`, unchanged. |

## S-0109 revision 31

| Claim | Class | Notes |
|---|---|---|
| 17, each delivery is an area detonation with resource reduction allowed, no attacker or house; the inputs delegated to S-0303 claim 17 (Q-1425) | satisfied | `walker::advance_damage` (`resources: true`, no source, no house), re-cited. New test: `s0109_claim_17_walker_damage_takes_bales` (`claims_s0401.rs`: an expiring walker on a chain-reaction ore cell takes bales; without the flag it does not). |
| Backing, scenario AN27 | (c) | |

## S-0205 revision 10

| Claim | Class | Notes |
|---|---|---|
| Quirk table: `chop_rounding` row for S-0616 claim 1 (railgun beam sampling) | satisfied | `railgun::fraction` and `sample` run in the program's mode. Tests: `last_sample_follows_the_mode`, `third_parameter_differs_by_mode`, `p22_part_d_samples` (unit tests in `combat::railgun`). |

## S-0107 revision 17

| Claim | Class | Notes |
|---|---|---|
| Terms: *cow-designated type*; claim 14 and Import mapping reworded around the engine's fixed idle-designation text | (c) | the rule is unchanged: `infantry.rs` compares the registry name with `COW`, case-insensitively. |
| 17, row 4 with kind 8: the virus effect follows the die-1 request; a parachutist gets only the earlier one | changed | as S-0615 claim 1 above. |
| 17, an empty entry at an in-range death-kind index of a type's own list gives entry 0 | seam | `death_anims_row`. |
| Backing and scenario text (M9, part f, the cow scenarios) | (c) | |

## S-0616 revisions 9 and 10

| Claim | Class | Notes |
|---|---|---|
| 1, destruction smoke: a building awaiting removal because it was sold (revision 9) | satisfied | `combat::death` follows S-0312 claim 21 (explodes or being sold). Test: `claim_1_destruction_smoke_of_a_sold_building`. |
| 1, the railgun shot's own effect now owned by S-0306 claim 27; Open questions entry removed (revision 9) | (c) | as built at step 1b. |
| F14, δ fixed at 10956549 / 536870912; every *t*ᵢ in the program's mode, no special last sample; the no-stop end point the exact *Q* (revision 10, Q-1405) | changed | `combat::railgun` (`STEP_SIGNIFICAND`, `fraction`, `sample`, `walk`). Tests: `step_is_the_single_nearest_one_49th`, `last_sample_follows_the_mode`, `third_parameter_differs_by_mode`, `p22_part_d_samples`. |
| Scenario P21 part d, P22 parts b and d, backing | (c) | |

## S-0306 revision 15

| Claim | Class | Notes |
|---|---|---|
| 27, line distance and precision (Q-1405 questions 2 and 3) | changed | `railgun::line_distance` (RZ₂₄ narrowings, double intermediates, the table root on L only). Tests: `collinear_distance_is_not_a_number`, `diagonal_distance_is_close_to_euclidean`. |
| 27, degenerate comparisons; Edge cases (`quirks.railgun_collinear_hits`) | changed | `railgun::near_line`, `euclidean_line_distance`. Test: `s0306_claim_27_collinear_hits_both_positions` (`claims_s0616_emitters.rs`; an on-line candidate at hit distance 0 and below, a zero-length beam, both positions). |
| Backing and scenarios | (c) | |

## S-0613 revision 11, S-0303 revision 18, S-0514 revision 9

| Claim | Class | Notes |
|---|---|---|
| S-0613 claim 3: the extra virus effect's order is S-0615 claim 1's | satisfied | as S-0615 claim 1 above. |
| S-0303 claim 23: a disc laser's detonation applies claims 1 to 20 only, with no projectile-only effects or light flash | satisfied | `disc` (area damage only). Test: `final_step`. |
| S-0514 claim 20: special-hit selection is projectile-only | satisfied | as above. |
| Backings | (c) | |

## S-0502 revision 18

| Claim | Class | Notes |
|---|---|---|
| 29, registration as a tracked flying object (Q-1500) | changed | `jump::took_off` (registered in the take-off update, seized units included), `jump::fly::tick` (sector moves only in ascending and cruising updates), `fly::touchdown_registration` (`quirks.jumpjet_falling_touchdown_tracked`), the crash end before the impact, `jump::leaving` (going off the map), `jump::placed_registration` (placement in the air needs the counts-as-aircraft flag). Tests (`claims_s0303_jumpjet.rs`): `a_jumpjet_is_tracked_from_take_off_until_touchdown`, `a_crashing_jumpjet_stays_tracked_until_the_crash_ends`, `falling_touchdown_quirk_both_positions`, `a_cruising_jumpjet_changes_sectors`, `placement_in_the_air_needs_the_counts_as_aircraft_flag`; `a_seized_unit_is_tracked_and_in_the_air_by_height` (`claims_s0607_release.rs`). |
| 30, in the air for combat (Q-1501) | changed | `weapon::aircraft_in_air`: on the map and height at least 208, for any unit with the jumpjet locomotor. Test: `a_jumpjet_is_in_the_air_by_the_height_test`. |
| 31, a low jumpjet both registered and listed | satisfied | Test: `a_low_jumpjet_is_found_twice`. |
| Backing, scenarios | (c) | |
| Aircraft-type registration (S-0303's other movers) | seam | not modelled by this step; only jumpjet movers register. |

## S-0309 revision 21

| Claim | Class | Notes |
|---|---|---|
| 7 and table C row *zone*: cells of no reach region, the searcher's deck read, the no-link deck exception (Q-1502) | changed | `acquire::same_zone` (`regions::reach_read`). Tests: `cells_of_no_reach_region_compare_as_one_region` (`claims_s0309.rs`), `the_search_region_reads_the_searchers_deck` (`claims_s0212.rs`). |
| 11, the area-search cell test follows the same exception | satisfied | the area search applies the zone test per offered object through the same `same_zone`. |
| 7, hovering jumpjets: the paths that reach one (Q-1504) | satisfied | as built. Test: `a_map_wide_search_never_finds_a_hovering_jumpjet`. Retaliation (claim 18) has no layer test. A locomotor that sets the limited-scan mark (claim 22) is still a seam. |
| Backing, scenario A11 parts j and k | (c) | |

## S-0405 revision 20

| Claim | Class | Notes |
|---|---|---|
| 13, disabled on a completed cell arrival, not at placement or creation; off-map vehicles passed over; a tethered vehicle in its own cell passed over at the 1-to-0 transition (Q-1503) | changed | `power::arrival` from `cloak::machine::cell_movement_completed`; `power::set_disabled`; `power::tethered_in_own_cell`. The repair-depot dock is Q-1505 (interim). Test: `claim_13_tethered_vehicle_is_passed_over`. |
| 20, at disabling and enabling: the guard reset, deselection, locomotor, sounds | changed | `power::disable`, `power::enable`; `Effect::Deactivated`, `Sound::Deactivate`, `Sound::Activate`; audio `TypeSounds::deactivate` and `activate` (`DeactivateSound`, `ActivateSound`; coverage rows). Tests: `claim_20_what_a_disabled_vehicle_cannot_do`, `claim_20_enabling_resets_an_enabled_vehicle_and_skips_off_map`. |
| 20, what a disabled vehicle cannot do: move, fire, be box-selected, take behaviour, stop, scatter, deploy or guard orders | changed | `targeting::movable`, `box_eligible`; `combat::check` (illegal; placed by S-0301 revision 26, see below); `orders::refused_while_disabled` in `orders::issue`, which the own AI goes through too. Test: `claim_20_what_a_disabled_vehicle_cannot_do`. |
| 20, half brightness of a disabled voxel vehicle | changed | through S-0104 revision 26 (second batch below): `Mobile::disabled`, halved in `mapview::build` for a voxel body. |
| Backing, scenarios | (c) | |

# Second batch (after eb1cc74, up to 539869b)

The lead's batch after eb1cc74 and the S-0611 merge: the glossary (revision 58), S-0301 revision 26, S-0303 revision 19, S-0607 revision 10, S-0502 revisions 19 and 20, S-0309 revisions 22 and 23, S-0215 revision 69, S-0308 revision 15, S-0104 revision 26, S-0212 revision 51, S-0214 revision 42 and S-0608 revision 5 (with S-0611, which its own step took in). S-0208 revision 34 is not taken in (handed off; see the end). The other specs the coordinator named (S-0606, S-0507, S-0501, S-0603, S-0605, S-0409, S-0203) have no revision after eb1cc74 (`git log eb1cc74..main -- spec/`).

**Format.** Both this branch and S-0611 had moved the save format to 63 with different state, so the merge takes 64 and all nine goldens were regenerated. The test-only computer houses add no state: they reuse `House::human`, `House::skill` and the difficulty index. `M7A_SEED` stays 1 (the M7a gate and its sweep pass on the merged code).

**New quirk switch:** `flying_listed_double_hit` (S-0303 claim 6 and Edge cases), tested in both positions.

## Glossary revision 58, S-0309 revision 23, S-0215 revision 69: test-only computer-controlled player houses

A player house with `human` clear is a *test-only computer-controlled player house*. Only the oracle arena makes one (`oracle::Ground::computer`, both houses; normal games, `setup::skirmish` and our AI never do). Such a house takes the original's computer-owner rules everywhere the neutral and special houses do, with its oracle house's skill level and difficulty index and no teams.

| Claim | Class | Notes |
|---|---|---|
| S-0309 Terms (*Test-only exception*) and claim 25 parts a to f and h | changed | `combat::acquire::player_owned` is now "a player house with `human` set", so every claim 25 site (classes, table C rows, retaliation, the engineer rows) applies the computer-owner rules to a marked house. Tests (`claims_s0309.rs`): `test_house_deploy_to_fire_and_no_auto_fire_search` (part b), `test_house_retaliation_outside_the_listed_behaviours` (part h with S-0215 claim 18). |
| S-0309 claim 25 parts g and i (difficulty-indexed) | seam | the disguise draw (`cloak::disguise::neutral_sees_through`) and the destroy-walls entry are still seams; the auto-deploy delay of S-0215 claim 31 reads only index 1 (`GeneralRules::auto_deploy_delay`). A marked house's index 0 is set, but nothing reads it yet. |
| S-0215 claim 18 (retaliation outside guard, area guard and patrol; a held target; the distance limit; the threat-sum comparison) | changed / satisfied | through `player_owned` (changed); the threat-sum comparison was already built (`neutral_retaliation_compares_threat_sums`). The retaliate property still applies: `test_house_needs_the_retaliate_property`. |
| S-0215 claim 19 (scatter case 2's computer-controlled owner) | satisfied / seam | infantry acceptance (S-0208 claim 11) and the idle fraidycat scatter (S-0107 claim 14) already read `human` (`claim_14_fraidycat_idle_scatter` uses a player house with `human` clear). The damage scatter of vehicles (cases 1 and 2) is not built. |
| Glossary *computer-controlled*, *player house* (the general rule for every spec) | changed | the other computer-owner gates now read `human` instead of the role: `approach` (S-0210 claim 11), `depot::send_away` (S-0314 claim 6), `combat::check` (S-0513 claim 28), `lift::non_human_rule` (S-0607 claims 6 and 20), `econ::sell` (S-0314 claim 15), `econ::harvest` (S-0402 claims 12, 13 and 14), `transport` (S-0507 claim 7, S-0312 claims 12 and 25). Each is exact for every house but a marked one; the goldens do not move for it. |
| Glossary *house skill level*, S-0412 claims 3 and 4 | changed | the oracle gives both arena houses the rules' maximum skill level (`GeneralRules::iq_max`, `[IQ] MaxIQLevels`, absent 5, S-0209 Import mapping) and difficulty index 0. Arena only; no unit test (the oracle needs an install). |

## S-0301 revision 26

| Claim | Class | Notes |
|---|---|---|
| 9, *Deactivated firers within rank 4* (answers Q-1506 in all but name) | changed | `combat::check`: the deactivated *illegal* now comes after the own-gripped-target *rearming* and yields to the materialising firer's *rearming*; it still outranks every other rank-4 answer, the absent weapon included. Test: `claim_20_deactivated_illegal_outranks_absent_weapon` (`claims_s0405.rs`). Q-1506 withdrawn. |
| 9, firers being erased; the impossible row no longer lists the disabled firer | satisfied | erasure assigns no answer; ordinary firing already stops (S-0514). |
| 1 (claim 9's unchecked building secondary attempts), backing, scenarios, Open questions | (c) | |

## S-0303 revision 19

| Claim | Class | Notes |
|---|---|---|
| 6 and Edge cases, an object found both as a flying entry and in a scanned cell | changed | `combat::area`: with `quirks.flying_listed_double_hit` on (the original) it is hit twice; off, only its flying entry. Test: `flying_listed_double_hit_both_positions` (`claims_s0303_jumpjet.rs`). |

## S-0607 revision 10, S-0502 revisions 19 and 20

| Claim | Class | Notes |
|---|---|---|
| S-0607 claim 7, flying-object registration and the height test for a seized unit | satisfied | built at step 1c's first batch (S-0502 claims 29 and 30): `a_seized_unit_is_tracked_and_in_the_air_by_height`. |
| S-0607 claim 7 and S-0502 claim 29 (revision 20), a seize whose search finds no cell: grounded, no take-off registration | satisfied | Test: `a_seized_unit_with_no_cell_stays_grounded_and_untracked` (`claims_s0607_release.rs`). |
| S-0502 claim 32, disc-laser attacks do not force a descent (Q-1504 question 2) | satisfied | `disc` never touches height or layer. Test: `claim_32_disc_laser_attacks_do_not_force_a_descent` (`claims_s0502_land.rs`: a hovering jumpjet fires discs for 120 ticks, keeps hovering, its target height and stays out of the ground layer). |
| S-0502 scenario J30, backing | (c) | |

## S-0309 revision 22

| Claim | Class | Notes |
|---|---|---|
| 9 (airborne candidates' registration follows S-0502 claim 29) | satisfied | built at the first batch (`jump::took_off`). |
| Backing | (c) | |

## S-0104 revision 26

| Claim | Class | Notes |
|---|---|---|
| 13, *Deactivated voxel vehicles*: trunc(b / 2) ‰ before claim 15's row mapping | changed | `mapview::build` halves the brightness of a disabled vehicle with a voxel body; `Mobile::disabled` is read from `power::unit_disabled` in `mapview::live`. Flash and effect-tint adjustments are not modelled, so there is nothing to order them against. Test: `a_disabled_voxel_vehicle_is_drawn_at_half_brightness` (`live_tests.rs`). Sprite vehicles stay undimmed (open in S-0405). |

## S-0214 revision 42, S-0308 revision 15, S-0212 revision 51, S-0608 revision 5

| Claim | Class | Notes |
|---|---|---|
| S-0214 Terms, *movable* and *able to fire* exclude a deactivated vehicle | satisfied | built with S-0405 revision 20 (`targeting::movable`, `able_to_fire`; `claim_20_what_a_disabled_vehicle_cannot_do`). |
| S-0308 Terms (the airborne test for other objects is S-0307's) | (c) | |
| S-0212 claim 4's backing (two no-region cells compare as one, through S-0309 scenario A11) | (c) | the rule was built at the first batch (Q-1502). |
| S-0608 claim 16 (an aircraft whose object target a drain attempt clears: S-0501 claim 12's continuation) | not taken in | S-0501 claim 12's *Drain continuation (departure)* (revision 18, before eb1cc74) is not built either: `air` has no drain case. Handed off with S-0208. |

## Not taken in

- **S-0208 revision 34** (claim 24, a scatter destination is not an escape path; claim 12 and 13 wording; scenarios). Our scatter only sets the destination and clears the path (`scatter::request`), and the walk then goes through the ordinary path search with S-0212's gate, so claim 24 probably holds as built, but it is not tested and the revision is long. Handed off.
- **S-0608 revision 5 with S-0501 claim 12's drain continuation** (above).


## Arena (local lane, nothing tuned to the records)

Failing pairings (and side-unanimous stock outcomes reproduced) per record: baseline before step 1c, M-0001 37 of 96 (146 of 172), M-0002 17 of 24 (43 of 47), M-0003 6 of 35 (55 of 64); after the first batch 36 (146), 16 (43), 4 (59); after the second batch and the S-0611 merge 35 (145), 16 (43), 4 (59). The details and what moved are in `docs/development.md`, *Results* of the oracle arena.
