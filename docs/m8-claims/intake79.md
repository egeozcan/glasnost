# S-0301 r69, S-0304 r6, S-0310 r8, S-0607 r38 and the other combat details: claim ledger, M8 step A3 intake 79

The last built revision of each spec is the one at 5f2b926 (batch brief of 2026-11-02); each delta
was read in full (`git diff 5f2b926 main -- <file>`):

- **S-0301** r67 to r69: claim 9's rank-4 row and its new paragraphs *A materialising firer
  within rank 4* and *Targets with a superweapon warp pending within rank 4*, the infantry
  answers in *Grip refusals within rank 4*, their backing and confidence, and the Changes for
  S-0607 claim 11. With S-0309's answer to Q-2264 (the held-target check by a materialising
  holder), this gives that holder's answer its place.
- **S-0304** r2 to r6: claim 2's drop occasions (S-0309 claim 29's periodic check, a building's
  attack processing, a healer's firing step), the infantry healer's exception to the countdown,
  every owner, the allied drop; V1 run 4b's tick numbers.
- **S-0310** r5 to r8: claim 11's keep row lasts only to S-0309 claim 29's next check; the
  infantry healer's reply in its firing step; T10 part d's tick numbers.
- **S-0607** r37 to r38: claim 11, an infantry lifter's answers ahead of the gripped-unit
  *rearming*, the drop they cause and the grip that stays.
- **S-0302** r39 to r40: claim 8, a warp member whose beam was locked before the departure is not
  being erased, up to a later lock.
- **S-0303** r31 to r32: Terms (*tracked flying object*, an aircraft placed in the air ends its
  registration at touchdown) and the `ConsideredAircraft` default by kind.
- **S-0306** r63 to r64: claim 17, a lock made while the member phases in, or after, has S-0513
  claim 39a's effects.
- **S-0307** r20 to r21: Terms, a departed warp member counted by its height for T1, T2, claim 20
  and claim 17's loss.
- **S-0308** r16 to r17: claim 11, a "none" result on a water target is refused by S-0306 claim
  23's water row.
- **S-0318** r25 to r27: claims 1, 5 and 6, the raised start and the direction from the unraised
  burst point.
- **S-0004** r19 to r20: claim 11 rule 3 and the Import mapping, open question 2 resolved.
- **S-0506** r19 to r20, **S-0601** r20 to r21, **S-0519** r17 to r18, **S-0516** r12 to r13,
  **S-0503** r27 to r28, **S-0207** r82 to r83: the lead's one-step revisions.

Requests filed: **Q-2356** (the attack cursor and order acceptance for an actor that is
materialising: whether S-0214 claim 6 (e) passes over the materialising *rearming*). Q-2357 to
Q-2365 are unused.

**The code change** is one reorder of rank 4 in `combat::check::answer` (S-0301 r69): after the
building's delayed shot and the vehicle's launching *busy*, the infantry healer's *illegal*, then
the materialising firer's *rearming* for every target and weapon (an empty slot included), then
the own-gripped-unit *rearming*, the deactivated vehicle's *illegal* and the own drained
building's *illegal*, each without its own phasing branch, then the rest unchanged. Before, the
materialising *rearming* came after the projectile, berserk, protected-object and hidden-target
answers and the empty slot's *impossible*. This closes intake 74's seam "materialising holder"
(S-0309 A6 part p, run p10a). The attack rule's check (`check_lasting`) passes over the
materialising *rearming* (interim under Q-2356). No new state, quirk switch or format: save
format 93 and AI game format 56 stay.

Status words: **tested** (built and checked by the named test), **partly** (the part named is
untested or not built), **seam** (not built here; the reason is given), **no change** (the
revision restates what is built).

## S-0301 r69

