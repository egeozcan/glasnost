# M7b step 1j claim ledger, simulation group (intake 14S)

This is the simulation group of step 1j (intake 14S). It covers the approved revisions that reached main by 57f6e9b for these specs: S-0314 revision 31, S-0402 revision 34, S-0501 revision 19, S-0504 revision 20, S-0611 revision 4, S-0613 revision 12, S-0615 revision 4, S-0516 revision 12, S-0513 revision 21 and S-0514 revision 11. Every changed claim gets one class, as in [step1q-m7a.md](step1q-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

Nothing new is saved or hashed. The save format stays 67 and the AI game format stays 29. The release gate (`cargo test --release -p glasnost-sim-gate -- --include-ignored`) passes with no golden update. No quirk switch is new. Nothing visible changes, so this group adds no pictures.

| Spec | Revision | Answers |
|---|---|---|
| S-0314 healing and repair | 31 (from 30) | none |
| S-0402 harvest cycle | 34 (from 33) | Q-1530 (with S-0611) |
| S-0501 aircraft | 19 (from 18) | none |
| S-0504 naval units | 20 (from 19) | none |
| S-0611 mutation | 4 (from 3) | Q-1530 (`answered-by`) |
| S-0613 gas clouds | 12 (from 11) | none |
| S-0615 special-hit effects | 4 (from 3) | none |
| S-0516 cloak and stealth | 12 (from 11) | none |
| S-0513 superweapon effects | 21 (from 20) | none |
| S-0514 chrono weapons | 11 (from 10) | none |

**The slot seam.** S-0314 revision 31 makes the depot read two of its own slot animations: *production* (claim 6) and *special-3* (claim 22). S-0108 claim 20 owns starting and stopping those slots. That slot model is not built in the simulation, and intake 14B builds it. This group builds only the depot side, behind `glasnost_sim::depot::slot_running(w: &World, k: ObjectId, slot: DepotSlot) -> bool` with `DepotSlot::{Production, Special3}` (`crates/deterministic/sim/src/depot.rs`). For now it returns false.

**Retired interim:**
- Q-1530 (whether the object that closes a spawn-and-slave cycle gets a controller or a group). The interim gave it neither. S-0611 revision 4 (F5, claim 14) and S-0402 revision 34 (claim 19) now say the same. The interim comment in `objects` is now a citation. The importer warning reads "created with neither a spawn controller nor a slave group".

**New request** (block Q-1610 to Q-1619; Q-1611 to Q-1619 are unused):
- Q-1610: S-0314 claim 22 says a depot that loses its contact reacts "at its next repair processing". A working depot's processings are K<sub>u</sub> apart, and a cannot-pay retry is *D* apart. S-0108 claim 20 starts the loss slots "on the same or the next tick" in the working and settling phases. These two disagree on when a working depot notices the loss. Interim: it notices at its next scheduled processing, and nothing is brought forward. In a synthetic test the reaction came 27 ticks after the loss. The interim is commented in `depot::process_repair`.

## S-0314 revision 31

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, date) | (c) | |
| 6: settling with contact: while the depot's *production* slot runs, no repair step, recheck after 1 tick | changed | `depot::process_repair` checks `slot_running(.., DepotSlot::Production)` first in the settling branch with contact, before the step. The hold cannot be reached while the seam reports false, so no test exercises it yet; 14B's slot tests will. `slot_seam_reports_no_running_slot` shows only that the seam reports false. |
| 6: the uncharacterised non-human-owner branch (a paid settling step sent back to waiting) | (c) | the spec's own open question. Nothing is built and no request is filed. |
| 6: backing (H23 `measurable`; the branch `unmeasured`), confidence | (c) | |
| 22: lost contact in waiting: guard at that processing, no animation wait | changed | before, every phase went to guard. Test: `claim_22_contact_loss_by_phase` (phase 0). |
| 22: lost contact in working: settling, recheck after 1 tick | changed | the progress moves to settling and the processing returns 1. The serviced vehicle slot is kept for the recheck. Test: `claim_22_contact_loss_by_phase` (working: settling, then guard one tick later). Fails without the change. When the loss is noticed is Q-1610. |
| 22: lost contact in settling: stay in repair, checking every tick, until *special-3* no longer runs, then guard | changed | while `slot_running(.., DepotSlot::Special3)` holds, the processing returns 1. Otherwise serviced is cleared and the depot guards. Test: `claim_22_contact_loss_by_phase` (settling, with the seam false: guard). The wait itself cannot be reached until 14B fills the seam, so no test exercises it yet. |
| 22: the slot starts themselves (*special-3* and *active-1* on a loss; *idle* after *special-3*) | seam | S-0108 claim 20, intake 14B, behind `depot::slot_running`. |
| 22: backing (S-0108 claim 20, B18 g), confidence | (c) | |
| State table: settling with contact and the *production* slot; settling scheduled; waiting, working and settling with no contact | changed | as claims 6 and 22. |
| H18 (separate waiting, settling and working cases), H23 (new) | (c) | their engine-design checks are the tests above. |
| Open questions: the non-human-owner branch | (c) | |

