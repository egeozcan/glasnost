# M7a step 1r claim ledger (S-0607 revision 9, S-0502 revision 17)

This is part 18 of M7a step 1 (intake 13). It covers the approved revisions that reached main after the step 1q intake, from 8343f11 to a0ea7eb (commits de319c2 and a0ea7eb): S-0607 revision 9, S-0502 revision 17, the index and the answered request Q-1344. Every changed claim gets one class, as in [step1q-m7a.md](step1q-m7a.md):
- *changed*: code changed, with the test;
- *satisfied*: already true, with the test that shows it (a new test where there was none);
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered and no quirk switch is new: both revisions use the existing `quirks.jumpjet_no_cell_self_destruct` (its doc comment and `data/schema/ruleset.schema.json` now give the revision 17 wording; `every_spec_quirk_has_a_switch` passes). The new tests are in `crates/deterministic/sim/tests/claims_s0607_release.rs` (2 tests) and `claims_s0502_land.rs` (1 test); `crates/convert/src/sim_econ_tests.rs` `jumpjet_type_keys` is reworded and extended.

| Spec | Revision | Answers |
|---|---|---|
| S-0607 magnetic lift | 9 (from 8) | Q-1344 (`answered-by`) |
| S-0502 jumpjet movement | 17 (from 16) | none (it states the same rule from the jumpjet side) |
| `spec/README.md` | | index rows revised: (c) |
| `spec/requests/` Q-1344 | | `status: answered`: (c) |

