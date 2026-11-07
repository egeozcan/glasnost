# The scatter from damage (M8 build; S-0215 claim 19, S-0208 claim 15)

Brief: `.claude/worktrees/M8-SCATTER-DAMAGE-BRIEF.md`. Specs: S-0215 r132 claims 16 to 19 (with
Terms *Houses in our engine* and *Test-only computer-controlled player house*), S-0208 r52 claims
6, 11 to 13 and 15 (with Terms *May retaliate*, *Target, destination*, *Locomotor moving*,
*Tether link*), S-0302 r43 claim 10 and its Changes for other specs (S-0208 claim 15), S-0309
claims 18 and 25 part h, S-0501 claim 49. No `requires:` line names any of these claims.

Seam sweep 6 (`seams-6.md`, Left: "the scatter from damage, requests A and B") and intake 99
(`intake99.md`, Seams) left the hit's scatter requests unbuilt: `damage::reactions` sent none.

**Result.** `damage::reactions` now sends them in S-0215 claim 16's order: anger, retaliation,
the scatter from damage (request A for infantry), the no-threat test, request B (infantry), fear;
the crush response follows in `damage::apply`. `acquire::retaliate` now answers what the
retaliation came to (`Retaliation::MayNot`, `Held`, `Done`), which is S-0208 Terms' *may
retaliate* and *does retaliate*. The berserk path (`damage::berserk_reactions`) sends request B,
not request A. A vehicle's scatter request without a threat point (`scatter::vehicle_request`) now
leaves a seized vehicle's destination unchanged (S-0607 claim 9); **Q-2781** asks what the branch
with a threat point does for a seized vehicle (left as it was). No quirk is named by these claims
(Edge cases of S-0215 and S-0208 checked; `every_spec_quirk_has_a_switch` passes), so no switch
and no C7 verdict is owed. Save format 113, AI game format 78. No golden moved.

## Order within a hit (`damage::reactions`)

1. The hit's own S-0302 effects: health, the damage stage, death.
2. Anger: S-0309 claim 16.
3. Retaliation: S-0215 claim 18, S-0309 claims 18 and 25 part h. Its answer:
   - `MayNot`: a condition other than F7's distance rule fails (S-0208 Terms *may retaliate*),
   - `Held`: it may retaliate but does not: F7 holds back a player-house actor that cannot reach
     the attacker, or the override is refused and keeps its slots, or it is an aircraft (seam
     below),
   - `Done`: the override installed the attacker as target.
4. The scatter from damage (S-0215 claim 19 for a vehicle or an aircraft, request A of S-0208
   claim 15 for infantry): one forced request with no threat point, not move-anywhere, through
   `scatter::request` (acceptance by S-0208 claim 11, S-0207 claim 16, S-0501 claim 49).
5. The no-threat test: S-0215 claim 16 (for an aircraft, on the behaviour the scatter's fall-back
   left).
6. Request B (infantry): the attacker's position as threat point, neither forced nor
   move-anywhere.
7. Fear: S-0107 claim 16.
8. The crush response: S-0215 claim 39, in `damage::apply`.

## Claim parts

