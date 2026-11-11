---
id: Q-2951
title: Arena rows that S-0508 r19's occupier rules do not reproduce
status: open
related: [S-0508, S-0215, S-0309, M-0001, M-0002]
answered-by: [S-0217]
---

## Need

S-0508 r19 answered Q-2926 and Q-2927. As we read it, it confirms the chain step A6b built and
traced for the arena's test-only computer-controlled houses (S-0215 Terms):

- an infantry unit of an occupier type finds a neutral garrisonable building through S-0309
  claim 25 part (e), whatever its behaviour (S-0508 claim 1, r19);
- S-0215 claim 27 (or 23) then makes the building its destination and forces *capture*, which
  removes a current or queued *hunt* (S-0215 claim 9);
- units on their way hold no place (S-0508 claim 1, r19), so more of them go than the building can
  take, and those that arrive after it is full are refused (S-0508 claim 4);
- a refused unit of an occupier type gets no *hunt* back from capture's hunt rule (S-0215 claim 28,
  S-0508 claim 4 r19, "No hunt after a refusal (original only)"), nor from the fall-back to rest
  (S-0215 claim 14).

Intake 123 brought the code in line with r19 (`docs/m8-claims/intake123.md`). The changes reach
assaulter types that are not occupier types and capture's hunt rule for non-occupier actors; the
arena's E1 and E2 are occupier types, so no arena game changed: every game of all six records
plays as before, game for game. The rows below still disagree with the measurement records.
Under PROVENANCE section 5 the measurement wins, so we do not change anything toward them, and
ask instead which step the original takes differently.

Our counts over seeds 0 to 63 on the failing side (the records' outcome is 6 of 6 on that side; a
row is within noise when our share *p* gives *p*⁶ ≥ 0.01):

| Record | Pairing, side | Record's outcome | Ours, 64 seeds | Verdict |
|---|---|---|---|---|
| M-0001 | TELE–E2, side B | col | none 64 | systematic |
| M-0002 | INIT–E1, side B | row | col 54, none 10, row 0 | systematic |
| M-0002 | YURI–E1, side B | col | row 46, col 16, none 2 (*p*⁶ < 0.001) | systematic |
| M-0001 | TELE–E1, side B | col | col 31, none 33 (*p*⁶ = 0.013) | within noise, at the edge |
| M-0002 | YURI–E2, side B | col | col 36, row 26, none 2 (*p*⁶ = 0.032) | within noise |
| M-0001 | YTNK–E1, side B | row | row 41, col 23 (*p*⁶ = 0.069) | within noise |

In each, our E1 or E2 take a neutral bunker as target early, go for it in *capture*, and those that
find it full are refused and stand in *area guard* near it for the rest of the game (A6b's watched
games: TELE–E2 and YURI–E1, seed 0). In INIT–E1 the E1 that get in win from inside. Before A6b built
claim 25 (e)'s occupation alternative and claims 23 and 27's infantry rule for these houses, the
six rows passed.

## Questions

1. In M-0001 TELE–E2, side B, and M-0002 YURI–E1, side B: do the stock E2 and E1 take a neutral
   bunker as target and go for it in *capture*, and are some refused at a full one? If they are,
   what gives a refused unit *hunt* again, or brings it to the other group, given S-0508 claim 4's
   "No hunt after a refusal"? This is S-0508's open question on what a refused unit does after the
   scatter request; scenario G21 records it, and G21 allows M-0001's set-up, side B, in place of its
   own map.
2. In M-0002 INIT–E1, side B: do the stock E1 enter the bunkers at all, and if so when, against the
   tick at which their first *hunt* processing would start?
3. If the stock units do not go for the bunkers in these games, which condition of S-0309 claim 25
   part (e), S-0215 claim 27 or S-0508 claim 1 keeps them out?

## Possible observation

S-0508 G21 on M-0001's set-up, side B, for TELE–E2 (and for M-0002 YURI–E1 and INIT–E1 if the
first does not settle it): per tick and per unit of the occupier group, the current, queued and
suspended behaviours, target, destination, position, entry or refusal tick, and the occupant count
of each bunker; one seed is enough to see which step differs.

## Current workaround

None: the claims as built (S-0508 r19 claims 1 and 4; S-0215 claims 9, 14, 23, 27 and 28; S-0309
claim 25 part (e)). The rows stay failing in the arena and are listed in
`docs/m8-claims/intake123.md` and `docs/m8-claims/a6b-arena-triage.md`. Wishlist entry A6b-1
(`docs/m8-claims/oracle-wishlist.md`) asks for the same recording.
