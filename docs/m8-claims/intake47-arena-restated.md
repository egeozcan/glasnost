# Intake 47: the arena records restated (M8 step A3)

Records: M-0001, M-0002, M-0003, M-0011, M-0015 and M-0017, revised on 2026-10-29. No spec revision
comes with them. **No simulation code changed**: the oracle tooling (`crates/tools/src/oracle.rs`,
`crates/tools/tests/arena.rs`) follows the records' new CSVs, tolerance and Setup. Requests
Q-2096 to Q-2099 (Q-2100 unused).

## What the records changed, and what we did

| item | status | tests or reason |
|---|---|---|
| The per-game CSVs have six columns, no army shares (M-0001 Procedure step 3 and per-game data). On main the arena read only the old header, so it skipped every record and its tests passed without reading one | tested | `oracle::tests::csv_has_the_six_columns_of_the_restated_records` (the old form is refused); `arena_records_parse` now fails on a record CSV with any other header (`oracle::unrecognised_csvs`) |
| Only the pairings without SREF or V3 stand as stock and are scored: 84 of M-0001's 96, all 24 of M-0002's, 24 of M-0003's 35 (M-0001 and M-0003 *Tolerance for tests*, M-0017). The others are played and shown with what the tests would say, no verdict | tested | `oracle::tests::sref_and_v3_pairings_are_shown_but_not_scored`; `scored_pairings_and_mind_against_e2_are_those_of_m0017` (in CI, on the records themselves) |
| Per scored pairing: the two-proportion test on the row's wins (p < 0.01), and the same winner on every seed of a side where the stock games all had one. The share KS tests are gone | tested | `oracle::tests::the_win_test_and_the_side_outcome_make_the_verdict` |
| MIND against E2 runs on M-0017's restated line (n = 5: 3 row wins, 2 column wins, no `none`; side B seeds 1 to 5; no side-A game). The 7 crashed games are `none` with an end frame short of the limit (M-0001 Procedure step 3) | tested | `oracle::tests::crashed_stock_games_are_left_out_of_both_sides`; `scored_pairings_and_mind_against_e2_are_those_of_m0017` |
| A unit its stated cell does not take is placed on a free neighbouring cell (M-0001 to M-0003 and M-0017 Setup; "a free cell nearby", M-0011 and M-0015 Setup) | built, our design; request | `oracle::placed_cell`: a cell takes a unit when no unit stands on it and the unit's entry outcome there, asked as S-0216 claim 5 asks it, is *clear*; a refused unit goes to the first cell that takes it on the rings around its stated cell, in S-0212 claim 25's scan order, first come, first served. Tests `rings_follow_the_nearby_search_scan_order`, `a_unit_on_the_stated_cell_sends_the_next_one_to_a_ring`. The test, the order and the rings past the first are ours: **Q-2096**. Superseded by S-0216 r37 claim 8 ([intake 71](intake71-S-0513-S-0216.md)): the height-aware take test, the eight neighbours in claim 8's order, an error past them; the rings are gone |
| The units are not map entries; they are created on their stated cells after the scenario has loaded, before tick 0 (M-0011, M-0015). S-0216 claim 6 rests on "every unit stood on its stated cell", which the records withdraw | request | **Q-2097**; A1's interim kept (cell centre, no standing-position draw, *hunt* current). Since intake 71 (S-0216 r37 claim 8): each unit created, tested and placed in turn, infantry on a drawn standing position, *hunt* queued |
| The map's four bunkers, owned by a neutral house, are in play from tick 0 | request | Our tick-0 state has the neutral house's map buildings in play, but none has S-0606's bunker flag; we cannot tell which four the records mean and do not pick them by name: **Q-2098**. Nothing changed in the loader. Answered in part by S-0216 r37 claim 8 ([intake 71](intake71-S-0513-S-0216.md)): the arena loads the map as any game does and picks nothing by type; acceptance is a neutral building covering each of M-0006's four bunker cells, checked on the local lane. The bunkers' kind stays open |
| The group counts stay round(9000 / cost), but the CSVs no longer show how an exact half goes: the army shares were what showed the even count | request | `oracle::round_half_even` keeps the even count; **Q-2099**. Answered in part by S-0216 r37 claim 8 ([intake 71](intake71-S-0513-S-0216.md)): the even count, for both kinds; tested |
| M-0011 and M-0015 make several per-unit values local | docs | The ledgers of [intake 38](intake38-M-0011.md) and [intake 40](intake40-M-0015.md) now show those as local; their other rows are unchanged, and so are our CAOS and TELE against BFRT games (no unit of theirs moves) |

