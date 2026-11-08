# S-0011 r29, S-0312 r54, S-0506 r37 and S-0208 r53: claim ledger, M8 step A3 intake 113

- **S-0011** r26 to r29 (commit 51bff48, from e6c9da3, the revision main built). Together with
  S-0312 it answers Q-2716. Claim 2: being destroyed, leaving play and being removed do not detach
  an object's tag, so a leftover still carries it, and a carrier that stops existing keeps its
  place in the tag's carrier count for the rest of the game. Claim 10 gives the consequence for an
  all-carriers tag, scenario T6 variant H tests it, and the Open question on a lost carrier's
  count is gone. New quirk `quirks.removed_carrier_keeps_tag` (default on).
- **S-0312** r51 to r54 (commit d86db4e, from 1391606). Only claim 29's *Infantry still showing
  death* changes: the removal of such an infantry unit does not detach its tag under the new
  switch, and detaches it with the switch off. Claims 10 and 11 are untouched by the delta, so
  they stay as the aircraft-retaliation build (64c600f) made them.
- **S-0506** r34 to r37 (commit 05bdaf2, from 7d84ba5). It answers Q-2748 in part and Q-2749.
  - Claim 7: a controller target in the air gives the missile no destination.
  - Claim 14: such a missile waits, with no flight, until a destination not in the air is given
    to it; that destination fixes the aim point then, and a destination in the air never does.
  - Claim 24: *t*₀ is the tick of that fixing.
  - Claim 11: the controller target is lost at the hit that destroys it, at going inside, at
    removal, and at the start of cloaking or going underground, whatever the spawner's house and
    sensors. It is cleared before the reset.
  - Claim 12: so the released missile gets the F8 cell.
  - Q-2748's questions 2 and 3 (which destination comes first, and when) stay open.
  - No quirk is added.
- **S-0208** r52 to r53 (commit 0eac343, from 1317d74). It answers Q-2746 (claim 6): an order
  whose behaviour is *capture* is not an enter order for the path-keeping rule. That covers the
  order to garrison (S-0214 row I8), capture, infiltrate and hijack. r53 adds scenario W17 parts
  (j) and (k) and rewrites the text of `quirks.enter_keeps_stored_path`. r53 does not touch
  Q-2750 (the scatter that drops a stored path), so its interim stays.

One request filed: **Q-2811** (S-0506 claim 11: is a lost controller target cleared when a
pending target is waiting, and what then follows claim 5's table). Interim: the target is cleared
only when the loss resets the controller; with a pending target waiting it stays, as r34 built it.
Q-2812 to Q-2820 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

### S-0011 r29 and S-0312 r54 (Q-2716)

- `quirks.removed_carrier_keeps_tag` in `glasnost_rules::quirks` (default on) and in the JSON
  schema.
- `objects::remove_with` calls `triggers::detach_object` only with the switch off. With it on, the
  object's entry in `TriggerState::objects` and its place in the count stay. Nothing removes them
  when the leftover ends or the object stops existing. Every reader of `objects` filters on
  `World::obj`, and object numbers are never reused, so a stale entry matches nothing.
- `leftover::sweep_hit`: the killing hit on a leftover with health left now makes S-0312 claim
  29's raises for a killing hit with no attacker and no attacking house
  (`death::leftover_destruction_raises`). They come after the registration (`with_restored`) and
  before the repeated death effects, with the leftover out of the world, so the actions they fire
  find nothing of it. The extra vehicle group follows `quirks.vehicle_destruction_extra_raises`.
- `triggers::destruction_raises`: its *not removed by a preceding action* stop now applies only to
  a victim present at the first raise, so a leftover's raises go on to the common group.
- The `Q-2716` interim lines in `leftover.rs` and `death.rs` are gone.

### S-0506 r37 (Q-2748 in part, Q-2749)

- `spawn::launch` gives a missile no destination when the controller target is an object in the
  air (`combat::weapon::in_air`), and fixes the aim at the launch otherwise, as before.
- `rocket::tick` no longer fixes the aim at a missile's first update (intake 110's Q-2748
  interim). A missile with no phase makes no update.
- `rocket::give_destination` is the new hook. It ignores a destination in the air. The first one
  not in the air becomes the missile's destination and fixes its aim from its heading point at
  that moment, starting claim 24's timer in that tick. A later one is ignored.
