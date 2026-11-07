# Aircraft retaliation and the marked-water death (M8 build; S-0215, S-0309, S-0501, S-0312)

Brief: `.claude/worktrees/M8-AIR-RETALIATE-BRIEF.md`. Specs: S-0215 claims 10, 16, 18 and 19
with F7 and scenario B7; S-0309 claim 18 and claim 25 part h; S-0501 Terms, claim 28, tables T1
to T3; S-0312 claims 4 to 12, State and ordering, Edge cases and scenario D8's marked-water
variant; S-0503 claim 11; S-0511 claims 9 and 10. Requests: **Q-2792** (an aircraft on its pad)
and **Q-2793** (the height and mark tests of claim 11).

**Result.**
- `acquire::retaliate` now overrides an aircraft that may retaliate to *attack* through the same
  `override_attack` as every other actor (S-0215 claims 10 and 18). S-0501 owns what follows:
  T1 runs the attack, and T2 row 1 restores the suspended behaviour at the target's loss.
- The aircraft seam of the scatter-damage build is closed. The one exception is an aircraft on
  the ground at its pad: S-0501 claim 28 contradicts S-0215 claim 18 there (Q-2792), so it is
  still answered `Held`.
- S-0309 claim 18's row for harvesting slaves, slave masters and spawner types was missing for
  every kind of actor, and is now built.
- In S-0312, a bomb-marked vehicle that dies within 10 leptons above water now takes claim 11's
  effects, in a kill and in the defeat sweep's repeat. A marked vessel no longer sinks (claim
  10).
- No quirk is named by these claims (`every_spec_quirk_has_a_switch` passes). The empty splash
  list is S-0312's "no switch" departure. So no switch and no C7 verdict is owed.
- Save format 114, AI game format 79.

## The bomb mark: who writes it

The engine writes the mark, so claim 11 is reachable in play:
- `hover::bomb_mark` (S-0503 claim 11): a hover whose switch-off spin ends over water below 104
  leptons. `hover::start_fall` then starts a fall with the mark.
- `fall::start(w, id, true)` (S-0511 claims 9 and 10), called from `bridges::blow` for a vehicle
  dropped from a collapsing deck, and from `hover::start_fall`. The mark is `Object::fall.bomb_mark`.
  Landing does not clear it.

