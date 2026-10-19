# M7a step 1b claim ledger (spec revisions that landed on main after e9ebf27)

This is part 2 of M7a step 1. Each spec is compared with the revision the code had taken in. The old state is the spec file at e9ebf27 (`git diff e9ebf27 c10e65a -- spec/<path>`); the intermediate revisions are listed for each spec. Every changed claim gets one class, as in M6c's step 1 ledgers ([../m6-claims/step1h-m6c.md](../m6-claims/step1h-m6c.md)):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason (a claim for a part not built yet);
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

Per-spec ledgers: [S-0610.md](S-0610.md), [../m6-claims/S-0513.md](../m6-claims/S-0513.md) and [../m6-claims/S-0515.md](../m6-claims/S-0515.md). S-0215, S-0203 and the glossary have no ledger of their own.

| Spec | Revision | Commits | Answers |
|---|---|---|---|
| S-0610 psychic weapons | 8 (from 4; 5 3cf56bc, 6 676118c, 7 2a5d7a9, 8 d11fe4a) | e9ebf27 | Q-1180 |
| S-0215 behaviours | 57 (from 54; 55 03e70fe, 56 953d069, 57 12285f4) | e9ebf27 | (with S-0610) |
| S-0203 tick order | 34 (from 33, 96beae9) | e9ebf27 | (with S-0610) |
| S-0513 superweapon effects | 15 (from 14, a4e44e5) | e9ebf27 | (with S-0610) |
| S-0515 tech buildings | 4 (from 3, 743ca59) | e9ebf27 | none |
| Glossary | 43 (from 41; 42 19290f6, 43 c235ae5) | e9ebf27 | none |

Requests:
- **Q-1180** (answered by S-0610 revision 8). The interim rule is retired. Under it, the countdown ran at the start of the unit's living update: before self-healing, before the first per-tick start, and also on an erasing update. Now it runs in the behaviour part, after the first per-tick start and before the processing, the per-tick search and every shot. Erasing and warp-only updates never get there. The interim's out-of-play and warp rows matched the answer.
- **Q-1166** (new). Interim rule 1: the late start does no fall-back to rest. Interim rule 2: the countdown comes after self-healing. S-0215 and S-0610 both list these two points as open questions.
- **Mind control (S-0601).** No changed claim in these revisions reaches mind control. S-0610's Scope bullet that points at S-0601 claims 3 and 13 is unchanged. So the parallel S-0601 track has nothing to hand on from here.