- `design: Q-2748 interim` (questions 2 and 3 still open, narrowed): `spawn::kamikaze_orders`
  gives a released rocket that has no flight its release cell through `give_destination`, at the
  shared re-order's attack on that cell. That is one of the candidates the Open question names.
  The rocket then flies to the centre of the cell under the target at its release, and detonates
  there, rather than standing on the map.
- `spawn::target_lost` (pub) settles the waiting clears and then runs the loss:
  - a lost pending target is cleared with no reset (claim 25);
  - a lost controller target with no pending target waiting is cleared first, then the reset
    runs, so `release_object` has no target and gives the F8 neighbour (claim 12).
  - Claim 8's release, the pulse and the other resets still release with the target.
- Callers of the loss:
  - the destroying hit (`spawn::killed`, from `death::kill`: before a death sequence, a sinking
    or a crash);
  - the start of cloaking (`cloak::machine::start`, before S-0516 claim 14's drop and with no
    exception);
  - going inside (`spawn::target_leaves_play`);
  - removal (`spawn::removing`).
- Going underground is a seam: no underground locomotor is built.

### S-0208 r53 (Q-2746)

- No code change. `walk::keeps_stored_path` already tests *enter* alone, and S-0214 row I8 already
  gives `Action::Capture` (*capture*). The citations now name r53, and W17 parts (j) and (k) are
  tested in both switch positions.

## Claims

| Claim | Status | Tests and notes |
|---|---|---|
| S-0011 2: a removed carrier keeps its tag and its place; not given back when it stops existing | tested | `claims_s0011_r29::a_removed_carrier_keeps_its_tag_and_its_place` (both positions) |
| 2 and 10, T6 variant H run 1: no firing on, one firing off | tested | `t6_h_run_1_a_carrier_destroyed_before_the_trigger_is_satisfied_still_counts`, `t6_h_run_1_switch_off_the_removal_detaches_and_b_fires` |
| 10, T6 variant H run 2: one firing at B in both positions | tested | `t6_h_run_2_both_destroyed_after_the_game_time_fire_once_in_both_positions` |
| 2: a leftover still carries its tag | tested | `the_sweep_hit_on_a_tagged_leftover_raises_its_tag` |
| 2: a carrier that leaves play by going inside keeps its tag | no change | `objects::take_inside` never detached a tag |
| Edge cases, `removed_carrier_keeps_tag` off | tested | `t6_h_run_1_switch_off_the_removal_detaches_and_b_fires`, `no_raise_from_the_sweep_with_the_switch_off_or_on_a_leftover_with_no_health`, and the off runs of the tests above |
| S-0312 29, *Infantry still showing death*: its removal keeps the tag on, detaches it off; no new raise | tested | `an_infantry_carrier_keeps_its_tag_through_its_death_sequence_and_removal` |
| S-0312 10 and 11 | no change | not in the delta; as the aircraft-retaliation build left them |
| S-0203 13 (g): claim 29's raises at the sweep's killing hit | tested | `the_sweep_hit_on_a_tagged_leftover_raises_its_tag` (four firings of a fresh repeating destroyed-by-anything trigger); none on a leftover with no health |
| S-0506 7: no destination for a controller target in the air | tested | `claims_s0506_r37::r26_a_no_destination_and_no_flight_for_a_target_in_the_air` |
| 14: no flight, no phase, speed 0, pitch 0, at its placement point | tested | the same test |
| 14: a destination in the air is ignored; the first not in the air fixes the aim then; later ones ignored | tested | `r26_the_first_destination_not_in_the_air_fixes_the_aim_then` |
| 14: the target landing gives the missile nothing | tested | `r26_b_the_target_landing_gives_the_missile_nothing` |
| 14: a landed aircraft target fixes the aim at the launch (R26 (d)) | tested | `r26_d_a_landed_aircraft_target_fixes_the_aim_at_the_launch` |
| 24: *t*₀ the tick of the fixing | tested | `r26_the_first_destination_not_in_the_air_fixes_the_aim_then` |
| 14, Open questions: which destination comes first (Q-2748 questions 2 and 3) | partly (interim) | `q2748_interim_the_release_cell_given_at_the_re_order_fixes_the_flight` |
| 11: the loss at the destroying hit of a target that stays in play (R7 (c)) | tested | `r7_c_the_reset_comes_at_the_destroying_hit_of_an_infantry_target` (an infantry death sequence; the sinking ship is not tested on its own: `death::kill` calls `spawn::killed` before the sinking branch) |
| 11: the loss at the start of cloaking, whatever house and sensors (R7 (d)) | tested | `r7_d_the_reset_comes_at_the_start_of_cloaking_whatever_house_and_sensors` |
| 11: the loss on going underground (R7 (e)) | seam | no underground locomotor is built |
| 11: hijacking (Open question) | no change | not given |
| 11: the controller target lost while a pending target waits | partly (interim) | Q-2811 filed; `design: Q-2811 interim` in `spawn::target_gone` keeps r34's behaviour (no clear, no reset) |
| 11 and 12: the target cleared before the reset; F8's cell after going inside, removal or a destroying hit (R21 lost-target variant) | tested | `r21_going_inside_or_removed_the_release_cell_is_f8s`, `r7_c_...` |
| 12: the sale of a building target (R21 (d)) | partly | a sale removes the building through `objects::remove`, the removal case of the test |
| S-0506 r34 tests | no change | `claims_s0506_r34.rs` passes; its Q-2748 interim test is removed |
| S-0208 6 (r53), W17 (j): a garrison order to a walker under *move* drops its path, both positions | tested | `claims_s0208_r53::w17_j_a_garrison_order_to_a_walker_under_move_drops_its_path` |
| 6 (r53), W17 (j) control: an enter order on the same layout keeps it | tested | `w17_j_control_an_enter_order_on_the_same_layout_keeps_the_path` |
| 6 (r53), W17 (k): a move order to a walker under *capture* drops its path, both positions | tested | `w17_k_a_move_order_to_a_walker_under_capture_drops_its_path` |
| Edge cases, `enter_keeps_stored_path` (text rewritten) | tested | the r52 tests and the both-position r53 tests |
| Q-2750 (the scatter that drops a stored path) | no change | not answered by r53; intake 110's interim stays |

