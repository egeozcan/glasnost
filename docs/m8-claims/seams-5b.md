# M8 seam sweep 5b: rows marked partly, untested or seam in intakes 73 to 95

Brief: `.claude/worktrees/M8-SEAMS-5B-BRIEF.md`. Method and row form follow [seams-3.md](seams-3.md)
and [seams-4.md](seams-4.md). Rows read: every row of `intake73.md` to `intake95.md` (there is no
`intake82.md`: that intake stopped with no commits and intake 81 took it over) that says partly,
not tested, untested, by construction, by inspection, built no test, not built or seam, and the
"Seams left" of seam traces 3 to 5 in `docs/step-ledger-m8.md`. Sweep 5a covers intakes 51 to 72.

**S-0501 left to intake 96.** During the sweep the lead pushed S-0501 r119 and r120, which answer
Q-2506 to Q-2508, and gave every S-0501 row to intake 96. This sweep therefore builds and tests no
S-0501 claim. Every S-0501 row is under *Left* with the reason "S-0501 r119/r120, intake 96",
including rows an earlier intake had closed and rows earlier blocked by Q-2506 to Q-2508. Before
that message the sweep had written three S-0501 tests (claim 29's unarmed branch at cruise height,
AC29 part f (5), and AC31 part b run 4, all passing on r111) and a request on claim 31's
"*move* for a type that carries vehicles". All four were removed before commit and are not on
this branch.

**Result.** 106 rows read. Three of them split into a tested part and a part left, so the counts
below are of 109 row parts: 32 closed earlier by a later intake or spec revision, 6 blocked by an
open request, 6 tested here, 0 built, 3 sent to the 2 requests filed here, and 62 left (22 of them
S-0501 row parts for intake 96). No test found the code differing from an approved claim, so the
simulation is unchanged: no hashed state, no format bump (save 98, AI game 61, wire 12), and
goldens and lanes are unaffected. Requests filed: **Q-2566** and **Q-2567** (Q-2568 to Q-2575
unused).

## Built

None. The one "not built" claim in scope outside S-0501 is not exact enough to build. S-0215
claim 11's infantry "in the middle of firing" names a firing state that no spec defines (Q-2566).

## Tested

| # | Spec | Claim or scenario part | State before | Test and result |
|---|---|---|---|---|
| 1 | S-0214 r80, S-0513 r108, S-0301 r70 | Edge cases, *Attack offered over a reason of the moment*: with `moment_reason_attack_cursor` off, part (e) passes over the held-temporal-target *rearming* of rank 4, so a protected target's *illegal* (S-0513 claim 28) fails it; on, the *rearming* decides (intake 89's seam) | untested ("by reading the code") | `claims_s0513_r89::r108_part_e_passes_over_the_held_temporal_rearming_only_with_the_switch_off` (a computer-controlled firer, whose check reads protection, beam locked on a protected BUGGY): the check answers *rearming*, an *illegal* holds at a lower rank, and the action and click give *attack* only with the switch on. Code agrees. |
| 2 | S-0214 r80 | Edge cases: off, part (e) passes over a building's own *impossible* (S-0405 claim 9, table T1) (intake 90) | untested ("same gate") | `claims_s0214_r80::off_a_switched_off_defence_over_an_illegal_target_fails_part_e` (DEFT, a twin of TOWER with the zero-versus weapon W, switched off, over TARGET): *impossible* holds part (e) with the switch on and fails it with the switch off. Code agrees. |
| 3 | S-0214 r78 | claim 6 (e): S-0306 claim 23's water row ranks above the rearm countdown and below the materialising *rearming* (intake 85) | untested ("same block") | `claims_s0214_r78::the_water_row_outranks_the_countdown_but_not_the_materialising_rearming` (NAVALER, naval mode 6, against a vehicle on a water cell): *illegal* with no countdown and with one, no attack. Phasing in, *rearming*, with the attack offered and ordered only with the switch on. The same vehicle on dry land is not refused. Code agrees. |
| 4 | S-0214 r78 | claim 6 (e): the other early *rearming* answers ahead of the water and land rows, the temporal hold among them (intake 85) | partly | The temporal-hold *rearming* is test 1 above, against the protected target's *illegal*. The spawner answer is left (below). |
| 5 | S-0513 r115 | claim 39a *Drops* and *Departure*, claim 51: an aircraft member with a beam attached drops its target and destination in its departure and off-map turns (intake 81) | partly ("no aircraft member test") | `claims_s0513_r89::claim39a_an_aircraft_member_with_a_beam_drops_in_its_departure_and_off_map_turns` (LANDP, a landable twin of PDPLANE standing on the block): dropped in t_L + 62, 63 and 64. The test reads S-0513's member rules only, not an S-0501 claim. Code agrees. |
| 6 | S-0513 r115 | claim 39a *The firer*: "its rearm countdown runs as in any turn" while the member waits to land (intake 81) | partly ("no test reads the countdown") | `claims_s0513_r89::claim39a_the_firers_rearm_countdown_runs_while_the_member_waits_to_land` (a CART whose countdown ends halfway through the wait): the countdown is neither restarted nor held, it runs out before the appearance, and there is no shot. The firer keeps the member. Code agrees. |

