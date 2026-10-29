# S-0607 r14: claim ledger, M8 step A3 intake 45

This intake takes in **S-0607 revision 14** (magnetic lift; spec commit 23c421b). Our engine
built revision 11 (taken in at M7b step 1c). Revisions 12 and 13 were not published to this
repository; the diff from r11 is read as one change. r14 answers **Q-2035** (intake 40,
[intake40-M-0015.md](intake40-M-0015.md)): when a set-down unit in *hunt* gets its own movement
back. No new quirk, import row or formula comes with it.

## What r14 changes

- **Claim 20, when restoration is tested.** The test is made at the end of each of the unit's
  turns (S-0203), after the point at which it may process its behaviour and after its movement,
  whatever its behaviour and house. The test at a fall-back to rest stays. A clear-cell
  touchdown is restored at the end of the touchdown turn; *guard* and *hunt* are restored at the
  same tick. Restoration starts no behaviour: a hunter keeps *hunt* and hunts from its next
  processing. Units being erased or in a chrono move's leaving stage are not restored.
- **Claim 20, the hovering set-down unit** (switch off): no test at any time; its stop's search
  is retried at the end of each of its turns, not at a fall-back to rest.
- **Claim 9:** the set-down unit is restored at the end of its first turn after the stop.
  **Claim 13:** a set-down unit is restored at the end of its turn in the tick of the grip's
  end when that turn comes later in the update order, else at its next test.
- **Claim 6:** *hunt* is still current when the unit is restored.
- Edge cases, scenarios L3, L9 part g, L11 parts a, c, e and f, L14, Open questions, and
  Changes for other specs follow these.

## Built

- `lift::end_of_turn`, called last in `tick::object_turn`. A turn that ends early has no test:
  a lost turn, a dead unit, a sinking vessel, being erased, and a warp member's warp-only update
  (the phase-in stage included, design for the open question).
- `lift::restoration_test` is shared by both tests (`restore_at_rest` at a fall-back to rest).
  It refuses a unit being erased or in the leaving stage (`superweapons::being_erased`), also a
  landed aircraft (design for the open question). The hovering set-down unit runs
  `jump::lift_retry_stop` only at the end of a turn.
- No new state: no save or AI format bump.

## Design choices for the open questions

| open question | our choice |
|---|---|
| a deactivated or paralysed vehicle | tested like any other turn (its turn does not end early) |
| a unit phasing in after a chrono move | no test in its warp-only updates |
| a landed aircraft being erased | not restored, as any unit being erased |
| claim 9's "first turn after the stop" | the first end-of-turn test after the stop; in our order the stop comes in the unit's own movement, so it is the same turn |

## Claims

| claim | status | tests or reason |
|---|---|---|
| claim 20, the test at the end of each turn; a clear-cell touchdown restored at the end of the touchdown turn | tested | `claim_20_restored_at_the_end_of_the_touchdown_turn` (`claims_s0607_r14.rs`) |
| claim 20, behaviour and house make no difference (*guard* and *hunt*, human and computer house) | tested | `claim_20_hunter_and_guard_restored_at_the_same_tick` |
| claims 6 and 20, after restoration a hunter keeps *hunt*, takes a target and fires; none before | tested | `claim_20_a_restored_hunter_takes_a_target_and_fires`, `m0015_a_set_down_computer_unit_in_hunt_is_restored_and_fires` (`claims_s0607_m0015.rs`) |
| claims 3 and 20, a restored unit can be seized again (L11 part e run 1b, k = 1, 5, 30) | tested | `claim_20_a_restored_unit_can_be_seized_again` |
| claim 20, not restored while being erased, neither at the end of a turn nor at a fall-back to rest; restored after (L11 part f) | tested | `claim_20_not_restored_while_being_erased` |
| claim 20 and S-0502 Edge cases, the hovering set-down unit: no test; retry at the end of each turn, not at a fall-back to rest | tested | `claim_20_hovering_set_down_retries_at_the_end_of_each_turn`; `claim_20_hovering_set_down_retries_the_search` and `claim_20_hovering_exclusion_for_a_committed_set_down` (`claims_s0607_release.rs`) |
| claim 20, the first order after touchdown moves the unit | tested | `claim_20_restored_at_landing_first_order_moves` (`claims_s0607.rs`) |
| claims 9 and 13, a grip ended at height 0 or less: grounded and restored at the end of that turn | tested | `claim_9_no_cell_on_the_ground_stays_grounded`, `claim_9_no_cell_ascending_at_height_zero_becomes_grounded` (`claims_s0607_release.rs`) |
| claim 10, a seized unit at height 0 that is reached deploys (L9 part g) | partly | `claim_10_gripped_seized_unit_deploys`: no order route reaches a seized unit at rest any more, so the test calls the deploy step on a gripped unit directly |
| claims 23 and 24, a landed aircraft with the teleporting stand-in (quirk off) restored with no order, then takes off | tested | `claim_23_landed_aircraft_restored_takes_off` (`claims_s0607_r7.rs`) |
| claim 24, the stand-in quirk on: never restored, at any test | tested | `quirk_lift_standin_stays_seized_on`, `_on_drop_pod` and `_off` (`claims_s0607_standin.rs`) |
| claim 20, a gatherer's harvest queue at restoration | unchanged | built at r6; its tests pass |
| Q-2035 | answered | by claim 20 above; intake 40's request row is retired |

## Left to the spec side

- S-0215 claim 13 row 1 and S-0502 Edge cases and J27 (c) still say "fall-back to rest"; S-0607's
  Changes for other specs names the replacement texts. We follow S-0607.
- The Questions for other specs (S-0417 claim 5, S-0207 claims 21 and 28, S-0402's chrono miner,
  Q-0233) are questions, not rules; nothing is built for them.

## Arena and sweeps

See the report of this intake for the before and after tables (local lane,
`local-lane/m8/a3-45/`).