`transport::bomb_marked` reads both. The fall's landing hit (`fall::landing_hit`) kills the
vehicle, which then takes claim 11's effects when it lands on water.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0215 claim 18 / S-0309 claim 18: an aircraft that may retaliate overrides to *attack* (claim 10) | built, tested | `acquire::retaliate`; `b7_b_an_aircraft_in_guard_overrides_to_attack_and_reverts_at_the_attackers_death` |
| S-0215 claim 18: a player-house mobile actor retaliates only in *guard*, *area guard* or *patrol* unless player return fire (scenario B7 parts a and c, for an aircraft) | built, tested | `b7_a_an_aircraft_in_move_does_not_retaliate`, `b7_c_with_player_return_fire_an_aircraft_in_move_retaliates` |
| S-0215 F7: an aircraft that cannot reach the attacker overrides only within 256 S + 128 | built, tested | `b7_d_f7_distance_boundary_for_an_aircraft` (896 and 897 leptons) |
| S-0309 claim 25 part h / S-0215 claim 18: a neutral-house aircraft retaliates whatever its distance | built, tested | `claim_18_a_neutral_aircraft_overrides_at_once_beyond_f7` |
| S-0215 claim 10 / S-0501 T2 row 1: the override is reverted at the target's loss | built (existing), tested | `b7_b_…_reverts_at_the_attackers_death` |
| S-0501 T1: an aircraft with no ammunition that is overridden to *attack* finishes the run at once and falls back (claim 17) | built (existing), tested | `claim_18_an_aircraft_with_no_ammunition_retaliates_and_falls_back` |
| S-0215 claim 19 case 1: no longer reaches an aircraft that retaliates (`damage_scatter_sent`, `Done` gives false) | built, tested | `claim_19_case_1_no_longer_reaches_an_aircraft_that_retaliates` (with the far control) |
| S-0215 claim 16: the order within a hit (retaliation before the scatter from damage) for an aircraft | built (existing order), tested | the same test, and the scatter-damage ledger |
| S-0501 claim 28 against S-0215 claim 18: an aircraft on its pad | blocked (Q-2792), interim built and tested | `air::on_its_pad`; `q2792_interim_an_aircraft_on_its_pad_is_held_back` |
| S-0501 T3 for an aircraft overridden to *attack* on its pad (take-off, contact, service) | blocked (Q-2792) | not reached under the interim |
| S-0309 claim 18: not of a spawner type | built, tested | `claim_18_a_spawner_type_never_retaliates` |
| S-0309 claim 18: neither a harvesting slave nor a slave master | built, tested (slave master) | `claim_18_a_slave_master_never_retaliates`; the harvesting-slave half is the same one-line test, `seams::is_harvesting_slave`, not tested separately |
| S-0312 claim 11: a marked vehicle within 10 leptons above water skips claims 4 (debris) and 9 and shows the wake, then the last splash entry, with no draws | built, tested | `death::marked_low_over_water`, `death::wake_and_splash`; `d8_marked_water_variant_wake_and_last_splash_in_place_of_the_ordinary_effects` |
| S-0312 claim 11: the 10-lepton boundary, inclusive | built, tested | `claim_11_the_10_lepton_boundary` |
| S-0312 claim 11: over a water cell only | built, tested | `claim_11_needs_a_water_cell` |
| S-0312 claim 11: claims 5 and 6 skipped; claim 7's bomb still goes off | built, tested | `claim_11_skips_the_death_weapon_and_keeps_the_attached_bomb` |
| S-0312 claims 11 and 12: passengers killed (type cannot crash) | built (claim 12 row 3, existing), tested | `claim_11_passengers_die_when_it_cannot_crash` |
| S-0312 claims 11 and 12: passengers kept for the crash (type can crash) | built, tested in part | `claim_11_a_crashable_type_keeps_its_passengers_inside`. Our engine starts a vehicle's crash only for jumpjet movement, so the wheeled test vehicle leaves play and its passengers are lost with it. |
| S-0312 claim 10: a marked vessel does not sink | built, tested | `naval::sinks_at_kill`; `claim_10_a_marked_vessel_does_not_sink` |
| S-0312 Edge cases: an empty splash list shows no splash (departure, no switch); no wake defined | built, tested | `claim_11_without_a_splash_list_or_a_wake` |
| S-0312 claim 11 through the real path: a fall with the mark ends in the landing hit over water | built, tested | `d8_marked_water_variant_through_the_landing_hit` |
| S-0203 claim 13 (g) with S-0312 claim 11: the defeat sweep's repeat for a marked leftover | built, tested | `Repeat::marked_water`; `claims_s0203_r53.rs` `a_bomb_marked_leftover_low_over_water_repeats_the_wake_and_splash` |
| S-0312 claim 11: where the height is measured (terrain or deck), negative heights | blocked (Q-2793 questions 1 and 2), interim built | terrain surface at its position, negatives count |
| S-0312 claim 12 row 3: a marked vehicle outside claim 11's conditions | blocked (Q-2793 question 3) | left as before: every marked vehicle that cannot crash loses its passengers |

## Seams

- **A crushed marked vehicle.** No seam and no request. S-0210 claim 4 says a crushed vehicle
  takes no damage, is not killed by a hit, and shows claim 9's explosion and destruction animations
  "whatever its type's sinking or death-frame properties". `crush` calls
  `death::vehicle_animations` directly, so a marked vehicle crushed at the water surface still
  shows claim 9's animations, as specified.
- **Crashing vehicles other than jumpjets.** Not built before this build either (claim 15 starts
  a crash only through `jump::start_crash`). Unchanged.