| Claim part | Status | Where / tests |
|---|---|---|
| S-0215 claim 19, case 1: known attacker, may retaliate, does not; no target and no destination when hit; player-scatter or scatter ability; owner and behaviour do not matter | built, tested | `damage::damage_scatter_sent`; `claim_19_b6_d_case_1_a_vehicle_hit_from_beyond_f7_scatters`, `claim_19_b6_d_control_1_player_scatter_off_sends_no_request`, `claim_19_case_1_the_scatter_ability_stands_in_for_the_setting` |
| S-0215 claim 19, case 1: a retaliating actor is never sent it | built, tested | `claim_19_b6_d_control_2_a_retaliating_vehicle_is_never_sent_case_1`, `claim_15_a_retaliating_soldier_is_never_sent_request_a` |
| S-0215 claim 19 and S-0208 claim 15: "takes damage and survives it" (a mobile actor; a hit reporting *now dead* or *post-mortem* destroys it, even a sinking vessel left in play at 1 health) | built | `damage::reactions` (`survived`); every test's hit is one the unit survives |
| S-0215 claim 19, case 2: no known attacker or may not retaliate; not an aircraft; scatter property; no tether; locomotor not moving; no target, no destination | built, tested | `claim_19_b6_b_case_2_needs_the_scatter_property`, `claim_19_case_2_damage_without_a_known_attacker`, `claim_19_case_2_a_target_a_destination_or_a_tether_stops_it` (a target, a destination while driving, the locomotor moving with the destination cleared, a tether, and the control), `claim_19_case_2_never_sends_an_aircraft_the_request` |
| S-0607 claim 9 against the scatter from damage: a seized vehicle ignores the request's destination | built, tested | `scatter::vehicle_request` (branch without a threat point); `a_seized_crusher_ignores_the_destination_and_queues_nothing` (claims_s0210_r29.rs, which now also covers the scatter from damage); the branch with a threat point is Q-2781 |
| S-0215 claim 19, case 2, In our engine: the last condition is player-scatter or the ability for every player house, AI houses included; the neutral and special houses and a test-only computer-controlled player house keep the computer-controlled condition | built, tested | `claim_19_b6_b_case_2_owner_rule_with_player_scatter_off` |
| S-0215 claim 19, Original only (computer player houses always meet the last condition) | left out, as the brief says | — |
| S-0215 claim 19: request options (forced, no threat point, not move-anywhere) | built, tested | the same tests; S-0207 claim 16's refusals and destination are `scatter::vehicle_request`'s |
| S-0215 claim 19 / S-0501 claim 49: the aircraft's answer (a fall-back to rest when its current behaviour has the scatter property) | built, tested | `claim_19_an_aircraft_answers_the_scatter_from_damage_by_a_fall_back` |
| S-0215 claim 16, Order within the hit: anger, retaliation, scatter from damage (A), no-threat test, B, fear; the aircraft's scatter-first fall-back | built, tested in part | order in `damage::reactions`; the two-draw tests give A before B; the no-threat test against the scatter requests is `unmeasured` in the spec and not tested here |
| S-0208 claim 15, request A first case | built, tested | `claim_15_w7_g_first_case_of_request_a` (g1, g2, g3) |
| S-0208 claim 15, request A second case and the owner rule (Original and our engine) | built, tested | `claim_15_w7_h_second_case_an_ai_house_and_the_neutral_house` (h2, h4, h6) |
| S-0208 claim 15, request B (attacker's position, not forced; only fraidycats act on it) | built, tested | the W7 g and h tests (refused), `claim_15_w7_j_w9_c_both_requests_on_a_side_position_take_two_draws` (accepted) |
| S-0208 claim 15: A's draw then B's; claim 3's draw between them on a position of class centre; the walking target chosen at once (claim 6) is B's scatter centre cell; B's destination replaces A's | built, tested | `claim_15_w7_j_w9_c_both_requests_on_a_side_position_take_two_draws`, `claim_15_w7_j_on_the_centre_claim_3_draws_between_a_and_b`: a real shot through the tick path for both requests' acceptance and B's destination standing; the draws counted over the hit itself (`damage::apply`) against W9 part c's reference (the property off), whose own draws are the background (S-0208 W9, *Background draws*), since over the whole tick a walking unit and a standing one take different background draws |
| S-0208 claim 15: the tether-link condition of the second case | built, tested by a set tether | `claim_19_case_2_a_target_a_destination_or_a_tether_stops_it` (the spec rates it `unmeasured`) |
| S-0302 r43 claim 10: a berserk hit sends request B (known attacker), not request A | built, tested | `damage::berserk_reactions`; `claim_15_a_berserk_hit_sends_request_b_only` |

Tests: `crates/deterministic/sim/tests/claims_s0215_scatter_damage.rs`. Changed:
`claims_s0210_r29.rs`, `a_hit_naming_no_attacker_gives_no_response` (S-0215 claim 39): the
computer-owned crusher of that scene, hit with no attacker in *guard* (scatter property), is now
sent case 2 of claim 19 and gets a cell destination; the test now checks that the crush response
(the infantry as destination, *move* queued) is not made, instead of no destination at all.

## What the berserk path sent before

Nothing: `damage::berserk_reactions` (intake 100) made the no-threat fall-back, the fear change
and the harvester announcement, and its doc named request B as a seam. S-0302 r43 claim 10 and its
note for S-0208 claim 15 say request B follows a berserk hit with a known attacker, after the
fall-back and before the fear, and request A does not. Built so.

## Q-2750

Q-2750 (open) asks about the extra locomotion of a walker standing with no walking target whose
stored path a scatter drops. Request A reaches only a unit with no destination (both cases), and
case 2 only one whose locomotor is not moving. A walker in the middle of a walk has a destination,
so request A does not reach the Q-2750 walker of the request's examples. If a walker with no
destination still held a stored path, `scatter::request` would apply its current workaround (the
state read before the request, no extra locomotion); this build does not change it. Request B has
a threat point, never uses the nearby-cell search and never gives the extra locomotion.

## Lanes (main ea9d5f5 against the branch)

- The M6c, M7a and M7b sweeps (seeds 1 to 16, reference and play-fixes): identical apart from
  timings.
- `glasnost-ai --include-ignored`: 263 passed, 0 failed on main and on the branch.
- The AI quick lane (seeds 7 to 10, maps 0, 2 and 8, 36 games): every game over with a winner on
  both; every winner unchanged; 2 tick counts moved (seeds 7 and 10).
- The arena (`arena --record all --threads 2`), failing on main and on the branch:
  - M-0001: 10 of 96 to 12 of 96 (new: LTNK-APOC, LTNK-E1, both in **Q-2782**; win counts move
    in other pairings; side-wise unanimous outcomes reproduced 157 to 155 of 171);
  - M-0002: 2 of 24, unchanged (INIT-E2's win count moves);
  - M-0003: 1 of 35 to 2 of 35 (new: MTNK-APOC, in **Q-2782**);
  - M-0019: 0 of 6, unchanged;
  - M-0020: 1 of 25, identical;
  - M-0021: 8 of 50 to 11 of 50 (new: part B's "c ≥ 3 in every judged shot", part B's "hits a
    judged shot owns" and the reading-points row on a part B death's shot whose later hit names
    no attacker; one part B run's G-dead count moves). All of these are in **Q-2782**.

  Traced (a temporary local probe, not committed), with M-0001 LTNK-E1 (side A, seed 0, its
  winner flips) and M-0003 MTNK-APOC (side A, seed 0): every scatter from damage (or request A) sent in
  those two games is case 2 of claim 19 (S-0208 claim 15's second case), sent to a free unit of a
  test-only computer-controlled arena house (S-0215 Terms). The answer that let case 2 through is
  a may-condition stated by S-0215 claim 18 or S-0309 claim 18, read from the installed data, or
  a hit naming no attacker. Before this build these hits sent nothing; now the unit takes a nearby
  cell as its destination, and the game drifts from there. So the changes follow from the claims
  as written, not from a reading of ours. The details have local values and stay out of this
  file.

  M-0021 part B, F on side B, seed 1001, was probed afterwards the same way. Every request of that
  run is case 2, sent to a free unit in *hunt* of a computer-controlled arena house. Each comes
  from a hit naming no attacker, or from the same claim 18 answer (one with an allied attacker
  too). In both of that run's judged shots with *c* < 3, the target *X* is a G vehicle that had
  been sent the request a few ticks before. It took the next cell toward the F group, and two of
  its G neighbours were sent it too, near the same ticks, with one shared destination cell. So
  fewer cells around *X* held a G vehicle and no F vehicle.

  This is in tension with the record. By the claim's Original-only note, a computer house always
  meets case 2's last condition, so the original would send the request at least as often as we
  do, yet the record has *c* ≥ 3 in every judged shot. **Q-2782** asks which reading is right:
  whether case 2 reaches a free vehicle in *hunt* of a computer player house in the original, or
  whether the original's hunting vehicles do not stand free (no target, no destination) when hit,
  as ours do between one target and the next. No code was tuned toward the lane.

## Seams

- **The arena records (Q-2782).** Under the claims as built, case 2 reaches the test-only
  computer-controlled arena houses' free units in *hunt*. As a result M-0021 part B (*c* ≥ 3, the
  hits a judged shot owns, a reading-points row, one G-dead count), M-0001 LTNK-APOC and LTNK-E1,
  and M-0003 MTNK-APOC now fail. Q-2782 asks which reading is right; the build stands meanwhile.
- **Aircraft retaliation.** Our engine builds no retaliation for an aircraft (`acquire::retaliate`
  never overrides one). An aircraft that may retaliate is answered `Held`, so with the
  player-scatter setting or the scatter ability, and with neither target nor destination, it is
  sent case 1's request even where the original would retaliate (within its weapon's reach, or
  within F7's distance, or for a computer-controlled owner). The request then makes it fall back
  to rest when its current behaviour has the scatter property (S-0501 claim 49). Before this
  build, `retaliate` did not judge an aircraft at all.
- **A refused override.** `quirks.refused_override_retargets` off makes a refused override change
  nothing, so the actor does not retaliate (`Held`) and case 1 can follow. An override to *attack*
  is refused only for an actor in *selling*, which no mobile actor is, so the case never reaches
  the scatter from damage.
