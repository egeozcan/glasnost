# M6c step 1g claim ledger (spec revisions that landed on main after step 1f)

This is part 7 of step 1. Each spec is compared with the revision the code had taken in. The old state is the spec file at the commit named in the table (`git diff <old> bd86e84 -- spec/<path>`; that commit introduced the previous revision). Every changed claim gets one class:
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason;
- *(c)*: editorial (front matter, scenario text, rewording).

Per-spec ledgers: [S-0502.md](S-0502.md), [../m5-claims/S-0417.md](../m5-claims/S-0417.md), [../m5-claims/S-0402.md](../m5-claims/S-0402.md), and [teleport.md](teleport.md) for S-0207 claim 20. S-0207 has no ledger of its own.

| Spec | Revision | Old state | Answers |
|---|---|---|---|
| S-0207 ground vehicle movement | 34 (from 32; 33 included, 3d4f76c and fc18fe5) | 410f5b9 | Q-1110 questions 1 and 2 |
| S-0417 factory unloading | 20 (from 19, bb22ecb) | e417078 | Q-1110 question 3 |
| S-0502 movement of jumpjets | 10 (from 9, 76a7dd3) | a3a21fc | (with S-0207) |
| S-0402 harvest cycle | 27 (from 26, bd86e84) | eac2245 | (with S-0207 and S-0417) |

Requests:
- **Q-1110**, answered and its interim rules retired:
  - The aftereffects seam that always answered no is now the warp-pending immobilization.
  - The "restored first" rule is now the spec's own rule. The code also makes sure the offer always reaches the restored locomotor.
  - The retained exit point is now S-0417 claim 16, already as built.
- **Q-1130**, new, with an interim rule:
  - Does pending point suppression skip every teleporting landing point, or only a restored predecessor's?
  - How is it ordered against claim 20 (a)'s phasing-in refusal?
  - Which *seeking* processings of a teleporting chrono miner drop its destination?

No spec commit landed on main after bd86e84 while this step was worked (`git log HEAD..main -- spec/` is empty).