Save format: unchanged (no new state). Hashes: the late start changes the course of games, so the sim-gate goldens are regenerated on this branch (see the step's report).

## S-0610 revisions 5 to 8 (answer Q-1180)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1180), approval date | (c) | |
| Scope, gas particles: "depends only on its imported data" (was: the stock chaos drone creates none) | satisfied | The importer copies a weapon's data and the area pulse is S-0302's. Gas clouds are S-0613, a later M7a step. |
| Scope, *Healing itself* (new pointer) | pointer | S-0314, S-0520 claim 14. |
| Terms, *Late start* (new) | pointer | S-0215 claim 11; built (below). |
| Terms, *Countdown update*, *Excluded update, warp-only update* (new) | changed | Built as claim 5 below. |
| 5: "In each countdown update" (was: in each of its own updates) | changed | `psychic::countdown_turn` is now called from `process::turn`, the behaviour part of the update (`tick::object_turn` no longer calls it). Erasing updates (`temporal::turn`) and warp-only updates (`warp::turn`) return before that point. Tests in `claims_s0610_r8.rs` and `claims_s0203_warp_turn.rs` (rows below). |
| 5, *Which updates are countdown updates*: not tied to the behaviour timer, the behaviour, the owner or a target | satisfied | The call comes before the due test in `process::turn`. `p9_d_no_countdown_while_being_erased` shows the twin counting down while its timer is not due. `claim_5_countdown_runs_in_the_turn` (`claims_s0610.rs`) is the plain case. |
| 5, out of play: no updates, time kept, resumes with the time it had; an open-topped passenger counts down | satisfied | A passenger of a closed transport is out of the update order. `claim_5_out_of_play_keeps_the_time_open_topped_counts` checks both kinds of transport. Boarding and leaving do not touch the berserk state (no code reads it). |
| 5, excluded updates: tunnel, being erased, leaving stage of a chrono move; landed aircraft being erased | changed (erasing); satisfied (leaving stage, landed aircraft); not built (tunnel) | Being erased: before, the countdown ran ahead of `temporal::turn`; now it does not. `p9_d_no_countdown_while_being_erased` failed with the old code. Leaving stage: `warp::turn`, test below. Landed aircraft: `temporal::turn` also returns for aircraft, and an aircraft's countdown sits at the start of its behaviour part (`process::turn`, before `air::behaviour_turn`). Tunnels are not built (a seam in `seams.rs`). |
| 5, warp-only updates from the first pending update through arrival effects (no step, none made up later); in arrival hold one step after the hold-end test | satisfied | `berserk_countdown_is_ordinary_work_of_the_warp_turn` (`claims_s0203_warp_turn.rs`): no step in any stage before the hold, the wait-expiry update (`Stage::Expired`) included; one step per hold update; no catch-up. |
| 5, *Effects in the update where the countdown ends*, no berserk shot in the expiry update, and later ordinary targets may be fired at | satisfied | The countdown precedes the processing, the per-tick search and both firing steps. `p9_a_no_berserk_shot_in_the_expiry_update`: with the expiry on the update of the first berserk shot there is no shot then or later; one update later and that shot is fired. |
| 5, the queued behaviour is not yet in charge: no guard search, hold or processing in the expiry update; a ready unit's late start may make it current; a vehicle moving under hunt keeps hunting | changed | The countdown now follows the first per-tick start, so the expiry's queued behaviour waits for the late start. Before, a ready unit made it current and processed it in the expiry update. Tests: `b46_infantry_expiry_late_start` (failed with the old code) and `p9_b_vehicle_keeps_hunting_while_moving`. |
| 5, a queued behaviour that never started is replaced by guard; a neutral unit under hunt with nothing queued: nothing changes | satisfied | `claim_5_a_queued_hunt_that_never_started_is_replaced`, `claim_5_neutral_unit_under_hunt_is_unchanged`. |
| 5, Backing and Confidence | (c) | |
| 6, *Healing is unaffected* (new bullet) | satisfied | No healing code reads the berserk state. `claim_6_healing_hit_keeps_the_time`. A negative-amount berserk hit goes through stage 3 of S-0302 unchanged (`psychic::enter`). Its order against the countdown is an open question of the spec (Q-1166 question 2). |
| 8: gas particles depend only on the data | satisfied | as the Scope row. |
| F2: countdown updates only; out-of-play, excluded and warp-only updates leave *t*; a berserk shot only when *t* > 0 after the step | changed | as claim 5. |
| Constants: the step is per countdown update | changed | as claim 5. |
| State and ordering, the three countdown rows | changed | as claim 5. |
| Edge cases, `quirks.berserk_zero_duration`: "until its next countdown update" | satisfied | `zero_duration_quirk_both_positions`. |
| P5 wording, P9 (new), Open questions | (c) | Measurement text. P9 parts a, b, d, e and f are mirrored by the tests above. |

## S-0215 revisions 55 to 57

| Claim | Class | Notes |
|---|---|---|
| Header, Terms (*late start*), *Chrono move, leaving stage*: from the member's first update after launch until departure, including the wait-expiry update | pointer / satisfied | S-0513 claim 39 and the glossary. `warp::turn` returns the whole turn from `Stage::Pending` through `Stage::Expired` (`berserk_countdown_is_ordinary_work_of_the_warp_turn`). |
| 2: on a tunnel, erasing or leaving-stage tick no per-tick start takes effect (first or late), nor the berserk countdown | changed (countdown, late start); satisfied (first start) | The late start and the countdown are new code placed after `temporal::turn` and `warp::turn` return. `p9_d_no_countdown_while_being_erased`. |
| 2, Backing and Confidence | (c) | |
| 3 (a) and (b): the first per-tick start is processed on its tick; a late start is processed on the next update | changed | `process::late_start` uses start-next, which sets the timer to expired. The update's processing has already run. `claim_11_late_start_after_a_vehicle_stops` and `b46_infantry_expiry_late_start` check the started, not yet processed state. |
| 11, *first per-tick start* (renamed) | (c) | |
| 11, the late start of infantry and vehicles (new): readiness with this update's movement counted; infantry's firing test reads the state earlier updates left; after the drop rule and the per-tick search; none for buildings and aircraft; no quirk switch | changed | `process::late_start`, called in `tick::object_turn` right after the locomotor step and the passenger sync. That is before the depot retry, the parasite step, the separate vehicle firing step and turret, and the infantry sequence rules and firing, so the infantry sequence it reads is the one earlier updates left. Tests: `claim_11_late_start_after_a_vehicle_stops`, `b46_infantry_expiry_late_start`; the existing tests whose end-of-tick reading moved, all judged against revision 57: `claim_10_second_route` (`claims_s0402.rs`), `claim_26_slave_processing_capacity` and `s0312_claim_31_removed_master` (`claims_s0402_slaves.rs`), `claim_4_refusal` and `claim_4_gatherer_silent` (`claims_s0415.rs`), `a_charger_keeps_the_guard_delay_after_taking_a_building` (`claims_s0518_r3.rs`). In each, a behaviour queued in the update is now current at its end, not yet processed. |
| 11, fall-back to rest: at the first per-tick start; at the late start not established | satisfied (first); interim (late) | Q-1166 question 1. Interim rule: no fall-back at the late start. |
| 11, the late start is subject to claim 2's exclusions and to S-0513 claim 51's suppression | satisfied | It is placed after both returns. |
| 33 (b): the per-tick search and the drop rule judge the behaviour before any late start | satisfied | Both run inside `process::turn`, before the locomotor. |
| State and ordering, the new late-start bullet | changed | as claim 11. |
| B3 case wording ("for as long as the vehicle is still moving"), B46 (new), Open questions, Cross-spec changes | (c) | B46 is mirrored by `b46_infantry_expiry_late_start`. |

## S-0203 revision 34

| Claim | Class | Notes |
|---|---|---|
| 4: the berserk countdown is among the ordinary work a warp update suppresses; in arrival hold it follows the hold-end test | satisfied | `berserk_countdown_is_ordinary_work_of_the_warp_turn`. |
| 4, Backing (S-0610 P9 part e replayed on our engine) | (c) | |

## S-0513 revision 15

| Claim | Class | Notes |
|---|---|---|
| 48: the berserk countdown takes no step on the leaving-stage turns (vehicle, infantry; design for a landed aircraft) | satisfied | as S-0203 claim 4. |
| 51: the berserk countdown is suppressed ordinary work; an update count, not an elapsed clock | satisfied | as S-0203 claim 4. Unlike the reload countdown (`combat::fire::reload_elapsed`), it takes no step in a warp update. |
| 51, Backing | (c) | |

## S-0515 revision 4

| Claim | Class | Notes |
|---|---|---|
| Terms, *Tech building*: the list of stock examples is removed | (c) | No rule depended on it; the code reads the type's fields only. |

## Glossary revisions 42 and 43

| Entry | Class | Notes |
|---|---|---|
| Batch note: S-0608, S-0611, S-0612 and S-0615 are approved | (c) | S-0611 built at M7b step 3.4 ([S-0611.md](S-0611.md)). |
| *pass counter*, *clock stage*, *crate places* (wording, units) | (c) | |
| *tech building*: the stock examples are removed | (c) | as S-0515. |
| *leaving stage*: S-0513 owns it; it starts at the first update after launch, not at departure, and ends at departure; other specs use this sense, the wait-expiry update included | satisfied | as S-0215's Terms row. |
| G3 (*hold* in the sense of S-0201), the G-scenario falsifier wording | (c) | |
