# M8 seam sweep 1: claims coded but not tested

Brief: `.claude/worktrees/M8-SEAMS-1-BRIEF.md`. The list below is every row of `docs/m8-claims/*.md`
(and the M6 and M7 ledgers they link to) that says "partly", "not tested", "untested", "satisfied
by construction" or "built, no test", whose spec is approved and whose blocker is not an open
request, minus the specs intake 15 revises (S-0108, S-0109, S-0212, S-0319, S-0201, S-0105, S-0703,
S-0701, S-0314) and the rows that need a real install. Each row was checked against the spec's
current revision (S-0318 r13, S-0503 r27, S-0501 r41, S-0306 r34, S-0317, S-0405 r36). Rows that an
open request blocks (Q-1600 and Q-1621 for the S-0513 and S-0519 interims, Q-1702 and Q-1762 for
voxel animations, Q-1772 for unmanned tech buildings, Q-1834, Q-1835 and Q-1836 for debris)
are left out by that rule and are not listed.

**Result.** 22 rows tested, through the real burst, turn or tick path. No test found a place where
the code differs from the spec, so no simulation code changed: no save format bump, no golden
change, and the oracle arena and the M6c, M7a and M7b sweeps are not affected (no sweep run was
needed, per the brief's rule for a simulation change). One new code gap found: the map's `Inert`
flag is not read (row 23 below).

## Tested

| # | Spec | Claim or scenario part | Ledger state before | Plan and result |
|---|---|---|---|---|
| 1 | S-0318 | SH1 part p (a wall hit in the shot's area damage takes its draw after the burst's six) | not tested | `sh1p_wall_hit_draw_follows_the_burst_draws`: 40 seeds, the wall's outcome follows the 7th draw; seeds that tell it from the 1st draw. The shot takes its own turn (`projectile::turn`), since an empty tick of this synthetic world already takes 8 raw draws from the other objects. Code agrees. |
| 2 | S-0318 | SH1 part r (burst point against detonation point, a building with an aim shift) | not tested | `sh1r_burst_point_against_detonation_point`: a visible straight shot (gravity 0, fired 20 leptons up at the firer's current target) reaches the building; the detonation point moves to the aim shift's cell, the burst stays at the building's cell, so no fragment; control run 2 gives the fragments of part a. Code agrees. |
| 3 | S-0318 | SH5 part c (an airburst bursts before its area damage and its nine children) | partly | `sh5c_airburst_bursts_before_its_area_damage_and_children`: candidates then drawn cells, then the rest of the draws equal a control run (no fragments) started from the state the six draws leave. Code agrees. |
| 4 | S-0318 | SH8 part a (a cloaked vehicle is a candidate) | satisfied by construction | `sh8a_a_cloaked_vehicle_is_a_candidate`. Code agrees. |
| 5 | S-0318 | SH8 part b (shroud) | satisfied | `sh8b_a_shrouded_vehicle_is_a_candidate`. Code agrees. |
| 6 | S-0318 | SH8 part c (disguise) | satisfied | `sh8c_a_disguise_plays_no_part`. Code agrees. |
| 7 | S-0318 | SH8 part d (anti-air and anti-ground flags) | satisfied | `sh8d_flags_of_the_fragment_projectile_are_ignored`. Code agrees. |
| 8 | S-0318 | SH8 part e (an aircraft in the air is not listed) | satisfied | `sh8e_an_aircraft_in_the_air_is_not_a_candidate`: the aircraft is lifted and relisted through `air::sync_listing` (S-0209 claim 13). Code agrees. |
| 9 | S-0318 | SH8 part f (high bridges, both runs) | satisfied | `sh8f_run_1_a_target_on_the_deck_launches_no_fragments`, `sh8f_run_2_a_vehicle_on_the_deck_is_not_a_candidate`. Code agrees. |
| 10 | S-0318 | Edge cases: a fragment range above 11 cells | satisfied, not tested | `range_above_eleven_cells_scans_to_ring_eleven_only`. Code agrees. |
| 11 | S-0318 | Claim 1 (a): the other special-hit rows | "not tested one by one" | `sh1n_every_special_hit_row_stops_the_burst`: all eleven rows of S-0514 claim 20's table, through the shot's detonation. Code agrees. |
| 12 | S-0503 | Claim 10: a vehicle in sleep or enter takes no draws, starts no spin, loses lift | untested (spec `unmeasured`) | `claim_10_sleep_takes_no_draws_and_still_loses_lift`. Code agrees. |
| 13 | S-0503 | Claim 11: the 104-lepton boundary | untested (spec `unmeasured`) | `claim_11_the_104_lepton_boundary`, with the hover bob switched off (`still_world`): the bob moves the height by up to 3 leptons inside a tick. Code agrees (103 marks, 104 does not). |
| 14 | S-0503 | Claim 11: a spin ending after a re-enabling still tests | untested | `claim_11_a_spin_ending_after_re_enabling_still_tests`. Code agrees. |
| 15 | S-0503 | Claim 11: a switch-off that starts no spin, with no spin active, makes no test | satisfied, no test | `claim_11_a_switch_off_without_a_spin_makes_no_test`. Code agrees. |
| 16 | S-0503 | Claim 10: repeated switch-offs take the draws again | partly (in the S-0314 changes rows) | Already tested by `claim_10_repeated_switch_off_replaces_the_spin_count`; the row was stale. Ledger row updated. |
| 17 | S-0503 | Claim 19: a final leg with unequal heights takes the unequal row | built, no test | `claim_19_final_leg_with_unequal_heights_takes_the_unequal_row`. The lift is put on *P* by hand (a real climb to 312 leptons is not set up). Code agrees. |
| 18 | S-0501 | Claim 45 and T2: a pad-bound visiting type crashes instead of retreating | built, not tested | `pad_bound_visiting_type_crashes_instead_of_retreating`. Code agrees. |
| 19 | S-0501 | Claim 45: area guard below the cruise height (two tests: no target, a held target) | built, not tested | `claim_45_below_the_cruise_height_takes_claim_26_with_k_2`, `claim_45_below_the_cruise_height_with_a_target_queues_attack`. Code agrees. |
| 20 | S-0306 | Claim 23: the water and land rows outrank *cloaked* (rank 8) | not tested on its own | `claim23_rows_outrank_a_cloaked_firer`, in all three cloaking states. Code agrees. |
| 21 | S-0317 | Claim 1: the hits of a piece's impact and end carry no attacker and no house | by inspection | `claim_1_hits_carry_no_attacker_and_no_house`: no kill is credited, one loss for the victim's own house. Code agrees. |
| 22 | S-0405 | Claim 9 (with S-0415 claim 9, W15 part k): a pack-up hides its slot animations from the build-down's first tick | built, not tested | `a_packing_buildings_slots_stay_until_its_build_down` (`mapview::capture_tests`, with `sale_slots(packs)`). Code agrees. |

## Found, not a test gap

| # | Spec | Claim or scenario part | Ledger state | What it is |
|---|---|---|---|---|
| 23 | S-0318 (SH1 part s run 2) and S-0303 | the map's `Inert` flag (claim 2 and Import mapping) | "partly: the inert-map flag is not tested" | Not implemented: the map's special-flags section is not read at all (`Inert` and `HarvesterImmune` are named only in the importer's coverage list), and `combat::area::detonate` and `combat::damage` carry a comment saying so. A code step (map reader, a world flag, the two checks), with a save-format decision if the flag is kept in the state. Left for a step of its own; no request needed, the spec is clear. |

## Left (no test written)

| Spec | Claim or scenario part | Ledger state | Why left |
|---|---|---|---|
| S-0501 | claim 18, the in-flight pad search needs the in-the-air answer, the search after the row needs neither | satisfied | A test needs a grounded aircraft that falls back with a pad present and no contact; no fixture does that yet. |
| S-0501 | claim 32, the revert at the loss brings back the target put aside | built, no test | Needs an overridden behaviour (S-0215 claim 10) with a put-aside target, and the loss in the same tick. Plan: build the override with `orders` and then `objects::take_inside` the very target. |
| S-0501 | T1, *firing* and *refiring* with no target: 1 tick; claim 35's interval | built, no test | The spec rates them `unmeasured` and no scenario places the loss inside *firing*. |
| S-0306 | claim 23, the late tests (rank 9) and the magnetic-beam *rearming* after the rows | not tested on their own | Needs a firer with a live wave and a grip. |
| S-0213 | *Deployed*: sorted at the press, applied later; a transition ending in the tick of the press | satisfied, no test | The observable is the order of two calls in one tick; `press` in `claims_s0213_deploy_rule.rs` applies at once. |
| S-0503 | claim 11, a naval-yard hover's water spin (S-0314 H4 part i run 6) | not implemented as a test | Belongs to S-0314, which intake 15 revises. |
| S-0301 | claim 14, the render-layer row for a lifting jumper | seam | Our engine has no render layer row to test. |
| S-0316, S-0317, S-0612 | the "partly" rows | interims | Their blockers are open requests (Q-1834 to Q-1836, Q-1762) or our own tuning. |

## Ledger rows changed

`intake6-S-0318-S-0314-S-0108-S-0701.md` (S-0318 terms, claim 1, claim 3, Edge cases, S-0503
claim 10 changes row, scenario rows, seams), `intake9-S-0501-S-0318-S-0503.md` (T2, S-0503 claim 11,
area guard seam), `intake14-S-0503.md` (claim 19 unequal heights), `intake5-S-0306-S-0309-S-0107.md`
(claim 23 cloaked), `intake10-S-0519-S-0215.md` (SH1 part r), `intake11-...md` (pack-up),
`c5-S-0317.md` (claim 1).
