# M8 seam sweep 7a: rows marked partly, untested or seam in intakes 112 to 116 and three steps

Brief: `.claude/worktrees/M8-SEAMS-7-BRIEF.md`, part 7a. Method and row form follow
[seams-6.md](seams-6.md). Rows read: every row of `intake112.md` to `intake116.md`,
[scatter-damage.md](scatter-damage.md), [air-retaliate.md](air-retaliate.md) and
[ai-stalled-harvester.md](ai-stalled-harvester.md) marked partly, not tested, untested, by
construction, by inspection, built with no test, not built or seam, and the *Seams* text of the
same steps' rows in `docs/step-ledger-m8.md`. Each was first checked for a later close (a later
intake, a spec revision, its request's answer).

**Counts.** 41 rows read: 15 closed earlier, 0 blocked by Q-2097 or Q-2597, 10 tested, 0 built,
16 left. A row restated in two places of one ledger counts once (intake 113's underground loss;
air-retaliate's non-jumpjet crash). The step ledger's rows for these steps restate their
ledgers' rows and requests; every request they name is answered (Q-2750, Q-2781, Q-2782, Q-2792,
Q-2793, Q-2801, Q-2802, Q-2811, Q-2821, Q-2822, Q-2831, Q-2841), so they add no row.

**Result.** 10 rows tested through the real order and tick path, in 10 new tests. Every test
agrees with the code: no code changed. No simulation change: no hashed or replayed state, no format
bump (save 125, AI game 91, wire 12), goldens unaffected, lanes skipped (tests only). No spec
request filed (Q-2916 to Q-2920 unused). One row is `unmeasured` (S-0215 claim 16's order against
the scatter requests); no spec scenario covers it, so no wishlist row is added.

**Overlap with part 7b.** No row here is the same claim as a row of intakes 117 to 122. The
nearest are intake 120's two S-0309 claim 31 Open-question seams (whether the unit left standing
holds up the new unit's exit track; which unit a factory with several places sends out first),
beside intake 116's apron-request row: different questions, all left, none tested here.

## Built

None.

## Tested

| # | Ledger | Spec | Claim or row | State before | Test and result |
|---|---|---|---|---|---|
| 1 | intake 112 | S-0309 r124 | claim 31, a vehicle deploying in place keeps its guard spot | partly | `seams7a_a_vehicle_deploying_in_place_keeps_its_guard_spot` (`claims_s0309_r124.rs`): a trigger owner change of an ATANK in the middle of deploying, and of one undeploying, keeps the spot; a mobile one of the same type forgets it. Code agrees. |
| 2 | intake 112 | S-0309 r124, S-0215 claim 11 | claim 31, a vehicle factory still in *unload* gets no fall-back | partly (by construction) | `seams7a_a_factory_still_in_unload_gets_no_fall_back` (`claims_s0309_r126.rs`): with a positive-delay idle range the start of *unload* used the mark, so the change keeps *unload* current with *guard* queued and requests no idle body; the control, a factory whose *guard* starts, gets the idle-body request. Code agrees. |
| 3 | intake 112 | S-0309 r124, S-0601 claim 9 | claim 31, an object off the map gets no fall-back | partly (by construction) | `seams7a_a_release_off_the_map_gets_no_fall_back` (`claims_s0601.rs`): released off the map, the engaged mark and the attack-move memory stay, nothing is resumed, no common effects, the guard request is made; on the map the refresh resumes the remembered destination. Code agrees. |
| 4 | intake 113 | S-0506 r37 | claim 11, the destroying hit of a sinking vessel (R7 (c)) | tested, the vessel not on its own | `seams7a_r7_c_the_reset_comes_at_the_destroying_hit_of_a_sinking_vessel` (`claims_s0506_r37.rs`): a naval synthetic type over water, a lethal hit; the vessel stays in play sinking and the controller resets at the hit, the missile released with F8's cell. Code agrees. |
| 5 | intake 113 | S-0506 r37 | claim 12, the sale of a building target (R21 (d)) | partly | `seams7a_r21_d_the_sale_of_a_building_target_resets_at_its_removal` (`claims_s0506_r37.rs`): through `Order::Sell`, the missile prepared after the build-down starts; still preparing in the tick before the removal, reset at the removal. Code agrees. |
| 6 | intake 115 | S-0411 r57 | claim 20, every state the same | partly | `seams7a_claim_20_the_icon_tip_is_the_same_building_and_on_hold` (`client_sidebar_tip_tests.rs`): the icon's tip text idle, building (`Order::Build`) and on hold (`Order::Hold`) is the same. The done, placing and superweapon states are not reached. Code agrees. |
| 7 | intake 116 | S-0309 r126, S-0601 claims 9 and 13 | claim 31, release then domination: a reset at each change, the second only if the owner changes | partly | `seams7a_release_then_domination_resets_at_each_change` (`claims_s0601.rs`): dominated by its original owner, only the release resets (the resumed destination kept); by another house, the second reset clears it and nothing resumes it. Code agrees. |
| 8 | intake 116 | S-0215 claim 11, S-0309 r126 | a turreted building's body range sets the mark while *selling* | partly | `seams7a_a_turreted_building_sold_ends_its_sale_at_the_next_turn` (`claims_s0309_r126.rs`): a turreted building changed in its build-down requests its idle body and is removed at its next turn, the build-down cut short. Code agrees. |
| 9 | air-retaliate | S-0309 claim 18 | a harvesting slave never retaliates | built, tested (slave master only) | `seams7a_claim_18_a_harvesting_slave_never_retaliates` (`claims_s0215_air_retaliate.rs`): the slave out on the map answers `MayNot`; the same type with no master answers `Done`. Code agrees. |
| 10 | scatter-damage | S-0215 claim 16 | the no-threat test against the scatter requests | built, tested in part (`unmeasured`) | `seams7a_claim_16_request_a_comes_before_the_no_threat_fall_back` (`claims_s0215_scatter_damage.rs`): request A is decided on the state before the no-threat fall-back, request B after it, and the fall-back leaves the guard spot as destination; without the no-threat property A's destination stays. Synthetic only. Code agrees. |