**Retired interims:**
- Q-1344, question 1 (which seized set-down units take claim 20's hovering rule). The interim applied it to every seized set-down unit hovering with no commitment, with the switch off; revision 9 says the same: every route. Kept, now cited to the spec; `jump::hovering_uncommitted` also requires no movement point, as claim 20 now names it (no observable change: the switch-off outcome always clears the point). New test for the route the request named: `claim_20_hovering_exclusion_for_a_committed_set_down` (fails with the exclusion disabled).
- Q-1344, question 3 (an *ascending* jumpjet at height 0). The interim let the flight state decide, so it hovered at height 0. Revision 9 and revision 17: the height above the surface decides; 0 or less stays put and becomes *grounded*. `jump::stop_with` changed.

**New requests:** none. Block Q-1345 to Q-1352 is unused.

## S-0607 revision 9

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision, Q-1344 in the request list) | (c) | |
| 9: no cell, switch off, on the ground (height 0 or less): it stays where it is and its flight state becomes *grounded*, even if it was *ascending* at height 0; no point, not moving; claim 20 restores it as any grounded set-down unit | changed | `jump::stop_with` now makes it *grounded* by height (before, an *ascending* unit was put into *hovering* at height 0, and `lift::set_down_released` then settled it: the end state was the same for a seized unit). Tests: `claim_9_no_cell_ascending_at_height_zero_becomes_grounded` (new: the update after the take-off, height 0, *ascending*), `claim_9_no_cell_on_the_ground_stays_grounded`. |
| 20: the third condition: not a seized set-down unit hovering with no movement point and no movement commitment after S-0502 claim 17's no-cell outcome with the switch off | changed | `jump::hovering_uncommitted` adds "no movement point" (`JumpState.dest` none). Test: `claim_20_hovering_exclusion_for_a_committed_set_down`. |
| 20: the hovering set-down unit, whatever route brought it there, including a set-down unit that kept its commitment, flew to its point and met a no-cell stop | satisfied | as the Q-1344 interim (`lift::restore_at_rest`). Test: `claim_20_hovering_exclusion_for_a_committed_set_down` (new: set down at height 0 with its commitment, a stop on its way with no cell, three refused orders with failed retries and no restoration, then a neighbouring cell made land again, the retry finds it, the unit lands there and is restored at a later fall-back). The seized landing exceptions skip the landing check, so the test reaches the outcome through a stop (claim 17), which claim 20 names ("or another stop"). |
| 20: S-0502's switch-off height test applies (1 lepton or more hovers; 0 or less stays put and becomes *grounded*, even from *ascending*) | changed | as claim 9. Tests: `claim_9_no_cell_ascending_at_height_zero_becomes_grounded`, S-0502's `claim_17_no_cell_switch_off_height_decides`. |
| 20: the summary ("subject to the hovering exclusion above") | (c) | |
| 20: backing and confidence (the exclusion, the retry for every route and the height boundary `unmeasured`, design; scenario L14) | (c) | |
| State table: the restoration row (outside the exclusion) and the hovering row (any route) | changed | as claim 20. |
| Edge cases: no cell for a seized set-down unit, switch off (any route; the height boundary; claim 13's lifter-ended safety departure unchanged) | changed | as claims 9 and 20; the lifter-ended departure is untouched (`claim_13_no_cell_lifter_set_down_stops`). |
| Import mapping: the switch-off rules add no imported field or default | (c) | |
| Scenario L14 (design checks, our engine only) | (c) | its checks are the tests named above. |

## S-0502 revision 17

| Claim | Class | Notes |
|---|---|---|
| Front matter (revision) | (c) | |
| 2: a type key written with different letter case does not set the field (S-0001 claim 13); the sentence about the stock data's spellings is gone | satisfied | the importer matches keys case-sensitively. `jumpjet_type_keys` no longer names the stock data: a type with altered-case keys only gets the absent-key defaults, and a later layer's altered-case keys leave the earlier layer's values. The doc comment of `UnitType::jumpjet_accel` is reworded the same way. |
| 2: backing (J1 b controls, J1 c case) | (c) | |
| 3: backing (uncommitted hovering idle values `unmeasured`) | (c) | |
| 4: backing (the independently falling exclusion `unmeasured`) | (c) | |
| 6: acceleration capped at *S*, braking floored at 0, not clamps to the maximum speed; an overshoot can brake in the same tick | satisfied | a restatement of F1, which did not change (`jump.rs` F1). Test: `claim_6_f1_speed` (the overshoot tick). |
| Edge cases, zero speed or turn rate: "Stock data has neither" removed | (c) | |
| Edge cases, no landing slot, switch off: the integer height above the surface decides: 1 lepton or more hovers; 0 or less stays put and becomes *grounded*, including from *ascending* at height 0 | changed | `jump::stop_with`: `height_above` < 1 sets *grounded*, clears point, order and commitment and releases any hover slot (as at claim 15's touchdown); before, only a *grounded* jumpjet stayed grounded and every other state hovered. Test: `claim_17_no_cell_switch_off_height_decides` (*ascending* at 0 after its take-off update, *hovering* at 0 and *ascending* at −1 become grounded with no slot; *ascending* at 1 hovers with its cell's slot; none moves over ten updates). Fails without the change. |
| Edge cases: every seized set-down unit in that outcome, by any route, follows S-0607 claims 9 and 20 | changed | as S-0607 claim 20. |
| Import mapping, Key case: matched case-sensitively; an altered-case key does not set the field; with no exact key the default applies, otherwise an earlier layer's value remains | satisfied | `jumpjet_type_keys` (both cases). |
| Common setup, scenarios J1 a and c, J24, J27 c | (c) | J27 c's engine-design checks are the tests named above. |

## Format and gate

- No saved state is added: the hovering set-down unit is still recognised from existing state. The save format stays 58 and the AI game format stays 23.
- The switch defaults on, and the gate fixtures never turn it off, so the changed rules do not run in the duel.
- Seed sweep over seeds 1 to 16: the same counters as at steps 1o to 1q, to the last digit, with the same early ends (seeds 3, 6, 7, 9, 11, 12, 13 and 16) and shortfalls. Seed 2 still plays all 12 000 ticks with every counter at least 3 except the sensor orders' known 2, so `M7A_SEED` stays 2. No golden changed, and `cargo test --release -p glasnost-sim-gate -- --include-ignored` passes without the update flag.
