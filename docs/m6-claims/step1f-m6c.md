# M6c step 1f claim ledger (spec revisions that landed on main after step 1e)

Part 6 of step 1. Each spec is compared with the revision the code had taken in; the old state is the spec file at the commit named in the table (`git diff <old> HEAD -- spec/<path>`: the commit that introduced the previous revision). Every changed claim is classified as *satisfied* (already true, with the test that shows it), *changed* (code changed, with the test), *pointer* (the claim only points at another spec's claim) or *not built* (with the reason); *(c)* is editorial (front matter, scenario text, rewording). Per-spec ledgers: [S-0502.md](S-0502.md), [../m5-claims/S-0417.md](../m5-claims/S-0417.md).

| Spec | Revision | Old state | Answers |
|---|---|---|---|
| S-0417 factory unloading | 19 (from 16; 17 and 18 landed between) | 379b6ea | Q-1070 |
| S-0502 movement of jumpjets | 9 (from 7; 8 landed between) | 7075dc8 | Q-1090 (with S-0207) |
| S-0207 ground vehicle movement | 32 (from 31) | 6af4fa6 | Q-1090 |
| S-0402 harvest cycle | 26 (from 25) | 4bf5787 | |
| S-0404 production queue and payment | 19 (from 18) | e6f099a | |
| glossary | 41 (from 40) | b0bc5b3 | |

Requests answered and their interim rules retired: Q-1070 (S-0417 revision 19, claim 15: the AI hold `scripted::jams_factory` is removed) and Q-1090 (S-0207 revision 32 claim 27 with S-0502 revision 8: the neither-pending-state interim rule is gone). One new request, with an interim rule: Q-1110 (what "immobilized by teleport aftereffects" names; whether a restoration inside an assignment precedes the assignment's own destination; which event clears the exit destination point of a teleporting vehicle). The other answered requests listed in the interim tables (Q-0172, Q-0193, Q-0663, Q-0970, Q-0263, Q-0329, Q-0333, Q-0275 to Q-0277, Q-0281, Q-0731) belong to specs whose revisions this step does not take in; their rows are unchanged and are checked in the last section.

## S-0417 revision 19 (answers Q-1070)

| Claim | Class | Notes |
|---|---|---|
| Header, Terms (*clear report* now "a unit's notice", claims 7, 10 and 15) | (c) | wording. |
| 5, restoration of a temporarily driving vehicle: a suspended predecessor, no driving movement report (no locomotor destination point and no heading point horizontally different from the actual position), no restoration lock, not seized; completing the exit track alone is insufficient while the point is retained; restoration alone enables neither pending state | changed | `econ::release::restorable` (used by `restore_locomotor` at the fall-back to rest and by the dock test), `drive::movement_report`. Before this step restoration was unconditional at the first fall-back. Tests `claim_5_restoration_needs_no_movement_report`, `claim_5_a_different_heading_point_is_a_report` (`claims_s0417_r19.rs`), `temporary_driving_restores_or_falls_back` (`claims_s0207_r32.rs`); the older tests re-reach the no-report state (`claim_21_previous_deck_landing`). Effect: a teleporting vehicle that left its factory keeps driving until its destination point is gone (Q-1110, question 3). |
| 5, Backing and Confidence text | (c) | |
| 7 (retitled "Clear report on vehicle cell arrival": a jumpjet's ascent uses claim 15 and has none of claim 7's arrival or building-cell tests) | satisfied | a jumpjet never reaches `release::arrival` (only walk, hover and drive arrivals call it); `claim_15_infantry_ends_contact_in_the_quarter_window` shows the report at a height over the factory without any arrival. |
| 10, "Release of jumpjet infantry" (claim 15 owns its ascent report; replaces the unresolved factory-contact paragraph) and the no-rally retention note (a jumpjet's ascent report is an independent contact-breaking event) | changed | as claim 15. `claim_15_infantry_place_is_free_after_the_report`; the no-rally walking and teleporting retention is unchanged (`claim_10_no_rally_no_destination`). |
| 15 (new): the ascent clear report of a released infantry unit or vehicle using jumpjet locomotion with the `JumpJet` or `BalloonHover` type flag, in contact with a vehicle factory or a building with a barracks door kind; the integer window trunc(*h*t / *q*) < *h* < *h*t after the update's physics, on every qualifying ascending update; both links end in that update; the design fallback for a skipped window; the vehicle factory closes at the first unloading processing that sees the ended link, the infantry place is free in the report tick; no rally test; the destination after the report: the saved guard spot if different from the order-level destination, otherwise the order-level destination is discarded (not claim 24's effective clear) and a forced scatter request without a threat point follows | changed | `econ::release::ascent_report` called from `jump::fly::ascending_outcome` (`jump::discard_order_destination`). Tests (`claims_s0417_r19.rs`): `claim_15_infantry_ends_contact_in_the_quarter_window`, `claim_15_balloon_hover_infantry_uses_a_quarter`, `claim_15_vehicles_half_and_quarter`, `claim_15_type_flags_gate_the_report`, `claim_15_other_buildings_give_no_report`, `claim_15_infantry_place_is_free_after_the_report`, `claim_15_no_rally_discards_the_destination`, `claim_15_skipped_window_reports_in_the_reaching_update`, `claim_15_report_survives_a_save`; the own AI: `builds_a_jumpjet_unit_and_its_factory_stays_free`. |
| Formulas, Import mapping rows (`JumpJet`, `BalloonHover`, `JumpjetHeight`, `GDIBarracks`, `NODBarracks`, `YuriBarracks`) | satisfied | all keys are imported (S-0502 and S-0404 mappings: `jumpjet`, `balloon_hover`, `jumpjet_height`, `door_first`, `door_second`, `door_third`). |
| U6 (restoration records), U10 c (ascent link observation), U14 (new jumpjet release scenario and the engine-only skipped-window variant) | (c) | measurement text; the engine-only variant is `claim_15_skipped_window_reports_in_the_reaching_update`. |
| Open question 9 (resolved for the ascent interaction) | (c) | |

## S-0502 revision 9 (from 7; revision 8 included; answers Q-1090 with S-0207 revision 32)

| Claim | Class | Notes |
|---|---|---|
| Pointers (Factory exit) | pointer | S-0417 claims 5, 10 and 15. |
| 7, factory release paragraph (the strict window with *q* = 2 only for a balloon-hover vehicle; no foundation, cell crossing, cruise, arrival or touchdown test; destination effects follow S-0417 claim 15; the safety departure for a skipped window; backing and confidence) | changed | as S-0417 claim 15 (J26 is `claim_15_skipped_window_reports_in_the_reaching_update`). Thresholds for the ascending physics (`ht/4`, `ht/2` for a balloon-hover vehicle, truncated toward zero) were already as built (`claims_s0502.rs`). |
| 24, note that the factory ascent report's discard is not an effective clear | changed | `jump::discard_order_destination` keeps the movement point and the commitment: `claim_15_infantry_ends_contact_in_the_quarter_window`, `claim_15_no_rally_discards_the_destination`. |
| 27 (placement: the normal initial fallback's empty-to-empty clear for a fresh ordinary vehicle is an effective locomotion stop that consumes refresh and enables no suppression, a jumpjet without a commitment gains no flight; type-specific fallback exceptions stay S-0215's) | changed | `objects::place_with` brackets the placement fall-back with `refresh::placement_begin` and `placement_end`; the fall-back rows that clear the destination (`process::fall_back_vehicle`) make the clear an assignment while refresh is enabled. A fresh vehicle has nothing to stop, so only the flags can be observed: `placement_leaves_no_pending_state`. |
| 27 (the later dock test: at each otherwise eligible destination assignment after unchanged-destination rejection, with the offered destination and the contact already held; first contact, arrival and entry do not trigger it; an unchanged offer without refresh does not reach it; a building destination is not the required cell destination; fallback enables refresh and suppression; consumption rules; separate suppression) | changed | `refresh::begin` / `finish` / `skip_point`; `claims_s0207_r32.rs`: `the_dock_fallback_skips_its_own_point_and_leaves_refresh`, `a_repeated_offer_reaches_the_test_again_through_refresh`, `contact_alone_does_not_trigger_the_test`, `every_predicate_is_needed`, `an_empty_clear_does_not_consume_the_suppression`, `refresh_makes_an_unchanged_jumpjet_assignment_effective`, `temporary_driving_restores_or_falls_back`. |
| J24 (revised), J26 (new), the Open question about the crash-end event | (c) | measurement text (J26: see claim 7). |

## S-0207 revision 32 (answers Q-1090)

| Claim | Class | Notes |
|---|---|---|
| Terms (*restoration lock*) | changed | the lock starts absent on a new driving locomotor and nothing in the engine sets it: `restorable` never finds it set (a seam; nothing to test beyond the other predicates). |
| 25 (new exception: claim 27's one-application suppression lets a nonempty order-level assignment be effective without installing the point, for driving and for jumpjets) | changed | `drive::take` (`refresh::skip_point`): `an_empty_clear_does_not_consume_the_suppression` (driving and jumpjet). |
| 27 (rewritten): placement interval; the dock evaluation event; the predicates (a ground-layer vehicle of any house, the mover included, disqualifies the cell; the type flag is independent of the current locomotor; failure adds no pending state); the table of locomotion states (temporary driving restorable, none or ordinary driving, restoration not permitted); separate consumption events; suppression for driving and jumpjets; empty clear; claim 20's landing-point rules only when a point reaches teleporting movement | changed | as S-0502 claim 27 above. The aftereffects predicate answers no and the order of a restoration against the assignment's own destination is an interim rule: Q-1110. |
| Import mapping rows (`Teleporter`, `DockUnload`, the new "no data key" row) | satisfied | both keys are imported (S-0214 and S-0402 mappings); the runtime flags are state (`econ.refresh`, `econ.point_suppression`). |
| D32 (rewritten conditional diagnostic), "Cross-spec changes to confirm" | (c) | measurement text and notes to the other specs. |

## S-0402 revision 26

| Claim | Class | Notes |
|---|---|---|
| 10, row 2 (the docking cell is offered at each processing; "Repeated destination offers": only an eligible assignment reaches the dock test, an unchanged offer without refresh is ignored, the fallback installs the order-level destination without its point and leaves refresh pending, no once-per-entry limit) | changed | `harvest::enter` offers again when refresh is pending (`crate::refresh::pending`); the rest is S-0207 claim 27. Test `claim_10_row_2_offers_reach_the_dock_test` (`claims_s0402.rs`). |
| 10, Backing and Confidence (the chrono-miner trace is `unmeasured`), H10 conditional trace | (c) | |
| Formula of the harvesting image's anchor (the table sine and cosine are renamed *S*t and *C*t) | (c) | wording only: the value is as before (`app::mapview::harvesting`, S-0105's table trigonometry). |

## S-0404 revision 19

| Claim | Class | Notes |
|---|---|---|
| 13, vehicle factory row (the ordinary arrival-report destination is claim 8's; for a jumpjet ascent S-0417 claim 15 governs contact and tether release and the subsequent destination; the factory does not wait for touchdown), infantry factory row (jumpjet ascent reports are claim 15's), the paragraph on the infantry initial destination (a no-rally jumpjet ends contact at its ascent), Backing and Confidence | pointer | S-0417 claim 15; the claim 13 release rules themselves are unchanged (`claim_13_way_out`, `claim_10_teleporting_infantry_goes_straight`, `claim_10_no_rally_no_destination`). |

## Glossary revisions

The glossary moved from revision 40 to 41 since step 1e (`git diff e5d9f3c e417078 -- spec/glossary.md`; the earlier revisions 38 to 40 restate claims of S-0513, S-0514, S-0516 and S-0701 that the code cites). The only change is the *tether link* example: a released jumpjet infantry unit or vehicle ends both links with an ascent clear report, without a cell arrival (S-0417 claim 15), while ordinary walking and teleporting infantry use the arrival reports of S-0417 claim 10. It restates S-0417 claim 15 (changed above) and needs no code of its own; the term's other uses (S-0402 claim 10, S-0417 claims 7, 10 and 11) are as built.

## Interim rows of the other answered requests

Checked against the answering specs: Q-0172 (S-0307 claims 15 to 25: the two edges left open are still open there), Q-0193 (S-0414 claim 17 gives only the defeat-count part; S-0404 Open question 3 keeps the killed-item production outcomes open), Q-0663 (S-0215 revision 39 claim 13, a seam: routes are not built), Q-0970 (answered at step 1 by S-0508 revision 7; the garrison row is a pointer), and the audio and client rows (Q-0263, Q-0329, Q-0333, Q-0275 to Q-0277, Q-0281, Q-0731), which wait for the client milestones. None of them is moved by the five specs of this step.