The Q-2811 interim row above is tested by
`claims_s0506_r34::no_reset_when_a_pending_target_waits` (now citing the interim, with the kept
target asserted).

## Formats and goldens

No hashed state was added, so the formats stay at save 114, AI game 79 and wire 12. The schema
gains the switch (`data/schema/ruleset.schema.json`). `glasnost-sim-gate --include-ignored`
passes with no golden moved.

## Lanes

Main 9a170d0 against the branch, measured before the commit.

- **Sweeps** (seeds 1 to 16): M6c, M7a and M7b are identical line for line. The smallest minima
  are M6c 0, M7a 1, and the M7b drain 3. Every median is unchanged.
- **Arena** (`arena --record all --threads 2`): the records (`ours-*.csv`, `oracle-*.txt`) are
  byte-identical. Failing rows are as on main: M-0001 12 of 96, M-0002 2 of 24, M-0003 2 of 35,
  M-0019 0 of 6, M-0020 1 of 25, M-0021 11 of 50.
- **`glasnost-ai --include-ignored`**: 263 passed, 0 failed, before and after.
- **AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games): 29 games are identical and 7
  changed. Six of the seven are on map 0, the map with water and ships.
  - Seed 7, map 0, sides 0,2: 44 252 to 38 088 ticks; house 0 wins in both.
  - Seed 7, map 0, sides 1,2: the winner changes from house 1 to house 0 (73 972 to 47 234 ticks).
  - Seed 8, map 0, sides 1,2: the winner changes from house 0 to house 1 (84 773 to 72 912 ticks).
  - Seed 9, map 0, sides 0,1: 41 567 to 48 463 ticks; house 0 wins in both.
  - Seed 9, map 0, sides 1,2: was won by house 0 at 45 490 ticks; is **not over at 120 000**
    (no winner). Both houses end with bases, small armies standing in *guard* or *area guard*,
    and house 0 at 0 credits. No stuck unit shows in the end-of-game behaviour list. Rocket
    flights went from 76 to 103 and spawners from 7 to 17 over the longer game.
  - Seed 10, map 0, sides 0,1: 40 519 to 39 055 ticks; house 0 wins in both.
  - Seed 10, map 2, sides 0,1: 55 224 to 47 222 ticks; house 0 wins in both.
  - Median: 44 306.5 to 44 996 ticks (the unfinished game counted as 120 000).
  - **Narrowing.** A diagnostic binary with `removed_carrier_keeps_tag` off by default
    reproduced the branch's three seed 9, map 0 results exactly, the unfinished game included.
    So the tag change is not the cause, and the change comes from the S-0506 r37 side: the
    loss at the destroying hit and at cloaking, F8's release cell after it, or rockets launched
    at aircraft. It has not been traced further. The diagnostic edit was made and reverted with
    Edit and was never committed.
