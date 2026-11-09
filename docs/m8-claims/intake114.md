# M8 A3 intake 114: S-0501 r154, S-0215 r137, S-0208 r58, S-0209 r60, S-0312 r66

Brief: `.claude/worktrees/M8-A3-INTAKE114-BRIEF.md`. Answered requests: **Q-2792** (S-0501 r154),
**Q-2782** (S-0215 r137 and S-0208 r58, backed by M-0024 and M-0025), **Q-2750** (S-0208 r58,
S-0209 r60) and **Q-2793** (S-0312 r66). New requests: **Q-2821** (claim 18's *Missile types*
against claim 35 for a flagged type that is not a rocket type) and **Q-2822** (the same destination
given again and the blockage delay). Two quirks are new, with switches at the spec defaults and C7
verdicts in [c7-play-fixes.md](c7-play-fixes.md); neither is admitted to the play-fixes layer.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0501 r154 claim 52 (Q-2792): an aircraft on its pad retaliates as any other aircraft; the override keeps the pad contact and gives no destination | built, tested | `acquire::retaliate` (the `air::on_its_pad` interim removed); `claims_s0215_air_retaliate.rs` `claim_52_an_aircraft_on_its_pad_retaliates` (on the pad and off it) |
| S-0501 r154 claim 52, scenario AC34 a: an aircraft in *guard* on its pad, hit by an enemy, takes off and attacks | built, tested | `claims_s0501.rs` `claim_52_ac34_a_guard_on_the_pad_retaliates` |
| S-0501 r154 claim 52, AC34 b: an aircraft docked in *sleep* retaliates only with player return fire | built, tested | `claim_52_ac34_b_the_docked_sleep_retaliates_with_return_fire` (both settings) |
| S-0501 r154 claim 52, AC34 b1 and T3's last row: the service step queues *sleep* for a settled aircraft with the retaliation's *attack* current, ending it on the pad | built, tested in part | `air::pad_turn`; `claim_52_ac34_b1_sleep_ends_the_retaliation_on_the_pad` (up to the second trigger: no round, repair, charge or take-off while *A* holds *V*); the second trigger's hand-back of *V* is the existing restore at the target's loss (S-0215 claim 10, T2 row 1), not run again here |
| S-0501 r154 quirk `aircraft_pad_sleep_ends_retaliation` (default on); off, the step skips the attack as one in *enter*, and the aircraft takes off | built, tested both ways | `quirks.rs`; `claim_52_ac34_b1_…` (on), `claim_52_quirk_off_the_service_step_skips_the_attack` (off); C7: not admitted |
| S-0501 r154 claim 26: servicing ends when no settled aircraft is left after the first-bullet release | built, tested | `air::pad_turn`; `claim_26_r154_servicing_ends_after_the_first_bullet_release` |
| S-0501 r154 claim 28: the half-reload test with a target comes before the round and the repair; at full ammunition with a target a step neither repairs nor releases | built, tested | `air::pad_turn`; `claim_26_r154_a_target_at_full_ammunition_stops_the_repair_and_release` (half-reload quirk both ways) |
| S-0501 r154 claim 18, *Missile types*, and T2's note: a missile-type object takes the rows for an aircraft on the ground at any height | built, tested | `air::fall_back_rows`; `claim_18_r154_a_missile_type_takes_the_ground_rows_in_flight` |
| S-0501 r154 claim 18 against claim 35: a flagged type that is not a rocket type | blocked (Q-2821), interim built and tested | `// design: Q-2821` in `air::fall_back_rows` (the flag decides); the same test. S-0506 r40 (on main after this intake started, not built here) keeps Terms' *missile type* as the flag, says S-0501 claim 18 gives such an object the ground rows at any height, and counts a flagged type that is not a rocket type as a non-missile slot for its claim 3; it does not answer claim 35's "missile slot types", so Q-2821 stays open. Later: S-0501 r172 answers Q-2821 as the interim built it (the flag alone decides); intake 118 replaces the interim cite ([intake118.md](intake118.md)) |
| S-0215 r137 claim 19 case 2 (Q-2782): reaches a free hunting vehicle of a computer-controlled house, B6 f1 | built (already), tested | `damage::damage_scatter_sent` unchanged, cites updated; `claims_s0215_scatter_damage.rs` `claim_19_b6_f1_a_hunting_vehicle_of_a_computer_house_scatters` |
| S-0215 r137 B6 f3: the control without the scatter property; a human owner | built, tested | `claim_19_b6_f3_control_needs_the_scatter_property` |
| S-0215 r137 B6 g1 and g2: an own or allied attacker gives case 2 | built, tested | `claim_19_b6_g1_g2_an_own_or_allied_attacker_gives_case_2` (with an enemy control that retaliates) |
| S-0215 r137 B6 g3: an enemy the weapon cannot harm gives case 2 | built, tested | `claim_19_b6_g3_an_enemy_the_weapon_cannot_harm_gives_case_2` |
| S-0215 r137 B6 f2 (baseline) and the arena parts of f1 | left (arena-scale records, M-0024 and M-0025); the lane rows are in the report | — |
| S-0208 r58 claim 6 / Q-2750: a walker standing with no walking target whose stored path a scatter drops gets the extra locomotion | built, tested | `walk::standing_free`, `scatter::request`; `claims_s0208_r58.rs` `w17_l_a_dropped_path_gives_the_extra_locomotion` |
| S-0208 r58 claim 6: a scatter to a different cell ends the retry delay and restarts the blockage delay | built, tested | `claims_s0208_r58.rs` `w17_m_a_scatter_ends_the_retry_delay_of_a_blocked_walker`; `claims_s0208_r52.rs` (the rewritten scatter test) |
| S-0208 r58: `walk::keeps_stored_path` and targeting row I8 cites (S-0508 claim 1 for the occupation test, claim 2 for the garrison order) | aligned | `walk.rs`, `targeting.rs` |
| S-0209 r60 claim 10, C5 c: a new destination restarts the blockage delay | built, tested | `path::destination_changed`, `PathState::blocked_for`; `claims_s0209.rs` `c5_c_a_new_destination_restarts_the_blockage_delay` |
| S-0209 r60 claim 10: a destination written by any other rule restarts it; a cleared destination ends the wait | built, tested | `a_destination_written_by_another_rule_restarts_the_blockage_delay`, `a_cleared_destination_ends_the_wait` |
| S-0209 r60 claim 10: the same destination given again | blocked (Q-2822), interim built and tested | `// design: Q-2822` in `path::destination_changed` (no restart); `q2822_interim_the_same_destination_again_keeps_the_blockage_delay`. Later: S-0209 r76 answers Q-2822 (every assignment restarts the delay); intake 121 replaces the interim and its test ([intake121.md](intake121.md)) |
| S-0312 r66 claim 11 (Q-2793): the height is read above the deck on a deck, 10 leptons or less with no lower bound | built, tested | `death::marked_low_over_water`, `World::height_above_surface`; `claims_s0312_marked_water.rs` `claim_11_the_10_lepton_boundary` (with -30), `d8_marked_deck_variant_a_both_ways` |
| S-0312 r66 quirk `bomb_mark_water_death_on_deck` (default on); off, a vehicle on a deck takes the ordinary effects | built, tested both ways | `quirks.rs`; `d8_marked_deck_variant_a_both_ways`, `d8_marked_deck_death_weapon_both_ways`; C7: not admitted |
| S-0312 r66 claim 12: the bomb-mark column tests the mark alone (passengers of a marked vehicle that cannot crash die, on land too) | built, tested | `d8_marked_deck_variant_b_the_mark_alone_kills_the_passengers` (both quirk positions) |
| S-0312 r66 claim 12, D9 h: passengers get out of an unmarked vehicle on a deck (height from the deck) | built, tested | `d9_h_passengers_get_out_on_a_deck` (with the off-deck control) |
| S-0312 r66 claim 15, D11 h: no crash starts from a deck | built, tested | `jump::start_crash` reads the height from the deck; `claims_s0502_crash.rs` `s0312_claim_15_d11_h_no_crash_from_a_deck` |
| Q-2801 and Q-2802 interims (intake 112) | left, as the brief says | — |

