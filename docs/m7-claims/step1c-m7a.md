# M7a step 1c claim ledger (spec revisions that landed on main after f368516)

This is part 3 of M7a step 1. Each spec is compared with the revision the code had taken in. The old state is the spec file at f368516 (`git diff f368516 96cf909 -- spec/<path>`; the three later hand-offs S-0601 revision 10, S-0011 revision 24 and S-0609 revision 4 came in with a merge of main at 8f5b743, `git diff 96cf909 8f5b743 -- spec/<path>`); the intermediate revisions are listed for each spec. Every changed claim gets one class, as in [step1b-m7a.md](step1b-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason (a claim for a part not built yet);
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

Per-spec ledgers: [S-0601.md](S-0601.md), [S-0610.md](S-0610.md), [../m6-claims/S-0509.md](../m6-claims/S-0509.md), [../m6-claims/S-0513.md](../m6-claims/S-0513.md), [../m4-claims/S-0312.md](../m4-claims/S-0312.md) and [../m4-claims/S-0314.md](../m4-claims/S-0314.md), [S-0609.md](S-0609.md). S-0011, S-0203, S-0214, S-0215, S-0701 and the glossary have no ledger of their own.

| Spec | Revision | Commits | Answers |
|---|---|---|---|
| S-0601 mind control | 10 (from 5; 6 f46469c, 7 db03e3b, 8 aa32cfa, 9 96cf909, 10 79ea771) | f368516 | Q-1200, Q-1201, Q-1202, Q-1203 |
| S-0610 psychic weapons | 9 (from 8, ea2fdab) | f368516 | Q-1166 question 2 |
| S-0215 behaviours | 59 (from 57; 58 a75a9ac, 59 dd1a9a9) | f368516 | Q-1166 question 1 |
| S-0203 tick order | 35 (from 34, 9e8a158) | f368516 | (with S-0601) |
| S-0214 order targeting rules | 39 (from 38, 3d8a0e0) | f368516 | (with S-0601, Q-1200) |
| S-0509 engineers and capture | 6 (from 5, e7b6afc) | f368516 | (with S-0601, Q-1203) |
| S-0312 death and destruction | 30 (from 29, da73feb) | f368516 | (with S-0011) |
| S-0314 healing and repair | 23 (from 22, 3a4729b) | f368516 | (with S-0610) |
| S-0513 superweapon effects | 17 (from 15; 16 9ae47f5, 17 313a784) | f368516 | (with S-0601) |
| S-0011 map triggers (MP subset) | 24 (from 21; 22 924e86d, 23 016cbd3, 24 93dbe6b) | f368516 | (with S-0601, Q-1203) |
| S-0609 gattling spin-up | 4 (from 3, 8f5b743) | f3d24e1 | Q-1220 (in part) |
| S-0701 sound selection | 17 (from 15; 16 1d760c8, 17 1cfca36) | f368516 | (with S-0601) |
| Glossary | 44 (from 43, b5bb9e7) | f368516 | none |

Requests:
- **Q-1166** (answered by S-0215 revision 59 and S-0610 revision 9). Interim rule 1 is retired: the late start now falls back to rest (S-0215 claim 11). Interim rule 2 is retired: S-0610 claim 6 says self-healing against the countdown has no observable effect for a unit with no overload pulse, so the existing order (countdown after self-healing) stands as an engine choice; for an unlimited controller S-0601 claim 16 places it.
- **Q-1200** (answered by S-0601 revision 9 claim 15). The interim already refused a controlled MCV; now only control by a controller refuses (a dominated MCV deploys), and the deploy command skips the controlled one.
- **Q-1201** (answered by S-0601 revision 9 claim 16). Interim: after the bookkeeping, before self-healing, also while being erased. Now after the first per-tick start and the berserk countdown, before self-healing, behaviour, movement and firing, in the countdown's updates only.
- **Q-1202** (answered by S-0601 revision 9 claim 8). The interim (an absent mark never selects its table) becomes the design rule; the importer warning is new.
- **Q-1203** (answered by S-0601 revision 9 claim 5 and S-0509 revision 6 claim 12). Interim: events 6 and 44 were raised but not judged (only an any-event trigger fired), and a taken building made no announcement. Now both events are judged (S-0011 claim 8) and the building announces on takeover and release.
- **Q-1220** (answered by S-0609 revision 4, items 1 and 2). The interim rules (negative settings used as 0, saturating spin arithmetic and elapsed-tick count) become the spec's Edge cases rules, which the spec still marks as proposed pending the lead's decision; the code already did this, so only the citations and warning texts change, plus a saturation test. Item 3 (a later layer lowering the stage count below 2) stays open in the spec: filed as Q-1151.
- **Q-1151** (new, filed here). Interim rule, kept from Q-1220's: while the stage count in force is below 2 the held thresholds are not used (all 0), and the importer warns.
- **Q-1150** (new, filed here). The default position of `quirks.removed_tag_attack_event` is the original's fault, which our engine cannot reproduce. Interim rule: in both positions event 44 is raised only if the target still carries a tag after the event-6 raise, and then on that tag.

Save format: unchanged (no new state). AI game format: unchanged. Hashes: the turn order of unlimited controllers, their erasure exclusion and the late start's fall-back change the course of games, so the sim-gate goldens are regenerated on this branch.

## S-0601 revisions 6 to 9 (answer Q-1200 to Q-1203)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests`), approval date, Terms (*mind-controlled*: one consumer reads only the controlled half), the new Terms (*tag*, *house value*, *triggering house*, *player-position value*, *one-time and all-carriers*, *deploy readiness*, *house-order position*) | (c) / pointer | S-0011, S-0214, S-0009. |
| 1: initial overload countdown of 30 *overload updates* (was ticks) | satisfied | The countdown was always stepped once per `overload_turn` call; claim 16 below decides which updates call it. Constants row likewise. |
| 4: stock wording removed; missing-record backing | (c) | |
| 5, *The two attack events*: order (every event-6 action completes before event 44 is judged; both before the landing test), cause (the firer; its owner at the raise is the attacking house) | changed | `mind::detonation` raises 6 then 44 with the firer as causer, re-reading the target's tag between them. `tags::claim_5_attack_events_order_cause_and_judging`, `tags::claim_5_event_6_action_seen_by_event_44_and_the_test`. |
| 5, *Judging* (S-0011 claim 8's) | changed | See S-0011 claim 8 below. |
| 5, *Tag removed by the first raise*, `quirks.removed_tag_attack_event` | changed | New switch in `quirks.rs` (default on), schema regenerated. Off: as the spec says. On: Q-1150 interim, the same as off. `tags::removed_tag_attack_event_both_positions`. This also turns main's failing `every_spec_quirk_has_a_switch` green. |
| 5, Backing and Confidence | (c) | |
| 6, *Announcement for a taken building* | changed | `Cause::MindControl` now announces (`structures::ownership::Cause::announces`), so `mind::set_owner` pushes `Effect::Captured { announce: true }` for a building; the audio crate's entry choice does not read the cause (S-0701 claim 39). `claim_6_building_taken`. |
| 7: backing wording | (c) | |
| 8, *Marks absent from every rules layer* | changed (importer); satisfied (rule) | The never-select rule was the Q-1202 interim; now cited as the design rule. Test `claim_8_absent_marks_never_select_their_table` (both absent; negative money; an absent low-money mark keeps the wounded table's place). The importer warns once for each absent mark when a fate table is non-empty: `absent_fate_marks_warn`. |
| 8, Backing | (c) | |
| 9, the release of a building announces (controlling house old, original owner new) | changed | as claim 6. `claim_9_released_building_announces`. |
| 13: permanent control blocks every consumer except the yard deploy readiness | changed | as claim 15. |
| 15: wording of the deploy-into example | (c) | |
| 15, *Which controlled vehicles can deploy*: not deploy-ready into a construction yard while controlled by a controller | changed | `econ::deploy::deploy_ready` reads `mind::controlled_by` (was: mind-controlled, permanent control included). `controlled_mcv_does_not_deploy_into_a_yard`. |
| 15, the deploy command gives no order; no refusal sound or message | changed | `targeting::key_command` requires `deploy_ready` for a deploys-into vehicle. `deploy_command_skips_a_controlled_mcv`. |
| 15, the refusal cursor (S-0214 table VD) | not built | `targeting::vehicle_deploy` has no row VD2 for a deploys-into vehicle: seam. |
| 15, a queued deploy does not survive the takeover's guard reset | satisfied | `queued_deploy_does_not_survive_the_takeover`. |
| 15, deploy-ready again after release | satisfied | `released_mcv_deploys`. |
| 15, trigger-held and permanently controlled vehicles are not refused; a dominated MCV's yard is the dominating house's | changed | as above. `dominated_mcv_deploys_for_the_dominating_house`. Trigger-held: no control record, so not refused (campaign state, no test). |
| 15, a taken vehicle that deploys into another building | satisfied | `claim_15_deploy_makes_control_permanent`. |
| 16, *Where the overload update falls*: after the berserk countdown (and the first per-tick start), before self-healing, behaviour, movement and firing | changed | `process::turn` is split into `process::head` (first per-tick start, countdown) and `process::tail`. For an unlimited controller that is not being erased, `tick::object_turn` runs `head`, then `mind::overload_turn`, then the rest of the turn with `tail`. Putting the controller's first start and countdown before its self-healing is our choice (S-0610 claim 6, S-0314 claim 9 leave it open). `claim_16_heal_follows_the_pulse`, `claim_16_steps_with_the_berserk_countdown`. |
| 16, a lethal pulse ends the update (no heal, behaviour, movement or firing); non-vehicle controllers the same | changed | Return after the pulse when the controller is dead or gone. `claim_16_lethal_pulse_ends_the_update` (a surviving pulse's update still fires its shot; a lethal one neither fires nor heals). |
| 16, the damaged-smoke draw follows the pulse draws | satisfied | The aircraft smoke draw is in `air::end_turn`, after the pulse. |
| 16, *Updates in which the overload update runs*: not while being erased | changed | Excluded through `temporal::erased`; before, the pulse ran while being erased. `claim_16_no_overload_update_while_erased` (failed with the old code). |
| 16, out of play (a closed transport) no update; open-topped passenger steps | satisfied | `claim_16_transports`. |
| 16, chrono leaving stage and warp-only updates suppressed, kept, no catch-up; one step per hold update after the hold-end test | satisfied | The pulse follows `warp::turn`'s return. `overload_update_is_ordinary_work_of_the_warp_turn` (`claims_s0203_warp_turn.rs`). |
| 16, tunnel travel | not built | Tunnels are a seam. |
| 16, Backing and Confidence | (c) | |
| State and ordering, the overload bullet | changed | as claim 16. |
| Edge cases (three new bullets) | changed | as claims 5, 8 and 15. |
| Import mapping, the two marks' absent default | changed | as claim 8. |
| MC1, MC3, MC7, MC8, MC9, MC11, MC12, MC13 wording; MC14 (new); Open questions; Changes named for other specs; Questions for the project lead | (c) | MC14 parts a and b are mirrored by the claim 5 and claim 16 tests above. The lead's question on the quirk is Q-1150's subject. |

## S-0601 revision 10

| Claim | Class | Notes |
|---|---|---|
| Terms, *house-order position* replaced by *house number* (S-0009 Terms, from 0); MC14 wording | (c) | The code already uses the house number (`HouseId.0`). |

## S-0610 revision 9

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1166) | (c) | |
| Terms, *Unlimited controller* (new) | pointer | S-0601. |
| 5: an unlimited controller's overload update runs in the same updates as the countdown, berserk or not | changed | as S-0601 claim 16. |
| 6, a negative-amount berserk hit relative to the countdown: open | (c) | Still an open question (scenario P3). |
| 6, *Self-healing against the countdown*: no observable effect for a unit with no overload pulse; for an unlimited controller S-0601 claim 16's | satisfied | Retires interim rule 2 of Q-1166. The countdown stays after self-healing for other units. |
| Open questions | (c) | |

## S-0215 revisions 58 and 59 (answer Q-1166 question 1)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1166), Terms (*overload pulse*), the Q-1166 note | (c) / pointer | S-0601. |
| 11, the late start's backing split | (c) | |
| 11, the late start falls back to rest for an infantry unit with both slots empty; the queued behaviour becomes current there, not processed until the next update; that update's firing check sees the result | changed | `process::late_start` calls the fall-back first, then start-next. It runs before the infantry firing step. `claim_11_late_start_falls_back_to_rest`, `claim_11_no_late_fall_back_for_a_vehicle` (`claims_s0215_r59.rs`). |
| Open questions (the late-start question removed) | (c) | |

## S-0203 revision 35

| Claim | Class | Notes |
|---|---|---|
| Terms, *Unlimited mind controller* (new) | pointer | S-0601. |
| 4: the overload update is among the ordinary work a warp update suppresses; countdown and timing kept, no catch-up | satisfied | `overload_update_is_ordinary_work_of_the_warp_turn`. |
| 4, Backing; the SW23 part f extension | (c) | |

## S-0214 revision 39

| Claim | Class | Notes |
|---|---|---|
| Terms, *deploy readiness*: the yard exclusion reads control by a controlling object; permanently controlled and trigger-held vehicles are not refused | changed | as S-0601 claim 15. |

## S-0509 revision 6

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1203), approval date | (c) | |
| 12: the mind-control takeover and release of a building request an announcement | changed | as S-0601 claims 6 and 9. |
| 12, Backing | (c) | |

## S-0312 revision 30

| Claim | Class | Notes |
|---|---|---|
| 29: the first attacker-dependent raise, code 6, is inside S-0011's subset | changed | `triggers::judge` now judges code 6 (S-0011 claims 8 and 19). `tags::s0011_claim_19_code_6_death_raise`. |

## S-0314 revision 23

| Claim | Class | Notes |
|---|---|---|
| Terms (*overload pulse*, *unlimited controller*) | pointer | S-0601. |
| 9 and 10: a berserk object heals like any other; the order against the countdown is free for a unit with no overload pulse, S-0601 claim 16's for an unlimited controller | satisfied | `heal::turn` reads no berserk state (`claim_6_healing_hit_keeps_the_time`); the controller's order: `claim_16_heal_follows_the_pulse`. |

## S-0513 revisions 16 and 17

| Claim | Class | Notes |
|---|---|---|
| Terms, *Overload update* (new) | pointer | S-0601 claim 16. |
| 51: the overload update is suppressed ordinary work; update counts, no catch-up; elapsed-tick clocks run on | satisfied | `overload_update_is_ordinary_work_of_the_warp_turn`. |
| SW23 part f: the overload update in every row | satisfied | The same test (one kind; the warp rows are shared by every kind through `warp::turn`). |

## S-0011 revisions 22 and 23

| Claim | Class | Notes |
|---|---|---|
| Terms, *house number* | pointer | S-0009 (`HouseId`, the creation order from 0). |
| 8, events 6 and 44: every event-6 action completes before event 44 is judged; 6 true in any raise of its code; 44 true only with a causing object whose owner's house number equals the value; −1, player-position values and neighbours never match; no triggering house; not latchable | changed | `triggers::judge` (`ATTACKED`, `ATTACKED_BY_HOUSE`, `attacked_by_house`); `latchable` already left out 6 and 44; the triggering house is set only by entered-by. `mp_triggers::event_coverage` lists 6 and 44 as implemented. `tags::claim_5_attack_events_order_cause_and_judging`, `tags::claim_5_event_6_action_seen_by_event_44_and_the_test`. |
| 8, Backing and Confidence | (c) | |
| 19: the common group's code-6 raise judges event-6 triggers; the vehicle group's raise has code 7 | changed | as S-0312 claim 29. `tags::s0011_claim_19_code_6_death_raise`. |
| 19, Backing and Confidence | (c) | |
| Constants (event 44's value); I3 rows 6 and 44; the destruction-codes mapping row | changed | as claim 8. |
| Open questions (the hand-off of the attack-event judging) | (c) | |

## S-0701 revisions 16 and 17

| Claim | Class | Notes |
|---|---|---|
| Scope, deferred causes | (c) | |
| Terms, *Vehicle-like takeover*, *Mind-control takeover and release* (new) | pointer | S-0509 claim 17, S-0601 claims 6 and 9. |
| 27 table, *building captured, tech building lost* row; 38 scope wording | satisfied | The audio crate's `Effect::Captured` handling picks the entry from `announce` and the building's type, not the cause. |
| 39: covers every announcing ownership change: engineer captures, vehicle-like takeovers, mind-control takeover and release | satisfied (entry choice); changed (mind control, in the simulation) | `capture_announcements` (audio) for the entries; the sim now marks the mind-control changes as announcing (S-0601 claims 6 and 9 above). |
| 39, Backing and Confidence; key-table note; G5 text | (c) | |

## Glossary revision 44

| Entry | Class | Notes |
|---|---|---|
| *freeze, timed freeze, stand-still* (new) | pointer | S-0201; no code change. |
| *vehicle thief*: may also take a vehicle-like building as any infantry | pointer | S-0509 claim 17; no code change. |
| Questions for the project lead (*director*) | (c) | |

## S-0011 revision 24

| Claim | Class | Notes |
|---|---|---|
| Open questions: the hand-off of the attack-event judging removed | (c) | |

## S-0609 revision 4 (answers Q-1220 items 1 and 2)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1220), approval date | (c) | |
| Formulas preamble: under the Edge cases rules F1, F2 and F4 see non-negative inputs and every product, sum and elapsed count lies in 0 … 2³¹ − 1 | satisfied | `gattling::f1`, `f2` and `elapse` use saturating arithmetic on clamped settings. `saturation_at_the_32_bit_bound`. |
| Constants, the saturation bound 2³¹ − 1 | satisfied | `i32::MAX` through `saturating_*`. |
| Edge cases, *Stage count 0 or 1 from creation*: a layered reduction below 2 after thresholds were populated, no result established | not built (open) | Q-1151 interim: the thresholds are not used; the importer warns (`gattling_spin_settings`). |
| Edge cases, *Negative settings are used as 0*, with an importer warning for each; a wrapped integer likewise; the clamp applies to the imported value, so an omitting later layer keeps 0 | satisfied | `SpinType::from_fields` clamps the stage count, amounts and thresholds at 0; the importer warns for each negative value. Clamping the value in force rather than the imported value gives the same result, because an omitting layer keeps the held value and the held value is clamped when used. `spin_type_from_fields`, `gattling_spin_settings`. |
| Edge cases, *Saturation* (products, sums, the elapsed-tick count; a saturated value stays; the count stays until its reset) | satisfied | `saturation_at_the_32_bit_bound` (new test). |
| Import mapping: negative values used as 0 with a warning; the layered reduction is an open question | satisfied / not built (open) | as above. |
| Questions for the project lead | (c) | Items 1 and 2 restate the proposed rules; item 3 is Q-1151. |