`glasnost-headless arena --groups` now also prints, per type and side, the units placed off their
stated cells and why, and the objects at tick 0 that neither side owns (local lane only: the cells
and types stay there). `--watch` prints the moved units of its game.

## Who moves

Only side-A units move, in the groups that reach the third column (x = W.x − 8): a neutral building stands
on its first cell, so its first unit moves in every such group, vehicles too; in the largest
infantry groups more units move, off that building, the cliff cells of Q-1502, a few more cells
with entry outcome *blocked* and the cells moved units took first. No side-B unit moves.

## Arena, before and after

Local lane, `local-lane/m8/a3-47/` (never committed). *Before* is main 630c04e's games scored with
the new tooling (main's own tooling skipped every record); *after* is this intake.

Scored pairings failing (win test failing, side outcome differing), and side-wise unanimous stock
outcomes reproduced:

| record | before | after |
|---|---|---|
| M-0001 | 10 of 84 (5, 9); 140 of 152 | 11 of 84 (3, 10); 139 of 152 |
| M-0002 | 5 of 24 (4, 5); 42 of 47 | 2 of 24 (0, 2); 45 of 47 |
| M-0003 | 4 of 24 (0, 4); 38 of 42 | 4 of 24 (0, 4); 38 of 42 |
| all | 19 of 132 | 17 of 132 |

Not scored (SREF or V3), before and after alike: M-0001 1 of 12 would fail, 17 of 19 unanimous
outcomes reproduced; M-0003 0 of 11, 22 of 22.

What moved: in M-0001, YTNK against E1 now passes, and MIND against E1 passes the win test (one
side outcome still differs): before, some of their games ended `none` with side-A units left on
cells no hunt reaches (Q-1502), and those units now stand on a cell around them. CAOS against E1 and CAOS against SHK
now fail by one seed each on side B (one `none`, one row win). In M-0002, INIT against E1, VIRUS
against E1 and VIRUS against E2 pass, and INIT against E2 passes the win test (one side outcome
still differs). M-0003 is unchanged in its verdicts.

Every game whose side-A group has no moved unit is byte for byte the same before and after
(M-0001: 972 games, M-0002: 120, M-0003: 378). Of the games with a moved side-A unit, 32 of 180,
53 of 168 and 2 of 42 change.

## The per-unit records against ours today

Our CAOS against BFRT and TELE against BFRT games (M-0001, side A, seeds 0 and 1) have no moved
unit, so this intake does not change them. Re-traced on this branch (`arena --trace-summary`,
local lane), with the records' values that are not local (seed 0; seed 1):

| record fact | stock | ours today |
|---|---|---|
| M-0011: tick of the first gas on each fortress | 50 to 58 | 51 to 58 |
| M-0011: end of game | BFRT wins at 608 and 912; every drone leaves play | frame limit (6080) in both; no drone leaves play (Q-2005, as at intake 38). Intake 52a ([ledger](intake52a-S-0210-S-0215-S-0216.md)) builds Q-2005's answer: side A now ends `col` in all six seeds (304 or 608) |
| M-0011: fire of one fortress at another | 198; 275 | 445; 476 over the whole game |
| M-0015: fire events that started a hold | 56; 52 | 35; 25 |
| M-0015: fire events at a fortress held by another Magnetron | 501; 439 | 130; 163 |
| M-0015: other fire events | 22; 20 | 52; 61 |
| M-0015: hits on Magnetrons by fortress fire | 15; 18 | 18; 17 |
| M-0015: end of game | tick limit (6080), units left on both sides | TELE wins at 3344 in both |

Our TELE games have changed since [intake 40](intake40-M-0015.md)'s table (later intakes); that
table stays as it was then. The local values were compared in the local lane only.

## Checks

- No simulation change: `git diff 630c04e -- crates/deterministic crates/sim-gate` is empty, so
  the save format stays 85 and the AI game format 47, and no golden was regenerated. The seed
  sweeps run the same code before and after; they were not re-run.
- `cargo test --workspace` passes; `cargo test --release -p glasnost-tools` passes (the arena
  tests in CI read the records' CSVs).
- `cargo xtask check`: deps, determinism, cite and canary ok; stock-guard reports the 16 problems
  of the records themselves (3 lines of M-0015, 13 of M-0013: a number in the Result table of a
  record whose values are local), which are the lead's to clear; nothing of this branch.
- `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean.