## Requests filed

| Request | Spec | Rows |
|---|---|---|
| Q-2566 | S-0215 r104 claim 11, S-0107 r31 claim 11, S-0513 r115 claim 39a | intake 94: SW12 l9b (the release at an *attack* processing before *unload* starts), and the firing state ending after both of the first update's per-tick starts. The infantry "in the middle of firing" names a firing state that no spec defines, gives no start and gives no end for a unit that keeps its target. Answered by S-0215 r118 claim 11, S-0107 r40 claim 11 and S-0513 r122 claim 39a (the attack state); built in intake 99 ([intake99.md](intake99.md)): l9b is dropped by r122, and the firing state's endings are placed. |
| Q-2567 | S-0301 r71 claim 9, S-0513 r115 claim 39a, S-0405 table T1 | intake 84: a building firer's outranking *illegal* or *impossible* drops the member and starts *guard*. S-0301 does not order T1's answers against the held-temporal-target *rearming*, so the test would assert an order no claim gives. |

## Closed earlier

| Rows | Closed by |
|---|---|
| intake 74: claim 29's materialising holder (and the Seams item for intake 79) | intake 79, `p10a_a_materialising_holder_keeps_its_refused_target_until_the_phase_in_ends` |
| intake 74: runs p8 to p10, the aircraft items of S-0215 claims 1, 7, 12, 19, 26 and 28, the phasing-in exclusions, the erasure update, the pad-bound scenario, the aircraft's start-of-turn drops; the Seams items for intakes 75 and 76 | intakes 75 (S-0513 claim 39a) and 76 (S-0501), both merged |
| intake 75: claim 39's widening of S-0309 claim 29, claim 51's allied drop (S-0309 claim 25 k), S-0610 claim 4's berserk entry | intake 74's S-0309 and S-0215 revisions |
| intake 75: no being-erased sparkles after the departure | intake 81, the l6 sparkle record |
| intake 75: buildings and deployed infantry take no approach step | intake 84, the r91 building and deployed-infantry tests |
| intake 75: *Turns cut short*, the phasing-in vehicle's drop | intake 81's drop tests. The check's own answer there is *rearming*, so the check-tick part cannot be seen |
| intake 78: the ground layer from the launch and after the hold (Q-2346, both questions) | Q-2346 answered (intakes 83 and 89) |
| intake 78: the three claim 20 seams (led shots, void arcing launch, projectiles below ground) | intake 79 (S-0306 r64) |
| intake 79: the materialising *rearming* in the attack rule (Q-2356) | Q-2356 answered (intake 85) |
| intake 81: a member protected or cloaked at its departure | intake 89, `r108_claim39a_a_firer_keeps_a_protected_or_hidden_member_off_the_map` |
| intake 83: the warp's membership (claims 4 and 34) | intake 84; its Q-2419 answered (intake 89) |
| intake 84: jumpjet member listed in a source cell above its surface (Q-2419) | Q-2419 answered (intake 89) |
| intake 88 Seams: the mind-control hold and the AI attack on an unhittable building | seam trace 4 (spec behaviour; AI-side fixes) |
| intake 88 Seams: S-0309 r113 | intake 89 |
| intake 89: Q-2466, Q-2467 and Q-2468 | answered (intake 94) |
| seam trace 3: the stuck infantry unit (Q-2396) | Q-2396 answered by S-0209 (intake 88) |

These 16 rows of the table hold 32 row parts.

## Blocked by an open request

| Request | Rows |
|---|---|
| Q-2526 | intake 94: claim 17's rest after the hold for a type that cannot land; P46 run 2's post-hold rest |
| Q-2536 | intake 93: part B's outcome table; seam trace 5's part B spread |
| Q-2546 | intake 93: F's first-shot distance; seam trace 5's first shot |

## Left (no test written)

