# S-0513 r86, S-0514 r12 and S-0610 r12: claim ledger, M8 step A3 intake 75

S-0513 r62 to r86 answers **Q-2276** (filed by [intake 71](intake71-S-0513-S-0216.md)) with the
new claim 39a: a temporal beam locked on a superweapon warp member, its erasure through the warp,
the departure's ending of the being-erased state while the beam stays attached, and two quirk
switches. S-0514 r11 to r12 and S-0610 r11 to r12 take the matching changes (S-0514 claims 3, 9,
14 and 17; S-0610 Terms and claim 5). The S-0309, S-0215, S-0203 and S-0302 passages that cite
claim 39a are built here where they read the new state; the rest of those specs' revisions is
intake 74's. Requests filed: **Q-2316** and **Q-2317**. Two later revisions are folded in:
S-0514 r13 (claim 1: a departed warp member is an ordinary object target) and S-0513 r87 (claim
45: the spy plane's heading after its target); neither needs new behaviour.

## Built

- **The state.** `temporal::Lock` gains `suspended`: the target, a warp member, stopped being
  erased at its departure while the beam stays attached. `temporal::attached` is a lock on the
  object; `temporal::erased` is a lock that is not suspended. `warp::depart` calls
  `temporal::departed`, which sets it; every later `lock` on the target (leading or joining)
  clears it; a leadership handover (`let_go`) keeps it; the lock's end removes it with the lock.
  Hashed and saved: save format 92 → 93, AI game format 55 → 56.
- **The erasure update before the warp step.** `temporal::turn` is split: `erasure_update` (the
  open-topped distance release, the reduction and completion) and `turn(w, id, update)` (the
  being-erased sparkle, claim 17's clearing when the turn is cut short, and the update unless
  already made). In the living turn (`tick.rs`), `temporal::member_update` makes the erasure
  update of a warp member with a beam attached before `warp::turn`, in every stage; a member it
  removes returns there and takes no warp step (the no-step rule). The later `temporal::turn`
  then makes no second update.
- **Turns cut short.** `temporal::cut_short`: being erased, or phasing in with a beam attached as
  a vehicle or aircraft, or as an infantry unit with `quirks.phase_in_beam_spares_infantry` off.
  It decides the early return of `temporal::turn`, the drainer's alliance test and the unlimited
  controller's early overload update in `tick.rs`, and the periodic held-target check's skip in
  `held::periodic_check`.
- **The lock refusal switch.** `temporal::refusal`: with `quirks.warp_departure_keeps_beam` off,
  a target with a pending warp (`superweapons::warp_pending_or_leaving`) is one more refusal; the
  refused hit keeps claim 2's side effects. The firer test of claim 3 reads `attached`, not
  `erased`.
- **The firer keeps its departed member.** `held::turn_start_drop` does not drop a departed
  member held by a firer whose beam is attached to it (interim **Q-2317**), so a firer in *attack*
  still has it as its target when it lands, and approaches it when it is beyond range (claim 39a,
  *Release*). The firing step still counts an off-map target as none.
- **Quirks.** `warp_departure_keeps_beam` and `phase_in_beam_spares_infantry` in
  `glasnost_rules::quirks`, default on, in `named()`. C7 judgement: both rejected
  ([c7-play-fixes.md](c7-play-fixes.md), intake 75).

## Claims

| Claim part | Status | Where, and why |
|---|---|---|
| S-0513 39: a beam can be locked on a member from the launch on, by claim 39a's routes | tested | `claims_s0513_r86::claim39a_l1_a_shot_in_flight_locks_the_member_and_the_erasure_removes_it_after_its_hold` (a real Gun shot fired before the launch, detonating after it) |
| S-0513 39: another firer's beam does not change a firer's *illegal* | tested | `claims_s0513_r62::claim39_a_firer_whose_beam_holds_the_member_gets_rearming_and_keeps_it` (unchanged answer) |
| S-0513 39 (r86 wording): S-0309 claim 29 covers every holder of every owner, every behaviour but capture and sabotage | seam | intake 74's S-0309 revision (claim 29's widening); not built here |
| 39a The lock routes: a temporal projectile in flight at the launch | tested | l1 and l2 tests (`claims_s0513_r86`) |
| 39a The lock routes: a fragment of a non-temporal bursting projectile | partly | Nothing to build: `shrapnel` fragments carry the bursting projectile's firer as attacker, and the special hit goes through `temporal::deliver` like any other; no test (no synthetic fragment-weapon set-up here; the spec rates it `unmeasured`) |
| 39a The lock routes: an invisible shot delayed past t_L; the unchecked building and aircraft shots | partly | Follows from S-0306 *Ordering*, S-0215 claim 32, S-0501 claim 12 and S-0301 claim 14 as built; no new code, no test |
| 39a The lock: by S-0514 claims 1 to 4 alone; neither the pending warp nor the leaving stage refuses | tested | `claim39a_switch_on_a_pending_warp_is_no_lock_refusal`, l1 |
| 39a The warp goes on: departure in t_L + 62 | tested | l1 (`departure_seen` in t_L + 62) |
| 39a The erasure goes on: an update in every member turn, before the warp step; r = d + k | tested | l1 (amount after t_L + 61 and t_L + 62, removal in d + 120), l2 (removal in d + 40 in the leaving stage), `claims_s0306_r62::claim17_an_erasure_lock_on_a_member_off_the_map_makes_progress_in_every_turn` (off the map, placement, arrival) |
| 39a The erasure goes on: a lock before the member's turn of d gives r = d + k − 1 | tested | `claims_s0513_r62::claim39_a_firer_whose_beam_holds_the_member_gets_rearming_and_keeps_it` (delivery before the turns of t_L + 1, removal in t_L + 40; the firers' opportunity shells from before the launch are taken out of play first, since a detonation would lock again and restart R) |
| 39a The erasure goes on: more than one lock before the departure | partly | S-0514 claims 2, 4, 9 and 10 as built (`claims_s0514*`); no member-specific test |
| 39a The no-step rule | tested | `claim39a_l4_a_member_removed_in_its_departure_update_takes_no_warp_step` (no warp-out animation or sound), `claim39a_l4_a_member_removed_in_its_expiry_update_is_removed_before_the_expiry`, l2 |
| 39a Departure: no release, amount kept, no longer being erased, beam attached | tested | l1 (`attached`, `holds`, `!erased`, `!being_erased`, amount) |
| 39a Departure: S-0302 claim 8's protection lapses | tested | l1 (an ordinary SA hit after the hold lowers health; l5 shows the same hit stopped again after a later lock) |
| 39a Departure: no being-erased sparkles, only phasing-in ones | partly | `temporal::turn` sparkles only while `erased`; `phasing_sparkle` unchanged; no test |
| 39a Departure: a vehicle or aircraft phasing in with a beam has its turns cut short | tested | `claim39a_a_vehicle_member_with_a_beam_has_its_turns_cut_short_while_it_phases_in` (redirected landing, hold of 7) |
| 39a Departure: an infantry member phasing in takes the turns of one with no beam (switch) | tested | `claim39a_an_infantry_member_with_a_beam_phases_in_as_one_with_no_beam_in_both_positions` |
| 39a Departure: ordinary turns after the hold, removal in r | tested | l1 (moves on an order, removed in d + k) |
| 39a Departure: a handover keeps the member not being erased | tested | `claim39a_a_handover_after_the_departure_keeps_the_member_not_erased_up_to_a_later_lock` |
| 39a A lock after the departure: joins without resetting R, being erased again, protection, skipped turns, earlier removal, leader credited | tested | `claim39a_l5_a_joining_lock_after_the_hold_makes_the_member_erased_again_and_removes_it_earlier` (exact tick from ⌈R′ / 20⌉) |
| 39a A lock after the departure: the firer already on the beam; zero or negative steps | partly | S-0514 claims 2, 4, 7, 9, 10 and F3 as built; no member-specific test |
| 39a Release: the warp is no release cause; a firer in attack approaches and lets go in a new cell | tested | `claim39a_l3_a_firer_in_attack_approaches_a_member_landed_beyond_range_and_lets_go`; the turn-start drop exemption is interim Q-2317 (without it, l1, l3 and l5 fail: the firer lets the beam go after the departure) |
| 39a Release: buildings and deployed infantry take no approach step | partly | S-0215 claim 37 g as built; no test |
| 39a The firer: *rearming* before and after the departure; no further shot | tested | l1 (one shot in all), `claims_s0513_r62` (rearming kept at the check) |
| 39a Switch `warp_departure_keeps_beam` off: one more refusal, the warp goes on | tested | `claim39a_switch_off_a_pending_warp_is_one_more_lock_refusal` |
| 39a Departure from the original (no switch) | tested | the l4 tests (our no-step rule) |
| 51: the erasure update is not suppressed ordinary work; cut-short phasing-in turns in the hold row | tested | l1, the vehicle and infantry hold tests |
| 51: the allied held-target drop at the check's place (S-0309 claim 25 part k) | seam | intake 74 (S-0309 claim 25 part k) |
| Edge cases: the two switches | tested | the switch tests above; `quirks::every_spec_quirk_has_a_switch` |
| S-0203 claim 4: the erasure update is an intrinsic exception | tested | as 39a *The erasure goes on* |
| S-0215 claim 2: the phasing-in exclusions, no per-tick start, no berserk countdown | tested | the vehicle hold test; `claim39a_the_berserk_countdown_skips_the_cut_short_phasing_in_updates` |
| S-0309 claim 29 *Turns cut short*: a phasing-in vehicle with a beam drops its target; an infantry unit makes the check unless the switch is off | partly | `held::periodic_check` skips on `cut_short`, and `temporal::turn` clears the target; no check-tick test |
| S-0302 claim 8: no protection after the departure, protection again from a later lock | tested | l1, l5 |
| S-0306 claim 17: a lock while phasing in or after has 39a's effects | tested | l5 (after the hold); the lock while off the map stays an Open question (Q-2231), built as erased with progress in every turn |
| S-0514 claim 3: a pending warp is no refusal; off, one more | tested | the two switch tests |
| S-0514 claim 9: exception for the handover | tested | the handover test |
| S-0514 claim 14: the target's own warp is no release cause | tested | l1, l3 |
| S-0514 claim 17: the warp member's turns | tested | l1, the vehicle and infantry hold tests, l5 |
| S-0610 Terms and claim 5: the phasing-in excluded updates take no countdown step | tested | `claim39a_the_berserk_countdown_skips_the_cut_short_phasing_in_updates` (vehicle, infantry both positions) |
| S-0610 claim 4: the crush response follows the berserk entry (S-0215 claim 39) | seam | intake 74 (S-0215 claim 39) |
| Q-2316: sparkles and claim 17 clearing in the warp-only turns | request | interim: none; only the erasure update runs there |
| S-0514 r13 claim 1: a departed warp member, off the map or phasing in, is an ordinary object target for claims 1 to 4 | tested | Already built (`temporal::deliver`'s ground-layer test passes through `warp::in_ground_layer`, S-0306 claim 17); cited r13. Phasing in (P11 run h): `claims_s0513_r86::s0514_r13_claim1_a_member_phasing_in_is_an_ordinary_object_target_for_the_lock`; off the map (run g): `claims_s0306_r62::claim17_an_erasure_lock_on_a_member_off_the_map_makes_progress_in_every_turn` |
| S-0513 r87 claim 45: after its target the spy plane keeps its approach heading and is not steered toward its exit cell (S-0501 claims 4, 10, 34 and 51) | tested | No new code: S-0501 r93 claim 51 as built by intake 76; cited r87 in `air::spy_overfly`. Test: `claims_s0501_r93_edge::claim_51_the_heading_is_held_and_nothing_ends_overfly` (intake 76's) |

## Lanes

Main 0cb93b5 against the branch. The M6c, M7a and M7b seed sweeps (seeds 1 to 16) are identical.
The arena (`arena --record all`) is byte-identical (records M-0001 to M-0003); the same 13
pairings fail, none changed: LTNK–E1, TELE–APOC, TELE–TTNK, TELE–V3, CAOS–APOC, CAOS–TTNK,
CAOS–V3, CAOS–E1, DISK–FV, INIT–E2, ROBO–HTK, YURI–GGI, YURI–SHK. The AI quick lane (seeds 7 to
10, maps 0, 2 and 8) is identical apart from timings. `glasnost-ai --include-ignored`: 246
passed, 0 failed, before and after. Golden moved: `duel-m6c` (the hash lines of the ticks in
which a temporal lock exists, now hashing `Lock::suspended`; the outcomes and the sweeps do not
move). The committed JSON schema gains the two switches.

## Requests

- **Q-2316**: whether a member being erased sparkles and drops a given target or destination in
  its warp-only turns. Interim: no; only the erasure update runs before the warp step.
- **Q-2317**: whether a firer whose beam is attached keeps a departed member as its target.
  Interim: yes (no turn-start drop), so the approach of claim 39a *Release* can happen.
