# S-0214 r78: claim ledger, M8 step A3 intake 85

The last built revision of S-0214 is r68. This intake builds r68 to r78. The delta was read in
full (`git diff c739d66~1 c739d66 -- spec/ui/S-0214-order-targeting-rules.md`). It answers
**Q-2356**, which intake 79 filed with an interim rule ([intake79.md](intake79.md), claim 9). That
interim is retired: the attack rule's part (e) no longer passes over the materialising
*rearming*. It now takes the firing check's answer.

Requests filed: **Q-2426** (which answers part (e) passes over with
`quirks.moment_reason_attack_cursor` off). Q-2427 to Q-2435 are unused.

**What changes.**
- `targeting::attack_rule` part (e) asks `combat::check::check` without the range test. That is
  S-0301 claim 9's answer of the highest rank that holds, and part (e) fails only when it is
  *illegal*.
- With the new switch off, part (e) asks `check::illegal_at_any_rank` instead (Q-2426 interim).
- `targeting::movable` leaves out a phasing-in mobile actor. That Terms part dates from r68 and
  was not built before. It reaches every rule that asks *movable* (see the Terms row below).
- `check::check_lasting` stays, as the own AI's lasting test (our design, PLAN §4.7). The AI
  modules that cited S-0214 claim 6 (e) for it now cite S-0301 claim 9. The AI still gives its
  orders through `targeting::click`, so its accepted attack orders follow the new part (e).

**Switch.** One new quirk switch, `quirks.moment_reason_attack_cursor` (default on, the
original), with both positions tested. Its C7 verdict is in [c7-play-fixes.md](c7-play-fixes.md)
under *Intake 85*: rejected. `quirks.phase_in_beam_spares_infantry` is named in claim 6 (e) and
in the Edge case. It is S-0513's and intake 84's, and only cited here.

**Formats.** The switch is a rules field, so `data/schema/ruleset.schema.json` gains it. No saved
or hashed simulation state is added, so the save format, the AI game format and wire stay as
they are on main. The goldens do not move: the gate replays orders, and no golden clicks.