| claim part | status |
|---|---|
| 9 *A materialising firer within rank 4*: *rearming* for every target and weapon; it outranks every other rank-4 answer, among them the hidden-target *impossible*, the projectile refusals of S-0306 claim 23, the parasite warhead's *illegal* against a protected object and the empty slot's *impossible* | Tested: `check::answer` asks `teleport::phasing_in` (a teleporting jump's phase-in and a superweapon warp member's) right after the infantry answers. `claims_s0301_r69::a_materialising_firer_gets_rearming_ahead_of_the_rank_4_refusals` (airborne target with a projectile that is not anti-air, a cell with one that is not anti-ground, a hidden target, a parasite warhead at a protected object, an empty slot, slot 1 empty; each control first) |
| 9 and S-0214 claim 6 (e): whether the attack rule's check counts the materialising *rearming* | Partly (interim, **Q-2356**): `check_lasting` (the attack rule and the AI's lasting tests, `moment` off) passes over it as a reason of the moment, as it passes over ranks 5 to 8, so the target tests decide the cursor. `q2356_the_attack_rules_check_passes_over_the_materialising_rearming` |
| 9: it yields, for vehicles, to ranks 1 to 3 and the tube-transit *impossible* | Partly: rank 2 (the launching *busy*) stays above it. Ranks 1 and 3 and the tube transit have no state in our engine (as before) |
| 9: it yields to the *illegal* of a firer bound to a slave master | Seam: our engine has no slave-bound firer (as before) |
| 9: for infantry it yields to the infantry healer's *illegal* (S-0310 claim 11) | Tested: the healer's test now comes ahead of it. `a_materialising_infantry_healer_gets_its_illegal_first` (full health, not infantry, no target: *illegal*; a hurt infantry target: *rearming*, and *illegal* by the zero versus value once the phase-in is taken away) |
| 9: for infantry it yields to the keep-acting *illegal* against a vehicle target melee-linked to another object | Seam: S-0214's melee link has no state in our engine (`targeting::melee_held` is a stub; the link's rules key is not imported). The answer's place is kept by comment in `check::answer`, with no test |
| 9: no answer for an infantry unit in a dying or falling sequence (S-0310's open question) | No change: open there; our order puts the falling answer below, as before |
| 9: the held-target check by a materialising holder (S-0309 A6 part p run p10a; Q-2264, answered by S-0309) | Tested: `p10a_a_materialising_holder_keeps_its_refused_target_until_the_phase_in_ends` (a teleporting vehicle holding a cell its anti-air-only weapon refuses keeps it at the check ticks while it phases in, through `tick::step`, and drops it at the first check tick after the phase-in ends, with no shot; the control without the phase-in drops at the first check tick). Intake 74's seam is closed |
| 9 *Grip refusals within rank 4*: for an infantry firer the answers asked first precede the gripped-unit *rearming*; a firer that meets all three gets the healer's *illegal*, then the materialising *rearming*, then the gripped-unit *rearming* | Tested: `an_infantry_healer_lifter_gets_the_healers_illegal_ahead_of_its_grip` (a gripped vehicle: *illegal*, phasing in or not; a gripped hurt infantry unit: *rearming*, from the phase-in and then from the grip, over the rank-9 zero versus value). The grip is set directly: no stock or synthetic infantry type grips by the ordinary rules |
| 9: such a firer drops the gripped unit at S-0309 claim 29's check or through its firing step's reply; a materialising one at the first of the two | Tested for the firing step: `an_infantry_healer_lifter_drops_its_gripped_vehicle_and_keeps_the_grip` (dropped in its first turn, off the check phase). The check-tick drop is the same answer through `held::periodic_check` |
| 9 *Deactivated firers within rank 4* and S-0608 claim 3: the deactivated and own-drained-building refusals yield to the materialising and gripped-unit *rearming* | Tested by the existing tests `claims_s0405::claim_20_deactivated_firer_phasing_in_answers_rearming` and the materialising case in `claims_s0608.rs` (claim 3's precedence test); the inner phasing branches are gone, since the materialising answer is now asked before them |
| 9 *Targets with a superweapon warp pending within rank 4* (S-0513 claim 39): after every other rank-4 reason except the spawner answers, the other weapon's live effect, every row of S-0306 claim 23 and the magnetic-beam *rearming*; ahead of ranks 5 and 7 to 16 | No change: built by intake 74 from S-0513 r62's Changes (`claims_s0513_r43.rs`, `claims_s0513_r62.rs`); the code comment now names S-0301 r69 |
| 9 backing and confidence | No change: measurement text |
| Changes for S-0607 claim 11 | See S-0607 r38 below |

## S-0304 r6

| claim part | status |
|---|---|
| 2: the drop at the first occasion that answers *illegal*: S-0309 claim 29's periodic check; a building's attack processing (S-0215 claim 32); a healer's firing step | No change: built by intake 68 and intake 74 (`held::periodic_check`, `behave.rs`'s healer reply, S-0215 claim 32's tables). `claims_s0309_r70.rs` p1 to p6 and the healer tests; `claims_s0304_s0305.rs` |
| 2: while the countdown runs the answer is *rearming*; the drop at the first check tick at whose turn the countdown has run out, the tick in which it runs out included (V1 run 4b) | Tested: `claims_s0301_r69::v1_4b_a_countdown_that_runs_out_in_a_check_tick_drops_in_that_tick` (a countdown ending exactly on a check tick drops there; one tick later drops 16 ticks later; no shot). `claims_s0309_r70::p2_*` covers a countdown ending off the check phase |
| 2: the infantry healer's own *illegal* outranks the countdown | Tested: `an_infantry_healer_drops_a_full_health_target_in_its_firing_step_while_its_countdown_runs` |
| 2: the healer's keep row leaves the target to the periodic check, which keeps it while the countdown runs | Tested: `an_infantry_healers_kept_hurt_target_waits_for_the_countdown` |
| 2: a SAM building's *aiming* and *aimed* rows, its power row | No change: S-0215 claim 32, built by intake 74 (`behave.rs`, `seams.rs`) |
| 2: every owner; the allied drop of a computer-controlled actor and its exceptions | No change: S-0309 claims 25 part k and 29, built by intake 74 |
| 2 backing, confidence; table row; open question removed | No change |

## S-0310 r8

| claim part | status |
|---|---|
| 11: a vehicle healer's keep row lasts only to S-0309 claim 29's next check (T10 part d with tick numbers) | No change: built by intake 74. `claims_s0309_r70::a_vehicle_healers_keep_lasts_to_the_check_tick`, now citing S-0310 r8 |
| 11: an infantry healer replies to *illegal* in its own firing step (drops a target that is not infantry or is at full health; keeps a hurt infantry target) | No change: built by intake 74 (`combat/behave.rs`, now citing S-0310 r8). `an_infantry_healer_drops_a_vehicle_target_in_its_firing_step` |
| 11: that keep lasts only to S-0309 claim 29's next check | No change: `an_infantry_healers_keep_lasts_to_the_check_tick` |
| 11: the infantry healer's *illegal* outranks every answer of the general part | Tested: the reorder puts it above the materialising and gripped-unit *rearming* as well (S-0301 r69, above) |
| T10 parts d and h, the claim 10 reply table's *illegal* row | No change: measurement text; the row's wording now points to claim 11's table, as built |

## S-0607 r38

| claim part | status |
|---|---|
| 11: for an infantry lifter the answers asked first precede the gripped-unit *rearming*; the drop they cause at the check or the firing step; the grip stays | Tested: see S-0301 claim 9 *Grip refusals* above (`claims_s0301_r69.rs`, the two lifter tests). The keep-acting answer is the same seam |
| 11 confidence | No change |

## The other combat specs

| claim part | status |
|---|---|
| S-0302 r40 claim 8: a warp member whose beam was locked before the departure is not being erased from its departure update and takes ordinary hits, up to a later lock, from which the protection applies again | No change: built by intake 75 from S-0513 r86 claim 39a. `claims_s0513_r86.rs` l1 and l5 (`claim39a_l5_a_joining_lock_after_the_hold_makes_the_member_erased_again_and_removes_it_earlier`). The damage gate (`combat/damage.rs`, `being_erased`) now names S-0302 r40 |
| S-0303 r32 Terms: an aircraft placed in the air with the counts-as-aircraft flag ends its registration at touchdown (S-0501 claim 47) | No change: S-0501 claim 47, built earlier (`air.rs` tracking; `claims_s0501_tracking.rs`) |
| S-0303 r32 Import mapping: `ConsideredAircraft` absent: yes for an aircraft type, no for others | No change: `econ::tech` resolves it by kind (`unwrap_or(air_default)`); the converter keeps an absent key as absent. `claims_s0411::absent_counts_as_aircraft_ranks_an_aircraft_as_air`, `claims_s0501_tracking.rs`. The converter test's stale "absent: no" comment is corrected |
| S-0306 r64 claim 17: a lock made while the member phases in, or after, has S-0513 claim 39a's effects | No change: built by intake 75. `claims_s0513_r86.rs` l5 and `s0514_r13_claim1_a_member_phasing_in_is_an_ordinary_object_target_for_the_lock` |
| S-0307 r21 Terms: a departed warp member is counted by its height above the surface beneath its kept position for T1, T2, claim 20 and claim 17's loss; the deck exception | No change: built earlier from S-0306 claim 17 (`combat::projectile::target_airborne`, used by the T1 anti-air row, claim 20's airborne question, claim 17's loss and the landing-check kill in `combat/death.rs`; `quirks.warp_deck_departure_airborne`). Its comment now names S-0307 r21 |
| S-0308 r17 claim 11: a "none" result on a water target is refused by S-0306 claim 23's water row | No change: `check::targeting_mode_illegal`, built earlier |
| S-0318 r27 claims 1 and 5: the start raised as S-0306 claim 8 gives | No change: `shrapnel::fragment` places the fragment at `projectile::start_point(burst)`. `claims_s0318::s0306_claim8_fragments_start_above_the_ground` |
| S-0318 r27 claim 6: the direction from the burst point as it is, not the raised start | No change: the yaw is taken from the unraised `burst`. Not separately observable (the two differ only in height; the yaw is horizontal), as the spec says |
| S-0004 r20 claim 11 rule 3: an aircraft type that leaves the key out ranks as air unless naval; open question 2 resolved | No change: `claims_s0411::absent_counts_as_aircraft_ranks_an_aircraft_as_air`, now citing r20; the `econ::tech` comment no longer points to the open question |

## The lead's one-step revisions

| claim part | status |
|---|---|
| S-0506 r20 claim 13: clearing the slots keeps the controller target; the passes go on while the spawner is seized; the launch after restoration (S-0607 claim 20) | No change: built earlier from S-0607 r37's L19 (`spawn::clear_slots`). `claims_s0607_r37::l19b_kept_controller_target_launches_after_restoration` and the other L19 tests |
| S-0601 r21 claim 5, S-0519 r18 claims 2, 7 and 16, S-0516 r13 claim 31: a departed warp member, off the map or phasing in, is an ordinary target for the takeover, the bomb, the disarm, the designation and the disguise copy | No change: built earlier from S-0306 claim 17. `claims_s0306_r62::claim17_p11_run_g_special_hits_act_on_a_member_off_the_map` and `claim17_p11_run_h_special_hits_act_on_a_member_phasing_in`, now citing each claim |
| S-0503 r28 claim 10: whether a switched-off hovering vehicle is led follows from whether it is moving; losing the heading point alone leaves it moving while it keeps its destination | Tested: `claims_s0306::claim3b_a_switched_off_hover_that_loses_only_its_heading_point_is_still_led` (new: a destination and no heading point, still led; neither, not led); `claim3b_a_switched_off_hover_that_arrives_is_no_longer_led` now cites r28 |
| S-0207 r83 claim 20 (e): the jump's drops take no targeting-timer draw | No change: built earlier from S-0513 claim 39's Changes for S-0207 (`teleport::targeters_drop`). `claims_s0513_r43::claim39_the_own_jump_drops_with_no_timer_draw`, now citing r83 |

## Intake 78's three seams (S-0405 r51 claim 20)

S-0306 r64 changes only claim 17, so the three rules S-0405 r51 claim 20 points to are S-0306's
earlier revisions, all built before this intake:

| claim part | status |
|---|---|
| The lead of shots at a switched-off hovering vehicle that S-0503 counts as moving (S-0306 claims 3 b and 6 a) | No change: `combat::projectile::led` reads `hover::moving` under `quirks.switched_off_hover_lead`. `claims_s0405_r47_hover::claim_20_a_disabled_hover_is_targeted_with_the_lead_quirk_on` (the power-off path), `claims_s0306::claim3b_*` |
| A led arcing shot can be void; a void launch leaves the countdown unchanged and is attempted again (S-0306 claim 6 a, S-0301 claim 15) | No change: `claims_s0306::claim6a_void_launch_in_the_firing_step_keeps_the_countdown` |
| A straight or arcing projectile whose firer is still in play passes below the ground and flies on, except a flak-scattering one with a target (S-0306 T1 and T2) | No change: `claims_s0306` `p44b_owned_projectile_falls_to_the_deep_fall_row` and `p45b_owned_flak_projectile_detonates_in_its_first_turn_below_the_surface` |
