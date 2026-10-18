# M7a step 1l claim ledger (S-0215 revision 67, S-0314 revision 29, S-0606 revision 7, and five more)

This is part 12 of M7a step 1. It covers the approved revisions that reached main after the step 1k intake, from 652bafc to 77ead43: S-0215 revisions 65 to 67, S-0603 revision 8, S-0606 revisions 6 and 7, S-0207 revision 39, S-0314 revision 29, S-0517 revision 10, S-0402 revision 31, S-0206 revision 16 and the index. `git log 77ead43..main -- spec/` is empty at the end of the step. Every changed claim gets one class, as in [step1k-m7a.md](step1k-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered. S-0606 revision 7 drops its open question 6 and renumbers the later ones; no code cites them.

| Spec | Revision | Answers |
|---|---|---|
| S-0215 behaviours | 67 (from 64) | Q-1160, Q-1161 |
| S-0207 ground vehicle movement | 39 (from 38) | Q-1160 question 2 |
| S-0206 turn rates | 16 (from 15) | none (the Terms pointer to S-0215 claim 11) |
| S-0314 healing and repair | 29 (from 28) | Q-1162 |
| S-0606 vehicle bunker | 7 (from 5) | Q-1163, Q-1164 |
| S-0517 deployable units | 10 (from 9) | none (the released in-place vehicle, S-0606's old open question 6) |
| S-0603 grinder | 8 (from 7) | none (the late start's current predicate; backing) |
| S-0402 harvest cycle | 31 (from 30) | none (the turn's end tick) |
| `spec/README.md` | | index rows moved and revised: (c) |
| `spec/requests/` Q-1160 to Q-1164 (answered) | | request status: (c) |

**Retired interims.**
- Q-1160 question 1 is answered by S-0215 claim 11 as the interim had it: a turn ends by tick number, so the first start of its end tick sees it ended, and the behaviour started there is processed in that tick. Question 2 is answered by S-0207 claim 11: as the interim had it, the attack order removes both destinations and the body completes its turn without the cancelled step. S-0207 lets the old path steps survive; ours are cleared with the destination, which the claim allows ("need not erase"), and retaining them would keep S-0206's in-progress test true for a vehicle with no destination.
- Q-1161 is answered by S-0215 claim 11 and B48, not as the interim had it. The interim kept "movement in progress" (a track, a turn in place, path steps). The answer is narrower: a body turn, or a heading point with a positive integer step while the locomotor moves the vehicle. Changed (below).
- Q-1162 is answered by S-0314 claim 17, not as the interim had it: arrival requests no turn. The step 1k rotation at arrival is gone, and so is the tethered vehicle's turn and arrival report in the *enter* processing (revision 29: no ordinary depot supplies the completed-entry confirmation).
- Q-1163 is answered by S-0606 claim 20 as the interim had it (one S1 row, then the depot's decision on the state it left; the depot's request replaces S1's queued *guard*).
- Q-1164 is answered by S-0606 claim 12 as the interim had it (S-0415 decides: no deploy-fire rule for a released vehicle).

Every interim citation is gone from the code, the tests and `development.md`. The step 1k ledger keeps its rows and gains a note that the five requests are answered.

**New requests:** none. The block Q-1165 and Q-1340 to Q-1349 is unused.

**Seized vehicles.** None of these revisions says anything about seized vehicles or the magnetic lift.

## S-0215 revisions 65 to 67

| Claim | Class | Notes |
|---|---|---|
| 11: for a vehicle with the driving locomotor, "moving" is a body still turning, for any reason, or all three of: the locomotor moves it (S-0206's term), a heading point, and an integer movement step *s* above 0; a retained destination alone is not enough | changed | `process::moving` calls `process::drive_moving` for a driving vehicle (by type or temporarily) and keeps `occupancy::in_progress` for every other actor. S-0206's term is a locomotor destination (`drive::goal_present`) or a track target off the vehicle's x and y; the heading point is the track; *s* is `drive::progress` at the read. Tests: `claim_11_retained_goal_and_path_alone_are_ready`, `claim_11_heading_point_and_positive_step` (*s* > 0 not ready, *s* = 0 ready), `claims_s0215_r67.rs`. |
| 11: a turn ends by tick number; the queued behaviour becomes current at the first start of tick *t*₀ + Θ and is processed that tick | satisfied | `Turning::turning(t)` is false from *t*₀ + Θ. Test: `claim_11_b47_turning_vehicle_keeps_a_queued_attack` (revised: the target lies along the turn's direction; *attack* has start tick *t* and is processed in tick *t* with a positive delay). |
| 11: an idle vehicle in *area guard*, *sleep* or *attack* starts an ordered *move* at its next first start despite the locomotor destination; a blocked or retry-waiting vehicle with no turn and no heading point (or *s* = 0) is ready | changed | the move from rest was already ready (assignment clears the path). The blocked, retry-waiting and between-tracks cases change: path steps and a turn-in-place mark no longer hold a driving vehicle back. Test: `claim_11_b48_fresh_move_from_area_guard`. |
| 11 (late start): judged with the movement state the update has left; for a driving vehicle the current predicate, not a remembered displacement | satisfied | `process::late_start` calls `process::ready`; no readiness test reads `moved_tick`. Test: `claim_11_queued_guard_starts_by_the_current_predicate` (`claims_s0610_r8.rs`, which replaces `claim_11_late_start_after_a_vehicle_stops`: that test assumed the vehicle stays "moving" until it stops, which revision 67 no longer gives). |
| 11a: a failure after the first start can reach the late start; a first start processes in its turn, a late start on the next update | satisfied | the late start is general. Existing harvest departure tests (`claims_s0215_harvest_departure.rs`). |
| 29, depot table: movement arrival requests no map-east turn, tethered or not; an ordinary depot supplies no completed-entry confirmation; an existing driving turn holds *sleep* only by claim 11; ships and hovers are not held | changed | as S-0314 revision 29 below. |
| State table row *move*: rest queued at a due processing with no destination, movement or queue, current at the next permitted start including the late start | satisfied | as built (`process::late_start`). |
| Q-1160 and Q-1161 paragraph, Import mapping row, backing and confidence, B3 e, B38, B47, B48, B21, open questions, cross-spec changes | (c) | |

## S-0207 revision 39

| Claim | Class | Notes |
|---|---|---|
| 11: an attack order during a pre-step turn removes the order-level and locomotor destinations; no heading point; the old steps authorize no track; the body completes its existing turn without displacement; no independent pre-step hold | satisfied | `drive::tick` ends the turn in place at its next update when the destination is gone (citation moved from the Q-1160 interim); `orders::apply_behaviour` clears the steps with the destination (cited "need not erase"). Test: `claim_11_b47_turning_vehicle_keeps_a_queued_attack` (no destination, no route, no track, no goal; the facing completes its turn). |
| D11 c, header | (c) | |

## S-0206 revision 16

| Claim | Class | Notes |
|---|---|---|
| Terms, *locomotor moving the vehicle*: S-0215 claim 11's readiness is not this term plus turning; without a turn it also needs a heading point and a positive step | pointer | `occupancy::in_progress` is unchanged; `process::drive_moving` is S-0215's predicate. |

## S-0314 revision 29

| Claim | Class | Notes |
|---|---|---|
| 17: movement arrival requests no map-east turn, tethered or not, and schedules none; if ready at the late start *sleep* starts there | changed | the rotation in `depot::movement_arrival` is removed. Test: `claim_17_arrival_requests_no_turn` (tethered and untethered: no turn, *sleep* current after the tick, facing kept). |
| 17: an ordinary depot's repeated request supplies no completed-entry confirmation, with or without the pad flag or a tether | changed | the tethered branch of `depot::process_enter` no longer turns the vehicle to 16384 or queues *repair* and *sleep*; it returns the standard delay. The constant 16384 is gone from the code (the Constants row now names a confirmation no ordinary depot supplies). Test: `claim_17_pad_depot_tethers_in_the_enter_processing` (revised). |
| 17: an existing driving turn holds *sleep* by S-0215 claim 11 only; ships and hovering vehicles have no body-turn hold | satisfied | `claim_17_existing_turn_holds_the_arrival_sleep` (current at the first start of the end tick), `claim_11_turning_clause_is_the_driving_locomotors`. |
| Constants row, state table row, Import mapping row, H16 a, confidence | (c) | |

## S-0606 revisions 6 and 7

| Claim | Class | Notes |
|---|---|---|
| 12: a released vehicle skips passenger unloading and the deploy-fire rule; harvesting, then building deploy, then the in-place cycle; none: 450 ticks, no draw | satisfied | `bunker::continue_unload`, as built under Q-1164's interim. Test: `claim_12_bunkered_deploy_fire_is_skipped` (renamed from `claim_12_bunkered_deploy_fire_interim`; now also checks no draw). |
| 12: a released simple-deployer reaches the in-place cycle in the same processing, whatever its passenger capacity; under the default quirk its *guard* replaces the queued *move*, at once or at completion | satisfied (newly tested) | `claim_12_released_simple_deployer_deploys_in_place` (with and without passenger capacity: deployed, 450 ticks, *guard* current, *move* gone), `claim_12_released_simple_deployer_with_presentation` (deploying, 1 tick, *move* still queued; *guard* replaces it at completion). |
| 20 (Q-1163): one S1 row; the depot decides on the state that row left; the depot's queued behaviour replaces S1's *guard*, otherwise it stays; the depot's delay | satisfied | as built under Q-1163's interim (`process`'s combined arm). Test: `claim_20_one_s1_row_then_the_depot` (S1's no-vehicle row runs, the depot's guard is current, nothing queued, delay 1). |
| Table S2 row, Import mapping rows, K6b, K9, open questions renumbered | (c) | |

## S-0517 revision 10

| Claim | Class | Notes |
|---|---|---|
| 1: the in-place cycle after the applicable passenger, deploy-fire, harvesting and building-deploy rules; bunker release is not a stopping rule; the released in-place vehicle's continuation | satisfied | `claim_1_the_in_place_cycle_is_the_last_rule`; the bunkered branch as S-0606 claim 12 above. |
| P8 a, backing, confidence | (c) | |

## S-0603 revision 8

| Claim | Class | Notes |
|---|---|---|
| Terms (*moving*): the current movement state of S-0215 claim 11, not remembered displacement | changed | through `process::moving` (S-0215 above). |
| 3: ready at the failing tick's late start by the full predicate, even after displacement earlier in the update; otherwise the first eligible start | changed | as S-0215 claim 11. The grinder's own tests (`claim_3_failed_approach_gives_up`) are unchanged. |
| F1: passenger levels and the parasite are refunded for a vehicle entrant only | satisfied | `grinder` pays passengers and the parasite only for a vehicle; an infantry entrant's parasite is released by S-0614 claim 15. |
| Summary, cross-spec list removed, 2, 5, 10 to 14, Edge cases, G6, G7, G9 to G11, open questions | (c) | |

## S-0402 revision 31

| Claim | Class | Notes |
|---|---|---|
| 10 row 4, 11 *Turning body*, *Refinery sold while unloading*, *Order while unloading*: a turn ended by tick number permits the first start of the end tick and processing in that tick | satisfied | the shared readiness (`process::ready`); B47's test. |
| Backing, confidence | (c) | |

## Formats and gate

- Save format 54 and AI game format 22, both unchanged: this step adds no saved or hashed state. The predicate reads the existing drive state; the depot and bunker changes remove code.
- Goldens: Regenerated. All four change: `combat.txt` from tick 7 (its first hashes after the opening moves), `duel.txt` from about tick 27 360, `duel-m6c.txt` from about tick 9 230 and `duel-m7a.txt` from about tick 6 760. The readiness predicate lets a driving vehicle start and process its behaviour on more ticks (an accelerating vehicle whose step truncates to 0, a vehicle between tracks, one waiting for a blocked path), and the ordinary depot no longer turns arriving vehicles or confirms a tethered entry. `duel.txt` now ends 320 ticks later (3 063 lines, from 3 031).
- The 16-seed sweep: `GLASNOST_SWEEP=1-16`, with no spec revisions on main since the base. Seeds 4 and 6 end at ticks 10 895 and 10 918 with Alpha's win; every other seed plays all 12 000 ticks. The gate seed 3 plays all 12 000 ticks and every counter but the sensor orders (2) is at least 3. **Two seeds fall short**: seed 8 gives no takeover orders (0), and seed 9 gives one takeover order and no grinder entry order or ground unit (0). Minimum / median / maximum:

| Counter | Min | Median | Max |
|---|---|---|---|
| berserk weapon orders (S-0610) | 11 | 23 | 32 |
| units made berserk (S-0610 claim 4) | 30 | 50 | 57 |
| psychic sensor orders (S-0610 claim 9) | 2 | 2 | 2 |
| hostile intent seen by a sensor (S-0610 claims 9 to 11) | 3 058 | 4 210 | 5 308 |
| psychic reveal orders (S-0610 claim 13) | 9 | 12 | 12 |
| psychic reveals fired (S-0610 claims 13 and 14) | 9 | 12 | 12 |
| slave-miner buildings (S-0402 claim 14) | 3 | 4 | 14 |
| slaves at work (S-0402 claim 14) | 2 941 | 6 165 | 9 293 |
| gattling stage rises (S-0609 claim 2) | 6 | 17 | 37 |
| gattling at its top stage (S-0609 claim 2) | 659 | 1 845 | 3 657 |
| takeover orders (S-0601) | **0** | 10 | 25 |
| takeovers (S-0601 claim 6) | 5 | 13 | 22 |
| gas clouds (S-0613 claims 2 to 4) | 36 | 108 | 285 |
| gas hits (S-0613 claim 13) | 67 | 218 | 1 136 |
| absorber entry orders (S-0605) | 5 | 17 | 120 |
| infantry absorbed (S-0605 claims 1 and 7) | 5 | 8 | 13 |
| grinder entry orders (S-0603) | **0** | 5 | 14 |
| units ground (S-0603 claims 5 and 8) | **0** | 4 | 13 |
| tank bunker build orders (S-0606) | 4 | 4 | 4 |
| bunker entry orders (S-0606 claim 3) | 3 | 8 | 15 |
| vehicles bunkered (S-0606 claim 7) | 3 | 8 | 10 |
| bunker exit orders (S-0606 claim 12) | 3 | 7 | 9 |

  Why: in seeds 8 and 9 the Yuri-style side's controllers take their victims by their own target acquisition (takeovers happen in both seeds) rather than from an idle or guard state the AI's mind-control think would send, and both houses run out of credits by about tick 8 500. In seed 9 the grinder only stands from about tick 7 500, when there is no money left to replace losses, and no idle controlled, wounded or surplus unit comes up for it.

  No setup-only fix was found. Each was tried over all 16 seeds and then reverted. Starting credits 20 000: seeds 8 and 9 short exactly as before. Start offset 14: seed 8 passes, but seeds 3, 6 and 9 fall short. Offset 15: smallest minimum 2 (infantry absorbed in seeds 1 and 6). Offset 18: smallest minimum 1 (seeds 3 and 5). A setup change moves which seeds fall short; it does not remove the shortfall. The fixture is left unchanged and the decision goes to the lead. Options: accept seeds 8 and 9 as known shortfalls (the gate seed is unaffected), or allow an AI-side change, which is not setup-only.