Status words: **tested** (built and checked by the named test), **partly** (the part named is
untested or not built), **seam** (built by another intake's spec; the reason is given), **no
change** (the revision restates what is built).

Tests are in `crates/deterministic/sim/tests/claims_s0214_r78.rs` unless another file is named.

## S-0214 r78

| claim part | status |
|---|---|
| 6 (e): part (e) takes the firing check's answer for that target and weapon, the answer of the highest rank that holds (S-0301 claim 9), and fails only when it is *illegal*; a reason of the moment ranked below every target test (not facing, still turning, moving, out of range) does not count (Q-2356) | Tested: `attack_rule` asks `check::check(.., with_range: false) == Illegal`. The range test is part (d)'s. The checks of ranks 13 to 15 come after every target test, so they cannot hide one. `t4f_the_countdown_offers_the_attack_over_a_target_its_warhead_cannot_affect`; the existing S-0214 tests (`claims_s0214_combat.rs`, `claims_s0214_garrison.rs`, `claims_s0214_tap.rs` and the unit tests in `targeting.rs`) still pass |
| 6 (e): while the rearm countdown runs (rank 5, with the firing-frame gate and the weapon's own live effect), the late target tests of rank 9 (zero versus, takeover, bomb, parasite, sinking) and rank 12's vehicle tests are not asked; the actor is offered the attack and a click gives an attack order | Tested for the countdown over a zero-versus target: `t4f_the_countdown_offers_the_attack_over_a_target_its_warhead_cannot_affect` (action and order, the control before and after the countdown). The gate and the live effect come from the same check (`answer`'s rank-5 block). The other rank-9 tests and rank 12 follow from the ranks and are not tested one by one |
| 6 (e): out of ammunition (rank 7) and *cloaked* (rank 8) hide the same tests | Tested: `out_of_ammunition_and_cloaked_hide_the_zero_versus_test` |
| 6 (e): S-0306 claim 23's water and land rows rank above the countdown and still refuse | Tested for the land row, for a building and for a vehicle: `the_land_row_outranks_the_countdown_but_not_the_early_rearming_answers`. The water row is the same block of `answer` and is not tested apart |
| 6 (e): a unit drops the target, without firing at it, at its first checking turn in a check tick at which no reason ranked above the refusal applies (S-0309 claim 29); a holder with a turn in every tick drops it within 16 ticks of the last such reason ending | Tested for a vehicle in its countdown: `t4f_the_target_ordered_in_the_countdown_is_kept_to_its_end_and_dropped_unfired` (kept through the check ticks in the countdown, dropped at the first one after it, no shot). The drop is the existing held-target check (`behave`, S-0309 claim 29). A missed or cut-short turn, the countdown ending while the unit is out of ammunition, and an infantry holder are that check's rules, not tested again here |
| 6 (e): its firing step makes no drop on *illegal*, except a healer's (S-0310 claim 11) | No change: the firing step's replies are S-0309's and S-0310's (intake 79). The no-shot assertions above show that no shot is made |
| 6 (e): a building drops it at its attack processing (S-0215 claim 32), or earlier where S-0309 claim 29's check covers it; a building without the SAM flag that is out of ammunition drops it at its first attack processing | No change: the building's attack processing and its drop rows are S-0215 claim 32's, built before (`behave`, non-SAM out-of-ammunition path). The building cases here test the offer and the order only |
| 6 (e): so over an enemy its weapon cannot affect, the action changes with the countdown | Tested: `t4f_the_countdown_offers_the_attack_over_a_target_its_warhead_cannot_affect` |
| 6 (e): a falling actor fails part (e) unless force-fire or the infiltrator exception, and a falling vehicle that is phasing in is the exception (the materialising *rearming* outranks the falling *illegal*) | Tested: `a_falling_vehicle_that_is_phasing_in_holds_part_e`; the falling actor itself, as before, in `claims_s0214_*` (S-0522 claim 17) |
| 6 (e): a falling infantry unit that is phasing in: which answer it gets rests on S-0310's open question | Partly: open in S-0301 claim 9 and S-0310. Our check gives *rearming* (the falling *illegal* is in the general part, after the materialising block). No request: the spec tracks it |
| 6 (e): a phasing-in actor (after its own jump, or as a warp member) gets *rearming* for every target and weapon, ahead of the target tests: a target in the air the weapon cannot hit, a protected object for a parasite warhead, a warhead that cannot affect the armour, the water and land rows; part (e) holds unless an answer ranked above it is *illegal* | Tested for a target in the air and for the land row: `t4e_a_phasing_in_actor_is_offered_the_attack_over_a_target_in_the_air_in_range`, `the_land_row_outranks_the_countdown_but_not_the_early_rearming_answers`. The other target tests follow from the same block. A warp member's phase-in is the same state (S-0513 claim 39, `teleport::phasing_in`) |
| 6 (e): a hidden target gives the same result as for an actor not phasing in (its answer is *impossible*, not *illegal*, S-0516 claim 24) | No change: the hidden-target *impossible* comes after the materialising block in `answer`, so while phasing in the actor gets *rearming* there instead. Part (e) passes either answer, as it does the *impossible* for an actor that is not phasing in |
| Terms and 6 (d): a phasing-in actor is not movable, so part (d) asks that the target be in range of the weapon it would use; in range, row G8 gives *attack* whether or not the weapon can hit it; out of range, row G9 gives *nothing* | Tested: `targeting::movable` asks `teleport::phasing_in` for infantry, vehicles and aircraft. `t4e_a_phasing_in_actor_is_offered_the_attack_over_a_target_in_the_air_in_range` checks `movable`, the attack and the order over the target in range, and *nothing* and no order over the one out of range. The Terms' "not phasing in" is r68 text that no intake had built (r78 leaves the Terms unchanged and relies on it in claim 6 (d)). Building it reaches every rule that asks *movable*, not only part (d): the object rows of table G that ask it (G6, G7 and the others), table M's orders (move, enter, capture, recycle, area guard), S-0213 table T2's scatter and guard and the deploy rule. A cell click was already refused by row GC1. The band box (claim 19) does not ask *movable* and is unchanged. `a_phasing_in_unit_is_not_movable_for_the_key_commands` checks scatter (refused) and stop (still given: able to fire) |
| Terms, *movable*: not being erased, not a slave unit | Partly, unchanged by r78: neither is asked in `movable`. Row G2 refuses a click of an actor being erased before the attack rule, and our engine has no slave units (S-0215's slave master is not built) |
| 6 (e): a vehicle or aircraft phasing in with a temporal beam attached and not being erased (an infantry unit too, with `quirks.phase_in_beam_spares_infantry` off) is offered the attack and given the order, but drops the target in each of its turns (S-0309 claim 29, S-0513 claim 39a) | Seam (S-0513 claim 39a, intake 84 and earlier): the drop in each turn is built in `behave` under S-0513's switch. The offer is the general phasing-in case above: G2 refuses only an actor being erased. Not tested together here |
| 6 (e): an aircraft actor makes S-0309 claim 29's two aircraft drops at the start of each turn (S-0501 claim 32) | Seam (S-0501 claim 32): built in the aircraft turn. Not tested together with phasing in |
| 6 (e): any other phasing-in actor keeps the target while phasing in, unless a drop that does not depend on its own turns takes it (for example S-0516 claim 14's cloak drop); once phasing in ends it drops a target answered *illegal* at its first checking turn in a check tick | Tested for a vehicle and a target in the air: `t4e_the_target_in_the_air_is_kept_through_the_phase_in_and_dropped_unfired` (kept through the check ticks in the phase-in, dropped at the first after it, no shot). The cloak drop is S-0516 claim 14's (seam) |
| 6 (e): the other early *rearming* answers before the water and land rows (a spawner weapon none of whose spawned aircraft is alive, a target the firer's temporal beam holds, an own building's pending delayed shot) give the same; they rank above ranks 9 and 12 too | Tested for the delayed shot, against the land row and against the zero-versus test: `the_land_row_outranks_the_countdown_but_not_the_early_rearming_answers`, `a_pending_delayed_shot_hides_the_zero_versus_test`. The spawner and temporal-hold answers are the same kind of block in `answer` and are not tested apart |
| 6 (e): the firer's own gripped unit is refused by neither row; a firer only waiting out its countdown is not offered the attack over a target a row refuses | Tested for the countdown: `the_land_row_outranks_the_countdown_but_not_the_early_rearming_answers`. The gripped unit is S-0306 claim 23's and S-0607 claim 11's, built before |
| 6, backing and confidence | No change: measurement text |
| Edge cases, `quirks.moment_reason_attack_cursor` (default on, the original): as claim 6 | Tested, on: every test above |
| Edge cases, the switch off: part (e) asks only whether a reason that gives *illegal* holds, at any rank, passing over every reason of the moment; over an enemy the weapon cannot hit or affect the actor gets *nothing* from row G9 and no attack order, whatever its countdown, ammunition or phasing in; part (d) and the rest of the attack rule are the same | Tested, off: `illegal_at_any_rank` (the `Ask::IllegalOnly` reading of `answer`). The off branch of each test above, and `claims_s0214_combat::claim_6e_switch_off_reloading_neither_gives_nor_takes_the_attack` (intake 79's countdown test, now set off). Which answers the off position passes over beyond those the quirk text names (the grip and magnetic-beam *rearming*, the *impossible* answers of rank 4, *busy*, *must deploy*): **Q-2426**, interim the literal reading (every answer that is not *illegal*) |
| Scenario T4 parts E and F | Tested as engine tests in the T4F and T4E tests above. Part E's variant, a superweapon warp member, is the same phasing-in state and is not run apart. The scenarios themselves are for the lead's harness |
| Changes for other specs: S-0304 claim 2 and its State table, S-0306 claim 23 and its Backing | No change here: notes to those specs' owners. Until they take the notes in, the two specs say the opposite of r78 (no attack cursor over a zero-versus target, no attack on a row-refused target while rearming). Our code follows S-0214 r78, whose part (e) owns the cursor, and its own Changes section covers the difference. `scripted::can_attack_tests` (the own AI's lasting test, re-cited to S-0301 claim 9) still asserts S-0304 claim 1's illegal answer |
| Changes for other specs: S-0204 claim 5 | No change: unchanged by r78 |

## Intake 79's Q-2356 interim

`claims_s0301_r69::q2356_the_attack_rules_check_passes_over_the_materialising_rearming` is renamed
`the_ai_lasting_check_passes_over_the_materialising_rearming` and re-cited to the own AI's lasting
test (S-0301 claim 9, our design). Its assertions are unchanged: `check_lasting` still passes over
the materialising *rearming*. The attack rule does not use it any more.
