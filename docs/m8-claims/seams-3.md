# M8 seam sweep 3: claims coded but not tested, and the seams A5 left

Brief: `.claude/worktrees/M8-SEAMS-3-BRIEF.md`. Method and row form follow [seams-2.md](seams-2.md).
Rows read: `intake45-*`, `intake46-S-0111.md`, `intake47-arena-restated.md`, `intake48-S-0111.md`,
`a4-rocking.md`, `c7-pathing.md`, `a6-arena-triage.md`, the B10 section of
`docs/own-ai-m8-track-b.md`, the seams of `S-0109-playback.md` (step A5), and sweep 2's *Left*
rows. Skipped as blocked by an open request: Q-2081 (rocking's rates, return to level, weight,
sideways limit, passenger fire point), Q-2096 to Q-2099 (arena), Q-2111 to Q-2114 (Q-2111 since
answered by S-0306 r45 and S-0313 r37 and built in intake 51,
[intake51-S-0306-S-0313.md](intake51-S-0306-S-0313.md)).

**Result.** One seam built (the animation clock's switches reach the simulation) and 11 tests
written: 6 for the switches in both positions, 3 for S-0207 claim 28's exclusions, 1 for S-0306
claim 23's gripped unit, 1 for S-0303 claim 20's cell layer. No test found the code differing from
an approved claim. The switches default on and no reference ruleset sets one off, so the
simulation plays as before: no hashed state, no save or AI format bump (save 86, AI game 48), the
sim-gate goldens and the M6c, M7a and M7b seed sweeps 1 to 16 are unchanged (the gate suite with
`--include-ignored` passes without regeneration). No spec request was filed (Q-2121 to Q-2125
unused).

## Built

| Seam | What changed |
|---|---|
| A5: the simulation plays with `AnimQuirks::default()` | `quirks.anim_loop_count_wrap`, `anim_reverse_raw_loop_end`, `anim_pingpong_lower_turn`, `anim_loop_end_from_sprite`, `anim_followup_start_doubled` and `anim_followup_keeps_step` are now fields of `glasnost_rules::quirks::Quirks` (serde default on, in `named()`), so a layer or a ruleset can set them. `AnimQuirks::from(&Quirks)` builds the clock's switches. Every clock call in `anims`, `mutation`, `depot_anim` and `domination` takes them from `w.rules().quirks`, and so does the client's own cosmetic clock (`mapview::live`, through its `SimRules`). One call stays on the defaults: `domination::play_fields`, the importer's warning about a sweep animation that cannot reach stage N − 1 (a design aid, no playback). |

## Tested

| # | Spec | Claim or scenario part | Ledger state before | Plan and result |
|---|---|---|---|---|
| 1 | S-0109 | claim 7, `quirks.anim_loop_count_wrap` | seam | `quirk_anim_loop_count_wrap_in_both_positions` (`claims_s0109_quirks.rs`): loop count 256 plays one pass on, 1024 ticks (256 passes) off. Code agrees. |
| 2 | S-0109 | claim 8, `quirks.anim_reverse_raw_loop_end` | seam | `quirk_anim_reverse_raw_loop_end_in_both_positions`: a reversed pass shows 3, 2, 1 on and 3, 2, 1, 0 off. Code agrees. |
| 3 | S-0109 | claims 6 and 25, `quirks.anim_loop_end_from_sprite` | seam | `quirk_anim_loop_end_from_sprite_in_both_positions`: an end field of 2, two passes, no loop end: 0 1 2 3 0 1 on, 0 1 0 1 off. Code agrees. |
| 4 | S-0109 | claim 15, `quirks.anim_followup_start_doubled` | seam | `quirk_anim_followup_start_doubled_in_both_positions`: a follow-up with start field 1 shows frame 2 first on, 1 off. Code agrees. |
| 5 | S-0109 | claim 15, `quirks.anim_followup_keeps_step` | seam | `quirk_anim_followup_keeps_step_in_both_positions`: a reversed type with a forward follow-up never ends on, ends off. Code agrees. |
| 6 | S-0109 | claim 9, `quirks.anim_pingpong_lower_turn` | seam | `quirk_anim_pingpong_lower_turn_in_both_positions`: with a = 1 and two passes the animation turns at frame 2, comes down past a and runs below 0 on, turns back at frame 0 off. Code agrees. |
| 7 | S-0207 | claim 28, a warp-pending vehicle (sweep 2 *Left*: "gripped needs S-0607") | by construction | `claim_28_a_warp_pending_vehicle_has_nothing_installed` (`claims_s0207_r82.rs`): at pending, leaving and expired, a real destination installs no driving and the teleporting locomotor stays. Code agrees. |
| 8 | S-0207 | claim 28, a gripped vehicle (S-0607 is built since intake 45) | by construction | `claim_28_a_gripped_vehicle_has_nothing_installed`. Code agrees. |
| 9 | S-0207 | claim 28, a seized vehicle on a held destination | tested weakly | The existing test gave an empty assignment to a vehicle with no destination, which claim 27 rejects as unchanged, so it passed without the exclusion. `claim_28_the_exclusions_against_a_control`: the same assignment with no exclusion installs driving; seized installs nothing. Code agrees. |
| 10 | S-0306 | claim 23, the firer's gripped unit (sweep 2 *Left*: "needs a live wave and a grip, S-0607") | not tested on its own | `claim23_the_firers_gripped_unit_is_never_refused_by_a_row` (`claims_s0306_modes.rs`): the land row (mode 1) and the airborne row both refuse the unit, and answer *rearming* once it is the firer's gripped unit; another object of the same kind is still refused. The grip is set on the two lift records, not through a live wave (S-0607 claim 11 has its own test with a wave, `claim_11_rearming_while_gripping`). Code agrees. |
| 11 | S-0303 | claim 20, rocking reaches the detonation's cell layer (`a4-rocking.md`: "the deck-layer case is not covered") | tested, deck case open | `claim_20_rocks_the_detonations_cell_layer` (`claims_s0303_rocking.rs`): on a high bridge a detonation more than 208 above the terrain rocks the deck's object only, one at the surface the ground's object only. Code agrees. |

## Left (no test written)

| Spec | Claim or scenario part | Ledger state | Why left |
|---|---|---|---|
| S-0312 | claim 18, survivors (A5's seam "no house side index in `SimRules`") | seam | The reason has changed but the claim is still not buildable. The side index now exists: `SimRules` carries `econ.tech.country_side` (S-0004 claim 9), already read by S-0516 and S-0522, and the survivor divisors and crew types are imported. What F4 and F5 still need is not in the simulation: the map entry's no-crew flag (S-0312 table, map specs), an object's "has ever changed owner" mark (hashed state: a format bump), a charge planter's mark, S-0312's *armed* test, and the owner's defeat state for the chance (`seams::game_over` exists). Building it also changes every vehicle crew (`crew_type` stops always giving the technician) and the draws of every building death, so it needs its own arena run and its own step. Not a test gap: a missing feature, to be taken as a step. |
| S-0607 | claim 10, a seized unit at height 0 that is reached deploys (intake 45, "partly") | partly | The reason holds: no order route reaches a seized unit at rest, so only a direct call of the deploy step can test it. |
| S-0607 | the early-ending turns of `lift::end_of_turn`, a phasing unit's warp-only updates | no test | The rows are design choices for S-0607's open questions, not claims. |
| S-0504, S-0313 | claim 16 *Passengers*, the fire point on the leaned body; the slope tilt in F5 | seam | Q-2081 answered by S-0113 (intake 53): rocking is game state with fire points built on it (claim 19), still a seam, [intake53-S-0107-S-0108-S-0113.md](intake53-S-0107-S-0108-S-0113.md). |
| S-0303, S-0614, S-0601, S-0210 | the rates, return to level, weight scaling | tested | Q-2081 answered by S-0113; built in intake 53, [intake53-S-0107-S-0108-S-0113.md](intake53-S-0107-S-0108-S-0113.md). |
| S-0111 | OV6 to OV12, OV10 b/c, OV15 | not run | Measurable by the lead; local-lane pictures only. |
| S-0111 | claim 1's culling is read at each status redraw, not at each scroll step | design | Presentation only. |
| c7 | pathing "Left open" items (shore maximum, the harvesters' dock, the yield's choice of side with more than two groups) | proposals | Not claims; the lead's decision of 2026-10-29 admits the improvements as our design. |
| B10 | the AI's broke-house recovery | own AI | Tuning of our own AI, no spec rows. |
| a6 | the arena triage rows | requests | Open requests Q-2096 to Q-2099 and others. Q-1710 was answered by S-0405 r47 (intake 52b). |
| S-0501 | claims 47 (a), 48 (c), 48 (d), 18, T1 | as sweep 2 | Sweep 2's reasons hold. |
| S-0212 | claims 17 and 18, the flattening's threat test | partly | As sweep 2: no gate or arena unit has a coefficient. |
| S-0306 | claim 23, the magnetic-beam *rearming* of a live wave | covered | `claim_11_rearming_while_gripping` (`claims_s0607.rs`) tests the wave; row 10 tests claim 23's exception. |
| S-0207 | claim 28, a sale with the miner still in *enter* | by construction | As sweep 2: the same assignment paths as rows 13 to 15 of sweep 2. |

## Ledger rows changed

`S-0109-playback.md` (the seam "quirk switches of the clock", and section *Seams*), the
development notes (*Animation playback in the simulation*: the seam sentence), `intake39-S-0207.md`
is unchanged (its rows already point at claim 28's tests).
