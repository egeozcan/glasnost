---
id: Q-2987
title: Arena rows that disagree once the test houses' checks put the bunker occupiers out
status: open
related: [S-0217, S-0508, S-0215, S-0309, M-0001, M-0002]
answered-by: [S-0309]
---

## Need

S-0217 (r20) and S-0508 r39 answered Q-2951. Intake 124 built them
(`docs/m8-claims/intake124.md`): both arena houses are test-only computer-controlled player houses
with no building and no construction vehicle, so each makes a check at tick 0 and every 106 to
112 ticks after, and every check gives the house-wide hunt order (S-0217 claims 2 and 4). The order
queues *hunt* for every unit of the house on the map and puts the occupiers of its occupied
civilian buildings out with *hunt* next (S-0217 claim 1, S-0508 claim 13).

Four of the six rows Q-2951 named now give the record's outcome (M-0001 TELE–E1, TELE–E2 and
YTNK–E1, M-0002 INIT–E1, all side B). Two get further from it, and two rows that passed before now
fail. In each of these four rows, our E1 or E2 enter the neutral bunkers early, as before; at
their house's first check after tick 0 (tick 110 to 112) they are put out with *hunt* next, and
again at each later check after they re-enter. Before intake 124 they stayed inside and won from
there. Our counts over seeds 0 to 63, on the side named (the records give 6 of 6 there; a row is
within noise when our share *p* of the record's outcome gives *p*⁶ ≥ 0.01):

| Record | Pairing, side | Record's outcome | Ours before (64 seeds) | Ours after (64 seeds) | Verdict after |
|---|---|---|---|---|---|
| M-0001 | CAOS–E1, side B | col | arena seeds 0 to 5: col 6 of 6 | row 57, col 5, none 2 | systematic, new |
| M-0001 | CAOS–E1, side A | col | col 21, row 32, none 11 (A6b) | row 58, col 6 | systematic (was already) |
| M-0001 | CAOS–E2, side B | col | arena seeds 0 to 5: col 6 of 6 | row 53, col 10, none 1 | systematic, new |
| M-0002 | YURI–E1, side B | col | row 46, col 16, none 2 | row 63, col 1 | systematic (was already) |
| M-0002 | YURI–E2, side B | col | col 36, row 26, none 2 | row 49, col 15 | systematic (was within noise) |
| M-0002 | INIT–E1, side A | row | row 62, none 2 | row 38, col 24, none 2 (*p*⁶ = 0.044) | within noise, but moved |

CAOS–E1 side A was already systematic before intake 124 (`docs/m8-claims/a6b-arena-triage.md`,
row 14: there the E1 lose most units early and do not reach the bunkers); its count moved further
from the record. Its "before" count is A6b's, which intake 123 left unchanged.

Watched games (seed 0, side B, `arena --trace`):

- **M-0001 CAOS–E1.** Before: E1 won at tick 304. After: the E1 that entered bunkers #258 and #259
  in *capture* (from tick 89) are put out at tick 111, their house's check; at tick 112 *hunt*
  becomes current and gives them the bunker they left as target; they go for it in *capture* again,
  meet the CAOS gas outside, turn berserk and shoot one another. CAOS wins at tick 912.
- **M-0002 YURI–E1.** Before: E1 won. After: the occupiers are put out at tick 110, their house's
  check, and YURI wins at tick 912 holding 20 units.

Every step of these chains is a claim as built: S-0217 claims 1, 2 and 4; S-0508 claims 1, 4, 11
and 13; S-0215 claims 7, 11, 23 and 27; S-0309 claim 25 part (e). We changed nothing toward the
records (PROVENANCE section 5).

## Questions

1. In the original's M-0001 CAOS–E1 and CAOS–E2, side B, and M-0002 YURI–E1 and YURI–E2, side B:
   do the E1 or E2 that occupy a neutral bunker come out at their house's checks (S-0217 claim 4),
   about every 106 to 112 ticks from tick 110 on? S-0217 scenario H1 records this for the pairings
   Q-2951 named; these four pairings are where our outcome turns on it.
2. If they do come out: what keeps the original's E1 or E2 winning? For example, do they re-enter
   at once, before the enemy reaches them, or does *hunt* give them the enemy and not the bunker as
   their first target?
3. If they do not come out: which condition of S-0217 claim 4 or claim 1 does the original's arena
   house not meet (a building it owns, a construction vehicle, the occupied bunker not counted as
   its own at the check)?

## Possible observation

S-0217 scenario H1 on M-0001's set-up, side B, for CAOS–E1, seed 0: the eviction ticks of each
bunker, and for each E1 its behaviour, target and position for 100 ticks after each eviction.

## Current workaround

None: the claims as built. The rows stay failing in the arena and are listed in
`docs/m8-claims/intake124.md`.