## Lanes

Base: main bce62e5 (save 116, AI game 81); its simulation, rules and AI code equal 83fb4e6's, the
base this branch is rebased onto.

- Oracle arena: the same failing rows as main for every record (M-0001 12 of 96, M-0002 2 of 24,
  M-0003 2 of 35, M-0019 0 of 6, M-0020 1 of 25, M-0021 11 of 50) and the same winners; 14 M-0001
  and 8 M-0002 games end at another sample. The rows the scatter from damage build moved still
  fail ([scatter-damage.md](scatter-damage.md)).
- Seed sweeps 1–16: M7b identical; M6c smallest minimum 0 on both, three medians move by one or
  two; M7a smallest minimum 1 on both, six seeds move.
- `glasnost-ai` with ignored tests, release: 263 pass, as on main.
- AI quick lane (seeds 7–10, maps 0, 2 and 8, three side pairs, 36 games): three winners change,
  all at seed 7; seeds 8 to 10 keep every winner.
- **Traced** with `GLASNOST_AI_HASH` and `GLASNOST_AI_OBJS`. The branch adds hashed state, so the
  trace used a throwaway build with `PathState::blocked_for` left out of the hash (not committed),
  and the dumps were compared with that field filtered out.

  | Game | First differing tick | First differing object |
  |---|---|---|
  | seed 7, map 0, sides 0 and 1 | 13579 | pad building 1206 (house 0): servicing ends on the branch, goes on on main |
  | seed 7, map 2, sides 0 and 2 | 10289 | pad building 871 (house 0): the same |
  | seed 7, map 8, sides 0 and 2 | 9738 | pad building 510 (house 0): the same |

  In each the pad's only aircraft (full health, full load, no target) is released by the service
  step's first bullet on both builds. Main counted it as settled before that release and kept the
  servicing running; S-0501 r154 claim 26 ends servicing when no settled aircraft is left after
  the first-bullet release (`air::pad_turn`). The next aircraft to land is then serviced on
  arrival, and the games drift from there. The change follows the spec.

## Formats

`PathState::blocked_for` is new hashed state (the destination the running blockage delay was
started for), so the save format goes to 117 and the AI game format to 82, and the determinism
gate's goldens are regenerated (every scenario's hashes move; the behaviour changes above move some
games' lengths too).