## S-0207 revision 34 (33 included; answers Q-1110 questions 1 and 2)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1110) | (c) | |
| Terms, *Warp-pending immobilization* (new): the external-teleport commitment from an accepted superweapon warp, before its leaving stage, until the object leaves its old position. It includes the leaving wait, not the arrival hold. It is distinct from phasing in, the held countdown and the restoration lock, and an own-locomotor jump does not create it | changed | `refresh::immobilized` is now `superweapons::warp_pending_or_leaving` (warp stages `Pending`, `Leaving` and `Expired`; S-0513 owns the stages). Before this step it was a seam that answered no. Tests in `claims_s0207_r34.rs`: `warp_pending_immobilization_fails_the_dock_test` and `phasing_in_is_not_the_immobilization`. |
| 20 (b): the case that leaves *L* unchanged is the warp-pending immobilization (it was "held in place by a superweapon teleport's after-effect") | changed | `teleport::take_destination` treats it as a search that gives no cell. Before this step the after-effect was a seam: [teleport.md](teleport.md) said "superweapons, S-06xx". Test: `claim_20_b_warp_pending_immobilization_keeps_the_landing_point`. If no *L* was fixed, the destination is removed. An earlier *L* stays. |
| 25, *Factory-exit lifetime* (new paragraph): S-0417 claim 16 owns the lifecycle. The point is retained after the no-order exit completion. The report, the contact and tether loss and the harvest queue are not stops. There is no timeout. A later rally, scatter or ore destination replaces the point. A qualifying completion or an effective stop clears it. Backing and confidence | satisfied | as built, the exit point lives in `DriveState::goal` until a later assignment replaces it or claim 25 or 26 clears it. Tests in `claims_s0417_r20.rs`: `claim_16_a_chrono_miner_drives_its_first_leg_to_the_ore`, `claim_16_a_later_move_clears_the_point_and_the_fallback_restores` and `claim_16_no_field_no_timer`. |
| 27, the predicate "has no warp-pending immobilization"; *Which immobilization* (read even when the locomotor is not teleporting; not phasing in, the held countdown or a cooldown; the ordinary warp installs teleporting movement, so the combination has no ordinary fixture) | changed | as Terms. The reachability remark is S-0513's, and the warp code is unchanged. Our warp member keeps its locomotor, so the test sets the stage directly on a driving vehicle with the flag. |
| 27, table row 1: eligibility uses the driving points already present, not the offer. The restored predecessor, not driving movement, receives this assignment's due point | changed | the order was already as built: `refresh::begin` restores before `teleport::take_destination`, and `restorable(…, true)` reads the stored point. New: `refresh::begin` forgets the destination the suspended state last took, so the restored locomotor always takes the offer (before, an offer equal to that stale destination was ignored). Tests: `the_restored_predecessor_takes_the_offer_and_jumps`, `eligibility_reads_the_stored_driving_points` and `the_restored_predecessor_takes_an_offer_equal_to_its_suspended_one`. |
| 27, *The restoring assignment's destination* (new). Restoration fixes *L* under claim 20 in the same assignment, and the jump comes at the next teleporting movement update. A pending earlier teleport can be replaced. Suppression already pending still skips the application. A retained driving point prevents restoration. The fall-back route is separate. The factory-built chrono miner drives its first leg | changed | `teleport::take_destination` now calls `refresh::skip_point` after claim 20 (a)'s phasing-in and held refusal. Whether that holds for every teleporting point, and in that order, is Q-1130 (interim, below). Tests: `the_restored_predecessor_takes_the_offer_and_jumps` (one step lands it), `a_pending_earlier_teleport_is_replaced`, `pending_suppression_still_skips_the_restored_point` and `a_teleporting_point_and_pending_suppression_interim`. The lifecycle sentences point at S-0417 claim 16. |
| 27, Backing and Confidence | (c) | |
| Import mapping rows: warp-pending immobilization and same-assignment routing ("no additional key"), `Teleporter` and `DockUnload` | satisfied | both keys are imported (`raw.teleporter`, `econ.dock_unload`). The two runtime rows import nothing. |
| D32 (the teleporting predecessor's same-assignment landing), D33 (new), *Cross-spec changes to confirm* | (c) | measurement text and notes to the other specs. |

## S-0417 revision 20 (answers Q-1110 question 3)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests`), the companion-passage paragraph (Q-1110 question 3 is answered by claim 16) | (c) | |
| 5: claim 16 gives the later replacement and clearing events, not an automatic restoration at the apron. Assignments completed while driving use driving movement. A qualified docking assignment that itself restores supplies its point to the restored locomotion | changed | as S-0207 claim 27 row 1 above (`the_restored_predecessor_takes_the_offer_and_jumps`). The driving half is unchanged (`claim_21_restoration_retains_the_teleporting_state`, `claims_s0207_teleport.rs`). |
| 16 (new), lifetime of temporary driving after a factory exit: the table of events, the active-locomotion seeking test, *No follow-up goal* | satisfied | already as built. Tests in `claims_s0417_r20.rs`, with a synthetic factory-built chrono miner whose own locomotor teleports: `claim_16_a_chrono_miner_drives_its_first_leg_to_the_ore` (the report and the harvest queue restore nothing; the ore destination replaces the exit point in the driving locomotor and is driven, with no jump of more than two cells; arrival and loading leave it driving), `claim_16_a_later_move_clears_the_point_and_the_fallback_restores` (a move replaces the point; its completion and the following fall-back restore the locomotor; the next move teleports), `claim_16_no_field_no_timer` (300 ticks, no field, no order: the point is retained). The qualified docking row is S-0207 claim 27. Our engine drops no seeking destination (Q-1130), so the temporarily driving miner keeps its destination, as the claim requires. |
| Import mapping rows: `Harvester`, `Weeder` (vehicle), `Teleporter` (vehicle), "no data key" | satisfied | all keys are imported (`harvester`, `weeder` and `teleporter` in `crates/convert/src/sim.rs`). Weeder behaviour is out of scope by S-0402's Edge cases. |
| U6 (assignments completed entirely in driving; the qualified docking assignment), U15 (new) | (c) | measurement text. |

## S-0502 revision 10 (with S-0207 revision 34)

| Claim | Class | Notes |
|---|---|---|
| Terms, *Warp-pending immobilization* (new) | pointer | S-0207 Terms (changed above). |
| 27: the predicate is the warp-pending immobilization, read whatever the current locomotor, not phasing in, held or a cooldown. Restoration enables neither pending state, and the same assignment's due point goes to the restored predecessor, subject to already-pending suppression and ordinary refusals. Eligibility reads the temporary locomotor's preceding state | changed | the shared code of S-0207 claim 27 (`refresh`, `teleport::take_destination`). The tests are in `claims_s0207_r34.rs` (see S-0207 above). In our engine a jumpjet never has a temporary locomotor, so for a jumpjet only the predicate applies. |
| 27, Backing and Confidence; J24 (the immobilization recorded apart from phasing in and held; the routing falsifiers) | (c) | |

## S-0402 revision 27

| Claim | Class | Notes |
|---|---|---|
| Terms, *Chrono miner* (its travel is teleporting only while its active locomotion is; a factory-built miner can keep temporary driving for its first field leg) | satisfied | travel is whatever locomotor is active: `claim_16_a_chrono_miner_drives_its_first_leg_to_the_ore`. |
| 9, the queueing-cell move ("its active locomotion determines whether that travel teleports or drives") | satisfied | `drive_to` assigns a destination that the active locomotor takes (`claim_17_chrono_too_far`, with the driving CHRONO). |
| 10: on a qualified successful restoration, the restored movement receives the same docking-cell offer. Eligibility reads the old driving points. The jump comes at the next teleporting update. No preliminary driving leg or second offer, and no pending state | changed | as S-0207 claim 27: `the_restored_predecessor_takes_the_offer_and_jumps`, `eligibility_reads_the_stored_driving_points`, `pending_suppression_still_skips_the_restored_point`. |
| 17, first bullet (the queueing cell is reached "by its active locomotion") | satisfied | as claim 9. |
| 17, seeking bullet (the trailing "the two go together in normal data" is removed; the rule tests the current locomotor) | not built | the drop of a seeking destination was never built (the ledger gave the missing teleport locomotor as the reason, which no longer holds). Claim 3's phase table has no "start" event inside *seeking*, and the drop target (the order-level destination, the landing point or both) is unstated: request Q-1130 question 3. Interim: nothing is dropped. The module text of `econ::harvest` now says so. |
| 17, *Temporary driving after production* (new paragraph) | satisfied | `claim_16_a_chrono_miner_drives_its_first_leg_to_the_ore` (`claims_s0417_r20.rs`). The docking route is claim 10's row above. A refinery's free unit (claim 18) is untouched. |
| 17, Backing and Confidence; H8 (active locomotion recorded; U15 for the factory-built miner); H10 (the routing falsifiers); the anchor formula's *S*t and *C*t | (c) | wording only. The anchor value is unchanged (`app::mapview::harvesting`). |

## Request Q-1130 and its interim rule

[spec/requests/Q-1130-teleporting-point-suppression-and-chrono-seeking-drop.md](../../spec/requests/Q-1130-teleporting-point-suppression-and-chrono-seeking-drop.md). The interim rules are in `docs/development.md`, section *Simulation (M6c: destination refresh and the dock fallback)*:
- Pending one-application suppression skips any teleporting vehicle's nonempty landing-point fixing and is consumed by it. The order-level destination stays.
- The suppression test comes after claim 20 (a)'s phasing-in and held refusal. A refused destination is no application, so the suppression stays pending (`a_teleporting_point_and_pending_suppression_interim`).
- No seeking destination of a chrono miner is dropped.

In ordinary play, suppression pending before a teleporting assignment is not reachable. The fallback that sets it is consumed by the driving or jumpjet point of the same assignment.

## Save format and goldens

No state field changed. The save format stays 45 and the AI game format stays 13. `cargo test --release -p glasnost-sim-gate -- --include-ignored` passes against the goldens of the base (669f002). The behaviour changes touch only warp members, pending suppression and restoring dock assignments, and none of these occurs in those gate games. The `duel-m6c` scenario merged on main after the base was not run here.
