# M7b step 1d claim ledger (drain, mutation and scatter revisions)

This is M7b step 1d. It takes in the revisions that step 1c listed as not taken in: S-0608 revisions 4 and 5 (the code implemented revision 3) with the companions S-0606 revision 8, S-0507 revision 9, S-0501 revision 18, S-0603 revision 9, S-0605 revision 9, S-0409 revision 16 and S-0203 revision 39; S-0611 revision 2; S-0208 revision 34. They answer Q-1440 to Q-1444, Q-1460, Q-1461 and part of Q-1502. Every changed claim gets one class, as in [step1b-m7b.md](step1b-m7b.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

**Format.** No hashed or saved state changed layout, so the save format stays 65 and the AI game format 26. The goldens were regenerated: `combat`, `duel`, `duel-m6c`, `duel-naval` and `skirmish` changed (building income now runs in the building's own turn before its behaviour, and `RevealOnFire` absent is yes). `duel-m7a`, `duel-m7b`, `duel-diagonal` and `scripted` are byte-identical. The `duel-m7b` sweep over seeds 1–16 did not move (equal to the values recorded after S-0612's merge): min / median are drain orders 3 / 5, links 3 / 4, credits drained 7 280 / 7 280, drained-building ticks 16 915 / 17 436, mutator launches 4 / 7, infantry mutated 12 / 21, dominator launches 4 / 5, objects taken over 11 / 15, all at least 3. The `duel-m7a` sweep over seeds 1–16 is unchanged: every seed line and every min, median and max equal a run of the base 226c5d2. `M7A_SEED` stays 1.

| Spec | Revision | Answers |
|---|---|---|
| S-0608 drain | 5 (from 3; 4 and 5 both new here) | Q-1440, Q-1441, Q-1442, Q-1443, Q-1444 |
| S-0606 tank bunkers | 8 (from 7) | Q-1444 (through S-0608 claim 18) |
| S-0507 transports | 9 (from 8) | Q-1443 (through S-0608 claim 17) |
| S-0501 aircraft | 18 (from 17) | Q-1442 (through S-0608 claim 16) |
| S-0603 grinders | 9 (from 8) | Q-1444 (cross-reference) |
| S-0605 absorbers | 9 (from 8) | Q-1444 (cross-reference) |
| S-0409 shroud and reveal | 16 (from 15) | Q-1442 (`RevealOnFire` absent) |
| S-0203 tick order | 39 (from 38) | Q-1440 (cross-reference) |
| S-0611 mutation | 2 (from 1) | Q-1460, Q-1461 |
| S-0208 infantry positions and scatter | 34 (from 33) | Q-1502, in part |

**Retired interims** (in code comments, `docs/development.md` and the ledgers):
- Q-1440 (turn placement). Replaced by S-0608 claim 15. The drainer's position check, then the alliance test unless it is being erased, both before the behaviour start; a building's income, then its payout, between the behaviour start and the due behaviour processing (changed).
- Q-1441 (the refusals' rank, the stranded building). Replaced by claim 3's *Precedence relevant to drain*. Part (a) moved next to the deactivated-vehicle answer and gives *rearming* while phasing in. Parts (b) and (c) moved after the temporal hold, before the spawner answers. Part (a) reads only the newest link (changed).
- Q-1442 (the attempt's other side effects). Replaced by claim 16. The suicide hit replaces the attempt. There is no last shot, reveal or uncloak, as built. The aircraft departure is new (changed).
- Q-1443 (temporary departure). Replaced by claim 17. Storage now ends the links, where the interim kept them (changed).
- Q-1444 (occupants). Replaced by claim 18. There is no occupant gate, as built. New pair tests (satisfied).
- Q-1460 (draws and numbering of creation attempts). Replaced by S-0611 claim 14. A refused attempt keeps its object number, where the interim gave it back (changed).
- Q-1461 (the crash flag). Replaced by S-0611 claims 7 and 8. A crashing victim keeps its position and its conversion animation still starts (changed).

**New requests:** none. Q-1507 to Q-1519 are unused. Q-1460's request file still says `status: open`, although S-0611 revision 2 claim 14 answers it; the lead should mark it answered.

## S-0608 revision 5 (from 3)

| Claim | Class | Notes |
|---|---|---|
| Front matter, Summary, Terms, backing and confidence, Open questions | (c) | |
| 1, clearing the object target also resets the burst to 0 | satisfied | `drain::attempt` already reset it. Test: `claim_1_a_drain_attempt_replaces_the_shot`. |
| 3, *Precedence relevant to drain* (Q-1441) | changed | `combat::check`. `own_drained_building` (part a) comes right after the deactivated-vehicle block: *rearming* while `teleport::phasing_in`, else *illegal*. `drain_weapon_refused` (parts b and c) comes after the temporal-hold *rearming*, before `held_spawner` and the empty-spawner *rearming*. Tests: `claim_3_precedence`, `claim_3_firing_check_refusals`. |
| 3, the stranded building (part a reads the newest link only) | changed | Tests: `quirk_relink_strands_previous_both_positions` (a stranded building is no longer refused, the newest one is). |
| 3, a power-sensitive weapon among the precedence rows | seam | No unit weapon has a power need in our data model, so that row cannot be reached. The other rows are tested. |
| 6, 11, wording (the turn placement points to claim 15) | (c) | |
| 15, turn boundaries (Q-1440) | changed | `tick::object_turn` order: the warp-only return, `drain::position_check`, `drain::alliance_test` unless `temporal::erased`, the behaviour start, then for a building `tech::turn` followed by `drain::building_turn`, then `process::tail`. `econ::events::turn` no longer calls `tech::turn`, and the later `drain::building_turn` call is gone. Tests: `claim_15_an_erased_drainer`, `claim_15_the_payout_takes_same_turn_income`, `claim_15_power_sees_the_position_check`, `claim_12_position_check`. The payout reads no operation state, so its order against the early operation-state refresh is not observable here. The warp-only stages skip both tests (S-0513 claim 51). |
| 16, no last shot, reveal or uncloak (Q-1442) | satisfied | Already as built in `fire::shot`, now cited. Test: `claim_16_no_shot_record_reveal_or_uncloak` (with and without a link). |
| 16, suicide replaces the attempt | satisfied | `fire::shot` tests suicide before drain. Test: `claim_16_suicide_replaces_the_attempt`. |
| 16, an aircraft's pending round and object-target departure | changed | See S-0501 below. |
| 17, temporary storage ends the link (Q-1443) | changed | `objects::take_inside` calls `drain::stored`, which ends every link with the stored object as drainer (the stranded one included) and then its link as a building. The warp sets the object off the map without `take_inside`, so it is not storage. Tests: `claim_17_boarding_ends_the_links`, `claim_17_failed_boarding_keeps_the_link`. The test helper `shift` (which moves an object through `take_inside`) now keeps the links across the move. |
| 18, occupants and entrants (Q-1444) | satisfied | No code. Tests: `a_drained_bunker_keeps_its_vehicle_firing` and `a_drained_grinder_still_recycles_for_its_owner` (new pairs); `a_drained_absorber_blacks_out_only_with_passengers` now lets the walker in while the absorber is drained. |
| Import mapping: `Suicide`, `DecloakToFire` (absent yes), `RevealOnFire` (absent yes) rows | satisfied / changed | `Suicide` and `DecloakToFire` were imported already (S-0312, S-0301). `RevealOnFire` absent changed to yes: see S-0409. |
| Scenarios D10 to D14 | (c) | Local tests above follow them, on synthetic data. |

## S-0606 revision 8 (from 7)

| Claim | Class | Notes |
|---|---|---|
| 9, the drain bullet (a bunkered vehicle keeps its own firing check and its bunker changes, S-0608 claim 18) | satisfied | Test: `a_drained_bunker_keeps_its_vehicle_firing` (`pairs_m7b.rs`). |
| Front matter | (c) | |

## S-0507 revision 9 (from 8)

| Claim | Class | Notes |
|---|---|---|
| 6, boarding ends the passenger's drain links (S-0608 claim 17) | changed | `objects::take_inside` → `drain::stored`. Test: `claim_17_boarding_ends_the_links` (`claims_s0608.rs`). |
| The reference from F7 to "S-0501 claim 40's landing test"; T12 text | (c) | |

## S-0501 revision 18 (from 17)

| Claim | Class | Notes |
|---|---|---|
| 12, *Drain continuation (departure)* | changed | `air`: `drain_cleared` and `drain_departure`. When a drain attempt clears an aircraft's object target and the aircraft stays in play, the remaining burst and the pass's scatter request are cancelled. The aircraft enters *done* with a 1-tick delay, keeping the link and the pending round. Neither the target nor the destination is restored, and no rearm starts. This applies in the first pass (after `full_burst`), refiring and strafing. Test: `claim_12_drain_departure` (`claims_s0501.rs`). |
| 15, an admitted initial pass with a draining weapon makes the round pending | changed | `air`: CanFire marks the round pending before the departure. The round is spent at the ordinary boundary (ammunition 3 → 2 in `claim_12_drain_departure`). |
| 16, the scatter exception | changed | `drain_departure` returns before `fired` makes the pass's scatter request (coded; `claim_12_drain_departure` checks the departure but has no unit in the target's cell to observe a scatter request). |
| T1 text, Edge cases (*Drain clears an aircraft's object target*), the `DrainWeapon` import row, AC28 | (c) | AC28's original-side check is the oracle's; the engine-only part is `claim_12_drain_departure`. |

## S-0603 revision 9 and S-0605 revision 9 (from 8)

| Claim | Class | Notes |
|---|---|---|
| Out-of-scope bullets pointing to S-0608 claim 18 | (c) | Behaviour: `a_drained_grinder_still_recycles_for_its_owner`, `a_drained_absorber_blacks_out_only_with_passengers`. |

## S-0409 revision 16 (from 15)

| Claim | Class | Notes |
|---|---|---|
| Import mapping, `RevealOnFire` absent: yes | changed | `econ::shroud::fired` reads `.unwrap_or(true)`, and the `reveal_on_fire` doc in `glasnost-rules` says so. `data/schema/ruleset.schema.json` regenerated. Test: `claim_16_reveal_on_fire_absent_is_yes` (`claims_s0409.rs`). |

## S-0203 revision 39 (from 38)

| Claim | Class | Notes |
|---|---|---|
| The drain ordering text referring to S-0608 claim 15 | (c) | Built with S-0608 claim 15 above. |

## S-0611 revision 2 (from 1)

| Claim | Class | Notes |
|---|---|---|
| 7, a crashing victim (Q-1461) | changed | `mutation::convert`. A victim crashes only with height above the surface (`start_crash`). A crashed victim keeps its position and records and does not leave play here. Test: `claim_7_crash_victim_still_converts` (a crashable infantry on the ground converts like any other; in the air it crashes and the maker still starts and later makes a unit). |
| 8, the conversion animation always starts and marks its position at once for a crashing victim | changed | `mutation::mark_now`. Same test. |
| 14, draws and numbering of creation attempts (Q-1460) | changed | `mutation::discard` no longer gives `w.next_id` back. Each attempt takes the raw creation draw (`objects::create_converted`), and the position-choice draw only where S-0208 claim 3 takes it. Test: `claim_11_placement_refused` (one raw draw per refused attempt with the vehicle blocking the record, and one object number used per refusal). |
| Front matter, backing, scenarios | (c) | |

## S-0208 revision 34 (from 33)

| Claim | Class | Notes |
|---|---|---|
| 24, a scatter destination is not an escape path (Q-1502, in part) | satisfied | `scatter::request` only sets the destination; the walk goes through the path search with S-0212 claim 6's gate. Test: `claim_24_scatter_is_no_escape` (`claims_s0208.rs`). A walker on a water cell (no reach region for its zone) gets a destination next to it with and without the forced and move-anywhere options, but never leaves its cell; a move order fails too. |
| 12, 13 wording; scenarios | (c) | |

## Not taken in

- **S-0405 revision 21** (aba2af6) and **S-0107 revision 18** (98e3b55) reached main after this step's base 226c5d2 (`git log 226c5d2..main -- spec/ measurements/`), and this branch does not merge main. S-0405 revision 21 answers Q-1505 in claim 13 only: a vehicle on its own house's repair depot is passed over at the last control centre's loss only if it is tethered to that depot and the depot stands in its cell (being serviced is not enough); the arrival exception needs no tether, and a passed-over vehicle is disabled at its first arrival in a cell without a building; the import table gains the `UnitRepair` and `Helipad` row and scenario W8 part (m). That replaces step 1c's Q-1505 interim (pad depot yes, ordinary depot no). It does not touch S-0405 claims 5, 7, 8 and 9, which this step's drain tests lean on. S-0107 revision 18 changes claim 17's row 7 only: for a successful replacement a crash changes only the victim's removal, the conversion animation still starts, and the standing position is marked taken again at crash entry (S-0611 claim 7). That is what this step built for S-0611 revision 2 (`mutation::convert`, `claim_7_crash_victim_still_converts`), so it needs no code; its ledger row waits for the intake. The next intake takes both in.
