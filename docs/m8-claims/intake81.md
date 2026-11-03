# S-0513 r89, S-0514 r14, S-0309 r95 and S-0203 r43: claim ledger, M8 step A3 intake 81

S-0513 r87 to r89 answers **Q-2316** and **Q-2317** (filed by [intake 75](intake75.md)) in claim
39a: *Sparkles and drops in the warp's turns*, and *The firer* while the member waits to land,
with the matching passages of claim 39, claim 51 and the Changes for S-0309 claim 27, S-0203
claim 4 and S-0514 claim 18. S-0514 r13 to r14 is that claim 18 change: the sparkle is an ordinary
animation with S-0109's simulation effects, its creation draw before a warp member's warp step.
S-0309 r94 to r95 (commit 33deb9c, claim 27) and S-0203 r42 to r43 (commit b520795, claim 4)
carry the same answers into those specs: the temporal-beam firer exception to claim 27's drop,
and the sparkle before and the claim 17 clearing after a warp member's warp work as
non-suppressed exceptions. Intake 82, which was to build them, was stopped, and this intake
took them on: both deltas were read against what is built here, and neither asks for more, so
they add citations and tests only. S-0404 r23 is another intake's. Requests filed: **Q-2376**
and **Q-2377**.

## Built

- **Sparkles in a member's turns.** `temporal::turn_sparkle` makes S-0514 claim 18's sparkle of
  an object being erased (`superweapons::being_erased`: a beam, or a warp member's leaving
  stage), phasing in (`teleport::phasing_in`: an ordinary teleport, or a warp member from its
  departure to its hold's end), or both, once per turn. The living turn (`tick.rs`) calls it for a
  warp member right after `held::turn_start_drop`, before the member's erasure update and warp
  step, so the state before the warp step decides and the sparkle's creation draws come first;
  any other object sparkles where it did, in the ordinary turn after `temporal::holder_turn`.
  `temporal::phasing_sparkle` is gone, and `temporal::turn` no longer sparkles; before, an
  object being erased and phasing in at once (a member in its hold after a later lock, an
  ordinary teleport with a beam) made two sparkles in one turn.
- **Drops after the warp step.** `temporal::member_drop`, called when `warp::turn` takes the
  whole turn: if the member is still being erased after its warp step, or its turns are cut
  short by a beam (`temporal::cut_short`), it clears its target and destination as S-0514 claim
  17 gives (`temporal::inactivity_clear`, the clearing `temporal::turn` already made, the
  destination kept under a pending point suppression; a building clears nothing). So the expiry
  update's target search (`teleport::phasing_search`) is dropped again in the same update, a
  vehicle or aircraft with a beam drops at its departure, off the map, at placement and arrival,
  and every member drops after a later lock. The arrival hold, whose ordinary turn follows the
  warp step, was already covered by `temporal::turn`.
- **The firer while the member waits to land.** `combat::check`'s `target_exists` now takes the
  firer: a departed member held by the firer's beam is still a target, so the rank-4 rows answer
  as before the departure (a deactivated vehicle's *illegal*, else S-0514 claim 19's
  *rearming*). Before, the check saw no target off the map and answered *can fire*.
  `held::firing_target` lets an infantry or vehicle holder keep such a member in its firing step
  (`behave::vehicle_step`, the infantry step), so the firing step answers *rearming* on it and
  neither clears it nor turns; range and aim read the member's old position, which it keeps
  until placement. `held::turn_start_drop` keeps the member for every holder kind, re-cited.
- **An older test re-read.** `claims_s0513_r62`'s two SW23 part f tests gave the member a target
  in its leaving turns and expected it kept, to show that no periodic check or allied drop ran
  there. Under r89 the member drops that target itself after the warp step, so those rows now
  expect it dropped in both runs, and only the departure, placement and arrival rows still tell
  a check from none.
- **No quirk, no state.** `quirks.warp_departure_keeps_beam` off refuses the lock, so its member
  sparkles and drops as one with no beam; the rules hold in both settings. Save format 93 and
  AI game format 56 unchanged.

## Claims

| Claim part | Status | Where, and why |
|---|---|---|
| S-0513 39a Sparkles: one in each turn whose tick is divisible by 24, from t_L + 2 to the phasing-end update, with no beam | tested | `claims_s0513_r89::claim39a_l6c_a_member_with_no_beam_sparkles_in_its_warp_turns_from_tl_plus_2_to_its_phasing_end` (a vehicle and an infantry unit, launch residues 23, 10, 9, 8 and 7 modulo 24, so t_L + 1, departure, appearance, arrival effects and phasing end each fall on a sparkle tick) |
| 39a Sparkles: the same with a beam, one per turn, none after the phasing end while the beam stays attached | tested | `claim39a_l6_a_member_with_a_beam_sparkles_once_per_turn_and_not_after_its_phasing_end` (the same five residues) |
| 39a Sparkles: none in t_L + 1 unless a beam was locked before that turn | tested | l6c (residue 23), `claim39a_a_lock_before_the_members_turn_of_tl_plus_1_makes_that_turn_sparkle` |
| 39a Sparkles: placed from the position before the warp step (old position to the appearance, landing point from arrival effects), claim 18's +120 leptons | tested | l6c, l6 (exact positions) |
| 39a Sparkles: after the phasing end, only while being erased again from a later lock | tested | `claim39a_a_later_lock_after_the_phasing_end_sparkles_again_once_per_turn` |
| 39a Sparkles: one per turn while being erased and phasing in at once | tested | `claim39a_a_member_phasing_in_and_being_erased_in_its_hold_sparkles_once` (a lock in a redirected landing's hold) |
| 39a Sparkles: a turn whose erasure update removes the member (open) | request | **Q-2377** (S-0513 Open questions, assigned to S-0514); interim: the sparkle comes before the erasure update, the order an ordinary turn of an object being erased already had, so such a turn sparkles and the sparkle's draws come first |
| 39a Sparkles: an ordinary animation with S-0109's effects; its draws before the warp step's, the expiry search's and the departure's | tested | `s0514_r14_claim18_the_sparkle_is_an_ordinary_animation_whose_creation_draw_precedes_the_warp_step` (a type with a ranged creation draw: the sparkle is an animation in play with its draw, made before the warp-out animation in the departure update). The damage and start and middle effects follow from `effects::anim` → `anims::spawn` as built (S-0109); no sparkle-specific test of them |
| 39a Drops: in each turn still being erased after the warp step, t_L + 1 to the expiry, with or without a beam; the expiry search's target dropped too | tested | `claim39a_a_member_with_no_beam_drops_a_given_target_in_each_leaving_turn_the_expiry_included` |
| 39a Drops: none from the departure for a member with no beam | tested | `claim39a_a_member_with_no_beam_keeps_a_given_target_from_its_departure_update` (departure, placement, arrival) |
| 39a Drops: a vehicle with a beam drops at its departure and while it phases in | tested | `claim39a_a_vehicle_member_with_a_beam_drops_in_its_departure_and_off_map_turns` |
| 39a Drops: an infantry member with a beam, by `phase_in_beam_spares_infantry` | tested | `claim39a_an_infantry_member_with_a_beam_drops_off_the_map_only_with_the_switch_off` |
| 39a Drops: an aircraft member | partly | the same `cut_short` row as a vehicle; no aircraft member test |
| 39a A lock while it waits to land, or at placement or arrival: being erased again, drops from then on, every kind | tested | `claim39a_a_lock_while_the_member_waits_to_land_makes_it_drop_whatever_its_kind` (infantry and vehicle). What such a lock does to the warp's own steps stays open (S-0306 claim 17), built as before |
| 39a The firer: an infantry firer within range keeps the member from the departure to the appearance; no shot, behaviour, cell and facing unchanged; *rearming*; beam attached after | tested | `claim39a_l3_an_infantry_firer_keeps_a_member_that_waits_to_land_with_its_turns_unchanged` (a redirected landing, so it waits more than one tick, with a check tick of S-0309 claim 29 in the span) |
| 39a The firer: the same for a vehicle firer | tested | `claim39a_a_vehicle_firer_keeps_a_member_that_waits_to_land_with_its_turns_unchanged` |
| 39a The firer: an outranking *illegal* (a deactivated firer's) drops the member at the check, the beam not released | tested | `claim39a_a_deactivated_vehicle_firer_drops_a_member_that_waits_to_land_at_the_check` |
| 39a The firer: its rearm countdown runs | partly | the firing step's reply to *rearming* is unchanged; no test reads the countdown |
| 39a The firer: a building firer | request | **Q-2376** question 1; interim: kept at turn start, its attack processing counts the off-map member as no target (S-0215 claim 32's no-target row) |
| 39a The firer: an infantry or vehicle firer beyond range of the old position | request | **Q-2376** question 2; interim: as one within range (kept; its approach tests range alone against the old position) |
| 39a The firer: an aircraft firer | seam | the aircraft specs' (S-0501, intake 80); meanwhile as a building firer: kept, no target in the firing step |
| 39a Switch `warp_departure_keeps_beam`: the sparkles and drops hold in both settings | tested | l6 runs both settings (off: the lock is refused, the sparkles are a beamless member's) |
| 39 Changes: the departure is not S-0309 claim 27's drop for a firer whose beam is attached | tested | as *The firer* |
| 51: the sparkle before the warp work, the clearing after it | tested | as *Sparkles* and *Drops* |
| S-0203 claim 4 (r89 Changes): the sparkle and the clearing are intrinsic exceptions | tested | as claim 51 |
| S-0309 claim 27 (r89 Changes): a firer whose beam is attached answers *rearming*, a deactivated firer's *illegal* drops at the check | tested | as *The firer* |
| S-0514 r14 claim 18: the sparkle is an ordinary animation, its creation draw before a warp member's warp step | tested | as the r14 test above |
| 39a Sparkles: the sparkle's draw before the expiry update's target search | tested | `s0203_r43_the_sparkles_creation_draw_comes_before_the_expiry_updates_target_search` (an expiry on a sparkle tick: the sparkle's creation draw moves the search's 4-to-8 draw, which a search made first would not feel). The landing check is in the same warp step, after the warp-out animation |
| 39a The firer: off the map, none of the rank-4 answers above *rearming* that read the target answers for the member | tested | the l3 and vehicle firer tests assert in each off-map tick that the member is not airborne (`airborne_by_height` reads an off-map object as not airborne, so `projectile_illegal`'s air row stays out), not protected and not hidden from the firer's house. These read the member's own state, not `on_map` |
| 39a The firer: a member protected or cloaked at its departure | seam | it keeps that state off the map (the cloak machine skips off-map objects, our engine's choice, not stated by claim 39a), so its firer gets *illegal* or *impossible*, ranked above *rearming*, and claim 29's check drops it. Consistent with "unless an outranking answer holds"; no spec says whether the cloak state may change off the map |
| 39a The firer: an infantry firer's attack state; the firing step's no-target row not reached | tested | `claim39a_l3_an_infantry_firer_in_its_attack_state_stays_in_it_while_the_member_waits_to_land` (attack state set at a non-fire, non-frozen stage before the departure: it holds to the appearance); the l3 test asserts `held::firing_target` is the member in each off-map tick and records the attack state and sequence as unchanged |
| 39a The firer: after a deactivated firer's drop the beam stays attached and the behaviour stays *attack* | tested | the deactivated test: the beam stays attached until the firer's next due attack processing with no target falls back to rest, and lets go there (S-0514 claim 14, *Release*), not at the drop |
| **S-0309 r95** claim 27: a firer whose temporal beam is attached to the departed member is the exception: *rearming* unless an answer ranked above it holds; a deactivated firer's *illegal* lets claim 29's check drop it; within range while the member waits to land it keeps it, makes no shot and neither moves nor turns | tested | `combat::check`'s `target_exists`, `held::firing_target`, `held::turn_start_drop` cite it; tests: the l3 infantry firer test (`measurable`, SW12 part l3), the vehicle firer test, the infantry attack-state test and the deactivated firer test |
| **S-0203 r43** claim 4: a warp member's sparkle before its warp work and S-0514 claim 17's clearing after it are not suppressed ordinary work | tested | `tick.rs` (the sparkle and `member_drop` calls) and `temporal::member_drop` cite it; tests: l6c, l6, the leaving-turns drop, the vehicle and infantry drop tests and the expiry draw-order test |

## Lanes

All before and after runs used one release build each: before is main at 4c57780, after is
this intake's code on that base. The branch was then rebased onto c782577 (intake 77, which
changes the arena set-up and parts of the sim); on that base the workspace tests, clippy, the
AI tests and `xtask gate` were run again.

| Lane | Before | After |
|---|---|---|
| AI tests (`glasnost-ai --include-ignored`) | 246 passed | 246 passed |
| M6c, M7a, M7b sweeps 1 to 16 | ok | identical (no diff line) |
| Arena (`glasnost-headless`) | 10 failing pairings | the same 10; all six output files byte-identical |
| AI quick lane, seeds 7 to 10 | recorded | identical apart from timings (`out.txt` and `ai-sides.txt`), all four seeds |
| Goldens (`xtask gate`, wasm32 duel goldens) | | unchanged; no golden moved |
