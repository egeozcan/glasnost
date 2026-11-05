# S-0215 r118, S-0107 r40, S-0513 r122: claim ledger, M8 step A3 intake 99

S-0215 r104 to r118 (through r115), S-0107 r31 to r40 and S-0513 r115 to r122 answer **Q-2566**
("in the middle of firing" is S-0107 claim 11's attack state) and **Q-2586** (the no-threat
fall-back's place within a hit). Both requests are at status answered. Request filed:
**Q-2616**, on target changes that the specs do not place (interim below).

**Result.** An infantry unit is held not ready (S-0215 claim 11) while it is in the attack state,
and the attack state now ends at the shot, at a stage-delay-0 start in its place in the update, and
at a change of target with a request for *deployed*, *prone* or *stand*. The no-threat fall-back
comes after the retaliation. `quirks.prone_attack_state_persists` has both positions built and
tested, and a C7 verdict (rejected). A new hashed field (`InfantryState::watch`) and changed
replays bump the formats: save 105, AI game 69 (main: 104 and 68; wire 12 unchanged). The
goldens were regenerated (below).

## Where the attack state's endings sit in the update

In `tick::object_turn`, for an infantry unit (S-0215 r118 claim 11, *In the middle of firing*):

1. At the start of its turn: `infantry::begins_frozen` notes whether the tick begins with a
   current sequence whose stage delay is 0, and `watch_set` notes the target.
2. Before the early (first per-tick) start: `watch_test`. A target that S-0309 claim 27's
   turn-start drop (`held::turn_start_drop`) changed ends the state and makes the request before
   any start sees it. A target's leaving play in another actor's turn ends it at once
   (`held::holders_drop` calls `infantry::target_changed`).
3. The first start, the berserk countdown, the behaviour processing (with claim 33's per-tick
   search) and movement. After them, and before the late start: `watch_test` (the unit's own
   processing or search changed its target), then `stage_delay_zero_end`: a unit whose tick began
   frozen and is in the attack state leaves it and asks (not forced) for *stand*. So only the
   first start sees the state; the late start does not.
4. After the fear turn, before the firing turn (`behave::infantry_step`): `watch_test`, for a
   change made since the late start.
5. The firing turn: the begin (request taken or turned down), the shot at *F* and its end, the
   not-ready end at *F* and the end with no target (S-0107 claim 11).

Orders: `orders::apply_pending` notes the target before each order and tests it after, so the
order's change ends the state as it takes effect (after the turn pass, S-0203 claim 7), before the
unit's next update. S-0309 claim 23's attack-move discard and put-back carry the note
(`watch_carry`), so they are no change.

## Claim parts

| Spec | Part | Status | Code | Tests |
|---|---|---|---|---|
| S-0215 r118 | Terms: *firing turn*, *in the middle of firing* | built | `tick::object_turn` order above | `claim_11_readiness_is_the_attack_state` |
| S-0215 r118 | Claim 11: in the middle of firing is the attack state and nothing else; the rearm countdown, a firing kind still playing and a shot in flight do not hold readiness | tested | `process::ready` (infantry branch) | `claim_11_readiness_is_the_attack_state`, `b45_c1_w1_attack_queued_inside_the_firing_kind_becomes_current_after_the_shot`, `b45_c1_w2_and_w3_attack_after_the_shot_is_current_at_the_first_update` |
| S-0215 r118, S-0107 r40 | Begins in the firing turn, the request taken or turned down; does not wait for the shot | tested | `behave::infantry_step` (as before) | `s0107_claim_11_firing_kind_by_stance`, `s0107_claim_11_enters_attack_and_snaps_facing`, `b45_c2_a_second_begin_inside_the_kind_holds_the_order_to_the_late_start_after_it` |
| S-0215 r118 | Begins with the airstrike reset in place of a request | seam | none: no airstrike sequence is built | none |
| S-0215 r118, S-0107 r40 | Ends at the shot | tested | `behave::infantry_step` (after `fire::shot`) | `s0107_claim_11_shot_at_the_fire_frame`, `b45_c1_w1_*` |
| S-0215 r118, S-0107 r40 | Ends at the not-ready fire frame, and with no target, in the firing turn | tested | `behave::infantry_step` (as before) | `s0107_claim_11_not_ready_at_the_fire_frame_ends_attack`, `s0107_claim_11_attack_ends_without_a_target` |
| S-0215 r118, S-0107 r40 | Stage-delay-0 ending: after processing, before the late start, with a request for *stand*; in the first update that begins with *stand* after the kind completes | tested | `infantry::stage_delay_zero_end`, `begins_frozen` | `b45_c2_*`, `s0107_attack_state_ends_the_tick_after_completion` (rewritten for r40: the kind completes before *F*) |
| S-0215 r118, S-0107 r40 | Change of target by an order (move, deploy, stop, attack on another target): ends at once, as the order takes effect | tested | `orders::apply_pending` with `watch_set` and `watch_test` | `b45_c3_a_move_order_ends_the_state_as_it_takes_effect`, `m8_d_an_attack_order_on_another_target_after_the_shot`, `r115_claim39a_sw12_l9a_and_l9_*` (the deploy order) |
| S-0215 r118, S-0107 r40 | Change of target by the target's leaving play: at once | partly | `held::holders_drop` calls `infantry::target_changed` | no test of its own; the same `target_changed` as the order tests |
| S-0215 r118, S-0107 r40 | Change by the unit's own processing or per-tick search: before the late start, and before the firing turn | partly | `watch_test` in `tick::object_turn` | no test that isolates these two routes |
| S-0215 r118, S-0107 r40 | Every change of target asks (not forced) for *deployed*, *prone* or *stand*, in or out of the attack state | partly | `infantry::end_attack(.., true)` | *stand*: `b45_c3_*`, `m8_d_*`. *Deployed* and *prone* not tested |
| S-0215 r118, S-0107 r40 | S-0309 claim 23's attack-move discard and put-back are no change | partly | `combat::acquire::attack_move_search`, `process::refresh_attack_move` with `watch_carry` | no test |
| S-0215 r118, S-0107 r40 | Ends when a completed *secondary-fire* or *secondary-fire-prone* is repeated for an airstrike | seam | none | none |
| S-0215 r118 | Other routes that change an infantry unit's target (retaliation, berserk drop, mind-control release, takeover) | interim (Q-2616) | the lazy firing-turn ending in `behave::infantry_step`, no request | `s0107_claim_11_attack_ends_on_a_new_target` |
| S-0215 r118 | Scenario B45 c (rewritten: *F*, rearm delay *R*, the order inside and after the kind) | tested | as above | the four `b45_*` tests (synthetic, one run each, not three) |
| S-0215 r118 | Claim 16: order within the hit (after anger, retaliation and the scatter from damage; before fear, request B and the crush response) | built | `damage::reactions` | `claim_16_b5_c_retaliation_comes_before_the_no_threat_test`, `claim_16_a_retaliation_into_a_no_threat_attack_falls_back` |
| S-0215 r118 | Claim 16: the scatter from damage before the test (request A); request B after it; an aircraft's scatter-first fall-back | seam | none: the scatter from damage is not built in our hit reactions | none |
| S-0215 r118 | Claim 16 *Results*: a hit that names an attacker, lowers health and reports *unchanged*, *now yellow* or *now red*; a heal or any *unaffected* hit does not trigger it | tested | `damage::reactions` | `claim_16_b5_d_a_heal_or_a_hit_that_lowers_no_health_gives_no_fall_back` (renamed from `claim_16_interim_*`), `claim_16_no_fall_back_on_a_killing_hit` |
| S-0215 r118 | Open question: a berserk hit and the fall-back | follows S-0302 claim 10 (r41, kept in r43) | S-0302 claim 10 (r41, kept in r43) says the fall-back follows a berserk hit from a known attacker. Intake 100 built that in `damage::berserk_reactions`, which does not go through `damage::reactions`, so the `dealt > 0` test there does not apply to it | intake 100's `claim_10_a_berserk_hit_makes_a_no_threat_infantry_unit_fall_back` |
| S-0215 r118 | Scenario B5 c and d | tested | as above | as above |
| S-0107 r40 | Claim 11 *End of the attack state*: at completion the unit is in the attack state only if it began again while the kind played, or the kind completed before *F* | tested | as above | `b45_c2_*`, `s0107_attack_state_ends_the_tick_after_completion` |
| S-0107 r40 | Prone repeat shot: at most once per start of *prone*, the shot ending the state | tested | `infantry::sequence_turn` and the shot's end | `m8_a_prone_short_rearm_repeat_shot_in_prone_by_the_switch` (both positions) |
| S-0107 r40 | Long rearm: every shot from a new *fire-prone* (scenario M8 a, long-delay runs) | tested | as above | `m8_a_prone_long_rearm_every_shot_from_a_new_fire_prone` (both positions) |
| S-0107 r40 | `quirks.prone_attack_state_persists`, off: a prone unit's attack state ends when *fire-prone* (or a non-repeated *secondary-fire-prone*) completes | tested | `infantry::sequence_turn` | the two `m8_a_*` tests; C7: rejected ([c7-play-fixes.md](c7-play-fixes.md)) |
| S-0107 r40 | State and ordering: leaves the attack state on *t* + 1 and can start a new firing kind on *t* + 1 | tested | as above | `s0107_attack_state_ends_the_tick_after_completion` |
| S-0107 r40 | Scenario M8 d rewritten (the request at a change of target; the destroyed-wall variant dropped) | tested | as above | `m8_d_an_attack_order_on_another_target_after_the_shot` |
| S-0107 r40 | Scenario M8 b (the airstrike reset) | seam | as the airstrike rows | none |
| S-0513 r122 | Claim 39a *The firer* (Q-2566): the deploy order's change of target ends the attack state as the order takes effect; *unload* current at the first update's first start and processed in it, before any *attack* processing | tested | `orders::apply_pending` | `r115_claim39a_sw12_l9a_and_l9_*` (now asserts *unload* in the first update, rec[1], and the order taken in its own tick, rec[0]) |
| S-0513 r122 | SW12 l9a sharpened; l9b dropped | tested (l9a); l9b gone | as above | as above |
| S-0513 r122 | The other changes of the delta: backing and wording of claim 39a and SW12 (`quirks.phase_in_beam_spares_infantry` reworded, default unchanged) | no code | none | none |
| S-0215 r118 | *Changes for other specs* (S-0208, S-0309, S-0107, S-0607 and S-0513 notes) | no code | none: the S-0107 and S-0513 parts are built above from those specs' own revisions | none |

Tests adjusted for r40's shot ending (no behaviour asserted against the spec was dropped):
`s0107_claim_11_enters_attack_and_snaps_facing` and `s0107_claim_11_firing_kind_by_stance` use a
fire frame of 3, so no shot ends the state within the step; `destination_on_the_target_is_lost_on_firing`
(S-0208 claim 17) observes the firing sequence's start or its shot, since with *F* = 0 the state
begins and ends in one turn.

## Sweep 5b and intake 94 rows

| Row | Now |
|---|---|
| intake 94: SW12 l9b, the release at an *attack* processing before *unload* starts | closed: r122 drops l9b |
| intake 94: 39a, the firing state ends after both of the first update's per-tick starts | closed: the firing state is the attack state, and the deploy order ends it as the order takes effect; l9a tested |
| seams-5b: Q-2566 | answered, built here |
| seam S-0215 claim 16, part 11 (the order against retaliation) | built: retaliation first ([seam-s0215-c16.md](seam-s0215-c16.md)) |
| seam S-0215 claim 16, part 13 (which damage triggers it) | built: r118 *Results* |

## Request filed

**Q-2616**: when an infantry unit's change of target by a route the specs do not place (a
retaliation, a berserk hit's drop, a mind-control release or a takeover, written from another
actor's turn) ends its attack state, and whether gaining a target from none is a change.
Interim: those routes keep the engine's earlier rule, the attack state ending lazily at the
unit's next firing turn when its target differs from the one it entered with, with no request
(`behave::infantry_step`); gaining a target from none is no change.

## Formats and goldens

`InfantryState::watch` is new hashed state, and the attack state's new endings change what a
saved game replays to: save format 104 to 105, AI game format 68 to 69 (one past main after
intake 104, d9ad36c; wire 12 unchanged). All nine goldens were regenerated
on the tree rebased onto d9ad36c with
`GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored`:
`scripted.txt`, `combat.txt`, `skirmish.txt`, `duel.txt`, `duel-diagonal.txt`,
`duel-naval.txt`, `duel-m6c.txt`, `duel-m7a.txt` and `duel-m7b.txt`. The hash lines move only while
an infantry unit is in play, which fits the state hash covering the new field of
`InfantryState`:
- In `scripted.txt`, `duel-m6c.txt`, `duel-m7a.txt` and `duel-m7b.txt` every line from line 2
  moves.
- In `combat.txt`, lines 2 to 392 move and the rest agree with main.
- In `duel.txt`, `duel-naval.txt`, `duel-diagonal.txt` and `skirmish.txt`, the lines agree with
  main before line 552 to 560 and move from there. Every golden kept its
length, so the skirmish ends on the same tick. The M6c, M7a and M7b sweeps over the same
fixtures are identical before and after. The lanes compare main at 7580c30 (intake 103; its
code is a143ecc's) with this branch rebased onto it; intake 104, merged after, changed no arena
row on its base. The synthetic fixtures give no infantry type a fire
frame or a firing sequence, so *F* is 0 and the attack state begins and ends in one firing turn,
and `process::ready` never sees it. So the goldens most likely moved only through the hash; the
games were not compared hash-free. With the install's art, games do play differently. The
arena keeps its verdicts (the same 15 failing pairings, M-0019 at 0 of 6, M-0020 at 1 of 25
checked, M-0021 at 10 of 50, the same failing rows), but game detail moves in pairings that have
infantry. In the AI quick lane, all 36 games change. 35 of the 36 end with a winner before
120000 ticks, on main and on the branch. 14 winners flip between the houses. Main's game with no
winner (seed 7, sides 1 and 2, map 0) ends on the branch at 49071 ticks with house 1 the winner.
The branch's game with no winner is seed 10, sides 0 and 2, map 2, where main's ends at 40002
ticks with house 1 the winner. In it house 0 is down to three buildings and one aircraft, with
no factory, while house 1 has 31 buildings and a running economy (14471 shots, 516 kills). So
the winner does not finish the last pieces off; it is not a pile-up of units held in the attack
state.

## Seams

- **The airstrike** (S-0107 claim 11's airstrike reset begin and repeat ending, scenario M8 b):
  no airstrike sequence is built.
- **The scatter from damage in hit reactions** (S-0215 claim 19, S-0208 claim 15 requests A and B,
  and the aircraft's scatter-first fall-back of claim 16): not built in `damage::reactions`.
- **Unplaced target changes**: Q-2616.
- **The berserk path of intake 100** (`damage::berserk_reactions`, merged from main at
  5061dee). This path calls `process::fall_back` and `fear::damaged` directly. r118 changes three
  things in `damage::reactions`, and none of them applies here, so the berserk path is left as
  intake 100 built it:
  - Retaliation before the no-threat test: a berserk hit makes no retaliation and no anger
    change (S-0302 r43 claim 10).
  - The result check (*unchanged*, *now yellow* or *now red*): a berserk hit reports *unchanged*.
  - The no-threat test before fear: the berserk path already orders the fall-back first and then
    fear.

  The berserk entry drops the target from another actor's turn, so the attack state ends lazily
  in the firing turn with no request. This is the Q-2616 interim (its question 2).