## S-0402 revision 34

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, request list with Q-1530) | (c) | |
| 19: an object that closes a cycle of spawn and slave types gets neither a spawn controller nor a slave group | satisfied | the Q-1530 interim, now a citation. Tests: `s0611_f5_mixed_cycle_closer_keeps_no_controller_or_group`, `s0611_f5_shared_descendant_is_not_a_cycle` (`claims_creation_set.rs`). |

## S-0501 revision 19

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 20: a drop attempt makes no shot animation (S-0301 claims 20 to 23; was "muzzle animation") | satisfied | `combat::fire::shot` hands an aircraft's drop to `paradrop::attempt` and returns before `combat::shotanim::create`, the only maker of a shot animation. Test: `s0301_claims_10_15_a_drop_spends_one_round_at_once_and_is_not_a_void_shot` (`claims_s0522.rs`) now also asserts no `Effect::ShotAnim`. |

## S-0504 revision 20

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 8: a naval yard takes S-0108 claim 20's slot rows | seam | intake 14B. The depot code serves yards too, so the S-0314 changes above apply to them through `depot::slot_running`. |

## S-0611 revision 4

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, Q-1530 `answered-by`) | (c) | |
| 1, 6, 10, 11: backing | (c) | |
| 5: height is no absolute ban on conversion | satisfied | `combat::damage` sets an airborne mutating hit's amount to 0 by S-0302 claim 17, but one that does not ignore defences can still kill through the minimum damage. Row 7 of `combat::death` then converts. Test: `claim_5_height_is_no_conversion_ban` (new). |
| 14: the branch wording | (c) | |
| 14 and MU12: the controller-only variants (a parasite primary weapon, an air-strike designator) take no draw and no number | satisfied | Test: `claim_14_parasite_and_designator_take_nothing` (new). |
| F5: the cycle rule (no controller, no group) | satisfied | the Q-1530 interim. Tests as S-0402 claim 19. |
| MU13, MU14 | (c) | |
| State table wording | (c) | |
| Import mapping: the cycle note | satisfied | the importer warning text now names both. |

## S-0613 revision 12

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 15, 16: cross-references to S-0616 claims 23, 25 and 26 | satisfied | built with S-0616 in `mapview::particles`. |

## S-0615 revision 4

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision), Terms (the beam) | (c) | |
| 15: arc beams with fade parameters S = 1, E = 0.5 (S-0518 claim 19, F6), drawn ordinarily in the house colours | satisfied | `mapview::beams` already follows S-0518. Tests: `disc_laser_beam_parameters` (new, `crates/app/src/mapview/beams_tests.rs`), `live_tests::disc_laser_beam_fades_in_its_colours`. Citations in `disc` and `mapview::beams` updated. |
| 16: the final beam, S = 1, E = 0 | satisfied | the same tests. |
| 18: the spread draws are unsynchronised | satisfied | cosmetic draws in `mapview::beams`. |
| F5, Constants, DL1, DL5 | (c) | |
| Open question removed | (c) | |

## S-0516 revision 12

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 28: an object fading at stage 0 is drawn normally | satisfied | `cloak::view::look`. Test: rows added to `claim_28_how_v_sees_it`. |
| 28 with S-0513 claim 29b: protection on a translucent or rippling drawing | satisfied | `mapview::live` (the F8 blend keeps the source pixels' protection brightness and takes no mask), tested in `live_tests`. |
| Edge cases: an empty disguise list | satisfied | `cloak::disguise`. |
| Edge cases: no infantry disguise type | satisfied | `side_default` returns none; nothing is invented. |
| 37: drawing a disguise | seam | not drawn yet (`mapview::live`). |
| Backing of the large stage count; scenarios CS1 to CS22 | (c) | |

## S-0513 revision 21

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 29: protection combined with the air-strike mark's brightness; the protection draw before the mark draw (S-0519 F7, claim 26) | seam | intake 14B. The protection mask is built in `mapview::protection`. |
| 40: phasing in is drawn 50 % translucent (S-0514 claim 27) | satisfied | `mapview::live` (`erased_code`). Test: `live_tests::warp_phasing_in_is_drawn_half_translucent` (new: from departure to the end of the arrival hold, and the leaving stage as being erased, code 2; a pending warp, none). |
| Import mapping: `IronCurtainColor`, `ForceShieldColor` | (c) | built with Q-1527. |

## S-0514 revision 11

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| Import mapping: `RadColor` (a byte triple; (0, 0, 0) at a fresh start) | satisfied | `crates/convert/src/sim_combat.rs`, tested in `sim_radiation_tests.rs`. Citation added in `mapview::beams`. |