| Spec | Claim or scenario part | Why left |
|---|---|---|
| S-0501 (intake 76) | claims 5 and 6: a flight point that differs from the destination object's cell; claim 49: the take-off "at once, in that processing", a cleared destination keeping the flight point; claim 50: AC31 part b runs 2 and 4, the own-cell no-result case, slowdown 0, no dock types, no building | S-0501 r119/r120, intake 96. |
| S-0501 (intake 76, closed by intakes 74 and 80) | claim 24, *enter* without contact (Q-2326); claim 49, the scatter from damage | S-0501 r119/r120, intake 96 (closed earlier; left for the new revision to restate). |
| S-0501 (intake 80, closed by intake 86) | claim 24: the wait of one processing; the touchdown and pad-bound rows (Q-2366) | S-0501 r119/r120, intake 96 (closed earlier; left for the new revision to restate). |
| S-0501 (intake 86) | claim 24: the move on to the docking stage needs a pad cell (Q-2437); the approaching stage gives the pad back (Q-2436), both closed by intake 92; claim 31, *move* for a type that carries vehicles (not built) | S-0501 r119/r120, intake 96. |
| S-0501 (intake 92) | claim 24, the docking stage in contact in flight (Q-2506); claim 8's landing point (Q-2507); the order of the run-away notices (Q-2508); claim 29, the unarmed aircraft at cruise height and a team member there; claim 31, *move* for a type that carries vehicles (not built); claim 46, AC29 part f (5) (not run) | S-0501 r119/r120, intake 96 (r119/r120 answer Q-2506 to Q-2508). |
| S-0701 (intake 73) | claim 17, scenario D3 | Oracle-side observation, not a rule of our engine. |
| S-0309 (intake 74) | claim 29, a second turn in a check tick | No setup gives a holder a second turn in a tick. |
| S-0215 (intake 74) | claim 32's periodic check note; F6's set-up exception | By construction: no holder holds a target during our set-up. Nothing to observe. |
| S-0513 (intake 75) | claim 39a lock routes: a fragment of a non-temporal bursting projectile; an invisible shot delayed past t_L; unchecked building and aircraft shots | No synthetic fragment weapon, and the shot routes are those of S-0306, S-0215 claim 32 and S-0501 claim 12 as already tested. No member-specific setup. |
| S-0513 (intake 75) | claim 39a: more than one lock before the departure; a lock after the departure (firer already on the beam, zero or negative steps) | S-0514's beam rules, tested in `claims_s0514*`. The member adds no rule. |
| S-0216 (intake 77) | claim 8, aircraft-kind groups | No record has one. |
| S-0411 (intake 78) | claim 19 *Pixels*: no change by any global effect | A client drawing property, by construction. No simulation test. |
| S-0301 (intake 79) | claim 9: ranks 1 and 3, the tube transit, the slave-master firer, the melee link | No state in our engine. |
| S-0513 (intake 81, intake 84) | claim 39a: an aircraft firer | "The aircraft specs'". No aircraft spec says it yet (S-0501 r119/r120 is intake 96's). |
| S-0102 (intake 83, intake 84) | a member moving from one bridge deck to another | Open question in S-0102 and S-0306. |
| S-0514 (intake 84) | claim 18: a building with damage-fire positions | Damage-fire positions are not imported. |
| S-0513 (intake 84) | claim 39a *Sparkles*: an aircraft member's removing turn | Same path as a vehicle member. A removing turn needs a completed erasure while waiting to land, which the aircraft set-up of test 5 does not give. |
| S-0513 (intake 84) | claim 39a: a SAM-flag building keeps an aircraft member above world height 0 | No setup puts an aircraft member above 0. |
| S-0513 (intake 84) | claim 39a: a lock while the member waits to land, and the warp's own steps | Open question of S-0306 claim 17 and S-0513 claim 39a. |
| S-0209 (intake 84) | drop-pod and tunnelling movement | Not built. Their no-cell listing comes with them. |
| S-0214 (intake 85) | claim 6 (e): the other rank 9 tests and rank 12 during the countdown; the drop's missed or cut-short turn, the countdown ending out of ammunition, an infantry holder | The same check ranks and held-target check, already tested for one case each. |
| S-0214 (intake 85) | claim 6 (e): a falling infantry unit that is phasing in | Open in S-0301 claim 9 and S-0310. |
| S-0214 (intake 85) | Terms *movable*: not being erased, not a slave unit; the phasing-in actor with a beam; the aircraft actor's drops; the cloak drop; the spawner *rearming* | Not being erased and not a slave unit are built in batch S4 (#209, `targeting::movable`; `claims_s0214_slaves.rs`). The others are other specs' built rules (S-0513 claim 39a, S-0501 claim 32, S-0516 claim 14) not tested in combination; the aircraft actor's drop is S-0501's (intake 96). |
| S-0506 (intake 86) | the rocket's docking-point cases | Seam in `rocket.rs`, not in this delta. |
| S-0202, S-0216 (intake 87) | S-0202 claim 14: a campaign mission's houses; S-0216 *Removal*: kills that bypass target search | No campaign start; no such kill reaches a unit at start 3 in the arena. |
| S-0209 (intake 88) | claim 9's expiry rows while the walker holds the occupant; a diagonal step past a building's corner | `unmeasured`, with no setup in the spec; the corner step is a note for the lead, not a claim. |
| S-0306, S-0513 (intake 89) | S-0306 claim 17: the aircraft pass considers it; S-0513 SW12 l9's deploy and the *sticky* row through orders | By construction (pass 0 takes every aircraft); our *sticky* is processed as *guard*. |
| S-0214 (intake 90) | Edge cases: the magnetic-beam *rearming*, the held spawner's *impossible*, the other weapon's live effect | The same `!pass` gates as tests 1 and 2, each needing its own subsystem set-up. |
| S-0513 (intake 94) | claim 39a row 6 (zombie, paralysed) and the attack-move refresh at a deploy order | Neither property nor attack-move memory reaches a deploy order in our engine. `unmeasured`. |
| — (seam trace 3) | the systematic part of the AI game-length rise | Unexplained and not called noise. No claim to test. |

These 31 rows of the table hold 62 row parts, 22 of them in the five S-0501 rows.
