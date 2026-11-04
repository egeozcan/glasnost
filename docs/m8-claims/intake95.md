# M-0021 as revised for Q-2516: claim ledger, M8 step A3 intake 95

M-0021 (revised in 92b83da) answers **Q-2516**; its file is at status answered. The change is in
the arena tool only (`crates/tools`): no simulation rule, hashed state, format or golden moved
(save 97, AI game format 60, wire 12 as on main). Request filed: **Q-2536** (when F first fires
once its target is in range, and why part B's outcome varies with the seed in the record).

| Q-2516 question | record's answer | change |
|---|---|---|
| 1. Creation order | each house's starting construction vehicle (removed), then F's 16, then G's 16 whichever side F stands on, then the observer's building; within a group in increasing index | the interim already did this (`oracle_m0021::F_FIRST`, now cited to the record); the observer's building is now created last, after both groups (`oracle::setup_with`, `SetupOptions::observer_building`); test `the_observer_keeps_no_vehicle_and_f_is_created_first` |
| 2. G's numbered weapons | slots 0 and 1 hold stage 0's weapon, slots 2 and 3 stage 1's; primary and secondary are both stage 0's (the standard references do not take effect with a turret count of 1, S-0004 claim 13) | the interim was the same; the comment in `test_ruleset` now cites the record (`the_test_types_carry_the_records_fields` checks it) |
| 3. The observer | British, start position 3, one building (type, cell, facing, mission `local`), nothing else on the map at tick 0; its starting construction vehicle is off the map at tick 0 and gone from tick 1 on | the observer's vehicle is taken off with the others (`take_off_vehicles`); one unarmed building of the rules (`GLASNOST_M0021_OBSERVER`, else the first unarmed 1 by 1 type) is placed on the start 3 waypoint's cell after the groups. The house stays the arena's idle test house (no target, no damage), as the observer is not a lobby slot in our setup |
| 4. The run's end | parts A, A0 and control play to tick 608 (limit 600 + 8); shots and hits are recorded to 608, per-tick rows to 607; a shot at tick 608 is not judged; every judged last shot has all its hits by 607. Part B ends after one side has no vehicle left, its per-tick rows end one tick before the last tick | `END_EXTRA`; `Recording::last_in_play` keeps the last tick's vehicles for deaths only; the cut-short rule (`Findings::cut_short`, `Recording::in_flight`) is removed; test `a_shot_at_the_runs_last_tick_is_not_judged` |

## Result rows on the install

22 of 24 rows pass, as on intake 93; the same two fail with the same numbers (see
M8-A3-INTAKE95-REPORT.md and Q-2536). Both are independent of the base tank (MTNK and HTNK give
the same numbers).
