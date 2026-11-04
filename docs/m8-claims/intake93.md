# S-0318 r29, S-0609 r7, S-0301 r71 and M-0021: claim ledger, M8 step A3 intake 93

S-0318 r27 to r29 (e33f8b2), S-0609 r5 to r7 (r6 rewords the scenario preamble; r7 is 1a3f521),
S-0301 r70 to r71 (fa9af8f) and the new arena record M-0021 (20280b0) answer **Q-1714**. Its file
stays at status open, answered-by S-0318, M-0021, S-0301 and S-0609. No interim rule for Q-1714
was ever in code.

The three spec deltas change only backing and confidence text: claims built earlier are now
`measured (M-0021)`, and S-0318's Open questions are restated. No rule, number, order or
quirk changed, so there is no simulation change and no new switch
(`every_spec_quirk_has_a_switch` passes, and no C7 verdict is due). What is new is the harness
that checks M-0021's rows in our engine. Request filed: **Q-2516** (what M-0021's setup leaves
unstated: creation order, G's numbered weapon slots, the observer, and a last shot cut by the
run's end).

| item | status | tests or reason |
|---|---|---|
| S-0318 claims 1, 2, 3 and 5 backed by M-0021 (a judged shot at a vehicle bursts into 3 fragments on 3 distinct G vehicles in rings 1 to 3, none on the firer's side, each naming the firer; no fragments from a projectile with no fragment weapon) | tested | synthetic: `oracle_m0021::tests::synthetic_part_a_every_judged_burst_hits_three_candidates`, `synthetic_part_a0_has_no_fragments`; install: M-0021's part A, A0, B and control rows (local lane, `arena::m0021_rows_hold_in_our_engine`), all pass |
| S-0609 claims 2 and 3 backed by M-0021 (rise 30 then 31, cap 60, part B's fall 30 then 29) | tested | synthetic: `oracle_m0021::tests::synthetic_part_a_stages_and_gaps`; install: the three stage rows pass |
| S-0301 claims 2 and 3 backed by M-0021 (the shortest gap between one vehicle's shots is trunc(*R* × *h*) with the draw at 0) | tested | synthetic: `synthetic_part_a_stages_and_gaps`; install: the three gap rows pass |
| M-0021's Procedure (void check, windows, judged shots, *c*, the first 3 candidate cells of T1, deaths) | tested | `the_procedure_judges_a_hand_made_recording`, `a_cell_holding_an_f_vehicle_is_not_a_candidate`, `a_last_shot_cut_short_by_the_end_is_not_judged`, `a_distance_rounds_to_whole_leptons` |
| M-0021's Setup (16 a side on M-0001's cells, F always house 1 and G house 2, the neutral house's objects taken off, the easy row for both houses, the control's rof factor) | tested | `the_test_types_carry_the_records_fields`, `the_houses_do_not_change_with_the_side`; `oracle::setup_with`, `oracle::take_off_neutral` |
| First shots: F fires first; G's centre distance | tested | install rows pass |
| First shots: F's centre distance | partly | install row fails: ours is 1504 leptons in every run, the record's 1527. The record's Notes say this row may depend on how the vehicles move (fields kept from the copied stock types, country speed factors) and Q-2516 asks about the setup |
| Part B's outcome table | partly | install row fails on 2 of 6 runs: (A, 1002) ours 4 G dead, the record 7; (B, 1001) ours 7, the record 6. Side left (G) and F dead (16) match in every run. Same caveat as above |
| Part B: every G vehicle reads stage 1 before the first death | tested | install row passes; the lead's range is shown (not for tests) |
| The first 3 candidate cells of T1's order (not for tests) | shown | ours, like the record's, is "all but a few" at *t* and fewer at *t* − 1 to *t* + 1 |

## Built

- **Effects for the recorder** (presentation only, `#[serde(skip)]`, not hashed: no format bump).
  `Effect::Shot` carries the shot's `target` and `Effect::Hit` its `warhead` and the target's
  health before and after (`combat::effects`, set in `combat::fire`, `combat::projectile` and
  `combat::damage`). The audio crate's patterns follow.
- **`oracle::setup_with`** with `SetupOptions` (the group size, the two houses, taking off the
  neutral house's objects) beside `oracle::setup`, and `oracle::stock_inputs` beside
  `stock_ground`. The old entry points behave as before.
- **`oracle_m0021`** builds the record's test types on a tank type of our choice
  (`GLASNOST_M0021_BASE`, default `MTNK`) and a copy of a stock invisible projectile that can
  hit ground targets (`GLASNOST_M0021_PROJECTILE` overrides). It plays the 16 runs, records
  shots, hits, positions and spin each tick, judges each run by the Procedure, and gives one
  verdict per row of the Result. `glasnost-headless arena --record M-0021` runs it (and
  `--record all` adds it after the other records). The verdicts go to stdout. The report with
  the record's local values goes to the git-ignored `oracle-M-0021.txt`.

## Interim rules (Q-2516)

1. F's group is created first, whichever side it stands on.
2. G's numbered slots are stage 0, stage 0, stage 1 and stage 1. Its primary and secondary
   weapons are both the stage 0 weapon.
3. The human player is the arena's idle human house with its base unit.
4. A firer's last shot is not judged when it falls in the run's last 2 ticks, or when a
   projectile of that firer is still in flight at the end (`Findings::cut_short`). Without this
   rule, the control's last shots fail the hit rows: the run stops before their third fragment
   lands.

## Checks and lanes

The base was 6b439cd, rebased onto 445d0f1 before hand-back. On 6b439cd, every lane gave the
same result before and after this change: the M6c, M7a and M7b sweeps over seeds 1 to 16, the
AI quick lane (36 games, seeds 7 to 10 on maps 0, 2 and 8), `glasnost-ai` with ignored tests,
the sim-gate goldens, and the arena (13 of 155 scored pairings failing, the same pairings). The
new effect fields change no game. After the rebase, the sweeps and the arena match 6b439cd. The
quick lane's games differ, which comes from main's own AI commits (seam trace 4). All 36 games
end with a winner. `glasnost-ai` with ignored tests passes. M-0021's rows are the same on both
bases.
