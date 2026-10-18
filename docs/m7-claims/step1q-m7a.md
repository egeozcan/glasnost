# M7a step 1q claim ledger (S-0607 revision 8, S-0502 revision 16)

This is part 17 of M7a step 1 (intake 12). It covers the approved revisions that reached main after the step 1p intake, from e0b0411 to b776294 (commits a7858fc and b776294): S-0607 revision 8, S-0502 revision 16, the index and the two answered requests. Every changed claim gets one class, as in [step1p-m7a.md](step1p-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered and no quirk switch is new: both revisions use the existing `quirks.jumpjet_no_cell_self_destruct` (its doc comment and `data/schema/ruleset.schema.json` now give the revision 16 wording; `every_spec_quirk_has_a_switch` passes). The new tests are in `crates/deterministic/sim/tests/claims_s0607_release.rs` (5 tests, one rewritten) and `claims_s0502_land.rs` (1 test).

| Spec | Revision | Answers |
|---|---|---|
| S-0607 magnetic lift | 8 (from 7) | Q-1342, Q-1343 (`answered-by`) |
| S-0502 jumpjet movement | 16 (from 15) | none (it states the same rules from the jumpjet side) |
| `spec/README.md` | | index rows revised: (c) |
| `spec/requests/` Q-1342, Q-1343 (answered) | | request files added with `status: answered`: (c) |

**Retired interims:**
- Q-1342 (whether a lifter-ended no-cell drop keeps the seize's point and commitment). The interim cleared both, as for a set-down unit, and that is now the spec's rule: claim 13 says a dropped and a set-down unit both lose their point and commitment. The over-centre fall then uses the direct reference. This was already the behaviour; the interim comment in `jump::stop_with` is now a citation and two tests are added.
- Q-1343 (with the switch off, whether claim 9's set-down descent follows the hover re-entry). The interim let the unit descend. Now it does not: in the air it keeps hovering, set down and seized, and claim 20 retries the search at each fall-back to rest instead of restoring it. `lift::set_down_released` no longer calls `jump::lift_descend` after that outcome. `lift::restore_at_rest` calls the new `jump::lift_retry_stop`.

**New request** (block Q-1344 to Q-1352; Q-1345 to Q-1352 are unused):
- Q-1344: S-0607 claim 20 excludes only "a set-down unit hovering under claim 9's switch-off rule" from restoration. S-0502's Edge cases cover every seized set-down unit that hovers. They differ for a still-moving set-down unit whose later landing finds no cell with the switch off: under claim 20 as written it would be restored in mid-air. Second, S-0502 says "grounded" (the flight state) while claim 9 splits a seized unit by height, which leaves open an *ascending* jumpjet still at height 0. Interim: the S-0502 wording, with the switch off only, as a derived test (`jump::hovering_uncommitted`: set down, no lifter, *hovering*, not moving), so no field is saved; in `jump::stop_with` the flight state decides.

## S-0607 revision 8

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, request list with Q-1342 and Q-1343) | (c) | |
| 9: no destination, a cell found: it flies to the found point and descends and lands there | satisfied | as built at step 1o (the sentence about claim 17's clear-cell descent is gone; the still-moving set-down path is unchanged). Test: `claim_9_no_destination_flies_to_the_found_point`. |
| 9: no destination, no cell, switch on: S-0502 claim 17's self-destruction, no credit | satisfied | Test: `claim_9_no_cell_keeps_the_ordinary_outcome` (switch on), `claim_9_no_cell_on_the_ground_stays_grounded` (switch on). |
| 9: no cell, switch off, in the air (1 lepton or more): it hovers over its current cell, no point, not moving, neither descends nor flies, stays set down and seized; claim 20 does not restore it while it hovers | changed | `lift::set_down_released` returns after the stop when the unit was moving, the stop left it not moving, the grip was not lifter-ended and it is in the air; before, `jump::lift_descend` landed it (Q-1343 interim). Test: `claim_9_no_cell_keeps_the_ordinary_outcome` (switch off: hovering, position and height unchanged through five refused orders and their fall-backs, still seized). Fails without the change. |
| 9: no cell, switch off, on the ground (0 or less, still moving): stays grounded, no point, not moving; claim 20 restores it as any set-down unit | changed | `jump::stop_with` keeps a *grounded* jumpjet grounded (S-0502 revision 16), and `lift::set_down_released` settles it. Test: `claim_9_no_cell_on_the_ground_stays_grounded`. |
| 9: backing (the switch-off rule `unmeasured`, design) | (c) | |
| 13: no cell, lifter-ended: both cases keep no point (the seize's is cleared) and lose the commitment; a set-down unit stays and claim 20 restores it | satisfied | as the Q-1342 interim. Tests: `claim_13_no_cell_lifter_set_down_stops`, `claim_13_no_cell_drop_over_the_centre_falls_by_the_direct_reference`. |
| 13: a dropped unit over its centre falls with no point, by S-0502 claim 9's direct reference; off its centre the detour gives its own cell's centre as its point | satisfied | `jump::lift_dropped`, `jump::height_step`. Tests: `claim_13_no_cell_drop_over_the_centre_falls_by_the_direct_reference` (a parked vehicle in its own cell; no fall step is held at its clearance), `claim_13_no_cell_drop_while_hovering_still_falls` (off its centre). |
| 13: losing its commitment does not hold back its fall: an update every tick until the fall ends, also when dropped while *hovering* | satisfied | our drop sets *crashing* (or the detour's *cruising*) at once, and both are moving now, so S-0502 claim 3's eligibility holds. Tests: `claim_13_no_cell_drop_over_the_centre_falls_by_the_direct_reference` (every fall step descends), `claim_13_no_cell_drop_while_hovering_still_falls` (state set to *hovering* just before the drop; centred and off-centre). |
| 13: "How high it is held": a dropped unit with no point uses the direct reference throughout | satisfied | as above. |
| 13: no cell, no destination: with the switch off, claim 9 says how it hovers or stays grounded | changed | as claim 9. |
| 20: a new third condition: not a set-down unit hovering under claim 9's switch-off rule | changed | `lift::restore_at_rest` tests, with the switch off, `jump::hovering_uncommitted` first (Q-1344 interim: any seized set-down unit hovering with no commitment and no lifter). With the switch on, claim 20's two original conditions decide. Test: `claim_9_no_cell_keeps_the_ordinary_outcome`. Fails without the change (restored in the air). |
| 20: the hovering set-down unit: at each fall-back to rest, S-0502 claim 17's search around its cell with no draw instead of restoration; found: re-ordered to that cell's centre through S-0502 claim 16 as claim 13's stop does, then flies, descends, lands and is restored later; not found: keeps hovering, not destroyed | changed | new `jump::lift_retry_stop` (the search with claim 17's settings, then `give_point`: a movement point only, the order-level destination stays none). Tests: `claim_20_hovering_set_down_retries_the_search` (a dry cell again: the retry finds it, the point is its centre, the hover slot is released while still in the air, it lands there and is restored), `claim_9_no_cell_keeps_the_ordinary_outcome` (failed retries change nothing). Fails without the change. |
| 20: a hoverer that never falls back to rest hovers indefinitely; the refused move order makes it fall back and so retries | satisfied | the tests drive the fall-backs with refused move orders (`order_move`). |
| 20: backing (the retry `unmeasured`, design) | (c) | |
| State table: the new row (set down, hovering after no cell with the switch off) | changed | as claim 20. |
| Edge cases: no cell for the stop when a lifter lets go (point and commitment cleared, the dropped unit with the direct reference) | satisfied | as claim 13. |
| Edge cases: no cell after a no-destination set-down, switch off (hover and retry in the air; on the ground stay and be restored) | changed | as claims 9 and 20. |

## S-0502 revision 16

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 3: neither physics nor outcome runs when not moving and not moving now, except during claim 17's no-cell magnetic-release safety fall; claim 17 guarantees every update of that fall | satisfied | our drop enters *crashing* (or the detour's *cruising*) in the same update, so the fall is moving now. Test: `claim_13_no_cell_drop_while_hovering_still_falls`. |
| 9: the safety fall uses the direct reference while it has no movement point (design) | satisfied | `jump::height_step` (no point: direct). Test: `claim_13_no_cell_drop_over_the_centre_falls_by_the_direct_reference`. |
| 17: magnetic-release safety departure: both a dropped and a set-down unit clear their point and commitment; the dropped unit's direct reference over its centre, the detour's own-centre point off it; an update every tick, even while hovering | satisfied | as S-0607 claim 13. Same tests. |
| Edge cases, `quirks.jumpjet_no_cell_self_destruct` off: point and commitment cleared; in the air it re-enters *hovering* over its current cell, neither descending nor flying on | satisfied | as built at step 1o. Test: `claim_17_no_cell_self_destruct`. |
| Edge cases, switch off: a grounded jumpjet stays grounded | changed | `jump::stop_with`: a *grounded* jumpjet clears its point, order and commitment and keeps its state and target height, with no hover slot; before, it entered *hovering* at height 0 with target height H. Test: `claim_17_no_cell_grounded_stays_grounded` (no take-off over ten updates, height 0, position unchanged; switch on: destroyed). Fails without the change. The boundary against an *ascending* jumpjet at height 0 is Q-1344. |
| Edge cases, switch off: a seized set-down unit follows S-0607 claims 9 and 20 (not restored while hovering, retries at each fall-back) | changed | as S-0607 claim 20. |
| Scenario J27 c (expanded expectations) | (c) | its engine-design checks are the tests named above. |

## Format and gate

- No saved state is added: the hovering set-down unit is recognised from existing state (`jump::hovering_uncommitted`). The save format stays 58 and the AI game format stays 23.
- The switch defaults on, and the gate fixtures never turn it off, so the new rules do not run in the duel.
- Seed sweep over seeds 1 to 16: the same counters as at steps 1o and 1p, to the last digit, with the same early ends and shortfalls. Seed 2 still plays all 12 000 ticks with every counter at least 3 except the sensor orders' known 2, so `M7A_SEED` stays 2. No golden changed, and `cargo test --release -p glasnost-sim-gate -- --include-ignored` passes without the update flag.