## Closed earlier (no work)

| Ledger | Row | Closed by |
|---|---|---|
| intake 112 | claim 31 Open questions 1 to 4 and three build cases (Q-2801) | intake 116 (S-0309 r126) |
| intake 112 | Open questions 32 and 33 (Q-2802) | intake 118 (S-0501 r167) |
| intake 112 | Seams: Q-2792 not answered | intake 114 (S-0501 r154 claim 52) |
| intake 113 | claim 11, a controller target lost while a pending target waits (Q-2811 interim) | intake 117 |
| intake 114 | Q-2821 (claim 18 against claim 35) | intake 118 (S-0501 r172) |
| intake 114 | Q-2822 (the same destination given again) | intake 121 (S-0209 r76) |
| intake 114 | the Q-2801 and Q-2802 interims | intakes 116 and 118 |
| intake 115 | Open question 11, other insertion-point layouts (Q-2831) | intake 117 |
| intake 116 | claim 31, a unit the factory has not sent out (Q-2841 interim) | intake 120 (S-0309 r129) |
| scatter-damage | aircraft retaliation seam | the aircraft-retaliation build and intake 114 |
| scatter-damage | Q-2750 | intake 114 (S-0208 r58) |
| scatter-damage | Q-2781 | intake 112 (S-0207 r85) |
| scatter-damage | Q-2782 | intake 114 (S-0215 r137, S-0208 r58) |
| air-retaliate | Q-2792 (an aircraft on its pad) | intake 114 (S-0501 r154 claim 52) |
| air-retaliate | Q-2793 (the height and mark tests) | intake 114 (S-0312 r66) |

## Blocked by an open request

None of these rows waits on Q-2097 or Q-2597.

## Left (no test written)

| Rows | Ledger | Spec | Claim or row | Why left |
|---|---|---|---|---|
| 1 | intake 112 | S-0501 r150 | claim 8, the hand-over covers claims 18 and 29's destinations | claim 29's hand-over is tested by `ac33_a_an_unarmed_aircraft_in_sticky_lands_short_of_its_order` (the cell it is over, no second take-off); claim 18's is not tested on its own. When the test is made is `unmeasured`, and no scenario moves the heading point or the flight point between a hand-over and a landing start, so no test can tell a hand-over test from one at every update |
| 1 | intake 112 | M-0024 | the row "every judged hit naming no attacker classed neighbouring destination" | left to the lead (S-0215 B6 (f1) allows the engine's class; the local test still asserts the row) |
| 1 | intake 113 | S-0506 r37 | claim 11, the loss on going underground (R7 (e)) | no underground locomotor is built (only tunnel land types) |
| 1 | intake 113 | S-0506 r37, r40 | claim 14, which destination comes first (Q-2748 questions 2 and 3) | S-0506 r40 still lists them as Open questions; the interim (`design: Q-2748 interim`) stays, tested by `q2748_interim_the_release_cell_given_at_the_re_order_fixes_the_flight` |
| 1 | intake 114 | S-0501 r154 | claim 52, AC34 b1: the second trigger's hand-back | tested up to the second trigger; the hand-back is the existing restore at the target's loss (S-0215 claim 10, T2 row 1) |
| 1 | intake 114 | S-0215 r137 | B6 f2 and the arena parts of f1 | arena-scale records (M-0024, M-0025) |
| 1 | intake 115 | S-0411 r57 | claim 20, *P* in its form in parts for a building whose cost pays for units | no synthetic type pays for a unit that comes with it; a test through `price` alone would be circular |
| 1 | intake 115 | S-0411 r57 | claim 20, a superweapon icon's text | the bar game's house holds no superweapon icon |
| 1 | intake 115 | S-0411 r57 | claim 20, the price follows a cost bonus gained or lost | the fixture has no cost-bonus building |
| 1 | intake 116 | S-0309 r126 | claim 31, the grand opening's other effects | S-0309 Open questions; none are made |
| 1 | intake 116 | S-0309 r126 | claim 31, a naval vehicle factory's rally cell | S-0309 Open questions (where the cell lies is not established) |
| 1 | intake 116 | S-0309 r126 | claim 31, what the factory's apron requests do to a unit not sent out | S-0309 Open questions; nothing is built |
| 1 | scatter-damage | S-0215 | a refused override | unreachable: only an actor in *selling* refuses one, and no mobile actor sells |
| 1 | scatter-damage | M-0001, M-0003, M-0021 | the arena rows still failing | arena-scale records |
| 1 | air-retaliate | S-0312 claims 11, 12 and 15 | passengers kept for a vehicle's crash; crashing vehicles other than jumpjets | a vehicle still starts a crash only through `jump::start_crash` (jumpjet movement); aircraft crash through `air::start_crash`. Unchanged |
| 1 | ai-stalled-harvester | own AI | a unit a factory has not sent out when the factory changes owner | still a seam, its reason updated: since S-0309 r129 (intake 120) the unit keeps its behaviour and stays, losing the factory's contact only at the factory's next release. Whether `scripted::idle` counts it depends on that behaviour; not traced in the lane. AI design, no claim behind it |
