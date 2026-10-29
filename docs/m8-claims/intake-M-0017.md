# Intake M-0017: stock replay of the arena's games without SREF or V3

Record: `measurements/combat/M-0017-stock-replay.md`. No simulation change; no spec requests (Q-2020 to Q-2024 unused).

| item | status | tests or reason |
|---|---|---|
| The 7 crashed MIND against E2 games of M-0001 are left out (stock and ours of the same side and seed), so that pairing is judged on n = 5 | tested | `oracle::tests::crashed_stock_games_are_left_out_of_both_sides`; `oracle::crashed` (winner `none`, end frame short of the limit sample) |
| SREF and V3 pairings reported apart as "not confirmed by M-0017", still judged; totals with and without | tested; since intake 47 not scored | since the records' revision (M-0001 and M-0003 *Tolerance for tests*) only the stock pairings are scored; the SREF and V3 pairings are played and shown with what the tests would say: `oracle::tests::sref_and_v3_pairings_are_shown_but_not_scored`; `oracle::confirmed`, `UNCONFIRMED_TYPES`; [intake 47](intake47-arena-restated.md) |
| M-0001 to M-0003 notes (harness changes were on) | lead's | M-0017 hands them to the lead; none of our docs repeat the "changes off" sentence |

Arena (local lane, `local-lane/m8/a3-m0017/before` and `after`), pairings failing any test / side-wise unanimous stock outcomes reproduced:

| record | before | after | confirmed only (after) | not confirmed (after) |
|---|---|---|---|---|
| M-0001 | 31 of 96, 150 of 172 | 30 of 96, 150 of 171 | 26 of 84, 132 of 152 | 4 of 12, 18 of 19 |
| M-0002 | 12 of 24, 42 of 47 | same | 12 of 24, 42 of 47 | none |
| M-0003 | 2 of 35, 60 of 64 | same | 2 of 24, 38 of 42 | 0 of 11, 22 of 22 |
| all | 45 of 155 | 44 of 155 | 40 of 132 | 4 of 23 |

Our CSVs are unchanged; MIND against E2 now passes on its 5 side-B games. The "unanimous outcomes" count falls by one because the pairing's side A has no stock game left.
